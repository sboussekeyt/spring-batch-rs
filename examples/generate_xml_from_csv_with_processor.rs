use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemReader},
        step::{Step, StepBuilder, StepExecution},
    },
    item::{
        csv::csv_reader::CsvItemReaderBuilder,
        xml::{xml_writer::XmlItemWriterBuilder, XmlItemReaderBuilder},
    },
    BatchError,
};

// Simple version that matches exactly the CSV format
#[derive(Debug, Deserialize, Clone)]
struct CsvHouse {
    id: String, // Use String to handle potential non-numeric values
    property_type: String,
    street: String,
    city: String,
    state: String,
    zip_code: String,
    country: String,
    price: String, // Use String to handle potential non-numeric values
    bedrooms: String,
    bathrooms: String,
    square_meters: String,
    year_built: String,
    has_garage: String,
    has_pool: String,
    has_garden: String,
    amenities: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct XmlAddress {
    street: String,
    city: String,
    state: String,
    zip_code: String,
    #[serde(rename = "@country")]
    country: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct XmlFeatures {
    #[serde(rename = "@has_garage")]
    has_garage: bool,
    #[serde(rename = "@has_pool")]
    has_pool: bool,
    #[serde(rename = "@has_garden")]
    has_garden: bool,
    amenities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct XmlHouse {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@type")]
    property_type: String,
    address: XmlAddress,
    price: i32,
    bedrooms: i32,
    bathrooms: i32,
    square_meters: i32,
    year_built: i32,
    features: XmlFeatures,
}

// Custom processor to convert from CSV format to XML format
struct HouseProcessor;

impl ItemProcessor<CsvHouse, XmlHouse> for HouseProcessor {
    fn process(&self, item: &CsvHouse) -> Result<XmlHouse, BatchError> {
        debug!("Processing CSV house: {:?}", item);

        // Extract and convert id
        let id = item.id.parse::<i32>().map_err(|e| {
            BatchError::ItemProcessor(format!("Failed to parse id '{}': {}", item.id, e))
        })?;

        // Extract amenities array
        let amenities_vec = item
            .amenities
            .trim_matches('"')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        debug!("Parsed amenities: {:?}", amenities_vec);

        // Parse numeric fields
        let price = item.price.parse::<i32>().unwrap_or(0);
        let bedrooms = item.bedrooms.parse::<i32>().unwrap_or(0);
        let bathrooms = item.bathrooms.parse::<i32>().unwrap_or(0);
        let square_meters = item.square_meters.parse::<i32>().unwrap_or(0);
        let year_built = item.year_built.parse::<i32>().unwrap_or(0);

        // Parse boolean fields
        let has_garage = item.has_garage.to_lowercase() == "true";
        let has_pool = item.has_pool.to_lowercase() == "true";
        let has_garden = item.has_garden.to_lowercase() == "true";

        Ok(XmlHouse {
            id,
            property_type: item.property_type.clone(),
            address: XmlAddress {
                street: item.street.clone(),
                city: item.city.clone(),
                state: item.state.clone(),
                zip_code: item.zip_code.clone(),
                country: item.country.clone(),
            },
            price,
            bedrooms,
            bathrooms,
            square_meters,
            year_built,
            features: XmlFeatures {
                has_garage,
                has_pool,
                has_garden,
                amenities: amenities_vec,
            },
        })
    }
}

fn main() -> Result<(), BatchError> {
    // Initialize logger
    env_logger::init();

    // Create output path in the examples/data directory
    let xml_output_path = std::path::Path::new("examples/data/output_houses.xml");

    info!("Reading CSV from examples/data/houses.csv");
    info!("Writing XML to {}", xml_output_path.display());

    // Create the CSV reader
    let csv_reader = CsvItemReaderBuilder::<CsvHouse>::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("examples/data/houses.csv");

    // Debug: check if the CSV file exists
    let csv_path = std::path::Path::new("examples/data/houses.csv");
    if !csv_path.exists() {
        error!("CSV file not found at: {}", csv_path.display());
        return Err(BatchError::ItemReader("CSV file not found".to_string()));
    } else {
        info!("CSV file found at: {}", csv_path.display());
    }

    // Create the XML writer using the temp directory
    let xml_writer = XmlItemWriterBuilder::new()
        .root_tag("houses")
        .item_tag("house")
        .from_path(xml_output_path)?;

    // Create a processor to convert CSV to XML format
    let processor = HouseProcessor;

    // Create and run the step, transforming CsvHouse to XmlHouse
    let step = StepBuilder::new("csv-to-xml")
        .chunk::<CsvHouse, XmlHouse>(3)
        .reader(&csv_reader)
        .processor(&processor)
        .writer(&xml_writer)
        .build();

    let mut step_execution = StepExecution::new("csv-to-xml");
    let result = step.execute(&mut step_execution);

    match result {
        Ok(_) => {
            info!("Step completed successfully");

            // Read the XML file for verification
            if !xml_output_path.exists() {
                error!("XML file not created at: {}", xml_output_path.display());
            } else {
                info!("XML file found at: {}", xml_output_path.display());
            }

            let xml_reader = XmlItemReaderBuilder::<XmlHouse>::new()
                .capacity(1024)
                .tag("house")
                .from_path(xml_output_path)?;

            let mut count = 0;
            while let Some(house) = xml_reader.read()? {
                println!(
                    "House {}: {} - {}€",
                    house.id, house.address.street, house.price
                );
                count += 1;
            }
            println!("Total: {} converted houses", count);
            println!("XML output available at: {}", xml_output_path.display());
            Ok(())
        }
        Err(e) => {
            error!("Step failed: {:?}", e);
            Err(e)
        }
    }
}
