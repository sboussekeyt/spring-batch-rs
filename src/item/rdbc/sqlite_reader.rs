use std::cell::{Cell, RefCell};

use sqlx::{sqlite::SqliteRow, Execute, FromRow, Pool, QueryBuilder, Sqlite};

use super::reader_common::{calculate_page_index, should_load_page};
use crate::core::item::{ItemReader, ItemReaderResult};

/// SQLite RDBC Item Reader for batch processing
///
/// # Construction
///
/// This reader can only be created through `RdbcItemReaderBuilder`.
/// Direct construction is not available to ensure proper configuration.
pub struct SqliteRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Sqlite>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
}

impl<'a, I> SqliteRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    /// Creates a new SqliteRdbcItemReader with the specified parameters
    ///
    /// This constructor is only accessible within the crate to enforce the use
    /// of `RdbcItemReaderBuilder` for creating reader instances.
    pub(crate) fn new(pool: Pool<Sqlite>, query: &'a str, page_size: Option<i32>) -> Self {
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
        let mut query_builder = QueryBuilder::<Sqlite>::new(self.query);

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

impl<I> ItemReader<I> for SqliteRdbcItemReader<'_, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    /// Reads the next item from the SQLite database
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

    #[tokio::test(flavor = "multi_thread")]
    async fn should_start_with_offset_zero_and_empty_buffer() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = SqliteRdbcItemReader::<Row>::new(pool, "SELECT id, name FROM items", None);
        assert_eq!(reader.offset.get(), 0, "initial offset should be 0");
        assert!(reader.buffer.borrow().is_empty(), "initial buffer should be empty");
        assert_eq!(reader.page_size, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_none_when_table_is_empty() {
        let pool = pool_with_rows(&[]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(pool, "SELECT id, name FROM items", None);
        let result = reader.read().unwrap();
        assert!(result.is_none(), "empty table should yield None");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_all_items_without_pagination() {
        let pool = pool_with_rows(&[(1, "alice"), (2, "bob")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items ORDER BY id",
            None,
        );

        let first = reader.read().unwrap().expect("first item should exist");
        assert_eq!(first.name, "alice");

        let second = reader.read().unwrap().expect("second item should exist");
        assert_eq!(second.name, "bob");

        assert!(reader.read().unwrap().is_none(), "should return None after all items");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_advance_offset_on_each_read() {
        let pool = pool_with_rows(&[(1, "x"), (2, "y")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items ORDER BY id",
            None,
        );

        assert_eq!(reader.offset.get(), 0);
        reader.read().unwrap();
        assert_eq!(reader.offset.get(), 1, "offset should increment after each read");
        reader.read().unwrap();
        assert_eq!(reader.offset.get(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_all_items_with_pagination() {
        let pool = pool_with_rows(&[(1, "a"), (2, "b"), (3, "c"), (4, "d")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(
            pool,
            "SELECT id, name FROM items ORDER BY id",
            Some(2), // page_size = 2
        );

        let mut count = 0;
        while reader.read().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 4, "should read all 4 items across 2 pages");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_read_single_item() {
        let pool = pool_with_rows(&[(42, "only")]).await;
        let reader = SqliteRdbcItemReader::<Row>::new(pool, "SELECT id, name FROM items", None);

        let item = reader.read().unwrap().expect("should return the single item");
        assert_eq!(item.id, 42);
        assert_eq!(item.name, "only");
        assert!(reader.read().unwrap().is_none(), "should return None after the only item");
    }
}
