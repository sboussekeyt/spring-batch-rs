use std::fmt;

use serde::{Deserialize, Serialize};

use spring_batch_rs::{
    core::step::{Step, StepBuilder},
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::logger::LoggerWriter,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
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

fn main() -> std::io::Result<()> {
    let csv = "year,make,model,description
    1948,Porsche,356,Luxury sports car
    1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(csv.as_bytes());

    let writer = LoggerWriter::new();

    let step: Step<Record, Record> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(4)
        .build();

    step.execute();

    Ok(())
}
