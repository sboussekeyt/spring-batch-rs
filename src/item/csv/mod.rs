/// This module provides functionality for reading CSV files.
pub mod csv_reader;

/// This module provides a CSV item writer implementation for Spring Batch.
/// It allows writing items to a CSV file using the `ItemWriter` trait.
///
/// The `CsvItemWriter` struct is responsible for writing items to a CSV file.
/// It uses the `csv` crate for CSV serialization and writing.
///
/// The `CsvItemWriterBuilder` struct is a builder for creating instances of `CsvItemWriter`.
/// It allows configuring options such as the delimiter and whether to include headers in the CSV file.
///
/// Example usage:
///
/// ```rust
/// use spring_batch_rs::item::csv::csv_writer::{CsvItemWriterBuilder, CsvItemWriter};
/// use spring_batch_rs::core::item::{ItemWriter, ItemWriterResult};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let writer = CsvItemWriterBuilder::new()
///     .has_headers(true)
///     .from_path("target/output.csv");
///
/// let people = vec![
///     Person { name: "Alice".to_string(), age: 25 },
///     Person { name: "Bob".to_string(), age: 30 },
/// ];
///
/// writer.write(&people).unwrap();
/// ```
pub mod csv_writer;
