use crate::BatchError;
use log::{debug, info, warn};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    cell::Cell,
    time::{Duration, Instant},
};
use uuid::Uuid;

use super::{
    build_name,
    item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter},
};

type StepResult<T> = Result<T, T>;

type ChunkResult<T> = Result<T, BatchError>;

pub trait Step {
    /// Executes the step.
    ///
    /// Returns a `StepResult` containing the execution details if the step is successful,
    /// or an error if the step fails.
    fn execute(&self) -> StepResult<StepExecution>;

    /// Gets the status of the step.
    ///
    /// Returns a `StepStatus` indicating the current status of the step.
    fn get_status(&self) -> StepStatus;

    /// Gets the name of the step.
    ///
    /// Returns a reference to the name of the step.
    fn get_name(&self) -> &String;

    /// Gets the ID of the step.
    ///
    /// Returns the UUID representing the ID of the step.
    fn get_id(&self) -> Uuid;

    /// Gets the number of items read by the step.
    ///
    /// Returns the count of items read by the step.
    fn get_read_count(&self) -> usize;

    /// Gets the number of items written by the step.
    ///
    /// Returns the count of items written by the step.
    fn get_write_count(&self) -> usize;

    /// Gets the number of read errors encountered by the step.
    ///
    /// Returns the count of read errors encountered by the step.
    fn get_read_error_count(&self) -> usize;

    /// Gets the number of write errors encountered by the step.
    ///
    /// Returns the count of write errors encountered by the step.
    fn get_write_error_count(&self) -> usize;
}

/// Represents the status of a chunk.
#[derive(Debug, PartialEq)]
pub enum ChunkStatus {
    /// The chunk has been fully processed.
    Finished,
    /// The chunk is full and ready to be processed.
    Full,
}

/// Represents the status of a step.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StepStatus {
    /// The step executed successfully.
    Success,
    /// An error occurred during the read operation.
    ReadError,
    /// An error occurred during the processing operation.
    ProcessorError,
    /// An error occurred during the write operation.
    WriteError,
    /// The step is starting.
    Starting,
}

/// Represents the execution details of a step.
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
pub struct StepInstance<'a, R, W> {
    id: Uuid,
    name: String,
    status: Cell<StepStatus>,
    reader: &'a dyn ItemReader<R>,
    processor: &'a dyn ItemProcessor<R, W>,
    writer: &'a dyn ItemWriter<W>,
    chunk_size: usize,
    skip_limit: usize,
    read_count: Cell<usize>,
    write_count: Cell<usize>,
    read_error_count: Cell<usize>,
    process_error_count: Cell<usize>,
    write_error_count: Cell<usize>,
}

impl<'a, R, W> Step for StepInstance<'a, R, W> {
    fn execute(&self) -> StepResult<StepExecution> {
        // Start the timer
        let start = Instant::now();

        // Log the start of the step
        info!("Start of step: {}, id: {}", self.name, self.id);

        // Open the writer and handle any errors
        Self::manage_error(self.writer.open());

        // Create a vector to store the read items
        let mut read_items: Vec<R> = Vec::with_capacity(self.chunk_size);

        // Loop until the chunk is finished or an error occurs
        loop {
            // Read a chunk of items
            let read_chunk_result = self.read_chunk(&mut read_items);

            // Handle read errors
            if read_chunk_result.is_err() {
                self.set_status(StepStatus::ReadError);
                break;
            }

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

            // Check if the chunk is finished
            if read_chunk_result.unwrap() == ChunkStatus::Finished {
                self.set_status(StepStatus::Success);
                break;
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

/// Represents an instance of a step in a batch job.
impl<'a, R, W> StepInstance<'a, R, W> {
    /// Sets the status of the step instance.
    ///
    /// # Arguments
    ///
    /// * `status` - The status to set for the step instance.
    fn set_status(&self, status: StepStatus) {
        self.status.set(status);
    }

    /// Checks if the skip limit for the step instance has been reached.
    ///
    /// Returns `true` if the skip limit has been reached, `false` otherwise.
    fn is_skip_limit_reached(&self) -> bool {
        self.read_error_count.get() + self.write_error_count.get() + self.process_error_count.get()
            > self.skip_limit
    }

    /// Reads a chunk of items from the reader.
    ///
    /// # Arguments
    ///
    /// * `read_items` - A mutable reference to a vector where the read items will be stored.
    ///
    /// Returns a `ChunkResult` indicating the status of the read operation.
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
                        }
                        None => {
                            // All items of reader have been read
                            debug!("End reading chunk: FINISHED");
                            return Ok(ChunkStatus::Finished);
                        }
                    };

                    if read_items.len() == self.chunk_size {
                        // The chunk is full, we can process and write items
                        debug!("End reading chunk: FULL");
                        return Ok(ChunkStatus::Full);
                    }
                }
                Err(err) => {
                    self.inc_read_error_count();
                    if self.is_skip_limit_reached() {
                        return Err(BatchError::ItemReader("error limit reached".to_string()));
                    } else {
                        warn!("Error occurred during read item: {}", err);
                    }
                }
            }
        }
    }

    /// Processes a chunk of read items using the processor.
    ///
    /// # Arguments
    ///
    /// * `read_items` - A reference to a vector containing the read items.
    ///
    /// Returns a `Result` containing a vector of processed items or a `BatchError` if an error occurred.
    fn process_chunk(&self, read_items: &Vec<R>) -> Result<Vec<W>, BatchError> {
        let mut processed_items = Vec::with_capacity(read_items.len());

        debug!("Start processing chunk");
        for item in read_items {
            let result = self.processor.process(item);

            match result {
                Ok(item) => {
                    debug!("Processing item");
                    processed_items.push(item)
                }
                Err(err) => {
                    self.inc_process_error_count(1);
                    if self.is_skip_limit_reached() {
                        return Err(BatchError::ItemProcessor(err.to_string()));
                    } else {
                        warn!("ItemProcessor error: {}", err.to_string());
                    }
                }
            };
        }
        debug!("End processing chunk");

        Ok(processed_items)
    }

    /// Writes a chunk of processed items using the writer.
    ///
    /// # Arguments
    ///
    /// * `processed_items` - A slice containing the processed items to write.
    ///
    /// Returns a `Result` indicating the success of the write operation or a `BatchError` if an error occurred.
    fn write_chunk(&self, processed_items: &[W]) -> Result<(), BatchError> {
        debug!("Start writing chunk");

        let result = self.writer.write(processed_items);
        match result {
            Ok(()) => {
                debug!("ItemWriter success")
            }
            Err(err) => {
                self.inc_write_error_count(processed_items.len());
                if self.is_skip_limit_reached() {
                    return Err(BatchError::ItemWriter(err.to_string()));
                } else {
                    warn!("Error occurred during write item: {}", err);
                }
            }
        }

        match self.writer.flush() {
            Ok(()) => {
                self.inc_write_count(processed_items.len());
                debug!("End writing chunk");
                Ok(())
            }
            Err(err) => {
                self.inc_write_error_count(processed_items.len());
                if self.is_skip_limit_reached() {
                    Err(BatchError::ItemWriter(err.to_string()))
                } else {
                    warn!("Error occurred during flush item: {}", err);
                    Ok(())
                }
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
    /// # Arguments
    ///
    /// * `write_count` - The amount to increment the write count by.
    fn inc_write_count(&self, write_count: usize) {
        self.write_count.set(self.write_count.get() + write_count);
    }

    /// Increments the write error count by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `write_count` - The amount to increment the write error count by.
    fn inc_write_error_count(&self, write_count: usize) {
        self.write_error_count
            .set(self.write_error_count.get() + write_count);
    }

    /// Increments the process error count by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `write_count` - The amount to increment the process error count by.
    fn inc_process_error_count(&self, write_count: usize) {
        self.process_error_count
            .set(self.process_error_count.get() + write_count);
    }

    /// Manages the error returned by a step instance operation.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of the step instance operation.
    fn manage_error(result: Result<(), BatchError>) {
        match result {
            Ok(()) => {}
            Err(error) => {
                panic!("{}", error.to_string());
            }
        };
    }
}

#[derive(Default)]
pub struct StepBuilder<'a, R, W> {
    name: Option<String>,
    reader: Option<&'a dyn ItemReader<R>>,
    processor: Option<&'a dyn ItemProcessor<R, W>>,
    writer: Option<&'a dyn ItemWriter<W>>,
    chunk_size: usize,
    skip_limit: usize,
}

impl<'a, R: Serialize, W: DeserializeOwned> StepBuilder<'a, R, W> {
    pub fn new() -> StepBuilder<'a, R, W> {
        Self {
            name: None,
            reader: None,
            processor: None,
            writer: None,
            chunk_size: 1,
            skip_limit: 0,
        }
    }

    pub fn name(mut self, name: String) -> StepBuilder<'a, R, W> {
        self.name = Some(name);
        self
    }

    pub fn reader(mut self, reader: &'a impl ItemReader<R>) -> StepBuilder<'a, R, W> {
        self.reader = Some(reader);
        self
    }

    pub fn processor(mut self, processor: &'a impl ItemProcessor<R, W>) -> StepBuilder<'a, R, W> {
        self.processor = Some(processor);
        self
    }

    pub fn writer(mut self, writer: &'a impl ItemWriter<W>) -> StepBuilder<'a, R, W> {
        self.writer = Some(writer);
        self
    }

    pub fn chunk(mut self, chunk_size: usize) -> StepBuilder<'a, R, W> {
        self.chunk_size = chunk_size;
        self
    }

    pub fn skip_limit(mut self, skip_limit: usize) -> StepBuilder<'a, R, W> {
        self.skip_limit = skip_limit;
        self
    }

    pub fn build(self) -> StepInstance<'a, R, W> {
        let default_processor = &DefaultProcessor {};

        StepInstance {
            id: Uuid::new_v4(),
            name: self.name.unwrap_or(build_name()),
            status: Cell::new(StepStatus::Starting),
            reader: self.reader.unwrap(),
            processor: self.processor.unwrap_or(default_processor),
            writer: self.writer.unwrap(),
            chunk_size: self.chunk_size,
            skip_limit: self.skip_limit,
            write_error_count: Cell::new(0),
            process_error_count: Cell::new(0),
            read_error_count: Cell::new(0),
            write_count: Cell::new(0),
            read_count: Cell::new(0),
        }
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
        writer.expect_write().times(1).returning(|_| Ok(()));

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
