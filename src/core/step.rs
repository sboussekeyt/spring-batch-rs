use std::time::{Duration, Instant};

use log::debug;

use super::{
    chunk::{Chunk, ChunkStatus},
    item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter},
};

pub struct StepResult {
    pub start: Instant,
    pub end: Instant,
    pub duration: Duration,
}

pub struct Step<'a, R, W> {
    reader: &'a mut dyn ItemReader<R>,
    processor: &'a dyn ItemProcessor<R, W>,
    writer: &'a mut dyn ItemWriter<W>,
    chunk_size: usize,
    read_count: usize,
    write_count: usize,
    read_skip_count: usize,
    write_skip_count: usize,
}

impl<'a, R, W> Step<'a, R, W> {
    pub fn execute(&mut self) -> StepResult {
        let start = Instant::now();

        let mut chunk = Chunk::new(self.chunk_size);

        self.writer.open();

        loop {
            match chunk.get_status() {
                ChunkStatus::CONTINUABLE => {
                    debug!("Read new item");
                    chunk.add_item(self.reader.read());
                    self.read_count += 1;
                }
                ChunkStatus::ERROR => {
                    self.read_skip_count += 1;
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

        self.writer.close();

        StepResult {
            start,
            end: Instant::now(),
            duration: start.elapsed(),
        }
    }

    fn execute_chunk(&mut self, chunk: &Chunk<R>) {
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
            self.writer.update(self.write_count == 0);
            self.write(item);
        }
        //self.writer.flush();
        debug!("End writting chunk");
    }

    fn write(&mut self, item: &W) {
        let result = self.writer.write(item);
        match result {
            Ok(_item) => {
                self.write_count += 1;
            }
            Err(_err) => {
                self.write_skip_count += 1;
            }
        };
    }
}

#[derive(Default)]
pub struct StepBuilder<'a, R, W> {
    reader: Option<&'a mut dyn ItemReader<R>>,
    processor: Option<&'a dyn ItemProcessor<R, W>>,
    writer: Option<&'a mut dyn ItemWriter<W>>,
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

    pub fn reader(mut self, reader: &'a mut impl ItemReader<R>) -> StepBuilder<'a, R, W> {
        self.reader = Some(reader);
        self
    }

    pub fn processor(
        mut self,
        processor: &'a mut impl ItemProcessor<R, W>,
    ) -> StepBuilder<'a, R, W> {
        self.processor = Some(processor);
        self
    }

    pub fn writer(mut self, writer: &'a mut impl ItemWriter<W>) -> StepBuilder<'a, R, W> {
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
            chunk_size: self.chunk_size,
            write_skip_count: 0,
            read_skip_count: 0,
            write_count: 0,
            read_count: 0,
        }
    }
}
