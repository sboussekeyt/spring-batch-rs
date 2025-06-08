#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(missing_docs)]

/*!
 <div align="center">
   <h1>Spring-Batch for Rust</h1>
   <h3>A toolkit for building enterprise-grade batch applications</h3>

   [![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
   [![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
   [![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml)
   [![Discord chat](https://img.shields.io/discord/1097536141617528966.svg?logo=discord&style=flat-square)](https://discord.gg/9FNhawNsG6)
   [![CodeCov](https://codecov.io/gh/sboussekeyt/spring-batch-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sboussekeyt/spring-batch-rs)
   ![license](https://shields.io/badge/license-MIT%2FApache--2.0-blue)

  </div>

 # Spring-Batch for Rust

 Inspired by the robust Java Spring Batch framework, **Spring Batch for Rust** brings its battle-tested concepts to the Rust ecosystem. It offers a comprehensive toolkit for developing efficient, reliable, and enterprise-grade batch applications.

 ## 📚 Complete Documentation

 For comprehensive documentation, tutorials, and examples, visit our website:
 **[https://sboussekeyt.github.io/spring-batch-rs/](https://sboussekeyt.github.io/spring-batch-rs/)**

 This crate provides the core functionality. See the website for:
 - [Getting Started Guide](https://sboussekeyt.github.io/spring-batch-rs/docs/getting-started)
 - [Feature-specific Tutorials](https://sboussekeyt.github.io/spring-batch-rs/docs/tutorials)
 - [Complete Examples Gallery](https://sboussekeyt.github.io/spring-batch-rs/docs/examples)
 - [Best Practices and Patterns](https://sboussekeyt.github.io/spring-batch-rs/docs/best-practices)

 ## Core Concepts

- **Job:** Represents the entire batch process composed of one or more steps
- **Step:** Encapsulates an independent phase of a batch job (chunk-oriented or tasklet)
- **ItemReader:** Retrieval of input for a step, one item at a time
- **ItemProcessor:** Business logic for processing items
- **ItemWriter:** Output of a step, one batch or chunk of items at a time
- **Tasklet:** Single task operations that don't fit the chunk-oriented model

 ## Features

The crate is modular, allowing you to enable only the features you need:

| **Feature**   | **Description**                                               |
|---------------|---------------------------------------------------------------|
| mongodb       | Enables `ItemReader` and `ItemWriter` for MongoDB databases   |
| rdbc-postgres | Enables RDBC `ItemReader` and `ItemWriter` for PostgreSQL     |
| rdbc-mysql    | Enables RDBC `ItemReader` and `ItemWriter` for MySQL and MariaDB |
| rdbc-sqlite   | Enables RDBC `ItemReader` and `ItemWriter` for SQLite         |
| orm           | Enables ORM `ItemReader` and `ItemWriter` using SeaORM        |
| json          | Enables JSON `ItemReader` and `ItemWriter`                    |
| csv           | Enables CSV `ItemReader` and `ItemWriter`                     |
| xml           | Enables XML `ItemReader` and `ItemWriter`                     |
| zip           | Enables ZIP compression `Tasklet` for file archiving          |
| ftp           | Enables FTP `Tasklet` for file and folder operations          |
| fake          | Enables a fake `ItemReader`, useful for generating mock datasets |
| logger        | Enables a logger `ItemWriter`, useful for debugging purposes  |
| full          | Enables all available features                                |

 ## Quick Example

```rust
# use serde::{Deserialize, Serialize};
# use spring_batch_rs::{
#     core::{job::{Job, JobBuilder}, step::StepBuilder, item::PassThroughProcessor},
#     item::{csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder},
#     BatchError,
# };
# #[derive(Deserialize, Serialize, Clone)]
# struct Product {
#     id: u32,
#     name: String,
#     price: f64,
# }

fn main() -> Result<(), BatchError> {
    let csv_data = "id,name,price\n1,Laptop,999.99\n2,Mouse,29.99";

    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json");

    let processor = PassThroughProcessor::<Product>::new();

    let step = StepBuilder::new("csv_to_json")
        .chunk::<Product, Product>(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ())
}
```

For more examples and detailed guides, visit [our website](https://sboussekeyt.github.io/spring-batch-rs/docs/examples).

*/

/// Core module for batch operations
pub mod core;

/// Error types for batch operations
pub mod error;

#[doc(inline)]
pub use error::*;

/// Set of items readers / writers  (for exemple: csv reader and writer)
pub mod item;

/// Set of tasklets for common batch operations
pub mod tasklet;
