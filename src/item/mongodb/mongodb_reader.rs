use std::cell::{Cell, RefCell};

use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::FindOptions,
    sync::Collection,
};
use serde::de::DeserializeOwned;

use crate::core::item::{ItemReader, ItemReaderResult};

pub trait WithObjectId {
    fn get_id(&self) -> ObjectId;
}

/// A MongoDB item reader that reads items from a MongoDB collection.
pub struct MongodbItemReader<'a, R: Send + Sync> {
    collection: &'a Collection<R>,
    filter: Document,
    options: Option<FindOptions>,
    page_size: Option<i64>,
    buffer: RefCell<Vec<R>>,
    last_id: Cell<Option<ObjectId>>,
    offset: Cell<usize>,
}

impl<R: DeserializeOwned + WithObjectId + Send + Sync> MongodbItemReader<'_, R> {
    /// Reads a page of items from the MongoDB collection and stores them in the buffer.
    fn read_page(&self) {
        self.buffer.borrow_mut().clear();

        let last_id = self.last_id.get();

        let mut filter = self.filter.clone();

        if last_id.is_some() {
            filter.extend(doc! {"oid": { "$gt": last_id }});
        };

        let options = &self.options;

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options.clone())
            .run()
            .unwrap();

        while cursor.advance().unwrap() {
            let result = cursor.deserialize_current();
            if let Ok(item) = result {
                self.last_id.set(Some(item.get_id()));
                self.buffer.borrow_mut().push(item);
            }
        }
    }
}

impl<R: DeserializeOwned + Clone + WithObjectId + Send + Sync> ItemReader<R>
    for MongodbItemReader<'_, R>
{
    /// Reads the next item from the MongoDB collection.
    ///
    /// Returns `Ok(Some(item))` if an item is read successfully,
    /// `Ok(None)` if there are no more items to read,
    /// or an error if reading the item fails.
    fn read(&self) -> ItemReaderResult<R> {
        let index = if let Some(page_size) = self.page_size {
            self.offset.get() % (page_size as usize)
        } else {
            self.offset.get()
        };

        if index == 0 {
            self.read_page();
        }

        let buffer = self.buffer.borrow();

        let result = buffer.get(index);

        match result {
            Some(item) => {
                self.offset.set(self.offset.get() + 1);
                Ok(Some(item.clone()))
            }
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct MongodbItemReaderBuilder<'a, R: Send + Sync> {
    collection: Option<&'a Collection<R>>,
    filter: Option<Document>,
    page_size: Option<i64>,
}

impl<'a, R: Send + Sync> MongodbItemReaderBuilder<'a, R> {
    /// Creates a new `MongodbItemReaderBuilder`.
    pub fn new() -> Self {
        Self {
            collection: None,
            filter: None,
            page_size: None,
        }
    }

    /// Sets the MongoDB collection to read from.
    pub fn collection(mut self, collection: &'a Collection<R>) -> MongodbItemReaderBuilder<'a, R> {
        self.collection = Some(collection);
        self
    }

    /// Sets the filter to apply when reading items from the collection.
    pub fn filter(mut self, filter: Document) -> MongodbItemReaderBuilder<'a, R> {
        self.filter = Some(filter);
        self
    }

    /// Sets the page size for reading items.
    pub fn page_size(mut self, page_size: i64) -> MongodbItemReaderBuilder<'a, R> {
        self.page_size = Some(page_size);
        self
    }

    /// Builds the `MongodbItemReader` with the configured options.
    pub fn build(&self) -> MongodbItemReader<'a, R> {
        let buffer: Vec<R> = if let Some(page_size) = self.page_size {
            let buffer_size = page_size.try_into().unwrap_or(1);
            Vec::with_capacity(buffer_size)
        } else {
            Vec::new()
        };

        let filter = if let Some(filter) = self.filter.to_owned() {
            filter
        } else {
            doc! {}
        };

        // We do not use skip because of performance issue for large dataset.
        // It is better to sort and filter with an indexed field (_id)
        let find_options = FindOptions::builder()
            .sort(doc! { "oid": 1 })
            .limit(Some(self.page_size.unwrap()))
            .build();

        MongodbItemReader {
            collection: self.collection.unwrap(),
            filter,
            options: Some(find_options),
            page_size: self.page_size,
            buffer: RefCell::new(buffer),
            last_id: Cell::new(None),
            offset: Cell::new(0),
        }
    }
}
