---
title: JSON Processing
description: Examples for reading and writing JSON files with Spring Batch RS
sidebar:
  order: 2
---

Spring Batch RS provides streaming JSON support for processing large JSON arrays efficiently. The reader processes objects one at a time without loading the entire file into memory.

## Quick Start

```rust
use spring_batch_rs::item::json::{JsonItemReaderBuilder, JsonItemWriterBuilder};

// Read JSON array
let reader = JsonItemReaderBuilder::<Order>::new()
    .from_reader(file);

// Write JSON with pretty formatting
let writer = JsonItemWriterBuilder::<Order>::new()
    .pretty_formatter(true)
    .from_path("output.json");
```

## Features

- **Streaming parser**: Memory-efficient processing of large JSON arrays
- **Pretty formatting**: Optional indented output for readability
- **Format conversion**: Convert to/from CSV, XML, and other formats
- **Type safety**: Automatic serialization/deserialization with serde

## Complete Example

The [`json_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/json_processing.rs) example demonstrates:

1. **Read JSON array**: Parse and log JSON objects
2. **JSON transformation**: Apply business logic during processing
3. **JSON to CSV export**: Convert orders to summary CSV
4. **Formatting options**: Compare compact vs pretty output

### Run the Example

```bash
cargo run --example json_processing --features json,csv,logger
```

## API Reference

### JsonItemReaderBuilder

| Method | Description |
|--------|-------------|
| `capacity(usize)` | Set internal buffer size (default: 8192 bytes) |
| `from_reader(R)` | Create reader from any `Read` source |

### JsonItemWriterBuilder

| Method | Description |
|--------|-------------|
| `pretty_formatter(bool)` | Enable indented output (default: `false`) |
| `indent(&[u8])` | Set custom indentation (default: 2 spaces) |
| `from_writer(W)` | Create writer for any `Write` destination |
| `from_path(P)` | Create writer to file path |

## Input Format

The JSON reader expects an array of objects:

```json
[
    {"id": 1, "name": "Alice", "total": 99.99},
    {"id": 2, "name": "Bob", "total": 149.50},
    {"id": 3, "name": "Charlie", "total": 75.00}
]
```

## Common Patterns

### Processing with Transformation

```rust
struct TaxProcessor { rate: f64 }

impl ItemProcessor<Order, Order> for TaxProcessor {
    fn process(&self, item: &Order) -> ItemProcessorResult<Order> {
        Ok(Some(Order {
            total: item.total * (1.0 + self.rate),
            ..item.clone()
        }))
    }
}
```

### Reading from In-Memory String

```rust
use std::io::Cursor;

let json_data = r#"[{"id": 1, "name": "test"}]"#;
let reader = JsonItemReaderBuilder::<Record>::new()
    .from_reader(Cursor::new(json_data));
```

### Multi-Format Pipeline

```rust
use spring_batch_rs::core::item::PassThroughProcessor;

// Step 1: JSON to intermediate format
let processor1 = PassThroughProcessor::<Internal>::new();
let step1 = StepBuilder::new("json-to-internal")
    .chunk::<Internal, Internal>(100)
    .reader(&json_reader)
    .processor(&processor1)
    .writer(&internal_writer)
    .build();

// Step 2: Internal to CSV export
let processor2 = PassThroughProcessor::<Internal>::new();
let step2 = StepBuilder::new("internal-to-csv")
    .chunk::<Internal, Internal>(100)
    .reader(&internal_reader)
    .processor(&processor2)
    .writer(&csv_writer)
    .build();

let job = JobBuilder::new()
    .start(&step1)
    .next(&step2)
    .build();
```

## See Also

- [CSV Processing](/spring-batch-rs/examples/csv/) - Convert JSON to CSV
- [XML Processing](/spring-batch-rs/examples/xml/) - Convert JSON to XML
- [Advanced Patterns](/spring-batch-rs/examples/advanced-patterns/) - Complex ETL pipelines
