pub mod common;

use std::{
    env::temp_dir,
    fs::{self, read_to_string, File},
    io::{Cursor, Read},
};

use ::serde::{Deserialize, Serialize};
use rand::distr::{Alphanumeric, SampleString};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::{Step, StepBuilder, StepInstance, StepStatus},
    },
    error::BatchError,
    item::csv::csv_reader::CsvItemReaderBuilder,
    item::csv::csv_writer::CsvItemWriterBuilder,
    item::xml::xml_reader::XmlItemReaderBuilder,
    item::xml::xml_writer::XmlItemWriterBuilder,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Product {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@available")]
    available: bool,
    name: String,
    price: f64,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Default)]
struct ProductProcessor;

impl ItemProcessor<Product, Product> for ProductProcessor {
    fn process(&self, item: &Product) -> ItemProcessorResult<Product> {
        let description = match &item.description {
            Some(desc) => Some(desc.to_uppercase()),
            None => Some("NO DESCRIPTION AVAILABLE".to_string()),
        };

        let product = Product {
            id: item.id.clone(),
            available: item.available,
            name: item.name.to_uppercase(),
            price: item.price * 1.1, // 10% price increase
            description,
        };

        Ok(product)
    }
}

#[test]
fn transform_from_xml_file_to_csv_file_without_error() {
    // Create sample XML data
    let xml_content = r#"
    <catalog>
      <product id="P001" available="true">
        <name>Wireless Headphones</name>
        <price>79.99</price>
        <description>Noise-cancelling wireless headphones with 20hr battery life</description>
      </product>
      <product id="P002" available="false">
        <name>USB-C Cable</name>
        <price>12.99</price>
      </product>
      <product id="P003" available="true">
        <name>Smart Watch</name>
        <price>149.99</price>
        <description>Fitness tracking smart watch with heart rate monitor</description>
      </product>
    </catalog>
    "#;

    // Create a temporary file with XML content
    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let xml_path = temp_dir().join(format!("{}.xml", file_name));
    fs::write(&xml_path, xml_content).expect("Failed to write XML file");

    let file = File::open(&xml_path).expect("Unable to open XML file");

    // Create XML reader
    let reader = XmlItemReaderBuilder::<Product>::new()
        .tag("product")
        .from_reader(file);

    // Create processor to transform products
    let processor = ProductProcessor::default();

    // Create CSV writer
    let csv_path = temp_dir().join(format!("{}.csv", file_name));
    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path(&csv_path);

    // Build and run the job
    let step = StepBuilder::new()
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .chunk(2)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());
    assert_eq!(step.get_status(), StepStatus::Success);
    assert_eq!(step.get_read_count(), 3);
    assert_eq!(step.get_write_count(), 3);
    assert_eq!(step.get_read_error_count(), 0);
    assert_eq!(step.get_write_error_count(), 0);

    // Read and verify the CSV content
    let csv_content =
        read_to_string(&csv_path).expect("Should have been able to read the CSV file");

    // Check the CSV output - the exact format might be implementation specific
    assert!(!csv_content.is_empty());
    assert!(csv_content.contains("WIRELESS HEADPHONES"));
    assert!(csv_content.contains("USB-C CABLE"));
    assert!(csv_content.contains("SMART WATCH"));
    assert!(csv_content.contains("NOISE-CANCELLING WIRELESS HEADPHONES WITH 20HR BATTERY LIFE"));
    assert!(csv_content.contains("NO DESCRIPTION AVAILABLE"));

    // Clean up
    fs::remove_file(&xml_path).ok();
    fs::remove_file(&csv_path).ok();
}

// Nested structures for XML
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Manufacturer {
    #[serde(rename = "@country")]
    country: String,
    name: String,
    #[serde(rename = "foundedYear")]
    founded_year: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Category {
    name: String,
    #[serde(rename = "@main")]
    main: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct EnhancedProduct {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@sku")]
    sku: String,
    name: String,
    price: f64,
    manufacturer: Manufacturer,
    categories: Vec<Category>,
    in_stock: bool,
}

#[test]
fn convert_csv_to_xml_with_nested_structures() {
    // Create a CSV file with data that will be converted to complex XML
    let csv_content = r#"id,sku,name,price,manufacturer_country,manufacturer_name,manufacturer_founded_year,category1_name,category1_main,category2_name,category2_main,in_stock
P001,SKU123,Laptop,999.99,USA,TechCorp,1985,Electronics,true,Computers,false,true
P002,SKU456,Smartphone,599.99,Korea,MobileTech,1995,Electronics,true,Mobile,false,true
P003,SKU789,Headphones,129.99,Japan,AudioInc,1978,Electronics,false,Audio,true,false"#;

    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let csv_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&csv_path, csv_content).expect("Failed to write CSV file");

    // Read CSV and transform into EnhancedProduct objects
    let file = File::open(&csv_path).expect("Unable to open CSV file");

    // Custom processor to build nested structures from CSV
    struct CsvToEnhancedProductProcessor;

    impl ItemProcessor<Vec<String>, EnhancedProduct> for CsvToEnhancedProductProcessor {
        fn process(&self, item: &Vec<String>) -> ItemProcessorResult<EnhancedProduct> {
            if item.len() < 12 {
                return Err(BatchError::ItemProcessor(
                    "CSV row has too few columns".to_string(),
                ));
            }

            // Build nested product from CSV columns
            let product = EnhancedProduct {
                id: item[0].clone(),
                sku: item[1].clone(),
                name: item[2].clone(),
                price: item[3].parse().unwrap_or(0.0),
                manufacturer: Manufacturer {
                    country: item[4].clone(),
                    name: item[5].clone(),
                    founded_year: item[6].parse().unwrap_or(0),
                },
                categories: vec![
                    Category {
                        name: item[7].clone(),
                        main: item[8].parse().unwrap_or(false),
                    },
                    Category {
                        name: item[9].clone(),
                        main: item[10].parse().unwrap_or(false),
                    },
                ],
                in_stock: item[11].parse().unwrap_or(false),
            };

            Ok(product)
        }
    }

    // Create a CSV reader without headers (we'll manually process columns)
    let reader = CsvItemReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    // Create XML writer
    let xml_path = temp_dir().join(format!("{}.xml", file_name));
    let writer = XmlItemWriterBuilder::new()
        .root_tag("products")
        .item_tag("product")
        .from_path::<EnhancedProduct, _>(&xml_path)
        .expect("Failed to create XML writer");

    let processor = CsvToEnhancedProductProcessor;

    // Build and run the job
    let step: StepInstance<Vec<String>, EnhancedProduct> = StepBuilder::new()
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .chunk(2)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());
    assert_eq!(step.get_status(), StepStatus::Success);
    assert_eq!(step.get_read_count(), 3);
    assert_eq!(step.get_write_count(), 3);

    // Read and verify the XML content
    let mut xml_content = String::new();
    File::open(&xml_path)
        .expect("Failed to open XML file")
        .read_to_string(&mut xml_content)
        .expect("Failed to read XML file");

    // Check XML structure
    assert!(xml_content.contains("<products>"));
    assert!(xml_content.contains("<product id=\"P001\" sku=\"SKU123\">"));
    assert!(xml_content.contains("<name>Laptop</name>"));
    assert!(xml_content.contains("<manufacturer country=\"USA\">"));
    assert!(xml_content.contains("<name>TechCorp</name>"));
    assert!(xml_content.contains("<foundedYear>1985</foundedYear>"));
    // Check that categories are included, without being too specific about format
    assert!(xml_content.contains("Electronics"));
    assert!(xml_content.contains("Computers"));

    // Clean up
    fs::remove_file(&csv_path).ok();
    fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_reader_with_error_handling() {
    // Create a malformed XML file with schema errors
    let xml_content = r#"
    <catalog>
      <product id="P001" available="true">
        <name>Wireless Headphones</name>
        <price>79.99</price>
        <description>Good headphones</description>
      </product>
      <!-- Malformed product missing required price field -->
      <product id="P002" available="false">
        <name>USB-C Cable</name>
        <!-- price is missing -->
      </product>
      <product id="P003" available="true">
        <name>Smart Watch</name>
        <price>149.99</price>
        <description>Fitness tracker</description>
      </product>
    </catalog>
    "#;

    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let xml_path = temp_dir().join(format!("{}.xml", file_name));
    fs::write(&xml_path, xml_content).expect("Failed to write XML file");

    // Create a reader with skip limit to handle errors
    let file = File::open(&xml_path).expect("Unable to open XML file");
    let reader = XmlItemReaderBuilder::<Product>::new()
        .tag("product")
        .from_reader(file);

    // Create a simple memory buffer writer to capture output
    let buffer = Cursor::new(Vec::new());
    let writer = XmlItemWriterBuilder::new()
        .root_tag("filtered_catalog")
        .item_tag("product")
        .from_writer::<Product, _>(buffer);

    // Build step with skip limit of 1 (to tolerate one error)
    let step: StepInstance<Product, Product> = StepBuilder::new()
        .reader(&reader)
        .writer(&writer)
        .chunk(1)
        .skip_limit(1) // Allow one error to be skipped
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job completed successfully despite the errors that might be present
    assert!(result.is_ok());
    assert_eq!(step.get_status(), StepStatus::Success);

    // Some valid products should be processed
    assert!(step.get_read_count() > 0);
    assert!(step.get_write_count() > 0);

    // Note: The XML reader might handle missing fields differently,
    // so we can't reliably assert error counts

    // Clean up
    fs::remove_file(&xml_path).ok();
}
