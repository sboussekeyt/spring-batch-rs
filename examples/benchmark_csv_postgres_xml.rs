//! # Example: Benchmark — CSV → PostgreSQL → XML (10 M transactions)
//!
//! Demonstrates a two-step ETL pipeline used to benchmark spring-batch-rs
//! against Spring Batch (Java) on a 10 million financial-transaction dataset.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example benchmark_csv_postgres_xml --features csv,xml,rdbc-postgres
//! ```
//!
//! ## What It Does
//!
//! 1. Step 1 — reads transactions from a generated CSV file, applies currency
//!    conversion, and bulk-inserts rows into PostgreSQL.
//! 2. Step 2 — reads enriched rows from PostgreSQL and streams them to an XML
//!    file.
//! 3. Prints wall-clock time, rows/s, and peak RSS for comparison with Java.

fn main() {
    println!("Benchmark example — implementation pending.");
}
