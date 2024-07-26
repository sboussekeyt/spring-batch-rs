use mongodb::{options::InsertManyOptions, sync::Collection};

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

/// Represents a MongoDB item writer.
pub struct MongodbItemWriter<'a, W: Send + Sync> {
    collection: &'a Collection<W>,
}

impl<'a, W: serde::Serialize + Send + Sync> ItemWriter<W> for MongodbItemWriter<'a, W> {
    /// Writes the items to the MongoDB collection.
    ///
    /// # Arguments
    ///
    /// * `items` - The items to be written.
    ///
    /// # Returns
    ///
    /// Returns an `ItemWriterResult` indicating the result of the write operation.
    fn write(&self, items: &[W]) -> ItemWriterResult {
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
pub struct MongodbItemWriterBuilder<'a, W: Send + Sync> {
    collection: Option<&'a Collection<W>>,
}

impl<'a, W: Send + Sync> MongodbItemWriterBuilder<'a, W> {
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
    pub fn collection(mut self, collection: &'a Collection<W>) -> MongodbItemWriterBuilder<'a, W> {
        self.collection = Some(collection);
        self
    }

    /// Builds a `MongodbItemWriter` instance.
    ///
    /// # Returns
    ///
    /// Returns a `MongodbItemWriter` instance with the specified configuration.
    pub fn build(&self) -> MongodbItemWriter<'a, W> {
        MongodbItemWriter {
            collection: self.collection.unwrap(),
        }
    }
}
