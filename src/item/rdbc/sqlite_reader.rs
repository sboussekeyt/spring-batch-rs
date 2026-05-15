use std::cell::{Cell, RefCell};

use sqlx::{Execute, FromRow, Pool, QueryBuilder, Sqlite, sqlite::SqliteRow};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::BatchError;
use crate::core::item::{ItemReader, ItemReaderResult};

/// SQLite RDBC Item Reader for batch processing.
///
/// Supports LIMIT/OFFSET pagination (default) and keyset pagination
/// (enabled via [`RdbcItemReaderBuilder::with_keyset`]).
///
/// # Construction
///
/// Use [`RdbcItemReaderBuilder`] — direct construction is not available.
pub struct SqliteRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Sqlite>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
    pub(crate) keyset_column: Option<String>,
    pub(crate) keyset_key: Option<Box<dyn Fn(&I) -> String>>,
    pub(crate) last_cursor: RefCell<Option<String>>,
}

impl<'a, I> SqliteRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    /// Creates a new SqliteRdbcItemReader with the specified parameters
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemReaderBuilder` for creating reader instances.
    pub(crate) fn new(
        pool: Pool<Sqlite>,
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
    /// # Errors
    ///
    /// Returns [`BatchError::ItemReader`] if the query fails.
    fn read_page(&self) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::<Sqlite>::new(self.query);

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

impl<I> ItemReader<I> for SqliteRdbcItemReader<'_, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
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
    use crate::core::item::ItemReader;
    use sqlx::{FromRow, SqlitePool};

    #[derive(Clone, FromRow)]
    struct Row {
        id: i32,
        name: String,
    }

    async fn pool_with_rows(rows: &[(i32, &str)]) -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE items (id INTEGER, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        for (id, name) in rows {
            sqlx::query("INSERT INTO items (id, name) VALUES (?, ?)")
                .bind(id)
                .bind(name)
                .execute(&pool)
                .await
                .unwrap();
        }
        pool
    }

    fn make_reader(pool: SqlitePool, query: &str, page_size: Option<i32>) -> SqliteRdbcItemReader<'_, Row> {
        SqliteRdbcItemReader::<Row>::new(pool, query, page_size, None, None)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_start_with_offset_zero_and_empty_buffer() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = make_reader(pool, "SELECT id, name FROM items", None);
        assert_eq!(reader.offset.get(), 0, "initial offset should be 0");
        assert!(
            reader.buffer.borrow().is_empty(),
            "initial buffer should be empty"
        );
        assert_eq!(reader.page_size, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_none_when_table_is_empty() {
        let pool = pool_with_rows(&[]).await;
        let reader = make_reader(pool, "SELECT id, name FROM items", None);
        let result = reader.read().unwrap();
        assert!(result.is_none(), "empty table should yield None");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_all_items_without_pagination() {
        let pool = pool_with_rows(&[(1, "alice"), (2, "bob")]).await;
        let reader = make_reader(pool, "SELECT id, name FROM items ORDER BY id", None);

        let first = reader.read().unwrap().expect("first item should exist");
        assert_eq!(first.name, "alice");

        let second = reader.read().unwrap().expect("second item should exist");
        assert_eq!(second.name, "bob");

        assert!(
            reader.read().unwrap().is_none(),
            "should return None after all items"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_advance_offset_on_each_read() {
        let pool = pool_with_rows(&[(1, "x"), (2, "y")]).await;
        let reader = make_reader(pool, "SELECT id, name FROM items ORDER BY id", None);

        assert_eq!(reader.offset.get(), 0);
        reader.read().unwrap();
        assert_eq!(
            reader.offset.get(),
            1,
            "offset should increment after each read"
        );
        reader.read().unwrap();
        assert_eq!(reader.offset.get(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_all_items_with_pagination() {
        let pool = pool_with_rows(&[(1, "a"), (2, "b"), (3, "c"), (4, "d")]).await;
        let reader = make_reader(pool, "SELECT id, name FROM items ORDER BY id", Some(2));

        let mut count = 0;
        while reader.read().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 4, "should read all 4 items across 2 pages");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_single_item() {
        let pool = pool_with_rows(&[(42, "only")]).await;
        let reader = make_reader(pool, "SELECT id, name FROM items", None);

        let item = reader
            .read()
            .unwrap()
            .expect("should return the single item");
        assert_eq!(item.id, 42);
        assert_eq!(item.name, "only");
        assert!(
            reader.read().unwrap().is_none(),
            "should return None after the only item"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_all_items_with_keyset_pagination() {
        let pool = pool_with_rows(&[(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items",
            Some(2),
            Some("id".to_string()),
            Some(Box::new(|r: &Row| r.id.to_string())),
        );

        let mut names = vec![];
        while let Some(item) = reader.read().unwrap() {
            names.push(item.name.clone());
        }
        assert_eq!(names, vec!["a", "b", "c", "d", "e"], "keyset should return all items in order");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_update_last_cursor_after_each_read_with_keyset() {
        let pool = pool_with_rows(&[(10, "x"), (20, "y")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items",
            Some(2),
            Some("id".to_string()),
            Some(Box::new(|r: &Row| r.id.to_string())),
        );

        assert!(reader.last_cursor.borrow().is_none(), "cursor should be None before first read");
        reader.read().unwrap();
        assert_eq!(
            reader.last_cursor.borrow().as_deref(),
            Some("10"),
            "cursor should be updated after first read"
        );
        reader.read().unwrap();
        assert_eq!(
            reader.last_cursor.borrow().as_deref(),
            Some("20"),
            "cursor should reflect last read item"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_none_for_empty_table_with_keyset() {
        let pool = pool_with_rows(&[]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items",
            Some(2),
            Some("id".to_string()),
            Some(Box::new(|r: &Row| r.id.to_string())),
        );
        assert!(reader.read().unwrap().is_none(), "empty table should yield None with keyset");
    }
}
