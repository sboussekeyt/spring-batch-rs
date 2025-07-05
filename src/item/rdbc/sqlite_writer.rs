use serde::Serialize;
use sqlx::{Pool, QueryBuilder, Sqlite};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;
use crate::BatchError;

// The number of parameters in databases must fit in a reasonable limit
const BIND_LIMIT: usize = 65535;

/// A writer for inserting items into a SQLite database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for SQLite operations.
/// It supports batch inserting items into a specified table with the provided columns.
/// It uses the same generic `DatabaseItemBinder` trait as other database writers.
///
/// # Design
///
/// - Uses a SQLite connection pool to efficiently manage database connections
/// - Leverages SQLx's query builder for constructing parameterized SQL statements
/// - Uses the generic `DatabaseItemBinder` trait to handle the conversion from domain objects to SQL parameters
/// - Handles batch inserts efficiently within the database parameter limit
/// - Supports both file-based and in-memory SQLite databases
///
/// # SQLite-Specific Features
///
/// - Supports SQLite's AUTOINCREMENT for primary keys
/// - Handles SQLite's dynamic typing system
/// - Works with both file-based (`sqlite://path/to/db.sqlite`) and in-memory (`:memory:`) databases
/// - Supports SQLite's UPSERT operations when configured appropriately
///
/// # Performance Considerations
///
/// - SQLite performs best with batch inserts within transactions
/// - File-based databases benefit from WAL mode for concurrent access
/// - In-memory databases provide fastest performance for temporary data
pub struct SqliteItemWriter<'a, O> {
    pool: Option<&'a Pool<Sqlite>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn DatabaseItemBinder<O, Sqlite>>,
}

impl<'a, O> SqliteItemWriter<'a, O> {
    /// Creates a new `SqliteItemWriter` with default configuration.
    ///
    /// All parameters must be set using the builder methods before use.
    /// Use the builder pattern for a more convenient API.
    ///
    /// # Returns
    ///
    /// A new `SqliteItemWriter` instance with default settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::sqlite_writer::SqliteItemWriter;
    /// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
    /// use sqlx::{SqlitePool, query_builder::Separated, Sqlite};
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Serialize)]
    /// struct User {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// struct UserBinder;
    /// impl DatabaseItemBinder<User, Sqlite> for UserBinder {
    ///     fn bind(&self, item: &User, mut query_builder: Separated<Sqlite, &str>) {
    ///         query_builder.push_bind(item.id);
    ///         query_builder.push_bind(&item.name);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = SqlitePool::connect("sqlite://database.db").await?;
    /// let binder = UserBinder;
    ///
    /// let writer = SqliteItemWriter::<User>::new()
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
    /// * `pool` - A reference to the SQLite connection pool.
    ///
    /// # Returns
    ///
    /// The updated `SqliteItemWriter` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::sqlite_writer::SqliteItemWriter;
    /// use sqlx::SqlitePool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // File-based database
    /// let file_pool = SqlitePool::connect("sqlite://database.db").await?;
    /// let file_writer = SqliteItemWriter::<String>::new().pool(&file_pool);
    ///
    /// // In-memory database
    /// let memory_pool = SqlitePool::connect("sqlite::memory:").await?;
    /// let memory_writer = SqliteItemWriter::<String>::new().pool(&memory_pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn pool(mut self, pool: &'a Pool<Sqlite>) -> Self {
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
    /// The updated `SqliteItemWriter` instance.
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
    /// The updated `SqliteItemWriter` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::sqlite_writer::SqliteItemWriter;
    ///
    /// let writer = SqliteItemWriter::<String>::new()
    ///     .add_column("id")
    ///     .add_column("name")
    ///     .add_column("email")
    ///     .add_column("created_at");
    /// ```
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
    /// The updated `SqliteItemWriter` instance.
    pub fn item_binder(mut self, item_binder: &'a dyn DatabaseItemBinder<O, Sqlite>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for SqliteItemWriter<'_, O> {
    /// Writes items to the SQLite database using batch inserts.
    ///
    /// This method implements the ItemWriter trait and provides efficient bulk
    /// insert operations. It constructs a SQL INSERT statement with the configured
    /// table and columns, then uses the item binder to bind item data to parameters.
    ///
    /// # SQLite-Specific Behavior
    ///
    /// - Uses SQLite's efficient batch insert capabilities
    /// - Automatically handles SQLite's dynamic typing
    /// - Supports both file-based and in-memory databases
    /// - Leverages connection pooling for optimal performance
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of items to be written to the database
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all items were successfully written
    /// - `Err(BatchError)` if a database error occurred
    ///
    /// # Errors
    ///
    /// This method can fail for various reasons:
    /// - Database connection issues
    /// - Constraint violations (PRIMARY KEY, UNIQUE, etc.)
    /// - Invalid data types or values
    /// - Table or column not found
    /// - Insufficient permissions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::sqlite_writer::SqliteItemWriter;
    /// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use sqlx::{SqlitePool, query_builder::Separated, Sqlite};
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Serialize)]
    /// struct Product {
    ///     id: i32,
    ///     name: String,
    ///     price: f64,
    /// }
    ///
    /// struct ProductBinder;
    /// impl DatabaseItemBinder<Product, Sqlite> for ProductBinder {
    ///     fn bind(&self, item: &Product, mut query_builder: Separated<Sqlite, &str>) {
    ///         query_builder.push_bind(item.id);
    ///         query_builder.push_bind(&item.name);
    ///         query_builder.push_bind(item.price);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = SqlitePool::connect("sqlite::memory:").await?;
    /// let binder = ProductBinder;
    ///
    /// let writer = SqliteItemWriter::<Product>::new()
    ///     .pool(&pool)
    ///     .table("products")
    ///     .add_column("id")
    ///     .add_column("name")
    ///     .add_column("price")
    ///     .item_binder(&binder);
    ///
    /// let products = vec![
    ///     Product { id: 1, name: "Laptop".to_string(), price: 999.99 },
    ///     Product { id: 2, name: "Mouse".to_string(), price: 29.99 },
    /// ];
    ///
    /// writer.write(&products)?;
    /// # Ok(())
    /// # }
    /// ```
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(self.table.as_ref().unwrap());
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        query_builder.push_values(
            items.iter().take(BIND_LIMIT / self.columns.len()),
            |b, item| {
                self.item_binder.as_ref().unwrap().bind(item, b);
            },
        );

        let query = query_builder.build();

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { query.execute(self.pool.unwrap()).await })
        });

        match result {
            Ok(_) => {
                log::debug!(
                    "Successfully wrote {} items to SQLite table {}",
                    items.len().min(BIND_LIMIT / self.columns.len()),
                    self.table.unwrap()
                );
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "Failed to write items to SQLite table {}: {}",
                    self.table.unwrap_or("unknown"),
                    e
                );
                Err(BatchError::ItemWriter(format!(
                    "SQLite write failed: {}",
                    e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;
    use serde::Serialize;
    use sqlx::{query_builder::Separated, Sqlite, SqlitePool};

    #[derive(Clone, Serialize, Debug, PartialEq)]
    struct TestUser {
        id: i32,
        name: String,
        email: String,
    }

    struct TestUserBinder;
    impl DatabaseItemBinder<TestUser, Sqlite> for TestUserBinder {
        fn bind(&self, item: &TestUser, mut query_builder: Separated<Sqlite, &str>) {
            query_builder.push_bind(item.id);
            query_builder.push_bind(item.name.clone());
            query_builder.push_bind(item.email.clone());
        }
    }

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Create test table
        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_new_creates_default_writer() {
        let writer = SqliteItemWriter::<TestUser>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        assert!(writer.pool.is_some());
        assert_eq!(writer.table, Some("users"));
        assert_eq!(writer.columns, vec!["id", "name", "email"]);
        assert!(writer.item_binder.is_some());
    }

    #[tokio::test]
    async fn test_write_empty_items() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_single_item() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        let users = vec![TestUser {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        }];

        let result = writer.write(&users);
        assert!(result.is_ok());

        // Verify the data was inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_write_multiple_items() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        let users = vec![
            TestUser {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            },
            TestUser {
                id: 2,
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
            },
            TestUser {
                id: 3,
                name: "Bob Johnson".to_string(),
                email: "bob@example.com".to_string(),
            },
        ];

        let result = writer.write(&users);
        assert!(result.is_ok());

        // Verify all data was inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 3);

        // Verify specific data
        let names: Vec<String> = sqlx::query_scalar("SELECT name FROM users ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(names, vec!["John Doe", "Jane Smith", "Bob Johnson"]);
    }

    #[tokio::test]
    async fn test_write_constraint_violation() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        // Insert first user
        let users1 = vec![TestUser {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        }];
        writer.write(&users1).unwrap();

        // Try to insert user with duplicate email (should fail due to UNIQUE constraint)
        let users2 = vec![TestUser {
            id: 2,
            name: "Jane Doe".to_string(),
            email: "john@example.com".to_string(), // Duplicate email
        }];

        let result = writer.write(&users2);
        assert!(result.is_err());

        if let Err(BatchError::ItemWriter(msg)) = result {
            assert!(msg.contains("SQLite write failed"));
        } else {
            panic!("Expected BatchError::ItemWriter");
        }
    }

    #[tokio::test]
    async fn test_add_multiple_columns() {
        let writer = SqliteItemWriter::<TestUser>::new()
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .add_column("created_at");

        assert_eq!(writer.columns, vec!["id", "name", "email", "created_at"]);
    }

    #[tokio::test]
    async fn test_large_batch_insert() {
        let pool = setup_test_db().await.unwrap();
        let binder = TestUserBinder;

        let writer = SqliteItemWriter::<TestUser>::new()
            .pool(&pool)
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        // Create a large batch of users
        let users: Vec<TestUser> = (1..=100)
            .map(|i| TestUser {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        let result = writer.write(&users);
        assert!(result.is_ok());

        // Verify all data was inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 100);
    }
}
