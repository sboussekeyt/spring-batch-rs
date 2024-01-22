use std::cell::{Cell, RefCell};

use serde::de::DeserializeOwned;
use sqlx::{any::AnyRow, Any, Pool, QueryBuilder};

use crate::{core::item::ItemReader, BatchError};

pub trait RdbcRowMapper<T> {
    fn map_row(&self, row: &AnyRow) -> T;
}

pub struct RdbcItemReader<'a, T> {
    pool: &'a Pool<Any>,
    query: &'a str,
    page_size: Option<i32>,
    offset: Cell<i32>,
    row_mapper: &'a dyn RdbcRowMapper<T>,
    buffer: RefCell<Vec<T>>,
}

impl<'a, T> RdbcItemReader<'a, T> {
    fn new(
        pool: &'a Pool<Any>,
        query: &'a str,
        page_size: Option<i32>,
        row_mapper: &'a dyn RdbcRowMapper<T>,
    ) -> Self {
        let buffer = if let Some(page_size) = page_size {
            let buffer_size = page_size.try_into().unwrap_or(1);
            Vec::with_capacity(buffer_size)
        } else {
            Vec::new()
        };

        Self {
            pool,
            query,
            page_size,
            offset: Cell::new(0),
            row_mapper,
            buffer: RefCell::new(buffer),
        }
    }

    fn _read_page(&self) {
        let mut query_builder = QueryBuilder::new(self.query);

        if self.page_size.is_some() {
            query_builder.push(format!(
                " LIMIT {} OFFSET {}",
                self.page_size.unwrap(),
                self.offset.get()
            ));
        }

        let query = query_builder.build();

        let rows = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { query.fetch_all(self.pool).await.unwrap() })
        });

        self.buffer.borrow_mut().clear();

        rows.iter().for_each(|x| {
            let item = self.row_mapper.map_row(x);
            self.buffer.borrow_mut().push(item);
        });
    }
}

impl<'a, T: DeserializeOwned + Clone> ItemReader<T> for RdbcItemReader<'a, T> {
    fn read(&self) -> Option<Result<T, BatchError>> {
        let index = if let Some(page_size) = self.page_size {
            self.offset.get() % page_size
        } else {
            self.offset.get()
        };

        if index == 0 {
            self._read_page();
        }

        let buffer = self.buffer.borrow();

        let result = buffer.get(index as usize);

        self.offset.set(self.offset.get() + 1);

        result.map(|item| Ok(item.clone()))
    }
}

#[derive(Default)]
pub struct RdbcItemReaderBuilder<'a, T> {
    pool: Option<&'a Pool<Any>>,
    query: Option<&'a str>,
    page_size: Option<i32>,
    row_mapper: Option<&'a dyn RdbcRowMapper<T>>,
}

impl<'a, T> RdbcItemReaderBuilder<'a, T> {
    pub fn new() -> Self {
        Self {
            pool: None,
            query: None,
            page_size: None,
            row_mapper: None,
        }
    }

    pub fn page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    pub fn pool(mut self, pool: &'a Pool<Any>) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn row_mapper(mut self, row_mapper: &'a dyn RdbcRowMapper<T>) -> Self {
        self.row_mapper = Some(row_mapper);
        self
    }

    pub fn build(self) -> RdbcItemReader<'a, T> {
        RdbcItemReader::new(
            self.pool.unwrap(),
            self.query.unwrap(),
            self.page_size,
            self.row_mapper.unwrap(),
        )
    }
}
