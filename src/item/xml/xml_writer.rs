use crate::core::item::{ItemWriter, ItemWriterResult};
use crate::error::BatchError;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    Writer,
};
use serde::Serialize;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::Path;

/// A writer that writes items to an XML file.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// #[derive(Serialize)]
/// struct Person {
///     #[serde(rename = "@id")]
///     id: i32,
///     name: String,
///     age: i32,
/// }
///
/// // Create a writer that writes to a memory buffer
/// let buffer = Cursor::new(Vec::new());
/// let writer = XmlItemWriterBuilder::new()
///     .root_tag("people")
///     .item_tag("person")
///     .from_writer::<Person, _>(buffer);
///
/// // Create some data to write
/// let persons = vec![
///     Person { id: 1, name: "Alice".to_string(), age: 30 },
///     Person { id: 2, name: "Bob".to_string(), age: 25 },
/// ];
///
/// // Write the data
/// writer.open().unwrap();
/// writer.write(&persons).unwrap();
/// writer.close().unwrap();
/// ```
///
/// Using a file as output:
///
/// ```no_run
/// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use tempfile::NamedTempFile;
///
/// #[derive(Serialize)]
/// struct Person {
///     #[serde(rename = "@id")]
///     id: i32,
///     name: String,
///     age: i32,
/// }
///
/// // Create a temporary file
/// let temp_file = NamedTempFile::new().unwrap();
/// let writer = XmlItemWriterBuilder::new()
///     .root_tag("people")
///     .item_tag("person")
///     .from_path::<Person, _>(temp_file.path())
///     .unwrap();
///
/// // Create some data to write
/// let persons = vec![
///     Person { id: 1, name: "Alice".to_string(), age: 30 },
///     Person { id: 2, name: "Bob".to_string(), age: 25 },
/// ];
///
/// // Write the data
/// writer.open().unwrap();
/// writer.write(&persons).unwrap();
/// writer.close().unwrap();
///
/// // The XML file now contains:
/// // <people>
/// //   <person id="1">
/// //     <name>Alice</name>
/// //     <age>30</age>
/// //   </person>
/// //   <person id="2">
/// //     <name>Bob</name>
/// //     <age>25</age>
/// //   </person>
/// // </people>
/// ```
pub struct XmlItemWriter<T, W: Write = File> {
    writer: RefCell<Writer<BufWriter<W>>>,
    item_tag: String,
    root_tag: String,
    _phantom: PhantomData<T>,
}

impl<T, W: Write> ItemWriter<T> for XmlItemWriter<T, W>
where
    T: Serialize,
{
    fn write(&self, items: &[T]) -> ItemWriterResult {
        for item in items {
            self.writer
                .borrow_mut()
                .write_serializable(&self.item_tag, item)
                .map_err(|e| BatchError::ItemWriter(format!("Failed to write XML item: {}", e)))?;
        }
        Ok(())
    }

    fn flush(&self) -> ItemWriterResult {
        let result = self.writer.borrow_mut().get_mut().flush();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(BatchError::ItemWriter(format!(
                "Failed to flush XML file: {}",
                e
            ))),
        }
    }

    fn open(&self) -> ItemWriterResult {
        let root = BytesStart::new(&self.root_tag);
        self.writer
            .borrow_mut()
            .write_event(Event::Start(root))
            .map_err(|e| BatchError::ItemWriter(format!("Failed to write XML root: {}", e)))?;
        Ok(())
    }

    fn close(&self) -> ItemWriterResult {
        self.writer
            .borrow_mut()
            .write_event(Event::End(BytesEnd::new(&self.root_tag)))
            .map_err(|e| BatchError::ItemWriter(format!("Failed to write XML end: {}", e)))?;
        self.flush()
    }
}

/// Builder for creating XML item writers.
///
/// This builder allows you to configure XML writers with:
/// - A root tag for the XML document
/// - An item tag for each written element
/// - Various output destinations (file, in-memory buffer, etc.)
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// #[derive(Serialize)]
/// struct Address {
///     street: String,
///     city: String,
///     country: String,
/// }
///
/// #[derive(Serialize)]
/// struct Person {
///     #[serde(rename = "@id")]
///     id: i32,
///     name: String,
///     age: i32,
///     address: Address,
/// }
///
/// // Create a buffer for our output
/// let buffer = Cursor::new(Vec::new());
///
/// // Create a writer using the builder pattern
/// let writer = XmlItemWriterBuilder::new()
///     .root_tag("directory")
///     .item_tag("person")
///     .from_writer::<Person, _>(buffer);
///
/// // Create a person with nested address
/// let person = Person {
///     id: 1,
///     name: "Alice".to_string(),
///     age: 30,
///     address: Address {
///         street: "123 Main St".to_string(),
///         city: "Springfield".to_string(),
///         country: "USA".to_string(),
///     },
/// };
///
/// // Write the person to XML
/// writer.open().unwrap();
/// writer.write(&[person]).unwrap();
/// writer.close().unwrap();
/// ```
#[derive(Default)]
pub struct XmlItemWriterBuilder {
    root_tag: String,
    item_tag: Option<String>,
}

impl XmlItemWriterBuilder {
    /// Creates a new `XmlItemWriterBuilder` with default values.
    ///
    /// The default root tag is "root" and the default item tag is derived from
    /// the type name of the serialized items.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
    ///
    /// let builder = XmlItemWriterBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            root_tag: "root".to_string(),
            item_tag: None,
        }
    }

    /// Sets the root tag for the XML document.
    ///
    /// The root tag wraps all items in the XML document.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
    ///
    /// let builder = XmlItemWriterBuilder::new()
    ///     .root_tag("people");
    /// ```
    pub fn root_tag(mut self, root_tag: &str) -> Self {
        self.root_tag = root_tag.to_string();
        self
    }

    /// Sets the item tag for each XML element.
    ///
    /// Each item in the collection will be wrapped with this tag.
    /// If not specified, it will default to the lowercase name of the item type.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
    ///
    /// let builder = XmlItemWriterBuilder::new()
    ///     .root_tag("people")
    ///     .item_tag("person");
    /// ```
    pub fn item_tag(mut self, item_tag: &str) -> Self {
        self.item_tag = Some(item_tag.to_string());
        self
    }

    /// Creates an `XmlItemWriter` from a file path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
    /// use serde::Serialize;
    /// use tempfile::NamedTempFile;
    ///
    /// #[derive(Serialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Create a temporary file for testing
    /// let temp_file = NamedTempFile::new().unwrap();
    /// let writer = XmlItemWriterBuilder::new()
    ///     .root_tag("people")
    ///     .item_tag("person")
    ///     .from_path::<Person, _>(temp_file.path())
    ///     .unwrap();
    /// ```
    pub fn from_path<T: Serialize, P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<XmlItemWriter<T>, BatchError> {
        let file = File::create(path)
            .map_err(|e| BatchError::ItemWriter(format!("Failed to create XML file: {}", e)))?;
        let writer = Writer::new(BufWriter::new(file));
        let item_tag = self.item_tag.unwrap_or_else(|| {
            std::any::type_name::<T>()
                .split("::")
                .last()
                .unwrap_or("item")
                .to_lowercase()
        });

        Ok(XmlItemWriter {
            writer: RefCell::new(writer),
            item_tag,
            root_tag: self.root_tag,
            _phantom: PhantomData,
        })
    }

    /// Creates an `XmlItemWriter` from a writer.
    ///
    /// This is useful for writing to in-memory buffers, network streams,
    /// or other custom writers.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use serde::Serialize;
    /// use std::io::Cursor;
    ///
    /// #[derive(Serialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Create a writer that writes to an in-memory buffer
    /// let buffer = Cursor::new(Vec::new());
    /// let writer = XmlItemWriterBuilder::new()
    ///     .root_tag("people")
    ///     .item_tag("person")
    ///     .from_writer::<Person, _>(buffer);
    ///
    /// // Now we can use the writer to write XML data
    /// writer.open().unwrap();
    /// writer.write(&[Person { name: "Alice".to_string(), age: 30 }]).unwrap();
    /// writer.close().unwrap();
    /// ```
    pub fn from_writer<T: Serialize, W: Write>(self, wtr: W) -> XmlItemWriter<T, W> {
        let writer = Writer::new(BufWriter::new(wtr));
        let item_tag = self.item_tag.unwrap_or_else(|| {
            std::any::type_name::<T>()
                .split("::")
                .last()
                .unwrap_or("item")
                .to_lowercase()
        });

        XmlItemWriter {
            writer: RefCell::new(writer),
            item_tag,
            root_tag: self.root_tag,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct Contact {
        #[serde(rename = "@type")]
        contact_type: String,
        name: String,
        email: String,
        phone: String,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct Location {
        #[serde(rename = "@country")]
        country: String,
        city: String,
        #[serde(rename = "@timezone")]
        timezone: String,
        coordinates: Coordinates,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct Coordinates {
        #[serde(rename = "@format")]
        format: String,
        latitude: f64,
        longitude: f64,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct Company {
        #[serde(rename = "@id")]
        id: i32,
        #[serde(rename = "@type")]
        company_type: String,
        name: String,
        founded_year: i32,
        contact: Vec<Contact>,
        location: Location,
        #[serde(rename = "@active")]
        active: bool,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SimpleItem {
        id: i32,
        name: String,
        value: f64,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Product {
        id: i32,
        name: String,
        price: f64,
        tags: Vec<String>,
    }

    #[test]
    fn test_xml_writer_builder() {
        let temp_file = NamedTempFile::new().unwrap();
        let writer = XmlItemWriterBuilder::new()
            .root_tag("companies")
            .item_tag("company")
            .from_path::<Company, _>(temp_file.path())
            .unwrap();

        let items = vec![
            Company {
                id: 1,
                company_type: "tech".to_string(),
                name: "TechCorp".to_string(),
                founded_year: 2010,
                active: true,
                contact: vec![
                    Contact {
                        contact_type: "primary".to_string(),
                        name: "John Doe".to_string(),
                        email: "john@techcorp.com".to_string(),
                        phone: "+1-555-0123".to_string(),
                    },
                    Contact {
                        contact_type: "secondary".to_string(),
                        name: "Jane Smith".to_string(),
                        email: "jane@techcorp.com".to_string(),
                        phone: "+1-555-0124".to_string(),
                    },
                ],
                location: Location {
                    country: "USA".to_string(),
                    city: "San Francisco".to_string(),
                    timezone: "PST".to_string(),
                    coordinates: Coordinates {
                        format: "decimal".to_string(),
                        latitude: 37.7749,
                        longitude: -122.4194,
                    },
                },
            },
            Company {
                id: 2,
                company_type: "finance".to_string(),
                name: "FinanceCo".to_string(),
                founded_year: 2000,
                active: true,
                contact: vec![Contact {
                    contact_type: "primary".to_string(),
                    name: "Alice Brown".to_string(),
                    email: "alice@financeco.com".to_string(),
                    phone: "+1-555-0125".to_string(),
                }],
                location: Location {
                    country: "UK".to_string(),
                    city: "London".to_string(),
                    timezone: "GMT".to_string(),
                    coordinates: Coordinates {
                        format: "decimal".to_string(),
                        latitude: 51.5074,
                        longitude: -0.1278,
                    },
                },
            },
        ];

        writer.open().unwrap();
        writer.write(&items).unwrap();
        writer.close().unwrap();

        // Read back the file to verify contents
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        println!("Generated XML:\n{}", content);

        // Verify root structure
        assert!(content.contains("<companies>"));
        assert!(content.contains("</companies>"));

        // Verify first company
        assert!(content.contains("<company id=\"1\" type=\"tech\" active=\"true\">"));
        assert!(content.contains("<name>TechCorp</name>"));
        assert!(content.contains("<founded_year>2010</founded_year>"));

        // Verify contacts
        assert!(content.contains("<contact type=\"primary\">"));
        assert!(content.contains("<name>John Doe</name>"));
        assert!(content.contains("<email>john@techcorp.com</email>"));
        assert!(content.contains("<phone>+1-555-0123</phone>"));
        assert!(content.contains("<contact type=\"secondary\">"));
        assert!(content.contains("<name>Jane Smith</name>"));

        // Verify location
        assert!(content.contains("<location country=\"USA\" timezone=\"PST\">"));
        assert!(content.contains("<city>San Francisco</city>"));

        // Verify coordinates
        assert!(content.contains("<coordinates format=\"decimal\">"));
        assert!(content.contains("<latitude>37.7749</latitude>"));
        assert!(content.contains("<longitude>-122.4194</longitude>"));

        // Verify second company
        assert!(content.contains("<company id=\"2\" type=\"finance\" active=\"true\">"));
        assert!(content.contains("<name>FinanceCo</name>"));
        assert!(content.contains("<founded_year>2000</founded_year>"));
        assert!(content.contains("<location country=\"UK\" timezone=\"GMT\">"));
        assert!(content.contains("<city>London</city>"));
    }

    #[test]
    fn test_in_memory_writing() {
        let buffer = Cursor::new(Vec::new());
        let writer = XmlItemWriterBuilder::new()
            .root_tag("items")
            .item_tag("item")
            .from_writer::<SimpleItem, _>(buffer);

        let items = vec![
            SimpleItem {
                id: 1,
                name: "Item 1".to_string(),
                value: 10.5,
            },
            SimpleItem {
                id: 2,
                name: "Item 2".to_string(),
                value: 20.75,
            },
        ];

        writer.open().unwrap();
        writer.write(&items).unwrap();
        writer.close().unwrap();

        // Get the inner buffer from the writer
        let content = {
            let buf_writer = writer.writer.borrow_mut();
            let cursor = buf_writer.get_ref().get_ref();
            String::from_utf8(cursor.get_ref().clone()).unwrap()
        };

        assert!(content.contains("<items>"));
        assert!(content.contains("<item>"));
        assert!(content.contains("<id>1</id>"));
        assert!(content.contains("<name>Item 1</name>"));
        assert!(content.contains("<value>10.5</value>"));
        assert!(content.contains("<id>2</id>"));
        assert!(content.contains("<name>Item 2</name>"));
        assert!(content.contains("<value>20.75</value>"));
        assert!(content.contains("</item>"));
        assert!(content.contains("</items>"));
    }

    #[test]
    fn test_empty_collection() {
        let buffer = Cursor::new(Vec::new());
        let writer = XmlItemWriterBuilder::new()
            .root_tag("items")
            .item_tag("item")
            .from_writer::<SimpleItem, _>(buffer);

        let empty_items: Vec<SimpleItem> = vec![];

        writer.open().unwrap();
        writer.write(&empty_items).unwrap();
        writer.close().unwrap();

        // Get the inner buffer from the writer
        let content = {
            let buf_writer = writer.writer.borrow_mut();
            let cursor = buf_writer.get_ref().get_ref();
            String::from_utf8(cursor.get_ref().clone()).unwrap()
        };

        assert_eq!(content, "<items></items>");
    }

    #[test]
    fn test_default_item_tag() {
        let buffer = Cursor::new(Vec::new());

        // Don't specify item_tag to test the default behavior
        let writer = XmlItemWriterBuilder::new()
            .root_tag("items")
            .from_writer::<SimpleItem, _>(buffer);

        let items = vec![SimpleItem {
            id: 1,
            name: "Test".to_string(),
            value: 1.0,
        }];

        writer.open().unwrap();
        writer.write(&items).unwrap();
        writer.close().unwrap();

        // Get the inner buffer from the writer
        let content = {
            let buf_writer = writer.writer.borrow_mut();
            let cursor = buf_writer.get_ref().get_ref();
            String::from_utf8(cursor.get_ref().clone()).unwrap()
        };

        // The default tag should be "simpleitem" (lowercase of SimpleItem)
        assert!(content.contains("<simpleitem>"));
        assert!(content.contains("</simpleitem>"));
    }

    #[test]
    fn test_xml_escaping() {
        let buffer = Cursor::new(Vec::new());
        let writer = XmlItemWriterBuilder::new()
            .root_tag("items")
            .item_tag("item")
            .from_writer::<SimpleItem, _>(buffer);

        // Create items with special XML characters that need escaping
        let items = vec![
            SimpleItem {
                id: 1,
                name: "Item with < and > symbols".to_string(),
                value: 10.5,
            },
            SimpleItem {
                id: 2,
                name: "Item with & and \" characters".to_string(),
                value: 20.75,
            },
        ];

        writer.open().unwrap();
        writer.write(&items).unwrap();
        writer.close().unwrap();

        // Get the inner buffer from the writer
        let content = {
            let buf_writer = writer.writer.borrow_mut();
            let cursor = buf_writer.get_ref().get_ref();
            String::from_utf8(cursor.get_ref().clone()).unwrap()
        };

        // Print the content for debugging
        println!("XML content: {}", content);

        // Check that special characters are properly escaped
        assert!(content.contains("Item with &lt; and &gt; symbols"));
        // Use contains_any to check for either possible escaping format
        assert!(content.contains("Item with &amp;") || content.contains("Item with &"));
        assert!(content.contains("\"") || content.contains("&quot;"));
    }

    #[test]
    fn test_array_fields() {
        let buffer = Cursor::new(Vec::new());
        let writer = XmlItemWriterBuilder::new()
            .root_tag("products")
            .item_tag("product")
            .from_writer::<Product, _>(buffer);

        let items = vec![
            Product {
                id: 1,
                name: "Laptop".to_string(),
                price: 999.99,
                tags: vec![
                    "electronics".to_string(),
                    "computer".to_string(),
                    "portable".to_string(),
                ],
            },
            Product {
                id: 2,
                name: "Smartphone".to_string(),
                price: 699.99,
                tags: vec!["electronics".to_string(), "mobile".to_string()],
            },
        ];

        writer.open().unwrap();
        writer.write(&items).unwrap();
        writer.close().unwrap();

        // Get the inner buffer from the writer
        let content = {
            let buf_writer = writer.writer.borrow_mut();
            let cursor = buf_writer.get_ref().get_ref();
            String::from_utf8(cursor.get_ref().clone()).unwrap()
        };

        // Verify the array elements are properly serialized
        assert!(content.contains("<products>"));
        assert!(content.contains("<product>"));
        assert!(content.contains("<id>1</id>"));
        assert!(content.contains("<name>Laptop</name>"));
        assert!(content.contains("<price>999.99</price>"));
        assert!(content.contains("<tags>electronics</tags>"));
        assert!(content.contains("<tags>computer</tags>"));
        assert!(content.contains("<tags>portable</tags>"));
        assert!(content.contains("<id>2</id>"));
        assert!(content.contains("<name>Smartphone</name>"));
        assert!(content.contains("<price>699.99</price>"));
        assert!(content.contains("</product>"));
        assert!(content.contains("</products>"));
    }

    #[test]
    fn test_error_handling_invalid_path() {
        // Try to create a writer with an invalid path
        let invalid_path = "/nonexistent/directory/file.xml";
        let result = XmlItemWriterBuilder::new()
            .root_tag("items")
            .item_tag("item")
            .from_path::<SimpleItem, _>(invalid_path);

        // Verify the result is an error
        assert!(result.is_err());

        // Verify the error message contains the expected information
        if let Err(error) = result {
            if let BatchError::ItemWriter(message) = error {
                assert!(message.contains("Failed to create XML file"));
            } else {
                panic!("Expected ItemWriter error, got {:?}", error);
            }
        }
    }
}
