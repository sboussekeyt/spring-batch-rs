#[cfg(feature = "logger")]
/// This module provides a logger item reader and writer implementation for Spring Batch.
pub mod logger;

#[cfg(feature = "csv")]
/// This module provides a CSV item reader and writer implementation for Spring Batch.
pub mod csv;

#[cfg(feature = "fake")]
/// This module provides a fake item reader and writer implementation for Spring Batch.
pub mod fake;

#[cfg(feature = "json")]
/// This module provides a JSON item reader and writer implementation for Spring Batch.
pub mod json;

#[cfg(feature = "xml")]
/// This module provides an XML item reader and writer implementation for Spring Batch.
pub mod xml;

#[cfg(feature = "rdbc-postgres")]
#[cfg(feature = "rdbc-mysql")]
#[cfg(feature = "rdbc-sqlite")]
/// This module provides an RDBC (PostgreSQL) item reader and writer implementation for Spring Batch.
pub mod rdbc;

#[cfg(feature = "mongodb")]
/// This module provides a MongoDB item reader and writer implementation for Spring Batch.
pub mod mongodb;

#[cfg(feature = "orm")]
/// This module provides an ORM item reader and writer implementation for Spring Batch (SeaORM-based).
pub mod orm;
