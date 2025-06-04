use anyhow::Result;
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::PassThroughProcessor,
        job::{Job, JobBuilder},
        step::{StepBuilder, StepStatus},
    },
    item::{csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder},
};
use std::env::temp_dir;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Car {
    year: u16,
    make: String,
    model: String,
    description: String,
}

fn main() -> Result<()> {
    let csv = "year,make,model,description
   1948,Porsche,356,Luxury sports car
   1995,Peugeot,205,City car
   bad_year,Mazda,CX-30,SUV Compact
   1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::<Car>::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(csv.as_bytes());

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join("cars.json"));

    let processor = PassThroughProcessor::<Car>::new();

    let step = StepBuilder::new("test")
        .chunk::<Car, Car>(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .skip_limit(1) // set fault tolerance to 1: only one error is allowed
        .build();

    let job = JobBuilder::new().start(&step).build();
    let _result = job.run();

    let step_execution = job.get_step_execution("test").unwrap();
    assert_eq!(1, step_execution.read_error_count); // The year of the 4th line is not valid
    assert!(step_execution.status == StepStatus::Success); // Step is successful despite of the previous error

    Ok(())
}
