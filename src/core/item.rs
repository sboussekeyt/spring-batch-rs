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
/// - `Ok(Some(O))` when an item is successfully processed and should be passed to the writer
/// - `Ok(None)` when an item is intentionally filtered out (not an error)
/// - `Err(BatchError)` when an error occurs during processing
pub type ItemProcessorResult<O> = Result<Option<O>, BatchError>;

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
/// in a batch processing pipeline. It takes an input item of type `I` and produces
/// an output item of type `O`.
///
/// # Filtering
///
/// Returning `Ok(None)` filters the item silently: it is not passed to the writer
/// and is counted in [`crate::core::step::StepExecution::filter_count`]. This is different from returning
/// `Err(BatchError)` which counts as a processing error and may trigger fault tolerance.
///
/// # Design Pattern
///
/// This follows the Strategy Pattern, allowing different processing strategies to be
/// interchangeable while maintaining a consistent interface.
///
/// # Type Parameters
///
/// - `I`: The input item type
/// - `O`: The output item type
///
/// # Example
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, ItemProcessorResult};
/// use spring_batch_rs::error::BatchError;
///
/// struct AdultFilter;
///
/// #[derive(Clone)]
/// struct Person { name: String, age: u32 }
///
/// impl ItemProcessor<Person, Person> for AdultFilter {
///     fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
///         if item.age >= 18 {
///             Ok(Some(item.clone())) // keep adults
///         } else {
///             Ok(None) // filter out minors
///         }
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
    /// - `Ok(Some(processed_item))` when the item is successfully processed
    /// - `Ok(None)` when the item is intentionally filtered out
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
/// assert_eq!(result, Some(input));
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
/// assert_eq!(result, Some(number));
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
/// assert_eq!(result, Some(person));
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
    /// - `Ok(Some(cloned_item))` - Always succeeds and returns a clone of the input item
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
    ///
    /// let processor = PassThroughProcessor::<Vec<i32>>::new();
    /// let input = vec![1, 2, 3];
    /// let result = processor.process(&input).unwrap();
    /// assert_eq!(result, Some(input));
    /// ```
    fn process(&self, item: &T) -> ItemProcessorResult<T> {
        Ok(Some(item.clone()))
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

/// A composite processor that chains two processors sequentially using static dispatch.
///
/// The output of the first processor becomes the input of the second.
/// If the first processor filters an item (returns `Ok(None)`), the chain
/// stops immediately and `Ok(None)` is returned — the second processor is
/// never called.
///
/// Both processors are stored by value — no heap allocation occurs inside the
/// struct itself. This mirrors the pattern used by standard library iterator
/// adapters such as [`std::iter::Chain`].
///
/// Construct chains using [`CompositeItemProcessorBuilder`] rather than
/// instantiating this struct directly.
///
/// # Type Parameters
///
/// - `P1`: The first processor type. Must implement `ItemProcessor<I, M>` for
///   some input type `I` and intermediate type `M`.
/// - `P2`: The second processor type. Must implement `ItemProcessor<M, O>` where
///   `M` is the output type of `P1` and `O` is the final output type.
/// - `M`: The intermediate type — output of `P1`, input of `P2`. Tracked via
///   `PhantomData` so it participates in type inference without being stored.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
/// use spring_batch_rs::BatchError;
///
/// struct DoubleProcessor;
/// impl ItemProcessor<i32, i32> for DoubleProcessor {
///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
///         Ok(Some(item * 2))
///     }
/// }
///
/// struct ToStringProcessor;
/// impl ItemProcessor<i32, String> for ToStringProcessor {
///     fn process(&self, item: &i32) -> Result<Option<String>, BatchError> {
///         Ok(Some(item.to_string()))
///     }
/// }
///
/// let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
///     .link(ToStringProcessor)
///     .build();
///
/// // 21 * 2 = 42, then converted to "42"
/// assert_eq!(composite.process(&21).unwrap(), Some("42".to_string()));
/// ```
///
/// # Errors
///
/// Returns [`BatchError`] if any processor in the chain returns an error.
pub struct CompositeItemProcessor<P1, P2, M> {
    first: P1,
    second: P2,
    /// Tracks the intermediate type `M` (output of `P1`, input of `P2`).
    /// Uses `fn(M) -> M` to keep the type parameter invariant and avoid
    /// unintended variance.
    _marker: std::marker::PhantomData<fn(M) -> M>,
}

impl<I, M, O, P1, P2> ItemProcessor<I, O> for CompositeItemProcessor<P1, P2, M>
where
    P1: ItemProcessor<I, M>,
    P2: ItemProcessor<M, O>,
{
    /// Applies the first processor, then — if the result is `Some` — applies
    /// the second. Returns `Ok(None)` immediately if the first processor
    /// filters the item.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] if either processor fails.
    fn process(&self, item: &I) -> ItemProcessorResult<O> {
        match self.first.process(item)? {
            Some(intermediate) => self.second.process(&intermediate),
            None => Ok(None),
        }
    }
}

/// Builder for creating a chain of [`ItemProcessor`]s using static dispatch.
///
/// Start the chain with [`new`](CompositeItemProcessorBuilder::new), append
/// processors with [`link`](CompositeItemProcessorBuilder::link), and finalise
/// with [`build`](CompositeItemProcessorBuilder::build). Each call to `link`
/// wraps the accumulated chain in a [`CompositeItemProcessor`], changing the
/// output type. Mismatched types are caught at compile time.
///
/// The built chain stores all processors by value — no heap allocations occur
/// inside the processor itself. The type of the built value encodes the full
/// chain structure (e.g. `CompositeItemProcessor<P1, CompositeItemProcessor<P2, P3>>`),
/// similar to how `Iterator` adapters compose in the standard library.
///
/// # Type Parameters
///
/// - `P`: The accumulated processor type. Starts as the first processor and
///   is wrapped in [`CompositeItemProcessor`] with each [`link`](CompositeItemProcessorBuilder::link) call.
///
/// # Examples
///
/// Two processors (`i32 → i32 → String`):
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
/// use spring_batch_rs::BatchError;
///
/// struct DoubleProcessor;
/// impl ItemProcessor<i32, i32> for DoubleProcessor {
///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
///         Ok(Some(item * 2))
///     }
/// }
///
/// struct ToStringProcessor;
/// impl ItemProcessor<i32, String> for ToStringProcessor {
///     fn process(&self, item: &i32) -> Result<Option<String>, BatchError> {
///         Ok(Some(item.to_string()))
///     }
/// }
///
/// let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
///     .link(ToStringProcessor)
///     .build();
///
/// assert_eq!(composite.process(&21).unwrap(), Some("42".to_string()));
/// ```
///
/// Three processors (`i32 → i32 → i32 → String`):
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
/// use spring_batch_rs::BatchError;
///
/// struct AddOneProcessor;
/// impl ItemProcessor<i32, i32> for AddOneProcessor {
///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
///         Ok(Some(item + 1))
///     }
/// }
///
/// struct DoubleProcessor;
/// impl ItemProcessor<i32, i32> for DoubleProcessor {
///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
///         Ok(Some(item * 2))
///     }
/// }
///
/// struct ToStringProcessor;
/// impl ItemProcessor<i32, String> for ToStringProcessor {
///     fn process(&self, item: &i32) -> Result<Option<String>, BatchError> {
///         Ok(Some(item.to_string()))
///     }
/// }
///
/// let composite = CompositeItemProcessorBuilder::new(AddOneProcessor)
///     .link(DoubleProcessor)
///     .link(ToStringProcessor)
///     .build();
///
/// // (4 + 1) * 2 = 10 → "10"
/// assert_eq!(composite.process(&4).unwrap(), Some("10".to_string()));
/// ```
pub struct CompositeItemProcessorBuilder<P> {
    processor: P,
}

impl<P> CompositeItemProcessorBuilder<P> {
    /// Creates a new builder with the given processor as the first in the chain.
    ///
    /// # Parameters
    ///
    /// - `first`: The first processor in the chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
    /// use spring_batch_rs::BatchError;
    ///
    /// struct UppercaseProcessor;
    /// impl ItemProcessor<String, String> for UppercaseProcessor {
    ///     fn process(&self, item: &String) -> Result<Option<String>, BatchError> {
    ///         Ok(Some(item.to_uppercase()))
    ///     }
    /// }
    ///
    /// let builder = CompositeItemProcessorBuilder::new(UppercaseProcessor);
    /// let composite = builder.build();
    /// assert_eq!(composite.process(&"hello".to_string()).unwrap(), Some("HELLO".to_string()));
    /// ```
    pub fn new(first: P) -> Self {
        Self { processor: first }
    }

    /// Appends a processor to the end of the chain.
    ///
    /// Returns a new builder whose accumulated type is
    /// `CompositeItemProcessor<P, P2>`. The input/output types are verified
    /// at compile time when the chain is used.
    ///
    /// # Type Parameters
    ///
    /// - `P2`: The processor type to append.
    /// - `M`: The intermediate type connecting `P` and `P2`. Inferred by the
    ///   compiler from the `ItemProcessor` impls on `P` and `P2`.
    ///
    /// # Parameters
    ///
    /// - `next`: The processor to append to the chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
    /// use spring_batch_rs::BatchError;
    ///
    /// struct AddOneProcessor;
    /// impl ItemProcessor<i32, i32> for AddOneProcessor {
    ///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
    ///         Ok(Some(item + 1))
    ///     }
    /// }
    ///
    /// struct ToStringProcessor;
    /// impl ItemProcessor<i32, String> for ToStringProcessor {
    ///     fn process(&self, item: &i32) -> Result<Option<String>, BatchError> {
    ///         Ok(Some(item.to_string()))
    ///     }
    /// }
    ///
    /// let composite = CompositeItemProcessorBuilder::new(AddOneProcessor)
    ///     .link(ToStringProcessor)
    ///     .build();
    ///
    /// assert_eq!(composite.process(&41).unwrap(), Some("42".to_string()));
    /// ```
    pub fn link<P2, M>(
        self,
        next: P2,
    ) -> CompositeItemProcessorBuilder<CompositeItemProcessor<P, P2, M>> {
        CompositeItemProcessorBuilder {
            processor: CompositeItemProcessor {
                first: self.processor,
                second: next,
                _marker: std::marker::PhantomData,
            },
        }
    }

    /// Builds and returns the composite processor.
    ///
    /// Returns the accumulated processor value `P`. When chained via `link`,
    /// `P` will be a nested `CompositeItemProcessor` such as
    /// `CompositeItemProcessor<P1, CompositeItemProcessor<P2, P3>>`.
    ///
    /// Pass `&composite` to the step builder's `.processor()` method — Rust
    /// will coerce it to `&dyn ItemProcessor<I, O>` automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::core::item::{ItemProcessor, CompositeItemProcessorBuilder};
    /// use spring_batch_rs::BatchError;
    ///
    /// struct DoubleProcessor;
    /// impl ItemProcessor<i32, i32> for DoubleProcessor {
    ///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
    ///         Ok(Some(item * 2))
    ///     }
    /// }
    ///
    /// struct AddTenProcessor;
    /// impl ItemProcessor<i32, i32> for AddTenProcessor {
    ///     fn process(&self, item: &i32) -> Result<Option<i32>, BatchError> {
    ///         Ok(Some(item + 10))
    ///     }
    /// }
    ///
    /// let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
    ///     .link(AddTenProcessor)
    ///     .build();
    ///
    /// // 5 * 2 = 10, then 10 + 10 = 20
    /// assert_eq!(composite.process(&5).unwrap(), Some(20));
    /// ```
    pub fn build(self) -> P {
        self.processor
    }
}

/// A composite writer that fans out the same chunk to two writers sequentially using static dispatch.
///
/// Both writers receive identical slices on every `write` call. All four lifecycle
/// methods (`write`, `flush`, `open`, `close`) are forwarded to `first` then `second`,
/// short-circuiting on the first `Err`. If `open()` on `first` fails, `second.open()`
/// is never called — lifecycle management is the step's responsibility.
///
/// Both writers are stored by value — no heap allocation occurs inside the struct.
/// The type encodes the full chain:
/// `CompositeItemWriter<CompositeItemWriter<W1, W2>, W3>` for three writers.
///
/// # Type Parameters
///
/// - `W1`: The first writer type. Must implement `ItemWriter<T>`.
/// - `W2`: The second writer type. Must implement `ItemWriter<T>`.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriter};
///
/// struct CountingWriter { count: std::cell::Cell<usize> }
/// impl CountingWriter { fn new() -> Self { Self { count: std::cell::Cell::new(0) } } }
/// impl ItemWriter<i32> for CountingWriter {
///     fn write(&self, items: &[i32]) -> Result<(), spring_batch_rs::BatchError> {
///         self.count.set(self.count.get() + items.len());
///         Ok(())
///     }
/// }
///
/// let composite = CompositeItemWriter {
///     first: CountingWriter::new(),
///     second: CountingWriter::new(),
/// };
/// composite.write(&[1, 2, 3]).unwrap();
/// assert_eq!(composite.first.count.get(), 3);
/// assert_eq!(composite.second.count.get(), 3);
/// ```
///
/// # Errors
///
/// Returns [`BatchError`] if any writer in the chain returns an error.
pub struct CompositeItemWriter<W1, W2> {
    /// The first writer in the fan-out chain.
    pub first: W1,
    /// The second writer in the fan-out chain.
    pub second: W2,
}

impl<T, W1, W2> ItemWriter<T> for CompositeItemWriter<W1, W2>
where
    W1: ItemWriter<T>,
    W2: ItemWriter<T>,
{
    /// Writes `items` to `first`, then to `second`. Short-circuits on the first error.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemWriter`] if either writer fails.
    fn write(&self, items: &[T]) -> ItemWriterResult {
        self.first.write(items)?;
        self.second.write(items)
    }

    /// Flushes `first`, then `second`. Short-circuits on the first error.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemWriter`] if either flush fails.
    fn flush(&self) -> ItemWriterResult {
        self.first.flush()?;
        self.second.flush()
    }

    /// Opens `first`, then `second`. Short-circuits on the first error.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemWriter`] if either open fails.
    fn open(&self) -> ItemWriterResult {
        self.first.open()?;
        self.second.open()
    }

    /// Closes `first`, then `second`. Short-circuits on the first error.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::ItemWriter`] if either close fails.
    fn close(&self) -> ItemWriterResult {
        self.first.close()?;
        self.second.close()
    }
}

/// Allows any `Box<P>` where `P: ItemProcessor<I, O>` to be used wherever
/// `&dyn ItemProcessor<I, O>` is expected — including boxed concrete types
/// (`Box<MyProcessor>`) and boxed trait objects (`Box<dyn ItemProcessor<I, O>>`).
///
/// The `?Sized` bound is what makes this cover trait objects: `dyn Trait` is
/// unsized, so without `?Sized` the impl would not apply to them.
impl<I, O, P: ItemProcessor<I, O> + ?Sized> ItemProcessor<I, O> for Box<P> {
    fn process(&self, item: &I) -> ItemProcessorResult<O> {
        (**self).process(item)
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

        assert_eq!(result, Some(expected));
        Ok(())
    }

    #[test]
    fn should_pass_through_integer_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();
        let input = 42i32;

        let result = processor.process(&input)?;

        assert_eq!(result, Some(input));
        Ok(())
    }

    #[test]
    fn should_pass_through_vector_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();
        let input = vec![1, 2, 3, 4, 5];
        let expected = input.clone();

        let result = processor.process(&input)?;

        assert_eq!(result, Some(expected));
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

        assert_eq!(result, Some(expected));
        Ok(())
    }

    #[test]
    fn should_pass_through_option_unchanged() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();

        // Test with Some value
        let input_some = Some("test".to_string());
        let result_some = processor.process(&input_some)?;
        assert_eq!(result_some, Some(input_some));

        // Test with None value
        let input_none: Option<String> = None;
        let result_none = processor.process(&input_none)?;
        assert_eq!(result_none, Some(input_none));

        Ok(())
    }

    #[test]
    fn should_handle_empty_collections() -> Result<(), BatchError> {
        let vec_processor = PassThroughProcessor::new();
        let empty_vec: Vec<i32> = vec![];
        let result_vec = vec_processor.process(&empty_vec)?;
        assert_eq!(result_vec, Some(empty_vec));

        let string_processor = PassThroughProcessor::new();
        let empty_string = String::new();
        let result_string = string_processor.process(&empty_string)?;
        assert_eq!(result_string, Some(empty_string));

        Ok(())
    }

    #[test]
    fn should_clone_input_not_move() {
        let processor = PassThroughProcessor::new();
        let input = "original".to_string();
        let input_copy = input.clone();

        let _result = processor.process(&input).unwrap();

        assert_eq!(input, input_copy);
        assert_eq!(input, "original");
    }

    #[test]
    fn should_work_with_multiple_processors() -> Result<(), BatchError> {
        let processor1 = PassThroughProcessor::<String>::new();
        let processor2 = PassThroughProcessor::<String>::new();

        let input = "test data".to_string();
        let result1 = processor1.process(&input)?;
        let inner = result1.unwrap();
        let result2 = processor2.process(&inner)?;

        assert_eq!(result2, Some(input));
        Ok(())
    }

    #[test]
    fn should_handle_large_data_structures() -> Result<(), BatchError> {
        let processor = PassThroughProcessor::new();

        let large_input: Vec<i32> = (0..10000).collect();
        let expected_len = large_input.len();

        let result = processor.process(&large_input)?;

        // PassThroughProcessor always returns Some — unwrap is safe
        assert_eq!(result.unwrap().len(), expected_len);
        Ok(())
    }

    #[test]
    fn should_use_default_flush_open_close_implementations() {
        struct MinimalWriter;
        impl ItemWriter<String> for MinimalWriter {
            fn write(&self, _: &[String]) -> ItemWriterResult {
                Ok(())
            }
            // flush, open, close use the trait's default implementations
        }
        let w = MinimalWriter;
        assert!(w.flush().is_ok(), "default flush should return Ok");
        assert!(w.open().is_ok(), "default open should return Ok");
        assert!(w.close().is_ok(), "default close should return Ok");
    }

    // --- CompositeItemProcessor / CompositeItemProcessorBuilder ---

    struct DoubleProcessor;
    impl ItemProcessor<i32, i32> for DoubleProcessor {
        fn process(&self, item: &i32) -> ItemProcessorResult<i32> {
            Ok(Some(item * 2))
        }
    }

    struct AddTenProcessor;
    impl ItemProcessor<i32, i32> for AddTenProcessor {
        fn process(&self, item: &i32) -> ItemProcessorResult<i32> {
            Ok(Some(item + 10))
        }
    }

    struct ToStringProcessor;
    impl ItemProcessor<i32, String> for ToStringProcessor {
        fn process(&self, item: &i32) -> ItemProcessorResult<String> {
            Ok(Some(item.to_string()))
        }
    }

    struct FilterEvenProcessor;
    impl ItemProcessor<i32, i32> for FilterEvenProcessor {
        fn process(&self, item: &i32) -> ItemProcessorResult<i32> {
            if item % 2 == 0 {
                Ok(Some(*item))
            } else {
                Ok(None) // filter odd numbers
            }
        }
    }

    struct FailingProcessor;
    impl ItemProcessor<i32, i32> for FailingProcessor {
        fn process(&self, _item: &i32) -> ItemProcessorResult<i32> {
            Err(BatchError::ItemProcessor("forced failure".to_string()))
        }
    }

    #[test]
    fn should_chain_two_same_type_processors() -> Result<(), BatchError> {
        let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
            .link(AddTenProcessor)
            .build();

        // 5 * 2 = 10, then 10 + 10 = 20
        assert_eq!(
            composite.process(&5)?,
            Some(20),
            "5 * 2 + 10 should equal 20"
        );
        Ok(())
    }

    #[test]
    fn should_chain_two_type_changing_processors() -> Result<(), BatchError> {
        let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
            .link(ToStringProcessor)
            .build();

        // 21 * 2 = 42, then "42"
        assert_eq!(composite.process(&21)?, Some("42".to_string()));
        Ok(())
    }

    #[test]
    fn should_chain_three_processors() -> Result<(), BatchError> {
        let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
            .link(AddTenProcessor)
            .link(ToStringProcessor)
            .build();

        // 5 * 2 = 10, then 10 + 10 = 20, then "20"
        assert_eq!(composite.process(&5)?, Some("20".to_string()));
        Ok(())
    }

    #[test]
    fn should_stop_chain_when_first_processor_filters_item() -> Result<(), BatchError> {
        let composite = CompositeItemProcessorBuilder::new(FilterEvenProcessor)
            .link(ToStringProcessor)
            .build();

        // 3 is odd → filtered by first processor → second processor never called
        assert_eq!(
            composite.process(&3)?,
            None,
            "odd number should be filtered"
        );
        // 4 is even → passes through → converted to string
        assert_eq!(
            composite.process(&4)?,
            Some("4".to_string()),
            "even number should pass"
        );
        Ok(())
    }

    #[test]
    fn should_propagate_error_from_first_processor() {
        let composite = CompositeItemProcessorBuilder::new(FailingProcessor)
            .link(ToStringProcessor)
            .build();

        let result = composite.process(&1);
        assert!(
            result.is_err(),
            "error from first processor should propagate"
        );
    }

    #[test]
    fn should_propagate_error_from_second_processor() {
        struct AlwaysFailI32;
        impl ItemProcessor<i32, i32> for AlwaysFailI32 {
            fn process(&self, _: &i32) -> ItemProcessorResult<i32> {
                Err(BatchError::ItemProcessor("second failed".to_string()))
            }
        }

        let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
            .link(AlwaysFailI32)
            .build();

        let result = composite.process(&5);
        assert!(
            result.is_err(),
            "error from second processor should propagate"
        );
    }

    #[test]
    fn should_use_box_blanket_impl_as_item_processor() -> Result<(), BatchError> {
        // build() returns the concrete type; Box::new() it to get a trait object.
        // Box<dyn ItemProcessor<I, O>> implements ItemProcessor<I, O> via the ?Sized blanket impl.
        let composite = CompositeItemProcessorBuilder::new(DoubleProcessor)
            .link(ToStringProcessor)
            .build();
        let boxed: Box<dyn ItemProcessor<i32, String>> = Box::new(composite);

        let result = boxed.process(&3)?;
        assert_eq!(
            result,
            Some("6".to_string()),
            "boxed trait object should delegate to inner processor"
        );
        Ok(())
    }

    #[test]
    fn should_use_box_concrete_type_as_item_processor() -> Result<(), BatchError> {
        // Box<ConcreteProcessor> also implements ItemProcessor<I, O> via the ?Sized blanket impl
        let boxed: Box<DoubleProcessor> = Box::new(DoubleProcessor);

        let result = boxed.process(&7)?;
        assert_eq!(
            result,
            Some(14),
            "boxed concrete processor should delegate to inner processor"
        );
        Ok(())
    }

    // --- CompositeItemWriter ---

    use std::cell::Cell;

    struct RecordingWriter {
        write_calls: Cell<usize>,
        items_written: Cell<usize>,
        open_calls: Cell<usize>,
        close_calls: Cell<usize>,
        flush_calls: Cell<usize>,
        fail_write: bool,
        fail_open: bool,
    }

    impl RecordingWriter {
        fn new() -> Self {
            Self {
                write_calls: Cell::new(0),
                items_written: Cell::new(0),
                open_calls: Cell::new(0),
                close_calls: Cell::new(0),
                flush_calls: Cell::new(0),
                fail_write: false,
                fail_open: false,
            }
        }
        fn failing_write() -> Self {
            Self { fail_write: true, ..Self::new() }
        }
        fn failing_open() -> Self {
            Self { fail_open: true, ..Self::new() }
        }
    }

    impl ItemWriter<i32> for RecordingWriter {
        fn write(&self, items: &[i32]) -> ItemWriterResult {
            if self.fail_write {
                return Err(BatchError::ItemWriter("forced write failure".to_string()));
            }
            self.write_calls.set(self.write_calls.get() + 1);
            self.items_written.set(self.items_written.get() + items.len());
            Ok(())
        }
        fn open(&self) -> ItemWriterResult {
            if self.fail_open {
                return Err(BatchError::ItemWriter("forced open failure".to_string()));
            }
            self.open_calls.set(self.open_calls.get() + 1);
            Ok(())
        }
        fn close(&self) -> ItemWriterResult {
            self.close_calls.set(self.close_calls.get() + 1);
            Ok(())
        }
        fn flush(&self) -> ItemWriterResult {
            self.flush_calls.set(self.flush_calls.get() + 1);
            Ok(())
        }
    }

    #[test]
    fn should_write_to_both_writers() -> Result<(), BatchError> {
        let w1 = RecordingWriter::new();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        composite.write(&[1, 2, 3])?;
        assert_eq!(composite.first.write_calls.get(), 1, "first writer should be called");
        assert_eq!(composite.first.items_written.get(), 3, "first writer should receive 3 items");
        assert_eq!(composite.second.write_calls.get(), 1, "second writer should be called");
        assert_eq!(composite.second.items_written.get(), 3, "second writer should receive 3 items");
        Ok(())
    }

    #[test]
    fn should_open_both_writers_in_order() -> Result<(), BatchError> {
        let w1 = RecordingWriter::new();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        composite.open()?;
        assert_eq!(composite.first.open_calls.get(), 1, "first writer should be opened");
        assert_eq!(composite.second.open_calls.get(), 1, "second writer should be opened");
        Ok(())
    }

    #[test]
    fn should_close_both_writers_in_order() -> Result<(), BatchError> {
        let w1 = RecordingWriter::new();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        composite.close()?;
        assert_eq!(composite.first.close_calls.get(), 1, "first writer should be closed");
        assert_eq!(composite.second.close_calls.get(), 1, "second writer should be closed");
        Ok(())
    }

    #[test]
    fn should_flush_both_writers() -> Result<(), BatchError> {
        let w1 = RecordingWriter::new();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        composite.flush()?;
        assert_eq!(composite.first.flush_calls.get(), 1, "first writer should be flushed");
        assert_eq!(composite.second.flush_calls.get(), 1, "second writer should be flushed");
        Ok(())
    }

    #[test]
    fn should_short_circuit_on_write_error() {
        let w1 = RecordingWriter::failing_write();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        let result = composite.write(&[1, 2, 3]);
        assert!(result.is_err(), "error should propagate");
        assert_eq!(composite.second.write_calls.get(), 0, "second writer should not be called after first fails");
    }

    #[test]
    fn should_short_circuit_on_open_error() {
        let w1 = RecordingWriter::failing_open();
        let w2 = RecordingWriter::new();
        let composite = CompositeItemWriter { first: w1, second: w2 };
        let result = composite.open();
        assert!(result.is_err(), "error should propagate");
        assert_eq!(composite.second.open_calls.get(), 0, "second writer should not be opened after first fails");
    }
}
