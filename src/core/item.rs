use crate::error::BatchError;

pub trait ItemReader<R> {
    fn read(&self) -> Option<Result<R, BatchError>>;
}

pub trait ItemProcessor<R, W> {
    fn process<'a>(&'a self, item: &'a R) -> W;
}

pub trait ItemWriter<W> {
    fn write(&self, item: &W) -> Result<(), BatchError>;
    fn flush(&self) -> Result<(), BatchError>;
    fn open(&self) -> Result<(), BatchError> {
        Ok(())
    }
    fn update(&self, _is_first_item: bool) -> Result<(), BatchError> {
        Ok(())
    }
    fn close(&self) -> Result<(), BatchError> {
        Ok(())
    }
}

pub struct DefaultProcessor {}

impl<R: Clone> ItemProcessor<R, R> for DefaultProcessor {
    fn process<'a>(&'a self, item: &'a R) -> R {
        item.clone()
    }
}
