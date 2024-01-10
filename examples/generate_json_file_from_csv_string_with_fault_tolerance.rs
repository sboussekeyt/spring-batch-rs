use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepStatus},
    error::BatchError,
    item::csv::csv_reader::CsvItemReaderBuilder,
    JsonItemWriterBuilder,
};
use std::env::temp_dir;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Car {
    year: u16,
    make: String,
    model: String,
    description: String,
}

fn main() -> Result<(), BatchError> {
    let csv = "year,make,model,description
   1948,Porsche,356,Luxury sports car
   1995,Peugeot,205,City car
   bad_year,Mazda,CX-30,SUV Compact
   1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(csv.as_bytes());

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join("cars.json"));

    let step: Step<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(2)
        .skip_limit(1) // set fault tolerance to 1: only one error is allowed
        .build();

    let result = step.execute();

    assert_eq!(1, result.read_error_count); // The year of the 4th line is not valid
    assert!(StepStatus::SUCCESS == result.status); // Step is successful despite of the previous error

    Ok(())
}
