use std::fmt::Display;

use log::info;

use crate::{core::item::ItemWriter, BatchError};

#[derive(Default)]
pub struct LoggerWriter {}

impl<T> ItemWriter<T> for LoggerWriter
where
    T: Display,
{
    fn write(&mut self, item: &T) -> Result<(), BatchError> {
        info!("Record:{}", item);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BatchError> {
        Ok(())
    }
}

impl LoggerWriter {
    pub fn new() -> Self {
        Self {}
    }
}
