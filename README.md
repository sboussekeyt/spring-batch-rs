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

 ## Getting Started
 Make sure you activated the suitable features crate on Cargo.toml:

```toml
[dependencies]
spring-batch-rs = { version = "<version>", features = ["<full|json|csv|fake|logger>"] }
```

Then, on your main.rs:

```rust,no_run
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
