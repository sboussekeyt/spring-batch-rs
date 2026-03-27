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
}

#[tokio::main]
async fn main() {
    // TODO: implementation in Task 5
    log::info!("Benchmark — implementation pending (Tasks 3–5)");
}
