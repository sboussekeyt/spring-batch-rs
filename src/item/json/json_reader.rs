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

/// Internal structure to represent the parsing state result
#[derive(Debug)]
enum JsonParserResult {
    /// Indicates that the parser has not yet reached the end of the JSON array
    NotEnded,
    /// Indicates a parsing error occurred with the specific serde_json error
    ParsingError { error: serde_json::Error },
}

/// A reader that reads items from a JSON source.
///
/// The reader expects JSON data in an array format, where each object in the array
/// represents a single item to be processed. It implements a streaming approach
/// that allows reading large JSON files without loading the entire file into memory.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::json::JsonItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// // Define a structure matching our JSON format
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Product {
///     id: u32,
///     name: String,
///     price: f64,
/// }
///
/// // Create some JSON data with products
/// let json_data = r#"[
///   {"id": 1, "name": "Keyboard", "price": 49.99},
///   {"id": 2, "name": "Mouse", "price": 29.99},
///   {"id": 3, "name": "Monitor", "price": 199.99}
/// ]"#;
///
/// // Create a reader using the builder
/// let cursor = Cursor::new(json_data);
/// let reader = JsonItemReaderBuilder::<Product>::new()
///     .from_reader(cursor);
///
/// // Read all products
/// let product1 = reader.read().unwrap().unwrap();
/// assert_eq!(product1.id, 1);
/// assert_eq!(product1.name, "Keyboard");
/// assert_eq!(product1.price, 49.99);
///
/// let product2 = reader.read().unwrap().unwrap();
/// assert_eq!(product2.id, 2);
///
/// let product3 = reader.read().unwrap().unwrap();
/// assert_eq!(product3.id, 3);
///
/// // No more products
/// assert!(reader.read().unwrap().is_none());
/// ```
pub struct JsonItemReader<R, T> {
    /// Phantom data to handle the generic type parameter T (item type)
    pd: PhantomData<T>,
    /// Buffered reader for the input source
    reader: RefCell<BufReader<R>>,
    /// Buffer capacity in bytes
    capacity: usize,
    /// Current nesting level while parsing JSON
    level: Cell<u16>,
    /// Current position within the buffer
    index: Cell<usize>,
    /// Buffer for the current JSON object being parsed
    object: RefCell<Vec<u8>>,
}

impl<R: Read, T: DeserializeOwned> JsonItemReader<R, T> {
    /// Creates a new JSON item reader with the specified input source and buffer capacity.
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

    /// Gets the character at the current index in the buffer
    fn get_current_char(&self, buffer: &[u8]) -> u8 {
        buffer[self.index.get()]
    }

    /// Checks if the current character is the beginning of a new JSON array
    fn is_new_seq(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b'['
    }

    /// Checks if the current character is the end of a JSON array
    fn is_end_seq(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b']'
    }

    /// Checks if the current character is the beginning of a new JSON object
    fn is_new_object(&self, buffer: &[u8]) -> bool {
        self.level == 0.into() && self.get_current_char(buffer) == b'{'
    }

    /// Checks if the current character is the end of a JSON object at level 1
    /// (an object directly inside the main array)
    fn is_end_object(&self, buffer: &[u8]) -> bool {
        self.level == 1.into() && self.get_current_char(buffer) == b'}'
    }

    /// Clears the object buffer to start parsing a new object
    fn start_new(&self) {
        self.object.borrow_mut().clear();
    }

    /// Adds the current character to the object buffer, ignoring whitespace
    fn append_char(&self, buffer: &[u8]) {
        let current_char = self.get_current_char(buffer);
        if current_char != b' ' && current_char != b'\n' {
            self.object.borrow_mut().push(self.get_current_char(buffer));
        }
    }

    /// Resets the index to the beginning of the buffer
    fn clear_buff(&self) {
        self.index.set(0);
    }

    /// Increments the nesting level when entering a new object or array
    fn level_inc(&self) {
        self.level.set(self.level.get() + 1);
    }

    /// Decrements the nesting level when exiting an object or array
    fn level_dec(&self) {
        self.level.set(self.level.get() - 1);
    }

    /// Moves to the next character in the buffer
    fn index_inc(&self) {
        self.index.set(self.index.get() + 1);
    }

    /// Attempts to read the next item from the current buffer
    ///
    /// This method parses the JSON buffer character by character, keeping track of nesting levels,
    /// and tries to extract a complete JSON object. When it finds a complete object at level 1,
    /// it deserializes it into the target type T.
    ///
    /// # Returns
    /// - `Ok(T)` - Successfully parsed an item
    /// - `Err(JsonParserResult::NotEnded)` - Need more data from the source
    /// - `Err(JsonParserResult::ParsingError)` - Failed to parse the JSON
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
    /// Reads the next item from the JSON stream
    ///
    /// This method reads data from the underlying input source in chunks,
    /// processes the buffer to find the next complete JSON object, and
    /// deserializes it into the target type.
    ///
    /// # Returns
    /// - `Ok(Some(T))` - Successfully read and deserialized an item
    /// - `Ok(None)` - End of input reached, no more items
    /// - `Err(BatchError)` - Error during reading or parsing
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

/// A builder for creating JSON item readers.
///
/// This builder provides a convenient way to configure and create a `JsonItemReader`
/// with custom parameters like buffer capacity.
///
/// # Examples
///
/// Reading from a string:
///
/// ```
/// use spring_batch_rs::item::json::JsonItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// #[derive(Debug, Deserialize)]
/// struct Person {
///     name: String,
///     age: u32,
///     occupation: String,
/// }
///
/// // Sample JSON data
/// let json = r#"[
///   {"name": "JohnDoe", "age": 30, "occupation": "SoftwareEngineer"},
///   {"name": "JaneSmith", "age": 28, "occupation": "DataScientist"}
/// ]"#;
///
/// // Create a reader
/// let cursor = Cursor::new(json);
/// let reader = JsonItemReaderBuilder::<Person>::new()
///     .capacity(4096)  // Set a custom buffer capacity
///     .from_reader(cursor);
///
/// // Read the items
/// let person1 = reader.read().unwrap().unwrap();
/// assert_eq!(person1.name, "JohnDoe");
/// assert_eq!(person1.age, 30);
///
/// let person2 = reader.read().unwrap().unwrap();
/// assert_eq!(person2.name, "JaneSmith");
/// assert_eq!(person2.occupation, "DataScientist");
/// ```
///
/// The builder can also be used to read from files or any other source that implements
/// the `Read` trait.
#[derive(Default)]
pub struct JsonItemReaderBuilder<T> {
    /// Phantom data to handle the generic type parameter T
    _pd: PhantomData<T>,
    /// Optional buffer capacity - defaults to 8KB if not specified
    capacity: Option<usize>,
}

impl<T: DeserializeOwned> JsonItemReaderBuilder<T> {
    /// Creates a new JSON item reader builder with default settings.
    ///
    /// The default buffer capacity is 8 KB (8192 bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::JsonItemReaderBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Record {
    ///     id: u32,
    ///     value: String,
    /// }
    ///
    /// let builder = JsonItemReaderBuilder::<Record>::new();
    /// ```
    pub fn new() -> JsonItemReaderBuilder<T> {
        Self {
            _pd: PhantomData,
            capacity: Some(8 * 1024),
        }
    }

    /// Sets the buffer capacity for the JSON reader.
    ///
    /// A larger capacity can improve performance when reading large files,
    /// but uses more memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::JsonItemReaderBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Record {
    ///     id: u32,
    ///     value: String,
    /// }
    ///
    /// // Create a builder with a 16 KB buffer
    /// let builder = JsonItemReaderBuilder::<Record>::new()
    ///     .capacity(16 * 1024);
    /// ```
    pub fn capacity(mut self, capacity: usize) -> JsonItemReaderBuilder<T> {
        self.capacity = Some(capacity);
        self
    }

    /// Creates a JSON item reader from any source that implements the `Read` trait.
    ///
    /// This allows reading from files, memory buffers, network connections, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::json::JsonItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    /// use std::io::Cursor;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Order {
    ///     id: String,
    ///     customer: String,
    ///     total: f64,
    /// }
    ///
    /// // Sample JSON data
    /// let json = r#"[
    ///   {"id": "ORD-001", "customer": "JohnDoe", "total": 125.50},
    ///   {"id": "ORD-002", "customer": "JaneSmith", "total": 89.99}
    /// ]"#;
    ///
    /// // Create a reader from a memory buffer
    /// let cursor = Cursor::new(json);
    /// let reader = JsonItemReaderBuilder::<Order>::new()
    ///     .from_reader(cursor);
    ///
    /// // Process the orders
    /// let first_order = reader.read().unwrap().unwrap();
    /// assert_eq!(first_order.id, "ORD-001");
    /// assert_eq!(first_order.total, 125.50);
    /// ```
    pub fn from_reader<R: Read>(self, rdr: R) -> JsonItemReader<R, T> {
        // Create a new JsonItemReader with the configured capacity
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

    /// Tests reading JSON data from a file
    ///
    /// This test verifies that the reader can correctly parse and deserialize
    /// JSON data from a file into Person objects.
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

    /// Tests reading from non-JSON input
    ///
    /// This test verifies that the reader gracefully handles input data
    /// that isn't valid JSON without crashing.
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
