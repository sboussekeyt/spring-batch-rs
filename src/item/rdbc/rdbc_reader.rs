use std::cell::{Cell, RefCell};

use serde::de::DeserializeOwned;
use sqlx::{any::AnyRow, Any, Pool, QueryBuilder};

use crate::core::item::{ItemReader, ItemReaderResult};

/// Trait for mapping a database row to a specific type.
pub trait RdbcRowMapper<T> {
    /// Maps a database row to the specified type.
    fn map_row(&self, row: &AnyRow) -> T;
}

/// A reader for reading items from a relational database using SQLx.
///
/// This reader provides an implementation of the `ItemReader` trait for database operations.
/// It supports reading data from any SQL database supported by SQLx's `Any` database driver,
/// with optional pagination for efficient memory usage when dealing with large datasets.
///
/// # Design
///
/// - Uses a connection pool to efficiently manage database connections
/// - Supports optional pagination to avoid loading the entire result set into memory
/// - Maintains an internal buffer of items and only fetches new data when necessary
/// - Uses a row mapper to convert database rows into domain objects
/// - Tracks the current position using an offset counter
///
/// # How Pagination Works
///
/// When `page_size` is provided:
/// - Data is loaded in batches of `page_size` items
/// - When all items in a batch have been read, a new batch is loaded
/// - The `offset` is used to determine both the SQL OFFSET clause and the position within the buffer
///
/// When `page_size` is not provided:
/// - All data is loaded in one query
/// - The `offset` is only used to track the current position in the buffer
pub struct RdbcItemReader<'a, T> {
    pool: &'a Pool<Any>,
    query: &'a str,
    page_size: Option<i32>,
    offset: Cell<i32>,
    row_mapper: &'a dyn RdbcRowMapper<T>,
    buffer: RefCell<Vec<T>>,
}

impl<'a, T> RdbcItemReader<'a, T> {
    /// Creates a new `RdbcItemReader`.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool.
    /// * `query` - The SQL query to execute.
    /// * `page_size` - The number of items to read per page.
    /// * `row_mapper` - The row mapper for mapping database rows to items.
    ///
    /// # Returns
    ///
    /// A new `RdbcItemReader` instance.
    pub fn new(
        pool: &'a Pool<Any>,
        query: &'a str,
        page_size: Option<i32>,
        row_mapper: &'a dyn RdbcRowMapper<T>,
    ) -> Self {
        let buffer = if let Some(page_size) = page_size {
            let buffer_size = page_size.try_into().unwrap_or(1);
            Vec::with_capacity(buffer_size)
        } else {
            Vec::new()
        };

        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            row_mapper,
            buffer: RefCell::new(buffer),
        }
    }

    /// Reads a page of items from the database.
    ///
    /// This method builds a SQL query with pagination parameters (if page_size is set),
    /// executes it against the database, and fills the internal buffer with the results.
    /// It uses tokio's block_in_place to run the async database query in a blocking context.
    fn read_page(&self) {
        let mut query_builder = QueryBuilder::new(self.query);

        if self.page_size.is_some() {
            query_builder.push(format!(
                " LIMIT {} OFFSET {}",
                self.page_size.unwrap(),
                self.offset.get()
            ));
        }

        let query = query_builder.build();

        let rows = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { query.fetch_all(self.pool).await.unwrap() })
        });

        self.buffer.borrow_mut().clear();

        rows.iter().for_each(|x| {
            let item = self.row_mapper.map_row(x);
            self.buffer.borrow_mut().push(item);
        });
    }
}

/// Implementation of ItemReader trait for RdbcItemReader.
///
/// This implementation provides a way to read items from a relational database
/// with support for pagination. It uses an internal buffer to store the results
/// of database queries and keeps track of the current offset to determine when
/// a new page of data needs to be fetched.
impl<T: DeserializeOwned + Clone> ItemReader<T> for RdbcItemReader<'_, T> {
    /// Reads the next item from the reader.
    ///
    /// This method manages pagination internally:
    /// - When the current offset reaches a multiple of the page size, a new page is loaded
    /// - Items are read sequentially from the internal buffer
    /// - The offset is incremented after each read operation
    ///
    /// # Returns
    ///
    /// The next item, or `None` if there are no more items.
    fn read(&self) -> ItemReaderResult<T> {
        // Calculate the index within the current page
        // If page_size is set, we're using pagination and need to find position within current page
        // Otherwise, we're using the absolute offset
        let index = if let Some(page_size) = self.page_size {
            self.offset.get() % page_size
        } else {
            self.offset.get()
        };

        // When index is 0, we've reached the start of a new page
        // or this is the first read operation, so we need to fetch data
        if index == 0 {
            self.read_page();
        }

        // Retrieve the item at the current index from the buffer
        let buffer = self.buffer.borrow();
        let result = buffer.get(index as usize);

        // Increment the offset for the next read operation
        self.offset.set(self.offset.get() + 1);

        // Return the item, wrapped in an Option to indicate whether an item was found
        Ok(result.cloned())
    }
}

/// Builder for creating an `RdbcItemReader`.
#[derive(Default)]
pub struct RdbcItemReaderBuilder<'a, T> {
    pool: Option<&'a Pool<Any>>,
    query: Option<&'a str>,
    page_size: Option<i32>,
    row_mapper: Option<&'a dyn RdbcRowMapper<T>>,
}

impl<'a, T> RdbcItemReaderBuilder<'a, T> {
    /// Creates a new `RdbcItemReaderBuilder`.
    pub fn new() -> Self {
        Self {
            pool: None,
            query: None,
            page_size: None,
            row_mapper: None,
        }
    }

    /// Sets the page size for the reader.
    ///
    /// # Arguments
    ///
    /// * `page_size` - The number of items to read per page.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemReaderBuilder` instance.
    pub fn page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Sets the SQL query for the reader.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query to execute.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemReaderBuilder` instance.
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    /// Sets the database connection pool for the reader.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemReaderBuilder` instance.
    pub fn pool(mut self, pool: &'a Pool<Any>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the row mapper for the reader.
    ///
    /// # Arguments
    ///
    /// * `row_mapper` - The row mapper for mapping database rows to items.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemReaderBuilder` instance.
    pub fn row_mapper(mut self, row_mapper: &'a dyn RdbcRowMapper<T>) -> Self {
        self.row_mapper = Some(row_mapper);
        self
    }

    /// Builds the `RdbcItemReader` instance.
    ///
    /// # Returns
    ///
    /// The built `RdbcItemReader` instance.
    pub fn build(self) -> RdbcItemReader<'a, T> {
        RdbcItemReader::new(
            self.pool.unwrap(),
            self.query.unwrap(),
            self.page_size,
            self.row_mapper.unwrap(),
        )
    }
}
