# Spring-Batch for Rust

> A toolkit for building enterprise-grade batch applications

[![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
[![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
[![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml)
[![Discord chat](https://img.shields.io/discord/1097536141617528966.svg?logo=discord&style=flat-square)](https://discord.gg/9FNhawNsG6)
[![CodeCov](https://codecov.io/gh/sboussekeyt/spring-batch-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sboussekeyt/spring-batch-rs)
![license](https://shields.io/badge/license-MIT%2FApache--2.0-blue)

Inspired by the robust Java Spring Batch framework, **Spring Batch for Rust** brings its battle-tested concepts to the Rust ecosystem. It offers a comprehensive toolkit for developing efficient, reliable, and enterprise-grade batch applications.

## 📚 Documentation

For comprehensive documentation, tutorials, and examples:

**🌐 [Visit our Website](https://sboussekeyt.github.io/spring-batch-rs/)**

- [Getting Started Guide](https://sboussekeyt.github.io/spring-batch-rs/docs/getting-started)
- [Feature Tutorials](https://sboussekeyt.github.io/spring-batch-rs/docs/tutorials)
- [API Reference](https://sboussekeyt.github.io/spring-batch-rs/docs/api)
- [Examples Gallery](https://sboussekeyt.github.io/spring-batch-rs/docs/examples)

## Why Spring Batch for Rust?

- **Performance & Safety:** Leverage Rust's performance and memory safety for demanding batch jobs
- **Familiar Concepts:** If you're familiar with Spring Batch, you'll feel right at home
- **Extensible:** Designed with modularity in mind, allowing for custom readers, writers, and processors
- **Ecosystem:** Integrates with popular Rust crates for various data sources and formats

## Core Concepts

- **Job:** Represents the entire batch process composed of one or more steps
- **Step:** Encapsulates an independent phase of a batch job (chunk-oriented or tasklet)
- **ItemReader:** Retrieval of input for a step, one item at a time
- **ItemProcessor:** Business logic for processing items
- **ItemWriter:** Output of a step, one batch or chunk of items at a time
- **Tasklet:** Single task operations that don't fit the chunk-oriented model

## Features

| **Feature**     | **Description**                                                      |
| --------------- | -------------------------------------------------------------------- |
| `mongodb`       | Enables `ItemReader` and `ItemWriter` for MongoDB databases          |
| `rdbc-postgres` | Enables RDBC `ItemReader` and `ItemWriter` for PostgreSQL            |
| `rdbc-mysql`    | Enables RDBC `ItemReader` and `ItemWriter` for MySQL and MariaDB     |
| `rdbc-sqlite`   | Enables RDBC `ItemReader` and `ItemWriter` for SQLite                |
| `orm`           | Enables ORM `ItemReader` and `ItemWriter` using SeaORM               |
| `json`          | Enables JSON `ItemReader` and `ItemWriter`                           |
| `csv`           | Enables CSV `ItemReader` and `ItemWriter`                            |
| `xml`           | Enables XML `ItemReader` and `ItemWriter`                            |
| `zip`           | Enables ZIP compression `Tasklet` for file archiving                 |
| `ftp`           | Enables FTP `Tasklet` for file and folder upload/download operations |
| `fake`          | Enables a fake `ItemReader`, useful for generating mock datasets     |
| `logger`        | Enables a logger `ItemWriter`, useful for debugging purposes         |
| `full`          | Enables all available features                                       |

## Quick Start

### 1. Add to your `Cargo.toml`

```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json"] }
```

### 2. Simple CSV to JSON Example

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

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
        .chunk(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ())
}
```

## More Examples

For comprehensive examples, tutorials, and advanced usage patterns, visit our website:

**[https://sboussekeyt.github.io/spring-batch-rs/docs/examples](https://sboussekeyt.github.io/spring-batch-rs/docs/examples)**

## Community

- [Discord](https://discord.gg/9FNhawNsG6) - Chat with the community
- [GitHub Issues](https://github.com/sboussekeyt/spring-batch-rs/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/sboussekeyt/spring-batch-rs/discussions) - Questions and discussions

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
