use std::{env::temp_dir, fs::File, path::Path};

use ::serde::{ser::Error, Deserialize, Serialize};
use serde::Serializer;
use spring_batch_rs::{
    core::{
        item::ItemProcessor,
        step::{Step, StepBuilder},
    },
    CsvItemWriterBuilder, JsonItemReaderBuilder,
};

use time::{format_description, Date, Month};

fn date_serializer<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let result = format_description::parse("[year]-[month]-[day]");

    match result {
        Ok(format) => {
            let s = date.format(&format).unwrap();
            serializer.serialize_str(&s)
        }
        Err(error) => Err(Error::custom(error.to_string())),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Person {
    first_name: String,
    last_name: String,
    title: String,
    email: String,
    #[serde(serialize_with = "date_serializer")]
    birth_date: Date,
}

#[derive(Default)]
struct UpperCaseProcessor {}

impl ItemProcessor<Person, Person> for UpperCaseProcessor {
    fn process<'a>(&'a self, item: &'a Person) -> Person {
        let person = Person {
            first_name: item.first_name.to_uppercase(),
            last_name: item.last_name.to_uppercase(),
            title: item.title.to_uppercase(),
            email: item.email.to_uppercase(),
            birth_date: Date::from_calendar_date(2019, Month::January, 1).unwrap(),
        };

        person
    }
}

fn main() -> std::io::Result<()> {
    let path = Path::new("examples/data/persons.json");

    let file = File::open(path).expect("Unable to open file");

    let reader = JsonItemReaderBuilder::new().from_reader(file);

    let processor = UpperCaseProcessor::default();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path(temp_dir().join("persons.csv"));

    let step: Step<Person, Person> = StepBuilder::new()
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .chunk(2)
        .build();

    step.execute();

    Ok(())
}
