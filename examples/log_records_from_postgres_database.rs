use anyhow::Result;
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepExecution},
    item::logger::LoggerWriter,
    item::rdbc::rdbc_reader::RdbcItemReaderBuilder,
};
use sqlx::{AnyPool, FromRow};
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
struct Person {
    id: i32,
    first_name: String,
    last_name: String,
    birth_date: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    // Prepare database
    sqlx::any::install_default_drivers();
    let port = 5432;
    let connection_uri = format!("postgres://postgres:postgres@localhost:{}/test", port);
    let pool = AnyPool::connect(&connection_uri).await?;

    // Prepare reader
    let reader = RdbcItemReaderBuilder::<Person>::new()
        .query("SELECT id, first_name, last_name, birth_date, email FROM persons")
        .pool(&pool)
        .page_size(5)
        .build();

    // Prepare writer
    let writer = LoggerWriter;

    // Execute process
    let step = StepBuilder::new("log_postgres_records")
        .chunk::<Person, Person>(3)
        .reader(&reader)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new("log_postgres_records");
    let _result = step.execute(&mut step_execution);

    Ok(())
}
