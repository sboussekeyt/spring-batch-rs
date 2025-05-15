use crate::BatchError;
use log::{debug, info, warn};
use std::{
    cell::Cell,
    time::{Duration, Instant},
};
use uuid::Uuid;

use super::{
    build_name,
    item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter},
};

/// Result type for step execution.
///
/// This represents the outcome of a step execution:
/// - `Ok(StepExecution)`: The step completed successfully
/// - `Err(StepExecution)`: The step failed, but we still have execution details
type StepResult<T> = Result<T, T>;

/// Result type for chunk processing.
///
/// This represents the outcome of processing a chunk of items:
/// - `Ok(T)`: The chunk was processed successfully
/// - `Err(BatchError)`: An error occurred during chunk processing
type ChunkResult<T> = Result<T, BatchError>;

/// Defines the contract for a step in a batch process.
///
/// A step is a self-contained unit of work in a batch job. Steps are executed
/// in sequence within a job, and each step has its own lifecycle and status.
///
/// # Design Pattern
///
/// This trait follows the Command Pattern, representing a discrete operation
/// that can be executed and track its own execution details.
///
/// # Core Operations
///
/// - Execution: Runs the step and returns the execution result
/// - Status tracking: Reports on the current state of the step
/// - Metrics: Provides counts of items read, written, and errors encountered
///
/// # Implementation Note
///
/// Implementing this trait directly is complex. Most users should use the
/// `StepBuilder` to create step instances rather than implementing this trait.
pub trait Step {
    /// Executes the step.
    ///
    /// This method represents the main operation of the step. It coordinates
    /// reading items, processing them, and writing them out.
    ///
    /// # Returns
    /// - `Ok(StepExecution)`: The step completed successfully
    /// - `Err(StepExecution)`: The step failed
    fn execute(&self) -> StepResult<StepExecution>;

    /// Gets the status of the step.
    ///
    /// # Returns
    /// A `StepStatus` indicating the current status of the step
    fn get_status(&self) -> StepStatus;

    /// Gets the name of the step.
    ///
    /// # Returns
    /// A reference to the name of the step
    fn get_name(&self) -> &String;

    /// Gets the ID of the step.
    ///
    /// # Returns
    /// The UUID representing the ID of the step
    fn get_id(&self) -> Uuid;

    /// Gets the number of items read by the step.
    ///
    /// # Returns
    /// The count of items read by the step
    fn get_read_count(&self) -> usize;

    /// Gets the number of items written by the step.
    ///
    /// # Returns
    /// The count of items written by the step
    fn get_write_count(&self) -> usize;

    /// Gets the number of read errors encountered by the step.
    ///
    /// # Returns
    /// The count of read errors encountered by the step
    fn get_read_error_count(&self) -> usize;

    /// Gets the number of write errors encountered by the step.
    ///
    /// # Returns
    /// The count of write errors encountered by the step
    fn get_write_error_count(&self) -> usize;
}

/// Represents the status of a chunk.
///
/// This enum indicates whether a chunk has been fully processed or if
/// there are more items to process.
#[derive(Debug, PartialEq)]
pub enum ChunkStatus {
    /// The chunk has been fully processed.
    ///
    /// This indicates that there are no more items to process in the current
    /// data source (typically because we've reached the end of the input).
    Finished,

    /// The chunk is full and ready to be processed.
    ///
    /// This indicates that we've collected a full chunk of items (based on
    /// the configured chunk size) and they are ready to be processed.
    Full,
}

/// Represents the status of a step.
///
/// This enum indicates the current state of a step execution, including
/// both success and various failure states.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StepStatus {
    /// The step executed successfully.
    ///
    /// All items were read, processed, and written without errors
    /// exceeding configured skip limits.
    Success,

    /// An error occurred during the read operation.
    ///
    /// This indicates that an error occurred while reading items from the
    /// source, and the error count exceeded the configured skip limit.
    ReadError,

    /// An error occurred during the processing operation.
    ///
    /// This indicates that an error occurred while processing items, and
    /// the error count exceeded the configured skip limit.
    ProcessorError,

    /// An error occurred during the write operation.
    ///
    /// This indicates that an error occurred while writing items to the
    /// destination, and the error count exceeded the configured skip limit.
    WriteError,

    /// The step is starting.
    ///
    /// This is the initial state of a step before execution begins.
    Starting,
}

/// Represents the execution details of a step.
///
/// This struct captures timing information about a step execution,
/// which is useful for monitoring and reporting.
#[derive(Debug)]
pub struct StepExecution {
    /// The start time of the step execution.
    pub start: Instant,
    /// The end time of the step execution.
    pub end: Instant,
    /// The duration of the step execution.
    pub duration: Duration,
}

/// Represents an instance of a step in a batch job.
///
/// A `StepInstance` is a concrete implementation of the `Step` trait that
/// manages the execution of a batch processing step. It coordinates the
/// reading, processing, and writing of items according to the configured
/// chunking parameters.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the references to the reader, processor, and writer
/// - `R`: Type of items read from the source
/// - `W`: Type of items written to the destination
///
/// # Key Concepts
///
/// - **Chunking**: Items are read individually, but processed and written in chunks
/// - **Skip limit**: Maximum number of errors allowed before failing the step
/// - **Status tracking**: The step maintains its current status and execution metrics
///
/// # Metrics
///
/// The step tracks several metrics during execution:
/// - Number of items read
/// - Number of items written
/// - Number of read errors
/// - Number of process errors
/// - Number of write errors
pub struct StepInstance<'a, R, W> {
    /// Unique identifier for this step instance
    id: Uuid,
    /// Human-readable name for the step
    name: String,
    /// Current status of the step execution
    status: Cell<StepStatus>,
    /// Component responsible for reading items from the source
    reader: &'a dyn ItemReader<R>,
    /// Component responsible for processing items
    processor: &'a dyn ItemProcessor<R, W>,
    /// Component responsible for writing items to the destination
    writer: &'a dyn ItemWriter<W>,
    /// Number of items to process in each chunk
    chunk_size: usize,
    /// Maximum number of errors allowed before failing the step
    skip_limit: usize,
    /// Number of items successfully read
    read_count: Cell<usize>,
    /// Number of items successfully written
    write_count: Cell<usize>,
    /// Number of errors encountered during reading
    read_error_count: Cell<usize>,
    /// Number of errors encountered during processing
    process_error_count: Cell<usize>,
    /// Number of errors encountered during writing
    write_error_count: Cell<usize>,
}

impl<R, W> Step for StepInstance<'_, R, W> {
    fn execute(&self) -> StepResult<StepExecution> {
        // Start the timer
        let start = Instant::now();

        // Log the start of the step
        info!("Start of step: {}, id: {}", self.name, self.id);

        // Open the writer and handle any errors
        Self::manage_error(self.writer.open());

        // Create a vector to store the read items
        let mut read_items: Vec<R> = Vec::with_capacity(self.chunk_size);

        // Read the first chunk
        let read_chunk_result = self.read_chunk(&mut read_items);

        // Handle read errors
        if read_chunk_result.is_err() {
            self.set_status(StepStatus::ReadError);
        } else {
            let chunk_status = read_chunk_result.unwrap();

            // If the chunk is finished (nothing to read), we should still handle it as success
            if chunk_status == ChunkStatus::Finished && read_items.is_empty() {
                // No need to call write_chunk with empty data since it will return early anyway
                self.set_status(StepStatus::Success);
            } else {
                // Continue processing chunks normally
                loop {
                    // Process the chunk of items
                    let processor_chunk_result = self.process_chunk(&read_items);

                    // Handle processing errors
                    if processor_chunk_result.is_err() {
                        self.set_status(StepStatus::ProcessorError);
                        break;
                    }

                    // Write the processed items
                    let write_chunk_result = self.write_chunk(&processor_chunk_result.unwrap());

                    // Handle write errors
                    if write_chunk_result.is_err() {
                        self.set_status(StepStatus::WriteError);
                        break;
                    }

                    // If we've already reached the end, break the loop
                    if chunk_status == ChunkStatus::Finished {
                        self.set_status(StepStatus::Success);
                        break;
                    }

                    // Read the next chunk
                    let read_chunk_result = self.read_chunk(&mut read_items);

                    // Handle read errors
                    if read_chunk_result.is_err() {
                        self.set_status(StepStatus::ReadError);
                        break;
                    }

                    // Check if the chunk is finished
                    if read_chunk_result.unwrap() == ChunkStatus::Finished {
                        self.set_status(StepStatus::Success);
                        break;
                    }
                }
            }
        }

        // Close the writer and handle any errors
        Self::manage_error(self.writer.close());

        // Log the end of the step
        info!("End of step: {}, id: {}", self.name, self.id);

        // Calculate the step execution details
        let step_execution = StepExecution {
            start,
            end: Instant::now(),
            duration: start.elapsed(),
        };

        // Return the step execution details if the step is successful,
        // or an error if the step failed
        if StepStatus::Success == self.status.get() {
            Ok(step_execution)
        } else {
            Err(step_execution)
        }
    }

    fn get_status(&self) -> StepStatus {
        self.status.get()
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_read_count(&self) -> usize {
        self.read_count.get()
    }

    fn get_write_count(&self) -> usize {
        self.write_count.get()
    }

    fn get_read_error_count(&self) -> usize {
        self.read_error_count.get()
    }

    fn get_write_error_count(&self) -> usize {
        self.write_error_count.get()
    }
}

/// Implementation of the `StepInstance` with methods for chunk processing.
impl<R, W> StepInstance<'_, R, W> {
    /// Sets the status of the step instance.
    ///
    /// # Parameters
    /// - `status`: The new status to set
    fn set_status(&self, status: StepStatus) {
        self.status.set(status);
    }

    /// Checks if the skip limit for the step instance has been reached.
    ///
    /// The skip limit is the maximum number of errors allowed across all
    /// operations (read, process, write) before the step fails.
    ///
    /// # Returns
    /// `true` if the total number of errors exceeds the skip limit
    fn is_skip_limit_reached(&self) -> bool {
        self.read_error_count.get() + self.write_error_count.get() + self.process_error_count.get()
            > self.skip_limit
    }

    /// Reads a chunk of items from the reader.
    ///
    /// This method attempts to read up to `chunk_size` items from the reader.
    /// It stops when either:
    /// - The chunk is full (reached `chunk_size` items)
    /// - There are no more items to read
    /// - The error skip limit is reached
    ///
    /// # Parameters
    /// - `read_items`: Vector to store the read items
    ///
    /// # Returns
    /// - `Ok(ChunkStatus::Full)`: The chunk is full with `chunk_size` items
    /// - `Ok(ChunkStatus::Finished)`: There are no more items to read
    /// - `Err(BatchError)`: An error occurred and skip limit was reached
    fn read_chunk(&self, read_items: &mut Vec<R>) -> ChunkResult<ChunkStatus> {
        debug!("Start reading chunk");
        read_items.clear();

        loop {
            let read_result = self.reader.read();

            match read_result {
                Ok(item) => {
                    match item {
                        Some(item) => {
                            read_items.push(item);
                            self.inc_read_count();

                            if read_items.len() >= self.chunk_size {
                                return Ok(ChunkStatus::Full);
                            }
                        }
                        None => {
                            if read_items.is_empty() {
                                return Ok(ChunkStatus::Finished);
                            } else {
                                return Ok(ChunkStatus::Full);
                            }
                        }
                    };
                }
                Err(error) => {
                    warn!("Error reading item: {}", error);
                    self.inc_read_error_count();

                    if self.is_skip_limit_reached() {
                        // Set the status to ReadError when we hit the limit
                        self.set_status(StepStatus::ReadError);
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Processes a chunk of items using the processor.
    ///
    /// This method applies the processor to each item in the input chunk.
    /// It collects the successfully processed items and tracks any errors.
    ///
    /// # Parameters
    /// - `read_items`: Vector of items to process
    ///
    /// # Returns
    /// - `Ok(Vec<W>)`: Vector of successfully processed items
    /// - `Err(BatchError)`: An error occurred and skip limit was reached
    fn process_chunk(&self, read_items: &Vec<R>) -> Result<Vec<W>, BatchError> {
        debug!("Processing chunk of {} items", read_items.len());
        let mut result = Vec::with_capacity(read_items.len());

        for item in read_items {
            match self.processor.process(item) {
                Ok(processed_item) => {
                    result.push(processed_item);
                }
                Err(error) => {
                    warn!("Error processing item: {}", error);
                    self.inc_process_error_count(1);

                    if self.is_skip_limit_reached() {
                        // Set the status to ProcessorError when we hit the limit
                        self.set_status(StepStatus::ProcessorError);
                        return Err(error);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Writes a chunk of processed items using the writer.
    ///
    /// This method writes the processed items to the destination
    /// and handles any errors that occur.
    ///
    /// # Parameters
    /// - `processed_items`: Vector of items to write
    ///
    /// # Returns
    /// - `Ok(())`: All items were written successfully
    /// - `Err(BatchError)`: An error occurred and skip limit was reached
    fn write_chunk(&self, processed_items: &[W]) -> Result<(), BatchError> {
        debug!("Writing chunk of {} items", processed_items.len());

        if processed_items.is_empty() {
            debug!("No items to write, skipping write call");
            return Ok(());
        }

        match self.writer.write(processed_items) {
            Ok(()) => {
                self.inc_write_count(processed_items.len());
                Self::manage_error(self.writer.flush());
                Ok(())
            }
            Err(error) => {
                warn!("Error writing items: {}", error);
                self.inc_write_error_count(processed_items.len());

                if self.is_skip_limit_reached() {
                    // Set the status to WriteError to indicate a write failure
                    self.set_status(StepStatus::WriteError);
                    return Err(error);
                }
                Ok(())
            }
        }
    }

    /// Increments the read count by 1.
    fn inc_read_count(&self) {
        self.read_count.set(self.read_count.get() + 1);
    }

    /// Increments the read error count by 1.
    fn inc_read_error_count(&self) {
        self.read_error_count.set(self.read_error_count.get() + 1);
    }

    /// Increments the write count by the specified amount.
    ///
    /// # Parameters
    /// - `write_count`: Number to add to the write count
    fn inc_write_count(&self, write_count: usize) {
        self.write_count.set(self.write_count.get() + write_count);
    }

    /// Increments the write error count by the specified amount.
    ///
    /// # Parameters
    /// - `write_count`: Number to add to the write error count
    fn inc_write_error_count(&self, write_count: usize) {
        self.write_error_count
            .set(self.write_error_count.get() + write_count);
    }

    /// Increments the process error count by the specified amount.
    ///
    /// # Parameters
    /// - `write_count`: Number to add to the process error count
    fn inc_process_error_count(&self, write_count: usize) {
        self.process_error_count
            .set(self.process_error_count.get() + write_count);
    }

    /// Helper method to handle errors gracefully.
    ///
    /// This method is used to handle errors from operations where we want
    /// to log the error but not fail the step.
    ///
    /// # Parameters
    /// - `result`: Result to check for errors
    fn manage_error(result: Result<(), BatchError>) {
        if let Err(error) = result {
            warn!("Non-fatal error: {}", error);
        }
    }
}

/// Builder for creating a step instance.
///
/// The `StepBuilder` implements the Builder Pattern to provide a fluent API
/// for constructing `StepInstance` objects. It allows configuring the step's
/// name, reader, processor, writer, chunk size, and skip limit.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the references to the reader, processor, and writer
/// - `R`: Type of items read from the source
/// - `W`: Type of items written to the destination
///
/// # Design Pattern
///
/// This implements the Builder Pattern to separate the construction of complex
/// `StepInstance` objects from their representation.
///
/// # Default Configuration
///
/// - Name: Random alphanumeric string
/// - Processor: `DefaultProcessor` (identity transformation if types allow)
/// - Chunk size: 10
/// - Skip limit: 0 (no errors allowed)
pub struct StepBuilder<'a, R, W> {
    /// Optional name for the step (generated randomly if not specified)
    name: Option<String>,
    /// Component responsible for reading items from the source
    reader: Option<&'a dyn ItemReader<R>>,
    /// Component responsible for processing items
    processor: Option<&'a dyn ItemProcessor<R, W>>,
    /// Component responsible for writing items to the destination
    writer: Option<&'a dyn ItemWriter<W>>,
    /// Number of items to process in each chunk
    chunk_size: usize,
    /// Maximum number of errors allowed before failing the step
    skip_limit: usize,
}

impl<'a, R: 'static, W: 'static + Clone> StepBuilder<'a, R, W> {
    /// Creates a new StepBuilder with default values for all fields.
    ///
    /// # Returns
    /// A new StepBuilder with default settings.
    pub fn new() -> StepBuilder<'a, R, W> {
        StepBuilder {
            name: None,
            reader: None,
            processor: None,
            writer: None,
            chunk_size: 10,
            skip_limit: 0,
        }
    }

    /// Sets the name of the step.
    ///
    /// # Parameters
    /// - `name`: The name to assign to the step
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn name(mut self, name: String) -> StepBuilder<'a, R, W> {
        self.name = Some(name);
        self
    }

    /// Sets the reader for the step.
    ///
    /// # Parameters
    /// - `reader`: The component responsible for reading items
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn reader(mut self, reader: &'a impl ItemReader<R>) -> StepBuilder<'a, R, W> {
        self.reader = Some(reader);
        self
    }

    /// Sets the processor for the step.
    ///
    /// # Parameters
    /// - `processor`: The component responsible for processing items
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn processor(mut self, processor: &'a impl ItemProcessor<R, W>) -> StepBuilder<'a, R, W> {
        self.processor = Some(processor);
        self
    }

    /// Sets the writer for the step.
    ///
    /// # Parameters
    /// - `writer`: The component responsible for writing items
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn writer(mut self, writer: &'a impl ItemWriter<W>) -> StepBuilder<'a, R, W> {
        self.writer = Some(writer);
        self
    }

    /// Sets the chunk size for the step.
    ///
    /// The chunk size determines how many items will be read before
    /// they are processed and written as a batch.
    ///
    /// # Parameters
    /// - `chunk_size`: The number of items to process in each chunk
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn chunk(mut self, chunk_size: usize) -> StepBuilder<'a, R, W> {
        self.chunk_size = chunk_size;
        self
    }

    /// Sets the skip limit for the step.
    ///
    /// The skip limit determines how many errors can occur before
    /// the step fails.
    ///
    /// # Parameters
    /// - `skip_limit`: The maximum number of errors allowed
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn skip_limit(mut self, skip_limit: usize) -> StepBuilder<'a, R, W> {
        self.skip_limit = skip_limit;
        self
    }

    /// Builds and returns a `StepInstance` based on the configured parameters.
    ///
    /// If any required components are missing:
    /// - Name: A random name is generated
    /// - Processor: A `DefaultProcessor` is used
    /// - Reader: Panics if not provided
    /// - Writer: Panics if not provided
    ///
    /// # Returns
    /// A fully configured `StepInstance` ready for execution
    ///
    /// # Panics
    /// Panics if no reader or writer has been provided
    pub fn build(self) -> StepInstance<'a, R, W> {
        StepInstance {
            id: Uuid::new_v4(),
            name: self.name.unwrap_or(build_name()),
            status: Cell::new(StepStatus::Starting),
            reader: self.reader.expect("Reader is required for building a step"),
            processor: self.processor.unwrap_or(&DefaultProcessor),
            writer: self.writer.expect("Writer is required for building a step"),
            chunk_size: self.chunk_size,
            skip_limit: self.skip_limit,
            read_count: Cell::new(0),
            write_count: Cell::new(0),
            read_error_count: Cell::new(0),
            process_error_count: Cell::new(0),
            write_error_count: Cell::new(0),
        }
    }
}

impl<R: 'static, W: 'static + Clone> Default for StepBuilder<'_, R, W> {
    /// Creates a new StepBuilder with default values.
    ///
    /// This is equivalent to calling `StepBuilder::new()`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mockall::mock;
    use serde::{Deserialize, Serialize};

    use crate::{
        core::{
            item::{
                ItemProcessor, ItemProcessorResult, ItemReader, ItemReaderResult, ItemWriter,
                ItemWriterResult,
            },
            step::StepStatus,
        },
        BatchError,
    };

    use super::{Step, StepBuilder, StepInstance};

    mock! {
        pub TestItemReader {}
        impl ItemReader<Car> for TestItemReader {
            fn read(&self) -> ItemReaderResult<Car>;
        }
    }

    mock! {
        pub TestProcessor {}
        impl ItemProcessor<Car, Car> for TestProcessor {
            fn process(&self, item: &Car) -> ItemProcessorResult<Car>;
        }
    }

    mock! {
        pub TestItemWriter {}
        impl ItemWriter<Car> for TestItemWriter {
            fn write(&self, items: &[Car]) -> ItemWriterResult;
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct Car {
        year: u16,
        make: String,
        model: String,
        description: String,
    }

    fn mock_read(i: &mut u16, error_count: u16, end_count: u16) -> ItemReaderResult<Car> {
        if end_count > 0 && *i == end_count {
            return Ok(None);
        } else if error_count > 0 && *i == error_count {
            return Err(BatchError::ItemReader("mock read error".to_string()));
        }

        let car = Car {
            year: 1979,
            make: "make".to_owned(),
            model: "model".to_owned(),
            description: "description".to_owned(),
        };
        *i += 1;
        Ok(Some(car))
    }

    fn mock_process(i: &mut u16, error_at: &[u16]) -> ItemProcessorResult<Car> {
        *i += 1;
        if error_at.contains(i) {
            return Err(BatchError::ItemProcessor("mock process error".to_string()));
        }

        let car = Car {
            year: 1979,
            make: "make".to_owned(),
            model: "model".to_owned(),
            description: "description".to_owned(),
        };
        Ok(car)
    }

    #[test]
    fn step_should_succeded_with_empty_data() -> Result<()> {
        let mut reader = MockTestItemReader::default();
        let reader_result = Ok(None);
        reader.expect_read().return_once(move || reader_result);

        let mut processor = MockTestProcessor::default();
        processor.expect_process().never();

        let mut writer = MockTestItemWriter::default();
        writer.expect_write().never();

        let step: StepInstance<Car, Car> = StepBuilder::new()
            .name("test".to_string())
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .chunk(3)
            .build();

        let result = step.execute();

        assert!(result.is_ok());
        assert_eq!(step.get_name(), "test");
        assert!(!step.get_name().is_empty());
        assert!(!step.get_id().is_nil());
        assert_eq!(step.get_status(), StepStatus::Success);

        Ok(())
    }

    #[test]
    fn step_should_failed_with_processor_error() -> Result<()> {
        let mut i = 0;
        let mut reader = MockTestItemReader::default();
        reader
            .expect_read()
            .returning(move || mock_read(&mut i, 0, 4));

        let mut processor = MockTestProcessor::default();
        let mut i = 0;
        processor
            .expect_process()
            .returning(move |_| mock_process(&mut i, &[2]));

        let mut writer = MockTestItemWriter::default();
        writer.expect_write().never();

        let step: StepInstance<Car, Car> = StepBuilder::new()
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .chunk(3)
            .build();

        let result = step.execute();

        assert!(result.is_err());
        assert_eq!(step.get_status(), StepStatus::ProcessorError);

        Ok(())
    }

    #[test]
    fn step_should_failed_with_write_error() -> Result<()> {
        let mut i = 0;
        let mut reader = MockTestItemReader::default();
        reader
            .expect_read()
            .returning(move || mock_read(&mut i, 0, 4));

        let mut processor = MockTestProcessor::default();
        let mut i = 0;
        processor
            .expect_process()
            .returning(move |_| mock_process(&mut i, &[]));

        let mut writer = MockTestItemWriter::default();
        let result = Err(BatchError::ItemWriter("mock write error".to_string()));
        writer.expect_write().return_once(move |_| result);

        let step: StepInstance<Car, Car> = StepBuilder::new()
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .chunk(3)
            .build();

        let result = step.execute();

        assert!(result.is_err());
        assert_eq!(step.get_status(), StepStatus::WriteError);

        Ok(())
    }

    #[test]
    fn step_should_succeed_even_with_processor_error() -> Result<()> {
        let mut i = 0;
        let mut reader = MockTestItemReader::default();
        reader
            .expect_read()
            .returning(move || mock_read(&mut i, 0, 4));

        let mut processor = MockTestProcessor::default();
        let mut i = 0;
        processor
            .expect_process()
            .returning(move |_| mock_process(&mut i, &[2]));

        let mut writer = MockTestItemWriter::default();
        writer.expect_write().times(2).returning(|_| Ok(()));

        let step: StepInstance<Car, Car> = StepBuilder::new()
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .chunk(3)
            .skip_limit(1)
            .build();

        let result = step.execute();

        assert!(result.is_ok());
        assert_eq!(step.get_status(), StepStatus::Success);

        Ok(())
    }

    #[test]
    fn step_should_succeed_even_with_write_error() -> Result<()> {
        let mut i = 0;
        let mut reader = MockTestItemReader::default();
        reader
            .expect_read()
            .returning(move || mock_read(&mut i, 0, 4));

        let mut processor = MockTestProcessor::default();
        let mut i = 0;
        processor
            .expect_process()
            .returning(move |_| mock_process(&mut i, &[2]));

        let mut writer = MockTestItemWriter::default();
        writer.expect_write().times(2).returning(|_| Ok(()));

        let step: StepInstance<Car, Car> = StepBuilder::new()
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .chunk(3)
            .skip_limit(1)
            .build();

        let result = step.execute();

        assert!(result.is_ok());
        assert_eq!(step.get_status(), StepStatus::Success);

        Ok(())
    }
}
