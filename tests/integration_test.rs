use std::{
    env::temp_dir,
    fs::{self, File},
    path::Path,
    time::Instant,
};

use ::serde::{ser::Error, Deserialize, Serialize};
use serde::Serializer;

use spring_batch_rs::{
    core::{
        item::ItemProcessor,
        step::{Step, StepBuilder, StepResult, StepStatus},
    },
    CsvItemReaderBuilder, CsvItemWriterBuilder, JsonItemReaderBuilder, JsonItemWriterBuilder,
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

#[test]
fn transform_from_json_file_to_csv_file_without_error() {
    let path = Path::new("examples/data/persons.json");

    let file = File::options()
        .read(true)
        .open(path)
        .expect("Unable to open file");

    let reader = JsonItemReaderBuilder::new().from_reader(file);

    let processor = UpperCaseProcessor::default();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path(temp_dir().join("persons.csv"));

    let step: Step<Person, Person> = StepBuilder::new()
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
    assert!(result.read_count == 4);
    assert!(result.write_count == 4);
    assert!(result.read_error_count == 0);
    assert!(result.write_error_count == 0);

    let file_content = fs::read_to_string(temp_dir().join("persons.csv"))
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        "first_name,last_name,title,email,birth_date
OCÉANE,DUPOND,MR.,LEOPOLD_ENIM@ORANGE.FR,2019-01-01
AMANDINE,ÉVRAT,MRS.,AMANDINE_IURE@OUTLOOK.FR,2019-01-01
UGO,NIELS,SIR.,XAVIER_VOLUPTATEM@SFR.FR,2019-01-01
LÉO,ZOLA,DR.,UGO_PRAESENTIUM@ORANGE.FR,2019-01-01
"
    );
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Car {
    year: u16,
    make: String,
    model: String,
    description: String,
}

#[test]
fn convert_csv_file_to_json_file_without_error() {
    let path = Path::new("examples/data/cars_with_headers.csv");

    let file = File::options()
        .read(true)
        .open(path)
        .expect("Unable to open file");

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join("cars.json"));

    let step: Step<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let result: StepResult = step.execute();

    assert!(result.duration.as_nanos() > 0);
    assert!(result.start.le(&Instant::now()));
    assert!(result.end.le(&Instant::now()));
    assert!(result.start.le(&result.end));
    assert!(result.status == StepStatus::SUCCESS);
    assert!(result.read_count == 7);
    assert!(result.write_count == 7);
    assert!(result.read_error_count == 0);
    assert!(result.write_error_count == 0);

    let file_content = fs::read_to_string(temp_dir().join("cars.json"))
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        r#"[{"year":1948,"make":"Porsche","model":"356","description":"Luxury sports car 1"},{"year":1949,"make":"Porsche","model":"357","description":"Luxury sports car 2"},{"year":1950,"make":"Porsche","model":"358","description":"Luxury sports car 3"},{"year":1951,"make":"Porsche","model":"359","description":"Luxury sports car 4"},{"year":1952,"make":"Porsche","model":"360","description":"Luxury sports car 5"},{"year":1967,"make":"Ford","model":"Mustang fastback 1967","description":"American car"},{"year":1967,"make":"Ford","model":"Mustang fastback 1967","description":"American car"}]
"#
    );
}
