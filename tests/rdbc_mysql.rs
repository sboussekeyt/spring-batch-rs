mod helpers;

use std::{io::Read, path::Path};

use anyhow::Error;
use helpers::{
    common::{DEFAULT_CHUNK_SIZE, EXPECTED_PERSON_COUNT, EXPECTED_PERSON_CSV, SAMPLE_CARS_CSV},
    mysql_helpers::{Car, MySqlCarItemBinder, CREATE_CARS_TABLE_SQL, SELECT_ALL_CARS_SQL},
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::PassThroughProcessor,
        job::{Job, JobBuilder},
        step::{StepBuilder, StepStatus},
    },
    item::{
        csv::{csv_reader::CsvItemReaderBuilder, csv_writer::CsvItemWriterBuilder},
        rdbc::{RdbcItemReaderBuilder, RdbcItemWriterBuilder},
    },
};
use sqlx::{migrate::Migrator, FromRow, MySqlPool};
use tempfile::NamedTempFile;
use testcontainers_modules::{mysql, testcontainers::runners::AsyncRunner};

#[derive(Serialize, Deserialize, Clone, FromRow)]
struct Person {
    id: Option<i32>,
    first_name: String,
    last_name: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn read_items_from_database() -> Result<(), Error> {
    // Prepare container
    let container = mysql::Mysql::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(3306).await?;

    // Prepare database
    let connection_uri = format!("mysql://{}:{}/test", host_ip, host_port);
    let pool = MySqlPool::connect(&connection_uri).await?;
    let migrator = Migrator::new(Path::new("tests/migrations/mysql")).await?;
    migrator.run(&pool).await?;

    // Prepare reader using unified builder
    let query = "SELECT * from person";
    let reader = RdbcItemReaderBuilder::new()
        .mysql(pool.clone())
        .query(query)
        .with_page_size(5)
        .build_mysql();

    // Prepare writer
    let tmpfile = NamedTempFile::new()?;
    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_writer(tmpfile.as_file());

    let processor = PassThroughProcessor::new();

    // Execute process
    let step = StepBuilder::new("test")
        .chunk::<Person, Person>(DEFAULT_CHUNK_SIZE)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    assert!(result.is_ok());

    let step_execution = job.get_step_execution("test").unwrap();

    assert_eq!(step_execution.status, StepStatus::Success);
    assert_eq!(step_execution.read_count, EXPECTED_PERSON_COUNT);
    assert_eq!(step_execution.write_count, EXPECTED_PERSON_COUNT);
    assert_eq!(step_execution.read_error_count, 0);
    assert_eq!(step_execution.write_error_count, 0);

    let mut tmpfile = tmpfile.reopen()?;
    let mut file_content = String::new();

    tmpfile
        .read_to_string(&mut file_content)
        .expect("Should have been able to read the file");

    assert_eq!(file_content, EXPECTED_PERSON_CSV);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn write_items_to_database() -> Result<(), Error> {
    // Prepare container
    let container = mysql::Mysql::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(3306).await?;

    // Prepare reader
    let reader = CsvItemReaderBuilder::<Car>::new()
        .has_headers(true)
        .from_reader(SAMPLE_CARS_CSV.as_bytes());

    // Prepare writer
    let connection_uri = format!("mysql://{}:{}/test", host_ip, host_port);
    let pool = MySqlPool::connect(&connection_uri).await?;

    // Create table
    sqlx::query(CREATE_CARS_TABLE_SQL).execute(&pool).await?;

    let item_binder = MySqlCarItemBinder;

    let writer = RdbcItemWriterBuilder::<Car>::new()
        .mysql(&pool)
        .table("cars")
        .add_column("year")
        .add_column("make")
        .add_column("model")
        .add_column("description")
        .mysql_binder(&item_binder)
        .build_mysql();

    let processor = PassThroughProcessor::<Car>::new();

    // Execute process
    let step = StepBuilder::new("test")
        .chunk::<Car, Car>(DEFAULT_CHUNK_SIZE)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();

    let result = job.run();
    assert!(result.is_ok());

    let step_execution = job.get_step_execution("test").unwrap();

    assert_eq!(step_execution.status, StepStatus::Success);
    assert_eq!(
        step_execution.read_count,
        helpers::common::EXPECTED_CAR_COUNT
    );
    assert_eq!(
        step_execution.write_count,
        helpers::common::EXPECTED_CAR_COUNT
    );
    assert_eq!(step_execution.read_error_count, 0);
    assert_eq!(step_execution.write_error_count, 0);

    let car_results = sqlx::query_as::<_, Car>(SELECT_ALL_CARS_SQL)
        .fetch_all(&pool)
        .await?;

    assert_eq!(car_results.len(), helpers::common::EXPECTED_CAR_COUNT);

    Ok(())
}
