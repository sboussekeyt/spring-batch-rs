mod helpers;

use std::{io::Read, path::Path};

use anyhow::Error;
use helpers::{
    common::{DEFAULT_CHUNK_SIZE, EXPECTED_PERSON_COUNT, EXPECTED_PERSON_CSV, SAMPLE_CARS_CSV},
    postgres_helpers::{Car, PostgresCarItemBinder, CREATE_CARS_TABLE_SQL, SELECT_ALL_CARS_SQL},
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemReader, PassThroughProcessor},
        job::{Job, JobBuilder},
        step::{StepBuilder, StepStatus},
    },
    item::{
        csv::{csv_reader::CsvItemReaderBuilder, csv_writer::CsvItemWriterBuilder},
        rdbc::{RdbcItemReaderBuilder, RdbcItemWriterBuilder},
    },
};
use sqlx::{migrate::Migrator, FromRow, PgPool};
use tempfile::NamedTempFile;
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

#[derive(Serialize, Deserialize, Clone, FromRow)]
struct Person {
    id: Option<i64>,
    first_name: String,
    last_name: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn read_items_from_database() -> Result<(), Error> {
    // Prepare container
    let container = postgres::Postgres::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(5432).await?;

    // Prepare database
    let connection_uri = format!("postgres://postgres:postgres@{}:{}", host_ip, host_port);
    let pool = PgPool::connect(&connection_uri).await?;
    let migrator = Migrator::new(Path::new("tests/migrations/postgres")).await?;
    migrator.run(&pool).await?;

    // Prepare reader using unified builder
    let query = "SELECT * from person";
    let reader = RdbcItemReaderBuilder::new()
        .postgres(pool.clone())
        .query(query)
        .build_postgres();

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
    let container = postgres::Postgres::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(5432).await?;

    // Prepare reader
    let reader = CsvItemReaderBuilder::<Car>::new()
        .has_headers(true)
        .from_reader(SAMPLE_CARS_CSV.as_bytes());

    // Prepare writer
    let connection_uri = format!("postgres://postgres:postgres@{}:{}", host_ip, host_port);
    let pool = PgPool::connect(&connection_uri).await?;

    // Create table
    sqlx::query(CREATE_CARS_TABLE_SQL).execute(&pool).await?;

    let item_binder = PostgresCarItemBinder;

    let writer = RdbcItemWriterBuilder::<Car>::new()
        .postgres(&pool)
        .table("cars")
        .add_column("year")
        .add_column("make")
        .add_column("model")
        .add_column("description")
        .postgres_binder(&item_binder)
        .build_postgres();

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

// --- PostgresRdbcItemReader-specific tests (migrated from src/) ---

use spring_batch_rs::item::rdbc::PostgresRdbcItemReader;

#[derive(Debug, Clone, PartialEq, FromRow)]
struct TestUser {
    id: i32,
    name: String,
    email: String,
}

async fn setup_reader_test_db() -> Result<
    (
        PgPool,
        testcontainers_modules::testcontainers::ContainerAsync<postgres::Postgres>,
    ),
    Box<dyn std::error::Error>,
> {
    let container = postgres::Postgres::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    let pool = PgPool::connect(&format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host_ip, host_port
    ))
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    for i in 1..=10 {
        sqlx::query("INSERT INTO test_users (name, email) VALUES ($1, $2)")
            .bind(format!("User{}", i))
            .bind(format!("user{}@test.com", i))
            .execute(&pool)
            .await?;
    }

    Ok((pool, container))
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_build_with_page_size() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader = RdbcItemReaderBuilder::<TestUser>::new()
        .postgres(pool)
        .query("SELECT * FROM test_users")
        .with_page_size(5)
        .build_postgres();

    // Behavioral check: page_size=5 reader can read items
    let first = reader.read()?;
    assert!(first.is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_build_without_page_size() -> Result<(), Box<dyn std::error::Error>>
{
    let (pool, _container) = setup_reader_test_db().await?;

    let reader = RdbcItemReaderBuilder::<TestUser>::new()
        .postgres(pool)
        .query("SELECT * FROM test_users")
        .build_postgres();

    // Behavioral check: reader without page_size can read items
    let first = reader.read()?;
    assert!(first.is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_build_using_builder_method(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader = RdbcItemReaderBuilder::<TestUser>::new()
        .postgres(pool)
        .query("SELECT * FROM test_users")
        .with_page_size(3)
        .build_postgres();

    let mut count = 0;
    while reader.read()?.is_some() {
        count += 1;
    }
    assert_eq!(count, 10);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_read_all_items_without_pagination(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", None);

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 10);
    assert_eq!(items[0].id, 1);
    assert_eq!(items[0].name, "User1");
    assert_eq!(items[9].id, 10);
    assert_eq!(items[9].name, "User10");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_read_items_with_pagination(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", Some(3));

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 10);
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.id, (i + 1) as i32);
        assert_eq!(item.name, format!("User{}", i + 1));
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_empty_result_set() -> Result<(), Box<dyn std::error::Error>>
{
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users WHERE id > 1000", Some(5));

    let result = reader.read()?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_single_page_result() -> Result<(), Box<dyn std::error::Error>>
{
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
        pool,
        "SELECT * FROM test_users WHERE id <= 2 ORDER BY id",
        Some(5),
    );

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, 1);
    assert_eq!(items[1].id, 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_page_size_larger_than_result_set(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
        pool,
        "SELECT * FROM test_users WHERE id <= 3 ORDER BY id",
        Some(10),
    );

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 3);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_page_size_of_one() -> Result<(), Box<dyn std::error::Error>>
{
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
        pool,
        "SELECT * FROM test_users WHERE id <= 3 ORDER BY id",
        Some(1),
    );

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 3);
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.id, (i + 1) as i32);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_complex_query_with_where_clause(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
        pool,
        "SELECT * FROM test_users WHERE id % 2 = 0 ORDER BY id",
        Some(2),
    );

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 5); // Even IDs: 2, 4, 6, 8, 10
    assert_eq!(items[0].id, 2);
    assert_eq!(items[1].id, 4);
    assert_eq!(items[4].id, 10);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_maintain_correct_read_order(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
        pool,
        "SELECT * FROM test_users WHERE id <= 5 ORDER BY id",
        Some(2),
    );

    let item1: TestUser = reader.read()?.unwrap();
    assert_eq!(item1.id, 1);

    let item2: TestUser = reader.read()?.unwrap();
    assert_eq!(item2.id, 2);

    // Third item triggers second page load
    let item3: TestUser = reader.read()?.unwrap();
    assert_eq!(item3.id, 3);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_sequential_reads_correctly(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    let reader: PostgresRdbcItemReader<TestUser> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", Some(3));

    let mut all_items = Vec::new();
    for _ in 0..10 {
        if let Some(item) = reader.read()? {
            all_items.push(item);
        } else {
            break;
        }
    }

    assert_eq!(all_items.len(), 10);
    for (i, item) in all_items.iter().enumerate() {
        assert_eq!(item.id, (i + 1) as i32);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_work_with_different_data_types(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_data (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255),
            age INTEGER,
            active BOOLEAN,
            score FLOAT8
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO test_data (name, age, active, score) VALUES
        ('Alice', 25, true, 95.5::FLOAT8),
        ('Bob', 30, false, 87.2::FLOAT8)",
    )
    .execute(&pool)
    .await?;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        id: i32,
        name: String,
        age: i32,
        active: bool,
        score: f64,
    }

    impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for TestData {
        fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
            use sqlx::Row;
            Ok(TestData {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                age: row.try_get("age")?,
                active: row.try_get("active")?,
                score: row.try_get::<f64, _>("score")?,
            })
        }
    }

    let reader: PostgresRdbcItemReader<TestData> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_data ORDER BY id", Some(1));

    let mut items = Vec::new();
    while let Some(item) = reader.read()? {
        items.push(item);
    }

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "Alice");
    assert_eq!(items[0].age, 25);
    assert!(items[0].active);
    assert_eq!(items[1].name, "Bob");
    assert!(!items[1].active);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn postgres_reader_should_handle_large_result_sets_efficiently(
) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _container) = setup_reader_test_db().await?;

    for i in 11..=100 {
        sqlx::query("INSERT INTO test_users (name, email) VALUES ($1, $2)")
            .bind(format!("User{}", i))
            .bind(format!("user{}@test.com", i))
            .execute(&pool)
            .await?;
    }

    let reader: PostgresRdbcItemReader<TestUser> =
        PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", Some(10));

    let mut count = 0;
    while let Some(_item) = reader.read()? {
        count += 1;
    }

    assert_eq!(count, 100);

    Ok(())
}
