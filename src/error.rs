use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
/// Batch error
pub enum BatchError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),

    #[error("the data for key `{0}` is not available")]
    Redaction(String),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },

    #[error("ItemReader from: {0}")]
    ItemReader(String),
}
