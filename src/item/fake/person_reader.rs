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

use crate::core::item::ItemReader;
use crate::core::item::ItemReaderResult;

/// Represents a person with their personal information.
#[derive(Serialize, Deserialize, Clone)]
pub struct Person {
    first_name: String,
    last_name: String,
    title: String,
    email: String,
    #[serde(serialize_with = "date_serializer")]
    birth_date: Date,
}

/// Serializes a `Date` object into a string representation.
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

/// A reader for generating fake `Person` objects.
pub struct PersonReader {
    count: Cell<usize>,
}

impl ItemReader<Person> for PersonReader {
    /// Reads the next `Person` object.
    ///
    /// Returns `Ok(Some(person))` if there are more `Person` objects to read.
    /// Returns `Ok(None)` if there are no more `Person` objects to read.
    fn read(&self) -> ItemReaderResult<Person> {
        if self.count.get() == 0 {
            return Ok(None);
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
        Ok(Some(person))
    }
}

/// Generates a random `Date` object.
fn fake_date() -> Date {
    let mut rng = rand::rng();
    let year = rng.random_range(1900..2022);
    let month = rng.random_range(1..12);
    let day = rng.random_range(1..28);

    Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap()
}

/// Builder for creating a `PersonReader`.
#[derive(Default)]
pub struct PersonReaderBuilder {
    number_of_items: usize,
}

impl PersonReaderBuilder {
    /// Creates a new `PersonReaderBuilder` instance.
    pub fn new() -> Self {
        Self { number_of_items: 0 }
    }

    /// Sets the number of `Person` objects to generate.
    pub fn number_of_items(mut self, number_of_items: usize) -> Self {
        self.number_of_items = number_of_items;
        self
    }

    /// Builds a `PersonReader` instance with the configured settings.
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
        assert_eq!(result1.is_ok(), true);

        let person = result1.unwrap();
        assert_eq!(person.is_some(), true);
        assert_eq!(person.as_ref().unwrap().first_name.is_empty(), false);
        assert_eq!(person.as_ref().unwrap().last_name.is_empty(), false);

        let result2 = reader.read();
        assert_eq!(reader.count.get(), 0);
        assert_eq!(result2.is_ok(), true);
        assert_eq!(result2.unwrap().is_some(), true);

        let result3 = reader.read();
        assert_eq!(reader.count.get(), 0);
        assert_eq!(result3.unwrap().is_none(), true);
    }
}
