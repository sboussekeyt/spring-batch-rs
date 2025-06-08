# Getting Started

Welcome to Spring Batch RS! This guide will help you get up and running with batch processing in Rust.

## What is Spring Batch RS?

Spring Batch RS is a comprehensive toolkit for building enterprise-grade batch applications in Rust. Inspired by the robust Java Spring Batch framework, it brings battle-tested concepts to the Rust ecosystem with the added benefits of Rust's performance and memory safety.

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70+** installed ([Install Rust](https://rustup.rs/))
- Basic familiarity with Rust programming
- Understanding of batch processing concepts (helpful but not required)

## Installation

Add Spring Batch RS to your `Cargo.toml`:

```toml
[dependencies]
spring-batch-rs = "0.3"
```

### Feature Flags

Spring Batch RS uses feature flags to keep your dependencies minimal. Enable only what you need:

```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json", "xml"] }
```

Available features:

| Feature         | Description                        |
| --------------- | ---------------------------------- |
| `csv`           | CSV file reading and writing       |
| `json`          | JSON file reading and writing      |
| `xml`           | XML file reading and writing       |
| `mongodb`       | MongoDB database integration       |
| `rdbc-postgres` | PostgreSQL database integration    |
| `rdbc-mysql`    | MySQL/MariaDB database integration |
| `rdbc-sqlite`   | SQLite database integration        |
| `orm`           | SeaORM integration                 |
| `zip`           | ZIP compression tasklets           |
| `ftp`           | FTP file transfer tasklets         |
| `fake`          | Mock data generation               |
| `logger`        | Debug logging writer               |
| `full`          | All features enabled               |

## Your First Batch Job

Let's create a simple batch job that reads CSV data and converts it to JSON:

### 1. Create a New Project

```bash
cargo new my-batch-app
cd my-batch-app
```

### 2. Add Dependencies

```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json"] }
serde = { version = "1.0", features = ["derive"] }
```

### 3. Define Your Data Structure

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
}
```

### 4. Create Your First Job

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};

fn main() -> Result<(), BatchError> {
    // Sample CSV data
    let csv_data = r#"id,name,price,category
1,Laptop,999.99,Electronics
2,Coffee Mug,12.99,Kitchen
3,Notebook,5.99,Office
4,Wireless Mouse,29.99,Electronics"#;

    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // Create processor (pass-through in this case)
    let processor = PassThroughProcessor::<Product>::new();

    // Build the step
    let step = StepBuilder::new("csv-to-json-step")
        .chunk(10)  // Process 10 items at a time
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // Build and run the job
    let job = JobBuilder::new()
        .start(&step)
        .build();

    // Execute the job
    let result = job.run()?;

    println!("Job completed successfully!");
    println!("Processed {} items", result.get_step_executions().len());

    Ok(())
}
```

### 5. Run Your Job

```bash
cargo run
```

You should see output similar to:

```
Job completed successfully!
Processed 1 items
```

And a `products.json` file will be created with your converted data.

## Core Concepts

Understanding these core concepts will help you build more complex batch applications:

### Job

A **Job** represents the entire batch process. It's composed of one or more steps that execute in sequence.

```rust
let job = JobBuilder::new()
    .start(&step1)
    .next(&step2)
    .next(&step3)
    .build();
```

### Step

A **Step** is an independent phase of a batch job. There are two types:

1. **Chunk-oriented steps**: Read-process-write pattern for large datasets
2. **Tasklet steps**: Single operations like file transfers or cleanup

```rust
// Chunk-oriented step
let chunk_step = StepBuilder::new("process-data")
    .chunk(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .build();

// Tasklet step
let tasklet_step = StepBuilder::new("cleanup")
    .tasklet(&cleanup_tasklet)
    .build();
```

### ItemReader

An **ItemReader** retrieves input data one item at a time:

```rust
// CSV reader
let csv_reader = CsvItemReaderBuilder::<Product>::new()
    .from_path("input.csv")?;

// Database reader
let db_reader = OrmItemReaderBuilder::<Product>::new()
    .connection(&db)
    .query(Product::find())
    .page_size(100)
    .build();
```

### ItemProcessor

An **ItemProcessor** applies business logic to transform items:

```rust
use spring_batch_rs::core::item::ItemProcessor;

struct PriceProcessor;

impl ItemProcessor<Product, Product> for PriceProcessor {
    fn process(&self, item: Product) -> Result<Option<Product>, BatchError> {
        let mut product = item;
        // Apply 10% discount
        product.price *= 0.9;
        Ok(Some(product))
    }
}
```

### ItemWriter

An **ItemWriter** outputs processed items:

```rust
// JSON writer
let json_writer = JsonItemWriterBuilder::new()
    .from_path("output.json")?;

// Database writer
let db_writer = OrmItemWriterBuilder::<Product>::new()
    .connection(&db)
    .build();
```

## Error Handling

Spring Batch RS provides robust error handling with configurable skip limits:

```rust
let step = StepBuilder::new("fault-tolerant-step")
    .chunk(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .skip_limit(10)  // Skip up to 10 errors
    .build();
```

## Next Steps

Now that you have the basics down, explore more advanced features:

1. **[Processing Models](./processing-models)** - Learn about chunk-oriented vs tasklet processing
2. **[Item Readers & Writers](./item-readers-writers)** - Explore all available data sources
3. **[Tasklets](./tasklets)** - File operations, FTP transfers, and custom tasks
4. **[Examples](./examples)** - Real-world examples and patterns
5. **[Tutorials](./tutorials)** - Step-by-step guides for common scenarios

## Need Help?

- 📖 **[API Documentation](https://docs.rs/spring-batch-rs)** - Complete API reference
- 💬 **[Discord](https://discord.gg/9FNhawNsG6)** - Chat with the community
- 🐛 **[GitHub Issues](https://github.com/sboussekeyt/spring-batch-rs/issues)** - Report bugs or request features
- 💡 **[GitHub Discussions](https://github.com/sboussekeyt/spring-batch-rs/discussions)** - Ask questions and share ideas

Happy batch processing! 🚀
