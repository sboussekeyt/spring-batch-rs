use std::{
    cell::{Cell, RefCell},
    fs::File,
    io::{BufWriter, Write},
    marker::PhantomData,
    path::Path,
};

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

/// A writer that writes items to a JSON output.
///
/// The writer serializes items to JSON format and writes them as an array to the output.
/// It handles proper JSON formatting, including opening and closing brackets for the array
/// and separating items with commas.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// // Define a data structure
/// #[derive(Serialize)]
/// struct Product {
///     id: u32,
///     name: String,
///     price: f64,
/// }
///
/// // Create some products to write
/// let products = vec![
///     Product { id: 1, name: "Widget".to_string(), price: 19.99 },
///     Product { id: 2, name: "Gadget".to_string(), price: 24.99 },
/// ];
///
/// // Create a writer to an in-memory buffer
/// let buffer = Cursor::new(Vec::new());
/// let writer = JsonItemWriterBuilder::new()
///     .from_writer(buffer);
///
/// // Write the products to JSON
/// let writer_ref = &writer as &dyn ItemWriter<Product>;
/// writer_ref.open().unwrap();
/// writer_ref.write(&products).unwrap();
/// writer_ref.close().unwrap();
/// ```
pub struct JsonItemWriter<O, W: Write> {
    /// The buffered writer for the output stream
    stream: RefCell<BufWriter<W>>,
    /// Whether to use pretty formatting (indentation and newlines)
    use_pretty_formatter: bool,
    /// Tracks whether we're writing the first element (to handle commas between items)
    is_first_element: Cell<bool>,
    _phantom: PhantomData<O>,
}

impl<O: serde::Serialize, W: Write> ItemWriter<O> for JsonItemWriter<O, W> {
    /// Writes a batch of items to the JSON output.
    ///
    /// This method serializes each item to JSON, adds commas between items,
    /// and writes the result to the output stream. It keeps track of whether
    /// it's writing the first element to properly format the array.
    ///
    /// # Parameters
    /// - `items`: A slice of items to be serialized and written
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if writing fails
    fn write(&self, items: &[O]) -> ItemWriterResult {
        let mut json_chunk = String::new();

        for item in items.iter() {
            if !self.is_first_element.get() {
                json_chunk.push(',');
            } else {
                self.is_first_element.set(false);
            }

            let result = if self.use_pretty_formatter {
                serde_json::to_string_pretty(item)
            } else {
                serde_json::to_string(item)
            };

            json_chunk.push_str(&result.unwrap());

            if self.use_pretty_formatter {
                json_chunk.push('\n');
            }
        }

        let result = self.stream.borrow_mut().write_all(json_chunk.as_bytes());

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    /// Flushes the output buffer to ensure all data is written to the underlying stream.
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if flushing fails
    fn flush(&self) -> ItemWriterResult {
        let result = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    /// Opens the JSON writer and writes the opening array bracket.
    ///
    /// This method should be called before any calls to write().
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if writing fails
    fn open(&self) -> ItemWriterResult {
        let begin_array = if self.use_pretty_formatter {
            b"[\n".to_vec()
        } else {
            b"[".to_vec()
        };

        let result = self.stream.borrow_mut().write_all(&begin_array);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    /// Closes the JSON writer and writes the closing array bracket.
    ///
    /// This method should be called after all items have been written.
    /// It also flushes the buffer to ensure all data is written.
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(BatchError)` if writing fails
    fn close(&self) -> ItemWriterResult {
        let end_array = if self.use_pretty_formatter {
            b"\n]\n".to_vec()
        } else {
            b"]\n".to_vec()
        };

        let result = self.stream.borrow_mut().write_all(&end_array);
        let _ = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

/// A builder for creating JSON item writers.
///
/// This builder provides a convenient way to configure and create a `JsonItemWriter`
/// with options like pretty formatting and custom indentation.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// // Define a data structure
/// #[derive(Serialize)]
/// struct Person {
///     id: u32,
///     name: String,
///     email: String,
/// }
///
/// // Create a writer with pretty formatting
/// let buffer = Cursor::new(Vec::new());
/// let writer = JsonItemWriterBuilder::new()
///     .pretty_formatter(true)
///     .from_writer(buffer);
///
/// // Use the writer to serialize a person
/// let person = Person {
///     id: 1,
///     name: "Alice".to_string(),
///     email: "alice@example.com".to_string(),
/// };
///
/// let writer_ref = &writer as &dyn ItemWriter<Person>;
/// writer_ref.open().unwrap();
/// writer_ref.write(&[person]).unwrap();
/// writer_ref.close().unwrap();
/// ```
#[derive(Default)]
pub struct JsonItemWriterBuilder {
    /// Indentation to use when pretty-printing (default is two spaces)
    indent: Box<[u8]>,
    /// Whether to use pretty formatting with indentation and newlines
    pretty_formatter: bool,
}

impl JsonItemWriterBuilder {
    /// Creates a new JSON item writer builder with default settings.
    ///
    /// By default, the writer uses compact formatting (not pretty-printed)
    /// and a standard indentation of two spaces.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
    ///
    /// let builder = JsonItemWriterBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            indent: Box::from(b"  ".to_vec()),
            pretty_formatter: false,
        }
    }

    /// Sets the indentation to use when pretty-printing JSON.
    ///
    /// This setting only has an effect if pretty formatting is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
    ///
    /// // Use 4 spaces for indentation
    /// let builder = JsonItemWriterBuilder::new()
    ///     .indent(b"    ")
    ///     .pretty_formatter(true);
    /// ```
    pub fn indent(mut self, indent: &[u8]) -> Self {
        self.indent = Box::from(indent);
        self
    }

    /// Enables or disables pretty formatting of the JSON output.
    ///
    /// When enabled, the JSON output will include newlines and indentation
    /// to make it more human-readable. This is useful for debugging or
    /// when the output will be read by humans.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
    ///
    /// // Enable pretty printing
    /// let pretty_builder = JsonItemWriterBuilder::new()
    ///     .pretty_formatter(true);
    ///
    /// // Disable pretty printing for compact output
    /// let compact_builder = JsonItemWriterBuilder::new()
    ///     .pretty_formatter(false);
    /// ```
    pub fn pretty_formatter(mut self, yes: bool) -> Self {
        self.pretty_formatter = yes;
        self
    }

    /// Creates a JSON item writer that writes to a file at the specified path.
    ///
    /// This method creates a new file (or truncates an existing one) and
    /// configures a JsonItemWriter to write to it.
    ///
    /// # Parameters
    /// - `path`: The path where the output file will be created
    ///
    /// # Returns
    /// A configured JsonItemWriter instance
    ///
    /// # Panics
    /// Panics if the file cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    /// use std::path::Path;
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     id: u32,
    ///     data: String,
    /// }
    ///
    /// // Create a writer to a file
    /// let writer = JsonItemWriterBuilder::new()
    ///     .pretty_formatter(true)
    ///     .from_path("output.json");
    ///
    /// // Generate some data
    /// let records = vec![
    ///     Record { id: 1, data: "First record".to_string() },
    ///     Record { id: 2, data: "Second record".to_string() },
    /// ];
    ///
    /// // Write the data to the file
    /// let writer_ref = &writer as &dyn ItemWriter<Record>;
    /// writer_ref.open().unwrap();
    /// writer_ref.write(&records).unwrap();
    /// writer_ref.close().unwrap();
    /// ```
    pub fn from_path<O, W: AsRef<Path>>(self, path: W) -> JsonItemWriter<O, File> {
        let file = File::create(path).expect("Unable to open file");

        let buf_writer = BufWriter::new(file);

        JsonItemWriter {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter: self.pretty_formatter,
            is_first_element: Cell::new(true),
            _phantom: PhantomData,
        }
    }

    /// Creates a JSON item writer that writes to any destination implementing the `Write` trait.
    ///
    /// This allows writing to in-memory buffers, network connections, or other custom destinations.
    ///
    /// # Parameters
    /// - `wtr`: The writer instance to use for output
    ///
    /// # Returns
    /// A configured JsonItemWriter instance
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    /// use std::io::Cursor;
    ///
    /// #[derive(Serialize)]
    /// struct Event {
    ///     timestamp: u64,
    ///     message: String,
    /// }
    ///
    /// // Create a writer to an in-memory buffer
    /// let buffer = Cursor::new(Vec::new());
    /// let writer = JsonItemWriterBuilder::new()
    ///     .from_writer(buffer);
    ///
    /// // Generate some data
    /// let events = vec![
    ///     Event { timestamp: 1620000000, message: "Server started".to_string() },
    ///     Event { timestamp: 1620000060, message: "Connected to database".to_string() },
    /// ];
    ///
    /// // Write the data
    /// let writer_ref = &writer as &dyn ItemWriter<Event>;
    /// writer_ref.open().unwrap();
    /// writer_ref.write(&events).unwrap();
    /// writer_ref.close().unwrap();
    /// ```
    pub fn from_writer<O, W: Write>(self, wtr: W) -> JsonItemWriter<O, W> {
        let buf_writer = BufWriter::new(wtr);

        JsonItemWriter {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter: self.pretty_formatter,
            is_first_element: Cell::new(true),
            _phantom: PhantomData,
        }
    }
}
