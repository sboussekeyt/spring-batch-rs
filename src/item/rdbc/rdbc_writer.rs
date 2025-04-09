use serde::Serialize;
use sqlx::{query_builder::Separated, Any, Pool, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};

// The number of parameters in MySQL must fit in a `u16`.
const BIND_LIMIT: usize = 65535;

pub trait RdbcItemBinder<T> {
    fn bind(&self, item: &T, query_builder: Separated<Any, &str>);
}

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

impl<'a, W: Serialize + Clone> ItemWriter<W> for RdbcItemWriter<'a, W> {
    /// Writes the items to the database.
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of items to be written.
    ///
    /// # Returns
    ///
    /// An `ItemWriterResult` indicating the result of the write operation.
    fn write(&self, items: &[W]) -> ItemWriterResult {
        let mut query_builder = QueryBuilder::new("INSERT INTO ");

        query_builder.push(self.table);
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        query_builder.push_values(
            items.iter().take(BIND_LIMIT / self.columns.len()),
            |b, item| {
                self.item_binder.bind(item, b.into());
            },
        );

        let query = query_builder.build();

        let _result = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { query.execute(self.pool).await.unwrap() })
        });

        Ok(())
    }
}

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
    /// # Panics
    ///
    /// This method will panic if the table name is not set or if no columns are added.
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
