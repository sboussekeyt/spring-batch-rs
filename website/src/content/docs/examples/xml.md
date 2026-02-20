---
title: XML Processing
description: Examples for reading and writing XML files with Spring Batch RS
sidebar:
  order: 3
---

Spring Batch RS provides XML processing capabilities using the `quick-xml` crate. Read XML elements by tag name and write structured XML documents with custom root and item tags.

## Quick Start

```rust
use spring_batch_rs::item::xml::{XmlItemReaderBuilder, XmlItemWriterBuilder};

// Read XML elements by tag
let reader = XmlItemReaderBuilder::<Book>::new()
    .tag("book")
    .from_reader(file);

// Write XML with custom structure
let writer = XmlItemWriterBuilder::<Book>::new()
    .root_tag("catalog")
    .item_tag("book")
    .from_path("output.xml");
```

## Features

- **Tag-based reading**: Extract elements by XML tag name
- **Attribute support**: Handle XML attributes with serde
- **Nested elements**: Process complex XML hierarchies
- **Custom output structure**: Define root and item tags for writing
- **Format conversion**: Convert to/from JSON, CSV

## Complete Example

The [`xml_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/xml_processing.rs) example demonstrates:

1. **XML to JSON**: Convert a book catalog to JSON format
2. **Create XML**: Generate XML from JSON data
3. **XML to CSV**: Export XML data as CSV with transformation
4. **XML to XML**: Transform XML structure

### Run the Example

```bash
cargo run --example xml_processing --features xml,json,csv
```

## API Reference

### XmlItemReaderBuilder

| Method | Description |
|--------|-------------|
| `tag(name)` | Set the XML element tag to search for (required) |
| `capacity(usize)` | Set buffer capacity (default: 1024 bytes) |
| `from_reader(R)` | Create reader from any `Read` source |

### XmlItemWriterBuilder

| Method | Description |
|--------|-------------|
| `root_tag(name)` | Set root element tag (required) |
| `item_tag(name)` | Set item element tag (required) |
| `from_writer(W)` | Create writer for any `Write` destination |
| `from_path(P)` | Create writer to file path |

## Working with Attributes

Use serde's rename attribute with `@` prefix for XML attributes:

```rust
#[derive(Deserialize, Serialize)]
struct Book {
    #[serde(rename = "@id")]  // XML attribute
    id: u32,
    title: String,           // XML element
    author: String,          // XML element
}
```

This handles XML like:

```xml
<book id="1">
    <title>The Rust Programming Language</title>
    <author>Steve Klabnik</author>
</book>
```

## Input Format

The XML reader extracts elements matching the specified tag:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<catalog>
    <book id="1">
        <title>Clean Code</title>
        <author>Robert Martin</author>
    </book>
    <book id="2">
        <title>Design Patterns</title>
        <author>Gang of Four</author>
    </book>
</catalog>
```

## Common Patterns

### Reading Nested XML

```rust
#[derive(Deserialize)]
struct House {
    #[serde(rename = "@id")]
    id: u32,
    address: String,
    city: String,
    rooms: Vec<Room>,
}

#[derive(Deserialize)]
struct Room {
    name: String,
    size: f64,
}
```

### Converting XML to CSV

```rust
// Processor to flatten XML structure for CSV
struct XmlToCsvProcessor;

impl ItemProcessor<House, HouseCsv> for XmlToCsvProcessor {
    fn process(&self, item: &House) -> Result<HouseCsv, BatchError> {
        Ok(HouseCsv {
            id: item.id,
            address: item.address.clone(),
            city: item.city.clone(),
            room_count: item.rooms.len() as u32,
        })
    }
}
```

### Writing Custom XML Structure

```rust
let writer = XmlItemWriterBuilder::<Person>::new()
    .root_tag("people")
    .item_tag("person")
    .from_path("output.xml");

// Produces:
// <people>
//   <person>...</person>
//   <person>...</person>
// </people>
```

## See Also

- [JSON Processing](/spring-batch-rs/examples/json/) - Convert XML to JSON
- [CSV Processing](/spring-batch-rs/examples/csv/) - Convert XML to CSV
- [Advanced Patterns](/spring-batch-rs/examples/advanced-patterns/) - Multi-format pipelines
