use std::env::temp_dir;

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepResult},
    error::BatchError,
    item::{
        csv::csv_writer::CsvItemWriterBuilder,
        fake::person_reader::{Person, PersonReaderBuilder},
    },
};

use log::info;

fn main() -> Result<(), BatchError> {
    env_logger::init();

    info!("Starting fake person generation");

    let mut reader = PersonReaderBuilder::new().number_of_items(1000000).build();

    let mut writer = CsvItemWriterBuilder::new()
        .has_headers(false)
        .from_path(temp_dir().join("example-fake-person.csv"));

    let mut step: Step<Person, Person> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(1000)
        .build();

    let result: StepResult = step.execute();

    info!("Time elapsed is: {:?}", result.duration);

    info!("Finishing fake person generation");
    Ok(())
}
