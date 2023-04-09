use std::cell::Cell;
use std::fmt;

use ::serde::{ser::Error, Deserialize, Serialize, Serializer};
use fake::faker::internet::raw::*;
use fake::locales::*;
use fake::{faker::name::raw::*, Fake};
use log::debug;
use rand::Rng;

use time::format_description;
use time::{Date, Month};

use crate::{core::item::ItemReader, error::BatchError};

#[derive(Serialize, Deserialize)]
pub struct Person {
    first_name: String,
    last_name: String,
    title: String,
    email: String,
    #[serde(serialize_with = "date_serializer")]
    birth_date: Date,
}

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

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "first_name:{}, last_name:{}, birth_date:{}",
            self.first_name, self.last_name, self.birth_date
        )
    }
}

pub struct PersonReader {
    count: Cell<usize>,
}

impl ItemReader<Person> for PersonReader {
    fn read(&self) -> Option<Result<Person, BatchError>> {
        if self.count.get() == 0 {
            return None;
        }

        self.count.set(self.count.get() - 1);

        let person = Person {
            first_name: FirstName(FR_FR).fake(),
            last_name: LastName(FR_FR).fake(),
            title: Title(FR_FR).fake(),
            email: FreeEmail(FR_FR).fake(),
            birth_date: fake_date(),
        };
        debug!("Person: {}", person.to_string());
        Some(Ok(person))
    }
}

fn fake_date() -> Date {
    let mut rng = rand::thread_rng();
    let year = rng.gen_range(1900..2022);
    let month = rng.gen_range(1..12);
    let day = rng.gen_range(1..28);

    Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap()
}

#[derive(Default)]
pub struct PersonReaderBuilder {
    number_of_items: usize,
}

impl PersonReaderBuilder {
    pub fn new() -> PersonReaderBuilder {
        PersonReaderBuilder { number_of_items: 0 }
    }

    pub fn number_of_items(mut self, number_of_items: usize) -> PersonReaderBuilder {
        self.number_of_items = number_of_items;
        self
    }

    pub fn build(self) -> PersonReader {
        PersonReader {
            count: self.number_of_items.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PersonReader, PersonReaderBuilder};
    use crate::core::item::ItemReader;

    #[test]
    fn this_test_will_pass() {
        let reader: PersonReader = PersonReaderBuilder::new().number_of_items(2).build();
        assert_eq!(reader.count.get(), 2);

        let result1 = reader.read();
        assert_eq!(reader.count.get(), 1);
        assert_eq!(result1.is_some(), true);

        let person = result1.unwrap();
        assert_eq!(person.is_ok(), true);
        assert_eq!(person.as_ref().unwrap().first_name.is_empty(), false);
        assert_eq!(person.as_ref().unwrap().last_name.is_empty(), false);

        let result2 = reader.read();
        assert_eq!(reader.count.get(), 0);
        assert_eq!(result2.is_some(), true);
        assert_eq!(result2.unwrap().is_ok(), true);

        let result3 = reader.read();
        assert_eq!(reader.count.get(), 0);
        assert_eq!(result3.is_none(), true);
    }
}
