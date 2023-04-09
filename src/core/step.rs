use log::debug;

use super::{
    chunk::{Chunk, ChunkStatus},
    item::{DefaultProcessor, ItemProcessor, ItemReader, ItemWriter},
};

pub struct Step<'a, R, W> {
    reader: &'a dyn ItemReader<R>,
    processor: &'a dyn ItemProcessor<R, W>,
    writer: &'a dyn ItemWriter<W>,
    chunk_size: usize,
}

impl<'a, R, W> Step<'a, R, W> {
    pub fn execute(&self) {
        let mut chunk = Chunk::new(self.chunk_size);

        loop {
            match chunk.get_status() {
                ChunkStatus::CONTINUABLE => {
                    debug!("Read new item");
                    chunk.add_item(self.reader.read())
                }
                ChunkStatus::COMPLETE => {
                    self.execute_chunk(chunk.get_items());
                    chunk.clear();
                    debug!("Chunk complete, start new one")
                }
                ChunkStatus::FINISHED => {
                    self.execute_chunk(chunk.get_items());
                    debug!("End of step");
                    break;
                }
            }
        }
    }

    fn execute_chunk(&self, chunk_items: &Vec<R>) {
        let mut outputs = Vec::with_capacity(chunk_items.len());

        debug!("Start processing chunk");
        for item in chunk_items {
            let item_processed = self.processor.process(&item);
            outputs.push(item_processed);
        }
        debug!("End processing chunk");

        debug!("Start writting chunk");
        for item in outputs {
            self.writer.write(&item);
        }
        debug!("End writting chunk");
    }
}

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
            chunk_size: self.chunk_size,
        }
    }
}
