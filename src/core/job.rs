use std::time::{Duration, Instant};

use log::info;
use uuid::Uuid;

use crate::BatchError;

use super::{build_name, step::Step};

/// Type alias for job execution results.
///
/// A `JobResult` is a `Result` that contains either:
/// - A successful `JobExecution` with execution details
/// - A `BatchError` indicating what went wrong
type JobResult<T> = Result<T, BatchError>;

/// Represents a job that can be executed.
///
/// This trait defines the contract for batch job execution. A job is a container
/// for a sequence of steps that are executed in order. The job is responsible for
/// orchestrating the steps and reporting the overall result.
///
/// # Design Pattern
///
/// The `Job` trait follows the Command Pattern, representing an operation that can be
/// executed and track its own execution details.
///
/// # Implementation Note
///
/// Implementations of this trait should:
/// - Execute the steps in the correct order
/// - Handle any errors that occur during execution
/// - Return execution details upon completion
///
/// # Example Usage
///
/// ```rust,no_run,compile_fail
/// use spring_batch_rs::core::job::{Job, JobBuilder};
/// use spring_batch_rs::core::step::StepBuilder;
///
/// // Create a step
/// let step = StepBuilder::new()
///     .name("process-data".to_string())
///     .reader(&some_reader)
///     .writer(&some_writer)
///     .build();
///
/// // Create and run a job
/// let job = JobBuilder::new()
///     .name("data-processing-job".to_string())
///     .start(&step)
///     .build();
///
/// let result = job.run();
/// ```
pub trait Job {
    /// Runs the job and returns the result of the job execution.
    ///
    /// # Returns
    /// - `Ok(JobExecution)` when the job executes successfully
    /// - `Err(BatchError)` when the job execution fails
    fn run(&self) -> JobResult<JobExecution>;
}

/// Represents the execution of a job.
///
/// A `JobExecution` contains timing information about a job run:
/// - When it started
/// - When it ended
/// - How long it took to execute
///
/// This information is useful for monitoring and reporting on job performance.
#[derive(Debug)]
pub struct JobExecution {
    /// The time when the job started executing
    pub start: Instant,
    /// The time when the job finished executing
    pub end: Instant,
    /// The total duration of the job execution
    pub duration: Duration,
}

/// Represents an instance of a job.
///
/// A `JobInstance` defines a specific configuration of a job that can be executed.
/// It contains:
/// - A unique identifier
/// - A name for the job
/// - A sequence of steps to be executed
///
/// # Lifecycle
///
/// A job instance is created through the `JobBuilder` and executed by calling
/// the `run` method. The steps are executed in the order they were added.
pub struct JobInstance<'a> {
    /// Unique identifier for this job instance
    id: Uuid,
    /// Human-readable name for the job
    name: String,
    /// Collection of steps that make up this job, in execution order
    steps: Vec<&'a dyn Step>,
}

impl Job for JobInstance<'_> {
    /// Runs the job by executing its steps in sequence.
    ///
    /// This method:
    /// 1. Records the start time
    /// 2. Logs the start of the job
    /// 3. Executes each step in sequence
    /// 4. If any step fails, returns an error
    /// 5. Logs the end of the job
    /// 6. Records the end time and calculates duration
    /// 7. Returns the job execution details
    ///
    /// # Returns
    /// - `Ok(JobExecution)` containing execution details if all steps succeed
    /// - `Err(BatchError)` if any step fails
    fn run(&self) -> JobResult<JobExecution> {
        // Record the start time
        let start = Instant::now();

        // Log the job start
        info!("Start of job: {}, id: {}", self.name, self.id);

        // Execute all steps in sequence
        let steps = &self.steps;
        for step in steps {
            let result = step.execute();

            // If a step fails, abort the job and return an error
            if result.is_err() {
                return Err(BatchError::Step(step.get_name().to_owned()));
            }
        }

        // Log the job completion
        info!("End of job: {}, id: {}", self.name, self.id);

        // Create and return the job execution details
        let job_execution = JobExecution {
            start,
            end: Instant::now(),
            duration: start.elapsed(),
        };

        Ok(job_execution)
    }
}

/// Builder for creating a job instance.
///
/// The `JobBuilder` implements the Builder Pattern to provide a fluent API for
/// constructing `JobInstance` objects. It allows setting the job name and adding
/// steps to the job in a chainable manner.
///
/// # Design Pattern
///
/// This implements the Builder Pattern to separate the construction of complex `JobInstance`
/// objects from their representation.
///
/// # Example
///
/// ```rust,no_run,compile_fail
/// use spring_batch_rs::core::job::JobBuilder;
///
/// let job = JobBuilder::new()
///     .name("import-customers".to_string())
///     .start(&read_step)
///     .next(&process_step)
///     .next(&write_step)
///     .build();
/// ```
#[derive(Default)]
pub struct JobBuilder<'a> {
    /// Optional name for the job (generated randomly if not specified)
    name: Option<String>,
    /// Collection of steps to be executed, in order
    steps: Vec<&'a dyn Step>,
}

impl<'a> JobBuilder<'a> {
    /// Creates a new `JobBuilder` instance.
    ///
    /// Initializes an empty job builder with no name and no steps.
    ///
    /// # Returns
    /// A new `JobBuilder` instance
    pub fn new() -> Self {
        Self {
            name: None,
            steps: Vec::new(),
        }
    }

    /// Sets the name of the job.
    ///
    /// # Parameters
    /// - `name`: The name to assign to the job
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn name(mut self, name: String) -> JobBuilder<'a> {
        self.name = Some(name);
        self
    }

    /// Sets the first step of the job.
    ///
    /// This method is semantically identical to `next()` but provides better readability
    /// when constructing the initial step of a job.
    ///
    /// # Parameters
    /// - `step`: The first step to be executed in the job
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn start(mut self, step: &'a dyn Step) -> JobBuilder<'a> {
        self.steps.push(step);
        self
    }

    /// Adds a step to the job.
    ///
    /// Steps are executed in the order they are added.
    ///
    /// # Parameters
    /// - `step`: The step to add to the job
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn next(mut self, step: &'a dyn Step) -> JobBuilder<'a> {
        self.steps.push(step);
        self
    }

    /// Builds and returns a `JobInstance` based on the configured parameters.
    ///
    /// If no name has been provided, a random name is generated.
    ///
    /// # Returns
    /// A fully configured `JobInstance` ready for execution
    pub fn build(self) -> JobInstance<'a> {
        JobInstance {
            id: Uuid::new_v4(),
            name: self.name.unwrap_or(build_name()),
            steps: self.steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use std::{
        env::{self, temp_dir},
        fs::File,
        path::Path,
    };

    use serde::{Deserialize, Serialize};

    use crate::{
        core::step::{StepBuilder, StepInstance},
        item::csv::csv_writer::CsvItemWriterBuilder,
        item::json::json_reader::JsonItemReaderBuilder,
    };

    use super::{Job, JobBuilder};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Person {
        first_name: String,
        last_name: String,
        title: String,
        email: String,
    }

    #[test]
    fn this_test_will_pass() -> Result<()> {
        env::set_var("RUST_LOG", "INFO");
        env_logger::init();

        let path = Path::new("examples/data/persons.json");

        let file = File::open(path).expect("Unable to open file");

        let reader = JsonItemReaderBuilder::new().from_reader(file);

        let writer = CsvItemWriterBuilder::new()
            .has_headers(true)
            .from_path(temp_dir().join("persons.csv"));

        let step: StepInstance<Person, Person> = StepBuilder::new()
            .reader(&reader)
            .writer(&writer)
            .chunk(2)
            .build();

        let job = JobBuilder::new()
            .name("test".to_string())
            .start(&step)
            .build();
        let _result = job.run();

        Ok(())
    }
}
