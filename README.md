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
 + CSV reader and writer
 + JSON reader and writer

 ## Roadmap
 + XML reader and writer
 + SQL reader and writer
 + Filter items
 + MongoDB reader and writer
 + Kafka reader and writer
 + Pulsar reader and writer
 + Retry/Skip policies
 + Save execution data in database

 ## Examples

 Make sure you activated the full features crate on Cargo.toml:

 ```toml
[dependencies]
spring-batch-rs = { version = "0.1.1", features = ["full"] }
```

Then, on your main.rs:

```rust,no_run
 fn main() -> Result<(), BatchError> {
    let csv = "year,make,model,description
    1948,Porsche,356,Luxury sports car
    1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new().delimiter(b',').from_reader(csv.as_bytes());

    let writer = LoggerWriter::new();

    let step: Step<Record, Record> = StepBuilder::new()
        .reader(&mut reader)
        .writer(&mut writer)
        .chunk(4)
        .build();

    step.execute();
    Ok(())
 }
 ```

 ### Read CSV file with headers
 ```sh
$ git clone git://github.com/sboussekeyt/spring-batch-rs
$ cd spring-batch-rs
$ cargo run --example csv_reader_with_headers --all-features < examples/data/cars_with_headers.csv
```

### Read Json file
 ```sh
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
