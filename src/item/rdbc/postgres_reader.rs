use std::cell::{Cell, RefCell};

use sqlx::{Execute, FromRow, Pool, Postgres, QueryBuilder, postgres::PgRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

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
///
/// # Construction
///
/// This reader can only be created through `RdbcItemReaderBuilder`.
/// Direct construction is not available to ensure proper configuration.
pub struct PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Database connection pool for executing queries
    /// Uses PostgreSQL-specific pool for optimal performance
    pub(crate) pool: Pool<Postgres>,
    /// Base SQL query (without LIMIT/OFFSET clauses)
    /// Should be a SELECT statement that returns columns matching type I
    pub(crate) query: &'a str,
    /// Optional page size for pagination - if None, reads all data at once
    /// When Some(n), data is read in chunks of n items
    pub(crate) page_size: Option<i32>,
    /// Current offset position in the result set
    /// Tracks both SQL OFFSET and buffer position
    pub(crate) offset: Cell<i32>,
    /// Internal buffer to store the current page of items
    /// Cleared and refilled for each new page
    pub(crate) buffer: RefCell<Vec<I>>,
}

impl<'a, I> PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Creates a new PostgresRdbcItemReader with the specified parameters
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemReaderBuilder` for creating reader instances.
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
    pub fn new(pool: Pool<Postgres>, query: &'a str, page_size: Option<i32>) -> Self {
        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            buffer: RefCell::new(vec![]),
        }
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
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the database query fails (e.g., connection
    /// error, SQL syntax error, or deserialization failure).
    ///
    /// # Performance Considerations
    ///
    /// - Uses `block_in_place` to run async code in sync context
    /// - Leverages connection pooling for efficient database access
    /// - Minimizes memory usage by clearing buffer between pages
    /// - Uses prepared statements through SQLx for query optimization
    fn read_page(&self) -> Result<(), BatchError> {
        // Build the paginated query by appending LIMIT/OFFSET to the base query
        // QueryBuilder allows us to dynamically construct SQL with proper escaping
        let mut query_builder = QueryBuilder::<Postgres>::new(self.query);

        // Add pagination clauses only if page_size is configured
        // This allows the same reader to work with both paginated and non-paginated queries
        if let Some(page_size) = self.page_size {
            query_builder.push(format!(" LIMIT {} OFFSET {}", page_size, self.offset.get()));
        }

        let query = query_builder.build();

        // Execute the query synchronously within the async runtime
        // This allows the reader to work in synchronous batch processing contexts
        // while still leveraging async database operations under the hood
        let items = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Use query_as for automatic deserialization via FromRow trait
                // This eliminates the need for manual row mapping
                sqlx::query_as::<_, I>(query.sql())
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| BatchError::ItemReader(e.to_string()))
            })
        })?;

        // Clear the buffer and load the new page of data
        // This ensures we don't accumulate items across pages, managing memory efficiently
        self.buffer.borrow_mut().clear();
        self.buffer.borrow_mut().extend(items);
        Ok(())
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
    /// - `Err(BatchError::ItemReader)` if a database error occurred
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use spring_batch_rs::core::item::ItemReader;
    /// # use spring_batch_rs::item::rdbc::PostgresRdbcItemReader;
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
        let index = calculate_page_index(self.offset.get(), self.page_size);

        if should_load_page(index) {
            self.read_page()?;
        }

        let buffer = self.buffer.borrow();
        let result = buffer.get(index as usize);

        self.offset.set(self.offset.get() + 1);

        Ok(result.cloned())
    }
}
