---
title: CSV Processing
description: Examples for reading and writing CSV files with Spring Batch RS
sidebar:
  order: 1
---

CSV (Comma-Separated Values) is one of the most common data formats for batch processing. Spring Batch RS provides powerful tools for reading and writing CSV files with full support for headers, custom delimiters, and data transformation.

## Quick Start

```rust
use spring_batch_rs::item::csv::{CsvItemReaderBuilder, CsvItemWriterBuilder};

// Read CSV with headers
let reader = CsvItemReaderBuilder::<Product>::new()
    .has_headers(true)
    .from_path("products.csv");

// Write CSV with headers
let writer = CsvItemWriterBuilder::<Product>::new()
    .has_headers(true)
    .from_path("output.csv");
```

## Features

- **Header handling**: Automatic header detection and generation
- **Custom delimiters**: Support for comma, semicolon, tab, and any custom delimiter
- **Data transformation**: Apply processors to transform data during processing
- **Fault tolerance**: Skip invalid records with configurable limits
- **Type safety**: Automatic deserialization into Rust structs using serde

## Complete Example

The [`csv_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/csv_processing.rs) example demonstrates:

1. **Basic CSV to CSV**: Copy CSV data with pass-through processing
2. **CSV to JSON with transformation**: Apply discounts during conversion
3. **Custom delimiters**: Process semicolon-separated files
4. **Fault tolerance**: Handle malformed records gracefully

### Run the Example

```bash
cargo run --example csv_processing --features csv,json
```

## API Reference

### CsvItemReaderBuilder

| Method | Description |
|--------|-------------|
| `has_headers(bool)` | Enable/disable header row parsing (default: `false`) |
| `delimiter(u8)` | Set field delimiter (default: `,`) |
| `from_reader(R)` | Create reader from any `Read` source |
| `from_path(P)` | Create reader from file path |

### CsvItemWriterBuilder

| Method | Description |
|--------|-------------|
| `has_headers(bool)` | Include header row in output (default: `false`) |
| `delimiter(u8)` | Set field delimiter (default: `,`) |
| `from_writer(W)` | Create writer for any `Write` destination |
| `from_path(P)` | Create writer to file path |

## Common Patterns

### Reading with Custom Delimiter

```rust
// European CSV format with semicolons
let reader = CsvItemReaderBuilder::<Record>::new()
    .has_headers(true)
    .delimiter(b';')
    .from_path("data.csv");
```

### Writing Without Headers

```rust
// Raw data export without header row
let writer = CsvItemWriterBuilder::<Record>::new()
    .has_headers(false)
    .from_path("export.csv");
```

### Error Handling with Skip Limit

```rust
let step = StepBuilder::new("csv-step")
    .chunk::<Input, Output>(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .skip_limit(10)  // Allow up to 10 parsing errors
    .build();
```

## See Also

- [JSON Processing](/spring-batch-rs/examples/json/) - Convert CSV to JSON
- [Database Processing](/spring-batch-rs/examples/database/) - Import CSV to database
- [Advanced Patterns](/spring-batch-rs/examples/advanced-patterns/) - Multi-step ETL with CSV
