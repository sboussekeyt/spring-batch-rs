use anyhow::Result;
use std::env::temp_dir;

use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepExecution},
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

    let step = StepBuilder::new("generate_fake_persons")
        .chunk::<Person, Person>(10)
        .reader(&reader)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new("generate_fake_persons");
    let _result = step.execute(&mut step_execution);

    Ok(())
}
