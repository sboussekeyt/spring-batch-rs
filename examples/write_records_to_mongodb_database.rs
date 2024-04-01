use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::Client,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        step::{Step, StepBuilder, StepInstance},
    },
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::mongodb::{mongodb_reader::WithObjectId, mongodb_writer::MongodbItemWriterBuilder},
};

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

    let book_collection = db.collection::<FormattedBook>("books");

    // Prepare reader
    let csv = "title,author
            Shining,Stephen King
            UN SAC DE BILLES,JOSEPH JOFFO";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // Prepare writer
    let writer = MongodbItemWriterBuilder::new()
        .collection(&book_collection)
        .build();

    // Execute process
    let step: StepInstance<FormattedBook, FormattedBook> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let _result = step.execute();

    Ok(())
}
