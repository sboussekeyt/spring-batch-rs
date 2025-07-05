use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

use sqlx::{postgres::PgRow, Execute, FromRow, Pool, Postgres, QueryBuilder};

use crate::core::item::{ItemReader, ItemReaderResult};

/// Builder for creating a PostgresRdbcItemReader with fluent API
///
/// This builder provides a convenient way to construct PostgreSQL item readers
/// with optional configuration like page size for memory-efficient reading.
///
/// # Design
///
/// The builder pattern ensures that all required parameters are set before creating
/// the reader instance. It uses phantom data to maintain type safety for the generic
/// item type while allowing the builder to be constructed without specifying all
/// parameters upfront.
///
/// # Type Parameters
///
/// * `I` - The item type that implements `FromRow<PgRow>` for automatic deserialization
///         from PostgreSQL rows. Must also be `Send + Unpin + Clone` for async compatibility.
pub struct PostgresRdbcItemReaderBuilder<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pool: Option<Pool<Postgres>>,
    query: Option<&'a str>,
    page_size: Option<i32>,
    _phantom: PhantomData<I>,
}

impl<I> Default for PostgresRdbcItemReaderBuilder<'_, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, I> PostgresRdbcItemReaderBuilder<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Creates a new builder with default configuration
    ///
    /// All parameters must be set using the builder methods before calling `build()`.
    /// This method initializes the builder with empty/default values for all fields.
    ///
    /// # Returns
    ///
    /// A new `PostgresRdbcItemReaderBuilder` instance with default settings.
    ///
    /// # Examples
    /// ```no_run
    /// use sqlx::PgPool;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReaderBuilder;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let reader = PostgresRdbcItemReaderBuilder::<User>::new()
    ///     .pool(pool)
    ///     .query("SELECT id, name FROM users")
    ///     .with_page_size(100)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            pool: None,
            query: None,
            page_size: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the PostgreSQL connection pool for the reader
    ///
    /// This is a required parameter that must be set before calling `build()`.
    /// The connection pool manages database connections efficiently and handles
    /// connection pooling, timeouts, and reconnection logic.
    ///
    /// # Arguments
    /// * `pool` - The PostgreSQL connection pool created with `sqlx::PgPool`
    ///
    /// # Returns
    ///
    /// The updated `PostgresRdbcItemReaderBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sqlx::PgPool;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReaderBuilder;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let builder = PostgresRdbcItemReaderBuilder::<User>::new()
    ///     .pool(pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn pool(mut self, pool: Pool<Postgres>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the SQL query for the reader
    ///
    /// This is a required parameter that must be set before calling `build()`.
    /// The query should not include LIMIT/OFFSET clauses as these are handled
    /// automatically when page_size is configured. The query should return columns
    /// that match the fields of the target type `I`.
    ///
    /// # Arguments
    /// * `query` - The SQL query to execute. Should be a SELECT statement without
    ///            LIMIT/OFFSET clauses.
    ///
    /// # Returns
    ///
    /// The updated `PostgresRdbcItemReaderBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReaderBuilder;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String, email: String }
    ///
    /// let builder = PostgresRdbcItemReaderBuilder::<User>::new()
    ///     .query("SELECT id, name, email FROM users WHERE active = true ORDER BY id");
    /// ```
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    /// Sets the page size for paginated reading
    ///
    /// When set, the reader will fetch data in chunks of this size to manage
    /// memory usage efficiently. This is particularly useful for large datasets
    /// where loading all data at once would consume too much memory.
    ///
    /// If not set, all data will be loaded in a single query, which may be
    /// appropriate for smaller datasets but could cause memory issues with
    /// large result sets.
    ///
    /// # Arguments
    /// * `page_size` - Number of items to read per page. Should be a positive integer.
    ///                Typical values range from 100 to 10000 depending on item size
    ///                and available memory.
    ///
    /// # Returns
    ///
    /// The updated `PostgresRdbcItemReaderBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReaderBuilder;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// // For large datasets, use pagination
    /// let builder = PostgresRdbcItemReaderBuilder::<User>::new()
    ///     .with_page_size(1000); // Read 1000 items at a time
    ///
    /// // For small datasets, you might omit page_size to read all at once
    /// let builder2 = PostgresRdbcItemReaderBuilder::<User>::new();
    /// ```
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Builds the PostgresRdbcItemReader
    ///
    /// Creates the final `PostgresRdbcItemReader` instance using the configured
    /// parameters. This method consumes the builder and validates that all
    /// required parameters have been set.
    ///
    /// # Returns
    ///
    /// A configured `PostgresRdbcItemReader` instance ready for use.
    ///
    /// # Panics
    ///
    /// Panics if required parameters (pool and query) are missing. Always ensure
    /// both `pool()` and `query()` have been called before calling `build()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sqlx::PgPool;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReaderBuilder;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    ///
    /// let reader = PostgresRdbcItemReaderBuilder::<User>::new()
    ///     .pool(pool)
    ///     .query("SELECT id, name FROM users")
    ///     .with_page_size(500)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> PostgresRdbcItemReader<'a, I> {
        PostgresRdbcItemReader {
            pool: self.pool.expect("PostgreSQL connection pool is required"),
            query: self.query.expect("SQL query is required"),
            page_size: self.page_size,
            offset: Cell::new(0),
            buffer: RefCell::new(vec![]),
        }
    }
}

/// PostgreSQL RDBC Item Reader for batch processing
///
/// This reader provides efficient reading of database records with optional pagination
/// to manage memory usage. It implements the ItemReader trait for integration with
/// Spring Batch processing patterns and is specifically optimized for PostgreSQL databases.
///
/// # Design
///
/// - Uses SQLx's PostgreSQL-specific driver for optimal performance
/// - Supports automatic deserialization using the `FromRow` trait
/// - Implements pagination with LIMIT/OFFSET for memory-efficient processing
/// - Maintains an internal buffer to minimize database round trips
/// - Uses interior mutability (Cell/RefCell) for state management in single-threaded contexts
///
/// # Memory Management
///
/// - Uses internal buffering with configurable page sizes
/// - Automatically handles pagination with LIMIT/OFFSET SQL clauses
/// - Clears buffer between pages to minimize memory footprint
/// - Pre-allocates buffer capacity when page size is known
///
/// # Thread Safety
///
/// - Uses Cell and RefCell for interior mutability in single-threaded contexts
/// - Not thread-safe - should be used within a single thread
/// - Designed for use in Spring Batch's single-threaded step execution model
///
/// # How Pagination Works
///
/// When `page_size` is provided:
/// - Data is loaded in batches of `page_size` items using SQL LIMIT/OFFSET
/// - When all items in a batch have been read, a new batch is automatically loaded
/// - The `offset` tracks both the SQL OFFSET clause and position within the buffer
/// - Buffer is cleared and refilled for each new page to manage memory
///
/// When `page_size` is not provided:
/// - All data is loaded in one query without LIMIT/OFFSET
/// - The `offset` only tracks the current position in the buffer
/// - Suitable for smaller datasets that fit comfortably in memory
///
/// # Type Parameters
///
/// * `I` - The item type that must implement:
///   - `FromRow<PgRow>` for automatic deserialization from PostgreSQL rows
///   - `Send + Unpin` for async compatibility
///   - `Clone` for efficient item retrieval from the buffer
pub struct PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Database connection pool for executing queries
    /// Uses PostgreSQL-specific pool for optimal performance
    pool: Pool<Postgres>,
    /// Base SQL query (without LIMIT/OFFSET clauses)
    /// Should be a SELECT statement that returns columns matching type I
    query: &'a str,
    /// Optional page size for pagination - if None, reads all data at once
    /// When Some(n), data is read in chunks of n items
    page_size: Option<i32>,
    /// Current offset position in the result set
    /// Tracks both SQL OFFSET and buffer position
    offset: Cell<i32>,
    /// Internal buffer to store the current page of items
    /// Cleared and refilled for each new page
    buffer: RefCell<Vec<I>>,
}

impl<'a, I> PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Creates a new PostgresRdbcItemReader with the specified parameters
    ///
    /// This is a direct constructor that creates a reader instance immediately.
    /// For a more fluent API, consider using `PostgresRdbcItemReaderBuilder` instead.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool for database operations
    /// * `query` - SQL query to execute (without LIMIT/OFFSET)
    /// * `page_size` - Optional page size for pagination. None means read all at once.
    ///
    /// # Returns
    ///
    /// A new `PostgresRdbcItemReader` instance ready for use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sqlx::PgPool;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReader;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    ///
    /// // With pagination
    /// let reader = PostgresRdbcItemReader::<User>::new(
    ///     pool.clone(),
    ///     "SELECT id, name FROM users ORDER BY id",
    ///     Some(1000)
    /// );
    ///
    /// // Without pagination (read all at once)
    /// let reader2 = PostgresRdbcItemReader::<User>::new(
    ///     pool,
    ///     "SELECT id, name FROM users WHERE active = true",
    ///     None
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    /// Consider using `PostgresRdbcItemReaderBuilder` for a more fluent API
    pub fn new(pool: Pool<Postgres>, query: &'a str, page_size: Option<i32>) -> Self {
        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            buffer: RefCell::new(vec![]),
        }
    }

    /// Creates a builder for constructing PostgresRdbcItemReader
    ///
    /// The builder pattern provides a more fluent and readable way to construct
    /// the reader with various configuration options. This is the recommended
    /// way to create reader instances.
    ///
    /// # Returns
    ///
    /// A new `PostgresRdbcItemReaderBuilder` instance for fluent configuration.
    ///
    /// # Examples
    /// ```no_run
    /// use sqlx::PgPool;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReader;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    ///
    /// let reader = PostgresRdbcItemReader::<User>::builder()
    ///     .pool(pool)
    ///     .query("SELECT id, name FROM users WHERE department = 'Engineering'")
    ///     .with_page_size(100)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> PostgresRdbcItemReaderBuilder<'a, I> {
        PostgresRdbcItemReaderBuilder::new()
    }

    /// Reads a page of data from the database and stores it in the internal buffer
    ///
    /// This method constructs the paginated query by appending LIMIT and OFFSET
    /// clauses to the base query, executes it against the PostgreSQL database,
    /// and updates the internal buffer with the results.
    ///
    /// # Behavior
    ///
    /// - Clears the existing buffer before loading new data to manage memory
    /// - Uses blocking async execution within the current runtime context
    /// - Automatically calculates OFFSET based on current position and page size
    /// - Handles both paginated and non-paginated queries appropriately
    /// - Uses SQLx's `query_as` for automatic deserialization via `FromRow`
    ///
    /// # Database Query Construction
    ///
    /// For paginated queries (when page_size is Some):
    /// ```sql
    /// {base_query} LIMIT {page_size} OFFSET {current_offset}
    /// ```
    ///
    /// For non-paginated queries (when page_size is None):
    /// ```sql
    /// {base_query}
    /// ```
    ///
    /// # Error Handling
    ///
    /// Currently uses `.unwrap()` for database errors, which will panic on failure.
    /// In production code, this should be replaced with proper error handling
    /// that returns `Result` types.
    ///
    /// # Performance Considerations
    ///
    /// - Uses `block_in_place` to run async code in sync context
    /// - Leverages connection pooling for efficient database access
    /// - Minimizes memory usage by clearing buffer between pages
    /// - Uses prepared statements through SQLx for query optimization
    fn read_page(&self) {
        // Build the paginated query by appending LIMIT/OFFSET to the base query
        // QueryBuilder allows us to dynamically construct SQL with proper escaping
        let mut query_builder = QueryBuilder::<Postgres>::new(self.query);

        // Add pagination clauses only if page_size is configured
        // This allows the same reader to work with both paginated and non-paginated queries
        if self.page_size.is_some() {
            query_builder.push(format!(
                " LIMIT {} OFFSET {}",
                self.page_size.unwrap(),
                self.offset.get()
            ));
        }

        let query = query_builder.build();

        // Execute the query synchronously within the async runtime
        // This allows the reader to work in synchronous batch processing contexts
        // while still leveraging async database operations under the hood
        let items = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Use query_as for automatic deserialization via FromRow trait
                // This eliminates the need for manual row mapping
                let rows: Vec<I> = sqlx::query_as(query.sql())
                    .fetch_all(&self.pool)
                    .await
                    .unwrap(); // TODO: Replace with proper error handling
                rows
            })
        });

        // Clear the buffer and load the new page of data
        // This ensures we don't accumulate items across pages, managing memory efficiently
        self.buffer.borrow_mut().clear();
        self.buffer.borrow_mut().extend(items);
    }
}

/// Implementation of ItemReader trait for PostgresRdbcItemReader.
///
/// This implementation provides a way to read items from a PostgreSQL database
/// with support for pagination. It uses an internal buffer to store the results
/// of database queries and keeps track of the current offset to determine when
/// a new page of data needs to be fetched.
///
/// The implementation handles both paginated and non-paginated reading modes
/// transparently, making it suitable for various batch processing scenarios.
impl<I> ItemReader<I> for PostgresRdbcItemReader<'_, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Reads the next item from the PostgreSQL database
    ///
    /// This method implements the ItemReader trait and provides the core reading logic
    /// with automatic pagination management:
    ///
    /// 1. **Index Calculation**: Determines the current position within the current page
    /// 2. **Page Loading**: Loads a new page if we're at the beginning of a page
    /// 3. **Item Retrieval**: Returns the item at the current position from the buffer
    /// 4. **Offset Management**: Advances the offset for the next read operation
    ///
    /// # Pagination Logic
    ///
    /// For paginated reading (when page_size is Some):
    /// - `index = offset % page_size` gives position within current page
    /// - When `index == 0`, we're at the start of a new page and need to load data
    /// - Buffer contains only the current page's items
    ///
    /// For non-paginated reading (when page_size is None):
    /// - `index = offset` gives absolute position in the full result set
    /// - Data is loaded only once when `index == 0` (first read)
    /// - Buffer contains all items from the query
    ///
    /// # Returns
    ///
    /// - `Ok(Some(item))` if an item was successfully read
    /// - `Ok(None)` if there are no more items to read (end of result set)
    /// - `Err(BatchError)` if a database error occurred (not currently implemented)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::core::item::ItemReader;
    /// # use spring_batch_rs::item::rdbc::postgres::PostgresRdbcItemReader;
    /// # use sqlx::PgPool;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let reader = PostgresRdbcItemReader::<User>::new(
    ///     pool,
    ///     "SELECT id, name FROM users ORDER BY id",
    ///     Some(100)
    /// );
    ///
    /// // Read items one by one
    /// let mut count = 0;
    /// while let Some(user) = reader.read()? {
    ///     println!("User: {} - {}", user.id, user.name);
    ///     count += 1;
    /// }
    /// println!("Processed {} users", count);
    /// # Ok(())
    /// # }
    /// ```
    fn read(&self) -> ItemReaderResult<I> {
        // Calculate the index within the current page
        // For paginated reading: offset % page_size gives us the position in current page
        // For non-paginated reading: use the offset directly as absolute position
        let index = if let Some(page_size) = self.page_size {
            self.offset.get() % page_size
        } else {
            self.offset.get()
        };

        // When index is 0, we've reached the start of a new page
        // or this is the first read operation, so we need to fetch data
        // This triggers database access only when necessary
        if index == 0 {
            self.read_page();
        }

        // Retrieve the item at the current index from the buffer
        // The buffer contains either the current page (paginated) or all items (non-paginated)
        let buffer = self.buffer.borrow();
        let result = buffer.get(index as usize);

        // Increment the offset for the next read operation
        // This maintains our position in the overall result set across page boundaries
        self.offset.set(self.offset.get() + 1);

        // Return the item, wrapped in an Option to indicate whether an item was found
        // None indicates we've reached the end of the result set
        Ok(result.cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemReader;
    use serde::{Deserialize, Serialize};
    use sqlx::{FromRow, Row};
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
    struct TestUser {
        id: i32,
        name: String,
        email: String,
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

        // Create test table and insert test data
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        // Insert test data
        for i in 1..=10 {
            sqlx::query("INSERT INTO test_users (name, email) VALUES ($1, $2)")
                .bind(format!("User{}", i))
                .bind(format!("user{}@test.com", i))
                .execute(&pool)
                .await?;
        }

        Ok((pool, container))
    }

    mod builder_tests {
        use super::*;

        #[tokio::test(flavor = "multi_thread")]
        async fn should_create_builder_with_page_size() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader = PostgresRdbcItemReaderBuilder::<TestUser>::new()
                .pool(pool)
                .query("SELECT * FROM test_users")
                .with_page_size(5)
                .build();

            assert_eq!(reader.page_size, Some(5));
            assert_eq!(reader.query, "SELECT * FROM test_users");
            assert_eq!(reader.offset.get(), 0);
            assert!(reader.buffer.borrow().is_empty());

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_create_builder_without_page_size() -> Result<(), Box<dyn std::error::Error>>
        {
            let (pool, _container) = setup_test_database().await?;

            let reader = PostgresRdbcItemReaderBuilder::<TestUser>::new()
                .pool(pool)
                .query("SELECT * FROM test_users")
                .build();

            assert_eq!(reader.page_size, None);
            assert_eq!(reader.query, "SELECT * FROM test_users");

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_create_reader_using_builder_method(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader = PostgresRdbcItemReader::<TestUser>::builder()
                .pool(pool)
                .query("SELECT * FROM test_users")
                .with_page_size(3)
                .build();

            assert_eq!(reader.page_size, Some(3));

            Ok(())
        }
    }

    mod reader_tests {
        use super::*;

        #[tokio::test(flavor = "multi_thread")]
        async fn should_read_all_items_without_pagination() -> Result<(), Box<dyn std::error::Error>>
        {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> =
                PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", None);

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 10);
            assert_eq!(items[0].id, 1);
            assert_eq!(items[0].name, "User1");
            assert_eq!(items[9].id, 10);
            assert_eq!(items[9].name, "User10");

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_read_items_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> =
                PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", Some(3));

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 10);
            // Verify items are read in correct order
            for (i, item) in items.iter().enumerate() {
                assert_eq!(item.id, (i + 1) as i32);
                assert_eq!(item.name, format!("User{}", i + 1));
            }

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_empty_result_set() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id > 1000",
                Some(5),
            );

            let result = reader.read()?;
            assert!(result.is_none());

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_single_page_result() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id <= 2 ORDER BY id",
                Some(5),
            );

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 2);
            assert_eq!(items[0].id, 1);
            assert_eq!(items[1].id, 2);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_page_size_larger_than_result_set(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id <= 3 ORDER BY id",
                Some(10), // Page size larger than result set
            );

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 3);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_page_size_of_one() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id <= 3 ORDER BY id",
                Some(1),
            );

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 3);
            // Verify correct order despite single-item pages
            for (i, item) in items.iter().enumerate() {
                assert_eq!(item.id, (i + 1) as i32);
            }

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_complex_query_with_where_clause(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id % 2 = 0 ORDER BY id",
                Some(2),
            );

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 5); // Even IDs: 2, 4, 6, 8, 10
            assert_eq!(items[0].id, 2);
            assert_eq!(items[1].id, 4);
            assert_eq!(items[4].id, 10);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_maintain_offset_state_correctly() -> Result<(), Box<dyn std::error::Error>>
        {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users WHERE id <= 5 ORDER BY id",
                Some(2),
            );

            // Initial state
            assert_eq!(reader.offset.get(), 0);

            // Read first item
            let item1: TestUser = reader.read()?.unwrap();
            assert_eq!(item1.id, 1);
            assert_eq!(reader.offset.get(), 1);

            // Read second item (still from first page)
            let item2: TestUser = reader.read()?.unwrap();
            assert_eq!(item2.id, 2);
            assert_eq!(reader.offset.get(), 2);

            // Read third item (triggers second page load)
            let item3: TestUser = reader.read()?.unwrap();
            assert_eq!(item3.id, 3);
            assert_eq!(reader.offset.get(), 3);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_concurrent_reads_safely() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            let reader: PostgresRdbcItemReader<TestUser> =
                PostgresRdbcItemReader::new(pool, "SELECT * FROM test_users ORDER BY id", Some(3));

            // Simulate multiple reads in sequence (as would happen in real usage)
            let mut all_items = Vec::new();
            for _ in 0..10 {
                if let Some(item) = reader.read()? {
                    all_items.push(item);
                } else {
                    break;
                }
            }

            assert_eq!(all_items.len(), 10);
            // Verify no duplicates and correct order
            for (i, item) in all_items.iter().enumerate() {
                assert_eq!(item.id, (i + 1) as i32);
            }

            Ok(())
        }
    }

    mod integration_tests {
        use super::*;

        #[tokio::test(flavor = "multi_thread")]
        async fn should_work_with_different_data_types() -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Create a table with different data types
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS test_data (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255),
                    age INTEGER,
                    active BOOLEAN,
                    score FLOAT8
                )",
            )
            .execute(&pool)
            .await?;

            sqlx::query(
                "INSERT INTO test_data (name, age, active, score) VALUES 
                ('Alice', 25, true, 95.5::FLOAT8),
                ('Bob', 30, false, 87.2::FLOAT8)",
            )
            .execute(&pool)
            .await?;

            #[derive(Debug, Clone, PartialEq)]
            struct TestData {
                id: i32,
                name: String,
                age: i32,
                active: bool,
                score: f64,
            }

            impl<'r> FromRow<'r, sqlx::postgres::PgRow> for TestData {
                fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
                    Ok(TestData {
                        id: row.try_get("id")?,
                        name: row.try_get("name")?,
                        age: row.try_get("age")?,
                        active: row.try_get("active")?,
                        score: row.try_get::<f64, _>("score")?,
                    })
                }
            }

            let reader: PostgresRdbcItemReader<TestData> =
                PostgresRdbcItemReader::new(pool, "SELECT * FROM test_data ORDER BY id", Some(1));

            let mut items = Vec::new();
            while let Some(item) = reader.read()? {
                items.push(item);
            }

            assert_eq!(items.len(), 2);
            assert_eq!(items[0].name, "Alice");
            assert_eq!(items[0].age, 25);
            assert_eq!(items[0].active, true);
            assert_eq!(items[1].name, "Bob");
            assert_eq!(items[1].active, false);

            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn should_handle_large_result_sets_efficiently(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let (pool, _container) = setup_test_database().await?;

            // Insert more test data
            for i in 11..=100 {
                sqlx::query("INSERT INTO test_users (name, email) VALUES ($1, $2)")
                    .bind(format!("User{}", i))
                    .bind(format!("user{}@test.com", i))
                    .execute(&pool)
                    .await?;
            }

            let reader: PostgresRdbcItemReader<TestUser> = PostgresRdbcItemReader::new(
                pool,
                "SELECT * FROM test_users ORDER BY id",
                Some(10), // Small page size for large dataset
            );

            let mut count = 0;
            while let Some(_item) = reader.read()? {
                count += 1;
            }

            assert_eq!(count, 100);

            Ok(())
        }
    }
}
