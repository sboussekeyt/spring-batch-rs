use std::{
    cell::Cell,
    time::{Duration, Instant},
};

use log::{debug, error};

use crate::BatchError;

use super::item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter};

#[derive(Debug, PartialEq)]
pub enum ChunkStatus {
    ERROR,
    FINISHED,
    FULL,
}

#[derive(PartialEq)]
pub enum StepStatus {
    ERROR,
    SUCCESS,
    STARTED,
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
    chunk_size: usize,
    skip_limit: usize,
    read_count: Cell<usize>,
    write_count: Cell<usize>,
    read_error_count: Cell<usize>,
    write_error_count: Cell<usize>,
}

impl<'a, R, W> Step<'a, R, W> {
    pub fn execute(&self) -> StepResult {
        let start = Instant::now();

        debug!("Start of step");

        Self::_manage_error(self.writer.open());

        let mut read_items: Vec<R> = Vec::with_capacity(self.chunk_size);

        let mut step_status;

        loop {
            let read_chunk_status = self._read_chunk(&mut read_items);

            if read_chunk_status == ChunkStatus::ERROR {
                step_status = StepStatus::ERROR;
                break;
            }

            let processed_items = self._process_chunk(&read_items);

            let write_chunk_status = self._write_chunk(&processed_items);

            step_status = self._to_step_status(read_chunk_status, write_chunk_status);

            if self._is_step_ended(&step_status) {
                break;
            }
        }

        Self::_manage_error(self.writer.close());

        debug!("End of step");

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

    fn _is_step_ended(&self, step_status: &StepStatus) -> bool {
        match step_status {
            StepStatus::SUCCESS => true,
            StepStatus::ERROR => true,
            StepStatus::STARTED => false,
        }
    }

    fn _to_step_status(
        &self,
        read_chunk_status: ChunkStatus,
        write_chunk_status: ChunkStatus,
    ) -> StepStatus {
        if write_chunk_status == ChunkStatus::ERROR || read_chunk_status == ChunkStatus::ERROR {
            return StepStatus::ERROR;
        } else if read_chunk_status == ChunkStatus::FINISHED {
            return StepStatus::SUCCESS;
        }
        StepStatus::STARTED
    }

    fn _is_skip_limit_reached(&self) -> bool {
        self.read_error_count.get() + self.write_error_count.get() > self.skip_limit
    }

    fn _read_chunk(&self, read_items: &mut Vec<R>) -> ChunkStatus {
        debug!("Start reading chunk");
        read_items.clear();

        loop {
            let read_result = self.reader.read();

            if let Some(result) = read_result {
                match result {
                    Ok(item) => {
                        read_items.push(item);
                        self._inc_read_count();
                    }
                    Err(err) => {
                        self._inc_read_error_count();
                        error!("Error occured during read item: {}", err);
                    }
                };

                // In first phase, there is no fault tolerance
                if self._is_skip_limit_reached() {
                    return ChunkStatus::ERROR;
                }

                if read_items.len() == self.chunk_size {
                    // The chunk is full, we can process and write items
                    debug!("End reading chunk: FULL");
                    return ChunkStatus::FULL;
                }
            } else {
                // All items of reader have been read
                debug!("End reading chunk: FINISHED");
                return ChunkStatus::FINISHED;
            }
        }
    }

    fn _process_chunk(&self, read_items: &Vec<R>) -> Vec<W> {
        let mut processesed_items = Vec::with_capacity(read_items.len());

        debug!("Start processing chunk");
        for item in read_items {
            let item_processed = self.processor.process(item);
            processesed_items.push(item_processed);
        }
        debug!("End processing chunk");

        processesed_items
    }

    fn _write_chunk(&self, processesed_items: &Vec<W>) -> ChunkStatus {
        debug!("Start writting chunk");

        let result = self.writer.write(processesed_items);
        match result {
            Ok(()) => debug!("ItemWriter error"),
            Err(err) => error!("ItemWriter error: {}", err.to_string()),
        };

        match self.writer.flush() {
            Ok(()) => {
                self._inc_write_count(processesed_items.len());
                debug!("End writting chunk");
                ChunkStatus::FULL
            }
            Err(err) => {
                self._inc_write_error_count(processesed_items.len());
                error!("ItemWriter error: {}", err.to_string());
                if self._is_skip_limit_reached() {
                    ChunkStatus::ERROR
                } else {
                    ChunkStatus::FULL
                }
            }
        }
    }

    fn _inc_read_count(&self) {
        self.read_count.set(self.read_count.get() + 1);
    }

    fn _inc_read_error_count(&self) {
        self.read_error_count.set(self.read_error_count.get() + 1);
    }

    fn _inc_write_count(&self, write_count: usize) {
        self.write_count.set(self.write_count.get() + write_count);
    }

    fn _inc_write_error_count(&self, write_count: usize) {
        self.write_error_count
            .set(self.write_error_count.get() + write_count);
    }

    fn _manage_error(result: Result<(), BatchError>) {
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
    skip_limit: usize,
}

impl<'a, R, W> StepBuilder<'a, R, W> {
    pub fn new() -> StepBuilder<'a, R, W> {
        Self {
            reader: None,
            processor: None,
            writer: None,
            chunk_size: 1,
            skip_limit: 0,
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

    pub fn skip_limit(mut self, skip_limit: usize) -> StepBuilder<'a, R, W> {
        self.skip_limit = skip_limit;
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
            chunk_size: self.chunk_size,
            skip_limit: self.skip_limit,
            write_error_count: Cell::new(0),
            read_error_count: Cell::new(0),
            write_count: Cell::new(0),
            read_count: Cell::new(0),
        }
    }
}
