use std::{
    cell::Cell,
    time::{Duration, Instant},
};

use log::debug;

use crate::BatchError;

use super::{
    chunk::{Chunk, ChunkStatus},
    item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter},
};

#[derive(PartialEq)]
pub enum StepStatus {
    ERROR,
    SUCCESS,
}

pub struct StepResult {
    pub start: Instant,
    pub end: Instant,
    pub duration: Duration,
    pub status: StepStatus,
    pub read_count: usize,
    pub write_count: usize,
    pub read_error_count: usize,
    pub write_error_count: usize,
}

pub struct Step<'a, R, W> {
    reader: &'a dyn ItemReader<R>,
    processor: &'a dyn ItemProcessor<R, W>,
    writer: &'a dyn ItemWriter<W>,
    chunk_size: Cell<usize>,
    read_count: Cell<usize>,
    write_count: Cell<usize>,
    read_error_count: Cell<usize>,
    write_error_count: Cell<usize>,
}

impl<'a, R, W> Step<'a, R, W> {
    pub fn execute(&self) -> StepResult {
        let start = Instant::now();

        let mut chunk = Chunk::new(self.chunk_size.get());

        Self::manage_error(self.writer.open());

        let mut step_status = StepStatus::SUCCESS;

        loop {
            match chunk.get_status() {
                ChunkStatus::CONTINUABLE => {
                    debug!("Read new item");
                    chunk.add_item(self.reader.read());

                    if chunk.get_status() != &ChunkStatus::FINISHED {
                        let read_count = self.read_count.get();
                        self.read_count.set(read_count + 1);
                    }
                }
                ChunkStatus::ERROR => {
                    let read_error_count = self.read_error_count.get();
                    self.read_error_count.set(read_error_count + 1);
                    step_status = StepStatus::ERROR;
                    break;
                }
                ChunkStatus::FULL => {
                    self.execute_chunk(&chunk);
                    chunk.clear();
                    debug!("Chunk full, start a new one")
                }
                ChunkStatus::FINISHED => {
                    self.execute_chunk(&chunk);
                    debug!("End of step");
                    break;
                }
            }
        }

        Self::manage_error(self.writer.close());

        StepResult {
            start,
            end: Instant::now(),
            duration: start.elapsed(),
            status: step_status,
            read_count: self.read_count.get(),
            write_count: self.write_count.get(),
            read_error_count: self.read_error_count.get(),
            write_error_count: self.write_error_count.get(),
        }
    }

    fn execute_chunk(&self, chunk: &Chunk<R>) {
        let chunk_items = chunk.get_items();
        let mut outputs = Vec::with_capacity(chunk_items.len());

        debug!("Start processing chunk");
        for item in chunk_items {
            let item_processed = self.processor.process(item);
            outputs.push(item_processed);
        }
        debug!("End processing chunk");

        debug!("Start writting chunk");
        for item in outputs {
            Self::manage_error(self.writer.update(self.write_count.get() == 0));
            self.write(&item);
        }
        Self::manage_error(self.writer.flush());

        debug!("End writting chunk");
    }

    fn write(&self, item: &W) {
        let result = self.writer.write(item);
        match result {
            Ok(()) => {
                let write_count = self.write_count.get();
                self.write_count.set(write_count + 1);
            }
            Err(_err) => {
                let write_error_count = self.write_error_count.get();
                self.write_error_count.set(write_error_count + 1);
            }
        };
    }

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
    reader: Option<&'a dyn ItemReader<R>>,
    processor: Option<&'a dyn ItemProcessor<R, W>>,
    writer: Option<&'a dyn ItemWriter<W>>,
    chunk_size: usize,
}

impl<'a, R, W> StepBuilder<'a, R, W> {
    pub fn new() -> StepBuilder<'a, R, W> {
        StepBuilder {
            reader: None,
            processor: None,
            writer: None,
            chunk_size: 1,
        }
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

    pub fn build(self) -> Step<'a, R, W>
    where
        DefaultProcessor: ItemProcessor<R, W>,
    {
        let default_processor = &DefaultProcessor {};
        Step {
            reader: self.reader.unwrap(),
            processor: self.processor.unwrap_or(default_processor),
            writer: self.writer.unwrap(),
            chunk_size: Cell::new(self.chunk_size),
            write_error_count: Cell::new(0),
            read_error_count: Cell::new(0),
            write_count: Cell::new(0),
            read_count: Cell::new(0),
        }
    }
}
