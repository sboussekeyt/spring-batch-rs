use anyhow::Result;
use std::{io::Read, time::Instant};

use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::Client,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::ItemProcessor,
        step::{Step, StepBuilder, StepResult, StepStatus},
    },
    item::mongodb::mongodb_reader::{MongodbItemReaderBuilder, WithObjectId},
    CsvItemWriterBuilder,
};
use tempfile::NamedTempFile;
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{clients, core::Port, RunnableImage},
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
    fn process<'a>(&'a self, item: &'a Book) -> FormattedBook {
        let book = FormattedBook {
            title: item.title.replace(" ", "_").to_uppercase(),
            author: item.author.replace(" ", "_").to_uppercase(),
        };

        book
    }
}

#[test]
fn read_items_from_database() -> Result<()> {
    let docker = clients::Cli::default();

    let local_port = 27018;
    let port = Port {
        local: local_port,
        internal: 27017,
    };
    let mongo_image = RunnableImage::from(Mongo::default())
        .with_tag("latest")
        .with_mapped_port(port);
    let _node = docker.run(mongo_image);

    let url = format!("mongodb://127.0.0.1:{local_port}/");

    let client: Client = Client::with_uri_str(&url).unwrap();

    let db = client.database("test");

    let book_collection = db.collection::<Book>("books");

    let books = vec![
        Book {
            id: ObjectId::new(),
            title: "01 The Grapes of Wrath".to_string(),
            author: "Harper Steinbeck".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "02 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "03 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "04 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "05 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "06 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "07 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "08 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "09 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "10 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "11 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "12 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
        Book {
            id: ObjectId::new(),
            title: "13 To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
    ];

    let _ = book_collection.insert_many(books, None);

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

    let result: StepResult = step.execute();

    assert!(result.duration.as_nanos() > 0);
    assert!(result.start.le(&Instant::now()));
    assert!(result.end.le(&Instant::now()));
    assert!(result.start.le(&result.end));
    assert!(result.status == StepStatus::SUCCESS);
    assert!(result.read_count == 12);
    assert!(result.write_count == 12);
    assert!(result.read_error_count == 0);
    assert!(result.write_error_count == 0);

    let mut tmpfile = tmpfile.reopen()?;
    let mut file_content = String::new();

    tmpfile
        .read_to_string(&mut file_content)
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        "02_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
03_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
04_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
05_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
06_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
07_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
08_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
09_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
10_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
11_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
12_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
13_TO_KILL_A_MOCKINGBIRD,HARPER_LEE
"
    );

    Ok(())
}
