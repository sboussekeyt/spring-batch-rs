use std::time::{Duration, Instant};

use log::{debug, info, warn};
use uuid::Uuid;

use crate::{core::step::ChunkStatus, BatchError};

use super::{
    build_name,
    item::{ItemProcessor, ItemReader, ItemWriter},
    step::StepStatus,
};

pub struct StepExecution {
    /// Unique identifier for this step instance
    pub id: Uuid,
    /// Human-readable name for the step
    pub name: String,
    /// Current status of the step execution
    pub status: StepStatus,
    pub start_time: Instant,
    pub end_time: Instant,
    pub duration: Duration,
    /// Number of items successfully read
    pub read_count: usize,
    /// Number of items successfully written
    pub write_count: usize,
    /// Number of errors encountered during reading
    pub read_error_count: usize,
    /// Number of errors encountered during processing
    pub process_error_count: usize,
    /// Number of errors encountered during writing
    pub write_error_count: usize,
}

pub enum BatchStatus {
    /**
     * The batch job has successfully completed its execution.
     */
    COMPLETED,
    /**
     * Status of a batch job prior to its execution.
     */
    STARTING,
    /**
     * Status of a batch job that is running.
     */
    STARTED,
    /**
     * Status of batch job waiting for a step to complete before stopping the batch job.
     */
    STOPPING,
    /**
     * Status of a batch job that has been stopped by request.
     */
    STOPPED,
    /**
     * Status of a batch job that has failed during its execution.
     */
    FAILED,
    /**
     * Status of a batch job that did not stop properly and can not be restarted.
     */
    ABANDONED,
    /**
     * Status of a batch job that is in an uncertain state.
     */
    UNKNOWN,
}
pub trait Step {
    /// Executes the step.
    ///
    /// This method represents the main operation of the step. It coordinates
    /// reading items, processing them, and writing them out.
    ///
    /// # Returns
    /// - `Ok(StepExecution)`: The step completed successfully
    /// - `Err(StepExecution)`: The step failed
    fn execute(&self, step_execution: &mut StepExecution) -> Result<(), BatchError>;
}

pub enum RepeatStatus {
    /// The tasklet can continue to execute.
    Continuable,
    /// The tasklet has finished executing.
    Finished,
}

trait Tasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError>;
}

pub struct TaskletStep<'a> {
    tasklet: &'a dyn Tasklet,
}

impl<'a> Step for TaskletStep<'a> {
    fn execute(&self, step_execution: &mut StepExecution) -> Result<(), BatchError> {
        self.tasklet.execute(step_execution);
        Ok(())
    }
}

pub struct TaskletBuilder<'a> {
    tasklet: Option<&'a dyn Tasklet>,
}

impl<'a> TaskletBuilder<'a> {
    fn new() -> Self {
        Self { tasklet: None }
    }

    fn tasklet(mut self, tasklet: &'a dyn Tasklet) -> Self {
        self.tasklet = Some(tasklet);
        self
    }

    fn build(&self) -> TaskletStep<'a> {
        TaskletStep {
            tasklet: self
                .tasklet
                .expect("Tasklet is required for building a step"),
        }
    }
}

pub struct ChunkOrientedStep<'a, I, O> {
    name: String,
    /// Component responsible for reading items from the source
    reader: &'a dyn ItemReader<I>,
    /// Component responsible for processing items
    processor: &'a dyn ItemProcessor<I, O>,
    /// Component responsible for writing items to the destination
    writer: &'a dyn ItemWriter<O>,
    /// Number of items to process in each chunk
    chunk_size: u16,
    /// Maximum number of errors allowed before failing the step
    skip_limit: u16,
}

impl<'a, I, O> Step for ChunkOrientedStep<'a, I, O> {
    fn execute(&self, step_execution: &mut StepExecution) -> Result<(), BatchError> {
        // Start the timer
        let start_time = Instant::now();
        step_execution.status = StepStatus::Starting;

        // Log the start of the step
        info!(
            "Start of step: {}, id: {}",
            step_execution.name, step_execution.id
        );

        // Open the writer and handle any errors
        Self::manage_error(self.writer.open());

        // Read the first chunk
        let read_chunk_result = self.read_chunk(step_execution);
        // Handle read errors
        if read_chunk_result.is_err() {
            step_execution.status = StepStatus::ReadError;
        } else {
            let read_chunk = read_chunk_result.unwrap();

            let chunk_status = read_chunk.1;
            let read_items = read_chunk.0;

            // If the chunk is finished (nothing to read), we should still handle it as success
            if chunk_status == ChunkStatus::Finished && read_items.is_empty() {
                // No need to call write_chunk with empty data since it will return early anyway
                step_execution.status = StepStatus::Success;
            } else {
                // Continue processing chunks normally
                loop {
                    // Process the chunk of items
                    let processor_chunk_result = self.process_chunk(step_execution, &read_items);

                    // Handle processing errors
                    if processor_chunk_result.is_err() {
                        step_execution.status = StepStatus::ProcessorError;
                        break;
                    }

                    // Write the processed items
                    let write_chunk_result =
                        self.write_chunk(step_execution, &processor_chunk_result.unwrap());

                    // Handle write errors
                    if write_chunk_result.is_err() {
                        step_execution.status = StepStatus::WriteError;
                        break;
                    }

                    // If we've already reached the end, break the loop
                    if chunk_status == ChunkStatus::Finished {
                        step_execution.status = StepStatus::Success;
                        break;
                    }

                    // Read the next chunk
                    let read_chunk_result = self.read_chunk(step_execution);

                    // Handle read errors
                    if read_chunk_result.is_err() {
                        step_execution.status = StepStatus::ReadError;
                        break;
                    }

                    // Check if the chunk is finished
                    if chunk_status == ChunkStatus::Finished {
                        step_execution.status = StepStatus::Success;
                        break;
                    }
                }
            }
        }

        // Close the writer and handle any errors
        Self::manage_error(self.writer.close());

        // Log the end of the step
        info!(
            "End of step: {}, id: {}",
            step_execution.name, step_execution.id
        );

        // Calculate the step execution details
        step_execution.start_time = start_time;
        step_execution.end_time = Instant::now();
        step_execution.duration = start_time.elapsed();

        // Return the step execution details if the step is successful,
        // or an error if the step failed
        if StepStatus::Success == step_execution.status {
            Ok(())
        } else {
            Err(BatchError::Step(step_execution.name.clone()))
        }
    }
}

impl<'a, I, O> ChunkOrientedStep<'a, I, O> {
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
    fn read_chunk(
        &self,
        step_execution: &mut StepExecution,
    ) -> Result<(Vec<I>, ChunkStatus), BatchError> {
        debug!("Start reading chunk");

        let mut read_items = Vec::with_capacity(self.chunk_size as usize);

        loop {
            let read_result = self.reader.read();

            match read_result {
                Ok(item) => {
                    match item {
                        Some(item) => {
                            read_items.push(item);
                            step_execution.read_count += 1;

                            if read_items.len() >= self.chunk_size as usize {
                                return Ok((read_items, ChunkStatus::Full));
                            }
                        }
                        None => {
                            if read_items.is_empty() {
                                return Ok((read_items, ChunkStatus::Finished));
                            } else {
                                return Ok((read_items, ChunkStatus::Full));
                            }
                        }
                    };
                }
                Err(error) => {
                    warn!("Error reading item: {}", error);
                    step_execution.read_error_count += 1;

                    if self.is_skip_limit_reached(step_execution) {
                        // Set the status to ReadError when we hit the limit
                        step_execution.status = StepStatus::ReadError;
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
    fn process_chunk(
        &self,
        step_execution: &mut StepExecution,
        read_items: &Vec<I>,
    ) -> Result<Vec<O>, BatchError> {
        debug!("Processing chunk of {} items", read_items.len());
        let mut result = Vec::with_capacity(read_items.len());

        for item in read_items {
            match self.processor.process(item) {
                Ok(processed_item) => {
                    result.push(processed_item);
                }
                Err(error) => {
                    warn!("Error processing item: {}", error);
                    step_execution.process_error_count += 1;

                    if self.is_skip_limit_reached(step_execution) {
                        // Set the status to ProcessorError when we hit the limit
                        step_execution.status = StepStatus::ProcessorError;
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
    fn write_chunk(
        &self,
        step_execution: &mut StepExecution,
        processed_items: &[O],
    ) -> Result<(), BatchError> {
        debug!("Writing chunk of {} items", processed_items.len());

        if processed_items.is_empty() {
            debug!("No items to write, skipping write call");
            return Ok(());
        }

        match self.writer.write(processed_items) {
            Ok(()) => {
                step_execution.write_count += processed_items.len();
                Self::manage_error(self.writer.flush());
                Ok(())
            }
            Err(error) => {
                warn!("Error writing items: {}", error);
                step_execution.write_error_count += processed_items.len();

                if self.is_skip_limit_reached(step_execution) {
                    // Set the status to WriteError to indicate a write failure
                    step_execution.status = StepStatus::WriteError;
                    return Err(error);
                }
                Ok(())
            }
        }
    }

    fn is_skip_limit_reached(&self, step_execution: &StepExecution) -> bool {
        step_execution.read_error_count
            + step_execution.write_error_count
            + step_execution.process_error_count
            > self.skip_limit.into()
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

pub struct ChunkOrientedStepBuilder<'a, I, O> {
    /// Optional name for the step (generated randomly if not specified)
    name: String,
    /// Component responsible for reading items from the source
    reader: Option<&'a dyn ItemReader<I>>,
    /// Component responsible for processing items
    processor: Option<&'a dyn ItemProcessor<I, O>>,
    /// Component responsible for writing items to the destination
    writer: Option<&'a dyn ItemWriter<O>>,
    /// Number of items to process in each chunk
    chunk_size: u16,
    /// Maximum number of errors allowed before failing the step
    skip_limit: u16,
}

impl<'a, I, O> ChunkOrientedStepBuilder<'a, I, O> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reader: None,
            processor: None,
            writer: None,
            chunk_size: 10,
            skip_limit: 0,
        }
    }

    pub fn reader(mut self, reader: &'a dyn ItemReader<I>) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn processor(mut self, processor: &'a dyn ItemProcessor<I, O>) -> Self {
        self.processor = Some(processor);
        self
    }

    pub fn writer(mut self, writer: &'a dyn ItemWriter<O>) -> Self {
        self.writer = Some(writer);
        self
    }

    pub fn chunk_size(mut self, chunk_size: u16) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn skip_limit(mut self, skip_limit: u16) -> Self {
        self.skip_limit = skip_limit;
        self
    }

    pub fn build(self) -> ChunkOrientedStep<'a, I, O> {
        ChunkOrientedStep {
            name: self.name,
            reader: self.reader.expect("Reader is required for building a step"),
            processor: self.processor.unwrap(),
            writer: self.writer.expect("Writer is required for building a step"),
            chunk_size: self.chunk_size,
            skip_limit: self.skip_limit,
        }
    }
}

pub struct StepBuilder {
    name: String,
}

impl StepBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn tasklet<'a>(self, tasklet: &'a dyn Tasklet) -> TaskletBuilder<'a> {
        TaskletBuilder::new().tasklet(tasklet)
    }

    pub fn chunk<'a, I, O>(self, chunk_size: u16) -> ChunkOrientedStepBuilder<'a, I, O> {
        ChunkOrientedStepBuilder::new(&self.name).chunk_size(chunk_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTasklet;
    impl Tasklet for TestTasklet {
        fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
            Ok(RepeatStatus::Continuable)
        }
    }

    struct TestItemReader;

    impl ItemReader<String> for TestItemReader {
        fn read(&self) -> Result<Option<String>, BatchError> {
            Ok(Some("test".to_string()))
        }
    }

    struct TestItemProcessor;

    impl ItemProcessor<String, String> for TestItemProcessor {
        fn process(&self, item: &String) -> Result<String, BatchError> {
            Ok(item.clone())
        }
    }

    struct TestItemWriter;

    impl ItemWriter<String> for TestItemWriter {
        fn write(&self, items: &[String]) -> Result<(), BatchError> {
            Ok(())
        }
    }

    #[test]
    fn test_tasklet_step() {
        let tasklet = TestTasklet;
        let reader = TestItemReader;
        let processor = TestItemProcessor;
        let writer = TestItemWriter;

        let step = StepBuilder::new("test").tasklet(&tasklet).build();

        let step2 = StepBuilder::new("test")
            .chunk(5)
            .reader(&reader)
            .processor(&processor)
            .writer(&writer)
            .build();
    }
}
