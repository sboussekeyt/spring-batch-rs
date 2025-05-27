use anyhow::Result;
use std::env::temp_dir;

use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::{
        csv::csv_writer::CsvItemWriterBuilder,
        fake::person_reader::{Person, PersonReaderBuilder},
    },
};

#[derive(Default)]
struct PassThroughProcessor;

impl ItemProcessor<Person, Person> for PassThroughProcessor {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        Ok(item.clone())
    }
}

fn main() -> Result<()> {
    let reader = PersonReaderBuilder::new().number_of_items(10).build();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(false)
        .from_path(temp_dir().join("fake-persons.csv"));

    let processor = PassThroughProcessor::default();

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
