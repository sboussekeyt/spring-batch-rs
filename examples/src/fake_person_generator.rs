use spring_batch_rs::{
    core::step::{Step, StepBuilder},
    error::BatchError,
    item::{logger::LoggerWriter, fake::person_reader::{PersonReaderBuilder, Person}},
};

use log::info;

fn main() -> Result<(), BatchError> {
    env_logger::init();

    info!("Starting fake person generation");

    let mut reader = PersonReaderBuilder::new()
        .number_of_items(100)
        .build();

    let mut writer = LoggerWriter::new();

    let mut step: Step<Person, Person> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(20)
        .build();

    step.execute();

    info!("Finishing fake person generation");
    Ok(())
}
