use std::io;

use serde::de::DeserializeOwned;
use serde_json::{de::IoRead, Deserializer, StreamDeserializer};

use crate::{core::item::ItemReader, BatchError};

pub struct JsonItemReader<'a, R: std::io::Read, T> {
    stream: StreamDeserializer<'a, IoRead<R>, T>,
}

impl<R: io::Read, T: DeserializeOwned> ItemReader<T> for JsonItemReader<'_, R, T> {
    fn read(&mut self) -> Option<Result<T, BatchError>> {
        if let Some(result) = self.stream.next() {
            match result {
                Ok(record) => Some(Ok(record)),
                Err(error) => Some(Err(BatchError::ItemReader(error.to_string()))),
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct JsonItemReaderBuilder {}

impl JsonItemReaderBuilder {
    pub fn new() -> JsonItemReaderBuilder {
        JsonItemReaderBuilder {}
    }

    pub fn from_reader<'a, R: io::Read, T: DeserializeOwned>(
        self,
        rdr: R,
    ) -> JsonItemReader<'a, R, T> {        
        let stream = Deserializer::from_reader(rdr).into_iter();

        JsonItemReader { stream }
    }
}
