# Migration Guide: Database-Specific Builders to Unified Builders

This guide explains how to migrate from the old database-specific builders to the new unified builder API.

## Overview

The database-specific builders (`PostgresRdbcItemReaderBuilder`, `MySqlRdbcItemReaderBuilder`, `SqliteRdbcItemReaderBuilder`) have been removed in favor of a unified `RdbcItemReaderBuilder` and `RdbcItemWriterBuilder` API that works consistently across all database types.

## Why the Change?

1. **Consistency**: Single API pattern for all database types
2. **Discoverability**: All database options available from one builder
3. **Maintainability**: Easier to add new features across all databases
4. **Clarity**: Database type is explicit in the builder chain

## Migration Steps

### Readers

#### Before (PostgreSQL)
```rust
use spring_batch_rs::item::rdbc::postgres_reader::PostgresRdbcItemReaderBuilder;

let reader = PostgresRdbcItemReaderBuilder::<User>::new()
    .pool(pool)
    .query("SELECT * FROM users")
    .with_page_size(100)
    .build();
```

#### After (PostgreSQL)
```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

let reader = RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)                    // Specify database type
    .query("SELECT * FROM users")
    .with_page_size(100)
    .build_postgres();                 // Build for PostgreSQL
```

#### Before (MySQL)
```rust
use spring_batch_rs::item::rdbc::mysql_reader::MySqlRdbcItemReaderBuilder;

let reader = MySqlRdbcItemReaderBuilder::<Product>::new()
    .pool(pool)
    .query("SELECT * FROM products")
    .with_page_size(100)
    .build();
```

#### After (MySQL)
```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

let reader = RdbcItemReaderBuilder::<Product>::new()
    .mysql(pool)                       // Specify database type
    .query("SELECT * FROM products")
    .with_page_size(100)
    .build_mysql();                    // Build for MySQL
```

#### Before (SQLite)
```rust
use spring_batch_rs::item::rdbc::sqlite_reader::SqliteRdbcItemReaderBuilder;

let reader = SqliteRdbcItemReaderBuilder::<Task>::new()
    .pool(pool)
    .query("SELECT * FROM tasks")
    .with_page_size(100)
    .build();
```

#### After (SQLite)
```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

let reader = RdbcItemReaderBuilder::<Task>::new()
    .sqlite(pool)                      // Specify database type
    .query("SELECT * FROM tasks")
    .with_page_size(100)
    .build_sqlite();                   // Build for SQLite
```

### Writers

The writers already use a builder-like API through method chaining on the writer struct itself. No changes are needed, but you can use the new `RdbcItemWriterBuilder` for a more consistent API.

#### Before (Direct Writer API)
```rust
use spring_batch_rs::item::rdbc::postgres_writer::PostgresItemWriter;

let writer = PostgresItemWriter::<User>::new()
    .pool(&pool)
    .table("users")
    .add_column("id")
    .add_column("name")
    .item_binder(&binder);
```

#### After (Unified Builder - Optional)
```rust
use spring_batch_rs::item::rdbc::RdbcItemWriterBuilder;

let writer = RdbcItemWriterBuilder::<User>::new()
    .postgres(&pool)
    .table("users")
    .add_column("id")
    .add_column("name")
    .postgres_binder(&binder)
    .build_postgres();
```

## Quick Reference

### Key Changes

| Old API | New API | Notes |
|---------|---------|-------|
| `PostgresRdbcItemReaderBuilder::new()` | `RdbcItemReaderBuilder::new()` | Import from `spring_batch_rs::item::rdbc` |
| `.pool(pool)` | `.postgres(pool)` | Database type specified in method name |
| `.build()` | `.build_postgres()` | Database type specified in build method |
| `MySqlRdbcItemReaderBuilder::new()` | `RdbcItemReaderBuilder::new()` | Same builder for all databases |
| `.pool(pool)` | `.mysql(pool)` | Database type specified in method name |
| `.build()` | `.build_mysql()` | Database type specified in build method |
| `SqliteRdbcItemReaderBuilder::new()` | `RdbcItemReaderBuilder::new()` | Same builder for all databases |
| `.pool(pool)` | `.sqlite(pool)` | Database type specified in method name |
| `.build()` | `.build_sqlite()` | Database type specified in build method |

### Import Changes

#### Before
```rust
use spring_batch_rs::item::rdbc::postgres_reader::PostgresRdbcItemReaderBuilder;
use spring_batch_rs::item::rdbc::mysql_reader::MySqlRdbcItemReaderBuilder;
use spring_batch_rs::item::rdbc::sqlite_reader::SqliteRdbcItemReaderBuilder;
```

#### After
```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;  // Single import for all databases
```

## Common Patterns

### Switching Between Databases

The unified builder makes it easy to switch between database types:

```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

// Configuration-driven database selection
let reader = match db_type {
    "postgres" => RdbcItemReaderBuilder::new()
        .postgres(pg_pool)
        .query(query)
        .build_postgres(),
    "mysql" => RdbcItemReaderBuilder::new()
        .mysql(mysql_pool)
        .query(query)
        .build_mysql(),
    "sqlite" => RdbcItemReaderBuilder::new()
        .sqlite(sqlite_pool)
        .query(query)
        .build_sqlite(),
    _ => panic!("Unsupported database type"),
};
```

### Testing with Multiple Databases

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

    #[test]
    fn test_postgres_reader() {
        let reader = RdbcItemReaderBuilder::<User>::new()
            .postgres(get_postgres_pool())
            .query("SELECT * FROM users")
            .build_postgres();
        // Test logic
    }

    #[test]
    fn test_mysql_reader() {
        let reader = RdbcItemReaderBuilder::<User>::new()
            .mysql(get_mysql_pool())
            .query("SELECT * FROM users")
            .build_mysql();
        // Test logic (same as above!)
    }
}
```

## Benefits

### Before
- Different import paths for each database
- Different builder types for each database
- Inconsistent API patterns

### After
- Single import for all databases
- One builder type for all databases
- Consistent API across all database types
- Explicit database type specification
- Easier to maintain and extend

## Need Help?

If you encounter issues during migration:

1. Check the [unified builders documentation](unified_rdbc_builders.md)
2. Review the [examples](../examples/unified_rdbc_builder_example.rs)
3. Look at the updated tests for real-world usage patterns

## Breaking Changes

- `PostgresRdbcItemReaderBuilder` type has been removed
- `MySqlRdbcItemReaderBuilder` type has been removed
- `SqliteRdbcItemReaderBuilder` type has been removed
- These are replaced by the unified `RdbcItemReaderBuilder`

The individual reader types (`PostgresRdbcItemReader`, `MySqlRdbcItemReader`, `SqliteRdbcItemReader`) are still available and can be constructed directly using their `new()` methods if you prefer not to use the builder pattern.
