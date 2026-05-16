# RDBC Column Mapping API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `DatabaseItemBinder<O, DB>` trait with a fluent `.column(name, extractor)` API on `RdbcItemWriterBuilder`, eliminating boilerplate and database-specific binder structs.

**Architecture:** A new `ColumnValue` enum (Int, Float, Text, Bool, Bytes, Null) with `From` impls for all primitive types and `Option<T>` acts as a type-erased value carrier. Writer structs store `column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>` instead of an `item_binder` reference; at write time, each closure is called and the result dispatched to `push_bind` with the concrete primitive type. The `DatabaseItemBinder` trait, all three binder fields on the builder, `add_column`, and `validate_config` are removed entirely.

**Tech Stack:** Rust 2021, sqlx (QueryBuilder, push_bind, push_values), tokio (block_in_place)

---

## File Map

| File | Change |
|---|---|
| `src/item/rdbc/column_value.rs` | **Create** — `ColumnValue` enum + `From` impls |
| `src/item/rdbc/writer_common.rs` | **Modify** — delete `DatabaseItemBinder`, update `validate_config` signature (no binder param), update tests |
| `src/item/rdbc/sqlite_writer.rs` | **Modify** — replace `item_binder` with `column_bindings`, update `write()` |
| `src/item/rdbc/postgres_writer.rs` | **Modify** — same as sqlite_writer |
| `src/item/rdbc/mysql_writer.rs` | **Modify** — same as sqlite_writer |
| `src/item/rdbc/unified_writer_builder.rs` | **Modify** — add `.column()`, remove binder fields/methods/`add_column` |
| `src/item/rdbc/mod.rs` | **Modify** — add `column_value` module, export `ColumnValue`, remove `DatabaseItemBinder` |
| `tests/helpers/sqlite_helpers.rs` | **Modify** — remove `SqliteCarItemBinder`, keep `Car` |
| `tests/helpers/postgres_helpers.rs` | **Modify** — remove `PostgresCarItemBinder`, keep `Car` |
| `tests/helpers/mysql_helpers.rs` | **Modify** — remove `MySqlCarItemBinder`, keep `Car` |
| `tests/rdbc_sqlite.rs` | **Modify** — rewrite writer test with `.column()`, add nullable test |
| `tests/rdbc_postgres.rs` | **Modify** — rewrite writer test with `.column()`, add nullable test |
| `tests/rdbc_mysql.rs` | **Modify** — rewrite writer test with `.column()`, add nullable test |
| `examples/database_processing.rs` | **Modify** — replace binder structs with `.column()` calls |
| `examples/benchmark_csv_postgres_xml.rs` | **Modify** — replace binder structs with `.column()` calls |

---

## Task 1: Create `ColumnValue` enum with `From` impls and unit tests

**Files:**
- Create: `src/item/rdbc/column_value.rs`

- [ ] **Step 1: Write the failing unit tests first**

Create `src/item/rdbc/column_value.rs` with only the test module (no impl yet):

```rust
//! Type-erased column value for RDBC item writers.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_i32_to_int() {
        assert!(matches!(ColumnValue::from(42i32), ColumnValue::Int(42)));
    }

    #[test]
    fn should_convert_i64_to_int() {
        assert!(matches!(ColumnValue::from(100i64), ColumnValue::Int(100)));
    }

    #[test]
    fn should_convert_f32_to_float() {
        let v = ColumnValue::from(1.5f32);
        assert!(matches!(v, ColumnValue::Float(_)));
        if let ColumnValue::Float(f) = v {
            assert!((f - 1.5f64).abs() < 1e-5, "f32 should be widened to f64");
        }
    }

    #[test]
    fn should_convert_f64_to_float() {
        assert!(matches!(ColumnValue::from(3.14f64), ColumnValue::Float(_)));
    }

    #[test]
    fn should_convert_bool_to_bool() {
        assert!(matches!(ColumnValue::from(true), ColumnValue::Bool(true)));
        assert!(matches!(ColumnValue::from(false), ColumnValue::Bool(false)));
    }

    #[test]
    fn should_convert_str_to_text() {
        assert!(matches!(ColumnValue::from("hello"), ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_string_to_text() {
        assert!(matches!(
            ColumnValue::from("world".to_string()),
            ColumnValue::Text(_)
        ));
    }

    #[test]
    fn should_convert_bytes_to_bytes() {
        let v = ColumnValue::from(vec![1u8, 2, 3]);
        assert!(matches!(v, ColumnValue::Bytes(_)));
    }

    #[test]
    fn should_convert_some_i32_to_int() {
        assert!(matches!(
            ColumnValue::from(Some(7i32)),
            ColumnValue::Int(7)
        ));
    }

    #[test]
    fn should_convert_none_i32_to_null() {
        assert!(matches!(ColumnValue::from(None::<i32>), ColumnValue::Null));
    }

    #[test]
    fn should_convert_some_string_to_text() {
        let v = ColumnValue::from(Some("abc".to_string()));
        assert!(matches!(v, ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_none_string_to_null() {
        assert!(matches!(
            ColumnValue::from(None::<String>),
            ColumnValue::Null
        ));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --features rdbc-sqlite column_value 2>&1 | head -30
```

Expected: compile error — `ColumnValue` not defined.

- [ ] **Step 3: Implement `ColumnValue` and all `From` impls**

Replace the file content with:

```rust
//! Type-erased column value for RDBC item writers.

/// A type-erased value that can be bound to a database column.
///
/// Used by [`RdbcItemWriterBuilder::column`](crate::item::rdbc::RdbcItemWriterBuilder::column)
/// to carry field values from item extractor closures to `push_bind` at write time.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::rdbc::ColumnValue;
///
/// let v: ColumnValue = 42i32.into();
/// assert!(matches!(v, ColumnValue::Int(42)));
///
/// let v: ColumnValue = None::<i32>.into();
/// assert!(matches!(v, ColumnValue::Null));
/// ```
pub enum ColumnValue {
    /// Signed integer (covers i32, i64).
    Int(i64),
    /// Floating-point number (covers f32, f64).
    Float(f64),
    /// UTF-8 text (covers &str, String).
    Text(String),
    /// Boolean value.
    Bool(bool),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// SQL NULL — produced by Option::None.
    Null,
}

impl From<i32> for ColumnValue {
    fn from(v: i32) -> Self { ColumnValue::Int(v as i64) }
}

impl From<i64> for ColumnValue {
    fn from(v: i64) -> Self { ColumnValue::Int(v) }
}

impl From<f32> for ColumnValue {
    fn from(v: f32) -> Self { ColumnValue::Float(v as f64) }
}

impl From<f64> for ColumnValue {
    fn from(v: f64) -> Self { ColumnValue::Float(v) }
}

impl From<bool> for ColumnValue {
    fn from(v: bool) -> Self { ColumnValue::Bool(v) }
}

impl From<&str> for ColumnValue {
    fn from(v: &str) -> Self { ColumnValue::Text(v.to_string()) }
}

impl From<String> for ColumnValue {
    fn from(v: String) -> Self { ColumnValue::Text(v) }
}

impl From<Vec<u8>> for ColumnValue {
    fn from(v: Vec<u8>) -> Self { ColumnValue::Bytes(v) }
}

impl<T: Into<ColumnValue>> From<Option<T>> for ColumnValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => inner.into(),
            None => ColumnValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_i32_to_int() {
        assert!(matches!(ColumnValue::from(42i32), ColumnValue::Int(42)));
    }

    #[test]
    fn should_convert_i64_to_int() {
        assert!(matches!(ColumnValue::from(100i64), ColumnValue::Int(100)));
    }

    #[test]
    fn should_convert_f32_to_float() {
        let v = ColumnValue::from(1.5f32);
        assert!(matches!(v, ColumnValue::Float(_)));
        if let ColumnValue::Float(f) = v {
            assert!((f - 1.5f64).abs() < 1e-5, "f32 should be widened to f64");
        }
    }

    #[test]
    fn should_convert_f64_to_float() {
        assert!(matches!(ColumnValue::from(3.14f64), ColumnValue::Float(_)));
    }

    #[test]
    fn should_convert_bool_to_bool() {
        assert!(matches!(ColumnValue::from(true), ColumnValue::Bool(true)));
        assert!(matches!(ColumnValue::from(false), ColumnValue::Bool(false)));
    }

    #[test]
    fn should_convert_str_to_text() {
        assert!(matches!(ColumnValue::from("hello"), ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_string_to_text() {
        assert!(matches!(
            ColumnValue::from("world".to_string()),
            ColumnValue::Text(_)
        ));
    }

    #[test]
    fn should_convert_bytes_to_bytes() {
        let v = ColumnValue::from(vec![1u8, 2, 3]);
        assert!(matches!(v, ColumnValue::Bytes(_)));
    }

    #[test]
    fn should_convert_some_i32_to_int() {
        assert!(matches!(
            ColumnValue::from(Some(7i32)),
            ColumnValue::Int(7)
        ));
    }

    #[test]
    fn should_convert_none_i32_to_null() {
        assert!(matches!(ColumnValue::from(None::<i32>), ColumnValue::Null));
    }

    #[test]
    fn should_convert_some_string_to_text() {
        let v = ColumnValue::from(Some("abc".to_string()));
        assert!(matches!(v, ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_none_string_to_null() {
        assert!(matches!(
            ColumnValue::from(None::<String>),
            ColumnValue::Null
        ));
    }
}
```

- [ ] **Step 4: Wire the module in `mod.rs`**

In `src/item/rdbc/mod.rs`, add before the existing module declarations:

```rust
/// Type-erased column value for item writers.
mod column_value;
```

And add to the re-exports section (after `pub use select_builder::SelectBuilder;`):

```rust
pub use column_value::ColumnValue;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --features rdbc-sqlite column_value 2>&1 | tail -10
```

Expected: `test result: ok. 12 passed`

- [ ] **Step 6: Commit**

```bash
git add src/item/rdbc/column_value.rs src/item/rdbc/mod.rs
git commit -m "feat(rdbc): add ColumnValue enum with From impls for column binding"
```

---

## Task 2: Update `writer_common.rs` — remove `DatabaseItemBinder`, simplify `validate_config`

**Files:**
- Modify: `src/item/rdbc/writer_common.rs`

The current `validate_config` takes `item_binder` as a parameter and returns it. With the new API, there is no binder — validation only checks pool, table, and that column_bindings is non-empty. The function signature changes; all three writer files that call it must be updated in the same task.

- [ ] **Step 1: Write the new `validate_config` tests**

The existing tests in `writer_common.rs` test the old signature. Replace only the binder-related tests. The new `validate_config` signature is:

```rust
pub fn validate_config<'a, DB: Database>(
    pool: Option<&'a Pool<DB>>,
    table: Option<&'a str>,
    column_count: usize,
) -> Result<(&'a Pool<DB>, &'a str), BatchError>
```

The new tests (replace the existing four `validate_config` tests with these):

```rust
#[test]
fn should_return_error_when_columns_is_empty() {
    let result = validate_config::<Sqlite>(None, Some("tbl"), 0);
    match result.err().unwrap() {
        BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "unexpected: {msg}"),
        e => panic!("expected ItemWriter, got {e:?}"),
    }
}

#[test]
fn should_return_error_when_pool_is_missing() {
    let result = validate_config::<Sqlite>(None, Some("tbl"), 1);
    match result.err().unwrap() {
        BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "unexpected: {msg}"),
        e => panic!("expected ItemWriter, got {e:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn should_return_error_when_table_is_missing() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let result = validate_config::<Sqlite>(Some(&pool), None, 1);
    match result.err().unwrap() {
        BatchError::ItemWriter(msg) => assert!(msg.contains("Table"), "unexpected: {msg}"),
        e => panic!("expected ItemWriter, got {e:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn should_return_ok_when_all_config_provided() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let result = validate_config::<Sqlite>(Some(&pool), Some("tbl"), 1);
    assert!(result.is_ok(), "should return Ok when all config is provided");
}
```

- [ ] **Step 2: Update `writer_common.rs`**

Replace the entire file content (keep `BIND_LIMIT`, `log_write_success`, `create_write_error`, `max_items_per_batch` unchanged; update only `ValidatedConfig`, `validate_config`, and the module-level `//!` to remove the `DatabaseItemBinder` mention, and remove the `use crate::item::rdbc::DatabaseItemBinder` import from tests):

```rust
//! Common functionality for database item writers.
//!
//! Provides shared utilities used across PostgreSQL, MySQL, and SQLite writers.

use crate::BatchError;
use sqlx::{Database, Pool};

/// The maximum number of parameters bound in a single SQL statement.
/// This is the most conservative limit across major databases (MySQL's limit).
pub const BIND_LIMIT: usize = 65535;

/// Type alias for the validated configuration returned by [`validate_config`].
type ValidatedConfig<'a, DB> = (&'a Pool<DB>, &'a str);

/// Validates that all required writer configuration fields are set.
///
/// # Errors
///
/// Returns [`BatchError::ItemWriter`] if columns is zero, pool is `None`, or table is `None`.
pub fn validate_config<'a, DB: Database>(
    pool: Option<&'a Pool<DB>>,
    table: Option<&'a str>,
    column_count: usize,
) -> Result<ValidatedConfig<'a, DB>, BatchError> {
    if column_count == 0 {
        return Err(BatchError::ItemWriter(
            "No columns specified for database write".to_string(),
        ));
    }

    let pool =
        pool.ok_or_else(|| BatchError::ItemWriter("Database pool not configured".to_string()))?;

    let table =
        table.ok_or_else(|| BatchError::ItemWriter("Table name not configured".to_string()))?;

    Ok((pool, table))
}

/// Logs a successful write operation.
#[inline]
pub fn log_write_success(items_count: usize, table: &str, db_name: &str) {
    log::debug!(
        "Successfully wrote {} items to {} table {}",
        items_count,
        db_name,
        table
    );
}

/// Creates a database write error.
///
/// # Returns
///
/// A [`BatchError::ItemWriter`] with a formatted error message.
pub fn create_write_error(table: &str, db_name: &str, error: impl std::fmt::Display) -> BatchError {
    log::error!(
        "Failed to write items to {} table {}: {}",
        db_name,
        table,
        error
    );
    BatchError::ItemWriter(format!("{} write failed: {}", db_name, error))
}

/// Calculates the maximum number of items per batch given the bind limit.
///
/// # Returns
///
/// `BIND_LIMIT / column_count`
#[inline]
pub fn max_items_per_batch(column_count: usize) -> usize {
    BIND_LIMIT / column_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BatchError;
    use sqlx::Sqlite;

    #[test]
    fn should_compute_max_items_per_batch() {
        assert_eq!(max_items_per_batch(1), 65535);
        assert_eq!(max_items_per_batch(2), 32767);
        assert_eq!(max_items_per_batch(10), 6553);
        assert_eq!(max_items_per_batch(100), 655);
    }

    #[test]
    fn should_define_bind_limit_as_65535() {
        assert_eq!(BIND_LIMIT, 65535);
    }

    #[test]
    fn should_return_error_when_columns_is_empty() {
        let result = validate_config::<Sqlite>(None, Some("tbl"), 0);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_is_missing() {
        let result = validate_config::<Sqlite>(None, Some("tbl"), 1);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_error_when_table_is_missing() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let result = validate_config::<Sqlite>(Some(&pool), None, 1);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("Table"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_ok_when_all_config_provided() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let result = validate_config::<Sqlite>(Some(&pool), Some("tbl"), 1);
        assert!(result.is_ok(), "should return Ok when all config is provided");
    }

    #[test]
    fn should_call_log_write_success_without_panic() {
        log_write_success(42, "users", "PostgreSQL");
    }

    #[test]
    fn should_create_write_error_with_formatted_message() {
        let err = create_write_error("orders", "MySQL", "connection refused");
        match err {
            BatchError::ItemWriter(msg) => {
                assert!(msg.contains("MySQL"), "missing db name: {msg}");
                assert!(msg.contains("connection refused"), "missing cause: {msg}");
            }
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }
}
```

- [ ] **Step 3: Run `writer_common` tests (they will fail because writers still use old signature)**

```bash
cargo test --features rdbc-sqlite writer_common 2>&1 | tail -10
```

Expected: `test result: ok` for this file's tests. Compile errors in `postgres_writer`, `mysql_writer`, `sqlite_writer` are expected and will be fixed in Tasks 3-5.

- [ ] **Step 4: Commit**

```bash
git add src/item/rdbc/writer_common.rs
git commit -m "refactor(rdbc): simplify validate_config — remove DatabaseItemBinder parameter"
```

---

## Task 3: Rewrite `sqlite_writer.rs` with `column_bindings`

**Files:**
- Modify: `src/item/rdbc/sqlite_writer.rs`

- [ ] **Step 1: Replace the entire file**

The struct loses its `'a` lifetime and `item_binder` field. It gains `column_bindings`. The `write()` implementation calls `validate_config` with the new 3-argument signature and dispatches `ColumnValue` variants to `push_bind`.

```rust
use serde::Serialize;
use sqlx::{Pool, QueryBuilder, Sqlite};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::ColumnValue;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a SQLite database using SQLx.
///
/// Supports batch INSERT via a list of column bindings supplied through
/// [`RdbcItemWriterBuilder::column`](crate::item::rdbc::RdbcItemWriterBuilder::column).
///
/// # Construction
///
/// Use [`RdbcItemWriterBuilder`](crate::item::rdbc::RdbcItemWriterBuilder) — direct
/// construction is not public.
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
/// use sqlx::SqlitePool;
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Task { id: i32, title: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
///
/// let writer = RdbcItemWriterBuilder::<Task>::new()
///     .sqlite(&pool)
///     .table("tasks")
///     .column("id", |t: &Task| t.id.into())
///     .column("title", |t: &Task| t.title.as_str().into())
///     .build_sqlite();
/// # Ok(())
/// # }
/// ```
pub struct SqliteItemWriter<O> {
    pool: Option<sqlx::Pool<Sqlite>>,
    table: Option<String>,
    #[allow(clippy::type_complexity)]
    column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>,
}

impl<O> SqliteItemWriter<O> {
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            column_bindings: Vec::new(),
        }
    }

    pub(crate) fn pool(mut self, pool: &Pool<Sqlite>) -> Self {
        self.pool = Some(pool.clone());
        self
    }

    pub(crate) fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    pub(crate) fn add_column_binding(
        mut self,
        name: String,
        extractor: Box<dyn Fn(&O) -> ColumnValue>,
    ) -> Self {
        self.column_bindings.push((name, extractor));
        self
    }
}

impl<O> Default for SqliteItemWriter<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for SqliteItemWriter<O> {
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        let (pool, table) = validate_config(
            self.pool.as_ref(),
            self.table.as_deref(),
            self.column_bindings.len(),
        )?;

        let col_names: Vec<&str> = self.column_bindings.iter().map(|(n, _)| n.as_str()).collect();

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(table);
        query_builder.push(" (");
        query_builder.push(col_names.join(","));
        query_builder.push(") ");

        let max_items = max_items_per_batch(self.column_bindings.len());
        let items_to_write: Vec<_> = items.iter().take(max_items).collect();
        let items_count = items_to_write.len();

        query_builder.push_values(items_to_write, |mut b, item| {
            for (_, extractor) in &self.column_bindings {
                match extractor(item) {
                    ColumnValue::Int(v) => { b.push_bind(v); }
                    ColumnValue::Float(v) => { b.push_bind(v); }
                    ColumnValue::Text(v) => { b.push_bind(v); }
                    ColumnValue::Bool(v) => { b.push_bind(v); }
                    ColumnValue::Bytes(v) => { b.push_bind(v); }
                    ColumnValue::Null => { b.push_bind(Option::<String>::None); }
                }
            }
        });

        let query = query_builder.build();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { query.execute(pool).await })
        });

        match result {
            Ok(_) => {
                log_write_success(items_count, table, "SQLite");
                Ok(())
            }
            Err(e) => Err(create_write_error(table, "SQLite", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;
    use crate::item::rdbc::ColumnValue;

    #[test]
    fn should_start_with_empty_state() {
        let writer = SqliteItemWriter::<String>::new();
        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.column_bindings.is_empty());
    }

    #[test]
    fn should_store_column_bindings_in_order() {
        let writer = SqliteItemWriter::<String>::new()
            .table("t")
            .add_column_binding("a".to_string(), Box::new(|_| ColumnValue::Null))
            .add_column_binding("b".to_string(), Box::new(|_| ColumnValue::Null));
        let names: Vec<&str> = writer.column_bindings.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a", "b"], "bindings should preserve insertion order");
    }

    #[test]
    fn should_return_ok_for_empty_items() {
        let writer = SqliteItemWriter::<String>::new();
        assert!(writer.write(&[]).is_ok());
    }

    #[test]
    fn should_return_error_when_no_columns_and_items_given() {
        use crate::BatchError;
        let writer = SqliteItemWriter::<String>::new().table("t");
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_not_configured() {
        use crate::BatchError;
        let writer = SqliteItemWriter::<String>::new()
            .table("t")
            .add_column_binding("v".to_string(), Box::new(|s: &String| s.as_str().into()));
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_write_items_to_in_memory_sqlite() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE t (v TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();

        let writer = SqliteItemWriter::<String>::new()
            .pool(&pool)
            .table("t")
            .add_column_binding("v".to_string(), Box::new(|s: &String| s.as_str().into()));

        writer.write(&["hello".to_string(), "world".to_string()]).unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM t")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "both items should have been written");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_error_when_query_fails() {
        use crate::BatchError;
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let writer = SqliteItemWriter::<String>::new()
            .pool(&pool)
            .table("nonexistent_table")
            .add_column_binding("v".to_string(), Box::new(|s: &String| s.as_str().into()));

        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("SQLite"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_write_null_for_none_optional_column() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE t (id INTEGER NOT NULL, note TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        #[derive(Clone, serde::Serialize)]
        struct Row { id: i32, note: Option<String> }

        let writer = SqliteItemWriter::<Row>::new()
            .pool(&pool)
            .table("t")
            .add_column_binding("id".to_string(), Box::new(|r: &Row| r.id.into()))
            .add_column_binding(
                "note".to_string(),
                Box::new(|r: &Row| r.note.clone().into()),
            );

        writer
            .write(&[Row { id: 1, note: None }])
            .unwrap();

        let (note,): (Option<String>,) = sqlx::query_as("SELECT note FROM t WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(note.is_none(), "note should be NULL in the database");
    }
}
```

- [ ] **Step 2: Run SQLite writer tests**

```bash
cargo test --features rdbc-sqlite sqlite_writer 2>&1 | tail -15
```

Expected: `test result: ok. 7 passed`

- [ ] **Step 3: Commit**

```bash
git add src/item/rdbc/sqlite_writer.rs
git commit -m "feat(rdbc): replace item_binder with column_bindings in SqliteItemWriter"
```

---

## Task 4: Rewrite `postgres_writer.rs` with `column_bindings`

**Files:**
- Modify: `src/item/rdbc/postgres_writer.rs`

- [ ] **Step 1: Replace the entire file**

```rust
use serde::Serialize;
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::ColumnValue;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a PostgreSQL database using SQLx.
///
/// Supports batch INSERT via a list of column bindings supplied through
/// [`RdbcItemWriterBuilder::column`](crate::item::rdbc::RdbcItemWriterBuilder::column).
///
/// # Construction
///
/// Use [`RdbcItemWriterBuilder`](crate::item::rdbc::RdbcItemWriterBuilder) — direct
/// construction is not public.
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
/// use sqlx::PgPool;
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct User { id: i32, name: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
///
/// let writer = RdbcItemWriterBuilder::<User>::new()
///     .postgres(&pool)
///     .table("users")
///     .column("id", |u: &User| u.id.into())
///     .column("name", |u: &User| u.name.as_str().into())
///     .build_postgres();
/// # Ok(())
/// # }
/// ```
pub struct PostgresItemWriter<O> {
    pub(crate) pool: Option<sqlx::Pool<Postgres>>,
    pub(crate) table: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(crate) column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>,
}

impl<O> PostgresItemWriter<O> {
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            column_bindings: Vec::new(),
        }
    }

    pub(crate) fn pool(mut self, pool: &Pool<Postgres>) -> Self {
        self.pool = Some(pool.clone());
        self
    }

    pub(crate) fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    pub(crate) fn add_column_binding(
        mut self,
        name: String,
        extractor: Box<dyn Fn(&O) -> ColumnValue>,
    ) -> Self {
        self.column_bindings.push((name, extractor));
        self
    }
}

impl<O> Default for PostgresItemWriter<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for PostgresItemWriter<O> {
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        let (pool, table) = validate_config(
            self.pool.as_ref(),
            self.table.as_deref(),
            self.column_bindings.len(),
        )?;

        let col_names: Vec<&str> = self.column_bindings.iter().map(|(n, _)| n.as_str()).collect();

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(table);
        query_builder.push(" (");
        query_builder.push(col_names.join(","));
        query_builder.push(") ");

        let max_items = max_items_per_batch(self.column_bindings.len());
        let items_to_write: Vec<_> = items.iter().take(max_items).collect();
        let items_count = items_to_write.len();

        query_builder.push_values(items_to_write, |mut b, item| {
            for (_, extractor) in &self.column_bindings {
                match extractor(item) {
                    ColumnValue::Int(v) => { b.push_bind(v); }
                    ColumnValue::Float(v) => { b.push_bind(v); }
                    ColumnValue::Text(v) => { b.push_bind(v); }
                    ColumnValue::Bool(v) => { b.push_bind(v); }
                    ColumnValue::Bytes(v) => { b.push_bind(v); }
                    ColumnValue::Null => { b.push_bind(Option::<String>::None); }
                }
            }
        });

        let query = query_builder.build();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { query.execute(pool).await })
        });

        match result {
            Ok(_) => {
                log_write_success(items_count, table, "PostgreSQL");
                Ok(())
            }
            Err(e) => Err(create_write_error(table, "PostgreSQL", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::rdbc::ColumnValue;

    #[test]
    fn should_start_with_empty_state() {
        let writer = PostgresItemWriter::<String>::new();
        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.column_bindings.is_empty());
    }

    #[test]
    fn should_store_column_bindings_in_order() {
        let writer = PostgresItemWriter::<String>::new()
            .add_column_binding("x".to_string(), Box::new(|_| ColumnValue::Null))
            .add_column_binding("y".to_string(), Box::new(|_| ColumnValue::Null));
        let names: Vec<&str> = writer.column_bindings.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["x", "y"]);
    }

    #[test]
    fn should_return_ok_for_empty_items() {
        let writer = PostgresItemWriter::<String>::new();
        assert!(writer.write(&[]).is_ok());
    }

    #[test]
    fn should_return_error_when_no_columns_and_items_given() {
        use crate::BatchError;
        let writer = PostgresItemWriter::<String>::new().table("t");
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_not_configured() {
        use crate::BatchError;
        let writer = PostgresItemWriter::<String>::new()
            .table("t")
            .add_column_binding("v".to_string(), Box::new(|s: &String| s.as_str().into()));
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }
}
```

- [ ] **Step 2: Run Postgres writer tests**

```bash
cargo test --features rdbc-postgres postgres_writer 2>&1 | tail -15
```

Expected: `test result: ok. 5 passed`

- [ ] **Step 3: Commit**

```bash
git add src/item/rdbc/postgres_writer.rs
git commit -m "feat(rdbc): replace item_binder with column_bindings in PostgresItemWriter"
```

---

## Task 5: Rewrite `mysql_writer.rs` with `column_bindings`

**Files:**
- Modify: `src/item/rdbc/mysql_writer.rs`

- [ ] **Step 1: Replace the entire file**

```rust
use serde::Serialize;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::ColumnValue;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a MySQL database using SQLx.
///
/// Supports batch INSERT via a list of column bindings supplied through
/// [`RdbcItemWriterBuilder::column`](crate::item::rdbc::RdbcItemWriterBuilder::column).
///
/// # Construction
///
/// Use [`RdbcItemWriterBuilder`](crate::item::rdbc::RdbcItemWriterBuilder) — direct
/// construction is not public.
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
/// use sqlx::MySqlPool;
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Product { id: i32, name: String, price: f64 }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
///
/// let writer = RdbcItemWriterBuilder::<Product>::new()
///     .mysql(&pool)
///     .table("products")
///     .column("id", |p: &Product| p.id.into())
///     .column("name", |p: &Product| p.name.as_str().into())
///     .column("price", |p: &Product| p.price.into())
///     .build_mysql();
/// # Ok(())
/// # }
/// ```
pub struct MySqlItemWriter<O> {
    pub(crate) pool: Option<sqlx::Pool<MySql>>,
    pub(crate) table: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(crate) column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>,
}

impl<O> MySqlItemWriter<O> {
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            column_bindings: Vec::new(),
        }
    }

    pub(crate) fn pool(mut self, pool: &Pool<MySql>) -> Self {
        self.pool = Some(pool.clone());
        self
    }

    pub(crate) fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    pub(crate) fn add_column_binding(
        mut self,
        name: String,
        extractor: Box<dyn Fn(&O) -> ColumnValue>,
    ) -> Self {
        self.column_bindings.push((name, extractor));
        self
    }
}

impl<O> Default for MySqlItemWriter<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for MySqlItemWriter<O> {
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        let (pool, table) = validate_config(
            self.pool.as_ref(),
            self.table.as_deref(),
            self.column_bindings.len(),
        )?;

        let col_names: Vec<&str> = self.column_bindings.iter().map(|(n, _)| n.as_str()).collect();

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(table);
        query_builder.push(" (");
        query_builder.push(col_names.join(","));
        query_builder.push(") ");

        let max_items = max_items_per_batch(self.column_bindings.len());
        let items_to_write: Vec<_> = items.iter().take(max_items).collect();
        let items_count = items_to_write.len();

        query_builder.push_values(items_to_write, |mut b, item| {
            for (_, extractor) in &self.column_bindings {
                match extractor(item) {
                    ColumnValue::Int(v) => { b.push_bind(v); }
                    ColumnValue::Float(v) => { b.push_bind(v); }
                    ColumnValue::Text(v) => { b.push_bind(v); }
                    ColumnValue::Bool(v) => { b.push_bind(v); }
                    ColumnValue::Bytes(v) => { b.push_bind(v); }
                    ColumnValue::Null => { b.push_bind(Option::<String>::None); }
                }
            }
        });

        let query = query_builder.build();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { query.execute(pool).await })
        });

        match result {
            Ok(_) => {
                log_write_success(items_count, table, "MySQL");
                Ok(())
            }
            Err(e) => Err(create_write_error(table, "MySQL", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::rdbc::ColumnValue;

    #[test]
    fn should_start_with_empty_state() {
        let writer = MySqlItemWriter::<String>::new();
        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.column_bindings.is_empty());
    }

    #[test]
    fn should_store_column_bindings_in_order() {
        let writer = MySqlItemWriter::<String>::new()
            .add_column_binding("a".to_string(), Box::new(|_| ColumnValue::Null))
            .add_column_binding("b".to_string(), Box::new(|_| ColumnValue::Null));
        let names: Vec<&str> = writer.column_bindings.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn should_return_ok_for_empty_items() {
        let writer = MySqlItemWriter::<String>::new();
        assert!(writer.write(&[]).is_ok());
    }

    #[test]
    fn should_return_error_when_no_columns_and_items_given() {
        use crate::BatchError;
        let writer = MySqlItemWriter::<String>::new().table("t");
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_not_configured() {
        use crate::BatchError;
        let writer = MySqlItemWriter::<String>::new()
            .table("t")
            .add_column_binding("v".to_string(), Box::new(|s: &String| s.as_str().into()));
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }
}
```

- [ ] **Step 2: Run MySQL writer tests**

```bash
cargo test --features rdbc-mysql mysql_writer 2>&1 | tail -15
```

Expected: `test result: ok. 5 passed`

- [ ] **Step 3: Commit**

```bash
git add src/item/rdbc/mysql_writer.rs
git commit -m "feat(rdbc): replace item_binder with column_bindings in MySqlItemWriter"
```

---

## Task 6: Rewrite `unified_writer_builder.rs` — add `.column()`, remove binder API

**Files:**
- Modify: `src/item/rdbc/unified_writer_builder.rs`

The builder gains `column_bindings`, loses `postgres_binder`/`mysql_binder`/`sqlite_binder` fields and methods and `add_column`. The `build_*` methods forward `column_bindings` via `add_column_binding` calls.

- [ ] **Step 1: Replace the entire file**

```rust
use sqlx::{MySql, Pool, Postgres, Sqlite};

use super::column_value::ColumnValue;
use super::database_type::DatabaseType;
use super::mysql_writer::MySqlItemWriter;
use super::postgres_writer::PostgresItemWriter;
use super::sqlite_writer::SqliteItemWriter;

/// Unified builder for creating RDBC item writers for any supported database.
///
/// # Type Parameters
///
/// * `O` - The item type to write
///
/// # Examples
///
/// ## SQLite
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
/// use sqlx::SqlitePool;
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Task { id: i32, title: String, done: bool }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
///
/// let writer = RdbcItemWriterBuilder::<Task>::new()
///     .sqlite(&pool)
///     .table("tasks")
///     .column("id", |t: &Task| t.id.into())
///     .column("title", |t: &Task| t.title.as_str().into())
///     .column("done", |t: &Task| t.done.into())
///     .build_sqlite();
/// # Ok(())
/// # }
/// ```
///
/// ## PostgreSQL
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
/// use sqlx::PgPool;
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct User { id: i32, name: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
///
/// let writer = RdbcItemWriterBuilder::<User>::new()
///     .postgres(&pool)
///     .table("users")
///     .column("id", |u: &User| u.id.into())
///     .column("name", |u: &User| u.name.as_str().into())
///     .build_postgres();
/// # Ok(())
/// # }
/// ```
pub struct RdbcItemWriterBuilder<O> {
    postgres_pool: Option<sqlx::Pool<Postgres>>,
    mysql_pool: Option<sqlx::Pool<MySql>>,
    sqlite_pool: Option<sqlx::Pool<Sqlite>>,
    table: Option<String>,
    #[allow(clippy::type_complexity)]
    column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>,
    db_type: Option<DatabaseType>,
}

impl<O: 'static> RdbcItemWriterBuilder<O> {
    /// Creates a new writer builder with default configuration.
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
            mysql_pool: None,
            sqlite_pool: None,
            table: None,
            column_bindings: Vec::new(),
            db_type: None,
        }
    }

    /// Sets the PostgreSQL connection pool.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::RdbcItemWriterBuilder;
    /// # use sqlx::PgPool;
    /// # use serde::Serialize;
    /// # #[derive(Clone, Serialize)] struct User { id: i32 }
    /// # async fn ex() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let builder = RdbcItemWriterBuilder::<User>::new().postgres(&pool);
    /// # Ok(()) }
    /// ```
    pub fn postgres(mut self, pool: &Pool<Postgres>) -> Self {
        self.postgres_pool = Some(pool.clone());
        self.db_type = Some(DatabaseType::Postgres);
        self
    }

    /// Sets the MySQL connection pool.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::RdbcItemWriterBuilder;
    /// # use sqlx::MySqlPool;
    /// # use serde::Serialize;
    /// # #[derive(Clone, Serialize)] struct Product { id: i32 }
    /// # async fn ex() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
    /// let builder = RdbcItemWriterBuilder::<Product>::new().mysql(&pool);
    /// # Ok(()) }
    /// ```
    pub fn mysql(mut self, pool: &Pool<MySql>) -> Self {
        self.mysql_pool = Some(pool.clone());
        self.db_type = Some(DatabaseType::MySql);
        self
    }

    /// Sets the SQLite connection pool.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::RdbcItemWriterBuilder;
    /// # use sqlx::SqlitePool;
    /// # use serde::Serialize;
    /// # #[derive(Clone, Serialize)] struct Task { id: i32 }
    /// # async fn ex() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = SqlitePool::connect("sqlite::memory:").await?;
    /// let builder = RdbcItemWriterBuilder::<Task>::new().sqlite(&pool);
    /// # Ok(()) }
    /// ```
    pub fn sqlite(mut self, pool: &Pool<Sqlite>) -> Self {
        self.sqlite_pool = Some(pool.clone());
        self.db_type = Some(DatabaseType::Sqlite);
        self
    }

    /// Sets the target table name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::RdbcItemWriterBuilder;
    /// # use serde::Serialize;
    /// # #[derive(Clone, Serialize)] struct Task { id: i32 }
    /// let builder = RdbcItemWriterBuilder::<Task>::new().table("tasks");
    /// ```
    pub fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Adds a column mapping: the extractor closure maps an item to a [`ColumnValue`].
    ///
    /// Column order in the generated `INSERT` matches the order of `.column()` calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, ColumnValue};
    /// # use serde::Serialize;
    /// # #[derive(Clone, Serialize)] struct Task { id: i32, title: String, score: Option<f64> }
    /// let builder = RdbcItemWriterBuilder::<Task>::new()
    ///     .table("tasks")
    ///     .column("id", |t: &Task| t.id.into())
    ///     .column("title", |t: &Task| t.title.as_str().into())
    ///     .column("score", |t: &Task| t.score.into());
    /// ```
    pub fn column(mut self, name: &str, extractor: impl Fn(&O) -> ColumnValue + 'static) -> Self {
        self.column_bindings.push((name.to_string(), Box::new(extractor)));
        self
    }

    /// Builds a PostgreSQL writer.
    ///
    /// # Panics
    ///
    /// Does not panic — misconfiguration surfaces as `BatchError` on first `.write()` call.
    pub fn build_postgres(self) -> PostgresItemWriter<O> {
        let mut writer = PostgresItemWriter::new();
        if let Some(pool) = self.postgres_pool {
            writer = writer.pool(&pool);
        }
        if let Some(table) = self.table {
            writer = writer.table(&table);
        }
        for (name, extractor) in self.column_bindings {
            writer = writer.add_column_binding(name, extractor);
        }
        writer
    }

    /// Builds a MySQL writer.
    ///
    /// # Panics
    ///
    /// Does not panic — misconfiguration surfaces as `BatchError` on first `.write()` call.
    pub fn build_mysql(self) -> MySqlItemWriter<O> {
        let mut writer = MySqlItemWriter::new();
        if let Some(pool) = self.mysql_pool {
            writer = writer.pool(&pool);
        }
        if let Some(table) = self.table {
            writer = writer.table(&table);
        }
        for (name, extractor) in self.column_bindings {
            writer = writer.add_column_binding(name, extractor);
        }
        writer
    }

    /// Builds a SQLite writer.
    ///
    /// # Panics
    ///
    /// Does not panic — misconfiguration surfaces as `BatchError` on first `.write()` call.
    pub fn build_sqlite(self) -> SqliteItemWriter<O> {
        let mut writer = SqliteItemWriter::new();
        if let Some(pool) = self.sqlite_pool {
            writer = writer.pool(&pool);
        }
        if let Some(table) = self.table {
            writer = writer.table(&table);
        }
        for (name, extractor) in self.column_bindings {
            writer = writer.add_column_binding(name, extractor);
        }
        writer
    }
}

impl<O: 'static> Default for RdbcItemWriterBuilder<O> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BatchError;
    use crate::item::rdbc::ColumnValue;

    #[test]
    fn should_transfer_table_to_postgres_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("users")
            .build_postgres();
        assert_eq!(writer.table.as_deref(), Some("users"));
    }

    #[test]
    fn should_accumulate_column_bindings_in_order() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .column("id", |_| ColumnValue::Int(0))
            .column("name", |_| ColumnValue::Null)
            .build_postgres();
        let names: Vec<&str> = writer.column_bindings.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["id", "name"]);
    }

    #[test]
    fn should_transfer_table_and_columns_to_mysql_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("orders")
            .column("id", |_| ColumnValue::Int(0))
            .column("total", |_| ColumnValue::Float(0.0))
            .build_mysql();
        assert_eq!(writer.table.as_deref(), Some("orders"));
        assert_eq!(writer.column_bindings.len(), 2);
    }

    #[test]
    fn should_return_error_for_missing_pool_on_write() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("t")
            .column("v", |s: &String| s.as_str().into())
            .build_sqlite();
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_pass_pool_to_sqlite_writer() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let writer = RdbcItemWriterBuilder::<String>::new()
            .sqlite(&pool)
            .table("t")
            .column("v", |s: &String| s.as_str().into())
            .build_sqlite();
        assert!(writer.pool.is_some(), "pool should be forwarded");
    }

    #[test]
    fn should_create_via_default() {
        let _b = RdbcItemWriterBuilder::<String>::default();
    }
}
```

- [ ] **Step 2: Run builder tests**

```bash
cargo test --features rdbc-sqlite unified_writer_builder 2>&1 | tail -15
```

Expected: `test result: ok. 6 passed`

- [ ] **Step 3: Remove `DatabaseItemBinder` from `mod.rs`**

In `src/item/rdbc/mod.rs`:

1. Delete the `DatabaseItemBinder` trait definition (lines containing `pub trait DatabaseItemBinder` through its closing `}`)
2. Delete the `use sqlx::{Database, query_builder::Separated};` import at the top (it was only needed for the trait)

The final `mod.rs` should look like:

```rust
/// Fluent SQL SELECT builder for RDBC item readers.
mod select_builder;

/// Common utilities for database item writers.
mod writer_common;

/// Common utilities for database item readers.
mod reader_common;

/// Database type enumeration.
mod database_type;

/// Unified reader builder for all database types.
mod unified_reader_builder;

/// Unified writer builder for all database types.
mod unified_writer_builder;

/// Type-erased column value for item writers.
mod column_value;

/// PostgreSQL-specific reader implementation.
pub mod postgres_reader;

/// MySQL-specific reader implementation.
pub mod mysql_reader;

/// SQLite-specific reader implementation.
pub mod sqlite_reader;

/// PostgreSQL-specific writer implementation.
pub mod postgres_writer;

/// MySQL-specific writer implementation.
pub mod mysql_writer;

/// SQLite-specific writer implementation.
pub mod sqlite_writer;

// Re-export database-specific reader and writer types (for direct usage)
pub use mysql_reader::MySqlRdbcItemReader;
pub use mysql_writer::MySqlItemWriter;
pub use postgres_reader::PostgresRdbcItemReader;
pub use postgres_writer::PostgresItemWriter;
pub use sqlite_reader::SqliteRdbcItemReader;
pub use sqlite_writer::SqliteItemWriter;

// Re-export unified builder types (recommended API)
pub use column_value::ColumnValue;
pub use database_type::DatabaseType;
pub use select_builder::SelectBuilder;
pub use unified_reader_builder::RdbcItemReaderBuilder;
pub use unified_writer_builder::RdbcItemWriterBuilder;
```

- [ ] **Step 4: Run all rdbc unit tests to confirm no regressions**

```bash
cargo test --features rdbc-sqlite,rdbc-postgres,rdbc-mysql 2>&1 | tail -20
```

Expected: all pass, zero compile errors.

- [ ] **Step 5: Commit**

```bash
git add src/item/rdbc/unified_writer_builder.rs src/item/rdbc/mod.rs
git commit -m "feat(rdbc): add .column() to RdbcItemWriterBuilder; remove DatabaseItemBinder"
```

---

## Task 7: Update test helpers — remove binder structs, keep `Car`

**Files:**
- Modify: `tests/helpers/sqlite_helpers.rs`
- Modify: `tests/helpers/postgres_helpers.rs`
- Modify: `tests/helpers/mysql_helpers.rs`

- [ ] **Step 1: Update `tests/helpers/sqlite_helpers.rs`**

Remove `SqliteCarItemBinder` struct and its `DatabaseItemBinder` impl. Remove the `use spring_batch_rs::item::rdbc::DatabaseItemBinder;` and `use sqlx::{..., query_builder::Separated};` imports (keep `FromRow`). Keep `Car`, `CREATE_CARS_TABLE_SQL`, `SELECT_ALL_CARS_SQL`, and the `test_car_creation` test.

Replace the file with:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Car domain model for database operations.
#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Car {
    pub year: i16,
    pub make: String,
    pub model: String,
    pub description: String,
}

impl Car {
    /// Creates a new Car instance.
    pub fn new(year: i16, make: String, model: String, description: String) -> Self {
        Self { year, make, model, description }
    }
}

/// SQL statement to create the cars table in SQLite.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year INTEGER NOT NULL,
        make VARCHAR(25) NOT NULL,
        model VARCHAR(25) NOT NULL,
        description VARCHAR(25) NOT NULL
    );";

/// SQL statement to select all cars from the table.
#[allow(dead_code)]
pub const SELECT_ALL_CARS_SQL: &str = "SELECT year, make, model, description FROM cars";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_creation() {
        let car = Car::new(
            1967,
            "Ford".to_string(),
            "Mustang fastback 1967".to_string(),
            "American car".to_string(),
        );
        assert_eq!(car.year, 1967);
        assert_eq!(car.make, "Ford");
        assert_eq!(car.model, "Mustang fastback 1967");
        assert_eq!(car.description, "American car");
    }
}
```

- [ ] **Step 2: Update `tests/helpers/postgres_helpers.rs`**

Same change — remove `PostgresCarItemBinder` and related imports, keep `Car`, SQL constants, test.

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Car domain model for database operations.
#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Car {
    pub year: i16,
    pub make: String,
    pub model: String,
    pub description: String,
}

impl Car {
    /// Creates a new Car instance.
    pub fn new(year: i16, make: String, model: String, description: String) -> Self {
        Self { year, make, model, description }
    }
}

/// SQL statement to create the cars table in PostgreSQL.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year SMALLINT NOT NULL,
        make TEXT NOT NULL,
        model TEXT NOT NULL,
        description TEXT NOT NULL
    );";

/// SQL statement to select all cars from the table.
#[allow(dead_code)]
pub const SELECT_ALL_CARS_SQL: &str = "SELECT year, make, model, description FROM cars";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_creation() {
        let car = Car::new(
            1948,
            "Porsche".to_string(),
            "356".to_string(),
            "Luxury sports car".to_string(),
        );
        assert_eq!(car.year, 1948);
        assert_eq!(car.make, "Porsche");
        assert_eq!(car.model, "356");
        assert_eq!(car.description, "Luxury sports car");
    }
}
```

- [ ] **Step 3: Update `tests/helpers/mysql_helpers.rs`**

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Car domain model for database operations.
#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Car {
    pub year: i16,
    pub make: String,
    pub model: String,
    pub description: String,
}

impl Car {
    /// Creates a new Car instance.
    pub fn new(year: i16, make: String, model: String, description: String) -> Self {
        Self { year, make, model, description }
    }
}

/// SQL statement to create the cars table in MySQL.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year SMALLINT NOT NULL,
        make VARCHAR(25) NOT NULL,
        model VARCHAR(25) NOT NULL,
        description VARCHAR(25) NOT NULL
    );";

/// SQL statement to select all cars from the table.
#[allow(dead_code)]
pub const SELECT_ALL_CARS_SQL: &str = "SELECT year, make, model, description FROM cars";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_creation() {
        let car = Car::new(
            2021,
            "Mazda".to_string(),
            "CX-30".to_string(),
            "SUV Compact".to_string(),
        );
        assert_eq!(car.year, 2021);
        assert_eq!(car.make, "Mazda");
        assert_eq!(car.model, "CX-30");
        assert_eq!(car.description, "SUV Compact");
    }
}
```

- [ ] **Step 4: Verify helpers compile**

```bash
cargo test --features rdbc-sqlite helpers 2>&1 | tail -10
```

Expected: `test result: ok. 1 passed` (just `test_car_creation`)

- [ ] **Step 5: Commit**

```bash
git add tests/helpers/sqlite_helpers.rs tests/helpers/postgres_helpers.rs tests/helpers/mysql_helpers.rs
git commit -m "refactor(tests): remove CarItemBinder structs from test helpers"
```

---

## Task 8: Update integration tests — rewrite writer tests with `.column()`, add nullable tests

**Files:**
- Modify: `tests/rdbc_sqlite.rs`
- Modify: `tests/rdbc_postgres.rs`
- Modify: `tests/rdbc_mysql.rs`

Each of these files has one writer integration test that creates a binder, calls `.sqlite_binder()`/`.postgres_binder()`/`.mysql_binder()`, and uses `.add_column()`. Replace those calls with `.column()` chains.

- [ ] **Step 1: Update the SQLite writer integration test**

In `tests/rdbc_sqlite.rs`, find the import line:
```rust
sqlite_helpers::{CREATE_CARS_TABLE_SQL, Car, SELECT_ALL_CARS_SQL, SqliteCarItemBinder},
```
Change it to:
```rust
sqlite_helpers::{CREATE_CARS_TABLE_SQL, Car, SELECT_ALL_CARS_SQL},
```

Find the writer test body (the function that creates `SqliteCarItemBinder` and calls `.sqlite_binder()`). The old code looks like:
```rust
let item_binder = SqliteCarItemBinder;
let writer = RdbcItemWriterBuilder::<Car>::new()
    .sqlite(&pool)
    .table("cars")
    .add_column("year")
    .add_column("make")
    .add_column("model")
    .add_column("description")
    .sqlite_binder(&item_binder)
    .build_sqlite();
```

Replace with:
```rust
let writer = RdbcItemWriterBuilder::<Car>::new()
    .sqlite(&pool)
    .table("cars")
    .column("year", |c: &Car| (c.year as i32).into())
    .column("make", |c: &Car| c.make.as_str().into())
    .column("model", |c: &Car| c.model.as_str().into())
    .column("description", |c: &Car| c.description.as_str().into())
    .build_sqlite();
```

Also remove the `use spring_batch_rs::item::rdbc::DatabaseItemBinder;` import if present (check the top of the file).

- [ ] **Step 2: Add the nullable SQLite integration test**

At the end of `tests/rdbc_sqlite.rs`, add this test function (replace `my_test_fn` with an appropriate `#[tokio::test]` annotation matching the existing tests in the file):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn should_write_null_optional_column_to_sqlite() -> Result<(), testcontainers::TestcontainersError> {
    use spring_batch_rs::item::rdbc::{ColumnValue, RdbcItemWriterBuilder};

    // Uses in-memory SQLite — no container needed
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("CREATE TABLE notes (id INTEGER NOT NULL, body TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    #[derive(Clone, serde::Serialize)]
    struct Note { id: i32, body: Option<String> }

    let writer = RdbcItemWriterBuilder::<Note>::new()
        .sqlite(&pool)
        .table("notes")
        .column("id", |n: &Note| n.id.into())
        .column("body", |n: &Note| n.body.clone().into())
        .build_sqlite();

    use spring_batch_rs::core::item::ItemWriter;
    writer
        .write(&[Note { id: 1, body: None }, Note { id: 2, body: Some("hello".to_string()) }])
        .unwrap();

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(body.is_none(), "id=1 body should be NULL");

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(body.as_deref(), Some("hello"), "id=2 body should be 'hello'");

    Ok(())
}
```

- [ ] **Step 3: Update the Postgres writer integration test**

In `tests/rdbc_postgres.rs`, change the import:
```rust
postgres_helpers::{CREATE_CARS_TABLE_SQL, Car, PostgresCarItemBinder, SELECT_ALL_CARS_SQL},
```
to:
```rust
postgres_helpers::{CREATE_CARS_TABLE_SQL, Car, SELECT_ALL_CARS_SQL},
```

Replace the writer construction block (old code has `PostgresCarItemBinder`, `.postgres_binder()`, and `.add_column()`):
```rust
let writer = RdbcItemWriterBuilder::<Car>::new()
    .postgres(&pool)
    .table("cars")
    .column("year", |c: &Car| (c.year as i32).into())
    .column("make", |c: &Car| c.make.as_str().into())
    .column("model", |c: &Car| c.model.as_str().into())
    .column("description", |c: &Car| c.description.as_str().into())
    .build_postgres();
```

Add nullable integration test at the end of the file (uses the same testcontainers Postgres container pattern as the existing tests — copy the container setup from the existing write test):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn should_write_null_optional_column_to_postgres() -> Result<(), testcontainers::TestcontainersError> {
    use spring_batch_rs::item::rdbc::{ColumnValue, RdbcItemWriterBuilder};
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

    let container = postgres::Postgres::default().start().await?;
    let host_ip = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:postgres@{}:{}", host_ip, host_port);
    let pool = sqlx::PgPool::connect(&url).await.unwrap();

    sqlx::query("CREATE TABLE notes (id INT NOT NULL, body TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    #[derive(Clone, serde::Serialize)]
    struct Note { id: i32, body: Option<String> }

    let writer = RdbcItemWriterBuilder::<Note>::new()
        .postgres(&pool)
        .table("notes")
        .column("id", |n: &Note| n.id.into())
        .column("body", |n: &Note| n.body.clone().into())
        .build_postgres();

    use spring_batch_rs::core::item::ItemWriter;
    writer
        .write(&[Note { id: 1, body: None }, Note { id: 2, body: Some("pg".to_string()) }])
        .unwrap();

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(body.is_none(), "id=1 body should be NULL");

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(body.as_deref(), Some("pg"));

    Ok(())
}
```

- [ ] **Step 4: Update the MySQL writer integration test**

In `tests/rdbc_mysql.rs`, change the import:
```rust
mysql_helpers::{CREATE_CARS_TABLE_SQL, Car, MySqlCarItemBinder, SELECT_ALL_CARS_SQL},
```
to:
```rust
mysql_helpers::{CREATE_CARS_TABLE_SQL, Car, SELECT_ALL_CARS_SQL},
```

Replace the writer construction block:
```rust
let writer = RdbcItemWriterBuilder::<Car>::new()
    .mysql(&pool)
    .table("cars")
    .column("year", |c: &Car| (c.year as i32).into())
    .column("make", |c: &Car| c.make.as_str().into())
    .column("model", |c: &Car| c.model.as_str().into())
    .column("description", |c: &Car| c.description.as_str().into())
    .build_mysql();
```

Add nullable integration test (uses the same testcontainers MySQL container pattern as existing tests):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn should_write_null_optional_column_to_mysql() -> Result<(), testcontainers::TestcontainersError> {
    use spring_batch_rs::item::rdbc::{ColumnValue, RdbcItemWriterBuilder};
    use testcontainers_modules::{mysql, testcontainers::runners::AsyncRunner};
    let container = mysql::Mysql::default().start().await?;
    let host_ip = container.get_host().await?;
    let port = container.get_host_port_ipv4(3306).await?;
    let url = format!("mysql://{}:{}/test", host_ip, port);
    let pool = sqlx::MySqlPool::connect(&url).await.unwrap();

    sqlx::query("CREATE TABLE notes (id INT NOT NULL, body VARCHAR(255))")
        .execute(&pool)
        .await
        .unwrap();

    #[derive(Clone, serde::Serialize)]
    struct Note { id: i32, body: Option<String> }

    let writer = RdbcItemWriterBuilder::<Note>::new()
        .mysql(&pool)
        .table("notes")
        .column("id", |n: &Note| n.id.into())
        .column("body", |n: &Note| n.body.clone().into())
        .build_mysql();

    use spring_batch_rs::core::item::ItemWriter;
    writer
        .write(&[Note { id: 1, body: None }, Note { id: 2, body: Some("mysql".to_string()) }])
        .unwrap();

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(body.is_none(), "id=1 body should be NULL");

    let (body,): (Option<String>,) = sqlx::query_as("SELECT body FROM notes WHERE id = 2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(body.as_deref(), Some("mysql"));

    Ok(())
}
```

- [ ] **Step 5: Run SQLite integration tests (fast, no Docker)**

```bash
cargo test --test rdbc_sqlite --features rdbc-sqlite 2>&1 | tail -15
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add tests/rdbc_sqlite.rs tests/rdbc_postgres.rs tests/rdbc_mysql.rs
git commit -m "test(rdbc): rewrite writer integration tests with .column() API; add nullable tests"
```

---

## Task 9: Update examples — replace binder structs with `.column()` chains

**Files:**
- Modify: `examples/database_processing.rs`
- Modify: `examples/benchmark_csv_postgres_xml.rs`

- [ ] **Step 1: Update `examples/database_processing.rs`**

Remove the `use spring_batch_rs::item::rdbc::DatabaseItemBinder;` import and the `use sqlx::{..., query_builder::Separated};` import (keep other imports).

Delete the `UserBinder` struct and its `DatabaseItemBinder<User, Sqlite>` impl entirely.

Delete the `ProductBinder` struct and its `DatabaseItemBinder<Product, Sqlite>` impl entirely.

Find the User writer construction (currently uses `sqlite_binder(&binder)` and four `.add_column()` calls) and replace with:

```rust
// 1. Build writer — maps User fields to SQLite columns
let writer = RdbcItemWriterBuilder::<User>::new()
    .sqlite(&pool)
    .table("users")
    .column("id", |u: &User| u.id.into())
    .column("name", |u: &User| u.name.as_str().into())
    .column("email", |u: &User| u.email.as_str().into())
    .column("active", |u: &User| u.active.into())
    .build_sqlite();
```

Find the Product writer construction and replace with:

```rust
// 1. Build writer — maps Product fields to SQLite columns
let writer = RdbcItemWriterBuilder::<Product>::new()
    .sqlite(&pool)
    .table("products")
    .column("id", |p: &Product| p.id.into())
    .column("name", |p: &Product| p.name.as_str().into())
    .column("price", |p: &Product| p.price.into())
    .column("stock", |p: &Product| p.stock.into())
    .build_sqlite();
```

Also remove `use spring_batch_rs::item::rdbc::ColumnValue` — it's not needed since `Into<ColumnValue>` is called via `.into()` and the compiler infers the type from the return type of the closure (which must be `ColumnValue`). No explicit import needed unless used directly.

Actually — the closures return `ColumnValue` so `ColumnValue` must be in scope. Add the import:
```rust
use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, RdbcItemWriterBuilder};
```
(The existing import likely already has `RdbcItemWriterBuilder` — just remove `DatabaseItemBinder` from it.)

- [ ] **Step 2: Update `examples/benchmark_csv_postgres_xml.rs`**

Remove `use spring_batch_rs::item::rdbc::DatabaseItemBinder;` from the import.

Delete the `TransactionBinder` struct and its `DatabaseItemBinder<Transaction, Postgres>` impl entirely.

Delete the `TransactionImportBinder` struct and its `DatabaseItemBinder<Transaction, Postgres>` impl entirely.

Find the first Transaction writer construction (step 2, writes to `transactions` table — currently has `TransactionBinder`, `.postgres_binder(&binder)`, and eight `.add_column()` calls). Replace with:

```rust
let writer = RdbcItemWriterBuilder::<Transaction>::new()
    .postgres(&pg_pool)
    .table("transactions")
    .column("transaction_id", |t: &Transaction| t.transaction_id.as_str().into())
    .column("amount", |t: &Transaction| t.amount.into())
    .column("currency", |t: &Transaction| t.currency.as_str().into())
    .column("timestamp", |t: &Transaction| t.timestamp.as_str().into())
    .column("account_from", |t: &Transaction| t.account_from.as_str().into())
    .column("account_to", |t: &Transaction| t.account_to.as_str().into())
    .column("status", |t: &Transaction| t.status.as_str().into())
    .column("amount_eur", |t: &Transaction| t.amount_eur.into())
    .build_postgres();
```

Find the second Transaction writer construction (step 3, writes to `transactions_import` table — currently has `TransactionImportBinder`). Replace with:

```rust
let writer = RdbcItemWriterBuilder::<Transaction>::new()
    .postgres(&pg_pool)
    .table("transactions_import")
    .column("transaction_id", |t: &Transaction| t.transaction_id.as_str().into())
    .column("amount", |t: &Transaction| t.amount.into())
    .column("currency", |t: &Transaction| t.currency.as_str().into())
    .column("timestamp", |t: &Transaction| t.timestamp.as_str().into())
    .column("account_from", |t: &Transaction| t.account_from.as_str().into())
    .column("account_to", |t: &Transaction| t.account_to.as_str().into())
    .column("status", |t: &Transaction| t.status.as_str().into())
    .column("amount_eur", |t: &Transaction| t.amount_eur.into())
    .build_postgres();
```

- [ ] **Step 3: Verify examples compile**

```bash
cargo build --example database_processing --features rdbc-sqlite,logger 2>&1 | tail -10
cargo build --example benchmark_csv_postgres_xml --features rdbc-postgres,csv,xml,logger 2>&1 | tail -10
```

Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add examples/database_processing.rs examples/benchmark_csv_postgres_xml.rs
git commit -m "feat(examples): replace DatabaseItemBinder with .column() API"
```

---

## Task 10: Final compile and test verification

- [ ] **Step 1: Run full unit test suite**

```bash
cargo test --features rdbc-sqlite,rdbc-postgres,rdbc-mysql 2>&1 | tail -20
```

Expected: all unit tests pass, no compile errors.

- [ ] **Step 2: Verify rustdoc builds clean**

```bash
cargo doc --no-deps --features rdbc-sqlite,rdbc-postgres,rdbc-mysql 2>&1 | grep -E "warning|error" | head -20
```

Expected: no warnings, no errors.

- [ ] **Step 3: Run clippy**

```bash
cargo clippy --features rdbc-sqlite,rdbc-postgres,rdbc-mysql -- -D warnings 2>&1 | tail -20
```

Expected: no warnings.

- [ ] **Step 4: Run rustfmt check**

```bash
cargo fmt --all -- --check 2>&1 | tail -10
```

If any formatting issues: run `cargo fmt --all` then re-check.

- [ ] **Step 5: Commit format fixes if needed**

```bash
git add -p
git commit -m "style: rustfmt formatting"
```
