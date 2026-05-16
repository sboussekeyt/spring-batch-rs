use std::cell::{Cell, RefCell};

use sqlx::{FromRow, Pool, Postgres, QueryBuilder, postgres::PgRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

/// PostgreSQL RDBC Item Reader for batch processing.
///
/// Supports two pagination strategies:
///
/// - **LIMIT/OFFSET** (default): simple but degrades at large offsets — use for small datasets.
/// - **Keyset pagination** (recommended for large datasets): uses `WHERE col > :last ORDER BY col LIMIT n`,
///   O(log n) per page regardless of dataset size. Enable with
///   [`RdbcItemReaderBuilder::with_keyset`](crate::item::rdbc::RdbcItemReaderBuilder::with_keyset).
///
/// # Type Parameters
///
/// * `I` - Must implement `FromRow<PgRow> + Send + Unpin + Clone`.
///
/// # Construction
///
/// Prefer [`RdbcItemReaderBuilder`](crate::item::rdbc::RdbcItemReaderBuilder) for ergonomic construction.
pub struct PostgresRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Postgres>,
    pub(crate) query: String,
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

impl<I> PostgresRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Creates a new `PostgresRdbcItemReader` with the specified parameters.
    ///
    /// Prefer [`RdbcItemReaderBuilder`](crate::item::rdbc::RdbcItemReaderBuilder) for a more
    /// ergonomic construction API.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool for database operations
    /// * `query` - SQL query to execute (without LIMIT/OFFSET)
    /// * `page_size` - Optional page size for pagination. None means read all at once.
    /// * `keyset_column` - Optional column name for keyset pagination.
    /// * `keyset_key` - Optional closure to extract the cursor value from an item.
    ///
    /// # Returns
    ///
    /// A new `PostgresRdbcItemReader` instance ready for use.
    #[allow(clippy::type_complexity)]
    pub fn new(
        pool: Pool<Postgres>,
        query: String,
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
    /// Uses keyset pagination when `keyset_column` is set, otherwise falls back
    /// to LIMIT/OFFSET.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the query fails.
    fn read_page(&self) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::<Postgres>::new(&self.query);

        if let Some(page_size) = self.page_size {
            if let Some(ref col) = self.keyset_column {
                {
                    let last = self.last_cursor.borrow();
                    if let Some(ref cursor_val) = *last {
                        query_builder.push(format!(" WHERE {} > ", col));
                        query_builder.push_bind(cursor_val.clone());
                    }
                }
                query_builder.push(format!(" ORDER BY {} LIMIT {}", col, page_size));
            } else {
                query_builder.push(format!(" LIMIT {} OFFSET {}", page_size, self.offset.get()));
            }
        }

        let query = query_builder.build_query_as::<I>();
        let items = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                query
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| BatchError::ItemReader(e.to_string()))
            })
        })?;

        *self.buffer.borrow_mut() = items;
        Ok(())
    }
}

impl<I> ItemReader<I> for PostgresRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Reads the next item from the PostgreSQL database.
    ///
    /// Manages automatic pagination: loads a new page when the buffer is exhausted,
    /// handles both LIMIT/OFFSET and keyset pagination transparently.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(item))` if an item was successfully read
    /// - `Ok(None)` if there are no more items to read (end of result set)
    /// - `Err(BatchError::ItemReader)` if a database error occurred
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::core::item::ItemReader;
    /// use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;
    /// use sqlx::PgPool;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct User { id: i32, name: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    /// let reader = RdbcItemReaderBuilder::<User>::new()
    ///     .postgres(pool)
    ///     .query("SELECT id, name FROM users ORDER BY id")
    ///     .with_page_size(100)
    ///     .build_postgres();
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

    fn reader_with_keyset(keyset: bool) -> PostgresRdbcItemReader<Dummy> {
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
        PostgresRdbcItemReader::new(pool, "SELECT 1".to_string(), Some(10), col, key)
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
