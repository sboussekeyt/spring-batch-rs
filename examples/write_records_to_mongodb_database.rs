use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::Client,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepExecution},
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

fn main() -> Result<()> {
    let url = "mongodb://127.0.0.1:27017/".to_string();

    let client: Client = Client::with_uri_str(&url).unwrap();

    let db = client.database("test");

    let book_collection = db.collection::<FormattedBook>("books");

    // Prepare reader
    let csv = "title,author
            Shining,Stephen King
            UN SAC DE BILLES,JOSEPH JOFFO";

    let reader = CsvItemReaderBuilder::<FormattedBook>::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // Prepare writer
    let writer = MongodbItemWriterBuilder::new()
        .collection(&book_collection)
        .build();

    // Execute process
    let step = StepBuilder::new("write_to_mongodb")
        .chunk::<FormattedBook, FormattedBook>(3)
        .reader(&reader)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new("write_to_mongodb");

    let _result = step.execute(&mut step_execution);

    Ok(())
}
