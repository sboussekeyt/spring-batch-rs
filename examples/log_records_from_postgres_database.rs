use std::env;

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder},
    item::rdbc::rdbc_reader::{RdbcItemReaderBuilder, RdbcRowMapper},
    LoggerWriter,
};
use sqlx::{AnyPool, Row};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    // Prepare database
    sqlx::any::install_default_drivers();
    let port = 5432;
    let connection_uri = format!("postgres://postgres:postgres@localhost:{}", port);
    let pool = AnyPool::connect(&connection_uri).await?;

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
    let writer = LoggerWriter::new();

    // Execute step
    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    step.execute();

    Ok(())
}
