//! # Tasklet Module
//!
//! This module provides various tasklet implementations for common batch operations.
//! Tasklets are single-task operations that don't follow the chunk-oriented processing pattern.

#[cfg(feature = "zip")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
pub mod zip;
