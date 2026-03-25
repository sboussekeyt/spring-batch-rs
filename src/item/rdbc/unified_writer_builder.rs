use sqlx::{MySql, Pool, Postgres, Sqlite};

use super::database_type::DatabaseType;
use super::mysql_writer::MySqlItemWriter;
use super::postgres_writer::PostgresItemWriter;
use super::sqlite_writer::SqliteItemWriter;
use super::DatabaseItemBinder;

/// Unified builder for creating RDBC item writers for any supported database type.
///
/// This builder provides a consistent API for constructing database writers
/// regardless of the underlying database (PostgreSQL, MySQL, or SQLite).
/// Users specify the database type, connection pool, table, and columns,
/// and the builder handles the creation of the appropriate writer implementation.
///
/// # Type Parameters
///
/// * `O` - The item type to write
///
/// # Examples
///
/// ## PostgreSQL
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use sqlx::{PgPool, query_builder::Separated, Postgres};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// struct UserBinder;
/// impl DatabaseItemBinder<User, Postgres> for UserBinder {
///     fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.name);
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
/// let binder = UserBinder;
///
/// let writer = RdbcItemWriterBuilder::<User>::new()
///     .postgres(&pool)
///     .table("users")
///     .add_column("id")
///     .add_column("name")
///     .postgres_binder(&binder)
///     .build_postgres();
/// # Ok(())
/// # }
/// ```
///
/// ## MySQL
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use sqlx::{MySqlPool, query_builder::Separated, MySql};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Product {
///     id: i32,
///     name: String,
/// }
///
/// struct ProductBinder;
/// impl DatabaseItemBinder<Product, MySql> for ProductBinder {
///     fn bind(&self, item: &Product, mut query_builder: Separated<MySql, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.name);
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
/// let binder = ProductBinder;
///
/// let writer = RdbcItemWriterBuilder::<Product>::new()
///     .mysql(&pool)
///     .table("products")
///     .add_column("id")
///     .add_column("name")
///     .mysql_binder(&binder)
///     .build_mysql();
/// # Ok(())
/// # }
/// ```
///
/// ## SQLite
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use sqlx::{SqlitePool, query_builder::Separated, Sqlite};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Task {
///     id: i32,
///     title: String,
/// }
///
/// struct TaskBinder;
/// impl DatabaseItemBinder<Task, Sqlite> for TaskBinder {
///     fn bind(&self, item: &Task, mut query_builder: Separated<Sqlite, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.title);
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
/// let binder = TaskBinder;
///
/// let writer = RdbcItemWriterBuilder::<Task>::new()
///     .sqlite(&pool)
///     .table("tasks")
///     .add_column("id")
///     .add_column("title")
///     .sqlite_binder(&binder)
///     .build_sqlite();
/// # Ok(())
/// # }
/// ```
pub struct RdbcItemWriterBuilder<'a, O> {
    postgres_pool: Option<&'a Pool<Postgres>>,
    mysql_pool: Option<&'a Pool<MySql>>,
    sqlite_pool: Option<&'a Pool<Sqlite>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    postgres_binder: Option<&'a dyn DatabaseItemBinder<O, Postgres>>,
    mysql_binder: Option<&'a dyn DatabaseItemBinder<O, MySql>>,
    sqlite_binder: Option<&'a dyn DatabaseItemBinder<O, Sqlite>>,
    db_type: Option<DatabaseType>,
}

impl<'a, O> RdbcItemWriterBuilder<'a, O> {
    /// Creates a new unified writer builder with default configuration.
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
            mysql_pool: None,
            sqlite_pool: None,
            table: None,
            columns: Vec::new(),
            postgres_binder: None,
            mysql_binder: None,
            sqlite_binder: None,
            db_type: None,
        }
    }

    /// Sets the PostgreSQL connection pool and database type.
    ///
    /// # Arguments
    /// * `pool` - The PostgreSQL connection pool
    ///
    /// # Returns
    /// The updated builder instance configured for PostgreSQL
    pub fn postgres(mut self, pool: &'a Pool<Postgres>) -> Self {
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
    pub fn mysql(mut self, pool: &'a Pool<MySql>) -> Self {
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
    pub fn sqlite(mut self, pool: &'a Pool<Sqlite>) -> Self {
        self.sqlite_pool = Some(pool);
        self.db_type = Some(DatabaseType::Sqlite);
        self
    }

    /// Sets the table name for the writer.
    ///
    /// # Arguments
    /// * `table` - The database table name
    ///
    /// # Returns
    /// The updated builder instance
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// Adds a column to the writer.
    ///
    /// # Arguments
    /// * `column` - The column name
    ///
    /// # Returns
    /// The updated builder instance
    pub fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the item binder for PostgreSQL.
    ///
    /// # Arguments
    /// * `binder` - The PostgreSQL-specific item binder
    ///
    /// # Returns
    /// The updated builder instance
    pub fn postgres_binder(mut self, binder: &'a dyn DatabaseItemBinder<O, Postgres>) -> Self {
        self.postgres_binder = Some(binder);
        self
    }

    /// Sets the item binder for MySQL.
    ///
    /// # Arguments
    /// * `binder` - The MySQL-specific item binder
    ///
    /// # Returns
    /// The updated builder instance
    pub fn mysql_binder(mut self, binder: &'a dyn DatabaseItemBinder<O, MySql>) -> Self {
        self.mysql_binder = Some(binder);
        self
    }

    /// Sets the item binder for SQLite.
    ///
    /// # Arguments
    /// * `binder` - The SQLite-specific item binder
    ///
    /// # Returns
    /// The updated builder instance
    pub fn sqlite_binder(mut self, binder: &'a dyn DatabaseItemBinder<O, Sqlite>) -> Self {
        self.sqlite_binder = Some(binder);
        self
    }

    /// Builds a PostgreSQL writer.
    ///
    /// # Returns
    /// A configured `PostgresItemWriter` instance
    ///
    /// # Panics
    /// Panics if required PostgreSQL-specific configuration is missing
    pub fn build_postgres(self) -> PostgresItemWriter<'a, O> {
        let mut writer = PostgresItemWriter::new();

        if let Some(pool) = self.postgres_pool {
            writer = writer.pool(pool);
        }

        if let Some(table) = self.table {
            writer = writer.table(table);
        }

        for column in self.columns {
            writer = writer.add_column(column);
        }

        if let Some(binder) = self.postgres_binder {
            writer = writer.item_binder(binder);
        }

        writer
    }

    /// Builds a MySQL writer.
    ///
    /// # Returns
    /// A configured `MySqlItemWriter` instance
    ///
    /// # Panics
    /// Panics if required MySQL-specific configuration is missing
    pub fn build_mysql(self) -> MySqlItemWriter<'a, O> {
        let mut writer = MySqlItemWriter::new();

        if let Some(pool) = self.mysql_pool {
            writer = writer.pool(pool);
        }

        if let Some(table) = self.table {
            writer = writer.table(table);
        }

        for column in self.columns {
            writer = writer.add_column(column);
        }

        if let Some(binder) = self.mysql_binder {
            writer = writer.item_binder(binder);
        }

        writer
    }

    /// Builds a SQLite writer.
    ///
    /// # Returns
    /// A configured `SqliteItemWriter` instance
    ///
    /// # Panics
    /// Panics if required SQLite-specific configuration is missing
    pub fn build_sqlite(self) -> SqliteItemWriter<'a, O> {
        let mut writer = SqliteItemWriter::new();

        if let Some(pool) = self.sqlite_pool {
            writer = writer.pool(pool);
        }

        if let Some(table) = self.table {
            writer = writer.table(table);
        }

        for column in self.columns {
            writer = writer.add_column(column);
        }

        if let Some(binder) = self.sqlite_binder {
            writer = writer.item_binder(binder);
        }

        writer
    }
}

impl<'a, O> Default for RdbcItemWriterBuilder<'a, O> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;
    use sqlx::query_builder::Separated;

    struct NopBinder;
    impl DatabaseItemBinder<String, Postgres> for NopBinder {
        fn bind(&self, _: &String, _: Separated<Postgres, &str>) {}
    }
    impl DatabaseItemBinder<String, MySql> for NopBinder {
        fn bind(&self, _: &String, _: Separated<MySql, &str>) {}
    }
    impl DatabaseItemBinder<String, Sqlite> for NopBinder {
        fn bind(&self, _: &String, _: Separated<Sqlite, &str>) {}
    }

    #[test]
    fn should_set_table_name_in_postgres_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("users")
            .build_postgres();
        assert_eq!(writer.table, Some("users"));
    }

    #[test]
    fn should_accumulate_columns_in_postgres_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .add_column("id")
            .add_column("name")
            .build_postgres();
        assert_eq!(writer.columns, vec!["id", "name"]);
    }

    #[test]
    fn should_transfer_table_and_columns_to_mysql_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("orders")
            .add_column("order_id")
            .add_column("total")
            .build_mysql();
        assert_eq!(writer.table, Some("orders"));
        assert_eq!(writer.columns, vec!["order_id", "total"]);
    }

    #[test]
    fn should_transfer_table_and_columns_to_sqlite_writer() {
        use crate::BatchError;
        // No pool configured → validate_config will fail on "pool", not on table/columns.
        // If table or columns were missing the error would mention those instead,
        // so reaching the "pool" error proves both were transferred correctly.
        let binder = NopBinder;
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("items")
            .add_column("sku")
            .sqlite_binder(&binder)
            .build_sqlite();
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(
                msg.contains("pool"),
                "table and columns were set so error should be about pool, got: {msg}"
            ),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_set_postgres_binder() {
        let binder = NopBinder;
        let writer = RdbcItemWriterBuilder::<String>::new()
            .postgres_binder(&binder)
            .build_postgres();
        assert!(writer.item_binder.is_some(), "postgres binder should be set");
    }

    #[test]
    fn should_set_mysql_binder() {
        let binder = NopBinder;
        let writer = RdbcItemWriterBuilder::<String>::new()
            .mysql_binder(&binder)
            .build_mysql();
        assert!(writer.item_binder.is_some(), "mysql binder should be set");
    }

    #[test]
    fn should_transfer_sqlite_binder_to_writer() {
        use crate::BatchError;
        // With binder set but no pool, write() should fail on "pool" not on "binder"
        let binder = NopBinder;
        let writer = RdbcItemWriterBuilder::<String>::new()
            .table("t")
            .add_column("v")
            .sqlite_binder(&binder)
            .build_sqlite();
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(
                msg.contains("pool"),
                "binder was set so error should be about pool, got: {msg}"
            ),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_transfer_sqlite_pool_to_writer() {
        use crate::BatchError;
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Pool set but no binder → error is "binder not configured"
        let writer = RdbcItemWriterBuilder::<String>::new()
            .sqlite(&pool)
            .table("t")
            .add_column("v")
            .build_sqlite();
        let result = writer.write(&["x".to_string()]);
        match result.err().unwrap() {
            BatchError::ItemWriter(msg) => assert!(
                msg.contains("binder"),
                "pool was set so error should be about binder, got: {msg}"
            ),
            e => panic!("expected ItemWriter, got {e:?}"),
        }
    }

    #[test]
    fn should_have_no_table_by_default_in_mysql_writer() {
        let writer = RdbcItemWriterBuilder::<String>::new().build_mysql();
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
    }

    #[test]
    fn should_create_via_default() {
        let _b = RdbcItemWriterBuilder::<String>::default();
    }
}
