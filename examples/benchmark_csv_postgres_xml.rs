//! # Benchmark: CSV → PostgreSQL → XML (10 Million Financial Transactions)
//!
//! Production-grade benchmark comparing Spring Batch RS (Rust) against
//! Spring Batch (Java) on a realistic ETL pipeline.
//!
//! ## What It Does
//!
//! 1. Generates 10 million financial transaction records as CSV
//! 2. Step 1 — reads CSV, converts currencies to EUR, normalises status,
//!    bulk-inserts into PostgreSQL (chunk = 1 000)
//! 3. Step 2 — reads PostgreSQL, exports to XML (chunk = 1 000)
//! 4. Prints wall-clock time, rows/s, and peak RSS
//!
//! ## Run
//!
//! ```bash
//! # Start PostgreSQL first (Docker example):
//! # docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15
//!
//! cargo run --release --example benchmark_csv_postgres_xml \
//!   --features csv,xml,rdbc-postgres
//! ```
//!
//! Set DATABASE_URL env var to override the default connection string.

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, PassThroughProcessor},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::{
        csv::csv_reader::CsvItemReaderBuilder,
        rdbc::{DatabaseItemBinder, RdbcItemReaderBuilder, RdbcItemWriterBuilder},
        xml::xml_writer::XmlItemWriterBuilder,
    },
    BatchError,
};
use sqlx::{query_builder::Separated, FromRow, PgPool, Postgres};
use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter, Write},
    time::Instant,
};

// =============================================================================
// Data Model
// =============================================================================

/// A financial transaction read from CSV (amount_eur defaults to 0.0).
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
struct Transaction {
    transaction_id: String,
    amount: f64,
    currency: String,
    #[serde(rename = "timestamp")]
    timestamp: String,
    account_from: String,
    account_to: String,
    status: String,
    #[serde(default)]
    amount_eur: f64,
}

// =============================================================================
// Processor
// =============================================================================

/// Converts transaction amounts to EUR and normalises status values.
///
/// Conversion rates (fixed for benchmark reproducibility):
/// - USD → EUR: × 0.92
/// - GBP → EUR: × 1.17
/// - EUR → EUR: × 1.00
///
/// Status normalisation: "CANCELLED" is mapped to "FAILED".
#[derive(Default)]
struct TransactionProcessor;

impl ItemProcessor<Transaction, Transaction> for TransactionProcessor {
    fn process(&self, item: &Transaction) -> Result<Transaction, BatchError> {
        let rate = match item.currency.as_str() {
            "USD" => 0.92,
            "GBP" => 1.17,
            _ => 1.0,
        };
        let status = if item.status == "CANCELLED" {
            "FAILED".to_string()
        } else {
            item.status.clone()
        };
        Ok(Transaction {
            transaction_id: item.transaction_id.clone(),
            amount: item.amount,
            currency: item.currency.clone(),
            timestamp: item.timestamp.clone(),
            account_from: item.account_from.clone(),
            account_to: item.account_to.clone(),
            status,
            amount_eur: (item.amount * rate * 100.0).round() / 100.0,
        })
    }
}

// =============================================================================
// PostgreSQL Binder
// =============================================================================

/// Binds `Transaction` fields to a PostgreSQL bulk-insert query.
struct TransactionBinder;

impl DatabaseItemBinder<Transaction, Postgres> for TransactionBinder {
    fn bind(&self, item: &Transaction, mut q: Separated<Postgres, &str>) {
        q.push_bind(item.transaction_id.clone());
        q.push_bind(item.amount);
        q.push_bind(item.currency.clone());
        q.push_bind(item.timestamp.clone());
        q.push_bind(item.account_from.clone());
        q.push_bind(item.account_to.clone());
        q.push_bind(item.status.clone());
        q.push_bind(item.amount_eur);
    }
}

// =============================================================================
// Data Generator
// =============================================================================

const CURRENCIES: [&str; 3] = ["USD", "EUR", "GBP"];
const STATUSES: [&str; 4] = ["PENDING", "COMPLETED", "FAILED", "CANCELLED"];
const TOTAL_RECORDS: u64 = 10_000_000;

/// Generates a CSV file with `count` random financial transaction rows.
///
/// Uses a fast linear-congruential generator seeded per row to avoid
/// the overhead of a thread-local RNG for 10M records.
fn generate_csv(path: &str, count: u64) -> Result<(), BatchError> {
    let file = File::create(path)
        .map_err(|e| BatchError::ItemWriter(format!("Cannot create CSV: {}", e)))?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);

    // Write header
    writeln!(
        writer,
        "transaction_id,amount,currency,timestamp,account_from,account_to,status"
    )
    .map_err(|e| BatchError::ItemWriter(e.to_string()))?;

    // LCG constants (Knuth)
    let mut seed: u64 = 0xDEAD_BEEF_CAFE_BABE;

    for i in 0..count {
        // Advance seed twice per record for two independent values
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let r1 = (seed >> 33) as u32;
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let r2 = (seed >> 33) as u32;

        let currency = CURRENCIES[(r1 % 3) as usize];
        let status = STATUSES[(r2 % 4) as usize];
        // Amount between 1.00 and 99_999.99
        let amount = ((r1 % 9_999_999) + 100) as f64 / 100.0;
        let month = r1 % 12 + 1;
        let day = r2 % 28 + 1;
        let hour = r1 % 24;
        let min = r2 % 60;
        let sec = r1 % 60;
        let acc_from = r1 % 1_000_000;
        let acc_to = r2 % 1_000_000;

        writeln!(
            writer,
            "TXN-{:010},{:.2},{},2024-{:02}-{:02}T{:02}:{:02}:{:02}Z,\
             ACC-{:08},ACC-{:08},{}",
            i + 1,
            amount,
            currency,
            month,
            day,
            hour,
            min,
            sec,
            acc_from,
            acc_to,
            status
        )
        .map_err(|e| BatchError::ItemWriter(e.to_string()))?;
    }

    writer
        .flush()
        .map_err(|e| BatchError::ItemWriter(e.to_string()))?;

    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use spring_batch_rs::core::item::ItemProcessor;

    fn make_transaction(currency: &str, amount: f64, status: &str) -> Transaction {
        Transaction {
            transaction_id: "TXN-0000000001".to_string(),
            amount,
            currency: currency.to_string(),
            timestamp: "2024-06-15T12:00:00Z".to_string(),
            account_from: "ACC-00000001".to_string(),
            account_to: "ACC-00000002".to_string(),
            status: status.to_string(),
            amount_eur: 0.0,
        }
    }

    #[test]
    fn should_convert_usd_to_eur() {
        let processor = TransactionProcessor;
        let input = make_transaction("USD", 1000.0, "COMPLETED");
        let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
        assert_eq!(result.amount_eur, 920.0, "USD 1000 * 0.92 = EUR 920");
        assert_eq!(result.currency, "USD", "currency field must not change");
    }

    #[test]
    fn should_convert_gbp_to_eur() {
        let processor = TransactionProcessor;
        let input = make_transaction("GBP", 100.0, "COMPLETED");
        let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
        assert_eq!(result.amount_eur, 117.0, "GBP 100 * 1.17 = EUR 117");
    }

    #[test]
    fn should_keep_eur_unchanged() {
        let processor = TransactionProcessor;
        let input = make_transaction("EUR", 500.0, "PENDING");
        let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
        assert_eq!(result.amount_eur, 500.0, "EUR passthrough: rate = 1.0");
    }

    #[test]
    fn should_normalise_cancelled_to_failed() {
        let processor = TransactionProcessor;
        let input = make_transaction("EUR", 100.0, "CANCELLED");
        let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
        assert_eq!(result.status, "FAILED", "CANCELLED must be mapped to FAILED");
    }

    #[test]
    fn should_preserve_other_statuses() {
        let processor = TransactionProcessor;
        for status in &["PENDING", "COMPLETED", "FAILED"] {
            let input = make_transaction("EUR", 100.0, status);
            let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
            assert_eq!(&result.status, status, "status '{}' must not be changed", status);
        }
    }

    #[test]
    fn should_round_amount_eur_to_two_decimals() {
        let processor = TransactionProcessor;
        // 333.33 * 0.92 = 306.6636 → rounds to 306.66
        let input = make_transaction("USD", 333.33, "COMPLETED");
        let result = processor.process(&input).unwrap(); // unwrap: process() always returns Ok
        assert!((result.amount_eur - 306.66_f64).abs() < 1e-9,
            "amount_eur must be rounded to 2 decimals, got {}", result.amount_eur);
    }

    #[test]
    fn should_generate_csv_with_correct_header_and_row_count() {
        use std::io::Read;
        let path = std::env::temp_dir().join("bench_smoke_test.csv");
        generate_csv(path.to_str().unwrap(), 5).unwrap(); // unwrap: temp dir is always writable

        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap(); // unwrap: file was just created

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(
            lines[0],
            "transaction_id,amount,currency,timestamp,account_from,account_to,status",
            "CSV header mismatch"
        );
        assert_eq!(lines.len(), 6, "header + 5 data rows expected, got {}", lines.len());
    }
}

// =============================================================================
// Step 1 — CSV → PostgreSQL
// =============================================================================

fn run_step1(pool: &PgPool, csv_path: &str) -> Result<u64, BatchError> {
    log::info!("[Step 1] CSV → PostgreSQL …");
    let t0 = Instant::now();

    let file = File::open(csv_path)
        .map_err(|e| BatchError::ItemReader(format!("Cannot open CSV: {}", e)))?;
    let buffered = BufReader::with_capacity(64 * 1024, file);

    let reader = CsvItemReaderBuilder::<Transaction>::new()
        .has_headers(true)
        .from_reader(buffered);

    let binder = TransactionBinder;
    let writer = RdbcItemWriterBuilder::<Transaction>::new()
        .postgres(pool)
        .table("transactions")
        .add_column("transaction_id")
        .add_column("amount")
        .add_column("currency")
        .add_column("timestamp")
        .add_column("account_from")
        .add_column("account_to")
        .add_column("status")
        .add_column("amount_eur")
        .postgres_binder(&binder)
        .build_postgres();

    let processor = TransactionProcessor;

    let step = StepBuilder::new("csv-to-postgres")
        .chunk::<Transaction, Transaction>(1_000)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()?;

    let exec = job.get_step_execution("csv-to-postgres").unwrap();
    let duration = t0.elapsed();
    let throughput = exec.write_count as f64 / duration.as_secs_f64();

    eprintln!(
        "[Step 1] Done — {} records written in {:.1}s ({:.0} rec/s)",
        exec.write_count,
        duration.as_secs_f64(),
        throughput
    );

    Ok(exec.write_count as u64)
}

// =============================================================================
// Step 2 — PostgreSQL → XML
// =============================================================================

fn run_step2(pool: &PgPool, xml_path: &str) -> Result<u64, BatchError> {
    log::info!("[Step 2] PostgreSQL → XML …");
    let t0 = Instant::now();

    let reader = RdbcItemReaderBuilder::<Transaction>::new()
        .postgres(pool.clone())
        .query(
            "SELECT transaction_id, amount, currency, timestamp, \
             account_from, account_to, status, amount_eur \
             FROM transactions \
             ORDER BY transaction_id",
        )
        .with_page_size(1_000)
        .build_postgres();

    let writer = XmlItemWriterBuilder::<Transaction>::new()
        .root_tag("transactions")
        .item_tag("transaction")
        .from_path(xml_path)
        .map_err(|e| BatchError::ItemWriter(e.to_string()))?;

    let processor = PassThroughProcessor::<Transaction>::new();

    let step = StepBuilder::new("postgres-to-xml")
        .chunk::<Transaction, Transaction>(1_000)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()?;

    let exec = job.get_step_execution("postgres-to-xml").unwrap();
    let duration = t0.elapsed();
    let throughput = exec.write_count as f64 / duration.as_secs_f64();

    eprintln!(
        "[Step 2] Done — {} records written in {:.1}s ({:.0} rec/s)",
        exec.write_count,
        duration.as_secs_f64(),
        throughput
    );

    Ok(exec.write_count as u64)
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), BatchError> {
    env_logger::init();

    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/benchmark".to_string());

    let csv_path = env::var("CSV_PATH")
        .unwrap_or_else(|_| "/tmp/transactions.csv".to_string());

    let xml_path = env::var("XML_PATH")
        .unwrap_or_else(|_| "/tmp/transactions_export.xml".to_string());

    eprintln!("╔══════════════════════════════════════════════════════════╗");
    eprintln!("║  Spring Batch RS — 10M Transaction Benchmark            ║");
    eprintln!("╚══════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("DB  : {}", db_url);
    eprintln!("CSV : {}", csv_path);
    eprintln!("XML : {}", xml_path);
    eprintln!();

    // 1. Connect to PostgreSQL
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .map_err(|e| BatchError::ItemWriter(format!("DB connect failed: {}", e)))?;

    // 2. Ensure table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            transaction_id  VARCHAR(36)       PRIMARY KEY,
            amount          DOUBLE PRECISION  NOT NULL,
            currency        VARCHAR(3)        NOT NULL,
            timestamp       VARCHAR(25)       NOT NULL,
            account_from    VARCHAR(15)       NOT NULL,
            account_to      VARCHAR(15)       NOT NULL,
            status          VARCHAR(15)       NOT NULL,
            amount_eur      DOUBLE PRECISION  NOT NULL DEFAULT 0.0
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| BatchError::ItemWriter(format!("Schema creation failed: {}", e)))?;

    // 3. Clean previous run
    sqlx::query("TRUNCATE TABLE transactions")
        .execute(&pool)
        .await
        .map_err(|e| BatchError::ItemWriter(format!("Truncate failed: {}", e)))?;

    // 4. Generate CSV
    eprintln!("[Generate] Writing {} rows to {} …", TOTAL_RECORDS, csv_path);
    let t_gen = Instant::now();
    generate_csv(&csv_path, TOTAL_RECORDS)?;
    eprintln!("[Generate] Done in {:.1}s", t_gen.elapsed().as_secs_f64());
    eprintln!();

    // 5. Total wall time starts here
    let t_total = Instant::now();

    // 6. Run Step 1
    run_step1(&pool, &csv_path)?;
    eprintln!();

    // 7. Run Step 2
    run_step2(&pool, &xml_path)?;
    eprintln!();

    // 8. Summary
    let total_secs = t_total.elapsed().as_secs_f64();
    eprintln!("╔══════════════════════════════════════════════════════════╗");
    eprintln!("║  BENCHMARK SUMMARY                                      ║");
    eprintln!("╠══════════════════════════════════════════════════════════╣");
    eprintln!("║  Total pipeline duration : {:.1}s", total_secs);
    eprintln!("║  Records processed       : {}", TOTAL_RECORDS);
    eprintln!("║  Average throughput      : {:.0} rec/s", TOTAL_RECORDS as f64 / total_secs);
    eprintln!("╚══════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Hint: measure peak RSS with:");
    eprintln!("  /usr/bin/time -v cargo run --release --example benchmark_csv_postgres_xml \\");
    eprintln!("    --features csv,xml,rdbc-postgres 2>&1 | grep 'Maximum resident'");

    Ok(())
}
