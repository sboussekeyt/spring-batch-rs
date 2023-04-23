use std::fmt;

use serde::{Deserialize, Serialize};

use spring_batch_rs::{
    core::step::{Step, StepBuilder},
    item::csv::csv_reader::CsvItemReaderBuilder,
    error::BatchError,
    item::logger::LoggerWriter,
};

use log::info;

#[derive(Deserialize, Serialize, Debug)]
struct Record {
    year: u16,
    make: String,
    model: String,
    description: String,
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(year={}, make={}, model={}, description={})",
            self.year, self.make, self.model, self.description
        )
    }
}

fn main() -> Result<(), BatchError> {
    env_logger::init();

    info!("Start batch processing");

    let mut reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("/Users/20014378/Projects/Perso/rusty/batch/test.csv");

    let mut writer = LoggerWriter::new();

    let mut step: Step<Record, Record> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(4)
        .build();

    step.execute();

    info!("End batch processing");
    Ok(())
}
