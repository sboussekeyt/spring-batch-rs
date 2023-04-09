#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(missing_docs)]

//! <div align="center">
//!
//!   <h1>Spring-Batch for Rust</h1>
//!
//!   <h3>🐚 A Batch tool (inspired by Spring) for Rust</h3>
//!
//!   [![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
//!   [![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
//!   [![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/rust.yml)
//!
//! </div>
//!
//! # Spring-Batch for Rust
//!
//! ## Features
//! + CSV reader and writer
//! + XML reader and writer (roadmap)
//! + JSON reader and writer (roadmap)
//! + SQL reader and writer (roadmap)
//! + MongoDB reader and writer (roadmap)
//! 
//! ## Exemples
//! ```
//! # use std::fmt;
//! # use serde::{Deserialize, Serialize};
//! # use spring_batch_rs::{
//! # core::step::{Step, StepBuilder},
//! # item::csv::csv_reader::CsvItemReaderBuilder,
//! # error::BatchError,
//! # item::logger::LoggerWriter,
//! # };
//! #[derive(Deserialize, Serialize, Debug)]
//! struct Record {
//!     year: u16,
//!     make: String,
//!     model: String,
//!     description: String,
//! }
//!
//! impl fmt::Display for Record {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(
//!             f,
//!             "(year={}, make={}, model={}, description={})",
//!             self.year, self.make, self.model, self.description
//!         )
//!     }
//! }
//!
//! let csv = "year,make,model,description
//! 1948,Porsche,356,Luxury sports car
//! 1967,Ford,Mustang fastback 1967,American car";
//!
//! let reader = CsvItemReaderBuilder::new().delimiter(b',').from_reader(csv.as_bytes());
//!
//! let writer = LoggerWriter::new();
//!
//! let step: Step<Record, Record> = StepBuilder::new()
//!     .reader(&reader)
//!     .writer(&writer)
//!     .chunk(4)
//!     .build();
//! ```
//! 
pub mod core;

/// Error types for batch operations
pub mod error;

/// Set of items readers / writers  (for exemple: csv reader and writer)
pub mod item;

#[doc(inline)]
pub use error::*;

#[cfg(feature = "logger")]
#[doc(inline)]
pub use item::logger::*;

#[cfg(feature = "csv")]
#[doc(inline)]
pub use item::csv::csv_reader::*;
