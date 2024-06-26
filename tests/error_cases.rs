mod common;

use common::MockFile;

use ::serde::ser::Error;

use rand::distributions::{Alphanumeric, DistString};

use serde::{Deserialize, Serialize, Serializer};
use std::{
    env::temp_dir,
    fs::{self},
    io::{self, ErrorKind},
};

use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::{Step, StepBuilder, StepInstance},
    },
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::json::json_writer::JsonItemWriterBuilder,
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
struct UpperCaseProcessor;

impl ItemProcessor<Person, Person> for UpperCaseProcessor {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        let person = Person {
            first_name: item.first_name.to_uppercase(),
            last_name: item.last_name.to_uppercase(),
            title: item.title.to_uppercase(),
            email: item.email.to_uppercase(),
            birth_date: Date::from_calendar_date(2019, Month::January, 1).unwrap(),
        };

        Ok(person)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Car {
    year: u16,
    make: String,
    model: String,
    description: String,
}

#[test]
fn transform_csv_stream_to_json_file_with_error_at_first() {
    let csv = "year,make,model,description
    1948d,Porsche,356,Luxury sports car
    2011,Peugeot,206+,City car
    2012,Citroën,C4 Picasso,SUV
    2021,Mazda,CX-30,SUV Compact
    1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    let file_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join(file_name.clone()));

    let step: StepInstance<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();
    assert!(result.is_err());
    assert!(step.get_read_count() == 0);
    assert!(step.get_write_count() == 0);
    assert!(step.get_read_error_count() == 1);
    assert!(step.get_write_error_count() == 0);

    let file_content = fs::read_to_string(temp_dir().join(file_name))
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        r#"[]
"#
    );
}

#[test]
fn transform_csv_stream_to_json_file_with_error_at_end() {
    let csv = "year,make,model,description
    1948,Porsche,356,Luxury sports car
    2011,Peugeot,206+,City car
    2012,Citroën,C4 Picasso,SUV
    2021,Mazda,CX-30,SUV Compact
    1967d,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    let file_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let writer = JsonItemWriterBuilder::new().from_path(temp_dir().join(file_name.clone()));

    let step: StepInstance<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(3)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();
    assert!(result.is_err());
    assert!(step.get_read_count() == 4);
    assert!(step.get_write_count() == 3);
    assert!(step.get_read_error_count() == 1);
    assert!(step.get_write_error_count() == 0);

    let file_content = fs::read_to_string(temp_dir().join(file_name))
        .expect("Should have been able to read the file");

    assert_eq!(
        file_content,
        r#"[{"year":1948,"make":"Porsche","model":"356","description":"Luxury sports car"},{"year":2011,"make":"Peugeot","model":"206+","description":"City car"},{"year":2012,"make":"Citroën","model":"C4 Picasso","description":"SUV"}]
"#
    );
}

#[test]
fn transform_csv_stream_to_writer_with_error() {
    let csv = "year,make,model,description
    1948,Porsche,356,Luxury sports car
    2011,Peugeot,206+,City car
    2012,Citroën,C4 Picasso,SUV
    2021,Mazda,CX-30,SUV Compact
    1967,Ford,Mustang fastback 1967,American car";

    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    let mut file = MockFile::default();
    file.expect_write().times(3).returning(|_buf| {
        let err = io::Error::from(ErrorKind::PermissionDenied);
        Result::Err(err)
    });

    let writer = JsonItemWriterBuilder::new().from_writer(file);

    let step: StepInstance<Car, Car> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(1)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();
    assert!(result.is_err());
    assert!(step.get_read_count() == 1);
    assert!(step.get_write_count() == 0);
    assert!(step.get_read_error_count() == 0);
    assert!(step.get_write_error_count() == 1);
}
