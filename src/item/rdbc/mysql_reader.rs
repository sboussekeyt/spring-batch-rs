use std::cell::{Cell, RefCell};

use sqlx::{FromRow, MySql, Pool, QueryBuilder, mysql::MySqlRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

/// MySQL RDBC Item Reader for batch processing.
///
/// Supports LIMIT/OFFSET pagination (default) and keyset pagination
/// (enabled via [`RdbcItemReaderBuilder::with_keyset`](crate::item::rdbc::RdbcItemReaderBuilder::with_keyset)).
///
/// # Construction
///
/// Prefer [`RdbcItemReaderBuilder`](crate::item::rdbc::RdbcItemReaderBuilder) for ergonomic construction.
pub struct MySqlRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<MySql>,
    pub(crate) query: String,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
    pub(crate) keyset_column: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(crate) keyset_key: Option<Box<dyn Fn(&I) -> String>>,
    pub(crate) last_cursor: RefCell<Option<String>>,
}

impl<I> MySqlRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    /// Creates a new `MySqlRdbcItemReader` with the specified parameters.
    ///
    /// Prefer [`RdbcItemReaderBuilder`](crate::item::rdbc::RdbcItemReaderBuilder) for a more
    /// ergonomic construction API.
    #[allow(clippy::type_complexity)]
    pub fn new(
        pool: Pool<MySql>,
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
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the query fails.
    fn read_page(&self) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::<MySql>::new(&self.query);

        if let Some(page_size) = self.page_size {
            if let Some(ref col) = self.keyset_column {
                let last = self.last_cursor.borrow();
                if let Some(ref cursor_val) = *last {
                    let escaped = cursor_val.replace('\'', "''");
                    query_builder.push(format!(" WHERE {} > '{}'", col, escaped));
                }
                drop(last);
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

impl<I> ItemReader<I> for MySqlRdbcItemReader<I>
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::MySqlPool;

    #[derive(Clone)]
    struct Dummy;

    impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for Dummy {
        fn from_row(_row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
            Ok(Dummy)
        }
    }

    fn reader_with_keyset(keyset: bool) -> MySqlRdbcItemReader<Dummy> {
        let pool = MySqlPool::connect_lazy("mysql://root:root@localhost/test")
            .expect("lazy pool creation should not fail");
        let (col, key): (Option<String>, Option<Box<dyn Fn(&Dummy) -> String>>) = if keyset {
            (
                Some("id".to_string()),
                Some(Box::new(|_: &Dummy| "0".to_string())),
            )
        } else {
            (None, None)
        };
        MySqlRdbcItemReader::new(pool, "SELECT 1".to_string(), Some(10), col, key)
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
