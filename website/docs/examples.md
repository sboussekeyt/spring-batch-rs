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
        .chunk::<Product, EnrichedProduct>(100)
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
        .chunk::<RawData, ProcessedData>(50)
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

````rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
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

    let processor = PassThroughProcessor::<User>::new();

    let step = StepBuilder::new("export_users")
        .chunk::<User, User>(500)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

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
        .chunk::<CsvUser, user::ActiveModel>(100)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .skip_limit(5)  // Skip invalid records
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run().map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
````

## Tasklet Examples

### File Archive and Cleanup

Create a multi-step job that processes files and then archives them:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, step::{Tasklet, StepExecution, RepeatStatus}, item::PassThroughProcessor},
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

    let processor = PassThroughProcessor::<Product>::new();

    let process_step = StepBuilder::new("process_data")
        .chunk::<Product, Product>(100)
        .reader(&reader)
        .processor(&processor)
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

### FTP File Transfer Operations

Transfer files to and from FTP servers as part of your batch workflow:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, csv::CsvItemWriterBuilder},
    tasklet::ftp::{FtpPutTaskletBuilder, FtpGetTaskletBuilder, FtpPutFolderTaskletBuilder, FtpGetFolderTaskletBuilder},
    BatchError,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Serialize, Clone)]
struct SalesReport {
    date: String,
    region: String,
    sales: f64,
    units: u32,
}

fn ftp_batch_workflow() -> Result<(), BatchError> {
    // Step 1: Download input files from FTP server
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host("ftp.company.com")
        .port(21)
        .username("batch_user")
        .password("secure_password")
        .remote_file("/incoming/sales_data.csv")
        .local_file("./input/sales_data.csv")
        .passive_mode(true)
        .secure(false)  // Plain FTP for internal network
        .build()?;

    let download_step = StepBuilder::new("download_input")
        .tasklet(&download_tasklet)
        .build();

    // Step 2: Process the downloaded data
    let reader = CsvItemReaderBuilder::<SalesReport>::new()
        .has_headers(true)
        .from_path("./input/sales_data.csv");

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path("./output/processed_sales.csv");

    let processor = PassThroughProcessor::<SalesReport>::new();

    let process_step = StepBuilder::new("process_sales")
        .chunk::<SalesReport, SalesReport>(100)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // Step 3: Upload processed files back to FTP server
    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host("ftp.company.com")
        .port(21)
        .username("batch_user")
        .password("secure_password")
        .local_file("./output/processed_sales.csv")
        .remote_file("/outgoing/processed_sales.csv")
        .passive_mode(true)
        .secure(false)  // Plain FTP for internal network
        .build()?;

    let upload_step = StepBuilder::new("upload_output")
        .tasklet(&upload_tasklet)
        .build();

    // Combine all steps into a complete workflow
    let job = JobBuilder::new()
        .start(&download_step)
        .next(&process_step)
        .next(&upload_step)
        .build();

    job.run()
}
```

### FTPS Secure File Transfer Operations

For sensitive data and external connections, use FTPS with TLS encryption:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, csv::CsvItemWriterBuilder},
    tasklet::ftp::{FtpPutTaskletBuilder, FtpGetTaskletBuilder},
    BatchError,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Serialize, Clone)]
struct FinancialReport {
    account_id: String,
    transaction_date: String,
    amount: f64,
    category: String,
}

fn secure_financial_batch_workflow() -> Result<(), BatchError> {
    // Step 1: Securely download sensitive financial data using FTPS
    let secure_download_tasklet = FtpGetTaskletBuilder::new()
        .host("secure-bank.example.com")
        .port(990)  // Standard FTPS port
        .username("financial_user")
        .password("strong_password_123")
        .remote_file("/secure/incoming/financial_data.csv")
        .local_file("./secure/input/financial_data.csv")
        .passive_mode(true)
        .secure(true)  // Enable FTPS encryption
        .timeout(Duration::from_secs(120))  // Longer timeout for secure handshake
        .build()?;

    let secure_download_step = StepBuilder::new("secure_download_financial")
        .tasklet(&secure_download_tasklet)
        .build();

    // Step 2: Process financial data with validation
    let reader = CsvItemReaderBuilder::<FinancialReport>::new()
        .has_headers(true)
        .from_path("./secure/input/financial_data.csv");

    let writer = CsvItemWriterBuilder::new()
        .has_headers(true)
        .from_path("./secure/output/processed_financial.csv");

    let processor = PassThroughProcessor::<FinancialReport>::new();

    let process_step = StepBuilder::new("process_financial")
        .chunk::<FinancialReport, FinancialReport>(50)  // Smaller chunks for financial data
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // Step 3: Securely upload processed financial reports using FTPS
    let secure_upload_tasklet = FtpPutTaskletBuilder::new()
        .host("secure-reporting.example.com")
        .port(990)
        .username("reporting_user")
        .password("secure_reporting_pass")
        .local_file("./secure/output/processed_financial.csv")
        .remote_file("/secure/reports/daily_financial_report.csv")
        .passive_mode(true)
        .secure(true)  // Always use FTPS for financial data
        .timeout(Duration::from_secs(180))
        .build()?;

    let secure_upload_step = StepBuilder::new("secure_upload_financial")
        .tasklet(&secure_upload_tasklet)
        .build();

    // Create secure financial processing workflow
    let job = JobBuilder::new()
        .start(&secure_download_step)
        .next(&process_step)
        .next(&secure_upload_step)
        .build();

    job.run()
}
```

### Environment-Based FTPS Configuration

Use environment variables for secure credential management in production:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::{FtpPutTaskletBuilder, FtpGetTaskletBuilder},
    BatchError,
};
use std::{env, time::Duration};

fn secure_environment_based_transfer() -> Result<(), BatchError> {
    // Read secure FTP credentials from environment
    let secure_host = env::var("SECURE_FTP_HOST")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_HOST not set".to_string()))?;
    let secure_user = env::var("SECURE_FTP_USER")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_USER not set".to_string()))?;
    let secure_pass = env::var("SECURE_FTP_PASS")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_PASS not set".to_string()))?;

    // Determine if secure mode should be enabled
    let use_secure = env::var("USE_FTPS").unwrap_or_else(|_| "true".to_string()) == "true";
    let ftp_port: u16 = env::var("FTP_PORT")
        .unwrap_or_else(|_| if use_secure { "990" } else { "21" }.to_string())
        .parse()
        .map_err(|_| BatchError::Configuration("Invalid FTP_PORT".to_string()))?;

    // Step 1: Download using environment-configured security
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host(&secure_host)
        .port(ftp_port)
        .username(&secure_user)
        .password(&secure_pass)
        .remote_file("/secure/data/input.csv")
        .local_file("./downloads/input.csv")
        .passive_mode(true)
        .secure(use_secure)  // Environment-controlled security
        .timeout(Duration::from_secs(if use_secure { 180 } else { 60 }))
        .build()?;

    let download_step = StepBuilder::new("env_secure_download")
        .tasklet(&download_tasklet)
        .build();

    // Step 2: Upload with same security configuration
    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host(&secure_host)
        .port(ftp_port)
        .username(&secure_user)
        .password(&secure_pass)
        .local_file("./processed/output.csv")
        .remote_file("/secure/processed/output.csv")
        .passive_mode(true)
        .secure(use_secure)
        .timeout(Duration::from_secs(if use_secure { 180 } else { 60 }))
        .build()?;

    let upload_step = StepBuilder::new("env_secure_upload")
        .tasklet(&upload_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&download_step)
        .next(&upload_step)
        .build();

    job.run()
}
```

### FTP Folder Operations

Transfer entire directories for bulk file operations:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::{FtpPutFolderTaskletBuilder, FtpGetFolderTaskletBuilder},
    BatchError,
};
use std::time::Duration;

fn ftp_folder_sync() -> Result<(), BatchError> {
    // Step 1: Download entire folder from FTP server
    let download_folder_tasklet = FtpGetFolderTaskletBuilder::new()
        .host("backup.company.com")
        .port(21)
        .username("backup_user")
        .password("backup_password")
        .remote_folder("/daily_reports")
        .local_folder("./downloads/daily_reports")
        .recursive(true)
        .create_directories(true)
        .passive_mode(true)
        .secure(false)  // Internal network, plain FTP
        .build()?;

    let download_step = StepBuilder::new("download_reports")
        .tasklet(&download_folder_tasklet)
        .build();

    // Step 2: Upload processed folder to different FTP location
    let upload_folder_tasklet = FtpPutFolderTaskletBuilder::new()
        .host("archive.company.com")
        .port(21)
        .username("archive_user")
        .password("archive_password")
        .local_folder("./processed/reports")
        .remote_folder("/archive/processed_reports")
        .recursive(true)
        .create_directories(true)
        .passive_mode(true)
        .secure(false)  // Internal archive, plain FTP
        .build()?;

    let upload_step = StepBuilder::new("archive_reports")
        .tasklet(&upload_folder_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&download_step)
        .next(&upload_step)
        .build();

    job.run()
}
```

### FTPS Secure Folder Operations

For secure bulk transfers with encryption:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::{FtpPutFolderTaskletBuilder, FtpGetFolderTaskletBuilder},
    BatchError,
};
use std::time::Duration;

fn secure_folder_operations() -> Result<(), BatchError> {
    // Step 1: Securely download entire confidential folder structure
    let secure_download_folder_tasklet = FtpGetFolderTaskletBuilder::new()
        .host("secure-vault.example.com")
        .port(990)  // FTPS port
        .username("secure_backup_user")
        .password("ultra_secure_password")
        .remote_folder("/confidential/client_data")
        .local_folder("./secure/downloads/client_data")
        .recursive(true)  // Include all subdirectories
        .create_directories(true)
        .passive_mode(true)
        .secure(true)  // Enable FTPS encryption
        .timeout(Duration::from_secs(600))  // 10 minutes for large transfers
        .build()?;

    let secure_download_step = StepBuilder::new("secure_download_confidential")
        .tasklet(&secure_download_folder_tasklet)
        .build();

    // Step 2: Securely upload processed confidential data
    let secure_upload_folder_tasklet = FtpPutFolderTaskletBuilder::new()
        .host("secure-distribution.example.com")
        .port(990)
        .username("distribution_user")
        .password("distribution_secure_pass")
        .local_folder("./secure/processed/client_reports")
        .remote_folder("/secure/distribution/processed_reports")
        .recursive(true)
        .create_directories(true)
        .passive_mode(true)
        .secure(true)  // Always encrypted for distribution
        .timeout(Duration::from_secs(900))  // 15 minutes for large uploads
        .build()?;

    let secure_upload_step = StepBuilder::new("secure_upload_distribution")
        .tasklet(&secure_upload_folder_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&secure_download_step)
        .next(&secure_upload_step)
        .build();

    job.run()
}
```

### Mixed FTP/FTPS Operations

Combine plain FTP for internal operations with FTPS for external transfers:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::{FtpGetTaskletBuilder, FtpPutTaskletBuilder},
    BatchError,
};
use std::time::Duration;

fn mixed_security_workflow() -> Result<(), BatchError> {
    // Step 1: Download from internal FTP server (plain FTP)
    let internal_download_tasklet = FtpGetTaskletBuilder::new()
        .host("internal-ftp.company.com")
        .port(21)
        .username("internal_user")
        .password("internal_pass")
        .remote_file("/internal/raw_data.csv")
        .local_file("./processing/raw_data.csv")
        .passive_mode(true)
        .secure(false)  // Internal network - plain FTP is acceptable
        .timeout(Duration::from_secs(60))
        .build()?;

    let internal_download_step = StepBuilder::new("internal_download")
        .tasklet(&internal_download_tasklet)
        .build();

    // Step 2: Upload to external partner using FTPS (secure)
    let external_upload_tasklet = FtpPutTaskletBuilder::new()
        .host("partner-secure.example.com")
        .port(990)
        .username("partner_user")
        .password("partner_secure_password")
        .local_file("./processing/processed_data.csv")
        .remote_file("/incoming/partner_data.csv")
        .passive_mode(true)
        .secure(true)  // External transfer - must be encrypted
        .timeout(Duration::from_secs(300))  // Longer timeout for external FTPS
        .build()?;

    let external_upload_step = StepBuilder::new("external_secure_upload")
        .tasklet(&external_upload_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&internal_download_step)
        .next(&external_upload_step)
        .build();

    job.run()
}
```

### Large File Transfer with FTPS

Handle large files efficiently with streaming and encryption:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::{FtpGetTaskletBuilder, FtpPutTaskletBuilder},
    BatchError,
};
use std::time::Duration;

fn large_file_secure_transfer() -> Result<(), BatchError> {
    // Step 1: Download large file (GB+ size) with streaming and encryption
    let large_file_download_tasklet = FtpGetTaskletBuilder::new()
        .host("bigdata.example.com")
        .port(990)
        .username("bigdata_user")
        .password("bigdata_secure_pass")
        .remote_file("/archive/huge_dataset_5gb.zip")  // 5GB file
        .local_file("./downloads/huge_dataset.zip")    // Streams directly to disk
        .passive_mode(true)
        .secure(true)  // FTPS for secure large file transfer
        .timeout(Duration::from_secs(3600))  // 1 hour for very large files
        .build()?;

    let download_step = StepBuilder::new("download_large_secure")
        .tasklet(&large_file_download_tasklet)
        .build();

    // Step 2: Upload processed large file securely
    let large_file_upload_tasklet = FtpPutTaskletBuilder::new()
        .host("results.example.com")
        .port(990)
        .username("results_user")
        .password("results_secure_pass")
        .local_file("./processed/processed_dataset_3gb.zip")
        .remote_file("/results/processed_dataset.zip")
        .passive_mode(true)
        .secure(true)  // Always encrypted for large sensitive data
        .timeout(Duration::from_secs(2700))  // 45 minutes for upload
        .build()?;

    let upload_step = StepBuilder::new("upload_large_secure")
        .tasklet(&large_file_upload_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&download_step)
        .next(&upload_step)
        .build();

    job.run()
}
```

### Secure FTP with Error Handling

Handle FTPS operations with comprehensive error handling and retry logic:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, step::{Tasklet, StepExecution, RepeatStatus}},
    tasklet::ftp::FtpPutTaskletBuilder,
    BatchError,
};
use std::time::Duration;
use log::{info, error, warn};

struct SecureFtpUploadTasklet {
    ftp_tasklet: spring_batch_rs::tasklet::ftp::FtpPutTasklet,
    retry_count: u32,
    use_secure: bool,
}

impl SecureFtpUploadTasklet {
    fn new(host: &str, username: &str, password: &str, local_file: &str, remote_file: &str, use_secure: bool) -> Result<Self, BatchError> {
        let port = if use_secure { 990 } else { 21 };
        let timeout = if use_secure {
            Duration::from_secs(120)  // Longer timeout for FTPS handshake
        } else {
            Duration::from_secs(60)
        };

        let ftp_tasklet = FtpPutTaskletBuilder::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .local_file(local_file)
            .remote_file(remote_file)
            .passive_mode(true)
            .secure(use_secure)  // Configurable security
            .timeout(timeout)
            .build()?;

        Ok(Self {
            ftp_tasklet,
            retry_count: if use_secure { 5 } else { 3 },  // More retries for secure connections
            use_secure,
        })
    }
}

impl Tasklet for SecureFtpUploadTasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let mut attempts = 0;
        let connection_type = if self.use_secure { "FTPS" } else { "FTP" };

        while attempts < self.retry_count {
            attempts += 1;
            info!("Attempting {} upload (attempt {}/{})", connection_type, attempts, self.retry_count);

            match self.ftp_tasklet.execute(step_execution) {
                Ok(status) => {
                    info!("{} upload successful on attempt {}", connection_type, attempts);
                    return Ok(status);
                }
                Err(e) => {
                    error!("{} upload failed on attempt {}: {}", connection_type, attempts, e);

                    if attempts >= self.retry_count {
                        return Err(BatchError::Tasklet(format!(
                            "{} upload failed after {} attempts: {}",
                            connection_type,
                            self.retry_count,
                            e
                        )));
                    }

                    // Progressive backoff: longer waits for secure connections
                    let wait_time = if self.use_secure {
                        Duration::from_secs(10 * attempts as u64)  // 10s, 20s, 30s, etc.
                    } else {
                        Duration::from_secs(5 * attempts as u64)   // 5s, 10s, 15s, etc.
                    };

                    warn!("Waiting {:?} before retry...", wait_time);
                    std::thread::sleep(wait_time);
                }
            }
        }

        Err(BatchError::Tasklet("Unexpected error in retry logic".to_string()))
    }
}

fn secure_ftp_upload() -> Result<(), BatchError> {
    // Read credentials from environment variables for security
    let ftp_host = std::env::var("FTP_HOST")
        .map_err(|_| BatchError::Configuration("FTP_HOST environment variable not set".to_string()))?;
    let ftp_user = std::env::var("FTP_USER")
        .map_err(|_| BatchError::Configuration("FTP_USER environment variable not set".to_string()))?;
    let ftp_pass = std::env::var("FTP_PASS")
        .map_err(|_| BatchError::Configuration("FTP_PASS environment variable not set".to_string()))?;

    // Check if secure mode should be used (default to true for safety)
    let use_secure = std::env::var("USE_FTPS")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);  // Default to secure

    let secure_upload_tasklet = SecureFtpUploadTasklet::new(
        &ftp_host,
        &ftp_user,
        &ftp_pass,
        "./sensitive_data/report.csv",
        "/secure/reports/daily_report.csv",
        use_secure
    )?;

    let upload_step = StepBuilder::new("secure_upload_with_retry")
        .tasklet(&secure_upload_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&upload_step)
        .build();

    job.run()
}
```

## Testing Examples

### Mock Data Generation

Generate test data for development and testing:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
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

    let processor = PassThroughProcessor::<Person>::new();

    let step = StepBuilder::new("generate_test_persons")
        .chunk::<Person, Person>(500)
        .reader(&reader)
        .processor(&processor)
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
        .chunk::<Product, Product>(10)  // Small chunks for detailed logging
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
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
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

    let processor = PassThroughProcessor::<Product>::new();

    let step = StepBuilder::new("process_large_data")
        .chunk::<Product, Product>(1000)  // Large chunks for better throughput
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

These examples demonstrate the flexibility and power of Spring Batch RS for various batch processing scenarios. You can combine and adapt these patterns to fit your specific use cases.
