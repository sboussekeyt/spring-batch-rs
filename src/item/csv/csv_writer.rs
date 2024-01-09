use std::{cell::RefCell, fs::File, io::Write, path::Path};

use csv::{Writer, WriterBuilder};
use serde::Serialize;

use crate::{core::item::ItemWriter, BatchError};

pub struct CsvItemWriter<T: Write> {
    writer: RefCell<Writer<T>>,
}

impl<T: Write, R: Serialize> ItemWriter<R> for CsvItemWriter<T> {
    fn write(&self, item: &R) -> Result<(), BatchError> {
        let result = self.writer.borrow_mut().serialize(item);
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
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
}

#[derive(Default)]
pub struct CsvItemWriterBuilder {
    delimiter: u8,
    has_headers: bool,
}

impl CsvItemWriterBuilder {
    pub fn new() -> CsvItemWriterBuilder {
        CsvItemWriterBuilder {
            delimiter: b',',
            has_headers: false,
        }
    }

    pub fn delimiter(mut self, delimiter: u8) -> CsvItemWriterBuilder {
        self.delimiter = delimiter;
        self
    }

    pub fn has_headers(mut self, yes: bool) -> CsvItemWriterBuilder {
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
    ///     wtr.write(&Row {
    ///         city: "Boston",
    ///         country: "United States",
    ///         population: 4628910,
    ///     });
    ///
    ///     wtr.write(&Row {
    ///         city: "Concord",
    ///         country: "United States",
    ///         population: 42695,
    ///     });
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

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, error::Error};

    use crate::{core::item::ItemWriter, item::csv::csv_writer::CsvItemWriterBuilder};

    #[derive(serde::Serialize)]
    struct Row<'a> {
        city: &'a str,
        country: &'a str,
        #[serde(rename = "popcount")]
        population: u64,
    }

    #[test]
    fn this_test_will_pass() -> Result<(), Box<dyn Error>> {
        let wtr = CsvItemWriterBuilder::new()
            .has_headers(true)
            .from_writer(vec![]);

        let resut = wtr.write(&Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        });
        assert!(resut.is_ok());

        let resut = wtr.write(&Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        });
        assert!(resut.is_ok());

        Ok(())
    }

    #[test]
    fn records_should_be_serialized() -> Result<(), Box<dyn Error>> {
        let wtr = CsvItemWriterBuilder::new()
            .has_headers(false)
            .from_path(temp_dir().join("foo.csv"));

        let resut = wtr.write(&Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        });
        assert!(resut.is_ok());

        let resut = wtr.write(&Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        });
        assert!(resut.is_ok());

        Ok(())
    }
}
