use crate::error::BatchError;

pub trait ItemReader<R> {
    fn read(&self) -> Option<Result<R, BatchError>>;
}

pub trait ItemProcessor<R, W> {
    fn process<'a>(&self, item: &'a R) -> &'a W;
}

pub trait ItemWriter<W> {
    fn write(&self, item: &W);
}

pub struct DefaultProcessor {}

impl<R> ItemProcessor<R, R> for DefaultProcessor {
    fn process<'a>(&self, item: &'a R) -> &'a R {
        item
    }
}
