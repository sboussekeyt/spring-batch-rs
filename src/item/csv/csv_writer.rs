use std::{fs::File, io, path::Path, result};

use csv::{IntoInnerError, Writer, WriterBuilder};

use crate::core::item::ItemWriter;

pub struct CsvItemWriter<T: io::Write> {
    wrapper: Box<Writer<T>>,
}

impl<T: io::Write, R: serde::Serialize> ItemWriter<R> for CsvItemWriter<T> {
    fn write(&mut self, item: &R) {
        self.wrapper.as_mut().serialize(item);
    }
}

impl<T: io::Write> CsvItemWriter<T> {
    pub fn into_inner(self) -> result::Result<T, IntoInnerError<Writer<T>>> {
        self.wrapper.into_inner()
    }

    /// Flush the contents of the internal buffer to the underlying writer.
    ///
    /// If there was a problem writing to the underlying writer, then an error
    /// is returned.
    ///
    /// Note that this also flushes the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.wrapper.as_mut().flush()?;
        Ok(())
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
            delimiter: b';',
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
        let wtr = WriterBuilder::new()
            .has_headers(self.has_headers)
            .from_path(path);

        CsvItemWriter {
            wrapper: Box::new(wtr.unwrap()),
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
    ///     let mut wtr = CsvItemWriterBuilder::new()
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
    ///     let data = String::from_utf8(wtr.into_inner()?)?;
    ///     assert_eq!(data, "\
    /// city,country,popcount
    /// Boston,United States,4628910
    /// Concord,United States,42695
    /// ");
    ///     Ok(())
    /// }
    /// ```
    pub fn from_writer<W: io::Write>(self, wtr: W) -> CsvItemWriter<W> {
        let wtr = WriterBuilder::new()
            .flexible(false)
            .has_headers(self.has_headers)
            .from_writer(wtr);

        CsvItemWriter {
            wrapper: Box::new(wtr),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

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
        let mut wtr = CsvItemWriterBuilder::new()
            .has_headers(true)
            .from_writer(vec![]);
        wtr.write(&Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        });
        wtr.write(&Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        });

        let data = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(
            data,
            "city,country,popcount
Boston,United States,4628910
Concord,United States,42695
"
        );

        Ok(())
    }

    #[test]
    fn records_should_be_serialized() -> Result<(), Box<dyn Error>> {
        let mut wtr = CsvItemWriterBuilder::new()
            .has_headers(false)
            .from_path("foo.csv");
        wtr.write(&Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        });
        wtr.write(&Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        });
        wtr.flush();

        Ok(())
    }
}
