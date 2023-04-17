use std::fmt::Display;

use log::info;

use crate::core::item::ItemWriter;

#[derive(Default)]
pub struct LoggerWriter {}

impl<T> ItemWriter<T> for LoggerWriter
where
    T: Display,
{
    fn write(&self, item: &T) {
        info!("Record:{}", item);
    }
}

impl LoggerWriter {
    pub fn new() -> Self {
        Self {}
    }
}
