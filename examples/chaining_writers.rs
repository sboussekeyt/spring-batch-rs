//! # Example: Chaining Item Writers (Fan-out)
//!
//! Demonstrates how to write the same chunk of items to multiple destinations
//! simultaneously using [`CompositeItemWriterBuilder`].
//!
//! Each writer in the chain receives an identical slice of items. Writers are
//! called in order; if any writer fails the chain short-circuits.
//!
//! This example models a product ingestion pipeline that simultaneously:
//! 1. Logs each product to the console (audit trail)
//! 2. Writes all products to a JSON file (persistence)
//!
//! ## Run
//!
//! ```bash
//! cargo run --example chaining_writers --features csv,json,logger
//! ```
//!
//! ## What It Does
//!
//! 1. Reads product records from an inline CSV string
//! 2. Fans out each chunk to two writers:
//!    - `LoggerWriter` — logs every item via the `log` crate
//!    - `JsonItemWriter` — writes all items to a temp JSON file
//! 3. Prints the output path and item counts

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{CompositeItemWriterBuilder, PassThroughProcessor},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::{
        csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder,
        logger::LoggerWriterBuilder,
    },
    BatchError,
};
use std::env::temp_dir;

/// A product record read from CSV and written to both destinations.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

fn main() -> Result<(), BatchError> {
    let csv = "\
id,name,price
1,Widget,9.99
2,Gadget,24.50
3,Doohickey,4.75
4,Thingamajig,14.00
5,Whatsit,2.99";

    // 1. Build reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // 2. Build individual writers
    let output = temp_dir().join("products.json");
    let json_writer = JsonItemWriterBuilder::<Product>::new().from_path(&output);
    let logger_writer = LoggerWriterBuilder::<Product>::new().build();

    // 3. Build composite fan-out writer: same items go to logger AND json file
    let composite = CompositeItemWriterBuilder::new(logger_writer)
        .link(json_writer)
        .build();

    // 4. Pass-through processor — items are not transformed
    let processor = PassThroughProcessor::<Product>::new();

    // 5. Build step
    let step = StepBuilder::new("fan-out-products")
        .chunk::<Product, Product>(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&composite)
        .build();

    // 6. Run job
    let job = JobBuilder::new().start(&step).build();
    job.run()?;

    // 7. Report results
    let exec = job.get_step_execution("fan-out-products").unwrap(); // step name is always registered after successful job.run()
    println!("JSON output: {}", output.display());
    println!("Read:      {}", exec.read_count); // 5
    println!("Processed: {}", exec.process_count); // 5 (pass-through)
    println!("Written:   {}", exec.write_count); // 5 (to both writers)

    Ok(())
}
