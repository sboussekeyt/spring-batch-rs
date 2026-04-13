//! # ZIP Tasklet Examples
//!
//! Demonstrates ZIP compression operations with Spring Batch RS tasklets.
//!
//! ## Features Demonstrated
//! - Basic directory compression
//! - File filtering with include/exclude patterns
//! - Compression levels
//! - Flattened vs preserved directory structure
//! - Single file compression
//!
//! ## Run
//! ```bash
//! cargo run --example tasklet_zip --features zip
//! ```

use log::info;
use spring_batch_rs::{
    BatchError,
    core::{
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    tasklet::zip::ZipTaskletBuilder,
};
use std::{
    env::temp_dir,
    fs::{self, File},
    io::Write,
    path::Path,
};

// =============================================================================
// Setup: Create Test Data
// =============================================================================

/// Creates a sample directory structure for the examples.
fn create_sample_data(base_path: &Path) -> Result<(), std::io::Error> {
    info!("Creating sample data at: {}", base_path.display());

    // Create directories
    fs::create_dir_all(base_path.join("data/documents"))?;
    fs::create_dir_all(base_path.join("data/config"))?;
    fs::create_dir_all(base_path.join("logs"))?;
    fs::create_dir_all(base_path.join("temp"))?;

    // Create sample files
    let files = vec![
        (
            "data/readme.txt",
            "Project README\n==============\nThis is a sample project.",
        ),
        ("data/data.csv", "name,value\nAlice,100\nBob,200"),
        (
            "data/documents/manual.txt",
            "User Manual\n-----------\n1. Getting Started",
        ),
        ("data/documents/notes.md", "# Notes\n\n- Item 1\n- Item 2"),
        (
            "data/config/settings.json",
            r#"{"debug": false, "version": "1.0"}"#,
        ),
        ("logs/app.log", "2024-01-01 INFO: Application started"),
        ("logs/error.log", "2024-01-01 ERROR: Connection failed"),
        ("temp/cache.tmp", "Temporary cache data"),
        ("temp/session.tmp", "Session data"),
    ];

    for (path, content) in files {
        let file_path = base_path.join(path);
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
    }

    info!("Sample data created successfully");
    Ok(())
}

// =============================================================================
// Example 1: Basic Compression
// =============================================================================

/// Compresses an entire directory with default settings.
fn example_basic_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    println!("=== Example 1: Basic Directory Compression ===");

    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data"))
        .target_path(output_dir.join("basic_archive.zip"))
        .build()?;

    let step = StepBuilder::new("basic-zip").tasklet(&zip_tasklet).build();
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    println!("  Created: basic_archive.zip");
    println!("  Duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 2: Filtered Compression
// =============================================================================

/// Compresses only specific file types.
fn example_filtered_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 2: Filtered Compression ===");

    // Compress only .txt files
    let txt_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir)
        .target_path(output_dir.join("text_files.zip"))
        .include_pattern("*.txt")
        .compression_level(9) // Maximum compression
        .build()?;

    // Compress only .log files
    let log_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("logs"))
        .target_path(output_dir.join("logs_archive.zip"))
        .include_pattern("*.log")
        .exclude_pattern("*.tmp")
        .build()?;

    let txt_step = StepBuilder::new("zip-txt").tasklet(&txt_tasklet).build();
    let log_step = StepBuilder::new("zip-logs").tasklet(&log_tasklet).build();

    let job = JobBuilder::new().start(&txt_step).next(&log_step).build();
    job.run()?;

    println!("  Created: text_files.zip (only .txt files)");
    println!("  Created: logs_archive.zip (only .log files)");
    Ok(())
}

// =============================================================================
// Example 3: Flattened Compression
// =============================================================================

/// Compresses files without preserving directory structure.
fn example_flattened_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 3: Flattened Compression ===");

    let flat_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data"))
        .target_path(output_dir.join("flat_archive.zip"))
        .preserve_structure(false) // All files at root level
        .compression_level(6)
        .build()?;

    let step = StepBuilder::new("flat-zip").tasklet(&flat_tasklet).build();
    let job = JobBuilder::new().start(&step).build();
    job.run()?;

    println!("  Created: flat_archive.zip (no subdirectories)");
    Ok(())
}

// =============================================================================
// Example 4: Single File Compression
// =============================================================================

/// Compresses a single file.
fn example_single_file_compression(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 4: Single File Compression ===");

    let single_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data/readme.txt"))
        .target_path(output_dir.join("readme.zip"))
        .compression_level(9)
        .build()?;

    let step = StepBuilder::new("single-zip")
        .tasklet(&single_tasklet)
        .build();
    let job = JobBuilder::new().start(&step).build();
    job.run()?;

    println!("  Created: readme.zip (single file)");
    Ok(())
}

// =============================================================================
// Example 5: Multi-Step Archive Job
// =============================================================================

/// Creates multiple archives in a single job.
fn example_multi_step_archive(source_dir: &Path, output_dir: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 5: Multi-Step Archive Job ===");

    // Step 1: Archive data files
    let data_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data"))
        .target_path(output_dir.join("data_backup.zip"))
        .compression_level(9)
        .build()?;

    // Step 2: Archive log files
    let log_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("logs"))
        .target_path(output_dir.join("logs_backup.zip"))
        .compression_level(6)
        .build()?;

    let step1 = StepBuilder::new("archive-data")
        .tasklet(&data_tasklet)
        .build();
    let step2 = StepBuilder::new("archive-logs")
        .tasklet(&log_tasklet)
        .build();

    let job = JobBuilder::new().start(&step1).next(&step2).build();
    let result = job.run()?;

    println!("  Created: data_backup.zip");
    println!("  Created: logs_backup.zip");
    println!("  Total duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Results Summary
// =============================================================================

/// Prints information about created ZIP files.
fn print_results_summary(output_dir: &Path) -> Result<(), std::io::Error> {
    println!("\n=== Results Summary ===");

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            let metadata = fs::metadata(&path)?;
            println!(
                "  {} ({} bytes)",
                path.file_name().unwrap().to_string_lossy(),
                metadata.len()
            );
        }
    }

    Ok(())
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<(), BatchError> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("ZIP Tasklet Examples");
    println!("====================\n");

    // Setup directories
    let temp_base = temp_dir().join("spring_batch_zip_examples");
    let source_dir = temp_base.join("source");
    let output_dir = temp_base.join("output");

    // Clean up existing
    if temp_base.exists() {
        fs::remove_dir_all(&temp_base).map_err(|e| {
            BatchError::Io(std::io::Error::other(format!("Failed to clean up: {}", e)))
        })?;
    }

    fs::create_dir_all(&source_dir).map_err(BatchError::Io)?;
    fs::create_dir_all(&output_dir).map_err(BatchError::Io)?;

    // Create sample data
    create_sample_data(&source_dir).map_err(BatchError::Io)?;
    println!();

    // Run examples
    example_basic_compression(&source_dir, &output_dir)?;
    example_filtered_compression(&source_dir, &output_dir)?;
    example_flattened_compression(&source_dir, &output_dir)?;
    example_single_file_compression(&source_dir, &output_dir)?;
    example_multi_step_archive(&source_dir, &output_dir)?;

    // Print results
    print_results_summary(&output_dir).map_err(BatchError::Io)?;

    println!("\n✓ All ZIP examples completed successfully!");
    println!("  Output directory: {}", output_dir.display());

    Ok(())
}
