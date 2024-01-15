use csv::{ReaderBuilder, StringRecordsIntoIter, Terminator, Trim};
use serde::de::DeserializeOwned;
use std::{cell::RefCell, fs::File, io::Read, path::Path};

use crate::{core::item::ItemReader, error::BatchError};

pub struct CsvItemReader<R> {
    records: RefCell<StringRecordsIntoIter<R>>,
}

impl<R: Read, T: DeserializeOwned> ItemReader<T> for CsvItemReader<R> {
    fn read(&self) -> Option<Result<T, BatchError>> {
        if let Some(result) = self.records.borrow_mut().next() {
            match result {
                Ok(string_record) => {
                    let result: Result<T, _> = string_record.deserialize(None);

                    match result {
                        Ok(record) => Some(Ok(record)),
                        Err(error) => Some(Err(BatchError::ItemReader(error.to_string()))),
                    }
                }
                Err(error) => Some(Err(BatchError::ItemReader(error.to_string()))),
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct CsvItemReaderBuilder {
    delimiter: u8,
    terminator: Terminator,
    has_headers: bool,
}

/// Create a new builder for configuring CSV parsing.
///
/// To convert a builder into a reader, call one of the methods starting
/// with `from_`.
///
/// # Example
///
/// ```
/// use std::error::Error;
/// use csv::{ReaderBuilder, StringRecord};
///
/// # fn main() { example().unwrap(); }
/// fn example() -> Result<(), Box<dyn Error>> {
///     let data = "\
/// city,country,pop
/// Boston,United States,4628910
/// Concord,United States,42695
/// ";
///     let mut rdr = ReaderBuilder::new().from_reader(data.as_bytes());
///
///     let records = rdr
///         .records()
///         .collect::<Result<Vec<StringRecord>, csv::Error>>()?;
///     assert_eq!(records, vec![
///         vec!["Boston", "United States", "4628910"],
///         vec!["Concord", "United States", "42695"],
///     ]);
///     Ok(())
/// }
/// ```
impl CsvItemReaderBuilder {
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            terminator: Terminator::CRLF,
            has_headers: false,
        }
    }

    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn terminator(mut self, terminator: Terminator) -> Self {
        self.terminator = terminator;
        self
    }

    pub fn has_headers(mut self, yes: bool) -> Self {
        self.has_headers = yes;
        self
    }

    pub fn from_reader<R: Read>(self, rdr: R) -> CsvItemReader<R> {
        let rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .delimiter(self.delimiter)
            .terminator(self.terminator)
            .has_headers(self.has_headers)
            .flexible(false)
            .from_reader(rdr);

        let records = rdr.into_records();

        CsvItemReader {
            records: RefCell::new(records),
        }
    }

    pub fn from_path<R: AsRef<Path>>(self, path: R) -> CsvItemReader<File> {
        let rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .delimiter(self.delimiter)
            .terminator(self.terminator)
            .has_headers(self.has_headers)
            .flexible(false)
            .from_path(path);

        let records = rdr.unwrap().into_records();

        CsvItemReader {
            records: RefCell::new(records),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use csv::StringRecord;

    use crate::CsvItemReaderBuilder;

    #[test]
    fn this_test_will_pass() -> Result<(), Box<dyn Error>> {
        let data = "city,country,pop
        Boston,United States,4628910
        Concord,United States,42695";

        let reader = CsvItemReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_reader(data.as_bytes());

        let records = reader
            .records
            .into_inner()
            .collect::<Result<Vec<StringRecord>, csv::Error>>()?;

        assert_eq!(
            records,
            vec![
                vec!["Boston", "United States", "4628910"],
                vec!["Concord", "United States", "42695"],
            ]
        );

        Ok(())
    }
}
