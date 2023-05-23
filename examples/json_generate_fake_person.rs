use std::{env::temp_dir, fs::File};

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

    let mut reader = PersonReaderBuilder::new().number_of_items(3).build();

    info!("{}", temp_dir().display());

    let path = temp_dir().join("example-fake-person.json");

    let file = File::create(path);

    let mut writer = JsonItemWriterBuilder::new()
        .file(file.unwrap())
        .indent(b"  ")
        .pretty_formatter(true)
        .build();

    let mut step: Step<Person, Person> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(10)
        .build();

    let result: StepResult = step.execute();

    info!("Time elapsed is: {:?}", result.duration);

    info!("Finishing fake person generation");
    Ok(())
}
