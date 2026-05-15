use std::cell::{Cell, RefCell};

use sqlx::{Execute, FromRow, MySql, Pool, QueryBuilder, mysql::MySqlRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

/// MySQL RDBC Item Reader for batch processing
///
/// # Construction
///
/// This reader can only be created through `RdbcItemReaderBuilder`.
/// Direct construction is not available to ensure proper configuration.
/// MySQL RDBC Item Reader for batch processing.
///
/// Supports LIMIT/OFFSET pagination (default) and keyset pagination
/// (enabled via [`RdbcItemReaderBuilder::with_keyset`]).
///
/// # Construction
///
/// Use [`RdbcItemReaderBuilder`] — direct construction is not available.
pub struct MySqlRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<MySql>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
    pub(crate) keyset_column: Option<String>,
    pub(crate) keyset_key: Option<Box<dyn Fn(&I) -> String>>,
    pub(crate) last_cursor: RefCell<Option<String>>,
}

impl<'a, I> MySqlRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    /// Creates a new MySqlRdbcItemReader with the specified parameters
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemReaderBuilder` for creating reader instances.
    pub(crate) fn new(
        pool: Pool<MySql>,
        query: &'a str,
        page_size: Option<i32>,
        keyset_column: Option<String>,
        keyset_key: Option<Box<dyn Fn(&I) -> String>>,
    ) -> Self {
        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            buffer: RefCell::new(vec![]),
            keyset_column,
            keyset_key,
            last_cursor: RefCell::new(None),
        }
    }

    /// Reads a page of data from the database and stores it in the internal buffer.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the database query fails.
    /// Fetches the next page from the database into the internal buffer.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the query fails.
    fn read_page(&self) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::<MySql>::new(self.query);

        if let Some(page_size) = self.page_size {
            if let Some(ref col) = self.keyset_column {
                let last = self.last_cursor.borrow();
                if let Some(ref cursor_val) = *last {
                    let escaped = cursor_val.replace('\'', "''");
                    query_builder.push(format!(" WHERE {} > '{}'", col, escaped));
                }
                query_builder.push(format!(" ORDER BY {} LIMIT {}", col, page_size));
            } else {
                query_builder.push(format!(" LIMIT {} OFFSET {}", page_size, self.offset.get()));
            }
        }

        let query = query_builder.build();

        let items = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, I>(query.sql())
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| BatchError::ItemReader(e.to_string()))
            })
        })?;

        self.buffer.borrow_mut().clear();
        self.buffer.borrow_mut().extend(items);
        Ok(())
    }
}

impl<I> ItemReader<I> for MySqlRdbcItemReader<'_, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    fn read(&self) -> ItemReaderResult<I> {
        let index = calculate_page_index(self.offset.get(), self.page_size);

        if should_load_page(index) {
            self.read_page()?;
        }

        let result = self.buffer.borrow().get(index as usize).cloned();

        if let (Some(item), Some(key_fn)) = (&result, &self.keyset_key) {
            *self.last_cursor.borrow_mut() = Some(key_fn(item));
        }

        self.offset.set(self.offset.get() + 1);

        Ok(result)
    }
}
