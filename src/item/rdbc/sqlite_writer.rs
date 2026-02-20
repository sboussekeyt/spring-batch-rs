use serde::Serialize;
use sqlx::{Pool, QueryBuilder, Sqlite};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a SQLite database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for SQLite operations.
/// It supports batch inserting items into a specified table with the provided columns.
///
/// # SQLite-Specific Features
///
/// - Supports SQLite's flexible type system
/// - Handles SQLite's AUTOINCREMENT for auto-incrementing columns
/// - Supports SQLite's INSERT OR REPLACE operations
/// - Leverages SQLite's efficient bulk insert capabilities
/// - Compatible with SQLite's connection pooling and prepared statements
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use spring_batch_rs::core::item::ItemWriter;
/// use sqlx::{SqlitePool, query_builder::Separated, Sqlite};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Task {
///     id: i32,
///     title: String,
///     completed: bool,
/// }
///
/// struct TaskBinder;
/// impl DatabaseItemBinder<Task, Sqlite> for TaskBinder {
///     fn bind(&self, item: &Task, mut query_builder: Separated<Sqlite, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.title);
///         // In real usage: query_builder.push_bind(item.completed);
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
/// let binder = TaskBinder;
///
/// let writer = RdbcItemWriterBuilder::<Task>::new()
///     .sqlite(&pool)
///     .table("tasks")
///     .add_column("id")
///     .add_column("title")
///     .add_column("completed")
///     .sqlite_binder(&binder)
///     .build_sqlite();
///
/// let tasks = vec![
///     Task { id: 1, title: "Task 1".to_string(), completed: false },
///     Task { id: 2, title: "Task 2".to_string(), completed: true },
/// ];
///
/// writer.write(&tasks)?;
/// # Ok(())
/// # }
/// ```
pub struct SqliteItemWriter<'a, O> {
    pool: Option<&'a Pool<Sqlite>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn DatabaseItemBinder<O, Sqlite>>,
}

impl<'a, O> SqliteItemWriter<'a, O> {
    /// Creates a new `SqliteItemWriter` with default configuration.
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    /// Sets the database connection pool for the writer.
    pub(crate) fn pool(mut self, pool: &'a Pool<Sqlite>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the table name for the writer.
    pub(crate) fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// Adds a column to the writer.
    pub(crate) fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the item binder for the writer.
    pub(crate) fn item_binder(
        mut self,
        item_binder: &'a dyn DatabaseItemBinder<O, Sqlite>,
    ) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<'a, O> Default for SqliteItemWriter<'a, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for SqliteItemWriter<'_, O> {
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        // Validate configuration
        let (pool, table, item_binder) =
            validate_config(self.pool, self.table, &self.columns, self.item_binder)?;

        // Build INSERT query
        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(table);
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        // Calculate max items per batch and add values
        let max_items = max_items_per_batch(self.columns.len());
        let items_to_write = items.iter().take(max_items);
        let items_count = items_to_write.len();

        query_builder.push_values(items_to_write, |b, item| {
            item_binder.bind(item, b);
        });

        // Execute query inline (QueryBuilder lifetime requires this to be in same scope)
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

    #[test]
    fn test_new_creates_default_writer() {
        let writer = SqliteItemWriter::<String>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[test]
    fn test_builder_pattern_configuration() {
        let writer = SqliteItemWriter::<String>::new()
            .table("tasks")
            .add_column("id")
            .add_column("title")
            .add_column("completed");

        assert_eq!(writer.table, Some("tasks"));
        assert_eq!(writer.columns, vec!["id", "title", "completed"]);
    }

    #[test]
    fn test_write_empty_items() {
        use crate::item::rdbc::DatabaseItemBinder;
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, Sqlite> for DummyBinder {
            fn bind(&self, _item: &String, _query_builder: Separated<Sqlite, &str>) {}
        }

        let binder = DummyBinder;
        let writer = SqliteItemWriter::<String>::new()
            .table("test")
            .add_column("value")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }
}
