use anyhow::Result;

use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::Client,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        step::{Step, StepBuilder, StepExecution},
    },
    item::csv::csv_writer::CsvItemWriterBuilder,
    item::mongodb::mongodb_reader::{MongodbItemReaderBuilder, WithObjectId},
};
use tempfile::NamedTempFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Book {
    #[serde(rename = "oid")]
    id: ObjectId,
    title: String,
    author: String,
}

impl WithObjectId for Book {
    fn get_id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct FormattedBook {
    title: String,
    author: String,
}

#[derive(Default)]
struct FormatBookProcessor;

impl ItemProcessor<Book, FormattedBook> for FormatBookProcessor {
    fn process(&self, item: &Book) -> ItemProcessorResult<FormattedBook> {
        let book = FormattedBook {
            title: item.title.replace(" ", "_").to_uppercase(),
            author: item.author.replace(" ", "_").to_uppercase(),
        };

        Ok(book)
    }
}

fn main() -> Result<()> {
    let url = format!("mongodb://127.0.0.1:27017/");

    let client: Client = Client::with_uri_str(&url).unwrap();

    let db = client.database("test");

    let book_collection = db.collection::<Book>("books");

    let filter = doc! {"title": {"$regex": "To Kill"}};

    // Prepare reader
    let reader = MongodbItemReaderBuilder::new()
        .collection(&book_collection)
        .filter(filter)
        .page_size(3)
        .build();

    // Prepare processor
    let processor = FormatBookProcessor::default();

    // Prepare writer
    let tmpfile = NamedTempFile::new()?;

    let writer = CsvItemWriterBuilder::<FormattedBook>::new().from_writer(tmpfile.as_file());

    let step = StepBuilder::new("read_from_mongodb")
        .chunk::<Book, FormattedBook>(3)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new("read_from_mongodb");
    let _result = step.execute(&mut step_execution);

    Ok(())
}
