#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(missing_docs)]

/*!
 <div align="center">
   <h1>Spring-Batch for Rust</h1>
   <h3>🐞 A Batch tool (inspired by Spring) for Rust</h3>

   [![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
   [![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
   [![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/rust.yml)
   [![Discord chat](https://img.shields.io/discord/1097536141617528966.svg?logo=discord&style=flat-square)](https://discord.gg/9FNhawNsG6)

  </div>

 # Spring-Batch for Rust

 ## Features
 + CSV reader and writer
 + JSON reader and writer
 + XML reader and writer (roadmap)
 + SQL reader and writer (roadmap)
 + MongoDB reader and writer (roadmap)

 ## Examples
```no_run
 # use std::fmt;
 # use serde::{Deserialize, Serialize};
 # use spring_batch_rs::{
 # core::step::{Step, StepBuilder},
 # item::csv::csv_reader::CsvItemReaderBuilder,
 # error::BatchError,
 # item::logger::LoggerWriter,
 # };
 #[derive(Deserialize, Serialize, Debug)]
 struct Record {
     year: u16,
     make: String,
     model: String,
     description: String,
 }

 impl fmt::Display for Record {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         write!(
             f,
             "(year={}, make={}, model={}, description={})",
             self.year, self.make, self.model, self.description
         )
     }
 }

 let csv = "year,make,model,description
 1948,Porsche,356,Luxury sports car
 1967,Ford,Mustang fastback 1967,American car";

 let mut reader = CsvItemReaderBuilder::new().delimiter(b',').from_reader(csv.as_bytes());

 let mut writer = LoggerWriter::new();

 let mut step: Step<Record, Record> = StepBuilder::new()
     .reader(&mut reader)
     .writer(&mut writer)
     .chunk(4)
     .build();

 step.execute();
 ```
 ### Read CSV file with headers
 ```ignore
$ git clone git://github.com/sboussekeyt/spring-batch-rs
$ cd spring-batch-rs
$ cargo run --example csv_reader_with_headers --all-features < examples/data/cars_with_headers.csv
```
  ### Read Json file
 ```ignore
$ git clone git://github.com/sboussekeyt/spring-batch-rs
$ cd spring-batch-rs
$ cargo run --example json_reader --all-features < examples/data/persons.json
```

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
