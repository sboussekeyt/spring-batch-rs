use anyhow::Result;
use std::env::temp_dir;

use spring_batch_rs::{
    core::{
        item::PassThroughProcessor,
        job::{Job, JobBuilder},
        step::StepBuilder,
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

    let processor = PassThroughProcessor::<Person>::new();

    let step = StepBuilder::new("test")
        .chunk::<Person, Person>(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let _result = job.run();

    Ok(())
}
