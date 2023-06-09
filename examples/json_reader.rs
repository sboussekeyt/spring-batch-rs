use std::io;

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepResult},
    item::{logger::LoggerWriter, fake::person_reader::Person},
    item::{json::json_reader::JsonItemReaderBuilder},
};

use log::info;

fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Start batch processing");

    let mut reader = JsonItemReaderBuilder::new().from_reader(io::stdin());

    let mut writer = LoggerWriter::new();

    let mut step: Step<Person, Person> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(4)
        .build();

    let result: StepResult = step.execute();

    info!("Time elapsed is: {:?}", result.duration);

    info!("Finishing generation");
    Ok(())
}
