use anyhow::Result;

use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::Client,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::ItemProcessor,
        step::{Step, StepBuilder},
    },
    item::mongodb::mongodb_reader::{MongodbItemReaderBuilder, WithObjectId},
    CsvItemWriterBuilder,
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
struct FormatBookProcessor {}

impl ItemProcessor<Book, FormattedBook> for FormatBookProcessor {
    fn process<'a>(&'a self, item: &'a Book) -> FormattedBook {
        let book = FormattedBook {
            title: item.title.replace(" ", "_").to_uppercase(),
            author: item.author.replace(" ", "_").to_uppercase(),
        };

        book
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

    let writer = CsvItemWriterBuilder::new().from_writer(tmpfile.as_file());

    let step: Step<Book, FormattedBook> = StepBuilder::new()
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .chunk(3)
        .build();

    step.execute();

    Ok(())
}
