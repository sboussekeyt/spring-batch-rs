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
    pub(crate) pool: Option<sqlx::Pool<Sqlite>>,
    pub(crate) table: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(crate) column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>,
}

impl<O> SqliteItemWriter<O> {
    /// Creates a new `SqliteItemWriter` with default configuration.
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            column_bindings: Vec::new(),
        }
    }

    /// Sets the database connection pool for the writer.
    pub(crate) fn pool(mut self, pool: &Pool<Sqlite>) -> Self {
        self.pool = Some(pool.clone());
        self
    }

    /// Sets the table name for the writer.
    pub(crate) fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Adds a column binding to the writer.
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

        let col_names: Vec<&str> = self
            .column_bindings
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();

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
                    ColumnValue::Int(v) => {
                        b.push_bind(v);
                    }
                    ColumnValue::Float(v) => {
                        b.push_bind(v);
                    }
                    ColumnValue::Text(v) => {
                        b.push_bind(v);
                    }
                    ColumnValue::Bool(v) => {
                        b.push_bind(v);
                    }
                    ColumnValue::Bytes(v) => {
                        b.push_bind(v);
                    }
                    ColumnValue::Null => {
                        b.push_bind(Option::<String>::None);
                    }
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
        let names: Vec<&str> = writer
            .column_bindings
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["a", "b"],
            "bindings should preserve insertion order"
        );
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

        writer
            .write(&["hello".to_string(), "world".to_string()])
            .unwrap();

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
        struct Row {
            id: i32,
            note: Option<String>,
        }

        let writer = SqliteItemWriter::<Row>::new()
            .pool(&pool)
            .table("t")
            .add_column_binding("id".to_string(), Box::new(|r: &Row| r.id.into()))
            .add_column_binding(
                "note".to_string(),
                Box::new(|r: &Row| r.note.clone().into()),
            );

        writer.write(&[Row { id: 1, note: None }]).unwrap();

        let (note,): (Option<String>,) = sqlx::query_as("SELECT note FROM t WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(note.is_none(), "note should be NULL in the database");
    }
}
