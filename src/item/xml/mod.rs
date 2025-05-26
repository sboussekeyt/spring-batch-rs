/// XML support for reading and writing structured data.
///
/// This module provides components for reading data from XML files and writing data to XML files
/// as part of a batch processing pipeline. The implementation uses `quick-xml` for efficient
/// XML parsing and serialization.
///
/// # Features
///
/// - Read XML documents with support for complex nested structures
/// - Write data to XML files with customizable root and item tags
/// - Support for XML attributes via serde's `#[serde(rename = "@attribute_name")]`
/// - Automatic type inference for tag names
///
/// # Examples
///
/// ## Reading from XML
///
/// ```
/// use spring_batch_rs::item::xml::XmlItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// // Define a data structure with XML attributes and nested elements
/// #[derive(Debug, Deserialize)]
/// struct Product {
///     #[serde(rename = "@id")]
///     id: String,
///     #[serde(rename = "@available")]
///     available: bool,
///     name: String,
///     price: f64,
///     #[serde(default)]
///     description: Option<String>,
/// }
///
/// // Sample XML data
/// let xml_data = r#"
/// <catalog>
///   <product id="P001" available="true">
///     <name>Wireless Headphones</name>
///     <price>79.99</price>
///     <description>Noise-cancelling wireless headphones with 20hr battery life</description>
///   </product>
///   <product id="P002" available="false">
///     <name>USB-C Cable</name>
///     <price>12.99</price>
///   </product>
/// </catalog>
/// "#;
///
/// // Create a reader from our XML
/// let cursor = Cursor::new(xml_data);
/// let reader = XmlItemReaderBuilder::<Product>::new()
///     .tag("product")
///     .from_reader(cursor);
///
/// // Read and process the products
/// let mut products = Vec::new();
/// while let Some(product) = reader.read().unwrap() {
///     products.push(product);
/// }
///
/// // Verify results
/// assert_eq!(products.len(), 2);
/// assert_eq!(products[0].id, "P001");
/// assert_eq!(products[0].name, "Wireless Headphones");
/// assert_eq!(products[0].price, 79.99);
/// assert!(products[0].available);
/// assert!(products[0].description.is_some());
///
/// assert_eq!(products[1].id, "P002");
/// assert_eq!(products[1].name, "USB-C Cable");
/// assert_eq!(products[1].price, 12.99);
/// assert!(!products[1].available);
/// assert!(products[1].description.is_none());
/// ```
///
/// ## Writing to XML
///
/// ```
/// use spring_batch_rs::item::xml::xml_writer::XmlItemWriterBuilder;
/// use spring_batch_rs::core::item::ItemWriter;
/// use serde::Serialize;
/// use std::io::Cursor;
///
/// // Define a data structure for serialization
/// #[derive(Serialize)]
/// struct Product {
///     #[serde(rename = "@id")]
///     id: String,
///     #[serde(rename = "@in_stock")]
///     in_stock: bool,
///     name: String,
///     price: f64,
///     categories: Vec<String>,
/// }
///
/// // Create some products
/// let products = vec![
///     Product {
///         id: "P001".to_string(),
///         in_stock: true,
///         name: "Smartphone".to_string(),
///         price: 599.99,
///         categories: vec!["Electronics".to_string(), "Mobile".to_string()],
///     },
///     Product {
///         id: "P002".to_string(),
///         in_stock: false,
///         name: "Laptop".to_string(),
///         price: 1299.99,
///         categories: vec!["Electronics".to_string(), "Computers".to_string()],
///     },
/// ];
///
/// // Create a writer with a memory buffer
/// let buffer = Cursor::new(Vec::new());
/// let writer = XmlItemWriterBuilder::<Product>::new()
///     .root_tag("catalog")
///     .item_tag("product")
///     .from_writer(buffer);
///
/// // Write the products to XML
/// writer.open().unwrap();
/// writer.write(&products).unwrap();
/// writer.close().unwrap();
///
/// // The resulting XML would look similar to:
/// // <catalog>
/// //   <product id="P001" in_stock="true">
/// //     <name>Smartphone</name>
/// //     <price>599.99</price>
/// //     <categories>Electronics</categories>
/// //     <categories>Mobile</categories>
/// //   </product>
/// //   <product id="P002" in_stock="false">
/// //     <name>Laptop</name>
/// //     <price>1299.99</price>
/// //     <categories>Electronics</categories>
/// //     <categories>Computers</categories>
/// //   </product>
/// // </catalog>
/// ```
pub mod xml_reader;
pub mod xml_writer;

pub use xml_reader::{XmlItemReader, XmlItemReaderBuilder};
pub use xml_writer::XmlItemWriter;
pub use xml_writer::XmlItemWriterBuilder;
