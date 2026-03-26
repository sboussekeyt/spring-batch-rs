use sqlx::{query_builder::Separated, Database};

/// Common utilities for database item writers.
mod writer_common;

/// Common utilities for database item readers.
mod reader_common;

/// Database type enumeration.
mod database_type;

/// Unified reader builder for all database types.
mod unified_reader_builder;

/// Unified writer builder for all database types.
mod unified_writer_builder;

/// PostgreSQL-specific reader implementation.
pub mod postgres_reader;

/// MySQL-specific reader implementation.
pub mod mysql_reader;

/// SQLite-specific reader implementation.
pub mod sqlite_reader;

/// PostgreSQL-specific writer implementation.
pub mod postgres_writer;

/// MySQL-specific writer implementation.
pub mod mysql_writer;

/// SQLite-specific writer implementation.
pub mod sqlite_writer;

/// Trait for binding item data to database query parameters.
///
/// This trait is generic over the database type, allowing it to work with
/// PostgreSQL, MySQL, SQLite, and other databases supported by SQLx.
/// This provides a unified interface for database-specific item writers.
///
/// # Type Parameters
///
/// * `O` - The item type to bind
/// * `DB` - The SQLx database type (e.g., `Postgres`, `MySql`, `Sqlite`)
///
/// # Examples
///
/// ## PostgreSQL Implementation
/// ```no_run
/// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
/// use sqlx::{query_builder::Separated, Postgres};
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
/// ```
///
/// ## MySQL Implementation
/// ```no_run
/// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
/// use sqlx::{query_builder::Separated, MySql};
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
/// ```
///
/// ## SQLite Implementation
/// ```no_run
/// use spring_batch_rs::item::rdbc::DatabaseItemBinder;
/// use sqlx::{query_builder::Separated, Sqlite};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct Task {
///     id: i32,
///     title: String,
///     completed: bool,
/// }
///
/// struct TaskBinder;
/// impl DatabaseItemBinder<Task, Sqlite> for TaskBinder {
///     fn bind(&self, item: &Task, mut query_builder: Separated<Sqlite, &str>) {
///         let _ = (item, query_builder); // Placeholder to avoid unused warnings
///         // In real usage: query_builder.push_bind(item.id);
///         // In real usage: query_builder.push_bind(&item.title);
///         // In real usage: query_builder.push_bind(item.completed);
///     }
/// }
/// ```
pub trait DatabaseItemBinder<O, DB: Database> {
    /// Binds the properties of an item to a separated query builder.
    ///
    /// # Arguments
    ///
    /// * `item` - The item whose properties should be bound.
    /// * `query_builder` - The separated query builder to bind parameters to.
    fn bind(&self, item: &O, query_builder: Separated<DB, &str>);
}

// Re-export database-specific reader and writer types (for direct usage)
pub use mysql_reader::MySqlRdbcItemReader;
pub use mysql_writer::MySqlItemWriter;
pub use postgres_reader::PostgresRdbcItemReader;
pub use postgres_writer::PostgresItemWriter;
pub use sqlite_reader::SqliteRdbcItemReader;
pub use sqlite_writer::SqliteItemWriter;

// Re-export unified builder types (recommended API)
pub use database_type::DatabaseType;
pub use unified_reader_builder::RdbcItemReaderBuilder;
pub use unified_writer_builder::RdbcItemWriterBuilder;
