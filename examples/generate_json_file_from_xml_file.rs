use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::step::{Step, StepBuilder, StepExecution},
    error::BatchError,
    item::{json::json_writer::JsonItemWriterBuilder, xml::XmlItemReaderBuilder},
};

// --- Struct Definitions for Complex Vehicles ---
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")] // For JSON output field names
struct Displacement {
    #[serde(rename = "@unit")]
    unit: String,
    #[serde(rename = "$value")]
    value: String, // Using String to accommodate both integers and floats like "3.5"
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
struct Engine {
    #[serde(rename = "@cylinders")]
    cylinders: i32,
    #[serde(rename = "type")]
    engine_type: String,
    displacement: Displacement,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
struct Features {
    #[serde(rename = "feature", default)]
    items: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename = "vehicle")] // XML item tag for reader
#[serde(rename_all = "camelCase")] // For JSON output field names
struct Vehicle {
    #[serde(rename = "@type")]
    vehicle_type: String,
    #[serde(rename = "@id")]
    id: String,
    make: String,
    model: String,
    year: i32,
    engine: Engine,
    features: Features,
}
// --- End of Struct Definitions ---

fn main() -> Result<(), BatchError> {
    let temp_dir = tempfile::tempdir().unwrap();
    // Output JSON will also be named based on the new structure
    let output_path = temp_dir.path().join("complex_vehicles.json");

    // Path to the new complex XML file
    let xml_input_path = "examples/data/complex_vehicles.xml";

    // Create XML reader for Vehicle structs, looking for "vehicle" tags
    let reader = XmlItemReaderBuilder::<Vehicle>::new()
        .tag("vehicle")
        .capacity(1024)
        .from_path(xml_input_path)?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true) // Make JSON output readable
        .from_path(&output_path);

    // Create and run the step
    let step = StepBuilder::new("xml_to_json_complex_vehicles")
        .chunk::<Vehicle, Vehicle>(2) // Process 2 vehicles at a time
        .reader(&reader)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new("xml_to_json_complex_vehicles");
    let _result = step.execute(&mut step_execution);

    println!("Generated JSON output at: {}", output_path.display());
    // Optionally print content for quick verification, though it might be large
    match std::fs::read_to_string(&output_path) {
        Ok(content) => println!("\nGenerated JSON output:\n{}", content),
        Err(e) => eprintln!("Failed to read output file: {}", e),
    }

    Ok(())
}
