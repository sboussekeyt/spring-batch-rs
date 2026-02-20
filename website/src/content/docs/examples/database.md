---
title: Database Processing
description: Examples for reading from and writing to databases with Spring Batch RS
sidebar:
  order: 4
---

Spring Batch RS provides unified database support for PostgreSQL, MySQL, and SQLite through the RDBC module. Read with pagination, write with batch inserts, and use type-safe item binders.

## Quick Start

```rust
use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, RdbcItemWriterBuilder};
use sqlx::SqlitePool;

// Read from database with pagination
let reader = RdbcItemReaderBuilder::<User>::new()
    .sqlite(pool.clone())
    .query("SELECT id, name, email FROM users")
    .with_page_size(100)
    .build_sqlite();

// Write to database with custom binder
let writer = RdbcItemWriterBuilder::<User>::new()
    .sqlite(&pool)
    .table("users")
    .add_column("id")
    .add_column("name")
    .add_column("email")
    .sqlite_binder(&binder)
    .build_sqlite();
```

## Features

- **Multi-database support**: PostgreSQL, MySQL, SQLite with unified API
- **Pagination**: Efficient reading of large datasets
- **Batch inserts**: Optimized writing with configurable chunk sizes
- **Type-safe binders**: Compile-time checked parameter binding
- **Async runtime**: Built on tokio with sqlx

## Complete Example

The [`database_processing`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/database_processing.rs) example uses SQLite (no external database required) and demonstrates:

1. **Read from database**: Query with pagination and logging
2. **Export to JSON**: Database to JSON file conversion
3. **Export to CSV**: Database to CSV file conversion
4. **Import from CSV**: CSV to database insertion
5. **Transform and write**: Read, process, and write back

### Run the Example

```bash
cargo run --example database_processing --features rdbc-sqlite,csv,json,logger
```

## API Reference

### RdbcItemReaderBuilder

| Method | Description |
|--------|-------------|
| `postgres(pool)` | Set PostgreSQL connection pool |
| `mysql(pool)` | Set MySQL connection pool |
| `sqlite(pool)` | Set SQLite connection pool |
| `query(sql)` | Set SQL query (without LIMIT/OFFSET) |
| `with_page_size(n)` | Set pagination size |
| `build_postgres()` | Build PostgreSQL reader |
| `build_mysql()` | Build MySQL reader |
| `build_sqlite()` | Build SQLite reader |

### RdbcItemWriterBuilder

| Method | Description |
|--------|-------------|
| `postgres(&pool)` | Set PostgreSQL connection pool |
| `mysql(&pool)` | Set MySQL connection pool |
| `sqlite(&pool)` | Set SQLite connection pool |
| `table(name)` | Set target table name |
| `add_column(name)` | Add column to INSERT statement |
| `postgres_binder(&binder)` | Set PostgreSQL binder |
| `mysql_binder(&binder)` | Set MySQL binder |
| `sqlite_binder(&binder)` | Set SQLite binder |

## Creating Item Binders

Binders map your struct fields to SQL parameters:

```rust
use spring_batch_rs::item::rdbc::DatabaseItemBinder;
use sqlx::{query_builder::Separated, Sqlite};

struct UserBinder;

impl DatabaseItemBinder<User, Sqlite> for UserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Sqlite, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.clone());
        query_builder.push_bind(item.email.clone());
    }
}
```

## Database-Specific Features

### PostgreSQL

```rust
use sqlx::PgPool;

let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;

let reader = RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)
    .query("SELECT * FROM users WHERE active = true")
    .with_page_size(100)
    .build_postgres();
```

### MySQL

```rust
use sqlx::MySqlPool;

let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;

let reader = RdbcItemReaderBuilder::<Product>::new()
    .mysql(pool)
    .query("SELECT * FROM products")
    .build_mysql();
```

### SQLite (In-Memory)

```rust
use sqlx::SqlitePool;

let pool = SqlitePool::connect("sqlite::memory:").await?;

let reader = RdbcItemReaderBuilder::<Task>::new()
    .sqlite(pool)
    .query("SELECT * FROM tasks")
    .build_sqlite();
```

## Common Patterns

### CSV Import to Database

```rust
let csv_reader = CsvItemReaderBuilder::<Product>::new()
    .has_headers(true)
    .from_path("products.csv");

let db_writer = RdbcItemWriterBuilder::<Product>::new()
    .sqlite(&pool)
    .table("products")
    .add_column("id")
    .add_column("name")
    .add_column("price")
    .sqlite_binder(&binder)
    .build_sqlite();

let step = StepBuilder::new("csv-to-db")
    .chunk::<Product, Product>(100)
    .reader(&csv_reader)
    .writer(&db_writer)
    .build();
```

### Database Export to JSON

```rust
let db_reader = RdbcItemReaderBuilder::<User>::new()
    .sqlite(pool.clone())
    .query("SELECT * FROM users WHERE active = 1")
    .build_sqlite();

let json_writer = JsonItemWriterBuilder::<User>::new()
    .pretty_formatter(true)
    .from_path("active_users.json");

let step = StepBuilder::new("db-to-json")
    .chunk::<User, User>(50)
    .reader(&db_reader)
    .writer(&json_writer)
    .build();
```

## See Also

- [ORM Processing](/spring-batch-rs/examples/orm/) - SeaORM integration
- [MongoDB Processing](/spring-batch-rs/examples/mongodb/) - NoSQL database support
- [CSV Processing](/spring-batch-rs/examples/csv/) - Import/export CSV files
