---
sidebar_position: 3
---

# Item Readers and Writers

Spring Batch RS provides a comprehensive set of item readers and writers for various data sources and formats. All readers and writers are built using the builder pattern for easy configuration.

## Features Overview

The crate is modular, allowing you to enable only the features you need:

| **Feature**     | **Description**                                                  |
| --------------- | ---------------------------------------------------------------- |
| `csv`           | Enables CSV `ItemReader` and `ItemWriter`                        |
| `json`          | Enables JSON `ItemReader` and `ItemWriter`                       |
| `xml`           | Enables XML `ItemReader` and `ItemWriter`                        |
| `mongodb`       | Enables `ItemReader` and `ItemWriter` for MongoDB databases      |
| `rdbc-postgres` | Enables RDBC `ItemReader` and `ItemWriter` for PostgreSQL        |
| `rdbc-mysql`    | Enables RDBC `ItemReader` and `ItemWriter` for MySQL and MariaDB |
| `rdbc-sqlite`   | Enables RDBC `ItemReader` and `ItemWriter` for SQLite            |
| `orm`           | Enables ORM `ItemReader` and `ItemWriter` using SeaORM           |
| `fake`          | Enables a fake `ItemReader`, useful for generating mock datasets |
| `logger`        | Enables a logger `ItemWriter`, useful for debugging purposes     |

## File-Based Readers and Writers

### CSV

Read and write CSV files with configurable delimiters and headers.

#### CSV Reader

```rust
use spring_batch_rs::item::csv::CsvItemReaderBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

// From file
let reader = CsvItemReaderBuilder::<Product>::new()
    .has_headers(true)
    .delimiter(b',')
    .from_path("products.csv");

// From string/bytes
let csv_data = "id,name,price\n1,Laptop,999.99";
let reader = CsvItemReaderBuilder::<Product>::new()
    .has_headers(true)
    .from_reader(csv_data.as_bytes());
```

#### CSV Writer

```rust
use spring_batch_rs::item::csv::CsvItemWriterBuilder;
use serde::Serialize;

#[derive(Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

let writer = CsvItemWriterBuilder::new()
    .has_headers(true)
    .delimiter(b',')
    .from_path("output.csv");
```

### JSON

Read and write JSON files with pretty printing support.

#### JSON Reader

```rust
use spring_batch_rs::item::json::JsonItemReaderBuilder;

// Read array of objects from file
let reader = JsonItemReaderBuilder::<Product>::new()
    .from_path("products.json");

// Read from string
let json_data = r#"[{"id":1,"name":"Laptop","price":999.99}]"#;
let reader = JsonItemReaderBuilder::<Product>::new()
    .from_reader(json_data.as_bytes());
```

#### JSON Writer

```rust
use spring_batch_rs::item::json::JsonItemWriterBuilder;

let writer = JsonItemWriterBuilder::new()
    .pretty_formatter(true)  // Enable pretty printing
    .from_path("output.json");
```

### XML

Read and write XML files with custom serialization.

#### XML Reader

```rust
use spring_batch_rs::item::xml::XmlItemReaderBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

let reader = XmlItemReaderBuilder::<Product>::new()
    .root_element("products")
    .item_element("product")
    .from_path("products.xml");
```

#### XML Writer

```rust
use spring_batch_rs::item::xml::XmlItemWriterBuilder;

let writer = XmlItemWriterBuilder::new()
    .root_element("products")
    .item_element("product")
    .pretty_formatter(true)
    .from_path("output.xml");
```

## Database Readers and Writers

### ORM (SeaORM)

Read and write using SeaORM with pagination and filtering support.

```rust
use spring_batch_rs::item::orm::OrmItemReaderBuilder;
use sea_orm::{Database, EntityTrait};

// Setup database connection
let db = Database::connect("sqlite::memory:").await?;

// Create query with filtering
let query = ProductEntity::find()
    .filter(product::Column::Active.eq(true))
    .order_by_asc(product::Column::Id);

let reader = OrmItemReaderBuilder::new()
    .connection(&db)
    .query(query)
    .page_size(100)
    .build();
```

### RDBC (Direct Database Access)

Direct database access for PostgreSQL, MySQL, and SQLite.

#### PostgreSQL

```rust
use spring_batch_rs::item::rdbc::postgres::PostgresItemReaderBuilder;

let reader = PostgresItemReaderBuilder::<Product>::new()
    .connection_string("postgresql://user:pass@localhost/db")
    .query("SELECT id, name, price FROM products WHERE active = true")
    .page_size(1000)
    .build().await?;
```

#### MySQL

```rust
use spring_batch_rs::item::rdbc::mysql::MysqlItemReaderBuilder;

let reader = MysqlItemReaderBuilder::<Product>::new()
    .connection_string("mysql://user:pass@localhost/db")
    .query("SELECT id, name, price FROM products")
    .build().await?;
```

#### SQLite

```rust
use spring_batch_rs::item::rdbc::sqlite::SqliteItemReaderBuilder;

let reader = SqliteItemReaderBuilder::<Product>::new()
    .connection_string("sqlite:products.db")
    .query("SELECT id, name, price FROM products")
    .build().await?;
```

### MongoDB

Native MongoDB document operations.

#### MongoDB Reader

```rust
use spring_batch_rs::item::mongodb::MongoItemReaderBuilder;
use mongodb::{Client, bson::doc};

let client = Client::with_uri_str("mongodb://localhost:27017").await?;
let db = client.database("mydb");
let collection = db.collection::<Product>("products");

let reader = MongoItemReaderBuilder::new()
    .collection(&collection)
    .filter(doc! { "active": true })
    .batch_size(100)
    .build();
```

#### MongoDB Writer

```rust
use spring_batch_rs::item::mongodb::MongoItemWriterBuilder;

let writer = MongoItemWriterBuilder::new()
    .collection(&collection)
    .upsert(true)  // Enable upsert mode
    .build();
```

## Utility Readers and Writers

### Fake Reader

Generate mock data for testing and development.

```rust
use spring_batch_rs::item::fake::person_reader::PersonReaderBuilder;

// Generate fake person data
let reader = PersonReaderBuilder::new()
    .number_of_items(1000)
    .locale("en_US")
    .build();

// Custom fake data
use spring_batch_rs::item::fake::FakeItemReaderBuilder;

let reader = FakeItemReaderBuilder::<Product>::new()
    .number_of_items(500)
    .generator(|| Product {
        id: rand::random(),
        name: fake::name::en::Name().fake(),
        price: rand::thread_rng().gen_range(10.0..1000.0),
    })
    .build();
```

### Logger Writer

Debug output for development and testing.

```rust
use spring_batch_rs::item::logger::LoggerItemWriterBuilder;

let writer = LoggerItemWriterBuilder::new()
    .log_level(log::Level::Info)
    .prefix("Processing item:")
    .build();
```

## Error Handling and Fault Tolerance

All readers and writers support configurable error handling:

```rust
use spring_batch_rs::core::step::StepBuilder;

let processor = PassThroughProcessor::<Product>::new();

let step = StepBuilder::new("fault_tolerant_step")
    .chunk::<Product, Product>(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .skip_limit(10)  // Skip up to 10 failed items
    .build();
```

## Performance Considerations

### Chunk Size

Choose appropriate chunk sizes based on your data and memory constraints:

```rust
// Small chunks for memory-constrained environments
.chunk(10)

// Medium chunks for balanced performance
.chunk(100)

// Large chunks for high-throughput scenarios
.chunk(1000)
```

### Database Pagination

For database readers, configure page sizes to optimize memory usage:

```rust
let reader = OrmItemReaderBuilder::new()
    .page_size(500)  // Fetch 500 records at a time
    .build();
```

### Buffered I/O

File-based readers and writers use buffered I/O by default for optimal performance.

## Custom Readers and Writers

You can create custom readers and writers by implementing the respective traits:

### Custom Reader

```rust
use spring_batch_rs::core::item::ItemReader;
use spring_batch_rs::BatchError;

struct CustomReader {
    data: Vec<String>,
    index: usize,
}

impl ItemReader<String> for CustomReader {
    fn read(&mut self) -> Result<Option<String>, BatchError> {
        if self.index < self.data.len() {
            let item = self.data[self.index].clone();
            self.index += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
```

### Custom Writer

```rust
use spring_batch_rs::core::item::ItemWriter;
use spring_batch_rs::BatchError;

struct CustomWriter {
    output: Vec<String>,
}

impl ItemWriter<String> for CustomWriter {
    fn write(&mut self, items: &[String]) -> Result<(), BatchError> {
        for item in items {
            self.output.push(item.clone());
            println!("Writing: {}", item);
        }
        Ok(())
    }
}
```

This comprehensive set of readers and writers allows you to build batch applications that work with virtually any data source or destination.
