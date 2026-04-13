//! # Example: Chaining Item Processors
//!
//! Demonstrates how to chain multiple [`ItemProcessor`]s into a single pipeline
//! using [`CompositeItemProcessorBuilder`]. Each processor in the chain receives
//! the output of the previous one.
//!
//! This example models an order-processing ETL:
//!
//! 1. **`ParseProcessor`** — converts raw string fields into typed values
//! 2. **`ValidateProcessor`** — filters out orders below a minimum amount
//! 3. **`EnrichProcessor`** — adds tax and computes the final total
//!
//! ## Run
//!
//! ```bash
//! cargo run --example chaining_processors --features csv,json
//! ```
//!
//! ## What It Does
//!
//! 1. Reads raw CSV orders (all fields as strings)
//! 2. Parses each row into typed fields (`RawOrder → ParsedOrder`)
//! 3. Filters out orders below €10.00 (`ParsedOrder → ParsedOrder`)
//! 4. Enriches each order with tax and total (`ParsedOrder → EnrichedOrder`)
//! 5. Writes the enriched orders to a JSON file

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{CompositeItemProcessorBuilder, ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::{csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder},
};
use std::env::temp_dir;

// =============================================================================
// Data Structures
// =============================================================================

/// Raw order record read directly from CSV — all fields are strings.
#[derive(Debug, Deserialize)]
struct RawOrder {
    id: String,
    customer: String,
    amount: String,
}

/// Order with parsed, typed fields.
#[derive(Debug, Clone)]
struct ParsedOrder {
    id: u32,
    customer: String,
    amount: f64,
}

/// Enriched order with tax and total computed, ready for output.
#[derive(Debug, Serialize)]
struct EnrichedOrder {
    id: u32,
    customer: String,
    amount: f64,
    tax: f64,
    total: f64,
}

// =============================================================================
// Processors
// =============================================================================

/// Parses raw string fields into typed values.
///
/// Returns `Ok(None)` if any field cannot be parsed, silently filtering the row.
struct ParseProcessor;

impl ItemProcessor<RawOrder, ParsedOrder> for ParseProcessor {
    fn process(&self, item: &RawOrder) -> ItemProcessorResult<ParsedOrder> {
        let id = match item.id.trim().parse::<u32>() {
            Ok(v) => v,
            Err(_) => return Ok(None), // unparseable id — filter silently
        };
        let amount = match item.amount.trim().parse::<f64>() {
            Ok(v) => v,
            Err(_) => return Ok(None), // unparseable amount — filter silently
        };
        Ok(Some(ParsedOrder {
            id,
            customer: item.customer.trim().to_string(),
            amount,
        }))
    }
}

/// Filters out orders below the minimum amount threshold.
struct ValidateProcessor {
    min_amount: f64,
}

impl ItemProcessor<ParsedOrder, ParsedOrder> for ValidateProcessor {
    fn process(&self, item: &ParsedOrder) -> ItemProcessorResult<ParsedOrder> {
        if item.amount < self.min_amount {
            Ok(None) // below threshold — discard
        } else {
            Ok(Some(item.clone()))
        }
    }
}

/// Adds 20% tax and computes the total for each order.
struct EnrichProcessor;

impl ItemProcessor<ParsedOrder, EnrichedOrder> for EnrichProcessor {
    fn process(&self, item: &ParsedOrder) -> ItemProcessorResult<EnrichedOrder> {
        let tax = (item.amount * 0.20 * 100.0).round() / 100.0;
        let total = ((item.amount + tax) * 100.0).round() / 100.0;
        Ok(Some(EnrichedOrder {
            id: item.id,
            customer: item.customer.clone(),
            amount: item.amount,
            tax,
            total,
        }))
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let csv = "\
id,customer,amount
1,Alice,50.00
2,Bob,8.00
3,Charlie,120.50
4,Diana,bad_amount
5,Eve,200.00";

    // 1. Build reader
    let reader = CsvItemReaderBuilder::<RawOrder>::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // 2. Build writer
    let output = temp_dir().join("enriched_orders.json");
    let writer = JsonItemWriterBuilder::<EnrichedOrder>::new().from_path(&output);

    // 3. Build composite processor: RawOrder → ParsedOrder → ParsedOrder → EnrichedOrder
    let composite = CompositeItemProcessorBuilder::new(ParseProcessor)
        .link(ValidateProcessor { min_amount: 10.0 })
        .link(EnrichProcessor)
        .build();

    // 4. Build step
    let step = StepBuilder::new("enrich-orders")
        .chunk::<RawOrder, EnrichedOrder>(10)
        .reader(&reader)
        .processor(&composite)
        .writer(&writer)
        .build();

    // 5. Run job
    let job = JobBuilder::new().start(&step).build();
    job.run().expect("job failed");

    // 6. Report results
    let exec = job.get_step_execution("enrich-orders").unwrap();
    println!("Output: {}", output.display());
    println!("Read:      {}", exec.read_count); // 5
    println!("Processed: {}", exec.process_count); // 3 (Alice, Charlie, Eve)
    println!("Filtered:  {}", exec.filter_count); // 2 (Bob below threshold, Diana invalid)
    println!("Written:   {}", exec.write_count); // 3
}
