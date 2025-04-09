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

impl<T: DeserializeOwned + Clone> ItemReader<T> for RdbcItemReader<'_, T> {
    /// Reads the next item from the reader.
    ///
    /// # Returns
    ///
    /// The next item, or `None` if there are no more items.
    fn read(&self) -> ItemReaderResult<T> {
        let index = if let Some(page_size) = self.page_size {
            self.offset.get() % page_size
        } else {
            self.offset.get()
        };

        if index == 0 {
            self.read_page();
        }

        let buffer = self.buffer.borrow();

        let result = buffer.get(index as usize);

        self.offset.set(self.offset.get() + 1);

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
