/// Core module for Spring Batch functionality.
///
/// This module contains the fundamental components that make up the Spring Batch framework.
/// It defines the core abstractions for batch processing, including:
///
/// - **Item processing**: Interfaces for reading, processing, and writing items
/// - **Jobs**: Container for a sequence of steps that defines a batch process
/// - **Steps**: Individual unit of work in a batch job
///
/// # Architecture
///
/// The Spring Batch architecture follows a chunking pattern:
///
/// 1. Items are read one by one from a data source using an `ItemReader`
/// 2. Batches of items are processed using an `ItemProcessor`
/// 3. Processed items are written in chunks using an `ItemWriter`
///
/// This chunking approach provides benefits for performance, restartability, and transaction management.
///
/// # Module Structure
///
/// - `item`: Core interfaces for reading, processing, and writing items
/// - `job`: Job execution and management
/// - `step`: Step definition and execution
use rand::distr::{Alphanumeric, SampleString};

/// Item processing interfaces.
///
/// Contains the fundamental interfaces that define the batch processing pipeline:
/// - `ItemReader`: Reads items one at a time from a data source
/// - `ItemProcessor`: Processes items from one type to another
/// - `ItemWriter`: Writes batches of items to a destination
pub mod item;

/// Job execution and management.
///
/// Contains types for defining and executing jobs:
/// - `Job`: Interface for executable batch jobs
/// - `JobInstance`: Specific instance of a job with configuration
/// - `JobExecution`: Execution details for a job run
/// - `JobBuilder`: Builder for creating job instances
pub mod job;

/// Step definition and execution.
///
/// Contains types for defining and executing steps:
/// - `Step`: Interface for individual units of work
/// - `StepInstance`: Specific instance of a step with configuration
/// - `StepExecution`: Execution details for a step run
/// - `StepBuilder`: Builder for creating step instances
pub mod step;

/// Generates a random name consisting of alphanumeric characters.
///
/// Used internally to create default names for jobs and steps when not explicitly provided.
///
/// # Returns
///
/// A `String` containing the generated random name of 8 characters.
fn build_name() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 8).clone()
}
