use std::{io::Read, path::Path};

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::{Step, StepBuilder, StepInstance, StepStatus},
    },
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::csv::csv_writer::CsvItemWriterBuilder,
    item::rdbc::{
        rdbc_reader::{RdbcItemReaderBuilder, RdbcRowMapper},
        rdbc_writer::{RdbcItemBinder, RdbcItemWriterBuilder},
    },
};
use sqlx::{migrate::Migrator, query_builder::Separated, Any, AnyPool, FromRow, Row};
use tempfile::NamedTempFile;
use testcontainers_modules::{
    mysql::Mysql,
    testcontainers::{
        clients::Cli,
        core::{Port, RunnableImage},
    },
};

#[derive(Serialize, Deserialize, Clone)]
struct Person {
    id: Option<i32>,
    first_name: String,
    last_name: String,
}

#[derive(Default)]
pub struct PersonRowMapper {}

impl RdbcRowMapper<Person> for PersonRowMapper {
    fn map_row(&self, row: &sqlx::any::AnyRow) -> Person {
        let id: i32 = row.get("id");
        let first_name: String = row.get("first_name");
        let last_name: String = row.get("last_name");

        Person {
            id: Some(id),
            first_name,
            last_name,
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn read_items_from_database() -> Result<(), sqlx::Error> {
    // Prepare container
    let docker = Cli::default();
    let local_port = 33061;
    let port = Port {
        local: local_port,
        internal: 3306,
    };
    let mysql_image = RunnableImage::from(Mysql::default())
        .with_tag("latest")
        .with_mapped_port(port);
    let _node = docker.run(mysql_image);

    // Prepare database
    sqlx::any::install_default_drivers();
    let connection_uri = format!("mysql://localhost:{}/test", local_port);
    let pool = AnyPool::connect(&connection_uri).await?;
    let migrator = Migrator::new(Path::new("tests/migrations/mysql")).await?;
    migrator.run(&pool).await?;

    // Prepare reader
    let query = "SELECT * from person";
    let row_mapper = PersonRowMapper::default();
    let reader = RdbcItemReaderBuilder::new()
        .pool(&pool)
        .query(&query)
        .row_mapper(&row_mapper)
        .page_size(5)
        .build();

    // Prepare writer
    let tmpfile = NamedTempFile::new()?;

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_writer(tmpfile.as_file());

    // Execute process
    let step: StepInstance<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();
    assert!(result.is_ok());
    assert!(step.get_status() == StepStatus::Success);
    assert!(step.get_read_count() == 18);
    assert!(step.get_write_count() == 18);
    assert!(step.get_read_error_count() == 0);
    assert!(step.get_write_error_count() == 0);

    let mut tmpfile = tmpfile.reopen()?;
    let mut file_content = String::new();

    tmpfile
        .read_to_string(&mut file_content)
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        "id,first_name,last_name
1,Melton,Finnegan
2,Pruitt,Brayan
3,Simmons,Kaitlyn
4,Dougherty,Kristen
5,Patton,Gina
6,Michael,Emiliano
7,Singh,Zion
8,Morales,Kaydence
9,Hull,Randy
10,Crosby,Daphne
11,Gates,Christopher
12,Colon,Melina
13,Alvarado,Nathan
14,Blackwell,Mareli
15,Lara,Kian
16,Montes,Cory
17,Larson,Iyana
18,Gentry,Sasha
"
    );

    Ok(())
}

struct CarItemBinder {}

impl RdbcItemBinder<Car> for CarItemBinder {
    fn bind(&self, item: &Car, mut query_builder: Separated<Any, &str>) {
        query_builder.push_bind(item.year);
        query_builder.push_bind(item.make.clone());
        query_builder.push_bind(item.model.clone());
        query_builder.push_bind(item.description.clone());
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
struct Car {
    year: i16,
    make: String,
    model: String,
    description: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn write_items_to_database() -> Result<(), sqlx::Error> {
    // Prepare container
    let docker = Cli::default();
    let local_port = 33062;
    let port = Port {
        local: local_port,
        internal: 3306,
    };
    let mysql_image = RunnableImage::from(Mysql::default())
        .with_tag("latest")
        .with_mapped_port(port);
    let _node = docker.run(mysql_image);

    // Prepare reader
    let csv = "year,make,model,description
            1948,Porsche,356,Luxury sports car
            2011,Peugeot,206+,City car
            2012,Citroën,C4 Picasso,SUV
            2021,Mazda,CX-30,SUV Compact
            1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // Prepare writer
    sqlx::any::install_default_drivers();
    let connection_uri = format!("mysql://localhost:{}/test", local_port);
    let pool = AnyPool::connect(&connection_uri).await?;

    // Create table
    let create_query = sqlx::query("CREATE TABLE IF NOT EXISTS cars (year INTEGER NOT NULL, make VARCHAR(25) NOT NULL, model VARCHAR(25) NOT NULL, description VARCHAR(25) NOT NULL);");
    create_query.execute(&pool).await?;

    let item_binder = CarItemBinder {};

    let writer = RdbcItemWriterBuilder::new()
        .table("cars")
        .add_column("year")
        .add_column("make")
        .add_column("model")
        .add_column("description")
        .pool(&pool)
        .item_binder(&item_binder)
        .build();

    // Execute process
    let step: StepInstance<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();
    assert!(result.is_ok());
    assert!(step.get_status() == StepStatus::Success);
    assert!(step.get_read_count() == 5);
    assert!(step.get_write_count() == 5);
    assert!(step.get_read_error_count() == 0);
    assert!(step.get_write_error_count() == 0);

    let car_results = sqlx::query_as::<_, Car>("SELECT year, make, model, description FROM cars")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!car_results.is_empty());
    Ok(())
}
