/// CSV support for reading and writing tabular data.
///
/// This module provides components for reading from and writing to CSV files with configurable
/// options for delimiters, headers, and other formatting preferences.
///
/// # Module Architecture
///
/// The CSV module consists of two main components:
///
/// 1. **CsvItemReader**: A reader that deserializes CSV data into Rust structs
///    using serde's deserialization capabilities. It supports reading CSV data
///    from files, strings, or any source implementing the `Read` trait.
///
/// 2. **CsvItemWriter**: A writer that serializes Rust structs into CSV format
///    with configurable options like custom delimiters and header handling.
///
/// Both components follow the builder pattern for easy configuration.
///
/// # Integration with Spring Batch
///
/// These components implement the core `ItemReader` and `ItemWriter` traits from the Spring Batch framework,
/// allowing them to be used in batch processing steps and pipelines. The design follows the Spring Batch
/// philosophy of modular, configurable components that can be combined in various ways.
///
/// # Ownership and Borrowing Considerations
///
/// When using the CSV writers, be aware of Rust's ownership rules:
/// - Writers borrow their destination (file, buffer, etc.) and hold that borrow until dropped
/// - To read from a buffer after writing to it, either:
///   1. Create a separate scope for the writer so it's dropped before reading the buffer
///   2. Clone the buffer before reading from it (less efficient)
///
/// # Features
///
/// - Read CSV data with or without headers
/// - Write data to CSV files with custom formatting
/// - Support for custom delimiters and terminators
/// - Flexible trimming options
/// - Integration with Serde for serialization/deserialization
///
/// # Examples
///
/// ## Reading from CSV
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
///
/// // Define a data structure matching our CSV format
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct City {
///     city: String,
///     country: String,
///     pop: u32,
/// }
///
/// // Sample CSV data
/// let csv_data = "\
/// city,country,pop
/// Boston,United States,4628910
/// Concord,United States,42695
/// ";
///
/// // Create a reader from our CSV
/// let reader = CsvItemReaderBuilder::new()
///     .has_headers(true)
///     .delimiter(b',')
///     .from_reader(csv_data.as_bytes());
///
/// // Read and process the cities
/// let mut cities: Vec<City> = Vec::new();
/// while let Some(city) = reader.read().unwrap() {
///     cities.push(city);
/// }
///
/// // Verify results
/// assert_eq!(cities.len(), 2);
/// assert_eq!(cities[0].city, "Boston");
/// assert_eq!(cities[0].country, "United States");
/// assert_eq!(cities[0].pop, 4628910);
///
/// assert_eq!(cities[1].city, "Concord");
/// assert_eq!(cities[1].country, "United States");
/// assert_eq!(cities[1].pop, 42695);
/// ```
///
/// ## Writing to CSV
///
/// ```
/// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
///
/// // Define a data structure for serialization
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
///     occupation: String,
/// }
///
/// // Create some people
/// let people = vec![
///     Person {
///         name: "Alice".to_string(),
///         age: 28,
///         occupation: "Engineer".to_string(),
///     },
///     Person {
///         name: "Bob".to_string(),
///         age: 35,
///         occupation: "Designer".to_string(),
///     },
/// ];
///
/// // Create a writer with a vector buffer (could also use a file)
/// let mut buffer = Vec::new();
/// {
///     let writer = CsvItemWriterBuilder::new()
///         .has_headers(true)
///         .delimiter(b',')
///         .from_writer(&mut buffer);
///
///     // Write the people to CSV
///     writer.write(&people).unwrap();
///     // Use explicit type parameter with flush to help type inference
///     ItemWriter::<Person>::flush(&writer).unwrap();
/// } // writer is dropped here, releasing the borrow
///
/// // Convert buffer to string to see the output
/// let csv_output = String::from_utf8(buffer).unwrap();
///
/// // Output will be:
/// // name,age,occupation
/// // Alice,28,Engineer
/// // Bob,35,Designer
/// ```
///
/// ## Converting data between CSV and other formats
///
/// The CSV module can be used in combination with other modules to build
/// data processing pipelines:
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
/// use spring_batch_rs::core::item::{ItemReader, ItemWriter};
/// use spring_batch_rs::core::step::{StepBuilder, StepInstance};
/// use spring_batch_rs::core::job::{JobBuilder, Job};
/// use serde::{Deserialize, Serialize};
/// use std::fs::File;
///
/// // This example shows how to use CSV reader and writer in a batch job
/// // that reads data, transforms it, and writes it back
///
/// #[derive(Debug, Deserialize, Serialize)]
/// struct Record {
///     id: u32,
///     value: String,
/// }
///
/// // In a real application, you would:
/// // 1. Set up input and output files
/// // 2. Configure readers and writers
/// // 3. Build and run the job
///
/// // Example (not actually executed in doctest):
/// // let reader = CsvItemReaderBuilder::new()
/// //     .has_headers(true)
/// //     .from_path("input.csv");
/// //
/// // let writer = CsvItemWriterBuilder::new()
/// //     .has_headers(true)
/// //     .from_path("output.csv");
/// //
/// // let step: StepInstance<Record, Record> = StepBuilder::new()
/// //     .reader(&reader)
/// //     .writer(&writer)
/// //     .build();
/// //
/// // let job = JobBuilder::new().start(&step).build();
/// // job.run().unwrap();
/// ```

/// A module providing facilities for reading CSV data records.
pub mod csv_reader;

/// A module providing facilities for writing CSV data records.
pub mod csv_writer;
