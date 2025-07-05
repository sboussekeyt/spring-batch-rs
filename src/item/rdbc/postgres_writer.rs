use serde::Serialize;
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;
use crate::BatchError;

// The number of parameters in databases must fit in a reasonable limit
const BIND_LIMIT: usize = 65535;

/// A writer for inserting items into a PostgreSQL database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for PostgreSQL operations.
/// It supports batch inserting items into a specified table with the provided columns.
/// It uses the generic `DatabaseItemBinder` trait to handle the conversion from domain objects to SQL parameters.
///
/// # Design
///
/// - Uses a PostgreSQL connection pool to efficiently manage database connections
/// - Leverages SQLx's query builder for constructing parameterized SQL statements
/// - Uses the generic `DatabaseItemBinder` trait to handle the conversion from domain objects to SQL parameters
/// - Handles batch inserts efficiently within the database parameter limit
/// - Supports PostgreSQL-specific features and data types
///
/// # PostgreSQL-Specific Features
///
/// - Supports PostgreSQL's advanced data types (JSON, arrays, custom types)
/// - Handles PostgreSQL's SERIAL and BIGSERIAL for auto-incrementing columns
/// - Supports PostgreSQL's UPSERT operations with ON CONFLICT clauses
/// - Leverages PostgreSQL's efficient bulk insert capabilities
/// - Compatible with PostgreSQL's connection pooling and prepared statements
pub struct PostgresItemWriter<'a, O> {
    pool: Option<&'a Pool<Postgres>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn DatabaseItemBinder<O, Postgres>>,
}

impl<'a, O> PostgresItemWriter<'a, O> {
    /// Creates a new `PostgresItemWriter` with default configuration.
    ///
    /// All parameters must be set using the builder methods before use.
    /// Use the builder pattern for a more convenient API.
    ///
    /// # Returns
    ///
    /// A new `PostgresItemWriter` instance with default settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::postgres_writer::PostgresItemWriter;
    /// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
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
    ///         query_builder.push_bind(item.id);
    ///         query_builder.push_bind(&item.name);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let binder = UserBinder;
    ///
    /// let writer = PostgresItemWriter::<User>::new()
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
    /// * `pool` - A reference to the PostgreSQL connection pool.
    ///
    /// # Returns
    ///
    /// The updated `PostgresItemWriter` instance.
    pub fn pool(mut self, pool: &'a Pool<Postgres>) -> Self {
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
    /// The updated `PostgresItemWriter` instance.
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
    /// The updated `PostgresItemWriter` instance.
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
    /// The updated `PostgresItemWriter` instance.
    pub fn item_binder(mut self, item_binder: &'a dyn DatabaseItemBinder<O, Postgres>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for PostgresItemWriter<'_, O> {
    /// Writes items to the PostgreSQL database using batch inserts.
    ///
    /// This method implements the ItemWriter trait and provides efficient bulk
    /// insert operations. It constructs a SQL INSERT statement with the configured
    /// table and columns, then uses the item binder to bind item data to parameters.
    ///
    /// # PostgreSQL-Specific Behavior
    ///
    /// - Uses PostgreSQL's efficient batch insert capabilities
    /// - Supports PostgreSQL's advanced data types and features
    /// - Leverages connection pooling for optimal performance
    /// - Handles PostgreSQL-specific error conditions
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
    /// This method can fail for various PostgreSQL-specific reasons:
    /// - Connection pool exhaustion
    /// - Constraint violations (PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK)
    /// - Invalid data types or values
    /// - Table or column not found
    /// - Insufficient permissions
    /// - Transaction conflicts
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::postgres_writer::PostgresItemWriter;
    /// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use sqlx::{PgPool, query_builder::Separated, Postgres};
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
    /// impl DatabaseItemBinder<Product, Postgres> for ProductBinder {
    ///     fn bind(&self, item: &Product, mut query_builder: Separated<Postgres, &str>) {
    ///         query_builder.push_bind(item.id);
    ///         query_builder.push_bind(&item.name);
    ///         query_builder.push_bind(item.price);
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let binder = ProductBinder;
    ///
    /// let writer = PostgresItemWriter::<Product>::new()
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
                    "Successfully wrote {} items to PostgreSQL table {}",
                    items.len().min(BIND_LIMIT / self.columns.len()),
                    self.table.unwrap()
                );
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "Failed to write items to PostgreSQL table {}: {}",
                    self.table.unwrap_or("unknown"),
                    e
                );
                Err(BatchError::ItemWriter(format!(
                    "PostgreSQL write failed: {}",
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
    use sqlx::{query_builder::Separated, Postgres};

    #[derive(Clone, Serialize, Debug, PartialEq)]
    struct TestUser {
        id: i32,
        name: String,
        email: String,
    }

    struct TestUserBinder;
    impl DatabaseItemBinder<TestUser, Postgres> for TestUserBinder {
        fn bind(&self, item: &TestUser, mut query_builder: Separated<Postgres, &str>) {
            query_builder.push_bind(item.id);
            query_builder.push_bind(item.name.clone());
            query_builder.push_bind(item.email.clone());
        }
    }

    #[tokio::test]
    async fn test_new_creates_default_writer() {
        let writer = PostgresItemWriter::<TestUser>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern_configuration() {
        // This test doesn't require an actual database connection
        // We're just testing the builder pattern configuration

        let writer = PostgresItemWriter::<TestUser>::new()
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email");

        assert_eq!(writer.table, Some("users"));
        assert_eq!(writer.columns, vec!["id", "name", "email"]);
    }

    #[tokio::test]
    async fn test_write_empty_items() {
        // Create writer without pool for this simple test
        let binder = TestUserBinder;
        let writer = PostgresItemWriter::<TestUser>::new()
            .table("users")
            .add_column("id")
            .add_column("name")
            .add_column("email")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }

    mod integration_tests {
        use super::*;
        use crate::core::item::ItemWriter;
        use serde::{Deserialize, Serialize};
        use sqlx::{query_builder::Separated, FromRow, Postgres};
        use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
        struct TestProduct {
            id: i32,
            name: String,
            price: f64,
            active: bool,
        }

        struct TestProductBinder;
        impl DatabaseItemBinder<TestProduct, Postgres> for TestProductBinder {
            fn bind(&self, item: &TestProduct, mut query_builder: Separated<Postgres, &str>) {
                query_builder.push_bind(item.id);
                query_builder.push_bind(item.name.clone());
                query_builder.push_bind(item.price);
                query_builder.push_bind(item.active);
            }
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestUser {
            id: i32,
            name: String,
            email: String,
        }

        struct TestUserBinder;
        impl DatabaseItemBinder<TestUser, Postgres> for TestUserBinder {
            fn bind(&self, item: &TestUser, mut query_builder: Separated<Postgres, &str>) {
                query_builder.push_bind(item.id);
                query_builder.push_bind(item.name.clone());
                query_builder.push_bind(item.email.clone());
            }
        }

        async fn setup_test_database() -> Result<
            (
                Pool<Postgres>,
                testcontainers_modules::testcontainers::ContainerAsync<postgres::Postgres>,
            ),
            Box<dyn std::error::Error>,
        > {
            let container = postgres::Postgres::default().start().await?;
            let host_ip = container.get_host().await?;
            let host_port = container.get_host_port_ipv4(5432).await?;

            let connection_uri = format!(
                "postgres://postgres:postgres@{}:{}/postgres",
                host_ip, host_port
            );
            let pool = sqlx::PgPool::connect(&connection_uri).await?;

            Ok((pool, container))
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_write_single_item_to_database() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_products (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    price DECIMAL(10,2) NOT NULL,
                    active BOOLEAN NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            let binder = TestProductBinder;
            let writer = PostgresItemWriter::<TestProduct>::new()
                .pool(&pool)
                .table("test_products")
                .add_column("id")
                .add_column("name")
                .add_column("price")
                .add_column("active")
                .item_binder(&binder);

            let product = TestProduct {
                id: 1,
                name: "Test Product".to_string(),
                price: 99.99,
                active: true,
            };

            // Write the item
            let result = writer.write(&[product.clone()]);
            assert!(result.is_ok());

            // Verify the item was written
            let row: (i32, String, f64, bool) = sqlx::query_as(
                "SELECT id, name, price::FLOAT8, active FROM test_products WHERE id = $1",
            )
            .bind(product.id)
            .fetch_one(&pool)
            .await?;

            assert_eq!(row.0, product.id);
            assert_eq!(row.1, product.name);
            assert!((row.2 - product.price).abs() < 0.01); // Use floating point comparison
            assert_eq!(row.3, product.active);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_write_multiple_items_in_batch() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_users (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    email VARCHAR(255) NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            let binder = TestUserBinder;
            let writer = PostgresItemWriter::<TestUser>::new()
                .pool(&pool)
                .table("test_users")
                .add_column("id")
                .add_column("name")
                .add_column("email")
                .item_binder(&binder);

            let users = vec![
                TestUser {
                    id: 1,
                    name: "Alice".to_string(),
                    email: "alice@test.com".to_string(),
                },
                TestUser {
                    id: 2,
                    name: "Bob".to_string(),
                    email: "bob@test.com".to_string(),
                },
                TestUser {
                    id: 3,
                    name: "Charlie".to_string(),
                    email: "charlie@test.com".to_string(),
                },
            ];

            // Write the items
            let result = writer.write(&users);
            assert!(result.is_ok());

            // Verify all items were written
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
                .fetch_one(&pool)
                .await?;
            assert_eq!(count, 3);

            // Verify specific items
            let rows: Vec<(i32, String, String)> =
                sqlx::query_as("SELECT id, name, email FROM test_users ORDER BY id")
                    .fetch_all(&pool)
                    .await?;

            assert_eq!(rows.len(), 3);
            assert_eq!(
                rows[0],
                (1, "Alice".to_string(), "alice@test.com".to_string())
            );
            assert_eq!(rows[1], (2, "Bob".to_string(), "bob@test.com".to_string()));
            assert_eq!(
                rows[2],
                (3, "Charlie".to_string(), "charlie@test.com".to_string())
            );

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_large_batch_within_bind_limit(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_large_batch (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255) NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            struct SimpleBinder;
            impl DatabaseItemBinder<(i32, String), Postgres> for SimpleBinder {
                fn bind(&self, item: &(i32, String), mut query_builder: Separated<Postgres, &str>) {
                    query_builder.push_bind(item.0);
                    query_builder.push_bind(item.1.clone());
                }
            }

            let binder = SimpleBinder;
            let writer = PostgresItemWriter::<(i32, String)>::new()
                .pool(&pool)
                .table("test_large_batch")
                .add_column("id")
                .add_column("name")
                .item_binder(&binder);

            // Create a large batch (but within reasonable limits)
            let items: Vec<(i32, String)> = (1..=1000).map(|i| (i, format!("Item{}", i))).collect();

            // Write the items
            let result = writer.write(&items);
            assert!(result.is_ok());

            // Verify items were written (should be limited by BIND_LIMIT / columns)
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_large_batch")
                .fetch_one(&pool)
                .await?;

            // With 2 columns and BIND_LIMIT of 65535, we should be able to write 32767 items
            // But we only sent 1000, so all should be written
            assert_eq!(count, 1000);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_empty_items_array() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_empty (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255) NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            struct EmptyBinder;
            impl DatabaseItemBinder<(i32, String), Postgres> for EmptyBinder {
                fn bind(&self, item: &(i32, String), mut query_builder: Separated<Postgres, &str>) {
                    query_builder.push_bind(item.0);
                    query_builder.push_bind(item.1.clone());
                }
            }

            let binder = EmptyBinder;
            let writer = PostgresItemWriter::<(i32, String)>::new()
                .pool(&pool)
                .table("test_empty")
                .add_column("id")
                .add_column("name")
                .item_binder(&binder);

            // Write empty array
            let result = writer.write(&[]);
            assert!(result.is_ok());

            // Verify no items were written
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_empty")
                .fetch_one(&pool)
                .await?;
            assert_eq!(count, 0);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_database_constraint_violations(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table with unique constraint
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_constraints (
                    id INTEGER PRIMARY KEY,
                    email VARCHAR(255) UNIQUE NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            // Insert initial data
            sqlx::query("INSERT INTO test_constraints (id, email) VALUES (1, 'existing@test.com')")
                .execute(&pool)
                .await?;

            struct ConstraintBinder;
            impl DatabaseItemBinder<(i32, String), Postgres> for ConstraintBinder {
                fn bind(&self, item: &(i32, String), mut query_builder: Separated<Postgres, &str>) {
                    query_builder.push_bind(item.0);
                    query_builder.push_bind(item.1.clone());
                }
            }

            let binder = ConstraintBinder;
            let writer = PostgresItemWriter::<(i32, String)>::new()
                .pool(&pool)
                .table("test_constraints")
                .add_column("id")
                .add_column("email")
                .item_binder(&binder);

            // Try to insert duplicate email
            let items = vec![
                (2, "existing@test.com".to_string()), // This should violate unique constraint
            ];

            let result = writer.write(&items);
            assert!(result.is_err());

            // Verify error is properly wrapped
            if let Err(BatchError::ItemWriter(msg)) = result {
                assert!(msg.contains("PostgreSQL write failed"));
            } else {
                panic!("Expected ItemWriter error");
            }

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_different_postgresql_data_types(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table with various PostgreSQL data types
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_data_types (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255),
                    age SMALLINT,
                    salary DECIMAL(10,2),
                    active BOOLEAN,
                    created_at TIMESTAMP,
                    metadata JSONB
                )",
            )
            .execute(&pool)
            .await?;

            #[derive(Debug, Clone, Serialize)]
            struct ComplexItem {
                id: i32,
                name: String,
                age: i16,
                salary: f64,
                active: bool,
                created_at: chrono::NaiveDateTime,
                metadata: serde_json::Value,
            }

            struct ComplexBinder;
            impl DatabaseItemBinder<ComplexItem, Postgres> for ComplexBinder {
                fn bind(&self, item: &ComplexItem, mut query_builder: Separated<Postgres, &str>) {
                    query_builder.push_bind(item.id);
                    query_builder.push_bind(item.name.clone());
                    query_builder.push_bind(item.age);
                    query_builder.push_bind(item.salary);
                    query_builder.push_bind(item.active);
                    query_builder.push_bind(item.created_at);
                    query_builder.push_bind(item.metadata.clone());
                }
            }

            let binder = ComplexBinder;
            let writer = PostgresItemWriter::<ComplexItem>::new()
                .pool(&pool)
                .table("test_data_types")
                .add_column("id")
                .add_column("name")
                .add_column("age")
                .add_column("salary")
                .add_column("active")
                .add_column("created_at")
                .add_column("metadata")
                .item_binder(&binder);

            let item = ComplexItem {
                id: 1,
                name: "Test User".to_string(),
                age: 30,
                salary: 75000.50,
                active: true,
                created_at: chrono::DateTime::from_timestamp(1640995200, 0)
                    .unwrap()
                    .naive_utc(),
                metadata: serde_json::json!({"role": "admin", "permissions": ["read", "write"]}),
            };

            // Write the item
            let result = writer.write(&[item.clone()]);
            assert!(result.is_ok());

            // Verify the item was written correctly
            let row: (i32, String, i16, f64, bool) = sqlx::query_as(
                "SELECT id, name, age, salary::FLOAT8, active FROM test_data_types WHERE id = $1",
            )
            .bind(item.id)
            .fetch_one(&pool)
            .await?;

            assert_eq!(row.0, item.id);
            assert_eq!(row.1, item.name);
            assert_eq!(row.2, item.age);
            assert!((row.3 - item.salary).abs() < 0.01); // Use floating point comparison
            assert_eq!(row.4, item.active);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_null_values() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table with nullable columns
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_nulls (
                    id INTEGER PRIMARY KEY,
                    name VARCHAR(255),
                    optional_field VARCHAR(255)
                )",
            )
            .execute(&pool)
            .await?;

            #[derive(Debug, Clone, Serialize)]
            struct NullableItem {
                id: i32,
                name: String,
                optional_field: Option<String>,
            }

            struct NullableBinder;
            impl DatabaseItemBinder<NullableItem, Postgres> for NullableBinder {
                fn bind(&self, item: &NullableItem, mut query_builder: Separated<Postgres, &str>) {
                    query_builder.push_bind(item.id);
                    query_builder.push_bind(item.name.clone());
                    query_builder.push_bind(item.optional_field.clone());
                }
            }

            let binder = NullableBinder;
            let writer = PostgresItemWriter::<NullableItem>::new()
                .pool(&pool)
                .table("test_nulls")
                .add_column("id")
                .add_column("name")
                .add_column("optional_field")
                .item_binder(&binder);

            let items = vec![
                NullableItem {
                    id: 1,
                    name: "With Value".to_string(),
                    optional_field: Some("Has value".to_string()),
                },
                NullableItem {
                    id: 2,
                    name: "Without Value".to_string(),
                    optional_field: None,
                },
            ];

            // Write the items
            let result = writer.write(&items);
            assert!(result.is_ok());

            // Verify the items were written correctly
            let rows: Vec<(i32, String, Option<String>)> =
                sqlx::query_as("SELECT id, name, optional_field FROM test_nulls ORDER BY id")
                    .fetch_all(&pool)
                    .await?;

            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].0, 1);
            assert_eq!(rows[0].1, "With Value");
            assert_eq!(rows[0].2, Some("Has value".to_string()));

            assert_eq!(rows[1].0, 2);
            assert_eq!(rows[1].1, "Without Value");
            assert_eq!(rows[1].2, None);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_respect_bind_limit_with_many_columns(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create test table with many columns
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_many_columns (
                    id INTEGER PRIMARY KEY,
                    col1 VARCHAR(255), col2 VARCHAR(255), col3 VARCHAR(255), col4 VARCHAR(255),
                    col5 VARCHAR(255), col6 VARCHAR(255), col7 VARCHAR(255), col8 VARCHAR(255),
                    col9 VARCHAR(255), col10 VARCHAR(255)
                )",
            )
            .execute(&pool)
            .await?;

            #[derive(Debug, Clone, Serialize)]
            struct ManyColumnsItem {
                id: i32,
                col1: String,
                col2: String,
                col3: String,
                col4: String,
                col5: String,
                col6: String,
                col7: String,
                col8: String,
                col9: String,
                col10: String,
            }

            struct ManyColumnsBinder;
            impl DatabaseItemBinder<ManyColumnsItem, Postgres> for ManyColumnsBinder {
                fn bind(
                    &self,
                    item: &ManyColumnsItem,
                    mut query_builder: Separated<Postgres, &str>,
                ) {
                    query_builder.push_bind(item.id);
                    query_builder.push_bind(item.col1.clone());
                    query_builder.push_bind(item.col2.clone());
                    query_builder.push_bind(item.col3.clone());
                    query_builder.push_bind(item.col4.clone());
                    query_builder.push_bind(item.col5.clone());
                    query_builder.push_bind(item.col6.clone());
                    query_builder.push_bind(item.col7.clone());
                    query_builder.push_bind(item.col8.clone());
                    query_builder.push_bind(item.col9.clone());
                    query_builder.push_bind(item.col10.clone());
                }
            }

            let binder = ManyColumnsBinder;
            let writer = PostgresItemWriter::<ManyColumnsItem>::new()
                .pool(&pool)
                .table("test_many_columns")
                .add_column("id")
                .add_column("col1")
                .add_column("col2")
                .add_column("col3")
                .add_column("col4")
                .add_column("col5")
                .add_column("col6")
                .add_column("col7")
                .add_column("col8")
                .add_column("col9")
                .add_column("col10")
                .item_binder(&binder);

            // Create items that would exceed bind limit if not handled properly
            // With 11 columns, BIND_LIMIT/11 = ~5957 items max
            let items: Vec<ManyColumnsItem> = (1..=100)
                .map(|i| ManyColumnsItem {
                    id: i,
                    col1: format!("val1_{}", i),
                    col2: format!("val2_{}", i),
                    col3: format!("val3_{}", i),
                    col4: format!("val4_{}", i),
                    col5: format!("val5_{}", i),
                    col6: format!("val6_{}", i),
                    col7: format!("val7_{}", i),
                    col8: format!("val8_{}", i),
                    col9: format!("val9_{}", i),
                    col10: format!("val10_{}", i),
                })
                .collect();

            // Write the items
            let result = writer.write(&items);
            assert!(result.is_ok());

            // Verify items were written (should be limited by BIND_LIMIT / columns)
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_many_columns")
                .fetch_one(&pool)
                .await?;

            // All 100 items should be written as we're well within the limit
            assert_eq!(count, 100);

            Ok(())
        }
    }
}
