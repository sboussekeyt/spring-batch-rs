use std::time::{Duration, Instant};

use log::info;
use uuid::Uuid;

use crate::BatchError;

use super::{build_name, step::Step};

type JobResult<T> = Result<T, BatchError>;

/// Represents a job that can be executed.
pub trait Job {
    /// Runs the job and returns the result of the job execution.
    fn run(&self) -> JobResult<JobExecution>;
}

/// Represents the execution of a job.
#[derive(Debug)]
pub struct JobExecution {
    pub start: Instant,
    pub end: Instant,
    pub duration: Duration,
}

/// Represents an instance of a job.
pub struct JobInstance<'a> {
    id: Uuid,
    name: String,
    steps: Vec<&'a dyn Step>,
}

impl Job for JobInstance<'_> {
    fn run(&self) -> JobResult<JobExecution> {
        let start = Instant::now();

        info!("Start of job: {}, id: {}", self.name, self.id);
        let steps = &self.steps;
        for step in steps {
            let result = step.execute();

            if result.is_err() {
                return Err(BatchError::Step(step.get_name().to_owned()));
            }
        }
        info!("End of job: {}, id: {}", self.name, self.id);

        let job_execution = JobExecution {
            start,
            end: Instant::now(),
            duration: start.elapsed(),
        };

        Ok(job_execution)
    }
}

/// Builder for creating a job instance.
#[derive(Default)]
pub struct JobBuilder<'a> {
    name: Option<String>,
    steps: Vec<&'a dyn Step>,
}

impl<'a> JobBuilder<'a> {
    /// Creates a new `JobBuilder` instance.
    pub fn new() -> Self {
        Self {
            name: None,
            steps: Vec::new(),
        }
    }

    /// Sets the name of the job.
    pub fn name(mut self, name: String) -> JobBuilder<'a> {
        self.name = Some(name);
        self
    }

    /// Sets the first step of the job.
    pub fn start(mut self, step: &'a dyn Step) -> JobBuilder<'a> {
        self.steps.push(step);
        self
    }

    /// Adds a step to the job.
    pub fn next(mut self, step: &'a dyn Step) -> JobBuilder<'a> {
        self.steps.push(step);
        self
    }

    /// Builds and returns a `JobInstance` based on the configured parameters.
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
