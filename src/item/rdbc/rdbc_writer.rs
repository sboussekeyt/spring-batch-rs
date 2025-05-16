use serde::Serialize;
use sqlx::{query_builder::Separated, Any, Pool, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};

// The number of parameters in MySQL must fit in a `u16`.
const BIND_LIMIT: usize = 65535;

/// Trait for binding item data to SQL query parameters.
///
/// This trait is responsible for taking a single item and binding its properties
/// to a SQL query as parameters in the appropriate order.
pub trait RdbcItemBinder<T> {
    /// Binds the properties of an item to a separated query builder.
    ///
    /// # Arguments
    ///
    /// * `item` - The item whose properties should be bound.
    /// * `query_builder` - The separated query builder to bind parameters to.
    fn bind(&self, item: &T, query_builder: Separated<Any, &str>);
}

/// A writer for inserting items into a relational database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for database operations.
/// It supports batch inserting items into a specified table with the provided columns.
///
/// # Design
///
/// - Uses a connection pool to efficiently manage database connections
/// - Leverages SQLx's query builder for constructing parameterized SQL statements
/// - Uses a custom item binder to handle the conversion from domain objects to SQL parameters
/// - Handles batch inserts efficiently within the database parameter limit
///
/// # Limitations
///
/// - Currently has a parameter limit of 65,535 (MySQL's limit)
/// - Performs insert operations but does not support update or upsert operations
/// - Does not handle database-specific SQL syntax differences (relies on SQLx's Any driver)
pub struct RdbcItemWriter<'a, W> {
    pool: &'a Pool<Any>,
    table: &'a str,
    columns: Vec<&'a str>,
    item_binder: &'a dyn RdbcItemBinder<W>,
}

impl<'a, W> RdbcItemWriter<'a, W> {
    /// Creates a new instance of `RdbcItemWriter`.
    ///
    /// # Arguments
    ///
    /// * `pool` - A reference to the connection pool.
    /// * `table` - The name of the database table.
    /// * `columns` - A vector of column names.
    /// * `item_binder` - A reference to the item binder.
    ///
    /// # Returns
    ///
    /// A new instance of `RdbcItemWriter`.
    pub fn new(
        pool: &'a Pool<Any>,
        table: &'a str,
        columns: Vec<&'a str>,
        item_binder: &'a dyn RdbcItemBinder<W>,
    ) -> Self {
        Self {
            pool,
            table,
            columns,
            item_binder,
        }
    }
}

impl<W: Serialize + Clone> ItemWriter<W> for RdbcItemWriter<'_, W> {
    /// Writes the items to the database.
    ///
    /// This method constructs an INSERT statement with the following format:
    /// ```sql
    /// INSERT INTO [table] ([column1], [column2], ...) VALUES (?, ?, ...), (?, ?, ...), ...
    /// ```
    ///
    /// The method handles:
    /// - Creating the basic INSERT statement with table and column names
    /// - Binding values for each item using the provided item binder
    /// - Executing the query in a blocking manner (converting from async to sync)
    /// - Limiting the number of parameters to stay within database constraints
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of items to be written.
    ///
    /// # Returns
    ///
    /// An `ItemWriterResult` indicating the result of the write operation.
    fn write(&self, items: &[W]) -> ItemWriterResult {
        // Build the base INSERT statement with table and column names
        let mut query_builder = QueryBuilder::new("INSERT INTO ");

        query_builder.push(self.table);
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        // Add VALUES clause with proper parameter binding for each item
        // Limit the number of items to stay within database parameter limits
        query_builder.push_values(
            items.iter().take(BIND_LIMIT / self.columns.len()),
            |b, item| {
                self.item_binder.bind(item, b);
            },
        );

        let query = query_builder.build();

        // Execute the query in a blocking manner
        let _result = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { query.execute(self.pool).await.unwrap() })
        });

        Ok(())
    }
}

/// Builder for creating an `RdbcItemWriter` with a fluent interface.
///
/// This builder implements the Builder pattern to allow for a more readable
/// and flexible way to construct `RdbcItemWriter` instances. It enforces
/// required parameters and provides a clear API for configuring the writer.
///
/// # Example
///
/// ```rust
/// let writer = RdbcItemWriterBuilder::new()
///     .pool(&pool)
///     .table("users")
///     .add_column("id")
///     .add_column("name")
///     .add_column("email")
///     .item_binder(&user_binder)
///     .build();
/// ```
#[derive(Default)]
pub struct RdbcItemWriterBuilder<'a, T> {
    pool: Option<&'a Pool<Any>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn RdbcItemBinder<T>>,
}

impl<'a, T> RdbcItemWriterBuilder<'a, T> {
    /// Creates a new instance of `RdbcItemWriterBuilder`.
    ///
    /// Initializes an empty builder with no configuration. All required
    /// parameters must be set before calling `build()`.
    ///
    /// # Returns
    ///
    /// A new instance of `RdbcItemWriterBuilder`.
    pub fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    /// Sets the table name for the item writer.
    ///
    /// This parameter is required. The builder will panic during build
    /// if this parameter is not set.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the database table.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemWriterBuilder` instance.
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// Sets the connection pool for the item writer.
    ///
    /// This parameter is required. The builder will panic during build
    /// if this parameter is not set.
    ///
    /// # Arguments
    ///
    /// * `pool` - A reference to the connection pool.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemWriterBuilder` instance.
    pub fn pool(mut self, pool: &'a Pool<Any>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the item binder for the item writer.
    ///
    /// This parameter is required. The builder will panic during build
    /// if this parameter is not set. The item binder is responsible for
    /// mapping item properties to SQL parameters.
    ///
    /// # Arguments
    ///
    /// * `item_binder` - A reference to the item binder.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemWriterBuilder` instance.
    pub fn item_binder(mut self, item_binder: &'a dyn RdbcItemBinder<T>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }

    /// Adds a column to the item writer.
    ///
    /// This method can be called multiple times to add multiple columns.
    /// At least one column must be added before calling `build()`.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to add.
    ///
    /// # Returns
    ///
    /// The updated `RdbcItemWriterBuilder` instance.
    pub fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    /// Builds an instance of `RdbcItemWriter` based on the configured parameters.
    ///
    /// This method validates that all required parameters have been set and then
    /// constructs a new `RdbcItemWriter` instance with those parameters.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - The table name is not set (call `table()` first)
    /// - No columns are added (call `add_column()` at least once)
    /// - The connection pool is not set (call `pool()` first)
    /// - The item binder is not set (call `item_binder()` first)
    ///
    /// # Returns
    ///
    /// An instance of `RdbcItemWriter`.
    pub fn build(self) -> RdbcItemWriter<'a, T> {
        if self.table.is_none() {
            panic!("Table name is mandatory");
        }

        if self.columns.is_empty() {
            panic!("One or more columns are required");
        }

        RdbcItemWriter::new(
            self.pool.unwrap(),
            self.table.unwrap(),
            self.columns.clone(),
            self.item_binder.unwrap(),
        )
    }
}
