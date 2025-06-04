---
sidebar_position: 4
---

# Examples

This page showcases various real-world examples and patterns for using Spring Batch RS in different scenarios.

## File Processing Examples

### CSV to JSON Transformation with Processing

Transform CSV data to JSON while applying business logic:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::ItemProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
}

#[derive(Serialize)]
struct EnrichedProduct {
    id: u32,
    name: String,
    price: f64,
    category: String,
    price_tier: String,
    discounted_price: f64,
}

struct ProductEnrichmentProcessor;

impl ItemProcessor<Product, EnrichedProduct> for ProductEnrichmentProcessor {
    fn process(&self, item: &Product) -> Result<EnrichedProduct, BatchError> {
        let price_tier = match item.price {
            p if p < 50.0 => "Budget",
            p if p < 200.0 => "Mid-range",
            _ => "Premium",
        };

        let discount = if item.category == "Electronics" { 0.1 } else { 0.05 };
        let discounted_price = item.price * (1.0 - discount);

        Ok(EnrichedProduct {
            id: item.id,
            name: item.name.clone(),
            price: item.price,
            category: item.category.clone(),
            price_tier: price_tier.to_string(),
            discounted_price,
        })
    }
}

fn main() -> Result<(), BatchError> {
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv");

    let processor = ProductEnrichmentProcessor;

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("enriched_products.json");

    let step = StepBuilder::new("enrich_products")
        .chunk(100)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

### Fault-Tolerant Processing

Handle errors gracefully with skip limits:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::ItemProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};
use serde::{Deserialize, Serialize};
use log::warn;

#[derive(Deserialize, Serialize, Clone)]
struct RawData {
    id: String,
    value: String,
    timestamp: String,
}

#[derive(Serialize)]
struct ProcessedData {
    id: u32,
    value: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

struct DataValidationProcessor;

impl ItemProcessor<RawData, ProcessedData> for DataValidationProcessor {
    fn process(&self, item: &RawData) -> Result<ProcessedData, BatchError> {
        // Parse ID
        let id = item.id.parse::<u32>()
            .map_err(|e| BatchError::ItemProcessor(format!("Invalid ID '{}': {}", item.id, e)))?;

        // Parse value
        let value = item.value.parse::<f64>()
            .map_err(|e| BatchError::ItemProcessor(format!("Invalid value '{}': {}", item.value, e)))?;

        // Parse timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&item.timestamp)
            .map_err(|e| BatchError::ItemProcessor(format!("Invalid timestamp '{}': {}", item.timestamp, e)))?
            .with_timezone(&chrono::Utc);

        // Validate business rules
        if value < 0.0 {
            return Err(BatchError::ItemProcessor("Value cannot be negative".to_string()));
        }

        Ok(ProcessedData { id, value, timestamp })
    }
}

fn main() -> Result<(), BatchError> {
    let reader = CsvItemReaderBuilder::<RawData>::new()
        .has_headers(true)
        .from_path("raw_data.csv");

    let processor = DataValidationProcessor;

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("processed_data.json");

    let step = StepBuilder::new("validate_data")
        .chunk(50)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .skip_limit(10)  // Skip up to 10 invalid records
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

## Database Examples

### Database to File Export

Export database records to CSV with pagination:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    item::{orm::OrmItemReaderBuilder, csv::CsvItemWriterBuilder},
    BatchError,
};
use sea_orm::{Database, EntityTrait, ColumnTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct User {
    id: i32,
    email: String,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    active: bool,
}

async fn export_active_users() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("postgresql://user:pass@localhost/myapp").await?;

    // Query only active users created in the last year
    let one_year_ago = chrono::Utc::now() - chrono::Duration::days(365);
    let query = UserEntity::find()
        .filter(user::Column::Active.eq(true))
        .filter(user::Column::CreatedAt.gte(one_year_ago))
        .order_by_asc(user::Column::Id);

    let reader = OrmItemReaderBuilder::new()
        .connection(&db)
        .query(query)
        .page_size(1000)  // Process 1000 records at a time
        .build();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path("active_users_export.csv");

    let step = StepBuilder::new("export_users")
        .chunk(500)
        .reader(&reader)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
```

### File to Database Import

Import CSV data into a database:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::ItemProcessor},
    item::{csv::CsvItemReaderBuilder, orm::OrmItemWriterBuilder},
    BatchError,
};
use sea_orm::{Database, ActiveModelTrait, Set};

#[derive(Deserialize, Clone)]
struct CsvUser {
    email: String,
    name: String,
    department: String,
}

struct UserImportProcessor {
    default_active: bool,
}

impl ItemProcessor<CsvUser, user::ActiveModel> for UserImportProcessor {
    fn process(&self, item: &CsvUser) -> Result<user::ActiveModel, BatchError> {
        // Validate email format
        if !item.email.contains('@') {
            return Err(BatchError::ItemProcessor(
                format!("Invalid email format: {}", item.email)
            ));
        }

        // Create ActiveModel for database insertion
        Ok(user::ActiveModel {
            id: NotSet,
            email: Set(item.email.clone()),
            name: Set(item.name.clone()),
            department: Set(Some(item.department.clone())),
            active: Set(self.default_active),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        })
    }
}

async fn import_users() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("postgresql://user:pass@localhost/myapp").await?;

    let reader = CsvItemReaderBuilder::<CsvUser>::new()
        .has_headers(true)
        .from_path("users_import.csv");

    let processor = UserImportProcessor { default_active: true };

    let writer = OrmItemWriterBuilder::new()
        .connection(&db)
        .build();

    let step = StepBuilder::new("import_users")
        .chunk(100)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .skip_limit(5)  // Skip invalid records
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
```

## Tasklet Examples

### File Archive and Cleanup

Create a multi-step job that processes files and then archives them:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, step::{Tasklet, StepExecution, RepeatStatus}},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    tasklet::zip::ZipTaskletBuilder,
    BatchError,
};
use std::fs;
use log::info;

struct CleanupTasklet {
    directory: String,
    file_pattern: String,
}

impl Tasklet for CleanupTasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        info!("Starting cleanup for step: {}", step_execution.name);

        let entries = fs::read_dir(&self.directory)
            .map_err(|e| BatchError::Tasklet(format!("Failed to read directory: {}", e)))?;

        let mut deleted_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| BatchError::Tasklet(e.to_string()))?;
            let path = entry.path();

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(&self.file_pattern) {
                    fs::remove_file(&path)
                        .map_err(|e| BatchError::Tasklet(format!("Failed to delete file: {}", e)))?;
                    deleted_count += 1;
                    info!("Deleted file: {:?}", path);
                }
            }
        }

        info!("Cleanup completed. Deleted {} files", deleted_count);
        Ok(RepeatStatus::Finished)
    }
}

fn main() -> Result<(), BatchError> {
    // Step 1: Process data
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("input/products.csv");

    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("output/products.json");

    let process_step = StepBuilder::new("process_data")
        .chunk(100)
        .reader(&reader)
        .writer(&writer)
        .build();

    // Step 2: Archive output files
    let archive_tasklet = ZipTaskletBuilder::new()
        .source_path("output/")
        .target_path("archive/products_archive.zip")
        .compression_level(6)
        .build()?;

    let archive_step = StepBuilder::new("archive_files")
        .tasklet(&archive_tasklet)
        .build();

    // Step 3: Cleanup temporary files
    let cleanup_tasklet = CleanupTasklet {
        directory: "temp/".to_string(),
        file_pattern: ".tmp".to_string(),
    };

    let cleanup_step = StepBuilder::new("cleanup_temp")
        .tasklet(&cleanup_tasklet)
        .build();

    // Combine all steps
    let job = JobBuilder::new()
        .start(&process_step)
        .next(&archive_step)
        .next(&cleanup_step)
        .build();

    job.run()
}
```

## Testing Examples

### Mock Data Generation

Generate test data for development and testing:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    item::{fake::person_reader::PersonReaderBuilder, csv::CsvItemWriterBuilder},
    BatchError,
};

fn generate_test_data() -> Result<(), BatchError> {
    // Generate 10,000 fake person records
    let reader = PersonReaderBuilder::new()
        .number_of_items(10_000)
        .locale("en_US")
        .build();

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path("test_data/persons.csv");

    let step = StepBuilder::new("generate_test_persons")
        .chunk(500)
        .reader(&reader)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

### Debug Logging

Use the logger writer for debugging and development:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::ItemProcessor},
    item::{csv::CsvItemReaderBuilder, logger::LoggerItemWriterBuilder},
    BatchError,
};
use log::Level;

struct DebugProcessor;

impl ItemProcessor<Product, Product> for DebugProcessor {
    fn process(&self, item: &Product) -> Result<Product, BatchError> {
        // Add debug information
        log::debug!("Processing product: {} (ID: {})", item.name, item.id);

        // Simulate some processing time
        std::thread::sleep(std::time::Duration::from_millis(10));

        Ok(item.clone())
    }
}

fn debug_processing() -> Result<(), BatchError> {
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv");

    let processor = DebugProcessor;

    let writer = LoggerItemWriterBuilder::new()
        .log_level(Level::Info)
        .prefix("Processed product:")
        .build();

    let step = StepBuilder::new("debug_processing")
        .chunk(10)  // Small chunks for detailed logging
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

## Performance Optimization Examples

### Large Dataset Processing

Optimize for processing large datasets:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    item::{csv::CsvItemReaderBuilder, csv::CsvItemWriterBuilder},
    BatchError,
};

fn process_large_dataset() -> Result<(), BatchError> {
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .buffer_size(8192)  // Larger buffer for file I/O
        .from_path("large_dataset.csv");

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .buffer_size(8192)
        .from_path("processed_large_dataset.csv");

    let step = StepBuilder::new("process_large_data")
        .chunk(1000)  // Large chunks for better throughput
        .reader(&reader)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

These examples demonstrate the flexibility and power of Spring Batch RS for various batch processing scenarios. You can combine and adapt these patterns to fit your specific use cases.
