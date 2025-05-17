use crate::core::item::{ItemReader, ItemReaderResult};
use crate::error::BatchError;
use log::{debug, error};
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read};
use std::marker::PhantomData;
use std::path::Path;
use std::str;

/// A builder for creating XML item readers.
///
/// This builder helps configure XML readers with:
/// - A tag name to identify items in the XML
/// - Buffer capacity for performance tuning
/// - Various input sources (files, in-memory data, etc.)
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// // Define a structure that matches our XML format
/// #[derive(Debug, Deserialize)]
/// struct Person {
///     #[serde(rename = "@id")]
///     id: i32,
///     name: String,
///     age: i32,
/// }
///
/// // Create some XML data
/// let xml_data = r#"
/// <people>
///   <person id="1">
///     <name>Alice</name>
///     <age>30</age>
///   </person>
///   <person id="2">
///     <name>Bob</name>
///     <age>25</age>
///   </person>
/// </people>
/// "#;
///
/// // Create a reader from an in-memory buffer
/// let cursor = Cursor::new(xml_data);
/// let reader = XmlItemReaderBuilder::<Person>::new()
///     .tag("person")
///     .from_reader(cursor);
///
/// // Read all persons from the XML
/// let mut persons = Vec::new();
/// let mut person_count = 0;
/// while let Some(person) = reader.read().unwrap() {
///     persons.push(person);
///     person_count += 1;
/// }
///
/// assert_eq!(person_count, 2);
/// assert_eq!(persons[0].id, 1);
/// assert_eq!(persons[0].name, "Alice");
/// assert_eq!(persons[1].name, "Bob");
/// ```
pub struct XmlItemReaderBuilder<T: DeserializeOwned> {
    tag_name: Option<String>,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned> Default for XmlItemReaderBuilder<T> {
    fn default() -> Self {
        Self {
            tag_name: None,
            capacity: 1024,
            _marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned> XmlItemReaderBuilder<T> {
    /// Creates a new XML item reader builder.
    ///
    /// By default, it will:
    /// - Look for XML elements matching the type name
    /// - Use a buffer capacity of 1024 bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let builder = XmlItemReaderBuilder::<Person>::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the buffer capacity for the XML reader.
    ///
    /// Higher capacity can improve performance for larger XML documents
    /// but will use more memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Increase buffer capacity for better performance with large files
    /// let builder = XmlItemReaderBuilder::<Person>::new()
    ///     .capacity(4096);
    /// ```
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Sets the XML tag name to search for items.
    ///
    /// This method specifies which XML element represents a single item.
    /// The reader will look for elements with this tag name and deserialize
    /// them into the target type.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Look for <person> elements in the XML
    /// let builder = XmlItemReaderBuilder::<Person>::new()
    ///     .tag("person");
    /// ```
    pub fn tag<S: AsRef<str>>(mut self, tag_name: S) -> Self {
        self.tag_name = Some(tag_name.as_ref().to_string());
        self
    }

    /// Creates an XML item reader from a reader.
    ///
    /// This allows reading from any source that implements the `Read` trait,
    /// such as files, network streams, or in-memory buffers.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    /// use std::io::Cursor;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Create XML data with two persons
    /// let xml_data = r#"
    /// <people>
    ///   <person>
    ///     <name>Alice</name>
    ///     <age>30</age>
    ///   </person>
    ///   <person>
    ///     <name>Bob</name>
    ///     <age>25</age>
    ///   </person>
    /// </people>
    /// "#;
    ///
    /// // Create a reader from an in-memory buffer
    /// let cursor = Cursor::new(xml_data);
    /// let reader = XmlItemReaderBuilder::<Person>::new()
    ///     .tag("person")
    ///     .from_reader(cursor);
    ///
    /// // Read and process each person
    /// let first_person = reader.read().unwrap().unwrap();
    /// assert_eq!(first_person.name, "Alice");
    /// assert_eq!(first_person.age, 30);
    ///
    /// let second_person = reader.read().unwrap().unwrap();
    /// assert_eq!(second_person.name, "Bob");
    /// assert_eq!(second_person.age, 25);
    ///
    /// // No more persons
    /// assert!(reader.read().unwrap().is_none());
    /// ```
    pub fn from_reader<R: Read + 'static>(self, reader: R) -> XmlItemReader<R, T> {
        let tag = match self.tag_name {
            Some(tag) => tag.into_bytes(),
            None => {
                // Default tag name is derived from the type name
                let type_str = type_name::<T>();
                let tag_name = type_str.split("::").last().unwrap_or(type_str);
                tag_name.as_bytes().to_vec()
            }
        };

        XmlItemReader::with_tag(reader, self.capacity, tag)
    }

    /// Creates an XML item reader from a file path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
    /// use spring_batch_rs::core::item::ItemReader;
    /// use serde::Deserialize;
    /// use std::path::Path;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Person {
    ///     #[serde(rename = "@id")]
    ///     id: i32,
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// // Read from an XML file
    /// let reader = XmlItemReaderBuilder::<Person>::new()
    ///     .tag("person")
    ///     .from_path("data/persons.xml")
    ///     .unwrap();
    ///
    /// // Process each person from the file
    /// while let Some(person) = reader.read().unwrap() {
    ///     println!("Read person: {} (id: {})", person.name, person.id);
    /// }
    /// ```
    pub fn from_path<P: AsRef<Path>>(self, path: P) -> Result<XmlItemReader<File, T>, BatchError> {
        let file_path = path.as_ref();
        let file = File::open(file_path).map_err(|e| {
            error!("Failed to open XML file {}: {}", file_path.display(), e);
            BatchError::ItemReader(format!(
                "Failed to open XML file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(self.from_reader(file))
    }
}

/// A simple reader that reads items from an XML file.
///
/// This reader parses XML content and deserializes elements with a specific tag
/// into the desired type. It handles XML attributes, nested elements, and text content.
///
/// # Examples
///
/// Reading complex nested XML structures:
///
/// ```
/// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// // Define a nested structure matching our XML format
/// #[derive(Debug, Deserialize)]
/// struct Address {
///     street: String,
///     city: String,
///     country: String,
/// }
///
/// #[derive(Debug, Deserialize)]
/// struct Person {
///     #[serde(rename = "@id")]
///     id: i32,
///     name: String,
///     age: i32,
///     address: Address,
/// }
///
/// // Create XML with nested elements
/// let xml_data = r#"
/// <directory>
///   <person id="1">
///     <name>Alice</name>
///     <age>30</age>
///     <address>
///       <street>123 Main St</street>
///       <city>Springfield</city>
///       <country>USA</country>
///     </address>
///   </person>
/// </directory>
/// "#;
///
/// // Create a reader from the XML
/// let cursor = Cursor::new(xml_data);
/// let reader = XmlItemReaderBuilder::<Person>::new()
///     .tag("person")
///     .from_reader(cursor);
///
/// // Read and verify the person with nested address
/// let person = reader.read().unwrap().unwrap();
/// assert_eq!(person.id, 1);
/// assert_eq!(person.name, "Alice");
/// assert_eq!(person.address.street, "123 Main St");
/// assert_eq!(person.address.city, "Springfield");
/// assert_eq!(person.address.country, "USA");
/// ```
pub struct XmlItemReader<R, T> {
    reader: RefCell<XmlReader<BufReader<R>>>,
    buffer: RefCell<Vec<u8>>,
    item_tag_name: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<R: Read, T: DeserializeOwned> XmlItemReader<R, T> {
    /// Creates a new XML item reader with a specific tag name.
    fn with_tag<S: AsRef<[u8]>>(rdr: R, capacity: usize, tag: S) -> Self {
        let buf_reader = BufReader::with_capacity(capacity, rdr);
        let mut xml_reader = XmlReader::from_reader(buf_reader);
        xml_reader.config_mut().trim_text(true);

        Self {
            reader: RefCell::new(xml_reader),
            buffer: RefCell::new(Vec::with_capacity(1024)),
            item_tag_name: tag.as_ref().to_vec(),
            _marker: PhantomData,
        }
    }
}

impl<R: Read, T: DeserializeOwned> ItemReader<T> for XmlItemReader<R, T> {
    fn read(&self) -> ItemReaderResult<T> {
        let mut reader = self.reader.borrow_mut();
        let mut buffer = self.buffer.borrow_mut();

        let tag_name_str = str::from_utf8(&self.item_tag_name).unwrap_or("<binary>");
        debug!("Looking for tag: '{}'", tag_name_str);

        // Skip events until we find a start element matching our tag
        loop {
            buffer.clear();
            let event = reader
                .read_event_into(&mut buffer)
                .map_err(|e| BatchError::ItemReader(format!("XML parsing error: {}", e)))?;

            match event {
                Event::Start(ref e) => {
                    let e_name = e.name();
                    let name_ref = e_name.as_ref();
                    let tag_name = str::from_utf8(name_ref).unwrap_or("<binary>");

                    if name_ref == self.item_tag_name.as_slice() {
                        debug!("Found start tag: '{}'", tag_name);

                        // Extract the full XML for this element
                        let mut xml_string = String::new();
                        xml_string.push('<');
                        if let Ok(name) = str::from_utf8(tag_name.as_ref()) {
                            xml_string.push_str(name);
                        }
                        for attr in e.attributes().flatten() {
                            xml_string.push(' ');
                            if let Ok(key) = str::from_utf8(attr.key.as_ref()) {
                                xml_string.push_str(key);
                            }
                            xml_string.push_str("=\"");
                            if let Ok(value) = str::from_utf8(attr.value.as_ref()) {
                                xml_string.push_str(value);
                            }
                            xml_string.push('"');
                        }
                        xml_string.push('>');

                        // Continue reading to get the content
                        let mut depth = 1;
                        while depth > 0 {
                            buffer.clear();
                            match reader.read_event_into(&mut buffer) {
                                Ok(Event::Start(ref start)) => {
                                    depth += 1;
                                    let s_name = start.name();
                                    if let Ok(name) = str::from_utf8(s_name.as_ref()) {
                                        xml_string.push('<');
                                        xml_string.push_str(name);

                                        // Add attributes
                                        for attr in start.attributes().flatten() {
                                            xml_string.push(' ');
                                            if let Ok(key) = str::from_utf8(attr.key.as_ref()) {
                                                xml_string.push_str(key);
                                            }
                                            xml_string.push_str("=\"");
                                            if let Ok(value) = str::from_utf8(attr.value.as_ref()) {
                                                xml_string.push_str(value);
                                            }
                                            xml_string.push('"');
                                        }
                                        xml_string.push('>');
                                    }
                                }
                                Ok(Event::End(ref end)) => {
                                    depth -= 1;
                                    let e_name = end.name();
                                    if let Ok(name) = str::from_utf8(e_name.as_ref()) {
                                        xml_string.push_str("</");
                                        xml_string.push_str(name);
                                        xml_string.push('>');
                                    }
                                }
                                Ok(Event::Text(ref text)) => {
                                    // For text nodes, just add their raw content
                                    let bytes = text.as_ref();
                                    if let Ok(s) = str::from_utf8(bytes) {
                                        xml_string.push_str(s);
                                    }
                                }
                                Ok(Event::CData(ref cdata)) => {
                                    // For CDATA, wrap in CDATA tags
                                    let bytes = cdata.as_ref();
                                    if let Ok(s) = str::from_utf8(bytes) {
                                        xml_string.push_str("<![CDATA[");
                                        xml_string.push_str(s);
                                        xml_string.push_str("]]>");
                                    }
                                }
                                Ok(Event::Eof) => {
                                    return Err(BatchError::ItemReader(
                                        "Unexpected end of file".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    return Err(BatchError::ItemReader(format!(
                                        "Error reading XML: {}",
                                        e
                                    )));
                                }
                                _ => { /* Ignore other events */ }
                            }
                        }

                        debug!("Finished reading XML item: {}", xml_string);

                        // Now deserialize the complete XML string
                        match from_str(&xml_string) {
                            Ok(item) => return Ok(Some(item)),
                            Err(e) => {
                                error!(
                                    "Failed to deserialize XML item: {} from: {}",
                                    e, xml_string
                                );
                                continue; // Skip this item and try the next one
                            }
                        }
                    }
                }
                Event::Eof => {
                    debug!("Reached end of file");
                    return Ok(None);
                }
                _ => continue, // Skip other events
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    // This tells serde to deserialize from the XML tag "TestItem"
    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    #[serde(rename = "TestItem")]
    struct TestItem {
        name: String,
        value: i32,
    }

    // Complex nested structures for testing
    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct EngineSpecs {
        #[serde(rename = "@type")]
        engine_type: String,
        #[serde(rename = "@cylinders")]
        cylinders: i32,
        horsepower: i32,
        #[serde(rename = "fuelEfficiency")]
        fuel_efficiency: f32,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    struct Features {
        #[serde(rename = "feature", default)]
        items: Vec<String>,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
    #[serde(rename = "vehicle")]
    struct Vehicle {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@category")]
        category: String,
        make: String,
        model: String,
        year: i32,
        engine: EngineSpecs,
        features: Features,
    }

    #[test]
    fn test_xml_reader() {
        let xml_content = r#"
            <items>
                <TestItem>
                    <name>test1</name>
                    <value>42</value>
                </TestItem>
                <TestItem>
                    <name>test2</name>
                    <value>43</value>
                </TestItem>
            </items>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        // Use builder to create the reader
        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .capacity(1024)
            .from_path(temp_file.path())
            .unwrap();

        let item1 = reader.read().unwrap().unwrap();
        assert_eq!(
            item1,
            TestItem {
                name: "test1".to_string(),
                value: 42,
            }
        );

        let item2 = reader.read().unwrap().unwrap();
        assert_eq!(
            item2,
            TestItem {
                name: "test2".to_string(),
                value: 43,
            }
        );

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_xml_reader_with_custom_tag() {
        let xml_content = r#"
            <root>
                <car>
                    <name>test1</name>
                    <value>42</value>
                </car>
                <car>
                    <name>test2</name>
                    <value>43</value>
                </car>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("car")
            .capacity(1024)
            .from_path(temp_file.path())
            .unwrap();

        let item1 = reader.read().unwrap().unwrap();
        assert_eq!(
            item1,
            TestItem {
                name: "test1".to_string(),
                value: 42,
            }
        );

        let item2 = reader.read().unwrap().unwrap();
        assert_eq!(
            item2,
            TestItem {
                name: "test2".to_string(),
                value: 43,
            }
        );

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_complex_nested_objects() {
        let xml_content = r#"
            <root>
                <vehicle id="v001" category="sedan">
                    <make>Toyota</make>
                    <model>Camry</model>
                    <year>2022</year>
                    <engine type="hybrid" cylinders="4">
                        <horsepower>208</horsepower>
                        <fuelEfficiency>4.5</fuelEfficiency>
                    </engine>
                    <features>
                        <feature>Bluetooth</feature>
                        <feature>Navigation</feature>
                        <feature>Leather Seats</feature>
                    </features>
                </vehicle>
                <vehicle id="v002" category="suv">
                    <make>Honda</make>
                    <model>CR-V</model>
                    <year>2023</year>
                    <engine type="gasoline" cylinders="4">
                        <horsepower>190</horsepower>
                        <fuelEfficiency>7.2</fuelEfficiency>
                    </engine>
                    <features>
                        <feature>All-wheel drive</feature>
                        <feature>Sunroof</feature>
                    </features>
                </vehicle>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<Vehicle>::new()
            .tag("vehicle")
            .capacity(1024)
            .from_path(temp_file.path())
            .unwrap();

        // First item
        let vehicle1 = reader.read().unwrap().unwrap();
        assert_eq!(vehicle1.id, "v001");
        assert_eq!(vehicle1.category, "sedan");
        assert_eq!(vehicle1.make, "Toyota");
        assert_eq!(vehicle1.model, "Camry");
        assert_eq!(vehicle1.year, 2022);
        assert_eq!(vehicle1.engine.engine_type, "hybrid");
        assert_eq!(vehicle1.engine.cylinders, 4);
        assert_eq!(vehicle1.engine.horsepower, 208);
        assert_eq!(vehicle1.engine.fuel_efficiency, 4.5);
        assert_eq!(vehicle1.features.items.len(), 3);
        assert_eq!(vehicle1.features.items[0], "Bluetooth");
        assert_eq!(vehicle1.features.items[1], "Navigation");
        assert_eq!(vehicle1.features.items[2], "Leather Seats");

        // Second item
        let vehicle2 = reader.read().unwrap().unwrap();
        assert_eq!(vehicle2.id, "v002");
        assert_eq!(vehicle2.category, "suv");
        assert_eq!(vehicle2.make, "Honda");
        assert_eq!(vehicle2.model, "CR-V");
        assert_eq!(vehicle2.year, 2023);
        assert_eq!(vehicle2.engine.engine_type, "gasoline");
        assert_eq!(vehicle2.engine.cylinders, 4);
        assert_eq!(vehicle2.engine.horsepower, 190);
        assert_eq!(vehicle2.engine.fuel_efficiency, 7.2);
        assert_eq!(vehicle2.features.items.len(), 2);
        assert_eq!(vehicle2.features.items[0], "All-wheel drive");
        assert_eq!(vehicle2.features.items[1], "Sunroof");

        // No more items
        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_xml_reader_builder() {
        let xml_content = r#"
            <data>
                <vehicle id="v001" category="sedan">
                    <make>Toyota</make>
                    <model>Camry</model>
                    <year>2022</year>
                    <engine type="hybrid" cylinders="4">
                        <horsepower>208</horsepower>
                        <fuelEfficiency>4.5</fuelEfficiency>
                    </engine>
                    <features>
                        <feature>Bluetooth</feature>
                        <feature>Navigation</feature>
                    </features>
                </vehicle>
            </data>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        // Use builder to create the reader with custom tag and capacity
        let reader = XmlItemReaderBuilder::<Vehicle>::new()
            .tag("vehicle")
            .capacity(2048)
            .from_path(temp_file.path())
            .unwrap();

        // Verify the reader works correctly
        let vehicle = reader.read().unwrap().unwrap();
        assert_eq!(vehicle.id, "v001");
        assert_eq!(vehicle.make, "Toyota");
        assert_eq!(vehicle.model, "Camry");
        assert_eq!(vehicle.year, 2022);

        // No more items
        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_empty_xml_file() {
        // Empty XML file
        let xml_content = "<root></root>";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_path(temp_file.path())
            .unwrap();

        // Should return None immediately - no items to read
        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_xml_with_empty_tags() {
        // XML with empty tags that match our target
        let xml_content = r#"
            <root>
                <TestItem>
                    <name></name>
                    <value>0</value>
                </TestItem>
                <TestItem>
                    <name></name>
                    <value>0</value>
                </TestItem>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_path(temp_file.path())
            .unwrap();

        // Both items should be read as default values
        let item1 = reader.read().unwrap().unwrap();
        assert_eq!(item1.name, "");
        assert_eq!(item1.value, 0);

        let item2 = reader.read().unwrap().unwrap();
        assert_eq!(item2.name, "");
        assert_eq!(item2.value, 0);

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_xml_with_attributes() {
        // Define a type that captures XML attributes
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct ItemWithAttrs {
            #[serde(rename = "@id")]
            id: String,
            #[serde(rename = "@type")]
            item_type: String,
            content: String,
        }

        let xml_content = r#"
            <root>
                <item id="1" type="normal">
                    <content>First item</content>
                </item>
                <item id="2" type="special">
                    <content>Second item</content>
                </item>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<ItemWithAttrs>::new()
            .tag("item")
            .from_path(temp_file.path())
            .unwrap();

        let item1 = reader.read().unwrap().unwrap();
        assert_eq!(item1.id, "1");
        assert_eq!(item1.item_type, "normal");
        assert_eq!(item1.content, "First item");

        let item2 = reader.read().unwrap().unwrap();
        assert_eq!(item2.id, "2");
        assert_eq!(item2.item_type, "special");
        assert_eq!(item2.content, "Second item");

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_xml_with_cdata() {
        // Test with CDATA sections which may contain special characters
        let xml_content = r#"
            <root>
                <TestItem>
                    <name><![CDATA[name with <special> & chars]]></name>
                    <value>42</value>
                </TestItem>
                <TestItem>
                    <name>regular name</name>
                    <value><![CDATA[55]]></value>
                </TestItem>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_path(temp_file.path())
            .unwrap();

        let item1 = reader.read().unwrap().unwrap();
        assert_eq!(item1.name, "name with <special> & chars");
        assert_eq!(item1.value, 42);

        let item2 = reader.read().unwrap().unwrap();
        assert_eq!(item2.name, "regular name");
        assert_eq!(item2.value, 55);

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_malformed_xml() {
        // Malformed XML with unclosed tags
        let xml_content = r#"
            <root>
                <TestItem>
                    <name>test1</name>
                    <value>42
                </TestItem>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_path(temp_file.path())
            .unwrap();

        // Should return an error when trying to read
        let result = reader.read();
        assert!(result.is_err());
    }

    #[test]
    fn test_xml_type_mismatch() {
        // XML with a value that doesn't match the expected type
        let xml_content = r#"
            <root>
                <TestItem>
                    <name>test1</name>
                    <value>not_a_number</value>
                </TestItem>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_path(temp_file.path())
            .unwrap();

        // Should return an error when trying to deserialize
        let result = reader.read();
        assert!(result.is_ok()); // The outer result is Ok
        assert!(result.unwrap().is_none()); // But it should have skipped the bad item
    }

    #[test]
    fn test_default_tag_inference() {
        // When tag is not specified, it should use the type name
        let xml_content = r#"
            <root>
                <TestItem>
                    <name>test1</name>
                    <value>42</value>
                </TestItem>
            </root>
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        // Notice we don't specify the tag
        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .from_path(temp_file.path())
            .unwrap();

        // Should infer the tag name from the type
        let item = reader.read().unwrap().unwrap();
        assert_eq!(item.name, "test1");
        assert_eq!(item.value, 42);

        assert!(reader.read().unwrap().is_none());
    }

    #[test]
    fn test_read_from_memory() {
        // Test reading directly from a memory buffer
        let xml_content = r#"
            <root>
                <TestItem>
                    <name>memory test</name>
                    <value>100</value>
                </TestItem>
            </root>
        "#;

        // Create an in-memory reader
        let reader = XmlItemReaderBuilder::<TestItem>::new()
            .tag("TestItem")
            .from_reader(xml_content.as_bytes());

        // Should read correctly from memory
        let item = reader.read().unwrap().unwrap();
        assert_eq!(item.name, "memory test");
        assert_eq!(item.value, 100);

        assert!(reader.read().unwrap().is_none());
    }
}
