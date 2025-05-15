/// JSON support for reading and writing structured data.
///
/// This module provides components for reading JSON data from various sources and writing data to JSON output.
/// The implementation uses `serde_json` for efficient JSON parsing and serialization.
///
/// # Module Architecture
///
/// The JSON module consists of two main components:
///
/// 1. **JsonItemReader**: A streaming JSON reader that can efficiently process large JSON arrays
///    by reading and deserializing objects one at a time without loading the entire file into memory.
///    It works by parsing the JSON buffer character by character, tracking nesting levels, and
///    identifying complete JSON objects.
///
/// 2. **JsonItemWriter**: A JSON writer that serializes items into JSON format and writes them as
///    a properly formatted JSON array. It handles opening and closing array brackets, adding commas
///    between items, and supports both compact and pretty-printed output.
///
/// Each component follows the builder pattern for easy configuration.
///
/// # Features
///
/// - Read JSON data from files, strings, or any source implementing the `Read` trait
/// - Write data to JSON files with configurable formatting options
/// - Seamless integration with Serde for serialization and deserialization
/// - Support for both compact and pretty-printed JSON output
/// - Memory-efficient streaming processing for large datasets
///
/// # Examples
///
/// ## Reading from JSON
///
/// ```
/// use spring_batch_rs::item::json::json_reader::JsonItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// // Define a data structure matching our JSON format
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
///     email: String,
///     active: bool,
/// }
///
/// // Sample JSON data
/// let json_data = r#"[
///   {
///     "id": 1,
///     "name": "AliceJohnson",
///     "email": "alice@example.com",
///     "active": true
///   },
///   {
///     "id": 2,
///     "name": "BobSmith",
///     "email": "bob@example.com",
///     "active": false
///   }
/// ]"#;
///
/// // Create a reader from our JSON
/// let cursor = Cursor::new(json_data);
/// let reader = JsonItemReaderBuilder::<User>::new()
///     .capacity(1024)
///     .from_reader(cursor);
///
/// // Read and process the users
/// let mut users = Vec::new();
/// while let Some(user) = reader.read().unwrap() {
///     users.push(user);
/// }
///
/// // Verify results
/// assert_eq!(users.len(), 2);
/// assert_eq!(users[0].id, 1);
/// assert_eq!(users[0].name, "AliceJohnson");
/// assert_eq!(users[0].email, "alice@example.com");
/// assert_eq!(users[0].active, true);
///
/// assert_eq!(users[1].id, 2);
/// assert_eq!(users[1].name, "BobSmith");
/// assert_eq!(users[1].email, "bob@example.com");
/// assert_eq!(users[1].active, false);
/// ```
///
/// ## Writing to JSON
///
/// ```
/// use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// // Define a data structure for serialization
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
///     email: String,
///     role: String,
///     skills: Vec<String>,
/// }
///
/// // Create some users
/// let users = vec![
///     User {
///         id: 1,
///         name: "Alice Johnson".to_string(),
///         email: "alice@example.com".to_string(),
///         role: "Developer".to_string(),
///         skills: vec!["Rust".to_string(), "Python".to_string()],
///     },
///     User {
///         id: 2,
///         name: "Bob Smith".to_string(),
///         email: "bob@example.com".to_string(),
///         role: "Designer".to_string(),
///         skills: vec!["UI".to_string(), "UX".to_string(), "Figma".to_string()],
///     },
/// ];
///
/// // Create a writer with a memory buffer and pretty formatting
/// let buffer = Cursor::new(Vec::new());
/// let writer = JsonItemWriterBuilder::new()
///     .pretty_formatter(true)
///     .from_writer(buffer);
///
/// // Write the users to JSON
/// let writer_ref = &writer as &dyn ItemWriter<User>;
/// writer_ref.open().unwrap();
/// writer_ref.write(&users).unwrap();
/// writer_ref.close().unwrap();
///
/// // The resulting JSON would look similar to:
/// // [
/// //   {
/// //     "id": 1,
/// //     "name": "Alice Johnson",
/// //     "email": "alice@example.com",
/// //     "role": "Developer",
/// //     "skills": [
/// //       "Rust",
/// //       "Python"
/// //     ]
/// //   },
/// //   {
/// //     "id": 2,
/// //     "name": "Bob Smith",
/// //     "email": "bob@example.com",
/// //     "role": "Designer",
/// //     "skills": [
/// //       "UI",
/// //       "UX",
/// //       "Figma"
/// //     ]
/// //   }
/// // ]
/// ```
/// A module providing facilities for reading JSON data records.
pub mod json_reader;
/// The `json_writer` module contains the `JsonItemWriter` struct, which is the main entry point for writing items to a JSON data source.
/// It implements the `ItemWriter` trait and provides methods for serializing Rust structs into JSON and writing them to a data source.
pub mod json_writer;

// Re-export the main types for easier access
pub use json_reader::{JsonItemReader, JsonItemReaderBuilder};
pub use json_writer::{JsonItemWriter, JsonItemWriterBuilder};
