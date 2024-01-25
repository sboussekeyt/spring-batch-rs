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
 Spring Batch for Rust, offers a robust and flexible framework for the development of batch processing applications, addressing the challenges of handling large-scale data processing tasks efficiently and reliably. It provides developers a comprehensive toolkit for building enterprise-grade batch applications.

 ## Features
| **Feature**   | **Description**                                               |
|---------------|---------------------------------------------------------------|
| mongodb       | Enable reader and writer for Mongodb database                 |
| rdbc-postgres | Enable rdbc reader and writer for Postgres database           |
| rdbc-mysql    | Enable rdbc reader and writer for Mysql and MariaDb databases |
| rdbc-sqlite   | Enable rdbc reader and writer for Sqlite database             |
| json          | Enable json reader and writer                                 |
| csv           | Enable csv reader and writer                                  |
| fake          | Enable fake reader. Useful for generate fake dataset          |
| logger        | Enable logger writer. Useful for debugging                    |

 ## Roadmap
 + XML reader and writer
 + SQL reader and writer
 + Kafka reader and writer
 + Pulsar reader and writer
 + Retry/Skip policies
 + Save execution data in database

 ## Getting Started
 Make sure you activated the suitable features crate on Cargo.toml:

```toml
[dependencies]
spring-batch-rs = { version = "<version>", features = ["<full|json|csv|fake|logger>"] }
```

Then, on your main.rs:

```rust
# use serde::{Deserialize, Serialize};
# use spring_batch_rs::core::item::ItemProcessor;
# use spring_batch_rs::{
#     core::step::{Step, StepBuilder, StepStatus},
#     error::BatchError,
#     item::csv::csv_reader::CsvItemReaderBuilder,
#     JsonItemWriterBuilder,
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
#     fn process<'a>(&'a self, item: &'a Car) -> Car {
#         let car = Car {
#             year: item.year,
#             make: item.make.to_uppercase(),
#             model: item.model.to_uppercase(),
#             description: item.description.to_uppercase(),
#         };
#         car
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

    let step: Step<Car, Car> = StepBuilder::new()
        .reader(&reader) // set csv reader
        .writer(&writer) // set json writer
        .processor(&processor) // set upper case processor
        .chunk(2) // set commit interval
        .skip_limit(2) // set fault tolerance
        .build();

    let result = step.execute();

    assert!(StepStatus::SUCCESS == result.status);

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
pub use item::csv::{csv_reader::*, csv_writer::*};

#[cfg(feature = "json")]
#[doc(inline)]
pub use item::json::{json_reader::*, json_writer::*};

#[cfg(feature = "fake")]
#[doc(inline)]
pub use item::fake::person_reader::*;

#[cfg(feature = "rdbc")]
#[doc(inline)]
pub use item::rdbc::rdbc_reader::*;

#[cfg(feature = "mongodb")]
#[doc(inline)]
pub use item::mongodb::*;
