use std::cell::{Cell, RefCell};

use sqlx::{mysql::MySqlRow, Execute, FromRow, MySql, Pool, QueryBuilder};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::core::item::{ItemReader, ItemReaderResult};

/// MySQL RDBC Item Reader for batch processing
///
/// # Construction
///
/// This reader can only be created through `RdbcItemReaderBuilder`.
/// Direct construction is not available to ensure proper configuration.
pub struct MySqlRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<MySql>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
}

impl<'a, I> MySqlRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    /// Creates a new MySqlRdbcItemReader with the specified parameters
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemReaderBuilder` for creating reader instances.
    pub(crate) fn new(pool: Pool<MySql>, query: &'a str, page_size: Option<i32>) -> Self {
        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            buffer: RefCell::new(vec![]),
        }
    }

    /// Reads a page of data from the database and stores it in the internal buffer
    fn read_page(&self) {
        let mut query_builder = QueryBuilder::<MySql>::new(self.query);

        if let Some(page_size) = self.page_size {
            query_builder.push(format!(
                " LIMIT {} OFFSET {}",
                page_size,
                self.offset.get()
            ));
        }

        let query = query_builder.build();

        let items = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows: Vec<I> = sqlx::query_as(query.sql())
                    .fetch_all(&self.pool)
                    .await
                    .unwrap();
                rows
            })
        });

        self.buffer.borrow_mut().clear();
        self.buffer.borrow_mut().extend(items);
    }
}

impl<I> ItemReader<I> for MySqlRdbcItemReader<'_, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    /// Reads the next item from the MySQL database
    fn read(&self) -> ItemReaderResult<I> {
        let index = calculate_page_index(self.offset.get(), self.page_size);

        if should_load_page(index) {
            self.read_page();
        }

        let buffer = self.buffer.borrow();
        let result = buffer.get(index as usize);

        self.offset.set(self.offset.get() + 1);

        Ok(result.cloned())
    }
}
