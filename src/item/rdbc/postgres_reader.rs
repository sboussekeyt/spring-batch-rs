use std::cell::{Cell, RefCell};

use sqlx::{Execute, FromRow, Pool, Postgres, QueryBuilder, postgres::PgRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

/// PostgreSQL RDBC Item Reader for batch processing.
///
/// Supports two pagination strategies:
///
/// - **LIMIT/OFFSET** (default): simple but degrades at large offsets — use for small datasets.
/// - **Keyset pagination** (recommended for large datasets): uses `WHERE col > :last ORDER BY col LIMIT n`,
///   O(log n) per page regardless of dataset size. Enable with [`RdbcItemReaderBuilder::with_keyset`].
///
/// # Type Parameters
///
/// * `I` - Must implement `FromRow<PgRow> + Send + Unpin + Clone`.
///
/// # Construction
///
/// Use [`RdbcItemReaderBuilder`] — direct construction is not available.
pub struct PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Postgres>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
    /// Column name used as the keyset cursor (e.g. `"id"`).
    pub(crate) keyset_column: Option<String>,
    /// Extracts the cursor value from an item for use in the next page's WHERE clause.
    #[allow(clippy::type_complexity)]
    pub(crate) keyset_key: Option<Box<dyn Fn(&I) -> String>>,
    /// Last cursor value seen; drives the WHERE clause on subsequent pages.
    pub(crate) last_cursor: RefCell<Option<String>>,
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
    #[allow(clippy::type_complexity)]
    pub fn new(
        pool: Pool<Postgres>,
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

    /// Fetches the next page from the database into the internal buffer.
    ///
    /// Uses keyset pagination when [`keyset_column`] is set, otherwise falls back
    /// to LIMIT/OFFSET.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the query fails.
    fn read_page(&self) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::<Postgres>::new(self.query);

        if let Some(page_size) = self.page_size {
            if let Some(ref col) = self.keyset_column {
                let last = self.last_cursor.borrow();
                if let Some(ref cursor_val) = *last {
                    // Escape single quotes to prevent SQL injection from cursor values.
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

/// Implementation of ItemReader trait for PostgresRdbcItemReader.
///
/// This implementation provides a way to read items from a PostgreSQL database
/// with support for pagination. It uses an internal buffer to store the results
/// of database queries and keeps track of the current offset to determine when
/// a new page of data needs to be fetched.
///
/// The implementation handles both paginated and non-paginated reading modes
/// transparently, making it suitable for various batch processing scenarios.
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[derive(Clone)]
    struct Dummy;

    impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Dummy {
        fn from_row(_row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
            Ok(Dummy)
        }
    }

    fn reader_with_keyset(keyset: bool) -> PostgresRdbcItemReader<'static, Dummy> {
        let pool = PgPool::connect_lazy("postgres://postgres:postgres@localhost/test")
            .expect("lazy pool creation should not fail");
        let (col, key): (Option<String>, Option<Box<dyn Fn(&Dummy) -> String>>) = if keyset {
            (
                Some("id".to_string()),
                Some(Box::new(|_: &Dummy| "0".to_string())),
            )
        } else {
            (None, None)
        };
        PostgresRdbcItemReader::new(pool, "SELECT 1", Some(10), col, key)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_initialize_without_keyset() {
        let reader = reader_with_keyset(false);
        assert!(reader.keyset_column.is_none(), "no keyset column expected");
        assert!(reader.keyset_key.is_none(), "no keyset key fn expected");
        assert!(
            reader.last_cursor.borrow().is_none(),
            "cursor must start as None"
        );
        assert_eq!(reader.offset.get(), 0, "initial offset should be 0");
        assert!(
            reader.buffer.borrow().is_empty(),
            "buffer should start empty"
        );
        assert_eq!(reader.page_size, Some(10));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_initialize_with_keyset_column_and_none_cursor() {
        let reader = reader_with_keyset(true);
        assert_eq!(
            reader.keyset_column.as_deref(),
            Some("id"),
            "keyset column should be stored"
        );
        assert!(
            reader.keyset_key.is_some(),
            "keyset key fn should be stored"
        );
        assert!(
            reader.last_cursor.borrow().is_none(),
            "cursor must start as None before first read"
        );
    }
}

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

        let result = self.buffer.borrow().get(index as usize).cloned();

        if let (Some(item), Some(key_fn)) = (&result, &self.keyset_key) {
            *self.last_cursor.borrow_mut() = Some(key_fn(item));
        }

        self.offset.set(self.offset.get() + 1);

        Ok(result)
    }
}
