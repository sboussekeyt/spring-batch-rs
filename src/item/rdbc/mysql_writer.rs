use serde::Serialize;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;
use crate::BatchError;

// The number of parameters in MySQL must fit in a reasonable limit
const BIND_LIMIT: usize = 65535;

/// A writer for inserting items into a MySQL database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for MySQL operations.
/// It supports batch inserting items into a specified table with the provided columns.
/// It uses the same generic `DatabaseItemBinder` trait as the PostgreSQL writer.
///
/// # Design
///
/// - Uses a MySQL connection pool to efficiently manage database connections
/// - Leverages SQLx's query builder for constructing parameterized SQL statements
/// - Uses the generic `DatabaseItemBinder` trait to handle the conversion from domain objects to SQL parameters
/// - Handles batch inserts efficiently within the database parameter limit
pub struct MySqlItemWriter<'a, O> {
    pool: Option<&'a Pool<MySql>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn DatabaseItemBinder<O, MySql>>,
}

impl<'a, O> MySqlItemWriter<'a, O> {
    /// Creates a new `MySqlItemWriter` with default configuration.
    ///
    /// All parameters must be set using the builder methods before use.
    /// Use the builder pattern for a more convenient API.
    ///
    /// # Returns
    ///
    /// A new `MySqlItemWriter` instance with default settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::mysql_writer::MySqlItemWriter;
    /// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
    /// use sqlx::{MySqlPool, query_builder::Separated, MySql};
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Serialize)]
    /// struct User {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// struct UserBinder;
    /// impl DatabaseItemBinder<User, MySql> for UserBinder {
    ///     fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
    ///         query_builder.push_bind(item.id);
    ///         query_builder.push_bind(&item.name);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
    /// let binder = UserBinder;
    ///
    /// let writer = MySqlItemWriter::<User>::new()
    ///     .pool(&pool)
    ///     .table("users")
    ///     .add_column("id")
    ///     .add_column("name")
    ///     .item_binder(&binder);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    /// Sets the database connection pool for the writer.
    ///
    /// This is a required parameter that must be set before using the writer.
    ///
    /// # Arguments
    ///
    /// * `pool` - A reference to the connection pool.
    ///
    /// # Returns
    ///
    /// The updated `MySqlItemWriter` instance.
    pub fn pool(mut self, pool: &'a Pool<MySql>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the table name for the writer.
    ///
    /// This is a required parameter that must be set before using the writer.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the database table.
    ///
    /// # Returns
    ///
    /// The updated `MySqlItemWriter` instance.
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// Adds a column to the writer.
    ///
    /// This method can be called multiple times to add multiple columns.
    /// At least one column must be added before using the writer.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to add.
    ///
    /// # Returns
    ///
    /// The updated `MySqlItemWriter` instance.
    pub fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the item binder for the writer.
    ///
    /// This is a required parameter that must be set before using the writer.
    /// The item binder is responsible for mapping item properties to SQL parameters.
    ///
    /// # Arguments
    ///
    /// * `item_binder` - A reference to the item binder.
    ///
    /// # Returns
    ///
    /// The updated `MySqlItemWriter` instance.
    pub fn item_binder(mut self, item_binder: &'a dyn DatabaseItemBinder<O, MySql>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for MySqlItemWriter<'_, O> {
    /// Writes items to the MySQL database using batch inserts.
    ///
    /// This method implements the ItemWriter trait and provides efficient bulk
    /// insert operations. It constructs a SQL INSERT statement with the configured
    /// table and columns, then uses the item binder to bind item data to parameters.
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of items to be written to the database
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all items were successfully written
    /// - `Err(BatchError)` if a database error occurred
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        // Build the base INSERT statement with table and column names
        let mut query_builder = QueryBuilder::new("INSERT INTO ");

        query_builder.push(self.table.as_ref().unwrap());
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        // Add VALUES clause with proper parameter binding for each item
        // Limit the number of items to stay within database parameter limits
        query_builder.push_values(
            items.iter().take(BIND_LIMIT / self.columns.len()),
            |b, item| {
                self.item_binder.as_ref().unwrap().bind(item, b);
            },
        );

        let query = query_builder.build();

        // Execute the query in a blocking manner
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { query.execute(self.pool.unwrap()).await })
        });

        match result {
            Ok(_) => {
                log::debug!(
                    "Successfully wrote {} items to MySQL table {}",
                    items.len().min(BIND_LIMIT / self.columns.len()),
                    self.table.unwrap()
                );
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "Failed to write items to MySQL table {}: {}",
                    self.table.unwrap_or("unknown"),
                    e
                );
                Err(BatchError::ItemWriter(format!("MySQL write failed: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;
    use serde::Serialize;
    use sqlx::{query_builder::Separated, MySql};

    #[derive(Clone, Serialize, Debug, PartialEq)]
    struct TestUser {
        id: i32,
        name: String,
        email: String,
    }

    struct TestUserBinder;
    impl DatabaseItemBinder<TestUser, MySql> for TestUserBinder {
        fn bind(&self, item: &TestUser, mut query_builder: Separated<MySql, &str>) {
            query_builder.push_bind(item.id);
            query_builder.push_bind(item.name.clone());
            query_builder.push_bind(item.email.clone());
        }
    }

    #[tokio::test]
    async fn test_new_creates_default_writer() {
        let writer = MySqlItemWriter::<TestUser>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern_configuration() {
        // This test doesn't require an actual database connection
        // We're just testing the builder pattern configuration

        let writer = MySqlItemWriter::<TestUser>::new()
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email");

        assert_eq!(writer.table, Some("users"));
        assert_eq!(writer.columns, vec!["id", "name", "email"]);
    }

    #[tokio::test]
    async fn test_add_multiple_columns() {
        let writer = MySqlItemWriter::<TestUser>::new()
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .add_column("created_at");

        assert_eq!(writer.columns, vec!["id", "name", "email", "created_at"]);
    }

    #[tokio::test]
    async fn test_write_empty_items() {
        // Create writer without pool for this simple test
        let binder = TestUserBinder;
        let writer = MySqlItemWriter::<TestUser>::new()
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }

    // Note: Full database integration tests would require a running MySQL instance
    // and are better suited for the integration test suite
}
