use thiserror::Error;

#[derive(Error, Debug)]
/// Represents the possible errors that can occur during batch processing.
pub enum BatchError {
    #[error("Error occurred in the ItemWriter: {0}")]
    /// Error occurred in the ItemWriter.
    ItemWriter(String),

    #[error("Error occurred in the ItemProcessor: {0}")]
    /// Error occurred in the ItemProcessor.
    ItemProcessor(String),

    #[error("Error occurred in the ItemReader: {0}")]
    /// Error occurred in the ItemReader.
    ItemReader(String),

    #[error("Error occurred in the step: {0}")]
    /// Error occurred in the step.
    Step(String),
}
