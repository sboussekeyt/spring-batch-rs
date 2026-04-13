//! # Example: Filter Records from CSV with Processor
//!
//! Demonstrates how to filter items in a batch pipeline using a processor
//! that returns `Ok(None)` to silently discard items.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example filter_records_from_csv_with_processor --features csv,json
//! ```
//!
//! ## What It Does
//!
//! 1. Reads a list of persons (name, age) from an inline CSV string
//! 2. Filters out persons under 18 years old using a processor
//! 3. Writes the remaining adults to a JSON file in the temp directory
//! 4. Prints execution statistics including the filter count

use std::env::temp_dir;

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    BatchError,
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::{csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder},
};

/// A person record read from the CSV source.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    name: String,
    age: u32,
}

/// A processor that filters out persons under 18 years old.
///
/// Returns `Ok(None)` for minors, which causes the step to skip them
/// and increment `StepExecution::filter_count`.
#[derive(Default)]
struct AdultFilter;

impl ItemProcessor<Person, Person> for AdultFilter {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        if item.age >= 18 {
            Ok(Some(item.clone())) // keep adults
        } else {
            Ok(None) // filter out minors
        }
    }
}

const CSV_DATA: &str = "name,age\nAlice,30\nBob,16\nCharlie,25\nDiana,15\nEve,42\nFrank,17\n";

fn main() -> Result<(), BatchError> {
    // 1. Build reader from inline CSV string
    let reader = CsvItemReaderBuilder::<Person>::new()
        .has_headers(true)
        .from_reader(CSV_DATA.as_bytes());

    // 2. Build JSON writer to a temp file
    let output_path = temp_dir().join("adults.json");
    let writer = JsonItemWriterBuilder::<Person>::new().from_path(&output_path);

    // 3. Build the filter processor
    let processor = AdultFilter::default();

    // 4. Build step with processor
    let step = StepBuilder::new("filter-adults")
        .chunk::<Person, Person>(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // 5. Build and run job
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    // 6. Print execution statistics
    println!("Job status: Completed");
    println!("  Duration: {:?}", result.duration);

    // Retrieve per-step statistics
    let step_exec = job.get_step_execution("filter-adults").unwrap(); // unwrap: step name is known to exist
    println!("Step: {}", step_exec.name);
    println!("  Read:     {}", step_exec.read_count);
    println!("  Filtered: {}", step_exec.filter_count);
    println!("  Written:  {}", step_exec.write_count);
    println!("Output written to: {}", output_path.display());

    Ok(())
}
