use mongodb::{options::InsertManyOptions, sync::Collection};

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

/// Represents a MongoDB item writer.
pub struct MongodbItemWriter<'a, O: Send + Sync> {
    collection: &'a Collection<O>,
}

impl<O: serde::Serialize + Send + Sync> ItemWriter<O> for MongodbItemWriter<'_, O> {
    /// Writes the items to the MongoDB collection.
    ///
    /// # Arguments
    ///
    /// * `items` - The items to be written.
    ///
    /// # Returns
    ///
    /// Returns an `ItemWriterResult` indicating the result of the write operation.
    fn write(&self, items: &[O]) -> ItemWriterResult {
        let opts = InsertManyOptions::builder().ordered(false).build();

        let result = self.collection.insert_many(items).with_options(opts).run();

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

/// Builder for `MongodbItemWriter`.
#[derive(Default)]
pub struct MongodbItemWriterBuilder<'a, O: Send + Sync> {
    collection: Option<&'a Collection<O>>,
}

impl<'a, O: Send + Sync> MongodbItemWriterBuilder<'a, O> {
    /// Creates a new `MongodbItemWriterBuilder` instance.
    pub fn new() -> Self {
        Self { collection: None }
    }

    /// Sets the MongoDB collection for the writer.
    ///
    /// # Arguments
    ///
    /// * `collection` - The MongoDB collection to write to.
    ///
    /// # Returns
    ///
    /// Returns the updated `MongodbItemWriterBuilder` instance.
    pub fn collection(mut self, collection: &'a Collection<O>) -> MongodbItemWriterBuilder<'a, O> {
        self.collection = Some(collection);
        self
    }

    /// Builds a `MongodbItemWriter` instance.
    ///
    /// # Returns
    ///
    /// Returns a `MongodbItemWriter` instance with the specified configuration.
    pub fn build(&self) -> MongodbItemWriter<'a, O> {
        MongodbItemWriter {
            collection: self.collection.unwrap(),
        }
    }
}
