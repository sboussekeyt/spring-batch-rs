---
title: MongoDB Processing
description: Examples for reading from and writing to MongoDB with Spring Batch RS
sidebar:
  order: 5
---

Spring Batch RS provides MongoDB support through the synchronous MongoDB driver. Read with filters and pagination, write with batch inserts, and integrate with other data formats.

:::note
MongoDB examples require a running MongoDB instance at `localhost:27017`.
:::

## Quick Start

```rust
use spring_batch_rs::item::mongodb::{
    MongodbItemReaderBuilder, MongodbItemWriterBuilder, WithObjectId
};
use mongodb::bson::{doc, oid::ObjectId};

// Read with filter and pagination
let reader = MongodbItemReaderBuilder::new()
    .collection(&collection)
    .filter(doc! { "status": "active" })
    .page_size(100)
    .build();

// Write to collection
let writer = MongodbItemWriterBuilder::new()
    .collection(&collection)
    .build();
```

## Features

- **Synchronous API**: Uses MongoDB sync driver for batch processing
- **Query filters**: Filter documents using BSON queries
- **Pagination**: Efficient cursor-based pagination for large collections
- **Batch inserts**: Optimized writing with unordered inserts
- **Format conversion**: Export to CSV, JSON, and other formats

## Complete Example

The [`mongodb_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/mongodb_processing.rs) example demonstrates:

1. **Read all documents**: Export entire collection to JSON
2. **Read with filter**: Query by field and export to CSV
3. **Import from CSV**: Insert documents from CSV file
4. **Complex queries**: Filter by numeric ranges

### Prerequisites

```bash
# Start MongoDB locally using Docker
docker run -d -p 27017:27017 --name mongodb mongo:latest
```

### Run the Example

```bash
cargo run --example mongodb_processing --features mongodb,csv,json
```

## API Reference

### MongodbItemReaderBuilder

| Method | Description |
|--------|-------------|
| `collection(&Collection<T>)` | Set MongoDB collection (required) |
| `filter(Document)` | Set query filter (optional) |
| `page_size(i64)` | Set pagination size (optional) |
| `build()` | Build the reader |

### MongodbItemWriterBuilder

| Method | Description |
|--------|-------------|
| `collection(&Collection<T>)` | Set MongoDB collection (required) |
| `build()` | Build the writer |

## WithObjectId Trait

Your document types must implement `WithObjectId` for pagination:

```rust
use mongodb::bson::oid::ObjectId;
use spring_batch_rs::item::mongodb::WithObjectId;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Book {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "oid")]
    object_id: ObjectId,
    title: String,
    author: String,
}

impl WithObjectId for Book {
    fn get_id(&self) -> ObjectId {
        self.object_id
    }
}
```

## Common Patterns

### Reading with Complex Filters

```rust
use mongodb::bson::doc;

// Filter by multiple conditions
let reader = MongodbItemReaderBuilder::new()
    .collection(&collection)
    .filter(doc! {
        "status": "active",
        "price": { "$gte": 100.0 },
        "category": { "$in": ["electronics", "books"] }
    })
    .page_size(50)
    .build();
```

### Exporting to CSV

```rust
// Convert MongoDB documents to CSV-friendly format
struct BookToCsvProcessor;

impl ItemProcessor<Book, BookCsv> for BookToCsvProcessor {
    fn process(&self, item: &Book) -> ItemProcessorResult<BookCsv> {
        Ok(Some(BookCsv {
            title: item.title.clone(),
            author: item.author.clone(),
            // Exclude ObjectId for CSV
        }))
    }
}

let step = StepBuilder::new("mongo-to-csv")
    .chunk::<Book, BookCsv>(100)
    .reader(&mongo_reader)
    .processor(&BookToCsvProcessor)
    .writer(&csv_writer)
    .build();
```

### Importing from CSV

```rust
// Convert CSV records to MongoDB documents
struct CsvToBookProcessor;

impl ItemProcessor<BookInput, Book> for CsvToBookProcessor {
    fn process(&self, item: &BookInput) -> ItemProcessorResult<Book> {
        let oid = ObjectId::new();
        Ok(Some(Book {
            id: Some(oid),
            object_id: oid,
            title: item.title.clone(),
            author: item.author.clone(),
        }))
    }
}

let step = StepBuilder::new("csv-to-mongo")
    .chunk::<BookInput, Book>(50)
    .reader(&csv_reader)
    .processor(&CsvToBookProcessor)
    .writer(&mongo_writer)
    .build();
```

### Date/Time Filtering

```rust
use mongodb::bson::doc;

// Filter documents by date range
let reader = MongodbItemReaderBuilder::new()
    .collection(&collection)
    .filter(doc! {
        "created_at": {
            "$gte": "2024-01-01T00:00:00Z",
            "$lt": "2024-12-31T23:59:59Z"
        }
    })
    .build();
```

## Connection Setup

```rust
use mongodb::sync::Client;

fn main() -> Result<(), BatchError> {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .map_err(|e| BatchError::ItemReader(e.to_string()))?;

    let db = client.database("mydb");
    let collection = db.collection::<Book>("books");

    // Use collection with readers/writers...
}
```

## See Also

- [Database Processing](/spring-batch-rs/examples/database/) - SQL database support
- [ORM Processing](/spring-batch-rs/examples/orm/) - SeaORM integration
- [JSON Processing](/spring-batch-rs/examples/json/) - Export to JSON
