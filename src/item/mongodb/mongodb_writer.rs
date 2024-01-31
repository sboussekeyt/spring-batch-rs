use mongodb::{options::InsertManyOptions, sync::Collection};

use crate::{core::item::ItemWriter, BatchError};

pub struct MongodbItemWriter<'a, W> {
    collection: &'a Collection<W>,
}

impl<'a, W: serde::Serialize> ItemWriter<W> for MongodbItemWriter<'a, W> {
    fn write(&self, items: &[W]) -> Result<(), BatchError> {
        let opts = InsertManyOptions::builder().ordered(false).build();

        let result = self.collection.insert_many(items, opts);

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

#[derive(Default)]
pub struct MongodbItemWriterBuilder<'a, W> {
    collection: Option<&'a Collection<W>>,
}

impl<'a, W> MongodbItemWriterBuilder<'a, W> {
    pub fn new() -> Self {
        Self { collection: None }
    }

    pub fn collection(mut self, collection: &'a Collection<W>) -> MongodbItemWriterBuilder<'a, W> {
        self.collection = Some(collection);
        self
    }

    pub fn build(&self) -> MongodbItemWriter<'a, W> {
        MongodbItemWriter {
            collection: self.collection.unwrap(),
        }
    }
}
