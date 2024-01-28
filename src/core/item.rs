use serde::{de::DeserializeOwned, Serialize};

use crate::error::BatchError;

pub trait ItemReader<R> {
    fn read(&self) -> Option<Result<R, BatchError>>;
}

pub trait ItemProcessor<R, W> {
    fn process<'a>(&'a self, item: &'a R) -> W;
}

pub trait ItemWriter<W> {
    fn write(&self, items: &[W]) -> Result<(), BatchError>;
    fn flush(&self) -> Result<(), BatchError> {
        Ok(())
    }
    fn open(&self) -> Result<(), BatchError> {
        Ok(())
    }
    fn close(&self) -> Result<(), BatchError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DefaultProcessor {}

impl<R: Serialize, W: DeserializeOwned> ItemProcessor<R, W> for DefaultProcessor {
    fn process<'a>(&'a self, item: &'a R) -> W {
        // TODO: For performance reason the best is to return directly the item. R and W are of the same type
        // https://github.com/sboussekeyt/spring-batch-rs/issues/32
        let serialised = serde_json::to_string(&item).unwrap();
        serde_json::from_str(&serialised).unwrap()
    }
}
