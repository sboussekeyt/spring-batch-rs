use serde::Serialize;
use sqlx::{query_builder::Separated, Any, Pool, QueryBuilder};

use crate::{core::item::ItemWriter, BatchError};

// The number of parameters in MySQL must fit in a `u16`.
const BIND_LIMIT: usize = 65535;

pub trait RdbcItemBinder<T> {
    fn bind(&self, item: &T, query_builder: Separated<Any, &str>);
}

pub struct RdbcItemWriter<'a, W> {
    pool: &'a Pool<Any>,
    table: &'a str,
    columns: Vec<&'a str>,
    item_binder: &'a dyn RdbcItemBinder<W>,
}

impl<'a, W> RdbcItemWriter<'a, W> {
    fn new(
        pool: &'a Pool<Any>,
        table: &'a str,
        columns: Vec<&'a str>,
        item_binder: &'a dyn RdbcItemBinder<W>,
    ) -> Self {
        Self {
            pool,
            table,
            columns,
            item_binder,
        }
    }
}

impl<'a, W: Serialize + Clone> ItemWriter<W> for RdbcItemWriter<'a, W> {
    fn write(&self, items: &[W]) -> Result<(), BatchError> {
        let mut query_builder = QueryBuilder::new("INSERT INTO ");

        query_builder.push(self.table);
        query_builder.push(" (");
        query_builder.push(self.columns.join(","));
        query_builder.push(") ");

        query_builder.push_values(
            items.iter().take(BIND_LIMIT / self.columns.len()),
            |b: sqlx::query_builder::Separated<'_, '_, Any, &str>, item| {
                self.item_binder.bind(item, b);
            },
        );

        let query = query_builder.build();

        let _result = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { query.execute(self.pool).await.unwrap() })
        });

        Ok(())
    }
}

#[derive(Default)]
pub struct RdbcItemWriterBuilder<'a, T> {
    pool: Option<&'a Pool<Any>>,
    table: Option<&'a str>,
    columns: Vec<&'a str>,
    item_binder: Option<&'a dyn RdbcItemBinder<T>>,
}

impl<'a, T> RdbcItemWriterBuilder<'a, T> {
    pub fn new() -> Self {
        Self {
            pool: None,
            table: None,
            columns: Vec::new(),
            item_binder: None,
        }
    }

    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    pub fn pool(mut self, pool: &'a Pool<Any>) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn item_binder(mut self, item_binder: &'a dyn RdbcItemBinder<T>) -> Self {
        self.item_binder = Some(item_binder);
        self
    }

    pub fn add_column(mut self, column: &'a str) -> Self {
        self.columns.push(column);
        self
    }

    pub fn build(self) -> RdbcItemWriter<'a, T> {
        if self.table.is_none() {
            panic!("Table name is mandatory");
        }

        if self.columns.is_empty() {
            panic!("One or more columns are required");
        }

        RdbcItemWriter::new(
            self.pool.unwrap(),
            self.table.unwrap(),
            self.columns.clone(),
            self.item_binder.unwrap(),
        )
    }
}
