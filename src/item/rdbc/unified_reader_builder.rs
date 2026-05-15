use sqlx::{
    FromRow, MySql, Pool, Postgres, Sqlite, mysql::MySqlRow, postgres::PgRow, sqlite::SqliteRow,
};
use std::marker::PhantomData;

use super::database_type::DatabaseType;
use super::mysql_reader::MySqlRdbcItemReader;
use super::postgres_reader::PostgresRdbcItemReader;
use super::select_builder::SelectBuilder;
use super::sqlite_reader::SqliteRdbcItemReader;

/// Source of the SQL query for an RDBC item reader.
///
/// This is an internal type used by [`RdbcItemReaderBuilder`] to track whether
/// the query was provided as a raw string via [`.query()`] or constructed via
/// a [`SelectBuilder`] with [`.select()`].
enum QuerySource<'a> {
    /// A raw SQL string provided directly by the caller.
    Raw(&'a str),
    /// A SQL string built by [`SelectBuilder`].
    Built(String),
}

/// Unified builder for creating RDBC item readers for any supported database type.
///
/// This builder provides a consistent API for constructing database readers
/// regardless of the underlying database (PostgreSQL, MySQL, or SQLite).
/// Users specify the database type and connection pool, and the builder
/// handles the creation of the appropriate reader implementation.
///
/// # Type Parameters
///
/// * `I` - The item type that implements the appropriate `FromRow` trait for the database
///
/// # Examples
///
/// ## PostgreSQL
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, DatabaseType};
/// use sqlx::PgPool;
/// # use serde::Deserialize;
/// # #[derive(sqlx::FromRow, Clone, Deserialize)]
/// # struct User { id: i32, name: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
///
/// let reader = RdbcItemReaderBuilder::<User>::new()
///     .postgres(pool)
///     .query("SELECT id, name FROM users")
///     .with_page_size(100)
///     .build_postgres();
/// # Ok(())
/// # }
/// ```
///
/// ## MySQL
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, DatabaseType};
/// use sqlx::MySqlPool;
/// # use serde::Deserialize;
/// # #[derive(sqlx::FromRow, Clone, Deserialize)]
/// # struct Product { id: i32, name: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
///
/// let reader = RdbcItemReaderBuilder::<Product>::new()
///     .mysql(pool)
///     .query("SELECT id, name FROM products")
///     .with_page_size(100)
///     .build_mysql();
/// # Ok(())
/// # }
/// ```
///
/// ## SQLite
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, DatabaseType};
/// use sqlx::SqlitePool;
/// # use serde::Deserialize;
/// # #[derive(sqlx::FromRow, Clone, Deserialize)]
/// # struct Task { id: i32, title: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
///
/// let reader = RdbcItemReaderBuilder::<Task>::new()
///     .sqlite(pool)
///     .query("SELECT id, title FROM tasks")
///     .with_page_size(100)
///     .build_sqlite();
/// # Ok(())
/// # }
/// ```
pub struct RdbcItemReaderBuilder<'a, I> {
    postgres_pool: Option<Pool<Postgres>>,
    mysql_pool: Option<Pool<MySql>>,
    sqlite_pool: Option<Pool<Sqlite>>,
    query_source: Option<QuerySource<'a>>,
    page_size: Option<i32>,
    db_type: Option<DatabaseType>,
    keyset_column: Option<String>,
    #[allow(clippy::type_complexity)]
    keyset_key_fn: Option<Box<dyn Fn(&I) -> String>>,
    _phantom: PhantomData<I>,
}

impl<'a, I> RdbcItemReaderBuilder<'a, I> {
    /// Creates a new unified reader builder with default configuration.
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
            mysql_pool: None,
            sqlite_pool: None,
            query_source: None,
            page_size: None,
            db_type: None,
            keyset_column: None,
            keyset_key_fn: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the PostgreSQL connection pool and database type.
    ///
    /// # Arguments
    /// * `pool` - The PostgreSQL connection pool
    ///
    /// # Returns
    /// The updated builder instance configured for PostgreSQL
    pub fn postgres(mut self, pool: Pool<Postgres>) -> Self {
        self.postgres_pool = Some(pool);
        self.db_type = Some(DatabaseType::Postgres);
        self
    }

    /// Sets the MySQL connection pool and database type.
    ///
    /// # Arguments
    /// * `pool` - The MySQL connection pool
    ///
    /// # Returns
    /// The updated builder instance configured for MySQL
    pub fn mysql(mut self, pool: Pool<MySql>) -> Self {
        self.mysql_pool = Some(pool);
        self.db_type = Some(DatabaseType::MySql);
        self
    }

    /// Sets the SQLite connection pool and database type.
    ///
    /// # Arguments
    /// * `pool` - The SQLite connection pool
    ///
    /// # Returns
    /// The updated builder instance configured for SQLite
    pub fn sqlite(mut self, pool: Pool<Sqlite>) -> Self {
        self.sqlite_pool = Some(pool);
        self.db_type = Some(DatabaseType::Sqlite);
        self
    }

    /// Sets the SQL query for the reader.
    ///
    /// The query should not include LIMIT/OFFSET clauses as these are handled
    /// automatically when page_size is configured.
    ///
    /// # Arguments
    /// * `query` - The SQL query to execute
    ///
    /// # Returns
    /// The updated builder instance
    pub fn query(mut self, query: &'a str) -> Self {
        self.query_source = Some(QuerySource::Raw(query));
        self
    }

    /// Configures the reader query using a [`SelectBuilder`].
    ///
    /// This is an ergonomic alternative to [`Self::query`] that lets you build the
    /// SQL statement through a fluent API instead of writing raw SQL. If the
    /// [`SelectBuilder`] was configured with [`SelectBuilder::order_by_keyset`], the keyset
    /// column and key function are automatically propagated to the reader.
    ///
    /// Calling `.select()` after `.query()` (or vice-versa) is allowed; the
    /// **last** call wins.
    ///
    /// # Arguments
    ///
    /// * `builder` - A [`SelectBuilder`] instance ready to be compiled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, SelectBuilder};
    /// use sqlx::SqlitePool;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct Task { id: i32, title: String }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = SqlitePool::connect("sqlite::memory:").await?;
    ///
    /// let reader = RdbcItemReaderBuilder::<Task>::new()
    ///     .sqlite(pool)
    ///     .select(
    ///         SelectBuilder::from("tasks")
    ///             .columns(&["id", "title"])
    ///             .where_eq("done", false)
    ///             .order_by_asc("id"),
    ///     )
    ///     .with_page_size(50)
    ///     .build_sqlite();
    /// # Ok(())
    /// # }
    /// ```
    pub fn select(mut self, builder: SelectBuilder<I>) -> Self {
        let sql = if builder.keyset_column.is_some() {
            builder.build_sql_no_order()
        } else {
            builder.build_sql()
        };
        if let Some(col) = builder.keyset_column {
            self.keyset_column = Some(col);
        }
        if let Some(key_fn) = builder.keyset_key_fn {
            self.keyset_key_fn = Some(key_fn);
        }
        self.query_source = Some(QuerySource::Built(sql));
        self
    }

    /// Sets the page size for paginated reading.
    ///
    /// When set, the reader will fetch data in chunks of this size to manage
    /// memory usage efficiently.
    ///
    /// # Arguments
    /// * `page_size` - Number of items to read per page
    ///
    /// # Returns
    /// The updated builder instance
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Enables keyset (cursor) pagination instead of LIMIT/OFFSET.
    ///
    /// Keyset pagination is O(log n) per page regardless of dataset size, making it
    /// significantly faster than LIMIT/OFFSET for large tables.
    ///
    /// # Requirements
    ///
    /// - The query must **not** include `WHERE`, `ORDER BY`, or `LIMIT` clauses — the
    ///   framework appends them automatically.
    /// - The keyset column must be indexed and have unique, sortable values (e.g.
    ///   primary key, UUID, zero-padded string ID).
    /// - [`Self::with_page_size`] must also be set.
    ///
    /// # Arguments
    ///
    /// * `column` - Column name used as the cursor (appended to `WHERE` and `ORDER BY`).
    /// * `key_fn` - Closure that extracts the cursor value from an item as a `String`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;
    /// use sqlx::PgPool;
    /// # use serde::Deserialize;
    /// # #[derive(sqlx::FromRow, Clone, Deserialize)]
    /// # struct Order { order_id: String, amount: f64 }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    ///
    /// let reader = RdbcItemReaderBuilder::<Order>::new()
    ///     .postgres(pool)
    ///     .query("SELECT order_id, amount FROM orders")
    ///     .with_page_size(1_000)
    ///     .with_keyset("order_id", |o: &Order| o.order_id.clone())
    ///     .build_postgres();
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_keyset(mut self, column: &str, key_fn: impl Fn(&I) -> String + 'static) -> Self {
        self.keyset_column = Some(column.to_string());
        self.keyset_key_fn = Some(Box::new(key_fn));
        self
    }
}

impl<'a, I> RdbcItemReaderBuilder<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    /// Builds a PostgreSQL reader.
    ///
    /// # Returns
    /// A configured `PostgresRdbcItemReader` instance
    ///
    /// # Panics
    /// Panics if PostgreSQL pool or query are missing
    pub fn build_postgres(self) -> PostgresRdbcItemReader<I> {
        let query = match self
            .query_source
            .expect("Query is required — call .query() or .select()")
        {
            QuerySource::Raw(s) => s.to_string(),
            QuerySource::Built(s) => s,
        };
        PostgresRdbcItemReader::new(
            self.postgres_pool.expect("PostgreSQL pool is required"),
            query,
            self.page_size,
            self.keyset_column,
            self.keyset_key_fn,
        )
    }
}

impl<'a, I> RdbcItemReaderBuilder<'a, I>
where
    for<'r> I: FromRow<'r, MySqlRow> + Send + Unpin + Clone,
{
    /// Builds a MySQL reader.
    ///
    /// # Returns
    /// A configured `MySqlRdbcItemReader` instance
    ///
    /// # Panics
    /// Panics if MySQL pool or query are missing
    pub fn build_mysql(self) -> MySqlRdbcItemReader<I> {
        let query = match self
            .query_source
            .expect("Query is required — call .query() or .select()")
        {
            QuerySource::Raw(s) => s.to_string(),
            QuerySource::Built(s) => s,
        };
        MySqlRdbcItemReader::new(
            self.mysql_pool.expect("MySQL pool is required"),
            query,
            self.page_size,
            self.keyset_column,
            self.keyset_key_fn,
        )
    }
}

impl<'a, I> RdbcItemReaderBuilder<'a, I>
where
    for<'r> I: FromRow<'r, SqliteRow> + Send + Unpin + Clone,
{
    /// Builds a SQLite reader.
    ///
    /// # Returns
    /// A configured `SqliteRdbcItemReader` instance
    ///
    /// # Panics
    /// Panics if SQLite pool or query are missing
    pub fn build_sqlite(self) -> SqliteRdbcItemReader<I> {
        let query = match self
            .query_source
            .expect("Query is required — call .query() or .select()")
        {
            QuerySource::Raw(s) => s.to_string(),
            QuerySource::Built(s) => s,
        };
        SqliteRdbcItemReader::new(
            self.sqlite_pool.expect("SQLite pool is required"),
            query,
            self.page_size,
            self.keyset_column,
            self.keyset_key_fn,
        )
    }
}

impl<'a, I> Default for RdbcItemReaderBuilder<'a, I> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::select_builder::SelectBuilder;
    use sqlx::{FromRow, SqlitePool};

    #[derive(Clone, FromRow)]
    struct Dummy {
        id: i32,
    }

    #[test]
    fn should_create_via_default() {
        // Default == new(), both should produce identical builders
        let _b = RdbcItemReaderBuilder::<Dummy>::default();
    }

    #[test]
    #[should_panic(expected = "SQLite pool is required")]
    fn should_panic_when_building_sqlite_without_pool() {
        let _ = RdbcItemReaderBuilder::<Dummy>::new()
            .query("SELECT id FROM t")
            .build_sqlite();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_build_sqlite_reader_with_pool_and_query() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .query("SELECT 1 AS id")
            .build_sqlite();
        assert_eq!(reader.query, "SELECT 1 AS id");
        assert_eq!(reader.page_size, None);
        assert_eq!(reader.offset.get(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_propagate_page_size_to_sqlite_reader() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .query("SELECT 1 AS id")
            .with_page_size(25)
            .build_sqlite();
        assert_eq!(reader.page_size, Some(25));
    }

    #[test]
    #[should_panic(expected = "PostgreSQL pool is required")]
    fn should_panic_when_building_postgres_without_pool() {
        let _ = RdbcItemReaderBuilder::<Dummy>::new()
            .query("SELECT id FROM t")
            .build_postgres();
    }

    #[test]
    #[should_panic(expected = "MySQL pool is required")]
    fn should_panic_when_building_mysql_without_pool() {
        let _ = RdbcItemReaderBuilder::<Dummy>::new()
            .query("SELECT id FROM t")
            .build_mysql();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_propagate_keyset_to_sqlite_reader() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .query("SELECT 1 AS id")
            .with_page_size(5)
            .with_keyset("id", |d: &Dummy| d.id.to_string())
            .build_sqlite();
        assert_eq!(
            reader.keyset_column.as_deref(),
            Some("id"),
            "keyset column should be propagated to reader"
        );
        assert!(
            reader.keyset_key.is_some(),
            "keyset key fn should be propagated to reader"
        );
        assert!(
            reader.last_cursor.borrow().is_none(),
            "cursor starts as None"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_build_sqlite_reader_from_select_builder() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .select(
                SelectBuilder::from("items")
                    .columns(&["id"])
                    .where_eq("active", true)
                    .order_by_asc("id"),
            )
            .build_sqlite();
        assert_eq!(
            reader.query,
            "SELECT id FROM items WHERE active = true ORDER BY id ASC",
            "select builder SQL should be stored in reader"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_propagate_keyset_from_select_builder_to_sqlite_reader() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .select(SelectBuilder::from("items").order_by_keyset("id", |d: &Dummy| {
                d.id.to_string()
            }))
            .with_page_size(10)
            .build_sqlite();
        assert_eq!(
            reader.keyset_column.as_deref(),
            Some("id"),
            "keyset column should propagate from SelectBuilder"
        );
        assert!(
            reader.keyset_key.is_some(),
            "keyset key fn should propagate from SelectBuilder"
        );
        assert_eq!(
            reader.query,
            "SELECT * FROM items",
            "keyset select builder must store SQL without ORDER BY to avoid double ORDER BY in read_page"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_prefer_select_over_query_when_called_last() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .query("SELECT id FROM old_table")
            .select(SelectBuilder::from("new_table").columns(&["id"]))
            .build_sqlite();
        assert_eq!(
            reader.query, "SELECT id FROM new_table",
            "select() called last should win"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_prefer_query_over_select_when_called_last() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let reader = RdbcItemReaderBuilder::<Dummy>::new()
            .sqlite(pool)
            .select(SelectBuilder::from("old_table").columns(&["id"]))
            .query("SELECT id FROM new_table")
            .build_sqlite();
        assert_eq!(
            reader.query, "SELECT id FROM new_table",
            "query() called last should win"
        );
    }
}
