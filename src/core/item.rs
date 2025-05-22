use std::any::Any;

use crate::error::BatchError;

/// Represents the result of reading an item from the reader.
///
/// This type is a specialized `Result` that can be:
/// - `Ok(Some(R))` when an item is successfully read
/// - `Ok(None)` when there are no more items to read (end of data)
/// - `Err(BatchError)` when an error occurs during reading
pub type ItemReaderResult<I> = Result<Option<I>, BatchError>;

/// Represents the result of processing an item by the processor.
///
/// This type is a specialized `Result` that can be:
/// - `Ok(W)` when an item is successfully processed
/// - `Err(BatchError)` when an error occurs during processing
pub type ItemProcessorResult<O> = Result<O, BatchError>;

/// Represents the result of writing items by the writer.
///
/// This type is a specialized `Result` that can be:
/// - `Ok(())` when items are successfully written
/// - `Err(BatchError)` when an error occurs during writing
pub type ItemWriterResult = Result<(), BatchError>;

/// A trait for reading items.
///
/// This trait defines the contract for components that read items from a data source.
/// It is one of the fundamental building blocks of the batch processing pipeline.
///
/// # Design Pattern
///
/// This follows the Strategy Pattern, allowing different reading strategies to be
/// interchangeable while maintaining a consistent interface.
///
/// # Implementation Note
///
/// Implementors of this trait should:
/// - Return `Ok(Some(item))` when an item is successfully read
/// - Return `Ok(None)` when there are no more items to read (end of data)
/// - Return `Err(BatchError)` when an error occurs during reading
///
/// # Example
///
/// ```compile_fail
/// use spring_batch_rs::core::item::{ItemReader, ItemReaderResult};
/// use spring_batch_rs::error::BatchError;
///
/// struct StringReader {
///     items: Vec<String>,
///     position: usize,
/// }
///
/// impl ItemReader<String> for StringReader {
///     fn read(&mut self) -> ItemReaderResult<String> {
///         if self.position < self.items.len() {
///             let item = self.items[self.position].clone();
///             self.position += 1;
///             Ok(Some(item))
///         } else {
///             Ok(None) // End of data
///         }
///     }
/// }
/// ```
pub trait ItemReader<I> {
    /// Reads an item from the reader.
    ///
    /// # Returns
    /// - `Ok(Some(item))` when an item is successfully read
    /// - `Ok(None)` when there are no more items to read (end of data)
    /// - `Err(BatchError)` when an error occurs during reading
    fn read(&self) -> ItemReaderResult<I>;
}

/// A trait for processing items.
///
/// This trait defines the contract for components that transform or process items
/// in a batch processing pipeline. It takes an input item of type `R` and produces
/// an output item of type `W`.
///
/// # Design Pattern
///
/// This follows the Strategy Pattern, allowing different processing strategies to be
/// interchangeable while maintaining a consistent interface.
///
/// # Type Parameters
///
/// - `R`: The input item type
/// - `W`: The output item type
///
/// # Example
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, ItemProcessorResult};
/// use spring_batch_rs::error::BatchError;
///
/// struct UppercaseProcessor;
///
/// impl ItemProcessor<String, String> for UppercaseProcessor {
///     fn process(&self, item: &String) -> ItemProcessorResult<String> {
///         Ok(item.to_uppercase())
///     }
/// }
/// ```
pub trait ItemProcessor<I, O> {
    /// Processes an item and returns the processed result.
    ///
    /// # Parameters
    /// - `item`: The item to process
    ///
    /// # Returns
    /// - `Ok(processed_item)` when the item is successfully processed
    /// - `Err(BatchError)` when an error occurs during processing
    fn process(&self, item: &I) -> ItemProcessorResult<O>;
}

/// A trait for writing items.
///
/// This trait defines the contract for components that write items to a data destination.
/// It is one of the fundamental building blocks of the batch processing pipeline.
///
/// # Design Pattern
///
/// This follows the Strategy Pattern, allowing different writing strategies to be
/// interchangeable while maintaining a consistent interface.
///
/// # Lifecycle Methods
///
/// This trait includes additional lifecycle methods:
/// - `flush()`: Flushes any buffered data
/// - `open()`: Initializes resources before writing starts
/// - `close()`: Releases resources after writing completes
///
/// # Example
///
/// ```
/// use spring_batch_rs::core::item::{ItemWriter, ItemWriterResult};
/// use spring_batch_rs::error::BatchError;
///
/// struct ConsoleWriter;
///
/// impl ItemWriter<String> for ConsoleWriter {
///     fn write(&self, items: &[String]) -> ItemWriterResult {
///         for item in items {
///             println!("{}", item);
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait ItemWriter<O> {
    /// Writes the given items.
    ///
    /// # Parameters
    /// - `items`: A slice of items to write
    ///
    /// # Returns
    /// - `Ok(())` when items are successfully written
    /// - `Err(BatchError)` when an error occurs during writing
    fn write(&self, items: &[O]) -> ItemWriterResult;

    /// Flushes any buffered data.
    ///
    /// This method is called after a chunk of items has been written, and
    /// allows the writer to flush any internally buffered data to the destination.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    ///
    /// # Returns
    /// - `Ok(())` when the flush operation succeeds
    /// - `Err(BatchError)` when an error occurs during flushing
    fn flush(&self) -> ItemWriterResult {
        Ok(())
    }

    /// Opens the writer.
    ///
    /// This method is called before any items are written, and allows the writer
    /// to initialize any resources it needs.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    ///
    /// # Returns
    /// - `Ok(())` when the open operation succeeds
    /// - `Err(BatchError)` when an error occurs during opening
    fn open(&self) -> ItemWriterResult {
        Ok(())
    }

    /// Closes the writer.
    ///
    /// This method is called after all items have been written, and allows the writer
    /// to release any resources it acquired.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    ///
    /// # Returns
    /// - `Ok(())` when the close operation succeeds
    /// - `Err(BatchError)` when an error occurs during closing
    fn close(&self) -> ItemWriterResult {
        Ok(())
    }
}

/// A default implementation of the `ItemProcessor` trait.
///
/// This processor simply passes items through without modifying them, but handles
/// type conversion if the input and output types are compatible.
///
/// # Type Parameters
///
/// - `R`: The input item type
/// - `W`: The output item type, which must be clonable and downcastable from `R`
///
/// # Behavior
///
/// - If the input type `R` can be downcast to the output type `W`, the item is cloned and returned
/// - If the downcast fails, a `BatchError` is returned
///
/// # Use Cases
///
/// This processor is useful when:
/// - No processing is needed, just pass-through
/// - A trivial processor is needed as a placeholder
/// - Types are compatible but formally different
#[derive(Default)]
pub struct DefaultProcessor;

impl<I: Any, O: Clone + Any> ItemProcessor<I, O> for DefaultProcessor {
    /// Processes an item by attempting to downcast it to the target type.
    ///
    /// # Parameters
    /// - `item`: The item to process
    ///
    /// # Returns
    /// - `Ok(item_as_w)` when the item can be downcast to type `W`
    /// - `Err(BatchError)` when the item cannot be downcast to type `W`
    fn process(&self, item: &I) -> ItemProcessorResult<O> {
        // Treat the item as Any to enable downcasting
        let value_any = item as &dyn Any;

        // Try to downcast to the target type
        match value_any.downcast_ref::<O>() {
            Some(as_w) => Ok(as_w.clone()),
            None => Err(BatchError::ItemProcessor("Cannot downcast".to_string())),
        }
    }
}
