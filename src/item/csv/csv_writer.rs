use std::{cell::RefCell, fs::File, io::Write, marker::PhantomData, path::Path};

use csv::{Writer, WriterBuilder};
use serde::Serialize;

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

/// A CSV writer that implements the `ItemWriter` trait.
///
/// This writer serializes Rust structs to CSV format and writes them to
/// the underlying destination (file, memory buffer, etc.)
///
/// # Type Parameters
///
/// - `T`: The type of writer destination, must implement `Write` trait
///
/// # Implementation Details
///
/// - Uses `RefCell` for interior mutability of the CSV writer
/// - Integrates with serde for serialization of custom types
/// - Handles serialization of batch items one by one
/// - Converts CSV errors to Spring Batch errors
///
/// # Ownership Considerations
///
/// The writer borrows its destination mutably. When writing to a buffer:
/// - The buffer will be borrowed for the lifetime of the writer
/// - To read from the buffer after writing, ensure the writer is dropped first
/// - One approach is to use a separate scope for the writer operations
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Record {
///     id: u32,
///     name: String,
/// }
///
/// // Create records to write
/// let records = vec![
///     Record { id: 1, name: "Alice".to_string() },
///     Record { id: 2, name: "Bob".to_string() },
/// ];
///
/// // Write records to a CSV string
/// let mut buffer = Vec::new();
/// {
///     // Create a new scope for the writer to ensure it's dropped before we read the buffer
///     let writer = CsvItemWriterBuilder::new()
///         .has_headers(true)
///         .from_writer(&mut buffer);
///
///     writer.write(&records).unwrap();
///     ItemWriter::<Record>::flush(&writer).unwrap();
/// } // writer is dropped here, releasing the borrow on buffer
///
/// // Now we can safely read from the buffer
/// let csv_content = String::from_utf8(buffer).unwrap();
/// assert!(csv_content.contains("id,name"));
/// assert!(csv_content.contains("1,Alice"));
/// assert!(csv_content.contains("2,Bob"));
/// ```
pub struct CsvItemWriter<O, W: Write> {
    /// The underlying CSV writer
    ///
    /// Uses `RefCell` to allow interior mutability while conforming to the
    /// `ItemWriter` trait's immutable self reference in its methods.
    writer: RefCell<Writer<W>>,
    _phantom: PhantomData<O>,
}

impl<O: Serialize, W: Write> ItemWriter<O> for CsvItemWriter<O, W> {
    /// Writes a batch of items to CSV.
    ///
    /// This method serializes each item in the provided slice to CSV format
    /// and writes it to the underlying destination.
    ///
    /// # Serialization Process
    ///
    /// 1. For each item in the batch:
    ///    - Serialize the item to CSV format using serde
    ///    - Write the serialized row to the underlying destination
    /// 2. If any item fails to serialize, return an error immediately
    ///
    /// Note: This method doesn't flush the writer. You need to call `flush()`
    /// explicitly when you're done writing.
    ///
    /// # Parameters
    /// - `items`: A slice of items to be serialized and written
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if writing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Person {
    ///     name: String,
    ///     age: u8,
    /// }
    ///
    /// // Create people to write
    /// let people = vec![
    ///     Person { name: "Alice".to_string(), age: 28 },
    ///     Person { name: "Bob".to_string(), age: 35 },
    /// ];
    ///
    /// // Write to a buffer in a separate scope
    /// let mut buffer = Vec::new();
    /// {
    ///     let writer = CsvItemWriterBuilder::new()
    ///         .from_writer(&mut buffer);
    ///
    ///     // Write the batch of people
    ///     writer.write(&people).unwrap();
    ///     ItemWriter::<Person>::flush(&writer).unwrap();
    /// }
    /// ```
    fn write(&self, items: &[O]) -> ItemWriterResult {
        for item in items.iter() {
            // Try to serialize each item to CSV format
            let result = self.writer.borrow_mut().serialize(item);

            // If serialization fails, return the error immediately
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
    ///
    /// # Important
    ///
    /// You must call this method when you're done writing to ensure all data
    /// is written to the destination. The `write` method buffers data internally
    /// for efficiency, and `flush` ensures it's all written out.
    ///
    /// # When to Call
    ///
    /// - After writing all items in a batch
    /// - Before dropping the writer if you need the data immediately
    /// - When closing a file to ensure all data is written
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if flushing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     id: u32,
    ///     value: String,
    /// }
    ///
    /// // Write to a buffer in a separate scope
    /// let mut buffer = Vec::new();
    /// {
    ///     let writer = CsvItemWriterBuilder::new()
    ///         .from_writer(&mut buffer);
    ///
    ///     // Write some records
    ///     let records = vec![Record { id: 1, value: "test".to_string() }];
    ///     writer.write(&records).unwrap();
    ///
    ///     // Ensure all data is written - specify type explicitly
    ///     ItemWriter::<Record>::flush(&writer).unwrap();
    /// }
    /// ```
    fn flush(&self) -> ItemWriterResult {
        // Flush the underlying CSV writer
        let result = self.writer.borrow_mut().flush();
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

/// A builder for creating CSV item writers.
///
/// This builder allows you to customize the CSV writing behavior,
/// including delimiter and header handling.
///
/// # Design Pattern
///
/// This struct implements the Builder pattern, which allows for fluent, chainable
/// configuration of a `CsvItemWriter` before creation. Each method returns `self`
/// to allow method chaining.
///
/// # Default Configuration
///
/// - Delimiter: comma (,)
/// - Headers: disabled (no header row)
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Record {
///     id: u32,
///     name: String,
/// }
///
/// // Create a CSV writer with custom settings
/// let mut buffer = Vec::new();
/// let writer = CsvItemWriterBuilder::<Record>::new()
///     .delimiter(b';')  // Use semicolon as delimiter
///     .has_headers(true)  // Include headers in output
///     .from_writer(&mut buffer);
/// ```
#[derive(Default)]
pub struct CsvItemWriterBuilder<O> {
    /// The delimiter character (default: comma ',')
    delimiter: u8,
    /// Whether to include headers in the output (default: false)
    has_headers: bool,
    _pd: PhantomData<O>,
}

impl<O> CsvItemWriterBuilder<O> {
    /// Creates a new `CsvItemWriterBuilder` with default configuration.
    ///
    /// Default settings:
    /// - Delimiter: comma (,)
    /// - Headers: disabled
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     field: String,
    /// }
    ///
    /// let builder = CsvItemWriterBuilder::<Record>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            has_headers: false,
            _pd: PhantomData,
        }
    }

    /// Sets the delimiter character for the CSV output.
    ///
    /// # Parameters
    /// - `delimiter`: The character to use as field delimiter
    ///
    /// # Common Delimiters
    ///
    /// - `b','` - Comma (default in US/UK)
    /// - `b';'` - Semicolon (common in Europe)
    /// - `b'\t'` - Tab (for TSV format)
    /// - `b'|'` - Pipe (less common)
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     field: String,
    /// }
    ///
    /// // Use tab as delimiter
    /// let builder = CsvItemWriterBuilder::<Record>::new()
    ///     .delimiter(b'\t');
    ///
    /// // Use semicolon as delimiter
    /// let builder = CsvItemWriterBuilder::<Record>::new()
    ///     .delimiter(b';');
    /// ```
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets whether to include headers in the CSV output.
    ///
    /// When enabled, the writer will include a header row with field names
    /// derived from the struct field names or serde annotations.
    ///
    /// # Parameters
    /// - `yes`: Whether to include headers
    ///
    /// # Header Generation
    ///
    /// Headers are generated from:
    /// - Struct field names by default
    /// - Custom names specified by `#[serde(rename = "...")]` attributes
    /// - For nested fields, the serde flattening mechanism is used
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     field: String,
    /// }
    ///
    /// // Include headers (field names as first row)
    /// let builder = CsvItemWriterBuilder::<Record>::new()
    ///     .has_headers(true);
    ///
    /// // Exclude headers (data only)
    /// let builder = CsvItemWriterBuilder::<Record>::new()
    ///     .has_headers(false);
    /// ```
    pub fn has_headers(mut self, yes: bool) -> Self {
        self.has_headers = yes;
        self
    }

    /// Creates a CSV item writer that writes to a file.
    ///
    /// # Parameters
    /// - `path`: The path where the output file will be created
    ///
    /// # Returns
    /// A configured `CsvItemWriter` instance
    ///
    /// # Panics
    /// Panics if the file cannot be created
    ///
    /// # File Handling
    ///
    /// This method will:
    /// - Create the file if it doesn't exist
    /// - Truncate the file if it exists
    /// - Return a writer that writes to the file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     id: u32,
    ///     value: String,
    /// }
    ///
    /// // Create a writer to a file
    /// let writer = CsvItemWriterBuilder::<Record>::new()
    ///     .has_headers(true)
    ///     .from_path("output.csv");
    ///
    /// // Write some data
    /// let records = vec![
    ///     Record { id: 1, value: "data1".to_string() },
    ///     Record { id: 2, value: "data2".to_string() },
    /// ];
    ///
    /// writer.write(&records).unwrap();
    /// ItemWriter::<Record>::flush(&writer).unwrap();
    /// ```
    pub fn from_path<W: AsRef<Path>>(self, path: W) -> CsvItemWriter<O, File> {
        // Configure and create the CSV writer
        let writer = WriterBuilder::new()
            .flexible(false) // Use strict formatting to detect serialization issues
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .from_path(path);

        // Unwrap here is appropriate since file opening is an initialization step
        // If it fails, we want to fail fast
        CsvItemWriter {
            writer: RefCell::new(writer.unwrap()),
            _phantom: PhantomData,
        }
    }

    /// Creates a CSV item writer that writes to any destination implementing the `Write` trait.
    ///
    /// This allows writing to in-memory buffers, network connections, or other custom destinations.
    ///
    /// # Parameters
    /// - `wtr`: The writer instance to use for output
    ///
    /// # Returns
    /// A configured `CsvItemWriter` instance
    ///
    /// # Common Writer Types
    ///
    /// - `&mut Vec<u8>` - In-memory buffer (most common for tests)
    /// - `File` - File writer for permanent storage
    /// - `Cursor<Vec<u8>>` - In-memory cursor for testing
    /// - `TcpStream` - Network connection for remote writing
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::csv::csv_writer::CsvItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Row<'a> {
    ///     city: &'a str,
    ///     country: &'a str,
    ///     #[serde(rename = "popcount")]
    ///     population: u64,
    /// }
    ///
    /// // Prepare some data
    /// let rows = vec![
    ///     Row {
    ///         city: "Boston",
    ///         country: "United States",
    ///         population: 4628910,
    ///     },
    ///     Row {
    ///         city: "Concord",
    ///         country: "United States",
    ///         population: 42695,
    ///     }
    /// ];
    ///
    /// // Write to a vector buffer in a separate scope
    /// let mut buffer = Vec::new();
    /// {
    ///     let writer = CsvItemWriterBuilder::<Row>::new()
    ///         .has_headers(true)
    ///         .from_writer(&mut buffer);
    ///
    ///     // Write the data
    ///     writer.write(&rows).unwrap();
    ///     ItemWriter::<Row>::flush(&writer).unwrap();
    /// } // writer is dropped here, releasing the borrow
    ///
    /// // Check the output (with headers)
    /// let output = String::from_utf8(buffer).unwrap();
    /// assert!(output.contains("city,country,popcount"));
    /// assert!(output.contains("Boston,United States,4628910"));
    /// ```
    pub fn from_writer<W: Write>(self, wtr: W) -> CsvItemWriter<O, W> {
        // Configure and create the CSV writer
        let wtr = WriterBuilder::new()
            .flexible(false) // Use strict formatting to detect serialization issues
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .from_writer(wtr);

        CsvItemWriter {
            writer: RefCell::new(wtr),
            _phantom: PhantomData,
        }
    }
}
