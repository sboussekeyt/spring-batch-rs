use std::fmt::Debug;

use log::info;

use crate::{core::item::ItemWriter, BatchError};

#[derive(Default)]
pub struct LoggerWriter {}

impl LoggerWriter {
    pub fn new() -> Self {
        Self {}
    }
}

impl<T> ItemWriter<T> for LoggerWriter
where
    T: Debug,
{
    fn write(&self, item: &T) -> Result<(), BatchError> {
        info!("Record:{:?}", item);
        Ok(())
    }

    fn flush(&self) -> Result<(), BatchError> {
        Ok(())
    }
}
