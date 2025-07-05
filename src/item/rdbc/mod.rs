use sqlx::{query_builder::Separated, Database};

/// This module contains the RDBC reader implementation.
pub mod rdbc_reader;

/// This module contains the RDBC writer implementation.
pub mod rdbc_writer;

pub mod postgres_reader;

pub mod postgres_writer;

pub mod mysql_writer;

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
/// use spring_batch_rs::item::rdbc::{DatabaseItemBinder};
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
///         query_builder.push_bind(item.id);
///         query_builder.push_bind(&item.name);
///     }
/// }
/// ```
///
/// ## MySQL Implementation
/// ```no_run
/// use spring_batch_rs::item::rdbc::{DatabaseItemBinder};
/// use sqlx::{query_builder::Separated, MySql};
/// use serde::Serialize;
///
/// #[derive(Clone, Serialize)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// struct UserBinder;
/// impl DatabaseItemBinder<User, MySql> for UserBinder {
///     fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
///         query_builder.push_bind(item.id);
///         query_builder.push_bind(&item.name);
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

// Re-export the RDBC types for convenience
pub use mysql_writer::MySqlItemWriter;
pub use postgres_reader::{PostgresRdbcItemReader, PostgresRdbcItemReaderBuilder};
pub use postgres_writer::PostgresItemWriter;
pub use sqlite_writer::SqliteItemWriter;
