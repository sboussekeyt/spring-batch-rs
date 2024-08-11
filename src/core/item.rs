use std::any::Any;

use crate::error::BatchError;

/// Represents the result of reading an item from the reader.
pub type ItemReaderResult<R> = Result<Option<R>, BatchError>;

/// Represents the result of processing an item by the processor.
pub type ItemProcessorResult<W> = Result<W, BatchError>;

/// Represents the result of writing items by the writer.
pub type ItemWriterResult = Result<(), BatchError>;

/// A trait for reading items.
pub trait ItemReader<R> {
    /// Reads an item from the reader.
    fn read(&self) -> ItemReaderResult<R>;
}

/// A trait for processing items.
pub trait ItemProcessor<R, W> {
    /// Processes an item and returns the processed result.
    fn process(&self, item: &R) -> ItemProcessorResult<W>;
}

/// A trait for writing items.
pub trait ItemWriter<W> {
    /// Writes the given items.
    fn write(&self, items: &[W]) -> ItemWriterResult;

    /// Flushes any buffered data.
    fn flush(&self) -> ItemWriterResult {
        Ok(())
    }

    /// Opens the writer.
    fn open(&self) -> ItemWriterResult {
        Ok(())
    }

    /// Closes the writer.
    fn close(&self) -> ItemWriterResult {
        Ok(())
    }
}

/// A default implementation of the `ItemProcessor` trait.
#[derive(Default)]
pub struct DefaultProcessor;

impl<R: Any, W: Clone + Any> ItemProcessor<R, W> for DefaultProcessor {
    fn process(&self, item: &R) -> ItemProcessorResult<W> {
        let value_any = item as &dyn Any;

        match value_any.downcast_ref::<W>() {
            Some(as_w) => Ok(as_w.clone()),
            None => Err(BatchError::ItemProcessor("Cannot downcast".to_string())),
        }
    }
}
