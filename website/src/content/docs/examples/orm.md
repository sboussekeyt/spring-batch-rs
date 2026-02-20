---
title: ORM Processing (SeaORM)
description: Examples for using SeaORM with Spring Batch RS
sidebar:
  order: 6
---

Spring Batch RS integrates with SeaORM for type-safe ORM-based database operations. Use SeaORM's powerful query builder with batch processing capabilities.

## Quick Start

```rust
use spring_batch_rs::item::orm::{OrmItemReaderBuilder, OrmItemWriterBuilder};
use sea_orm::{Database, EntityTrait, QueryFilter};

// Read with SeaORM query
let query = products::Entity::find()
    .filter(products::Column::Active.eq(true));

let reader = OrmItemReaderBuilder::new()
    .connection(&db)
    .query(query)
    .page_size(100)
    .build();

// Write active models directly
let writer = OrmItemWriterBuilder::<products::ActiveModel>::new()
    .connection(&db)
    .build();
```

## Features

- **Type-safe queries**: SeaORM's compile-time checked queries
- **Pagination**: Efficient page-based reading
- **Direct entity writing**: Write active models without mappers
- **All SeaORM databases**: PostgreSQL, MySQL, SQLite, SQL Server
- **Async-to-sync bridge**: Works with batch framework's sync API

## Complete Example

The [`orm_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/orm_processing.rs) example uses SQLite in-memory (no external database required) and demonstrates:

1. **Read all products**: Export to JSON with pagination
2. **Filtered queries**: Query by category and stock status
3. **Complex filters**: Price-based filtering
4. **Write entities**: Insert new records from DTOs
5. **Verify writes**: Read back and export new records

### Run the Example

```bash
cargo run --example orm_processing --features orm,csv,json
```

## API Reference

### OrmItemReaderBuilder

| Method | Description |
|--------|-------------|
| `connection(&DatabaseConnection)` | Set database connection (required) |
| `query(Select<E>)` | Set SeaORM select query (required) |
| `page_size(u64)` | Set pagination size (optional) |
| `build()` | Build the reader |

### OrmItemWriterBuilder

| Method | Description |
|--------|-------------|
| `connection(&DatabaseConnection)` | Set database connection (required) |
| `build()` | Build the writer |

## Defining Entities

Use SeaORM's derive macros to define entities:

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub category: String,
    pub price: f64,
    pub in_stock: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

## Common Patterns

### Reading with Filters

```rust
use sea_orm::{EntityTrait, QueryFilter, QueryOrder};

let query = products::Entity::find()
    .filter(products::Column::Category.eq("Electronics"))
    .filter(products::Column::InStock.eq(true))
    .order_by_asc(products::Column::Name);

let reader = OrmItemReaderBuilder::new()
    .connection(&db)
    .query(query)
    .page_size(50)
    .build();
```

### Writing from DTOs

Convert business DTOs to SeaORM active models:

```rust
use sea_orm::ActiveValue::Set;

struct DtoToActiveModelProcessor;

impl ItemProcessor<ProductDto, products::ActiveModel> for DtoToActiveModelProcessor {
    fn process(&self, item: &ProductDto) -> Result<products::ActiveModel, BatchError> {
        Ok(products::ActiveModel {
            id: Set(item.id),
            name: Set(item.name.clone()),
            category: Set(item.category.clone()),
            price: Set(item.price),
            in_stock: Set(item.in_stock),
        })
    }
}

let step = StepBuilder::new("import-products")
    .chunk::<ProductDto, products::ActiveModel>(50)
    .reader(&dto_reader)
    .processor(&DtoToActiveModelProcessor)
    .writer(&orm_writer)
    .build();
```

### Complex Queries with Joins

```rust
// Query with related entities
let query = orders::Entity::find()
    .find_also_related(customers::Entity)
    .filter(orders::Column::Status.eq("pending"));
```

### Reading to CSV Export

```rust
// Convert SeaORM Model to CSV-friendly struct
struct ModelToCsvProcessor;

impl ItemProcessor<products::Model, ProductCsv> for ModelToCsvProcessor {
    fn process(&self, item: &products::Model) -> Result<ProductCsv, BatchError> {
        Ok(ProductCsv {
            id: item.id,
            name: item.name.clone(),
            price: item.price,
        })
    }
}

let step = StepBuilder::new("export-csv")
    .chunk::<products::Model, ProductCsv>(100)
    .reader(&orm_reader)
    .processor(&ModelToCsvProcessor)
    .writer(&csv_writer)
    .build();
```

## Database Connection Setup

```rust
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), BatchError> {
    // SQLite in-memory
    let db = Database::connect("sqlite::memory:").await?;

    // PostgreSQL
    // let db = Database::connect("postgresql://user:pass@localhost/db").await?;

    // MySQL
    // let db = Database::connect("mysql://user:pass@localhost/db").await?;

    // Create tables...
    db.execute_unprepared("CREATE TABLE products (...)").await?;

    // Use with readers/writers
}
```

## In-Memory Reader for DTOs

For reading from in-memory collections:

```rust
struct InMemoryReader<T> {
    items: RefCell<VecDeque<T>>,
}

impl<T: Clone> InMemoryReader<T> {
    fn new(items: Vec<T>) -> Self {
        Self { items: RefCell::new(items.into()) }
    }
}

impl<T: Clone> ItemReader<T> for InMemoryReader<T> {
    fn read(&self) -> Result<Option<T>, BatchError> {
        Ok(self.items.borrow_mut().pop_front())
    }
}
```

## See Also

- [Database Processing](/spring-batch-rs/examples/database/) - Raw SQL with RDBC
- [MongoDB Processing](/spring-batch-rs/examples/mongodb/) - NoSQL support
- [Advanced Patterns](/spring-batch-rs/examples/advanced-patterns/) - Complex ETL pipelines
