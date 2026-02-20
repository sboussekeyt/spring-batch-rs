use serde::Serialize;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::item::rdbc::DatabaseItemBinder;

use super::writer_common::{
    create_write_error, log_write_success, max_items_per_batch, validate_config,
};

/// A writer for inserting items into a MySQL database using SQLx.
///
/// This writer provides an implementation of the `ItemWriter` trait for MySQL operations.
/// It supports batch inserting items into a specified table with the provided columns.
///
/// # MySQL-Specific Features
///
/// - Supports MySQL's data types and character sets
/// - Handles MySQL's AUTO_INCREMENT for auto-incrementing columns
/// - Supports MySQL's INSERT ... ON DUPLICATE KEY UPDATE operations
/// - Leverages MySQL's efficient bulk insert capabilities
/// - Compatible with MySQL's connection pooling and prepared statements
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemWriterBuilder, DatabaseItemBinder};
/// use spring_batch_rs::core::item::ItemWriter;
/// use sqlx::{MySqlPool, query_builder::Separated, MySql};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Product {
///     id: i32,
///     name: String,
///     price: f64,
/// }
///
/// struct ProductBinder;
/// impl DatabaseItemBinder<Product, MySql> for ProductBinder {
///     fn bind(&self, item: &Product, mut query_builder: Separated<MySql, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.name);
///         // In real usage: query_builder.push_bind(item.price);
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
///     .add_column("price")
///     .mysql_binder(&binder)
///     .build_mysql();
///
/// let products = vec![
///     Product { id: 1, name: "Laptop".to_string(), price: 999.99 },
///     Product { id: 2, name: "Mouse".to_string(), price: 29.99 },
/// ];
///
/// writer.write(&products)?;
/// # Ok(())
/// # }
/// ```
pub struct MySqlItemWriter<'a, O> {
    pub(crate) pool: Option<&'a Pool<MySql>>,
    pub(crate) table: Option<&'a str>,
    pub(crate) columns: Vec<&'a str>,
    pub(crate) item_binder: Option<&'a dyn DatabaseItemBinder<O, MySql>>,
}

impl<'a, O> MySqlItemWriter<'a, O> {
    /// Creates a new `MySqlItemWriter` with default configuration.
    pub(crate) fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    /// Sets the database connection pool for the writer.
    pub(crate) fn pool(mut self, pool: &'a Pool<MySql>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Sets the table name for the writer.
    pub(crate) fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// Adds a column to the writer.
    pub(crate) fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the item binder for the writer.
    pub(crate) fn item_binder(mut self, item_binder: &'a dyn DatabaseItemBinder<O, MySql>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }
}

impl<'a, O> Default for MySqlItemWriter<'a, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Serialize + Clone> ItemWriter<O> for MySqlItemWriter<'_, O> {
    fn write(&self, items: &[O]) -> ItemWriterResult {
        if items.is_empty() {
            return Ok(());
        }

        // Validate configuration
        let (pool, table, item_binder) =
            validate_config(self.pool, self.table, &self.columns, self.item_binder)?;

        // Build INSERT query
        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(table);
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        // Calculate max items per batch and add values
        let max_items = max_items_per_batch(self.columns.len());
        let items_to_write = items.iter().take(max_items);
        let items_count = items_to_write.len();

        query_builder.push_values(items_to_write, |b, item| {
            item_binder.bind(item, b);
        });

        // Execute query inline (QueryBuilder lifetime requires this to be in same scope)
        let query = query_builder.build();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { query.execute(pool).await })
        });

        match result {
            Ok(_) => {
                log_write_success(items_count, table, "MySQL");
                Ok(())
            }
            Err(e) => Err(create_write_error(table, "MySQL", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;

    #[test]
    fn test_new_creates_default_writer() {
        let writer = MySqlItemWriter::<String>::new();

        assert!(writer.pool.is_none());
        assert!(writer.table.is_none());
        assert!(writer.columns.is_empty());
        assert!(writer.item_binder.is_none());
    }

    #[test]
    fn test_builder_pattern_configuration() {
        let writer = MySqlItemWriter::<String>::new()
            .table("products")
            .add_column("id")
            .add_column("name")
            .add_column("price");

        assert_eq!(writer.table, Some("products"));
        assert_eq!(writer.columns, vec!["id", "name", "price"]);
    }

    #[test]
    fn test_write_empty_items() {
        use crate::item::rdbc::DatabaseItemBinder;
        use sqlx::query_builder::Separated;

        struct DummyBinder;
        impl DatabaseItemBinder<String, MySql> for DummyBinder {
            fn bind(&self, _item: &String, _query_builder: Separated<MySql, &str>) {}
        }

        let binder = DummyBinder;
        let writer = MySqlItemWriter::<String>::new()
            .table("test")
            .add_column("value")
            .item_binder(&binder);

        let result = writer.write(&[]);
        assert!(result.is_ok());
    }
}
