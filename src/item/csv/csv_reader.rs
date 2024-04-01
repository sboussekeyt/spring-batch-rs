use csv::{ReaderBuilder, StringRecordsIntoIter, Terminator, Trim};
use serde::de::DeserializeOwned;
use std::{cell::RefCell, fs::File, io::Read, path::Path};

use crate::{
    core::item::{ItemReader, ItemReaderResult},
    error::BatchError,
};

/// A CSV item reader that implements the `ItemReader` trait.
pub struct CsvItemReader<R> {
    records: RefCell<StringRecordsIntoIter<R>>,
}

impl<R: Read, T: DeserializeOwned> ItemReader<T> for CsvItemReader<R> {
    /// Reads the next item from the CSV file.
    ///
    /// Returns `Ok(Some(record))` if a record is successfully read,
    /// `Ok(None)` if there are no more records to read, and
    /// `Err(BatchError::ItemReader(error))` if an error occurs during reading.
    fn read(&self) -> ItemReaderResult<T> {
        if let Some(result) = self.records.borrow_mut().next() {
            match result {
                Ok(string_record) => {
                    let result: Result<T, _> = string_record.deserialize(None);

                    match result {
                        Ok(record) => Ok(Some(record)),
                        Err(error) => Err(BatchError::ItemReader(error.to_string())),
                    }
                }
                Err(error) => Err(BatchError::ItemReader(error.to_string())),
            }
        } else {
            Ok(None)
        }
    }
}

/// A builder for configuring CSV item reading.
#[derive(Default)]
pub struct CsvItemReaderBuilder {
    delimiter: u8,
    terminator: Terminator,
    has_headers: bool,
}

impl CsvItemReaderBuilder {
    /// Creates a new `CsvItemReaderBuilder` with default configuration.
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            terminator: Terminator::CRLF,
            has_headers: false,
        }
    }

    /// Sets the delimiter character for the CSV parsing.
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets the line terminator for the CSV parsing.
    pub fn terminator(mut self, terminator: Terminator) -> Self {
        self.terminator = terminator;
        self
    }

    /// Sets whether the CSV file has headers.
    pub fn has_headers(mut self, yes: bool) -> Self {
        self.has_headers = yes;
        self
    }

    /// Creates a `CsvItemReader` from a reader.
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

    /// Creates a `CsvItemReader` from a file path.
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

    use crate::item::csv::csv_reader::CsvItemReaderBuilder;

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
