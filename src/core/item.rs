use crate::error::BatchError;
use serde::{de::DeserializeOwned, Serialize};

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

impl<R: Serialize, W: DeserializeOwned> ItemProcessor<R, W> for DefaultProcessor {
    /// Processes an item by serializing and deserializing it.
    fn process(&self, item: &R) -> ItemProcessorResult<W> {
        // TODO: For performance reason the best is to return directly the item. R and W are of the same type
        // https://github.com/sboussekeyt/spring-batch-rs/issues/32
        let serialised = serde_json::to_string(&item).unwrap();
        let item = serde_json::from_str(&serialised).unwrap();
        Ok(item)
    }
}
