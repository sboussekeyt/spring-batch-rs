use std::fmt::Debug;

use log::info;

use crate::{core::item::ItemWriter, BatchError};

#[derive(Default)]
pub struct LoggerWriter {}

impl<T> ItemWriter<T> for LoggerWriter
where
    T: Debug,
{
    fn write(&self, items: &[T]) -> Result<(), BatchError> {
        items.iter().for_each(|item| info!("Record:{:?}", item));
        Ok(())
    }
}
