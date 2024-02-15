use std::{
    cell::{Cell, RefCell},
    io::{BufRead, BufReader, Read},
    marker::PhantomData,
};

use log::debug;
use serde::de::DeserializeOwned;

use crate::{
    core::item::{ItemReader, ItemReaderResult},
    BatchError,
};

#[derive(Debug)]
enum JsonParserResult {
    NotEnded,
    ParsingError { error: serde_json::Error },
}

pub struct JsonItemReader<R, T> {
    pd: PhantomData<T>,
    reader: RefCell<BufReader<R>>,
    capacity: usize,
    level: Cell<u16>,
    index: Cell<usize>,
    object: RefCell<Vec<u8>>,
}

impl<R: Read, T: DeserializeOwned> JsonItemReader<R, T> {
    fn new(rdr: R, capacity: usize) -> Self {
        let buf_reader = BufReader::with_capacity(capacity, rdr);

        Self {
            pd: PhantomData,
            reader: RefCell::new(buf_reader),
            capacity,
            level: Cell::new(0),
            index: Cell::new(0),
            object: RefCell::new(Vec::new()),
        }
    }

    fn get_current_char(&self, buffer: &[u8]) -> u8 {
        buffer[self.index.get()]
    }

    fn is_new_seq(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b'['
    }

    fn is_end_seq(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b']'
    }

    fn is_new_object(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b'{'
    }

    fn is_end_object(&self, buffer: &[u8]) -> bool {
        self.level == 1.into() && self.get_current_char(buffer) == b'}'
    }

    fn start_new(&self) {
        self.object.borrow_mut().clear();
    }

    fn append_char(&self, buffer: &[u8]) {
        let current_char = self.get_current_char(buffer);
        if current_char != b' ' && current_char != b'\n' {
            self.object.borrow_mut().push(self.get_current_char(buffer));
        }
    }

    fn clear_buff(&self) {
        self.index.set(0);
    }

    fn level_inc(&self) {
        self.level.set(self.level.get() + 1);
    }

    fn level_dec(&self) {
        self.level.set(self.level.get() - 1);
    }

    fn index_inc(&self) {
        self.index.set(self.index.get() + 1);
    }

    fn next(&self, buffer: &[u8]) -> Result<T, JsonParserResult> {
        while self.index.get() < buffer.len() - 1 && !self.is_end_seq(buffer) {
            if self.is_new_object(buffer) {
                self.start_new();
            } else if self.is_new_seq(buffer) {
                self.index_inc();
                continue;
            }

            let current_char = self.get_current_char(buffer);

            if current_char == b'{' {
                self.level_inc();
            } else if current_char == b'}' {
                self.level_dec();
            }

            self.append_char(buffer);

            self.index_inc();

            if self.is_end_object(buffer) {
                self.append_char(buffer);

                let result = serde_json::from_slice(self.object.borrow_mut().as_slice());
                debug!(
                    "object ok: {}",
                    std::str::from_utf8(self.object.borrow().as_slice()).unwrap()
                );
                return match result {
                    Ok(record) => Ok(record),
                    Err(error) => Err(JsonParserResult::ParsingError { error }),
                };
            }
        }

        self.append_char(buffer);
        Err(JsonParserResult::NotEnded)
    }
}

impl<R: Read, T: DeserializeOwned> ItemReader<T> for JsonItemReader<R, T> {
    fn read(&self) -> ItemReaderResult<T> {
        let mut buf_reader = self.reader.borrow_mut();

        loop {
            let buffer = &mut buf_reader.fill_buf().unwrap();

            let buffer_length = buffer.len();

            if buffer_length == 0 {
                return Ok(None);
            }

            let result: Result<T, JsonParserResult> = self.next(buffer);

            if let Ok(record) = result {
                return Ok(Some(record));
            } else if let Err(error) = result {
                match error {
                    JsonParserResult::NotEnded => {
                        self.clear_buff();
                        buf_reader.consume(self.capacity)
                    }
                    JsonParserResult::ParsingError { error } => {
                        return Err(BatchError::ItemReader(error.to_string()))
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct JsonItemReaderBuilder<T> {
    _pd: PhantomData<T>,
    capacity: Option<usize>,
}

impl<T: DeserializeOwned> JsonItemReaderBuilder<T> {
    pub fn new() -> JsonItemReaderBuilder<T> {
        Self {
            _pd: PhantomData,
            capacity: Some(8 * 1024),
        }
    }

    pub fn capacity(mut self, capacity: usize) -> JsonItemReaderBuilder<T> {
        self.capacity = Some(capacity);
        self
    }

    pub fn from_reader<R: Read>(self, rdr: R) -> JsonItemReader<R, T> {
        JsonItemReader::new(rdr, self.capacity.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::File, io::Cursor, path::Path};

    use crate::{
        core::item::{ItemReader, ItemReaderResult},
        item::{fake::person_reader::Person, json::json_reader::JsonItemReaderBuilder},
    };

    #[test]
    fn content_from_file_should_be_deserialized() -> Result<(), Box<dyn Error>> {
        let path = Path::new("examples/data/persons.json");

        let file = File::options()
            .append(true)
            .read(true)
            .create(false)
            .open(path)
            .expect("Unable to open file");

        let reader = JsonItemReaderBuilder::new().capacity(320).from_reader(file);

        let result: ItemReaderResult<Person> = reader.read();

        assert!(result.is_ok());
        assert_eq!(
            "first_name:Océane, last_name:Dupond, birth_date:1963-05-16",
            result.unwrap().unwrap().to_string()
        );

        let result: ItemReaderResult<Person> = reader.read();
        assert!(result.is_ok());
        assert_eq!(
            "first_name:Amandine, last_name:Évrat, birth_date:1933-07-12",
            result.unwrap().unwrap().to_string()
        );

        let result: ItemReaderResult<Person> = reader.read();
        assert!(result.is_ok());
        assert_eq!(
            "first_name:Ugo, last_name:Niels, birth_date:1980-04-05",
            result.unwrap().unwrap().to_string()
        );

        let result: ItemReaderResult<Person> = reader.read();
        assert!(result.is_ok());
        assert_eq!(
            "first_name:Léo, last_name:Zola, birth_date:1914-08-13",
            result.unwrap().unwrap().to_string()
        );

        Ok(())
    }

    #[test]
    fn content_from_bytes_should_be_deserialized() -> Result<(), Box<dyn Error>> {
        let input = Cursor::new(String::from("foo\nbar\nbaz\n"));

        let reader = JsonItemReaderBuilder::new()
            .capacity(320)
            .from_reader(input);

        let result: ItemReaderResult<Person> = reader.read();

        assert!(result.is_ok());

        Ok(())
    }
}
