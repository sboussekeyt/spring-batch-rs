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

/// A pass-through processor that returns items unchanged.
///
/// This processor implements the identity function for batch processing pipelines.
/// It takes an input item and returns it unchanged, making it useful for scenarios
/// where you need a processor in the pipeline but don't want to transform the data.
///
/// # Type Parameters
///
/// - `T`: The item type that will be passed through unchanged. Must implement `Clone`.
///
/// # Use Cases
///
/// - Testing batch processing pipelines without data transformation
/// - Placeholder processor during development
/// - Pipelines where processing logic is conditional and sometimes bypassed
/// - Maintaining consistent pipeline structure when transformation is optional
///
/// # Performance
///
/// This processor performs a clone operation on each item. For large or complex
/// data structures, consider whether pass-through processing is necessary or if
/// the pipeline can be restructured to avoid unnecessary cloning.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
///
/// let processor = PassThroughProcessor::<String>::new();
/// let input = "Hello, World!".to_string();
/// let result = processor.process(&input).unwrap();
/// assert_eq!(result, input);
/// ```
///
/// Using with different data types:
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
///
/// // With integers
/// let int_processor = PassThroughProcessor::<i32>::new();
/// let number = 42;
/// let result = int_processor.process(&number).unwrap();
/// assert_eq!(result, number);
///
/// // With custom structs
/// #[derive(Clone, PartialEq, Debug)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person_processor = PassThroughProcessor::<Person>::new();
/// let person = Person {
///     name: "Alice".to_string(),
///     age: 30,
/// };
/// let result = person_processor.process(&person).unwrap();
/// assert_eq!(result, person);
/// ```
#[derive(Default)]
pub struct PassThroughProcessor<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone> ItemProcessor<T, T> for PassThroughProcessor<T> {
    /// Processes an item by returning it unchanged.
    ///
    /// # Parameters
    /// - `item`: The item to process (will be cloned and returned unchanged)
    ///
    /// # Returns
    /// - `Ok(cloned_item)` - Always succeeds and returns a clone of the input item
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
    ///
    /// let processor = PassThroughProcessor::<Vec<i32>>::new();
    /// let input = vec![1, 2, 3];
    /// let result = processor.process(&input).unwrap();
    /// assert_eq!(result, input);
    /// ```
    fn process(&self, item: &T) -> ItemProcessorResult<T> {
        Ok(item.clone())
    }
}

impl<T: Clone> PassThroughProcessor<T> {
    /// Creates a new `PassThroughProcessor`.
    ///
    /// # Returns
    /// A new instance of `PassThroughProcessor` that will pass through items of type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::PassThroughProcessor;
    ///
    /// let processor = PassThroughProcessor::<String>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_new_pass_through_processor() {
        let _processor = PassThroughProcessor::<String>::new();
        // Test that we can create the processor without panicking
        // Verify it's a zero-sized type (only contains PhantomData)
        assert_eq!(std::mem::size_of::<PassThroughProcessor<String>>(), 0);
    }

    #[test]
    fn should_create_pass_through_processor_with_default() {
        let _processor = PassThroughProcessor::<i32>::default();
        // Test that we can create the processor using Default trait
        // Verify it's a zero-sized type (only contains PhantomData)
        assert_eq!(std::mem::size_of::<PassThroughProcessor<i32>>(), 0);
    }

    #[test]
    fn should_pass_through_string_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();
        let input = "Hello, World!".to_string();
        let expected = input.clone();

        let result = processor.process(&input)?;

        assert_eq!(result, expected);
        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn should_pass_through_integer_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();
        let input = 42i32;

        let result = processor.process(&input)?;

        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn should_pass_through_vector_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();
        let input = vec![1, 2, 3, 4, 5];
        let expected = input.clone();

        let result = processor.process(&input)?;

        assert_eq!(result, expected);
        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn should_pass_through_custom_struct_unchanged() -> Result<(), BatchError> {
        #[derive(Clone, PartialEq, Debug)]
        struct TestData {
            id: u32,
            name: String,
            values: Vec<f64>,
        }

        let processor = PassThroughProcessor::new();
        let input = TestData {
            id: 123,
            name: "Test Item".to_string(),
            values: vec![1.1, 2.2, 3.3],
        };
        let expected = input.clone();

        let result = processor.process(&input)?;

        assert_eq!(result, expected);
        assert_eq!(result.id, input.id);
        assert_eq!(result.name, input.name);
        assert_eq!(result.values, input.values);
        Ok(())
    }

    #[test]
    fn should_pass_through_option_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();

        // Test with Some value
        let input_some = Some("test".to_string());
        let result_some = processor.process(&input_some)?;
        assert_eq!(result_some, input_some);

        // Test with None value
        let input_none: Option<String> = None;
        let result_none = processor.process(&input_none)?;
        assert_eq!(result_none, input_none);

        Ok(())
    }

    #[test]
    fn should_handle_empty_collections() -> Result<(), BatchError> {
        // Test empty vector
        let vec_processor = PassThroughProcessor::new();
        let empty_vec: Vec<i32> = vec![];
        let result_vec = vec_processor.process(&empty_vec)?;
        assert_eq!(result_vec, empty_vec);
        assert!(result_vec.is_empty());

        // Test empty string
        let string_processor = PassThroughProcessor::new();
        let empty_string = String::new();
        let result_string = string_processor.process(&empty_string)?;
        assert_eq!(result_string, empty_string);
        assert!(result_string.is_empty());

        Ok(())
    }

    #[test]
    fn should_clone_input_not_move() {
        let processor = PassThroughProcessor::new();
        let input = "original".to_string();
        let input_copy = input.clone();

        let _result = processor.process(&input).unwrap();

        // Original input should still be accessible (not moved)
        assert_eq!(input, input_copy);
        assert_eq!(input, "original");
    }

    #[test]
    fn should_work_with_multiple_processors() -> Result<(), BatchError> {
        let processor1 = PassThroughProcessor::<String>::new();
        let processor2 = PassThroughProcessor::<String>::new();

        let input = "test data".to_string();
        let result1 = processor1.process(&input)?;
        let result2 = processor2.process(&result1)?;

        assert_eq!(result2, input);
        assert_eq!(result1, result2);
        Ok(())
    }

    #[test]
    fn should_handle_large_data_structures() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();

        // Create a large vector
        let large_input: Vec<i32> = (0..10000).collect();
        let expected = large_input.clone();

        let result = processor.process(&large_input)?;

        assert_eq!(result.len(), expected.len());
        assert_eq!(result, expected);
        Ok(())
    }
}
