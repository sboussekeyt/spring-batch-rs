use sqlx::{
    mysql::MySqlRow, postgres::PgRow, sqlite::SqliteRow, FromRow, MySql, Pool, Postgres, Sqlite,
};
use std::marker::PhantomData;

use super::database_type::DatabaseType;
use super::mysql_reader::MySqlRdbcItemReader;
use super::postgres_reader::PostgresRdbcItemReader;
use super::sqlite_reader::SqliteRdbcItemReader;

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
    query: Option<&'a str>,
    page_size: Option<i32>,
    db_type: Option<DatabaseType>,
    _phantom: PhantomData<I>,
}

impl<'a, I> RdbcItemReaderBuilder<'a, I> {
    /// Creates a new unified reader builder with default configuration.
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
            mysql_pool: None,
            sqlite_pool: None,
            query: None,
            page_size: None,
            db_type: None,
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
        self.query = Some(query);
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
    pub fn build_postgres(self) -> PostgresRdbcItemReader<'a, I> {
        PostgresRdbcItemReader::new(
            self.postgres_pool.expect("PostgreSQL pool is required"),
            self.query.expect("Query is required"),
            self.page_size,
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
    pub fn build_mysql(self) -> MySqlRdbcItemReader<'a, I> {
        MySqlRdbcItemReader::new(
            self.mysql_pool.expect("MySQL pool is required"),
            self.query.expect("Query is required"),
            self.page_size,
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
    pub fn build_sqlite(self) -> SqliteRdbcItemReader<'a, I> {
        SqliteRdbcItemReader::new(
            self.sqlite_pool.expect("SQLite pool is required"),
            self.query.expect("Query is required"),
            self.page_size,
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

    // Tests verify builder does not panic on chaining and accepts valid inputs.
    // Field-level state is verified indirectly (fields may be private).
    // Full execution is covered by tests/rdbc_*.rs integration tests.

    #[test]
    fn should_build_with_query_and_page_size_without_panicking() {
        // Verify chaining does not panic
        let _builder = RdbcItemReaderBuilder::<String>::new()
            .query("SELECT id FROM users")
            .with_page_size(50);
        // If we get here, chaining worked
    }

    #[test]
    fn should_accept_various_page_sizes() {
        let _b1 = RdbcItemReaderBuilder::<String>::new().with_page_size(1);
        let _b2 = RdbcItemReaderBuilder::<String>::new().with_page_size(100);
        let _b3 = RdbcItemReaderBuilder::<String>::new().with_page_size(1000);
    }

    #[test]
    fn should_accept_various_sql_queries() {
        let _b = RdbcItemReaderBuilder::<String>::new()
            .query("SELECT * FROM table WHERE id > 0 ORDER BY id");
    }

    #[test]
    fn should_chain_all_non_pool_methods() {
        let _builder = RdbcItemReaderBuilder::<String>::new()
            .query("SELECT id, name FROM users")
            .with_page_size(100);
    }

    #[test]
    fn should_create_via_default() {
        let _builder = RdbcItemReaderBuilder::<String>::default();
    }
}
