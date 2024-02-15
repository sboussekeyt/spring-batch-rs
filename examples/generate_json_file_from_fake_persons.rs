use anyhow::Result;
use std::env::temp_dir;

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepInstance},
    item::{
        fake::person_reader::{Person, PersonReaderBuilder},
        json::json_writer::JsonItemWriterBuilder,
    },
};

fn main() -> Result<()> {
    let reader = PersonReaderBuilder::new().number_of_items(100).build();

    let path = temp_dir().join("fake-persons.json");

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(false)
        .from_path(path);

    let step: StepInstance<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(10)
        .build();

    let _result = step.execute();

    Ok(())
}
