use std::{io::Read, path::Path, time::Instant};

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepResult, StepStatus},
    item::rdbc::rdbc_reader::{RdbcItemReaderBuilder, RowMapper},
    CsvItemWriterBuilder,
};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    AnyPool, Row, Sqlite,
};
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Clone)]
struct Person {
    id: Option<i32>,
    first_name: String,
    last_name: String,
}

#[derive(Default)]
pub struct PersonRowMapper {}

impl RowMapper<Person> for PersonRowMapper {
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
async fn test() -> Result<(), sqlx::Error> {
    // Prepare database
    let database_file = NamedTempFile::new()?;
    let database_path = database_file.path().to_str().unwrap();
    let connection_uri = format!("sqlite://{}", database_path);

    if !Sqlite::database_exists(&connection_uri)
        .await
        .unwrap_or(false)
    {
        match Sqlite::create_database(&connection_uri).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }

    sqlx::any::install_default_drivers();
    let pool = AnyPool::connect(&connection_uri).await?;
    let migrator = Migrator::new(Path::new("tests/migrations/sqlite")).await?;
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
    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let result: StepResult = step.execute();

    assert!(result.duration.as_nanos() > 0);
    assert!(result.start.le(&Instant::now()));
    assert!(result.end.le(&Instant::now()));
    assert!(result.start.le(&result.end));
    assert!(result.status == StepStatus::SUCCESS);
    assert!(result.read_count == 18);
    assert!(result.write_count == 18);
    assert!(result.read_error_count == 0);
    assert!(result.write_error_count == 0);

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
