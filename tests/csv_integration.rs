pub mod common;

use std::{
    env::temp_dir,
    fs::{self, read_to_string, File},
    io::Cursor,
};

use ::serde::{Deserialize, Serialize};
use rand::distr::{Alphanumeric, SampleString};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::{StepBuilder, StepStatus},
    },
    item::{
        csv::{csv_reader::CsvItemReaderBuilder, csv_writer::CsvItemWriterBuilder},
        xml::{xml_reader::XmlItemReaderBuilder, xml_writer::XmlItemWriterBuilder},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Product {
    id: String,
    name: String,
    price: f64,
    #[serde(default)]
    description: Option<String>,
    available: bool,
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
            name: item.name.to_uppercase(),
            price: item.price * 1.1, // 10% price increase
            description,
            available: item.available,
        };

        Ok(product)
    }
}

#[test]
fn transform_from_csv_file_to_csv_file_without_error() {
    // Create sample CSV data
    let csv_content = r#"id,name,price,description,available
P001,Wireless Headphones,79.99,"Noise-cancelling wireless headphones with 20hr battery life",true
P002,USB-C Cable,12.99,,false
P003,Smart Watch,149.99,"Fitness tracking smart watch with heart rate monitor",true"#;

    // Create a temporary file with CSV content
    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let input_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&input_path, csv_content).expect("Failed to write CSV file");

    let file = File::open(&input_path).expect("Unable to open CSV file");

    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(file);

    // Create processor to transform products
    let processor = ProductProcessor;

    // Create CSV writer
    let output_path = temp_dir().join(format!("output_{}.csv", file_name));
    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path(&output_path);

    // Build and run the job
    let step = StepBuilder::new("test")
        .chunk::<Product, Product>(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());

    // Read and verify the CSV content
    let csv_content =
        read_to_string(&output_path).expect("Should have been able to read the CSV file");

    // Check the CSV output
    assert!(!csv_content.is_empty());
    assert!(csv_content.contains("WIRELESS HEADPHONES"));
    assert!(csv_content.contains("USB-C CABLE"));
    assert!(csv_content.contains("SMART WATCH"));
    assert!(csv_content.contains("NOISE-CANCELLING WIRELESS HEADPHONES WITH 20HR BATTERY LIFE"));
    assert!(csv_content.contains("NO DESCRIPTION AVAILABLE"));

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct EnhancedProduct {
    id: String,
    sku: String,
    name: String,
    price: f64,
    manufacturer_country: String,
    manufacturer_name: String,
    manufacturer_founded_year: i32,
    category1_name: String,
    category1_main: bool,
    category2_name: String,
    category2_main: bool,
    in_stock: bool,
}

#[test]
fn test_csv_reader_with_error_handling() {
    // Create a CSV file with some malformed data
    let csv_content = r#"id,name,price,description,available
P001,Wireless Headphones,79.99,"Good headphones",true
P002,USB-C Cable,invalid_price,,false
P003,Smart Watch,149.99,"Fitness tracker",true"#;

    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let csv_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&csv_path, csv_content).expect("Failed to write CSV file");

    // Create a reader with skip limit to handle errors
    let file = File::open(&csv_path).expect("Unable to open CSV file");
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(file);

    // Create a simple memory buffer writer to capture output
    let buffer = Cursor::new(Vec::new());
    let writer = CsvItemWriterBuilder::<Product>::new()
        .has_headers(true)
        .from_writer(buffer);

    // Create processor to transform products
    let processor = ProductProcessor;

    // Build step with skip limit of 1 (to tolerate one error)
    let step = StepBuilder::new("test")
        .chunk::<Product, Product>(1)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .skip_limit(1) // Allow one error to be skipped
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job completed successfully despite the errors
    assert!(result.is_ok());

    let step_execution = job.get_step_execution("test").unwrap();

    assert!(step_execution.status == StepStatus::Success);
    assert!(step_execution.read_count == 2);
    assert!(step_execution.write_count == 2);
    assert!(step_execution.process_count == 2);
    assert!(step_execution.read_error_count == 1);
    assert!(step_execution.write_error_count == 0);

    // Clean up
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_csv_writer_with_custom_delimiter() {
    // Create sample CSV data
    let csv_content = r#"id,name,price,description,available
P001,Wireless Headphones,79.99,"Good headphones",true
P002,USB-C Cable,12.99,,false
P003,Smart Watch,149.99,"Fitness tracker",true"#;

    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let input_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&input_path, csv_content).expect("Failed to write CSV file");

    let file = File::open(&input_path).expect("Unable to open CSV file");

    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(file);

    // Create processor to transform products
    let processor = ProductProcessor;

    // Create CSV writer with semicolon delimiter
    let output_path = temp_dir().join(format!("output_{}.csv", file_name));
    let writer = CsvItemWriterBuilder::<Product>::new()
        .has_headers(true)
        .delimiter(b';')
        .from_path(&output_path);

    // Build and run the job
    let step = StepBuilder::new("test")
        .chunk::<Product, Product>(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());

    let step_execution = job.get_step_execution("test").unwrap();

    assert!(step_execution.status == StepStatus::Success);
    assert!(step_execution.read_count == 3);
    assert!(step_execution.write_count == 3);
    assert!(step_execution.process_count == 3);
    assert!(step_execution.read_error_count == 0);
    assert!(step_execution.write_error_count == 0);

    // Read and verify the CSV content
    let csv_content =
        read_to_string(&output_path).expect("Should have been able to read the CSV file");

    // Check that the delimiter is a semicolon
    assert!(csv_content.contains(";"));
    assert!(!csv_content.contains(","));

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn transform_from_csv_to_xml() {
    // Create sample CSV data
    let csv_content = r#"id,name,price,description,available
P001,Wireless Headphones,79.99,"Noise-cancelling wireless headphones with 20hr battery life",true
P002,USB-C Cable,12.99,,false
P003,Smart Watch,149.99,"Fitness tracking smart watch with heart rate monitor",true"#;

    // Create a temporary file with CSV content
    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let input_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&input_path, csv_content).expect("Failed to write CSV file");

    let file = File::open(&input_path).expect("Unable to open CSV file");

    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(file);

    // Create processor to transform products
    let processor = ProductProcessor;

    // Create XML writer
    let output_path = temp_dir().join(format!("output_{}.xml", file_name));
    let writer = XmlItemWriterBuilder::<Product>::new()
        .root_tag("products")
        .item_tag("product")
        .from_path(&output_path)
        .expect("Failed to create XML writer");

    // Build and run the job
    let step = StepBuilder::new("test")
        .chunk(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());

    // Read and verify the XML content
    let xml_content =
        read_to_string(&output_path).expect("Should have been able to read the XML file");

    // Check XML structure and content
    assert!(xml_content.contains("<products>"));
    assert!(xml_content.contains("<product>"));
    assert!(xml_content.contains("WIRELESS HEADPHONES"));
    assert!(xml_content.contains("USB-C CABLE"));
    assert!(xml_content.contains("SMART WATCH"));
    assert!(xml_content.contains("NOISE-CANCELLING WIRELESS HEADPHONES WITH 20HR BATTERY LIFE"));
    assert!(xml_content.contains("NO DESCRIPTION AVAILABLE"));

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn transform_from_xml_to_csv() {
    // Create sample XML data
    let xml_content = r#"
    <products>
      <product>
        <id>P001</id>
        <name>Wireless Headphones</name>
        <price>79.99</price>
        <description>Noise-cancelling wireless headphones with 20hr battery life</description>
        <available>true</available>
      </product>
      <product>
        <id>P002</id>
        <name>USB-C Cable</name>
        <price>12.99</price>
        <available>false</available>
      </product>
      <product>
        <id>P003</id>
        <name>Smart Watch</name>
        <price>149.99</price>
        <description>Fitness tracking smart watch with heart rate monitor</description>
        <available>true</available>
      </product>
    </products>
    "#;

    // Create a temporary file with XML content
    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let input_path = temp_dir().join(format!("{}.xml", file_name));
    fs::write(&input_path, xml_content).expect("Failed to write XML file");

    let file = File::open(&input_path).expect("Unable to open XML file");

    // Create XML reader
    let reader = XmlItemReaderBuilder::<Product>::new()
        .tag("product")
        .from_reader(file);

    // Create processor to transform products
    let processor = ProductProcessor;

    // Create CSV writer
    let output_path = temp_dir().join(format!("output_{}.csv", file_name));
    let writer = CsvItemWriterBuilder::<Product>::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path(&output_path);

    // Build and run the job
    let step = StepBuilder::new("test")
        .chunk(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job results
    assert!(result.is_ok());
    // Read and verify the CSV content
    let csv_content =
        read_to_string(&output_path).expect("Should have been able to read the CSV file");

    // Check CSV structure and content
    assert!(csv_content.contains("id,name,price,description,available"));
    assert!(csv_content.contains("WIRELESS HEADPHONES"));
    assert!(csv_content.contains("USB-C CABLE"));
    assert!(csv_content.contains("SMART WATCH"));
    assert!(csv_content.contains("NOISE-CANCELLING WIRELESS HEADPHONES WITH 20HR BATTERY LIFE"));
    assert!(csv_content.contains("NO DESCRIPTION AVAILABLE"));

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_bidirectional_transformation() {
    // Create sample CSV data
    let csv_content = r#"id,name,price,description,available
P001,Wireless Headphones,79.99,"Noise-cancelling wireless headphones with 20hr battery life",true
P002,USB-C Cable,12.99,,false
P003,Smart Watch,149.99,"Fitness tracking smart watch with heart rate monitor",true"#;

    let file_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let csv_path = temp_dir().join(format!("{}.csv", file_name));
    fs::write(&csv_path, csv_content).expect("Failed to write CSV file");

    // Step 1: CSV to XML
    let file = File::open(&csv_path).expect("Unable to open CSV file");
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_reader(file);

    let xml_path = temp_dir().join(format!("intermediate_{}.xml", file_name));
    let writer = XmlItemWriterBuilder::<Product>::new()
        .root_tag("products")
        .item_tag("product")
        .from_path(&xml_path)
        .expect("Failed to create XML writer");

    let processor1 = ProductProcessor;
    let step1 = StepBuilder::new("test")
        .chunk::<Product, Product>(2)
        .reader(&reader)
        .processor(&processor1)
        .writer(&writer)
        .build();

    let job1 = JobBuilder::new().start(&step1).build();
    let result1 = job1.run();
    assert!(result1.is_ok());

    // Step 2: XML back to CSV
    let file = File::open(&xml_path).expect("Unable to open XML file");
    let reader = XmlItemReaderBuilder::<Product>::new()
        .tag("product")
        .from_reader(file);

    let final_csv_path = temp_dir().join(format!("final_{}.csv", file_name));
    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path(&final_csv_path);

    let processor2 = ProductProcessor;
    let step2 = StepBuilder::new("test")
        .chunk::<Product, Product>(2)
        .reader(&reader)
        .processor(&processor2)
        .writer(&writer)
        .build();

    let job2 = JobBuilder::new().start(&step2).build();
    let result2 = job2.run();
    assert!(result2.is_ok());

    // Verify final CSV content
    let final_csv_content =
        read_to_string(&final_csv_path).expect("Should have been able to read the final CSV file");

    // Check that the data was transformed twice (processor applied twice)
    assert!(final_csv_content.contains("WIRELESS HEADPHONES"));
    assert!(final_csv_content.contains("USB-C CABLE"));
    assert!(final_csv_content.contains("SMART WATCH"));
    assert!(
        final_csv_content.contains("NOISE-CANCELLING WIRELESS HEADPHONES WITH 20HR BATTERY LIFE")
    );
    assert!(final_csv_content.contains("NO DESCRIPTION AVAILABLE"));

    // Clean up
    fs::remove_file(&csv_path).ok();
    fs::remove_file(&xml_path).ok();
    fs::remove_file(&final_csv_path).ok();
}
