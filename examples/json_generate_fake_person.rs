use std::env::temp_dir;

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepResult},
    error::BatchError,
    item::{
        fake::person_reader::{Person, PersonReaderBuilder},
        json::json_writer::JsonItemWriterBuilder,
    },
};

use log::info;

fn main() -> Result<(), BatchError> {
    env_logger::init();

    info!("Starting fake person generation");

    let reader = PersonReaderBuilder::new().number_of_items(100).build();

    let binding = temp_dir().join("example-fake-person.json");
    let path = binding.to_str();

    let writer = JsonItemWriterBuilder::new()
        .path(path)
        .indent(b"  ")
        .pretty_formatter(true)
        .build();

    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(10)
        .build();

    let result: StepResult = step.execute();

    info!("Time elapsed is: {:?}", result.duration);

    info!("Finishing fake person generation");
    Ok(())
}
