use std::fmt;

use fake::locales::*;
use fake::{faker::name::raw::*, Fake};
use log::debug;
use time::{Date, Month};

use crate::{core::item::ItemReader, error::BatchError};

pub struct Person {
    first_name: String,
    last_name: String,
    birth_date: Date,
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
    count: usize,
}

impl ItemReader<Person> for PersonReader {
    fn read(&mut self) -> Option<Result<Person, BatchError>> {
        if self.count == 0 {
            return None;
        }

        self.count -= 1;

        let person = Person {
            first_name: FirstName(FR_FR).fake(),
            last_name: LastName(FR_FR).fake(),
            birth_date: Date::from_calendar_date(2022, Month::January, 2).unwrap(),
        };
        debug!("Person: {}", person.to_string());
        Some(Ok(person))
    }
}

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
            count: self.number_of_items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PersonReaderBuilder, PersonReader};
    use crate::{core::item::ItemReader};

    #[test]
    fn this_test_will_pass() {
        let mut reader: PersonReader = PersonReaderBuilder::new().number_of_items(2).build();
        assert!(reader.count == 2);

        let result1 = reader.read();
        assert!(reader.count == 1);
        assert!(result1.is_some());

        let person = result1.unwrap();
        assert!(person.is_ok());
        assert!(!person.as_ref().unwrap().first_name.is_empty());
        assert!(!person.as_ref().unwrap().last_name.is_empty());

        let result2 = reader.read();
        assert!(reader.count == 0);
        assert!(result2.is_some());
        assert!(result2.unwrap().is_ok());
        
        let result3 = reader.read();
        assert!(reader.count == 0);
        assert!(result3.is_none());
    }
}