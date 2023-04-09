use std::{fs::File, path::Path};

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepResult},
    item::json::json_reader::JsonItemReaderBuilder,
    item::{fake::person_reader::Person, logger::LoggerWriter},
};

use log::info;

fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Start batch processing");

    let path = Path::new("examples/data/persons.json");

    let file = File::options()
        .append(true)
        .read(true)
        .create(false)
        .open(path)
        .expect("Unable to open file");

    let reader = JsonItemReaderBuilder::new().from_reader(file);

    let writer = LoggerWriter::new();

    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(4)
        .build();

    let result: StepResult = step.execute();

    info!("Time elapsed is: {:?}", result.duration);

    info!("Finishing generation");
    Ok(())
}
