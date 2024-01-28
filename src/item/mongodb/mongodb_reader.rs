use std::cell::{Cell, RefCell};

use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::FindOptions,
    sync::Collection,
};
use serde::de::DeserializeOwned;

use crate::{core::item::ItemReader, BatchError};

pub trait WithObjectId {
    fn get_id(&self) -> ObjectId;
}

pub struct MongodbItemReader<'a, R> {
    collection: &'a Collection<R>,
    filter: Document,
    options: FindOptions,
    page_size: Option<i64>,
    buffer: RefCell<Vec<R>>,
    last_id: Cell<Option<ObjectId>>,
    offset: Cell<usize>,
}

impl<'a, R: DeserializeOwned + WithObjectId> MongodbItemReader<'a, R> {
    fn read_page(&self) {
        self.buffer.borrow_mut().clear();

        let last_id = self.last_id.get();

        let mut filter = self.filter.clone();

        if last_id.is_some() {
            filter.extend(doc! {"oid": { "$gt": last_id }});
        };

        let options = &self.options;

        let mut cursor = self.collection.find(filter, options.clone()).unwrap();

        while cursor.advance().unwrap() {
            let result = cursor.deserialize_current();
            if let Ok(item) = result {
                self.last_id.set(Some(item.get_id()));
                self.buffer.borrow_mut().push(item);
            }
        }
    }
}

impl<'a, R: DeserializeOwned + Clone + WithObjectId> ItemReader<R> for MongodbItemReader<'a, R> {
    fn read(&self) -> Option<Result<R, BatchError>> {
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
                Some(Ok(item.clone()))
            }
            None => None,
        }
    }
}

#[derive(Default)]
pub struct MongodbItemReaderBuilder<'a, R> {
    collection: Option<&'a Collection<R>>,
    filter: Option<Document>,
    page_size: Option<i64>,
}

impl<'a, R> MongodbItemReaderBuilder<'a, R> {
    pub fn new() -> Self {
        Self {
            collection: None,
            filter: None,
            page_size: None,
        }
    }

    pub fn collection(mut self, collection: &'a Collection<R>) -> MongodbItemReaderBuilder<'a, R> {
        self.collection = Some(collection);
        self
    }

    pub fn filter(mut self, filter: Document) -> MongodbItemReaderBuilder<'a, R> {
        self.filter = Some(filter);
        self
    }

    pub fn page_size(mut self, page_size: i64) -> MongodbItemReaderBuilder<'a, R> {
        self.page_size = Some(page_size);
        self
    }

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
            options: find_options,
            page_size: self.page_size,
            buffer: RefCell::new(buffer),
            last_id: Cell::new(None),
            offset: Cell::new(0),
        }
    }
}
