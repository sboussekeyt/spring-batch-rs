/// Represents the supported database types for RDBC operations.
///
/// This enum allows users to specify which database backend to use
/// when configuring readers and writers through the unified builder API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    Postgres,
    /// MySQL database
    MySql,
    /// SQLite database
    Sqlite,
}
