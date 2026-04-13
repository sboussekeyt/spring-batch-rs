use serde::Serialize;
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a PostgreSQL database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for PostgreSQL operations.
/// It supports batch inserting items into a specified table with the provided columns.
///
/// # PostgreSQL-Specific Features
///
/// - Supports PostgreSQL's advanced data types (JSON, arrays, custom types)
/// - Handles PostgreSQL's SERIAL and BIGSERIAL for auto-incrementing columns
/// - Supports PostgreSQL's UPSERT operations with ON CONFLICT clauses
/// - Leverages PostgreSQL's efficient bulk insert capabilities
/// - Compatible with PostgreSQL's connection pooling and prepared statements
///
/// # Construction
///
/// This writer can only be created through `RdbcItemWriterBuilder`.
/// Direct construction is not available to ensure proper configuration.
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use spring_batch_rs::core::item::ItemWriter;
/// use sqlx::{PgPool, query_builder::Separated, Postgres};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// struct UserBinder;
/// impl DatabaseItemBinder<User, Postgres> for UserBinder {
///     fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.name);
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
/// let binder = UserBinder;
///
/// let writer = RdbcItemWriterBuilder::<User>::new()
///     .postgres(&pool)
///     .table("users")
///     .add_column("id")
///     .add_column("name")
///     .postgres_binder(&binder)
///     .build_postgres();
///
/// let users = vec![
///     User { id: 1, name: "Alice".to_string() },
///     User { id: 2, name: "Bob".to_string() },
/// ];
///
/// writer.write(&users)?;
/// # Ok(())
/// # }
/// ```
pub struct PostgresItemWriter<'a, O> {
    pub(crate) pool: Option<&'a Pool<Postgres>>,
    pub(crate) table: Option<&'a str>,
    pub(crate) columns: Vec<&'a str>,
    pub(crate) item_binder: Option<&'a dyn DatabaseItemBinder<O, Postgres>>,
}

impl<'a, O> PostgresItemWriter<'a, O> {
    /// Creates a new `PostgresItemWriter` with default configuration.
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemWriterBuilder` for creating writer instances.
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    /// Sets the database connection pool for the writer.
    pub(crate) fn pool(mut self, pool: &'a Pool<Postgres>) -> Self {
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
        item_binder: &'a dyn DatabaseItemBinder<O, Postgres>,
    ) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<'a, O> Default for PostgresItemWriter<'a, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for PostgresItemWriter<'_, O> {
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
    use crate::core::item::ItemWriter;

    #[test]
    fn test_new_creates_default_writer() {
        let writer = PostgresItemWriter::<String>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[test]
    fn test_builder_pattern_configuration() {
        let writer = PostgresItemWriter::<String>::new()
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email");

        assert_eq!(writer.table, Some("users"));
        assert_eq!(writer.columns, vec!["id", "name", "email"]);
    }

    #[test]
    fn test_write_empty_items() {
        use crate::item::rdbc::DatabaseItemBinder;
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, Postgres> for DummyBinder {
            fn bind(&self, _item: &String, _query_builder: Separated<Postgres, &str>) {}
        }

        let binder = DummyBinder;
        let writer = PostgresItemWriter::<String>::new()
            .table("test")
            .add_column("value")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_columns_missing_and_items_given() {
        use crate::{BatchError, item::rdbc::DatabaseItemBinder};
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, Postgres> for DummyBinder {
            fn bind(&self, _: &String, _: Separated<Postgres, &str>) {}
        }
        let binder = DummyBinder;
        let writer = PostgresItemWriter::<String>::new()
            .table("t")
            .item_binder(&binder); // no columns

        let result = writer.write(&["x".to_string()]);
        assert!(result.is_err(), "expected error for missing columns");
        match result.unwrap_err() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("columns"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_return_error_when_pool_not_configured_and_items_given() {
        use crate::{BatchError, item::rdbc::DatabaseItemBinder};
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, Postgres> for DummyBinder {
            fn bind(&self, _: &String, _: Separated<Postgres, &str>) {}
        }
        let binder = DummyBinder;
        let writer = PostgresItemWriter::<String>::new()
            .table("t")
            .add_column("v")
            .item_binder(&binder); // no pool

        let result = writer.write(&["x".to_string()]);
        assert!(result.is_err(), "expected error for missing pool");
        match result.unwrap_err() {
            BatchError::ItemWriter(msg) => assert!(msg.contains("pool"), "{msg}"),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }
}
