use std::env::temp_dir;

use spring_batch_rs::{
    core::step::{Step, StepBuilder},
    error::BatchError,
    item::{
        fake::person_reader::{Person, PersonReaderBuilder},
        json::json_writer::JsonItemWriterBuilder,
    },
};

fn main() -> Result<(), BatchError> {
    let reader = PersonReaderBuilder::new().number_of_items(100).build();

    let path = temp_dir().join("example-fake-person.json");

    let writer = JsonItemWriterBuilder::new()
        .indent(b"  ")
        .pretty_formatter(true)
        .from_path(path);

    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(10)
        .build();

    step.execute();

    Ok(())
}
