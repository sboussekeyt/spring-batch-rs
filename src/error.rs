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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_writer_error() {
        let error = BatchError::ItemWriter("Failed to write item".to_string());
        assert_eq!(
            error.to_string(),
            "Error occurred in the ItemWriter: Failed to write item"
        );
    }

    #[test]
    fn test_item_processor_error() {
        let error = BatchError::ItemProcessor("Failed to process item".to_string());
        assert_eq!(
            error.to_string(),
            "Error occurred in the ItemProcessor: Failed to process item"
        );
    }

    #[test]
    fn test_item_reader_error() {
        let error = BatchError::ItemReader("Failed to read item".to_string());
        assert_eq!(
            error.to_string(),
            "Error occurred in the ItemReader: Failed to read item"
        );
    }

    #[test]
    fn test_step_error() {
        let error = BatchError::Step("Step execution failed".to_string());
        assert_eq!(
            error.to_string(),
            "Error occurred in the step: Step execution failed"
        );
    }
}
