use log::error;

use crate::error::BatchError;

#[derive(Debug, PartialEq)]
pub enum ChunkStatus {
    CONTINUABLE,
    ERROR,
    FINISHED,
    FULL,
}

pub struct Chunk<R> {
    items: Vec<R>,
    status: ChunkStatus,
    chunk_size: usize,
}

impl<R> Chunk<R> {
    pub fn new(chunk_size: usize) -> Chunk<R> {
        Chunk {
            items: Vec::with_capacity(chunk_size),
            status: ChunkStatus::CONTINUABLE,
            chunk_size,
        }
    }

    pub fn add_item(&mut self, read_item: Option<Result<R, BatchError>>) {
        if let Some(result) = read_item {
            match result {
                Ok(item) => {
                    self.items.push(item);
                    self.status = ChunkStatus::CONTINUABLE;
                }
                Err(err) => {
                    self.status = ChunkStatus::ERROR;
                    error!("Error occured: {}", err);
                }
            };

            if self.items.len() == self.chunk_size {
                self.status = ChunkStatus::FULL;
            }
        } else {
            self.status = ChunkStatus::FINISHED;
        }
    }

    pub fn get_items(&self) -> &Vec<R> {
        &self.items
    }

    pub fn get_status(&self) -> &ChunkStatus {
        &self.status
    }

    pub fn clear(&mut self) {
        self.status = ChunkStatus::CONTINUABLE;
        self.items.clear();
    }
}
