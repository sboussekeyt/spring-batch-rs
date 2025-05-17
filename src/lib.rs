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
- **Step:** A domain object that encapsulates an independent, sequential phase of a batch job. Every `Job` is composed of one or more `Step`s. A `Step` typically involves reading data, processing it, and writing it out.
- **ItemReader:** An abstraction that represents the retrieval of input for a `Step`, one item at a time.
- **ItemProcessor:** An abstraction that represents the business logic of processing an item. The item read by the `ItemReader` is passed to the `ItemProcessor`.
- **ItemWriter:** An abstraction that represents the output of a `Step`, one batch or chunk of items at a time.

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
| fake          | Enables a fake `ItemReader`, useful for generating mock datasets |
| logger        | Enables a logger `ItemWriter`, useful for debugging purposes  |
| full          | Enables all available features                                |

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
spring-batch-rs = { version = "<version>", features = ["<full|json|csv|fake|logger>"] }
```

Then, on your main.rs:

```rust
# use serde::{Deserialize, Serialize};
# use spring_batch_rs::{
#     core::{
#         item::{ItemProcessor, ItemProcessorResult},
#         job::{Job, JobBuilder},
#         step::{Step, StepBuilder, StepInstance, StepStatus},
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

    let reader = CsvItemReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(csv.as_bytes());

    let processor = UpperCaseProcessor::default();

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join("cars.json"));

    let step: StepInstance<Car, Car> = StepBuilder::new()
        .reader(&reader) // set csv reader
        .writer(&writer) // set json writer
        .processor(&processor) // set upper case processor
        .chunk(2) // set commit interval
        .skip_limit(2) // set fault tolerance
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    assert!(result.is_ok());
    assert!(step.get_status() == StepStatus::Success);

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
