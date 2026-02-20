# Unified RDBC Builders

This document explains how to use the unified builder API for RDBC readers and writers.

## Overview

The Spring Batch RS library now provides unified builders that allow you to create database readers and writers with a consistent API across PostgreSQL, MySQL, and SQLite. This simplifies the code and makes it easier to switch between database types.

## Key Benefits

1. **Consistent API**: Same builder pattern for all database types
2. **Type Safety**: Compile-time validation of database-specific types
3. **Clear Intent**: Database type is explicitly specified in the builder
4. **Easy Migration**: Switch database types by changing a single method call

## RdbcItemReaderBuilder

The `RdbcItemReaderBuilder` provides a unified way to create database readers.

### PostgreSQL Example

```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn create_postgres_reader() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;

    let reader = RdbcItemReaderBuilder::<User>::new()
        .postgres(pool)                              // Specify PostgreSQL
        .query("SELECT id, name, email FROM users")  // SQL query
        .with_page_size(100)                         // Optional pagination
        .build_postgres();                           // Build PostgreSQL reader

    Ok(())
}
```

### MySQL Example

```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;
use sqlx::MySqlPool;

async fn create_mysql_reader() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;

    let reader = RdbcItemReaderBuilder::<User>::new()
        .mysql(pool)                                 // Specify MySQL
        .query("SELECT id, name, email FROM users")  // SQL query
        .with_page_size(100)                         // Optional pagination
        .build_mysql();                              // Build MySQL reader

    Ok(())
}
```

### SQLite Example

```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;
use sqlx::SqlitePool;

async fn create_sqlite_reader() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let reader = RdbcItemReaderBuilder::<User>::new()
        .sqlite(pool)                                // Specify SQLite
        .query("SELECT id, name, email FROM users")  // SQL query
        .with_page_size(100)                         // Optional pagination
        .build_sqlite();                             // Build SQLite reader

    Ok(())
}
```

## RdbcItemWriterBuilder

The `RdbcItemWriterBuilder` provides a unified way to create database writers.

### PostgreSQL Example

```rust
use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
use sqlx::{PgPool, query_builder::Separated, Postgres};

struct UserBinder;
impl DatabaseItemBinder<User, Postgres> for UserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(&item.name);
        query_builder.push_bind(&item.email);
    }
}

async fn create_postgres_writer() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    let binder = UserBinder;

    let writer = RdbcItemWriterBuilder::<User>::new()
        .postgres(&pool)           // Specify PostgreSQL
        .table("users")            // Target table
        .add_column("id")          // Column names
        .add_column("name")
        .add_column("email")
        .postgres_binder(&binder)  // Database-specific binder
        .build_postgres();         // Build PostgreSQL writer

    Ok(())
}
```

### MySQL Example

```rust
use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
use sqlx::{MySqlPool, query_builder::Separated, MySql};

struct UserBinder;
impl DatabaseItemBinder<User, MySql> for UserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(&item.name);
        query_builder.push_bind(&item.email);
    }
}

async fn create_mysql_writer() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
    let binder = UserBinder;

    let writer = RdbcItemWriterBuilder::<User>::new()
        .mysql(&pool)              // Specify MySQL
        .table("users")            // Target table
        .add_column("id")          // Column names
        .add_column("name")
        .add_column("email")
        .mysql_binder(&binder)     // Database-specific binder
        .build_mysql();            // Build MySQL writer

    Ok(())
}
```

### SQLite Example

```rust
use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
use sqlx::{SqlitePool, query_builder::Separated, Sqlite};

struct UserBinder;
impl DatabaseItemBinder<User, Sqlite> for UserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Sqlite, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(&item.name);
        query_builder.push_bind(&item.email);
    }
}

async fn create_sqlite_writer() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    let binder = UserBinder;

    let writer = RdbcItemWriterBuilder::<User>::new()
        .sqlite(&pool)             // Specify SQLite
        .table("users")            // Target table
        .add_column("id")          // Column names
        .add_column("name")
        .add_column("email")
        .sqlite_binder(&binder)    // Database-specific binder
        .build_sqlite();           // Build SQLite writer

    Ok(())
}
```

## Migration from Database-Specific Builders

### Before (Database-Specific Builder)

```rust
// PostgreSQL-specific
use spring_batch_rs::item::rdbc::PostgresRdbcItemReaderBuilder;

let reader = PostgresRdbcItemReaderBuilder::<User>::new()
    .pool(pg_pool)
    .query("SELECT * FROM users")
    .with_page_size(100)
    .build();
```

### After (Unified Builder)

```rust
// Unified builder with explicit database type
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

let reader = RdbcItemReaderBuilder::<User>::new()
    .postgres(pg_pool)              // Database type specified here
    .query("SELECT * FROM users")
    .with_page_size(100)
    .build_postgres();              // And here
```

## Key Differences

1. **Database Type Parameter**: The unified builder uses method chaining to specify the database type (`.postgres()`, `.mysql()`, `.sqlite()`)
2. **Build Method**: Each database has its own build method (`.build_postgres()`, `.build_mysql()`, `.build_sqlite()`)
3. **Binder Methods**: Writers use database-specific binder methods (`.postgres_binder()`, `.mysql_binder()`, `.sqlite_binder()`)

## Advantages Over Database-Specific Builders

1. **Consistency**: Same API pattern across all databases
2. **Discoverability**: All database options available from single import
3. **Maintainability**: Easier to switch databases without changing entire code structure
4. **Clarity**: Database type is explicit in the builder chain

## When to Use Each Approach

### Use Unified Builders When:
- Building applications that might support multiple databases
- Creating reusable components that work across database types
- Preferring consistent API patterns

### Use Database-Specific Builders When:
- Working exclusively with one database type
- Needing database-specific optimizations
- Preferring specialized type names

Both approaches are fully supported and produce identical results. The unified builders are built on top of the database-specific builders and provide an alternative, more consistent API.
