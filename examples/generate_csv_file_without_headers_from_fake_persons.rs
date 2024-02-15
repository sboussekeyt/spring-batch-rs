use anyhow::Result;
use std::env::temp_dir;

use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::{StepBuilder, StepInstance},
    },
    item::{
        csv::csv_writer::CsvItemWriterBuilder,
        fake::person_reader::{Person, PersonReaderBuilder},
    },
};

fn main() -> Result<()> {
    let reader = PersonReaderBuilder::new().number_of_items(10).build();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(false)
        .from_path(temp_dir().join("fake-persons.csv"));

    let step: StepInstance<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(10)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let _result = job.run();

    Ok(())
}
