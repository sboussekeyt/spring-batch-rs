//! # FTP Transfer Tasklet Example
//!
//! This example demonstrates how to use the FTP tasklets to upload and download files
//! to/from an FTP server as part of a batch processing job. It shows both PUT and GET
//! operations with various configuration options.
//!
//! ## Features Demonstrated
//!
//! - FTP PUT operations (file upload)
//! - FTP GET operations (file download)
//! - FTP PUT FOLDER operations (folder upload with multiple files)
//! - FTP GET FOLDER operations (folder download with multiple files)
//! - Connection configuration (host, port, credentials)
//! - Passive/Active mode configuration
//! - Recursive folder operations
//! - Error handling for FTP operations
//! - Integration with Spring Batch job execution
//!
//! ## Prerequisites
//!
//! To run this example, you'll need access to an FTP server. You can set up a local
//! FTP server for testing or use environment variables to configure connection details.
//!
//! ## Usage
//!
//! ```bash
//! # Set FTP server details via environment variables (optional)
//! export FTP_HOST=ftp.example.com
//! export FTP_USER=username
//! export FTP_PASS=password
//! export FTP_PORT=21
//!
//! # Run the example
//! cargo run --example ftp_transfer_tasklet --features ftp
//! ```

use log::info;
use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    tasklet::ftp::{
        FtpGetFolderTaskletBuilder, FtpGetTaskletBuilder, FtpPutFolderTaskletBuilder,
        FtpPutTaskletBuilder,
    },
    BatchError,
};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    time::Duration,
};

/// Configuration for FTP connection details.
///
/// This struct holds all the necessary information to connect to an FTP server.
/// It can be populated from environment variables or hardcoded values.
#[derive(Debug, Clone)]
struct FtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl FtpConfig {
    /// Creates FTP configuration from environment variables or defaults.
    ///
    /// Environment variables used:
    /// - `FTP_HOST`: FTP server hostname (default: "localhost")
    /// - `FTP_PORT`: FTP server port (default: 21)
    /// - `FTP_USER`: FTP username (default: "anonymous")
    /// - `FTP_PASS`: FTP password (default: "anonymous@example.com")
    fn from_env() -> Self {
        Self {
            host: env::var("FTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("FTP_PORT")
                .unwrap_or_else(|_| "21".to_string())
                .parse()
                .unwrap_or(21),
            username: env::var("FTP_USER").unwrap_or_else(|_| "anonymous".to_string()),
            password: env::var("FTP_PASS").unwrap_or_else(|_| "anonymous@example.com".to_string()),
        }
    }

    /// Creates a test configuration for demonstration purposes.
    ///
    /// Note: This configuration is for demonstration only and may not work
    /// with a real FTP server without proper credentials.
    fn test_config() -> Self {
        Self {
            host: "test.rebex.net".to_string(), // Public test FTP server
            port: 21,
            username: "demo".to_string(),
            password: "password".to_string(),
        }
    }
}

/// Creates sample files for FTP upload demonstration.
///
/// This function creates a directory structure with various files that will be
/// used to demonstrate FTP upload operations.
///
/// # Parameters
/// - `base_path`: The base directory where sample files will be created
///
/// # Returns
/// - `Ok(())`: Sample files created successfully
/// - `Err(std::io::Error)`: Error creating the files
fn create_sample_files(base_path: &Path) -> Result<(), std::io::Error> {
    info!(
        "Creating sample files for FTP upload at: {}",
        base_path.display()
    );

    // Create upload directory
    let upload_dir = base_path.join("upload");
    fs::create_dir_all(&upload_dir)?;

    // Create sample files with different content types
    let files_to_create = vec![
        (
            upload_dir.join("data.txt"),
            "Sample data file for FTP upload demonstration.\nThis file contains text data that will be uploaded to the FTP server.",
        ),
        (
            upload_dir.join("config.json"),
            r#"{
  "application": {
    "name": "FTP Transfer Demo",
    "version": "1.0.0",
    "settings": {
      "timeout": 30,
      "retry_count": 3,
      "passive_mode": true
    }
  }
}"#,
        ),
        (
            upload_dir.join("report.csv"),
            "id,name,value,timestamp\n1,Sample A,100,2024-01-01T10:00:00Z\n2,Sample B,200,2024-01-01T11:00:00Z\n3,Sample C,300,2024-01-01T12:00:00Z",
        ),
        (
            upload_dir.join("readme.md"),
            "# FTP Transfer Example\n\nThis directory contains sample files for demonstrating FTP upload operations.\n\n## Files\n\n- `data.txt`: Sample text file\n- `config.json`: Configuration file in JSON format\n- `report.csv`: Sample CSV data\n- `readme.md`: This documentation file",
        ),
    ];

    for (file_path, content) in files_to_create {
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        info!("Created sample file: {}", file_path.display());
    }

    info!("Sample files created successfully");
    Ok(())
}

/// Demonstrates FTP PUT operation (file upload).
///
/// This function shows how to upload a local file to an FTP server using the FtpPutTasklet.
fn demo_ftp_put(
    config: &FtpConfig,
    local_file: &Path,
    remote_path: &str,
) -> Result<(), BatchError> {
    info!("=== Demo: FTP PUT Operation ===");
    info!(
        "Uploading {} to {}:{}{}",
        local_file.display(),
        config.host,
        config.port,
        remote_path
    );

    let ftp_put_tasklet = FtpPutTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .local_file(local_file)
        .remote_file(remote_path)
        .passive_mode(true)
        .timeout(Duration::from_secs(30))
        .build()?;

    let step = StepBuilder::new("ftp-upload-step")
        .tasklet(&ftp_put_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting FTP upload job...");
    let result = job.run()?;
    info!("FTP upload completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates FTP GET operation (file download).
///
/// This function shows how to download a file from an FTP server using the FtpGetTasklet.
fn demo_ftp_get(
    config: &FtpConfig,
    remote_path: &str,
    local_file: &Path,
) -> Result<(), BatchError> {
    info!("=== Demo: FTP GET Operation ===");
    info!(
        "Downloading {}:{}{} to {}",
        config.host,
        config.port,
        remote_path,
        local_file.display()
    );

    let ftp_get_tasklet = FtpGetTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .remote_file(remote_path)
        .local_file(local_file)
        .passive_mode(true)
        .timeout(Duration::from_secs(30))
        .build()?;

    let step = StepBuilder::new("ftp-download-step")
        .tasklet(&ftp_get_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting FTP download job...");
    let result = job.run()?;
    info!("FTP download completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates a complete FTP workflow with multiple operations.
///
/// This function shows how to chain multiple FTP operations in a single job,
/// including both upload and download operations.
fn demo_ftp_workflow(config: &FtpConfig, base_path: &Path) -> Result<(), BatchError> {
    info!("=== Demo: Complete FTP Workflow ===");

    let upload_file = base_path.join("upload").join("config.json");
    let download_file = base_path.join("download").join("retrieved_config.json");
    let remote_path = "/tmp/config.json";

    // Ensure download directory exists
    if let Some(parent) = download_file.parent() {
        fs::create_dir_all(parent).map_err(|e| BatchError::Io(e))?;
    }

    // Step 1: Upload file
    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .local_file(&upload_file)
        .remote_file(remote_path)
        .passive_mode(true)
        .build()?;

    let upload_step = StepBuilder::new("upload-config-step")
        .tasklet(&upload_tasklet)
        .build();

    // Step 2: Download the same file to a different location
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .remote_file(remote_path)
        .local_file(&download_file)
        .passive_mode(true)
        .build()?;

    let download_step = StepBuilder::new("download-config-step")
        .tasklet(&download_tasklet)
        .build();

    // Create a job with both steps
    let job = JobBuilder::new()
        .start(&upload_step)
        .next(&download_step)
        .build();

    info!("Starting complete FTP workflow...");
    let result = job.run()?;
    info!("FTP workflow completed in {:?}", result.duration);

    // Verify the downloaded file exists and has content
    if download_file.exists() {
        let content = fs::read_to_string(&download_file).map_err(|e| BatchError::Io(e))?;
        info!(
            "Downloaded file content preview: {}",
            content.chars().take(100).collect::<String>()
        );
    }

    Ok(())
}

/// Demonstrates FTP PUT FOLDER operation (folder upload).
///
/// This function shows how to upload an entire folder with multiple files to an FTP server.
fn demo_ftp_put_folder(
    config: &FtpConfig,
    local_folder: &Path,
    remote_folder: &str,
) -> Result<(), BatchError> {
    info!("=== Demo: FTP PUT FOLDER Operation ===");
    info!(
        "Uploading folder {} to {}:{}{}",
        local_folder.display(),
        config.host,
        config.port,
        remote_folder
    );

    let ftp_put_folder_tasklet = FtpPutFolderTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .local_folder(local_folder)
        .remote_folder(remote_folder)
        .passive_mode(true)
        .timeout(Duration::from_secs(60))
        .create_directories(true)
        .recursive(false) // Only upload files in the root folder
        .build()?;

    let step = StepBuilder::new("ftp-upload-folder-step")
        .tasklet(&ftp_put_folder_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting FTP folder upload job...");
    let result = job.run()?;
    info!("FTP folder upload completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates FTP GET FOLDER operation (folder download).
///
/// This function shows how to download an entire folder with multiple files from an FTP server.
fn demo_ftp_get_folder(
    config: &FtpConfig,
    remote_folder: &str,
    local_folder: &Path,
) -> Result<(), BatchError> {
    info!("=== Demo: FTP GET FOLDER Operation ===");
    info!(
        "Downloading folder {}:{}{} to {}",
        config.host,
        config.port,
        remote_folder,
        local_folder.display()
    );

    let ftp_get_folder_tasklet = FtpGetFolderTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .remote_folder(remote_folder)
        .local_folder(local_folder)
        .passive_mode(true)
        .timeout(Duration::from_secs(60))
        .create_directories(true)
        .recursive(false) // Only download files in the root folder
        .build()?;

    let step = StepBuilder::new("ftp-download-folder-step")
        .tasklet(&ftp_get_folder_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting FTP folder download job...");
    let result = job.run()?;
    info!("FTP folder download completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates a complete FTP folder workflow with recursive operations.
///
/// This function shows how to upload and download folders recursively,
/// including subdirectories and their contents.
fn demo_ftp_recursive_workflow(config: &FtpConfig, base_path: &Path) -> Result<(), BatchError> {
    info!("=== Demo: Recursive FTP Folder Workflow ===");

    // Create a nested folder structure for demonstration
    let source_folder = base_path.join("recursive_upload");
    let subfolder = source_folder.join("subfolder");
    fs::create_dir_all(&subfolder).map_err(|e| BatchError::Io(e))?;

    // Create files in root folder
    fs::write(source_folder.join("root_file.txt"), "File in root folder")
        .map_err(|e| BatchError::Io(e))?;

    // Create files in subfolder
    fs::write(subfolder.join("sub_file.txt"), "File in subfolder")
        .map_err(|e| BatchError::Io(e))?;
    fs::write(
        subfolder.join("data.json"),
        r#"{"nested": true, "level": 1}"#,
    )
    .map_err(|e| BatchError::Io(e))?;

    let remote_folder = "/tmp/recursive_test";
    let download_folder = base_path.join("recursive_download");

    // Step 1: Upload folder recursively
    let upload_tasklet = FtpPutFolderTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .local_folder(&source_folder)
        .remote_folder(remote_folder)
        .passive_mode(true)
        .create_directories(true)
        .recursive(true) // Upload subdirectories recursively
        .build()?;

    let upload_step = StepBuilder::new("recursive-upload-step")
        .tasklet(&upload_tasklet)
        .build();

    // Step 2: Download folder recursively
    let download_tasklet = FtpGetFolderTaskletBuilder::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .remote_folder(remote_folder)
        .local_folder(&download_folder)
        .passive_mode(true)
        .create_directories(true)
        .recursive(true) // Download subdirectories recursively
        .build()?;

    let download_step = StepBuilder::new("recursive-download-step")
        .tasklet(&download_tasklet)
        .build();

    // Create a job with both steps
    let job = JobBuilder::new()
        .start(&upload_step)
        .next(&download_step)
        .build();

    info!("Starting recursive FTP workflow...");
    let result = job.run()?;
    info!("Recursive FTP workflow completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates error handling in FTP operations.
///
/// This function intentionally triggers various error conditions to show
/// how the FTP tasklets handle different types of failures.
fn demo_error_handling() -> Result<(), BatchError> {
    info!("=== Demo: FTP Error Handling ===");

    // Test 1: Invalid host
    info!("Testing connection to invalid host...");
    let invalid_host_tasklet = FtpPutTaskletBuilder::new()
        .host("invalid.nonexistent.host")
        .port(21)
        .username("test")
        .password("test")
        .local_file("Cargo.toml") // Use existing file
        .remote_file("/test.txt")
        .timeout(Duration::from_secs(5)) // Short timeout for quick failure
        .build()?;

    let step = StepBuilder::new("invalid-host-test")
        .tasklet(&invalid_host_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    match job.run() {
        Ok(_) => info!("Unexpected success with invalid host"),
        Err(e) => info!("Expected error with invalid host: {}", e),
    }

    // Test 2: Missing local file
    info!("Testing upload of nonexistent file...");
    match FtpPutTaskletBuilder::new()
        .host("localhost")
        .username("test")
        .password("test")
        .local_file("/nonexistent/file.txt")
        .remote_file("/test.txt")
        .build()
    {
        Ok(_) => info!("Unexpected success with nonexistent file"),
        Err(e) => info!("Expected error with nonexistent file: {}", e),
    }

    Ok(())
}

/// Prints a summary of the FTP operations performed.
fn print_results_summary(base_path: &Path) -> Result<(), std::io::Error> {
    info!("=== FTP Transfer Results Summary ===");

    let upload_dir = base_path.join("upload");
    let download_dir = base_path.join("download");

    if upload_dir.exists() {
        info!("Upload directory: {}", upload_dir.display());
        for entry in fs::read_dir(&upload_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            info!(
                "  - {} ({} bytes)",
                entry.file_name().to_string_lossy(),
                metadata.len()
            );
        }
    }

    if download_dir.exists() {
        info!("Download directory: {}", download_dir.display());
        for entry in fs::read_dir(&download_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            info!(
                "  - {} ({} bytes)",
                entry.file_name().to_string_lossy(),
                metadata.len()
            );
        }
    }

    Ok(())
}

fn main() -> Result<(), BatchError> {
    // Initialize logging
    env_logger::init();

    info!("Starting FTP Transfer Tasklet Example");

    // Create working directory
    let base_path = std::env::temp_dir().join("ftp_transfer_example");
    if base_path.exists() {
        fs::remove_dir_all(&base_path).map_err(|e| BatchError::Io(e))?;
    }
    fs::create_dir_all(&base_path).map_err(|e| BatchError::Io(e))?;

    info!("Working directory: {}", base_path.display());

    // Create sample files
    create_sample_files(&base_path).map_err(|e| BatchError::Io(e))?;

    // Get FTP configuration
    let config = if env::var("FTP_HOST").is_ok() {
        info!("Using FTP configuration from environment variables");
        FtpConfig::from_env()
    } else {
        info!("Using test FTP configuration (test.rebex.net)");
        info!("Note: This requires internet connection and may not always be available");
        FtpConfig::test_config()
    };

    info!(
        "FTP Configuration: {}:{} (user: {})",
        config.host, config.port, config.username
    );

    // Run demonstrations
    let results = vec![
        // Individual file operations
        demo_ftp_put(
            &config,
            &base_path.join("upload").join("data.txt"),
            "/tmp/demo_data.txt",
        ),
        demo_ftp_get(
            &config,
            "/tmp/demo_data.txt",
            &base_path.join("download").join("retrieved_data.txt"),
        ),
        // Folder operations
        demo_ftp_put_folder(&config, &base_path.join("upload"), "/tmp/demo_folder"),
        demo_ftp_get_folder(
            &config,
            "/tmp/demo_folder",
            &base_path.join("download_folder"),
        ),
        // Complete workflows
        demo_ftp_workflow(&config, &base_path),
        demo_ftp_recursive_workflow(&config, &base_path),
        // Error handling
        demo_error_handling(),
    ];

    let mut success_count = 0;
    let mut error_count = 0;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => {
                success_count += 1;
                info!("Demo {} completed successfully", i + 1);
            }
            Err(e) => {
                error_count += 1;
                info!("Demo {} failed: {}", i + 1, e);
                // Continue with other demos even if one fails
            }
        }
    }

    // Print summary
    print_results_summary(&base_path).map_err(|e| BatchError::Io(e))?;

    info!("=== Example Summary ===");
    info!("Successful operations: {}", success_count);
    info!("Failed operations: {}", error_count);
    info!(
        "Working directory: {} (preserved for inspection)",
        base_path.display()
    );

    if error_count > 0 {
        info!("Note: Some operations failed, which is expected when demonstrating error handling");
        info!("or when the test FTP server is not available.");
    }

    info!("FTP Transfer Tasklet Example completed");

    Ok(())
}
