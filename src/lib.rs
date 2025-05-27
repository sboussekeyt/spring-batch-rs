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

 ## Roadmap

We are actively working on enhancing `spring-batch-rs` with more features:

- [ ] Item filtering capabilities
- [ ] Kafka reader and writer
- [ ] Parquet reader and writer
- [ ] Advanced Retry/Skip policies for fault tolerance
- [ ] Persist job execution metadata (e.g., in a database)

 ## Getting Started
 Make sure you activated the suitable features crate on Cargo.toml:

```toml
[dependencies]
spring-batch-rs = { version = "<version>", features = ["<full|json|csv|xml|zip|fake|logger>"] }
```

### Chunk-Oriented Processing Example

```rust
# use serde::{Deserialize, Serialize};
# use spring_batch_rs::{
#     core::{
#         item::{ItemProcessor, ItemProcessorResult},
#         job::{Job, JobBuilder},
#         step::{Step, StepBuilder, StepStatus},
#     },
#     error::BatchError,
#     item::csv::csv_reader::CsvItemReaderBuilder,
#     item::json::json_writer::JsonItemWriterBuilder,
# };
# use std::env::temp_dir;
# #[derive(Deserialize, Serialize, Debug, Clone)]
# struct Car {
#     year: u16,
#     make: String,
#     model: String,
#     description: String,
# }
# #[derive(Default)]
# struct UpperCaseProcessor {}
# impl ItemProcessor<Car, Car> for UpperCaseProcessor {
#     fn process(&self, item: &Car) -> ItemProcessorResult<Car> {
#         let car = Car {
#             year: item.year,
#             make: item.make.to_uppercase(),
#             model: item.model.to_uppercase(),
#             description: item.description.to_uppercase(),
#         };
#         Ok(car)
#     }
# }

fn main() -> Result<(), BatchError> {
    let csv = "year,make,model,description
   1948,Porsche,356,Luxury sports car
   1995,Peugeot,205,City car
   2021,Mazda,CX-30,SUV Compact
   1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::<Car>::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(csv.as_bytes());

    let processor = UpperCaseProcessor::default();

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join("cars.json"));

    let step = StepBuilder::new("process_cars")
        .chunk(2) // set commit interval
        .reader(&reader) // set csv reader
        .processor(&processor) // set upper case processor
        .writer(&writer) // set json writer
        .skip_limit(2) // set fault tolerance
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    assert!(result.is_ok());

    Ok(())
}
```

### Tasklet Processing Example

For operations that don't fit the chunk-oriented processing model, you can use tasklets:

```rust
# use spring_batch_rs::{
#     core::{
#         job::{Job, JobBuilder},
#         step::{StepBuilder, StepExecution, RepeatStatus, Tasklet},
#     },
#     BatchError,
# };
# #[cfg(feature = "zip")]
# use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
# use log::info;
# use std::fs;
# use std::env::temp_dir;

// Custom tasklet example
struct FileCleanupTasklet {
    directory: String,
}

impl Tasklet for FileCleanupTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        // Perform file cleanup logic here
        info!("Cleaning up directory: {}", self.directory);
        Ok(RepeatStatus::Finished)
    }
}

fn main() -> Result<(), BatchError> {
    // Create a cleanup tasklet
    let cleanup_tasklet = FileCleanupTasklet {
        directory: "/tmp/batch_files".to_string(),
    };

    // Create steps using tasklets
    let cleanup_step = StepBuilder::new("cleanup")
        .tasklet(&cleanup_tasklet)
        .build();

    #[cfg(feature = "zip")]
    {
        // Create test data directory and file for the example
        let temp_data_dir = temp_dir().join("test_data");
        fs::create_dir_all(&temp_data_dir).unwrap();
        fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();

        let archive_path = temp_dir().join("archive.zip");

        // Create a ZIP compression tasklet (requires 'zip' feature)
        let zip_tasklet = ZipTaskletBuilder::new()
            .source_path(&temp_data_dir)
            .target_path(&archive_path)
            .compression_level(6)
            .build()?;

        let zip_step = StepBuilder::new("compress")
            .tasklet(&zip_tasklet)
            .build();

        // Create and run job with multiple steps
        let job = JobBuilder::new()
            .start(&cleanup_step)
            .next(&zip_step)
            .build();

        let result = job.run();
        assert!(result.is_ok());

        // Cleanup test files
        fs::remove_file(&archive_path).ok();
        fs::remove_dir_all(&temp_data_dir).ok();
    }

    #[cfg(not(feature = "zip"))]
    {
        // Create and run job with single step
        let job = JobBuilder::new().start(&cleanup_step).build();
        let result = job.run();
        assert!(result.is_ok());
    }

    Ok(())
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
