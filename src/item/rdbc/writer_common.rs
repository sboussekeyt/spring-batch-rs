//! Common functionality for database item writers.
//!
//! This module provides shared utilities and constants used across all database-specific
//! item writers (PostgreSQL, MySQL, SQLite) to reduce code duplication and ensure
//! consistent behavior.

use crate::BatchError;
use sqlx::{Database, Pool};

/// The maximum number of parameters that can be bound in a single SQL statement.
/// This is the most conservative limit across major databases (MySQL's limit).
pub const BIND_LIMIT: usize = 65535;

/// Type alias for the validated configuration tuple returned by `validate_config`.
type ValidatedConfig<'a, O, DB> = (
    &'a Pool<DB>,
    &'a str,
    &'a dyn super::DatabaseItemBinder<O, DB>,
);

/// Validates that all required writer configuration fields are set.
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `table` - The table name
/// * `columns` - The list of column names
/// * `item_binder` - The item binder
///
/// # Returns
///
/// A tuple of validated references, or an error if any required field is missing.
pub fn validate_config<'a, O, DB: Database>(
    pool: Option<&'a Pool<DB>>,
    table: Option<&'a str>,
    columns: &[&'a str],
    item_binder: Option<&'a dyn super::DatabaseItemBinder<O, DB>>,
) -> Result<ValidatedConfig<'a, O, DB>, BatchError> {
    if columns.is_empty() {
        return Err(BatchError::ItemWriter(
            "No columns specified for database write".to_string(),
        ));
    }

    let pool =
        pool.ok_or_else(|| BatchError::ItemWriter("Database pool not configured".to_string()))?;

    let table =
        table.ok_or_else(|| BatchError::ItemWriter("Table name not configured".to_string()))?;

    let item_binder = item_binder
        .ok_or_else(|| BatchError::ItemWriter("Item binder not configured".to_string()))?;

    Ok((pool, table, item_binder))
}

/// Logs a successful write operation.
///
/// # Arguments
///
/// * `items_count` - The number of items written
/// * `table` - The table name
/// * `db_name` - The database name (e.g., "PostgreSQL", "MySQL", "SQLite")
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
/// # Arguments
///
/// * `table` - The table name
/// * `db_name` - The database name
/// * `error` - The underlying error
///
/// # Returns
///
/// A `BatchError::ItemWriter` with formatted error message.
pub fn create_write_error(table: &str, db_name: &str, error: impl std::fmt::Display) -> BatchError {
    log::error!(
        "Failed to write items to {} table {}: {}",
        db_name,
        table,
        error
    );
    BatchError::ItemWriter(format!("{} write failed: {}", db_name, error))
}

/// Calculates the maximum number of items that can be written in a single batch
/// based on the bind limit and number of columns.
///
/// # Arguments
///
/// * `column_count` - The number of columns in the table
///
/// # Returns
///
/// The maximum number of items per batch.
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
        let result = validate_config::<String, Sqlite>(None, Some("tbl"), &[], None);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_is_missing() {
        // columns non-empty, pool = None → "pool not configured"
        let result = validate_config::<String, Sqlite>(None, Some("tbl"), &["col"], None);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_error_when_table_is_missing() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let result = validate_config::<String, Sqlite>(Some(&pool), None, &["col"], None);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("Table"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_error_when_binder_is_missing() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let result = validate_config::<String, Sqlite>(Some(&pool), Some("tbl"), &["col"], None);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("binder"), "unexpected: {msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_ok_when_all_config_provided() {
        use crate::item::rdbc::DatabaseItemBinder;
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, Sqlite> for DummyBinder {
            fn bind(&self, _: &String, _: Separated<Sqlite, &str>) {}
        }

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let binder = DummyBinder;
        let result = validate_config::<String, Sqlite>(
            Some(&pool),
            Some("tbl"),
            &["col"],
            Some(&binder as &dyn DatabaseItemBinder<String, Sqlite>),
        );
        assert!(
            result.is_ok(),
            "should return Ok when all config is provided"
        );
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
