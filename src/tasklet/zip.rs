//! # Zip File Tasklet
//!
//! This module provides a tasklet for creating ZIP archives from files and directories.
//! It's designed to be similar to Spring Batch's file compression capabilities.
//!
//! ## Features
//!
//! - Compress single files or entire directories
//! - Configurable compression level
//! - Support for filtering files to include/exclude
//! - Proper error handling and logging
//! - Builder pattern for easy configuration
//!
//! ## Examples
//!
//! ### Basic ZIP Creation
//!
//! ```rust
//! use spring_batch_rs::core::step::{StepBuilder, StepExecution, Step};
//! use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
//! use std::path::Path;
//! use std::fs;
//! use std::env::temp_dir;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! // Create test data directory and file
//! let temp_data_dir = temp_dir().join("test_data_zip");
//! fs::create_dir_all(&temp_data_dir).unwrap();
//! fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();
//!
//! let archive_path = temp_dir().join("archive_test.zip");
//!
//! let zip_tasklet = ZipTaskletBuilder::new()
//!     .source_path(&temp_data_dir)
//!     .target_path(&archive_path)
//!     .compression_level(6)
//!     .build()?;
//!
//! let step = StepBuilder::new("zip-files")
//!     .tasklet(&zip_tasklet)
//!     .build();
//!
//! let mut step_execution = StepExecution::new("zip-files");
//! step.execute(&mut step_execution)?;
//!
//! // Cleanup test files
//! fs::remove_file(&archive_path).ok();
//! fs::remove_dir_all(&temp_data_dir).ok();
//! # Ok(())
//! # }
//! ```
//!
//! ### ZIP with File Filtering
//!
//! ```rust
//! use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let zip_tasklet = ZipTaskletBuilder::new()
//!     .source_path("./logs")
//!     .target_path("./logs_archive.zip")
//!     .include_pattern("*.log")
//!     .exclude_pattern("*.tmp")
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use crate::{
    core::step::{RepeatStatus, StepExecution, Tasklet},
    BatchError,
};
use log::{debug, info, warn};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

/// A tasklet for creating ZIP archives from files and directories.
///
/// This tasklet provides functionality similar to Spring Batch's file compression
/// capabilities, allowing you to compress files and directories into ZIP archives
/// as part of a batch processing step.
///
/// # Examples
///
/// ```rust
/// use spring_batch_rs::core::step::{StepExecution, RepeatStatus, Tasklet};
/// use spring_batch_rs::tasklet::zip::ZipTasklet;
/// use spring_batch_rs::BatchError;
/// use std::path::Path;
/// use std::fs;
/// use std::env::temp_dir;
///
/// # fn example() -> Result<(), BatchError> {
/// // Create test data directory and file
/// let temp_source_dir = temp_dir().join("test_source");
/// fs::create_dir_all(&temp_source_dir).unwrap();
/// fs::write(temp_source_dir.join("test.txt"), "test content").unwrap();
///
/// let archive_path = temp_dir().join("test_archive.zip");
///
/// let tasklet = ZipTasklet::new(
///     &temp_source_dir,
///     &archive_path,
/// )?;
///
/// let step_execution = StepExecution::new("zip-step");
/// let result = tasklet.execute(&step_execution)?;
/// assert_eq!(result, RepeatStatus::Finished);
///
/// // Cleanup test files
/// fs::remove_file(&archive_path).ok();
/// fs::remove_dir_all(&temp_source_dir).ok();
/// # Ok(())
/// # }
/// ```
pub struct ZipTasklet {
    /// Source path to compress (file or directory)
    source_path: PathBuf,
    /// Target ZIP file path
    target_path: PathBuf,
    /// Compression level (0-9, where 9 is maximum compression)
    compression_level: i32,
    /// Pattern for files to include (glob pattern)
    include_pattern: Option<String>,
    /// Pattern for files to exclude (glob pattern)
    exclude_pattern: Option<String>,
    /// Whether to preserve directory structure
    preserve_structure: bool,
}

impl ZipTasklet {
    /// Creates a new ZipTasklet with default settings.
    ///
    /// # Parameters
    /// - `source_path`: Path to the file or directory to compress
    /// - `target_path`: Path where the ZIP file will be created
    ///
    /// # Returns
    /// - `Ok(ZipTasklet)`: Successfully created tasklet
    /// - `Err(BatchError)`: Error if paths are invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test data directory
    /// let temp_data_dir = temp_dir().join("test_data_new");
    /// fs::create_dir_all(&temp_data_dir).unwrap();
    /// fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();
    ///
    /// let backup_path = temp_dir().join("backup.zip");
    ///
    /// let tasklet = ZipTasklet::new(
    ///     &temp_data_dir,
    ///     &backup_path,
    /// )?;
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_data_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(source_path: P, target_path: P) -> Result<Self, BatchError> {
        let source = source_path.as_ref().to_path_buf();
        let target = target_path.as_ref().to_path_buf();

        // Validate source path exists
        if !source.exists() {
            return Err(BatchError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", source.display()),
            )));
        }

        // Ensure target directory exists
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    BatchError::Io(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("Cannot create target directory {}: {}", parent.display(), e),
                    ))
                })?;
            }
        }

        Ok(Self {
            source_path: source,
            target_path: target,
            compression_level: 6, // Default compression level
            include_pattern: None,
            exclude_pattern: None,
            preserve_structure: true,
        })
    }

    /// Sets the compression level for the ZIP archive.
    ///
    /// # Parameters
    /// - `level`: Compression level (0-9, where 0 is no compression and 9 is maximum)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test data directory
    /// let temp_data_dir = temp_dir().join("test_data_compression");
    /// fs::create_dir_all(&temp_data_dir).unwrap();
    /// fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();
    ///
    /// let backup_path = temp_dir().join("backup_compression.zip");
    ///
    /// let mut tasklet = ZipTasklet::new(
    ///     &temp_data_dir,
    ///     &backup_path,
    /// )?;
    /// tasklet.set_compression_level(9); // Maximum compression
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_data_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level.clamp(0, 9);
    }

    /// Sets a pattern for files to include in the archive.
    ///
    /// # Parameters
    /// - `pattern`: Glob pattern for files to include (e.g., "*.txt", "**/*.log")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test logs directory
    /// let temp_logs_dir = temp_dir().join("test_logs");
    /// fs::create_dir_all(&temp_logs_dir).unwrap();
    /// fs::write(temp_logs_dir.join("app.log"), "log content").unwrap();
    ///
    /// let logs_zip_path = temp_dir().join("logs.zip");
    ///
    /// let mut tasklet = ZipTasklet::new(
    ///     &temp_logs_dir,
    ///     &logs_zip_path,
    /// )?;
    /// tasklet.set_include_pattern("*.log");
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_logs_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_include_pattern<S: Into<String>>(&mut self, pattern: S) {
        self.include_pattern = Some(pattern.into());
    }

    /// Sets a pattern for files to exclude from the archive.
    ///
    /// # Parameters
    /// - `pattern`: Glob pattern for files to exclude (e.g., "*.tmp", "**/target/**")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test project directory
    /// let temp_project_dir = temp_dir().join("test_project");
    /// fs::create_dir_all(&temp_project_dir).unwrap();
    /// fs::write(temp_project_dir.join("src.rs"), "source code").unwrap();
    ///
    /// let project_zip_path = temp_dir().join("project.zip");
    ///
    /// let mut tasklet = ZipTasklet::new(
    ///     &temp_project_dir,
    ///     &project_zip_path,
    /// )?;
    /// tasklet.set_exclude_pattern("target/**");
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_project_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_exclude_pattern<S: Into<String>>(&mut self, pattern: S) {
        self.exclude_pattern = Some(pattern.into());
    }

    /// Sets whether to preserve directory structure in the archive.
    ///
    /// # Parameters
    /// - `preserve`: If true, maintains directory structure; if false, flattens all files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test data directory
    /// let temp_data_dir = temp_dir().join("test_data_flat");
    /// fs::create_dir_all(&temp_data_dir).unwrap();
    /// fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();
    ///
    /// let flat_zip_path = temp_dir().join("flat.zip");
    ///
    /// let mut tasklet = ZipTasklet::new(
    ///     &temp_data_dir,
    ///     &flat_zip_path,
    /// )?;
    /// tasklet.set_preserve_structure(false); // Flatten all files
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_data_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_preserve_structure(&mut self, preserve: bool) {
        self.preserve_structure = preserve;
    }

    /// Checks if a file should be included based on include/exclude patterns.
    ///
    /// # Parameters
    /// - `path`: Path to check
    ///
    /// # Returns
    /// - `true` if the file should be included
    /// - `false` if the file should be excluded
    fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude pattern first
        if let Some(ref exclude) = self.exclude_pattern {
            if self.matches_pattern(&path_str, exclude) {
                debug!("Excluding file due to exclude pattern: {}", path.display());
                return false;
            }
        }

        // Check include pattern
        if let Some(ref include) = self.include_pattern {
            if !self.matches_pattern(&path_str, include) {
                debug!("Excluding file due to include pattern: {}", path.display());
                return false;
            }
        }

        true
    }

    /// Simple pattern matching for file paths.
    ///
    /// This is a basic implementation that supports:
    /// - `*` for any characters within a filename
    /// - `**` for any characters including directory separators
    ///
    /// # Parameters
    /// - `path`: Path to match against
    /// - `pattern`: Pattern to match
    ///
    /// # Returns
    /// - `true` if the path matches the pattern
    /// - `false` otherwise
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob-like pattern matching
        if pattern == "*" || pattern == "**" {
            return true;
        }

        if pattern.contains("**") {
            // Handle recursive patterns like "**/*.txt"
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                // For "**/*.txt", prefix is "" and suffix is "/*.txt"
                // We need to check if the path ends with the suffix pattern
                if prefix.is_empty() && suffix.starts_with('/') {
                    // Remove the leading '/' from suffix and check if path ends with it
                    let suffix_pattern = &suffix[1..];
                    return self.matches_simple_pattern(path, suffix_pattern)
                        || path
                            .split('/')
                            .any(|segment| self.matches_simple_pattern(segment, suffix_pattern));
                } else {
                    return path.starts_with(prefix) && path.ends_with(suffix);
                }
            }
        }

        self.matches_simple_pattern(path, pattern)
    }

    fn matches_simple_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Handle single-level wildcards
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }

        // Exact match
        path == pattern
    }

    /// Compresses a single file into the ZIP archive.
    ///
    /// This method handles compression level 0 specially by using the `Stored` compression method
    /// (no compression) instead of `Deflated` with level 0, which is the correct approach for
    /// the ZIP format specification.
    ///
    /// # Parameters
    /// - `zip_writer`: ZIP writer instance
    /// - `file_path`: Path to the file to compress
    /// - `archive_path`: Path within the archive
    ///
    /// # Returns
    /// - `Ok(())`: File successfully compressed
    /// - `Err(BatchError)`: Error during compression
    fn compress_file(
        &self,
        zip_writer: &mut ZipWriter<File>,
        file_path: &Path,
        archive_path: &str,
    ) -> Result<(), BatchError> {
        debug!(
            "Compressing file: {} -> {}",
            file_path.display(),
            archive_path
        );

        let options = if self.compression_level == 0 {
            // Use stored method (no compression) for level 0 - this is the correct approach
            // for ZIP format as Deflated with level 0 can cause issues with some ZIP readers
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
        } else {
            // Use deflated method with specified compression level (1-9)
            SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(self.compression_level as i64))
        };

        zip_writer
            .start_file(archive_path, options)
            .map_err(|e| BatchError::Io(io::Error::other(e)))?;

        let file_content = fs::read(file_path).map_err(BatchError::Io)?;
        zip_writer
            .write_all(&file_content)
            .map_err(BatchError::Io)?;

        info!("Successfully compressed: {}", archive_path);
        Ok(())
    }

    /// Recursively compresses a directory into the ZIP archive.
    ///
    /// # Parameters
    /// - `zip_writer`: ZIP writer instance
    /// - `dir_path`: Path to the directory to compress
    /// - `base_path`: Base path for calculating relative paths
    ///
    /// # Returns
    /// - `Ok(usize)`: Number of files compressed
    /// - `Err(BatchError)`: Error during compression
    fn compress_directory(
        &self,
        zip_writer: &mut ZipWriter<File>,
        dir_path: &Path,
        base_path: &Path,
    ) -> Result<usize, BatchError> {
        let mut file_count = 0;

        let entries = fs::read_dir(dir_path).map_err(BatchError::Io)?;

        for entry in entries {
            let entry = entry.map_err(BatchError::Io)?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                if self.should_include_file(&entry_path) {
                    let archive_path = if self.preserve_structure {
                        entry_path
                            .strip_prefix(base_path)
                            .unwrap_or(&entry_path)
                            .to_string_lossy()
                            .replace('\\', "/") // Normalize path separators for ZIP
                    } else {
                        entry_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    };

                    self.compress_file(zip_writer, &entry_path, &archive_path)?;
                    file_count += 1;
                }
            } else if entry_path.is_dir() {
                file_count += self.compress_directory(zip_writer, &entry_path, base_path)?;
            }
        }

        Ok(file_count)
    }
}

impl Tasklet for ZipTasklet {
    /// Executes the ZIP compression operation.
    ///
    /// This method creates a ZIP archive from the configured source path,
    /// applying any specified filters and compression settings.
    ///
    /// # Parameters
    /// - `step_execution`: The current step execution context
    ///
    /// # Returns
    /// - `Ok(RepeatStatus::Finished)`: Compression completed successfully
    /// - `Err(BatchError)`: Error during compression
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::core::step::{StepExecution, Tasklet};
    /// use spring_batch_rs::tasklet::zip::ZipTasklet;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// let tasklet = ZipTasklet::new(
    ///     Path::new("./data"),
    ///     Path::new("./archive.zip"),
    /// )?;
    ///
    /// let step_execution = StepExecution::new("zip-step");
    /// let result = tasklet.execute(&step_execution)?;
    /// # Ok(())
    /// # }
    /// ```
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        info!(
            "Starting ZIP compression: {} -> {}",
            self.source_path.display(),
            self.target_path.display()
        );

        // Create the ZIP file
        let zip_file = File::create(&self.target_path).map_err(BatchError::Io)?;
        let mut zip_writer = ZipWriter::new(zip_file);

        let file_count = if self.source_path.is_file() {
            // Compress single file
            if self.should_include_file(&self.source_path) {
                let archive_name = self
                    .source_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                self.compress_file(&mut zip_writer, &self.source_path, &archive_name)?;
                1
            } else {
                warn!(
                    "Source file excluded by filters: {}",
                    self.source_path.display()
                );
                0
            }
        } else if self.source_path.is_dir() {
            // Compress directory
            self.compress_directory(&mut zip_writer, &self.source_path, &self.source_path)?
        } else {
            return Err(BatchError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid source path: {}", self.source_path.display()),
            )));
        };

        // Finalize the ZIP file
        zip_writer
            .finish()
            .map_err(|e| BatchError::Io(io::Error::other(e)))?;

        info!(
            "ZIP compression completed successfully. {} files compressed to {}",
            file_count,
            self.target_path.display()
        );

        Ok(RepeatStatus::Finished)
    }
}

/// Builder for creating ZipTasklet instances with a fluent interface.
///
/// This builder provides a convenient way to configure ZIP tasklets with
/// various options such as compression level, file filters, and directory
/// structure preservation.
///
/// # Examples
///
/// ```rust
/// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = ZipTaskletBuilder::new()
///     .source_path("./data")
///     .target_path("./backup.zip")
///     .compression_level(9)
///     .include_pattern("*.txt")
///     .exclude_pattern("*.tmp")
///     .preserve_structure(true)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct ZipTaskletBuilder {
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
    compression_level: i32,
    include_pattern: Option<String>,
    exclude_pattern: Option<String>,
    preserve_structure: bool,
}

impl Default for ZipTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ZipTaskletBuilder {
    /// Creates a new ZipTaskletBuilder with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            source_path: None,
            target_path: None,
            compression_level: 6,
            include_pattern: None,
            exclude_pattern: None,
            preserve_structure: true,
        }
    }

    /// Sets the source path to compress.
    ///
    /// # Parameters
    /// - `path`: Path to the file or directory to compress
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .source_path("./data");
    /// ```
    pub fn source_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the target ZIP file path.
    ///
    /// # Parameters
    /// - `path`: Path where the ZIP file will be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .target_path("./archive.zip");
    /// ```
    pub fn target_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.target_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the compression level.
    ///
    /// # Parameters
    /// - `level`: Compression level (0-9, where 0 is no compression and 9 is maximum)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .compression_level(9); // Maximum compression
    /// ```
    pub fn compression_level(mut self, level: i32) -> Self {
        self.compression_level = level.clamp(0, 9);
        self
    }

    /// Sets a pattern for files to include in the archive.
    ///
    /// # Parameters
    /// - `pattern`: Glob pattern for files to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .include_pattern("*.log");
    /// ```
    pub fn include_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.include_pattern = Some(pattern.into());
        self
    }

    /// Sets a pattern for files to exclude from the archive.
    ///
    /// # Parameters
    /// - `pattern`: Glob pattern for files to exclude
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .exclude_pattern("*.tmp");
    /// ```
    pub fn exclude_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.exclude_pattern = Some(pattern.into());
        self
    }

    /// Sets whether to preserve directory structure.
    ///
    /// # Parameters
    /// - `preserve`: If true, maintains directory structure; if false, flattens all files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    ///
    /// let builder = ZipTaskletBuilder::new()
    ///     .preserve_structure(false); // Flatten files
    /// ```
    pub fn preserve_structure(mut self, preserve: bool) -> Self {
        self.preserve_structure = preserve;
        self
    }

    /// Builds the ZipTasklet instance.
    ///
    /// # Returns
    /// - `Ok(ZipTasklet)`: Successfully created tasklet
    /// - `Err(BatchError)`: Error if required parameters are missing or invalid
    ///
    /// # Errors
    /// - Returns error if source_path or target_path are not set
    /// - Returns error if source path doesn't exist
    /// - Returns error if target directory cannot be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
    /// use std::fs;
    /// use std::env::temp_dir;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// // Create test data directory
    /// let temp_data_dir = temp_dir().join("test_data_builder");
    /// fs::create_dir_all(&temp_data_dir).unwrap();
    /// fs::write(temp_data_dir.join("test.txt"), "test content").unwrap();
    ///
    /// let archive_path = temp_dir().join("archive_builder.zip");
    ///
    /// let tasklet = ZipTaskletBuilder::new()
    ///     .source_path(&temp_data_dir)
    ///     .target_path(&archive_path)
    ///     .build()?;
    ///
    /// // Cleanup test files
    /// fs::remove_dir_all(&temp_data_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<ZipTasklet, BatchError> {
        let source_path = self
            .source_path
            .ok_or_else(|| BatchError::Configuration("Source path is required".to_string()))?;

        let target_path = self
            .target_path
            .ok_or_else(|| BatchError::Configuration("Target path is required".to_string()))?;

        let mut tasklet = ZipTasklet::new(source_path, target_path)?;
        tasklet.set_compression_level(self.compression_level);

        if let Some(pattern) = self.include_pattern {
            tasklet.set_include_pattern(pattern);
        }

        if let Some(pattern) = self.exclude_pattern {
            tasklet.set_exclude_pattern(pattern);
        }

        tasklet.set_preserve_structure(self.preserve_structure);

        Ok(tasklet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Creates a test directory structure for testing.
    fn create_test_structure(base_dir: &Path) -> Result<(), io::Error> {
        // Create directories
        fs::create_dir_all(base_dir.join("subdir1"))?;
        fs::create_dir_all(base_dir.join("subdir2"))?;

        // Create files
        fs::write(base_dir.join("file1.txt"), "Content of file1")?;
        fs::write(base_dir.join("file2.log"), "Log content")?;
        fs::write(base_dir.join("file3.tmp"), "Temporary content")?;
        fs::write(
            base_dir.join("subdir1").join("nested.txt"),
            "Nested content",
        )?;
        fs::write(base_dir.join("subdir2").join("data.log"), "Data log")?;

        Ok(())
    }

    #[test]
    fn test_zip_tasklet_creation() -> Result<(), BatchError> {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source");
        let target_path = temp_dir.path().join("archive.zip");

        fs::create_dir(&source_path).unwrap();
        fs::write(source_path.join("test.txt"), "test content").unwrap();

        let tasklet = ZipTasklet::new(&source_path, &target_path)?;
        assert_eq!(tasklet.source_path, source_path);
        assert_eq!(tasklet.target_path, target_path);
        assert_eq!(tasklet.compression_level, 6);

        Ok(())
    }

    #[test]
    fn test_zip_tasklet_builder() -> Result<(), BatchError> {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source");
        let target_path = temp_dir.path().join("archive.zip");

        fs::create_dir(&source_path).unwrap();
        fs::write(source_path.join("test.txt"), "test content").unwrap();

        let tasklet = ZipTaskletBuilder::new()
            .source_path(&source_path)
            .target_path(&target_path)
            .compression_level(9)
            .include_pattern("*.txt")
            .exclude_pattern("*.tmp")
            .preserve_structure(false)
            .build()?;

        assert_eq!(tasklet.compression_level, 9);
        assert_eq!(tasklet.include_pattern, Some("*.txt".to_string()));
        assert_eq!(tasklet.exclude_pattern, Some("*.tmp".to_string()));
        assert!(!tasklet.preserve_structure);

        Ok(())
    }

    #[test]
    fn test_zip_single_file() -> Result<(), BatchError> {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test.txt");
        let target_zip = temp_dir.path().join("archive.zip");

        fs::write(&source_file, "Hello, World!").unwrap();

        let tasklet = ZipTasklet::new(&source_file, &target_zip)?;
        let step_execution = StepExecution::new("test-step");

        let result = tasklet.execute(&step_execution)?;
        assert_eq!(result, RepeatStatus::Finished);
        assert!(target_zip.exists());

        Ok(())
    }

    #[test]
    fn test_zip_directory() -> Result<(), BatchError> {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_zip = temp_dir.path().join("archive.zip");

        fs::create_dir(&source_dir).unwrap();
        create_test_structure(&source_dir).unwrap();

        let tasklet = ZipTasklet::new(&source_dir, &target_zip)?;
        let step_execution = StepExecution::new("test-step");

        let result = tasklet.execute(&step_execution)?;
        assert_eq!(result, RepeatStatus::Finished);
        assert!(target_zip.exists());

        Ok(())
    }

    #[test]
    fn test_pattern_matching() {
        let tasklet = ZipTasklet {
            source_path: PathBuf::new(),
            target_path: PathBuf::new(),
            compression_level: 6,
            include_pattern: Some("*.txt".to_string()),
            exclude_pattern: Some("*.tmp".to_string()),
            preserve_structure: true,
        };

        assert!(tasklet.matches_pattern("file.txt", "*.txt"));
        assert!(!tasklet.matches_pattern("file.log", "*.txt"));
        assert!(tasklet.matches_pattern("path/to/file.txt", "**/*.txt"));
        assert!(!tasklet.matches_pattern("file.txt", "*.log"));

        assert!(tasklet.should_include_file(Path::new("test.txt")));
        assert!(!tasklet.should_include_file(Path::new("test.tmp")));
        assert!(!tasklet.should_include_file(Path::new("test.log")));
    }

    #[test]
    fn test_compression_levels() -> Result<(), BatchError> {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test.txt");
        let target_zip = temp_dir.path().join("archive.zip");

        fs::write(&source_file, "Hello, World!".repeat(1000)).unwrap();

        let mut tasklet = ZipTasklet::new(&source_file, &target_zip)?;
        tasklet.set_compression_level(0); // No compression
        assert_eq!(tasklet.compression_level, 0);

        tasklet.set_compression_level(15); // Should clamp to 9
        assert_eq!(tasklet.compression_level, 9);

        tasklet.set_compression_level(-5); // Should clamp to 0
        assert_eq!(tasklet.compression_level, 0);

        Ok(())
    }

    #[test]
    fn test_builder_validation() {
        let result = ZipTaskletBuilder::new().build();
        assert!(result.is_err());

        let result = ZipTaskletBuilder::new()
            .source_path("/nonexistent/path")
            .build();
        assert!(result.is_err());

        let result = ZipTaskletBuilder::new()
            .target_path("/some/path.zip")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_source() {
        let result = ZipTasklet::new("/nonexistent/path", "/tmp/test.zip");
        assert!(result.is_err());
    }
}
