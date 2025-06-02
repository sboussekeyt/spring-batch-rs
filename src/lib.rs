#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(missing_docs)]

/*!
 <div align="center">
   <h1>Spring-Batch for Rust</h1>
   <h3>🐞 A toolkit for building enterprise-grade batch applications</h3>

   [![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
   [![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
   [![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml)
   [![Discord chat](https://img.shields.io/discord/1097536141617528966.svg?logo=discord&style=flat-square)](https://discord.gg/9FNhawNsG6)
   [![CodeCov](https://codecov.io/gh/sboussekeyt/spring-batch-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sboussekeyt/spring-batch-rs)
   ![license](https://shields.io/badge/license-MIT%2FApache--2.0-blue)

  </div>

 # Spring-Batch for Rust

 Inspired by the robust Java Spring Batch framework, **Spring Batch for Rust** brings its battle-tested concepts to the Rust ecosystem. It offers a comprehensive toolkit for developing efficient, reliable, and enterprise-grade batch applications. This framework is designed to address the challenges of handling large-scale data processing tasks, providing developers with the tools needed for complex batch operations.

 ## Architecture Overview

 ```mermaid
 graph TB
     subgraph "Spring Batch RS Architecture"
         Job[Job] --> Step1[Step 1]
         Job --> Step2[Step 2]
         Job --> StepN[Step N]

         subgraph "Chunk-Oriented Processing"
             Step1 --> Reader[ItemReader]
             Reader --> Processor[ItemProcessor]
             Processor --> Writer[ItemWriter]
         end

         subgraph "Tasklet Processing"
             Step2 --> Tasklet[Tasklet]
         end
     end
 ```

 ## Core Concepts

Understanding these core components will help you get started:

- **Job:** Represents the entire batch process. A `Job` is composed of one or more `Step`s.
- **Step:** A domain object that encapsulates an independent, sequential phase of a batch job. Every `Job` is composed of one or more `Step`s. A `Step` can either process data in chunks (chunk-oriented processing) or execute a single task (tasklet).
- **ItemReader:** An abstraction that represents the retrieval of input for a `Step`, one item at a time.
- **ItemProcessor:** An abstraction that represents the business logic of processing an item. The item read by the `ItemReader` is passed to the `ItemProcessor`.
- **ItemWriter:** An abstraction that represents the output of a `Step`, one batch or chunk of items at a time.
- **Tasklet:** An abstraction that represents a single task or operation that can be executed as part of a step. Tasklets are useful for operations that don't fit the chunk-oriented processing model, such as file operations, database maintenance, or custom business logic.

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
| fake          | Enables a fake `ItemReader`, useful for generating mock datasets |
| logger        | Enables a logger `ItemWriter`, useful for debugging purposes  |
| full          | Enables all available features                                |

 ## Processing Models

Spring Batch for Rust supports two main processing models:

### Chunk-Oriented Processing

This is the traditional batch processing model where data is read, processed, and written in configurable chunks. It's ideal for:
- Processing large datasets
- ETL operations
- Data transformations
- Scenarios where you need transaction boundaries and fault tolerance

#### Available Item Readers & Writers

**File-based:**
- **CSV**: Read/write CSV files with configurable delimiters and headers
- **JSON**: Read/write JSON files with pretty printing support
- **XML**: Read/write XML files with custom serialization

**Database:**
- **ORM**: Read/write using SeaORM with pagination and filtering support
- **RDBC**: Direct database access for PostgreSQL, MySQL, and SQLite
- **MongoDB**: Native MongoDB document operations

**Utility:**
- **Fake**: Generate mock data for testing
- **Logger**: Debug output for development

### Tasklet Processing

Tasklets provide a simple interface for executing single tasks that don't fit the chunk-oriented model. They're perfect for:
- File operations (compression, cleanup, archiving)
- Database maintenance tasks
- System administration operations
- Custom business logic that operates on entire datasets

#### Built-in Tasklets

- **ZipTasklet**: Compress files and directories into ZIP archives with configurable compression levels and file filtering

#### Creating Custom Tasklets

Implement the `Tasklet` trait to create your own custom operations:

```rust
use spring_batch_rs::core::step::{Tasklet, StepExecution, RepeatStatus};
use spring_batch_rs::BatchError;
use log::info;

struct MyCustomTasklet;

impl Tasklet for MyCustomTasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        // Your custom logic here
        info!("Executing custom tasklet for step: {}", step_execution.name);
        Ok(RepeatStatus::Finished)
    }
}
```

 ## Getting Started
 Make sure you activated the suitable features crate on Cargo.toml:

```toml
[dependencies]
spring-batch-rs = { version = "<version>", features = ["csv", "json"] }
# Or for database operations:
spring-batch-rs = { version = "<version>", features = ["orm", "csv"] }
# Or for all features:
spring-batch-rs = { version = "<version>", features = ["full"] }
```

### Chunk-Oriented Processing Example

Simple CSV to JSON transformation:

```rust
# use serde::{Deserialize, Serialize};
# use spring_batch_rs::{
#     core::{job::{Job, JobBuilder}, step::StepBuilder, item::ItemProcessor},
#     item::{csv::csv_reader::CsvItemReaderBuilder, json::json_writer::JsonItemWriterBuilder},
#     BatchError,
# };
# #[derive(Deserialize, Serialize, Clone)]
# struct Product {
#     id: u32,
#     name: String,
#     price: f64,
# }
# struct PassThroughProcessor;
# impl ItemProcessor<Product, Product> for PassThroughProcessor {
#     fn process(&self, item: &Product) -> Result<Product, BatchError> {
#         Ok(item.clone())
#     }
# }

fn main() -> Result<(), BatchError> {
    let csv_data = "id,name,price\n1,Laptop,999.99\n2,Mouse,29.99";

    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json");

    let processor = PassThroughProcessor;

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

### ORM Database Processing Example

Reading from database with SeaORM:

```rust
# use spring_batch_rs::{
#     core::{job::{Job, JobBuilder}, step::StepBuilder},
#     item::{csv::csv_writer::CsvItemWriterBuilder, fake::person_reader::{PersonReaderBuilder, Person}},
#     BatchError,
# };
# use serde::{Deserialize, Serialize};
#
# // This example shows the pattern for ORM usage
# // In real usage, you would use your actual SeaORM entities
# #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
# struct Product {
#     id: i32,
#     name: String,
#     active: bool,
# }

// This example shows the pattern - in real usage you would use your SeaORM entities
async fn process_database_to_csv() -> Result<(), Box<dyn std::error::Error>> {
    // In real usage with SeaORM:
    // let db = Database::connect("sqlite::memory:").await?;
    // let query = ProductEntity::find()
    //     .filter(product::Column::Active.eq(true))
    //     .order_by_asc(product::Column::Id);
    // let reader = OrmItemReaderBuilder::new()
    //     .connection(&db)
    //     .query(query)
    //     .page_size(100)
    //     .build();

    // For this example, we'll use a fake reader to demonstrate the pattern
    let reader = PersonReaderBuilder::new().number_of_items(100).build();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path("active_products.csv");

    let step = StepBuilder::new("db_to_csv")
        .chunk::<Person, Person>(50)
        .reader(&reader)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
```

### Tasklet Processing Example

File operations using tasklets:

```rust
# use spring_batch_rs::{
#     core::{job::{Job, JobBuilder}, step::StepBuilder},
#     BatchError,
# };
# #[cfg(feature = "zip")]
# use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
# use std::fs;
# use std::env::temp_dir;

fn main() -> Result<(), BatchError> {
    #[cfg(feature = "zip")]
    {
        // Create test data for the example
        let temp_data_dir = temp_dir().join("test_data");
        fs::create_dir_all(&temp_data_dir).unwrap();
        fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();

        let zip_tasklet = ZipTaskletBuilder::new()
            .source_path(&temp_data_dir)
            .target_path("./backup.zip")
            .compression_level(6)
            .build()?;

        let step = StepBuilder::new("backup_files")
            .tasklet(&zip_tasklet)
            .build();

        let job = JobBuilder::new().start(&step).build();
        let result = job.run();

        // Cleanup test files
        fs::remove_file("./backup.zip").ok();
        fs::remove_dir_all(&temp_data_dir).ok();

        result.map(|_| ())
    }

    #[cfg(not(feature = "zip"))]
    {
        // Example without zip feature
        use spring_batch_rs::core::step::{Tasklet, StepExecution, RepeatStatus};

        struct SimpleTasklet;
        impl Tasklet for SimpleTasklet {
            fn execute(&self, _: &StepExecution) -> Result<RepeatStatus, BatchError> {
                println!("Simple tasklet executed");
                Ok(RepeatStatus::Finished)
            }
        }

        let tasklet = SimpleTasklet;
        let step = StepBuilder::new("simple_task")
            .tasklet(&tasklet)
            .build();

        let job = JobBuilder::new().start(&step).build();
        job.run().map(|_| ())
    }
}
```

## Examples
+ [Generate CSV file from JSON file with processor](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/generate_csv_file_from_json_file_with_processor.rs)
+ [Generate JSON file from CSV string with fault tolerance](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/generate_json_file_from_csv_string_with_fault_tolerance.rs)
+ [Generate JSON file from fake persons](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/generate_json_file_from_fake_persons.rs)
+ [Generate CSV file without headers from fake persons](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/generate_csv_file_without_headers_from_fake_persons.rs)
+ [Insert records into Mysql database](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/insert_records_into_mysql_database.rs)
+ [Log records from Postgres database](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/log_records_from_postgres_database.rs)
+ [Read records from MongoDb database](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/read_records_from_mongodb_database.rs)
+ [Write records to MongoDb database](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/write_records_to_mongodb_database.rs)
+ [ZIP files using tasklet](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/zip_files_tasklet.rs)

 ## Roadmap

We are actively working on enhancing `spring-batch-rs` with more features:

- [ ] Item filtering capabilities
- [ ] Kafka reader and writer
- [ ] Parquet reader and writer
- [ ] Advanced Retry/Skip policies for fault tolerance
- [ ] Persist job execution metadata (e.g., in a database)

 ## License
 Licensed under either of

 -   Apache License, Version 2.0
     ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 -   MIT license
     ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

 at your option.

 ## Contribution
 Unless you explicitly state otherwise, any contribution intentionally submitted
 for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
 dual licensed as above, without any additional terms or conditions

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
