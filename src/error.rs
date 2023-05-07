use thiserror::Error;

#[derive(Error, Debug)]
/// Batch error
pub enum BatchError {
    #[error("ItemWriter from: {0}")]
    ItemWriter(String),

    #[error("ItemReader from: {0}")]
    ItemReader(String),
}
