//! # ZIP Tasklet Integration Tests
//!
//! This module contains comprehensive integration tests for the ZIP tasklet functionality.
//! It tests various scenarios including basic compression, file filtering, directory structure
//! preservation, error handling, and integration with Spring Batch job execution.

pub mod common;

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::{StepBuilder, StepStatus},
    },
    tasklet::zip::{ZipTasklet, ZipTaskletBuilder},
};
use tempfile::TempDir;
use zip::ZipArchive;

/// Helper function to create a comprehensive test directory structure
fn create_test_directory_structure(base_dir: &Path) -> Result<(), std::io::Error> {
    // Create main directories
    let data_dir = base_dir.join("data");
    let logs_dir = base_dir.join("logs");
    let temp_dir = base_dir.join("temp");
    let docs_dir = data_dir.join("documents");
    let config_dir = data_dir.join("config");
    let nested_dir = docs_dir.join("nested");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&logs_dir)?;
    fs::create_dir_all(&temp_dir)?;
    fs::create_dir_all(&docs_dir)?;
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&nested_dir)?;

    // Create test files with various extensions and content
    let test_files = vec![
        // Root level files
        (base_dir.join("readme.txt"), "This is a README file\nWith multiple lines\nAnd some content."),
        (base_dir.join("license.md"), "# License\n\nMIT License\n\nCopyright (c) 2024"),
        
        // Data directory files
        (data_dir.join("data.csv"), "name,age,city\nJohn,30,New York\nJane,25,Los Angeles\nBob,35,Chicago"),
        (data_dir.join("info.txt"), "Important information about the data processing."),
        (data_dir.join("backup.bak"), "Backup file content that should be preserved."),
        
        // Documents directory files
        (docs_dir.join("manual.txt"), "User Manual\n============\n\n1. Getting Started\n2. Configuration\n3. Advanced Usage"),
        (docs_dir.join("guide.md"), "# User Guide\n\n## Introduction\nThis is a comprehensive guide."),
        (docs_dir.join("notes.txt"), "Development notes and important reminders."),
        
        // Nested directory files
        (nested_dir.join("deep.txt"), "This file is deeply nested in the directory structure."),
        (nested_dir.join("config.json"), r#"{"setting": "value", "enabled": true}"#),
        
        // Config directory files
        (config_dir.join("app.properties"), "app.name=TestApp\napp.version=1.0.0\ndebug=true"),
        (config_dir.join("database.json"), r#"{"host": "localhost", "port": 5432, "database": "test"}"#),
        
        // Log files
        (logs_dir.join("application.log"), "2024-01-01 10:00:00 INFO Application started\n2024-01-01 10:01:00 INFO Processing data\n2024-01-01 10:02:00 INFO Processing completed"),
        (logs_dir.join("error.log"), "2024-01-01 10:01:30 ERROR Database connection failed\n2024-01-01 10:01:35 ERROR Retrying connection"),
        (logs_dir.join("debug.log"), "2024-01-01 10:00:01 DEBUG Initializing components\n2024-01-01 10:00:02 DEBUG Loading configuration"),
        (logs_dir.join("access.log"), "127.0.0.1 - - [01/Jan/2024:10:00:00 +0000] \"GET / HTTP/1.1\" 200 1234"),
        
        // Temporary files (should be excluded in some tests)
        (temp_dir.join("cache.tmp"), "Temporary cache data that should be excluded in filtered tests"),
        (temp_dir.join("session.tmp"), "Temporary session data"),
        (temp_dir.join("processing.tmp"), "Temporary processing file"),
    ];

    for (file_path, content) in test_files {
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

/// Helper function to verify ZIP archive contents
fn verify_zip_contents(zip_path: &Path, expected_files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    let mut found_files: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        found_files.push(file.name().to_string());
    }
    
    // Sort both vectors for comparison
    let mut expected_sorted = expected_files.to_vec();
    expected_sorted.sort();
    found_files.sort();
    
    assert_eq!(found_files.len(), expected_sorted.len(), 
               "Expected {} files, found {}: {:?}", expected_sorted.len(), found_files.len(), found_files);
    
    for expected in expected_sorted {
        assert!(found_files.contains(&expected.to_string()), 
                "Expected file '{}' not found in archive. Found files: {:?}", expected, found_files);
    }
    
    Ok(())
}

/// Helper function to count files in ZIP archive
fn count_zip_files(zip_path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let archive = ZipArchive::new(file)?;
    Ok(archive.len())
}

/// Helper function to verify file content in ZIP archive
fn verify_file_content_in_zip(zip_path: &Path, file_name: &str, expected_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    let mut zip_file = archive.by_name(file_name)?;
    let mut content = String::new();
    zip_file.read_to_string(&mut content)?;
    
    assert_eq!(content, expected_content, "File content mismatch for '{}'", file_name);
    Ok(())
}

#[test]
fn test_basic_directory_compression() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let target_zip = temp_dir.path().join("archive.zip");

    // Create test directory structure
    create_test_directory_structure(&source_dir).unwrap();

    // Create ZIP tasklet
    let zip_tasklet = ZipTasklet::new(&source_dir, &target_zip).unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-directory-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify ZIP contains expected files
    let file_count = count_zip_files(&target_zip).unwrap();
    assert!(file_count > 15, "Should contain multiple files, found: {}", file_count);

    // Verify some specific files exist in the archive
    verify_file_content_in_zip(&target_zip, "readme.txt", "This is a README file\nWith multiple lines\nAnd some content.").unwrap();
    verify_file_content_in_zip(&target_zip, "data/data.csv", "name,age,city\nJohn,30,New York\nJane,25,Los Angeles\nBob,35,Chicago").unwrap();
}

#[test]
fn test_single_file_compression() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("test.txt");
    let target_zip = temp_dir.path().join("single_file.zip");

    // Create test file
    fs::write(&source_file, "Hello, World!\nThis is a test file for single file compression.").unwrap();

    // Create ZIP tasklet
    let zip_tasklet = ZipTasklet::new(&source_file, &target_zip).unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-single-file-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify ZIP contains exactly one file
    let file_count = count_zip_files(&target_zip).unwrap();
    assert_eq!(file_count, 1, "Should contain exactly one file");

    // Verify file content
    verify_file_content_in_zip(&target_zip, "test.txt", "Hello, World!\nThis is a test file for single file compression.").unwrap();
}

#[test]
fn test_compression_with_include_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let target_zip = temp_dir.path().join("txt_files.zip");

    // Create test directory structure
    create_test_directory_structure(&source_dir).unwrap();

    // Create ZIP tasklet with include pattern for .txt files only
    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(&source_dir)
        .target_path(&target_zip)
        .include_pattern("*.txt")
        .compression_level(9)
        .build().unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-txt-files-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify only .txt files are included
    let expected_txt_files = vec![
        "readme.txt",
        "data/info.txt",
        "data/documents/manual.txt",
        "data/documents/notes.txt",
        "data/documents/nested/deep.txt",
    ];

    verify_zip_contents(&target_zip, &expected_txt_files).unwrap();
}

#[test]
fn test_compression_with_exclude_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let target_zip = temp_dir.path().join("no_tmp_files.zip");

    // Create test directory structure
    create_test_directory_structure(&source_dir).unwrap();

    // Create ZIP tasklet with exclude pattern for .tmp files
    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(&source_dir)
        .target_path(&target_zip)
        .exclude_pattern("*.tmp")
        .compression_level(6)
        .build().unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-no-tmp-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify .tmp files are excluded
    let file = File::open(&target_zip).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        assert!(!file.name().ends_with(".tmp"), "Found .tmp file in archive: {}", file.name());
    }

    // Should still contain other files
    let file_count = count_zip_files(&target_zip).unwrap();
    assert!(file_count > 10, "Should contain multiple non-tmp files, found: {}", file_count);
}

#[test]
fn test_compression_with_flattened_structure() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let target_zip = temp_dir.path().join("flattened.zip");

    // Create test directory structure
    create_test_directory_structure(&source_dir).unwrap();

    // Create ZIP tasklet with flattened structure
    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(&source_dir)
        .target_path(&target_zip)
        .preserve_structure(false)
        .compression_level(3)
        .build().unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-flattened-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify all files are at root level (no directory separators)
    let file = File::open(&target_zip).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        assert!(!file.name().contains('/'), "Found file with directory structure: {}", file.name());
    }
}

#[test]
fn test_multi_step_job_with_zip_tasklets() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let logs_zip = temp_dir.path().join("logs.zip");
    let docs_zip = temp_dir.path().join("docs.zip");
    let data_zip = temp_dir.path().join("data.zip");

    // Create test directory structure
    create_test_directory_structure(&source_dir).unwrap();

    // Create multiple ZIP tasklets for different directories
    let logs_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("logs"))
        .target_path(&logs_zip)
        .include_pattern("*.log")
        .compression_level(9)
        .build().unwrap();

    let docs_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data").join("documents"))
        .target_path(&docs_zip)
        .compression_level(6)
        .build().unwrap();

    let data_tasklet = ZipTaskletBuilder::new()
        .source_path(source_dir.join("data"))
        .target_path(&data_zip)
        .exclude_pattern("documents/**")
        .compression_level(3)
        .build().unwrap();

    // Create steps
    let logs_step = StepBuilder::new("zip-logs-step")
        .tasklet(&logs_tasklet)
        .build();

    let docs_step = StepBuilder::new("zip-docs-step")
        .tasklet(&docs_tasklet)
        .build();

    let data_step = StepBuilder::new("zip-data-step")
        .tasklet(&data_tasklet)
        .build();

    // Create multi-step job
    let job = JobBuilder::new()
        .start(&logs_step)
        .next(&docs_step)
        .next(&data_step)
        .build();

    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");

    // Verify all ZIP files were created
    assert!(logs_zip.exists(), "Logs ZIP should be created");
    assert!(docs_zip.exists(), "Docs ZIP should be created");
    assert!(data_zip.exists(), "Data ZIP should be created");

    // Verify step executions
    let logs_execution = job.get_step_execution("zip-logs-step").unwrap();
    let docs_execution = job.get_step_execution("zip-docs-step").unwrap();
    let data_execution = job.get_step_execution("zip-data-step").unwrap();

    assert_eq!(logs_execution.status, StepStatus::Success);
    assert_eq!(docs_execution.status, StepStatus::Success);
    assert_eq!(data_execution.status, StepStatus::Success);

    // Verify logs ZIP contains only .log files
    let logs_file_count = count_zip_files(&logs_zip).unwrap();
    assert_eq!(logs_file_count, 4, "Logs ZIP should contain 4 .log files");

    // Verify docs ZIP contains documentation files
    let docs_file_count = count_zip_files(&docs_zip).unwrap();
    assert!(docs_file_count >= 3, "Docs ZIP should contain at least 3 files");
}

#[test]
fn test_error_handling_nonexistent_source() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_source = temp_dir.path().join("nonexistent");
    let target_zip = temp_dir.path().join("should_not_be_created.zip");

    // Try to create ZIP tasklet with nonexistent source
    let result = ZipTasklet::new(&nonexistent_source, &target_zip);
    assert!(result.is_err(), "Should fail with nonexistent source");

    // Verify ZIP file was not created
    assert!(!target_zip.exists(), "ZIP file should not be created on error");
}

#[test]
fn test_error_handling_invalid_target_directory() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("test.txt");
    
    // Create source file
    fs::write(&source_file, "test content").unwrap();

    // Try to create ZIP in a directory that cannot be created (using a file as directory)
    let invalid_target = source_file.join("invalid").join("target.zip");
    
    let result = ZipTasklet::new(&source_file, &invalid_target);
    assert!(result.is_err(), "Should fail with invalid target directory");
}

#[test]
fn test_builder_validation() {
    // Test missing source path
    let result = ZipTaskletBuilder::new()
        .target_path("/tmp/test.zip")
        .build();
    assert!(result.is_err(), "Should fail without source path");

    // Test missing target path
    let result = ZipTaskletBuilder::new()
        .source_path("/tmp/source")
        .build();
    assert!(result.is_err(), "Should fail without target path");

    // Test both missing
    let result = ZipTaskletBuilder::new().build();
    assert!(result.is_err(), "Should fail without both paths");
}

#[test]
fn test_empty_directory_compression() {
    let temp_dir = TempDir::new().unwrap();
    let empty_source = temp_dir.path().join("empty");
    let target_zip = temp_dir.path().join("empty.zip");

    // Create empty directory
    fs::create_dir(&empty_source).unwrap();

    // Create ZIP tasklet
    let zip_tasklet = ZipTasklet::new(&empty_source, &target_zip).unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-empty-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created even for empty directory");

    // Verify ZIP is empty or contains only directory entries
    let file_count = count_zip_files(&target_zip).unwrap();
    assert_eq!(file_count, 0, "Empty directory should produce empty ZIP");
}

#[test]
fn test_large_file_compression() {
    let temp_dir = TempDir::new().unwrap();
    let large_file = temp_dir.path().join("large.txt");
    let target_zip = temp_dir.path().join("large.zip");

    // Create a large file (1MB of repeated content)
    let large_content = "This is a line of text that will be repeated many times to create a large file.\n".repeat(10000);
    fs::write(&large_file, &large_content).unwrap();

    // Create ZIP tasklet with maximum compression
    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path(&large_file)
        .target_path(&target_zip)
        .compression_level(9)
        .build().unwrap();

    // Create and execute step
    let step = StepBuilder::new("zip-large-file-step")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // Verify job execution
    assert!(result.is_ok(), "Job should complete successfully");
    assert!(target_zip.exists(), "ZIP file should be created");

    // Verify compression was effective
    let original_size = fs::metadata(&large_file).unwrap().len();
    let compressed_size = fs::metadata(&target_zip).unwrap().len();
    
    assert!(compressed_size < original_size, 
            "Compressed size ({} bytes) should be less than original ({} bytes)", 
            compressed_size, original_size);

    // For repetitive content, compression should be very effective
    let compression_ratio = compressed_size as f64 / original_size as f64;
    assert!(compression_ratio < 0.1, 
            "Compression ratio should be less than 10% for repetitive content, got {:.2}%", 
            compression_ratio * 100.0);
}

#[test]
fn test_compression_levels() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("large_test.txt");
    
    // Create a larger file for better compression testing
    let large_content = "This is a test file with repeated content. ".repeat(1000);
    fs::write(&source_file, &large_content).unwrap();

    // Test different compression levels
    let compression_levels = vec![0, 3, 6, 9];
    let mut zip_sizes = Vec::new();

    for level in compression_levels {
        let target_zip = temp_dir.path().join(format!("compressed_level_{}.zip", level));
        
        let zip_tasklet = ZipTaskletBuilder::new()
            .source_path(&source_file)
            .target_path(&target_zip)
            .compression_level(level)
            .build().unwrap();

        let step = StepBuilder::new(&format!("zip-level-{}-step", level))
            .tasklet(&zip_tasklet)
            .build();

        let job = JobBuilder::new().start(&step).build();
        let result = job.run();

        if let Err(ref e) = result {
            eprintln!("Job failed for level {}: {:?}", level, e);
        }
        assert!(result.is_ok(), "Job should complete successfully for level {}", level);
        assert!(target_zip.exists(), "ZIP file should be created for level {}", level);

        let zip_size = fs::metadata(&target_zip).unwrap().len();
        zip_sizes.push((level, zip_size));
    }

    // Verify that higher compression levels generally produce smaller files
    // (though this might not always be true for very small files)
    let level_0_size = zip_sizes.iter().find(|(level, _)| *level == 0).unwrap().1;
    let level_9_size = zip_sizes.iter().find(|(level, _)| *level == 9).unwrap().1;
    
    // Level 9 should be smaller or equal to level 0 for repetitive content
    assert!(level_9_size <= level_0_size, 
            "Level 9 compression ({} bytes) should be <= level 0 ({} bytes)", 
            level_9_size, level_0_size);
} 