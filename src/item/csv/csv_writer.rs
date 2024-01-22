use std::{cell::RefCell, fs::File, io::Write, path::Path};

use csv::{Writer, WriterBuilder};
use serde::Serialize;

use crate::{core::item::ItemWriter, BatchError};

pub struct CsvItemWriter<T: Write> {
    writer: RefCell<Writer<T>>,
}

impl<T: Write, R: Serialize> ItemWriter<R> for CsvItemWriter<T> {
    fn write(&self, items: &[R]) -> Result<(), BatchError> {
        for item in items.iter() {
            let result = self.writer.borrow_mut().serialize(item);

            if result.is_err() {
                let error = result.err().unwrap();
                return Err(BatchError::ItemWriter(error.to_string()));
            }
        }
        Ok(())
    }

    /// Flush the contents of the internal buffer to the underlying writer.
    ///
    /// If there was a problem writing to the underlying writer, then an error
    /// is returned.
    ///
    /// Note that this also flushes the underlying writer.
    fn flush(&self) -> Result<(), BatchError> {
        let result = self.writer.borrow_mut().flush();
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn open(&self) -> Result<(), BatchError> {
        Ok(())
    }

    fn close(&self) -> Result<(), BatchError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct CsvItemWriterBuilder {
    delimiter: u8,
    has_headers: bool,
}

impl CsvItemWriterBuilder {
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            has_headers: false,
        }
    }

    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_headers(mut self, yes: bool) -> Self {
        self.has_headers = yes;
        self
    }

    pub fn from_path<R: AsRef<Path>>(self, path: R) -> CsvItemWriter<File> {
        let writer = WriterBuilder::new()
            .has_headers(self.has_headers)
            .from_path(path);

        CsvItemWriter {
            writer: RefCell::new(writer.unwrap()),
        }
    }

    /// Serialize a single record using Serde.
    ///
    /// # Example
    ///
    /// This shows how to serialize normal Rust structs as CSV records. The
    /// fields of the struct are used to write a header row automatically.
    /// (Writing the header row automatically can be disabled by building the
    /// CSV writer with a [`WriterBuilder`](struct.WriterBuilder.html) and
    /// calling the `has_headers` method.)
    ///
    /// ```
    /// # use std::error::Error;
    /// # use csv::Writer;
    /// # use spring_batch_rs::{item::csv::csv_writer::CsvItemWriterBuilder, core::item::ItemWriter};
    /// #[derive(serde::Serialize)]
    /// struct Row<'a> {
    ///     city: &'a str,
    ///     country: &'a str,
    ///     #[serde(rename = "popcount")]
    ///     population: u64,
    /// }
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let wtr = CsvItemWriterBuilder::new()
    ///         .has_headers(true)
    ///         .from_writer(vec![]);
    ///
    ///     let rows = &[
    ///         Row {
    ///             city: "Boston",
    ///             country: "United States",
    ///             population: 4628910,
    ///         },
    ///         Row {
    ///             city: "Concord",
    ///             country: "United States",
    ///             population: 42695,
    ///         }
    ///     ];
    ///     wtr.write(rows);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_writer<W: Write>(self, wtr: W) -> CsvItemWriter<W> {
        let wtr = WriterBuilder::new()
            .flexible(false)
            .has_headers(self.has_headers)
            .from_writer(wtr);

        CsvItemWriter {
            writer: RefCell::new(wtr),
        }
    }
}
