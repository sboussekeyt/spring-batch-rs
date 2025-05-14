use csv::{ReaderBuilder, StringRecordsIntoIter, Terminator, Trim};
use serde::de::DeserializeOwned;
use std::{cell::RefCell, fs::File, io::Read, path::Path};

use crate::{
    core::item::{ItemReader, ItemReaderResult},
    error::BatchError,
};

/// A CSV item reader that implements the `ItemReader` trait.
///
/// This reader deserializes CSV data into Rust structs row by row
/// using Serde's deserialization capabilities. It can process CSV
/// data from files, strings, or any source implementing the `Read` trait.
///
/// # Type Parameters
///
/// - `R`: The type of reader providing the CSV data. Must implement `Read`.
///
/// # Implementation Details
///
/// - Uses a `RefCell` to provide interior mutability for the CSV record iterator
/// - Requires `DeserializeOwned` for types that can be deserialized from CSV rows
/// - Automatically converts CSV parsing errors into Spring Batch errors
/// - Allows streaming data processing without loading the entire file into memory
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct Record {
///     name: String,
///     value: i32,
/// }
///
/// // Create a CSV string
/// let data = "\
/// name,value
/// foo,123
/// bar,456
/// ";
///
/// // Build a reader
/// let reader = CsvItemReaderBuilder::new()
///     .has_headers(true)
///     .from_reader(data.as_bytes());
///
/// // Read the first record
/// let record: Record = reader.read().unwrap().unwrap();
/// assert_eq!(record.name, "foo");
/// assert_eq!(record.value, 123);
///
/// // Read the second record
/// let record: Record = reader.read().unwrap().unwrap();
/// assert_eq!(record.name, "bar");
/// assert_eq!(record.value, 456);
///
/// // No more records - explicitly use Record type again
/// assert!(ItemReader::<Record>::read(&reader).unwrap().is_none());
/// ```
pub struct CsvItemReader<R> {
    /// Iterator over the CSV records
    ///
    /// Uses `RefCell` to provide interior mutability so we can iterate
    /// through records while keeping the `read` method signature compatible
    /// with the `ItemReader` trait.
    records: RefCell<StringRecordsIntoIter<R>>,
}

impl<R: Read, T: DeserializeOwned> ItemReader<T> for CsvItemReader<R> {
    /// Reads the next item from the CSV file.
    ///
    /// This method reads and deserializes the next row from the CSV source.
    /// The row is converted to the specified type `T` using Serde's deserialization.
    ///
    /// # Deserialization Process
    ///
    /// 1. Gets the next record from the CSV iterator
    /// 2. If no more records, returns `Ok(None)`
    /// 3. Deserializes the record to type `T` using serde
    /// 4. Wraps errors in the Spring Batch error system
    ///
    /// # Returns
    /// - `Ok(Some(record))` if a record is successfully read
    /// - `Ok(None)` if there are no more records to read
    /// - `Err(BatchError::ItemReader(error))` if an error occurs during reading or deserialization
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Person {
    ///     name: String,
    ///     age: u8,
    /// }
    ///
    /// let data = "name,age\nAlice,30\nBob,25";
    /// let reader = CsvItemReaderBuilder::new()
    ///     .has_headers(true)
    ///     .from_reader(data.as_bytes());
    ///
    /// // Read all people
    /// let mut people: Vec<Person> = Vec::new();
    /// while let Some(person) = reader.read().unwrap() {
    ///     people.push(person);
    /// }
    ///
    /// assert_eq!(people.len(), 2);
    /// assert_eq!(people[0].name, "Alice");
    /// assert_eq!(people[0].age, 30);
    /// ```
    fn read(&self) -> ItemReaderResult<T> {
        // Try to get the next CSV record from the iterator
        if let Some(result) = self.records.borrow_mut().next() {
            match result {
                Ok(string_record) => {
                    // Attempt to deserialize the record to type T
                    let result: Result<T, _> = string_record.deserialize(None);

                    match result {
                        Ok(record) => Ok(Some(record)),
                        Err(error) => Err(BatchError::ItemReader(error.to_string())),
                    }
                }
                Err(error) => Err(BatchError::ItemReader(error.to_string())),
            }
        } else {
            // No more records in the CSV file
            Ok(None)
        }
    }
}

/// A builder for configuring CSV item reading.
///
/// This builder allows you to customize the CSV reading behavior,
/// including delimiter, terminator, and header handling.
///
/// # Design Pattern
///
/// This struct implements the Builder pattern, which allows for fluent, chainable
/// configuration of a `CsvItemReader` before creation. Each method returns `self`
/// to allow method chaining.
///
/// # Default Configuration
///
/// - Delimiter: comma (,)
/// - Terminator: CRLF (Windows-style line endings)
/// - Headers: disabled
/// - Trimming: All fields trimmed
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use csv::Terminator;
///
/// // Custom CSV configuration
/// let reader = CsvItemReaderBuilder::new()
///     .delimiter(b';')  // Use semicolon as delimiter
///     .terminator(Terminator::Any(b'\n'))  // Unix line endings
///     .has_headers(true)  // First row contains headers
///     .from_reader("name;age\nAlice;30".as_bytes());
/// ```
#[derive(Default)]
pub struct CsvItemReaderBuilder {
    /// The delimiter character (default: comma ',')
    delimiter: u8,
    /// The line terminator (default: CRLF)
    terminator: Terminator,
    /// Whether the CSV has headers (default: false)
    has_headers: bool,
}

impl CsvItemReaderBuilder {
    /// Creates a new `CsvItemReaderBuilder` with default configuration.
    ///
    /// Default settings:
    /// - Delimiter: comma (,)
    /// - Terminator: CRLF (Windows-style line endings)
    /// - Headers: disabled
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    ///
    /// let builder = CsvItemReaderBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            terminator: Terminator::CRLF,
            has_headers: false,
        }
    }

    /// Sets the delimiter character for the CSV parsing.
    ///
    /// # Parameters
    /// - `delimiter`: The character to use as a field delimiter
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    ///
    /// // Use tab as delimiter
    /// let builder = CsvItemReaderBuilder::new()
    ///     .delimiter(b'\t');
    ///
    /// // Use semicolon as delimiter
    /// let builder = CsvItemReaderBuilder::new()
    ///     .delimiter(b';');
    /// ```
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets the line terminator for the CSV parsing.
    ///
    /// # Parameters
    /// - `terminator`: The line terminator to use
    ///
    /// # Terminator Options
    ///
    /// - `Terminator::CRLF`: Windows-style line endings (default)
    /// - `Terminator::Any(byte)`: Custom terminator, often `b'\n'` for Unix-style
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    /// use csv::Terminator;
    ///
    /// // Use Unix-style line endings (LF)
    /// let builder = CsvItemReaderBuilder::new()
    ///     .terminator(Terminator::Any(b'\n'));
    /// ```
    pub fn terminator(mut self, terminator: Terminator) -> Self {
        self.terminator = terminator;
        self
    }

    /// Sets whether the CSV file has headers.
    ///
    /// When enabled, the first row is treated as headers and is not returned
    /// as part of the data. The header names can be used to match fields in
    /// the deserialization process.
    ///
    /// # Parameters
    /// - `yes`: Whether headers are present
    ///
    /// # Deserialization Impact
    ///
    /// When enabled, column names from headers can be matched to struct field names
    /// during deserialization. This is often more robust than relying on column order.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    ///
    /// // Enable headers (first row is column names)
    /// let builder = CsvItemReaderBuilder::new()
    ///     .has_headers(true);
    ///
    /// // Disable headers (all rows are data)
    /// let builder = CsvItemReaderBuilder::new()
    ///     .has_headers(false);
    /// ```
    pub fn has_headers(mut self, yes: bool) -> Self {
        self.has_headers = yes;
        self
    }

    /// Creates a `CsvItemReader` from a reader.
    ///
    /// This allows reading CSV data from any source that implements the `Read` trait,
    /// such as files, strings, or network connections.
    ///
    /// # Parameters
    /// - `rdr`: The reader containing CSV data
    ///
    /// # Configuration Applied
    ///
    /// The following configurations are applied:
    /// - Trims all whitespace from fields
    /// - Uses specified delimiter (default: comma)
    /// - Uses specified terminator (default: CRLF)
    /// - Handles headers according to configuration
    /// - Strict parsing (not flexible) to identify formatting issues
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    /// use std::io::Cursor;
    ///
    /// #[derive(Deserialize)]
    /// struct Record {
    ///     id: u32,
    ///     name: String,
    /// }
    ///
    /// // Read from a string
    /// let data = "id,name\n1,Alice\n2,Bob";
    /// let reader = CsvItemReaderBuilder::new()
    ///     .has_headers(true)
    ///     .from_reader(data.as_bytes());
    ///
    /// // Or read from a Cursor
    /// let cursor = Cursor::new("id,name\n1,Alice\n2,Bob");
    /// let reader = CsvItemReaderBuilder::new()
    ///     .has_headers(true)
    ///     .from_reader(cursor);
    /// ```
    pub fn from_reader<R: Read>(self, rdr: R) -> CsvItemReader<R> {
        // Configure the CSV reader with builder options
        let rdr = ReaderBuilder::new()
            .trim(Trim::All) // Trim whitespace from all fields
            .delimiter(self.delimiter)
            .terminator(self.terminator)
            .has_headers(self.has_headers)
            .flexible(false) // Use strict parsing to catch formatting errors
            .from_reader(rdr);

        // Convert to a record iterator
        let records = rdr.into_records();

        CsvItemReader {
            records: RefCell::new(records),
        }
    }

    /// Creates a `CsvItemReader` from a file path.
    ///
    /// # Parameters
    /// - `path`: The path to the CSV file
    ///
    /// # Returns
    /// A new `CsvItemReader` configured to read from the specified file
    ///
    /// # Panics
    /// Panics if the file cannot be opened
    ///
    /// # Error Handling
    ///
    /// This method panics immediately if the file cannot be opened, which is appropriate
    /// for initialization failures. Subsequent reading errors are returned as `Result` values
    /// from the `read` method.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Record {
    ///     id: u32,
    ///     name: String,
    /// }
    ///
    /// // Read from a file
    /// let reader = CsvItemReaderBuilder::new()
    ///     .has_headers(true)
    ///     .from_path("data.csv");
    ///
    /// // Process records
    /// let mut records: Vec<Record> = Vec::new();
    /// while let Some(record) = ItemReader::<Record>::read(&reader).unwrap() {
    ///     println!("ID: {}, Name: {}", record.id, record.name);
    ///     records.push(record);
    /// }
    /// ```
    pub fn from_path<R: AsRef<Path>>(self, path: R) -> CsvItemReader<File> {
        // Configure the CSV reader with builder options
        let rdr = ReaderBuilder::new()
            .trim(Trim::All) // Trim whitespace from all fields
            .delimiter(self.delimiter)
            .terminator(self.terminator)
            .has_headers(self.has_headers)
            .flexible(false) // Use strict parsing to catch formatting errors
            .from_path(path);

        // Unwrap here is appropriate since file opening is an initialization step
        // If it fails, we want to fail fast rather than returning an error
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

    /// Tests basic CSV parsing functionality
    ///
    /// This test verifies that the CsvItemReader can correctly parse
    /// CSV data with headers and multiple records.
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
