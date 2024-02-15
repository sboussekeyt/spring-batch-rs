use anyhow::Result;
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepInstance},
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::rdbc::rdbc_writer::{RdbcItemBinder, RdbcItemWriterBuilder},
};
use sqlx::{query_builder::Separated, Any, AnyPool, FromRow};

struct CarItemBinder;

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

#[tokio::main]
async fn main() -> Result<()> {
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
    let port = 3306;
    let connection_uri = format!("mysql://localhost:{}/test", port);
    let pool = AnyPool::connect(&connection_uri).await?;

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

    let _result = step.execute();

    Ok(())
}
