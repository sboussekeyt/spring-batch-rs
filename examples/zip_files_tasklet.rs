//! # ZIP Files Tasklet Example
//!
//! This example demonstrates how to use the ZipTasklet to compress files and directories
//! as part of a batch processing job. It shows various configuration options including
//! compression levels, file filtering, and directory structure preservation.
//!
//! ## Features Demonstrated
//!
//! - Basic file and directory compression
//! - File filtering with include/exclude patterns
//! - Different compression levels
//! - Directory structure preservation options
//! - Integration with Spring Batch job execution
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example zip_files_tasklet --features zip
//! ```

use log::info;
use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    tasklet::zip::ZipTaskletBuilder,
    BatchError,
};
use std::{
    env::temp_dir,
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Creates a sample directory structure for demonstration purposes.
///
/// This function creates a temporary directory with various files and subdirectories
/// to showcase the ZIP tasklet's capabilities.
///
/// # Parameters
/// - `base_path`: The base directory where the sample structure will be created
///
/// # Returns
/// - `Ok(())`: Sample structure created successfully
/// - `Err(std::io::Error)`: Error creating the structure
fn create_sample_data(base_path: &Path) -> Result<(), std::io::Error> {
    info!("Creating sample data structure at: {}", base_path.display());

    // Create main directories
    let data_dir = base_path.join("data");
    let logs_dir = base_path.join("logs");
    let temp_dir = base_path.join("temp");
    let docs_dir = data_dir.join("documents");
    let config_dir = data_dir.join("config");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&logs_dir)?;
    fs::create_dir_all(&temp_dir)?;
    fs::create_dir_all(&docs_dir)?;
    fs::create_dir_all(&config_dir)?;

    // Create sample files
    let files_to_create = vec![
        (data_dir.join("readme.txt"), "This is a sample README file.\nIt contains important information about the project."),
        (data_dir.join("data.csv"), "name,age,city\nJohn,30,New York\nJane,25,Los Angeles\nBob,35,Chicago"),
        (docs_dir.join("manual.txt"), "User Manual\n============\n\n1. Getting Started\n2. Configuration\n3. Advanced Usage"),
        (docs_dir.join("changelog.md"), "# Changelog\n\n## Version 1.0.0\n- Initial release\n- Basic functionality"),
        (config_dir.join("settings.json"), r#"{"database": {"host": "localhost", "port": 5432}, "logging": {"level": "info"}}"#),
        (config_dir.join("app.properties"), "app.name=MyApplication\napp.version=1.0.0\ndebug=false"),
        (logs_dir.join("application.log"), "2024-01-01 10:00:00 INFO Application started\n2024-01-01 10:01:00 INFO Processing data\n2024-01-01 10:02:00 INFO Processing completed"),
        (logs_dir.join("error.log"), "2024-01-01 10:01:30 ERROR Database connection failed\n2024-01-01 10:01:35 ERROR Retrying connection"),
        (logs_dir.join("debug.log"), "2024-01-01 10:00:01 DEBUG Initializing components\n2024-01-01 10:00:02 DEBUG Loading configuration"),
        (temp_dir.join("cache.tmp"), "Temporary cache data that should be excluded"),
        (temp_dir.join("session.tmp"), "Temporary session data"),
        (base_path.join("important.txt"), "This is an important file in the root directory."),
    ];

    for (file_path, content) in files_to_create {
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        info!("Created file: {}", file_path.display());
    }

    info!("Sample data structure created successfully");
    Ok(())
}

/// Demonstrates basic ZIP compression of a directory.
///
/// This function shows how to compress an entire directory with default settings.
fn demo_basic_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    info!("=== Demo 1: Basic Directory Compression ===");

    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir)
        .target_path(output_dir.join("basic_archive.zip"))
        .build()?;

    let step = StepBuilder::new("basic-zip-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting basic compression job...");
    let result = job.run()?;
    info!("Basic compression completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates ZIP compression with file filtering.
///
/// This function shows how to include only specific file types and exclude others.
fn demo_filtered_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    info!("=== Demo 2: Filtered Compression (Only .txt and .md files, exclude .tmp) ===");

    // First, compress only text files
    let txt_zip_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir)
        .target_path(output_dir.join("text_files.zip"))
        .include_pattern("*.txt")
        .compression_level(9) // Maximum compression
        .build()?;

    let txt_step = StepBuilder::new("txt-zip-step")
        .tasklet(&txt_zip_tasklet)
        .build();

    // Then, compress log files excluding temporary files
    let log_zip_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("logs"))
        .target_path(output_dir.join("logs_archive.zip"))
        .include_pattern("*.log")
        .exclude_pattern("*.tmp")
        .compression_level(6)
        .build()?;

    let log_step = StepBuilder::new("log-zip-step")
        .tasklet(&log_zip_tasklet)
        .build();

    // Create a job with multiple steps
    let job = JobBuilder::new().start(&txt_step).next(&log_step).build();

    info!("Starting filtered compression job...");
    let result = job.run()?;
    info!("Filtered compression completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates ZIP compression with flattened directory structure.
///
/// This function shows how to compress files without preserving directory structure.
fn demo_flattened_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    info!("=== Demo 3: Flattened Compression (No Directory Structure) ===");

    let flat_zip_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data"))
        .target_path(output_dir.join("flattened_archive.zip"))
        .preserve_structure(false) // Flatten all files
        .compression_level(3) // Medium compression
        .build()?;

    let step = StepBuilder::new("flat-zip-step")
        .tasklet(&flat_zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting flattened compression job...");
    let result = job.run()?;
    info!("Flattened compression completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates ZIP compression of a single file.
///
/// This function shows how to compress individual files.
fn demo_single_file_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    info!("=== Demo 4: Single File Compression ===");

    let single_file_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("important.txt"))
        .target_path(output_dir.join("important_file.zip"))
        .compression_level(9) // Maximum compression for single file
        .build()?;

    let step = StepBuilder::new("single-file-zip-step")
        .tasklet(&single_file_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();

    info!("Starting single file compression job...");
    let result = job.run()?;
    info!("Single file compression completed in {:?}", result.duration);

    Ok(())
}

/// Demonstrates error handling with invalid paths.
///
/// This function shows how the ZIP tasklet handles various error conditions.
fn demo_error_handling() -> Result<(), BatchError> {
    info!("=== Demo 5: Error Handling ===");

    // Try to compress a non-existent directory
    info!("Testing compression of non-existent source...");
    let result = ZipTaskletBuilder::new()
        .source_path("/nonexistent/path")
        .target_path("/tmp/error_test.zip")
        .build();

    match result {
        Ok(_) => info!("Unexpected success - this should have failed"),
        Err(e) => info!("Expected error caught: {}", e),
    }

    // Try to build without required parameters
    info!("Testing builder validation...");
    let result = ZipTaskletBuilder::new().build();

    match result {
        Ok(_) => info!("Unexpected success - this should have failed"),
        Err(e) => info!("Expected validation error: {}", e),
    }

    Ok(())
}

/// Prints information about created ZIP files.
///
/// This function examines the output directory and reports on the ZIP files created.
fn print_results_summary(output_dir: &Path) -> Result<(), std::io::Error> {
    info!("=== Results Summary ===");

    let entries = fs::read_dir(output_dir)?;
    let mut zip_files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            let metadata = fs::metadata(&path)?;
            zip_files.push((path, metadata.len()));
        }
    }

    if zip_files.is_empty() {
        info!("No ZIP files found in output directory");
    } else {
        info!("Created ZIP files:");
        for (path, size) in zip_files {
            info!(
                "  - {} ({} bytes)",
                path.file_name().unwrap().to_string_lossy(),
                size
            );
        }
    }

    Ok(())
}

fn main() -> Result<(), BatchError> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting ZIP Files Tasklet Example");

    // Create temporary directories for the demo
    let temp_base = temp_dir().join("spring_batch_zip_demo");
    let source_dir = temp_base.join("source");
    let output_dir = temp_base.join("output");

    // Clean up any existing demo directory
    if temp_base.exists() {
        fs::remove_dir_all(&temp_base).map_err(|e| {
            BatchError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to clean up existing demo directory: {}", e),
            ))
        })?;
    }

    // Create directories
    fs::create_dir_all(&source_dir).map_err(BatchError::Io)?;
    fs::create_dir_all(&output_dir).map_err(BatchError::Io)?;

    info!("Demo directories created:");
    info!("  Source: {}", source_dir.display());
    info!("  Output: {}", output_dir.display());

    // Create sample data
    create_sample_data(&source_dir).map_err(BatchError::Io)?;

    // Run demonstrations
    demo_basic_compression(&source_dir, &output_dir)?;
    demo_filtered_compression(&source_dir, &output_dir)?;
    demo_flattened_compression(&source_dir, &output_dir)?;
    demo_single_file_compression(&source_dir, &output_dir)?;
    demo_error_handling()?;

    // Print summary
    print_results_summary(&output_dir).map_err(BatchError::Io)?;

    info!("ZIP Files Tasklet Example completed successfully!");
    info!(
        "Check the output directory for created ZIP files: {}",
        output_dir.display()
    );

    Ok(())
}
