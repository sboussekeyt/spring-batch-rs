use crate::error::BatchError;

pub trait ItemReader<R> {
    fn read(&mut self) -> Option<Result<R, BatchError>>;
}

pub trait ItemProcessor<R, W> {
    fn process<'a>(&self, item: &'a R) -> &'a W;
}

pub trait ItemWriter<W> {
    fn write(&mut self, item: &W) -> Result<(), BatchError>;
    fn flush(&mut self) -> Result<(), BatchError>;
    fn open(&mut self) -> Result<(), BatchError> {
        Ok(())
    }
    fn update(&mut self, _is_first_item: bool) -> Result<(), BatchError> {
        Ok(())
    }
    fn close(&mut self) -> Result<(), BatchError> {
        Ok(())
    }
}

pub struct DefaultProcessor {}

impl<R> ItemProcessor<R, R> for DefaultProcessor {
    fn process<'a>(&self, item: &'a R) -> &'a R {
        item
    }
}
