//! # FTP Tasklet
//!
//! This module provides tasklets for FTP file transfer operations (put and get).
//! It's designed to be similar to Spring Batch's FTP capabilities for batch file transfers.
//!
//! ## Features
//!
//! - FTP PUT operations (upload files to FTP server)
//! - FTP GET operations (download files from FTP server)
//! - FTP PUT FOLDER operations (upload entire folder contents to FTP server)
//! - FTP GET FOLDER operations (download entire folder contents from FTP server)
//! - Support for both active and passive FTP modes
//! - Configurable connection parameters
//! - Proper error handling and logging
//! - Builder pattern for easy configuration
//!
//! ## Memory Efficiency Features
//!
//! **Streaming Downloads (Implemented):**
//! - Both `FtpGetTasklet` and `FtpGetFolderTasklet` use `retr()` streaming method
//!   to download files directly from FTP server to local storage without loading
//!   entire files into memory
//! - This approach is memory-efficient for files of any size, from small to very large
//! - Uses proper error type conversion between `std::io::Error` and `FtpError`
//!   through the `io_error_to_ftp_error` helper function
//!
//! **Performance Benefits:**
//! - Constant memory usage regardless of file size
//! - Improved performance for large file transfers
//! - Reduced risk of out-of-memory errors when processing large files
//! - Direct streaming from network to disk without intermediate buffering
//!
//! ## Examples
//!
//! ### FTP PUT Operation
//!
//! ```rust
//! use spring_batch_rs::core::step::{StepBuilder, StepExecution, Step};
//! use spring_batch_rs::tasklet::ftp::FtpPutTaskletBuilder;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let ftp_put_tasklet = FtpPutTaskletBuilder::new()
//!     .host("ftp.example.com")
//!     .port(21)
//!     .username("user")
//!     .password("password")
//!     .local_file("./local_file.txt")
//!     .remote_file("/remote/path/file.txt")
//!     .passive_mode(true)
//!     .build()?;
//!
//! let step = StepBuilder::new("ftp-upload")
//!     .tasklet(&ftp_put_tasklet)
//!     .build();
//!
//! let mut step_execution = StepExecution::new("ftp-upload");
//! step.execute(&mut step_execution)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### FTP GET Operation (Memory-Efficient Streaming)
//!
//! ```rust
//! use spring_batch_rs::tasklet::ftp::FtpGetTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! // This tasklet streams large files directly to disk without loading into memory
//! let ftp_get_tasklet = FtpGetTaskletBuilder::new()
//!     .host("ftp.example.com")
//!     .username("user")
//!     .password("password")
//!     .remote_file("/remote/path/large_file.zip")  // Works efficiently with any file size
//!     .local_file("./downloaded_large_file.zip")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### FTPS (Secure FTP) Operations
//!
//! ```rust
//! use spring_batch_rs::tasklet::ftp::{FtpPutTaskletBuilder, FtpGetTaskletBuilder};
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! // Secure upload using FTPS (FTP over TLS)
//! let secure_upload = FtpPutTaskletBuilder::new()
//!     .host("secure-ftp.example.com")
//!     .port(990)  // Common FTPS port
//!     .username("user")
//!     .password("password")
//!     .local_file("./sensitive_data.txt")
//!     .remote_file("/secure/path/data.txt")
//!     .secure(true)  // Enable FTPS
//!     .build()?;
//!
//! // Secure download using FTPS with streaming for memory efficiency
//! let secure_download = FtpGetTaskletBuilder::new()
//!     .host("secure-ftp.example.com")
//!     .port(990)
//!     .username("user")
//!     .password("password")
//!     .remote_file("/secure/path/confidential.zip")
//!     .local_file("./confidential.zip")
//!     .secure(true)  // Enable FTPS
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use crate::{
    core::step::{RepeatStatus, StepExecution, Tasklet},
    BatchError,
};
use log::info;
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};
use suppaftp::{FtpError, FtpStream, Mode};

#[cfg(feature = "ftp")]
use suppaftp::{NativeTlsConnector, NativeTlsFtpStream};

#[cfg(feature = "ftp")]
use suppaftp::native_tls::TlsConnector;

/// Helper function to convert std::io::Error to FtpError for use in suppaftp closures.
///
/// This allows us to use std::io operations within FTP streaming closures while
/// maintaining proper error type compatibility. The suppaftp library's `retr()` method
/// expects closures to return `Result<T, FtpError>`, but standard I/O operations return
/// `Result<T, std::io::Error>`. This function bridges that gap by wrapping the I/O error
/// in a `FtpError::ConnectionError` variant.
///
/// # Arguments
///
/// * `error` - The std::io::Error to convert
///
/// # Returns
///
/// An FtpError that can be used in suppaftp closure contexts
fn io_error_to_ftp_error(error: std::io::Error) -> FtpError {
    FtpError::ConnectionError(error)
}

/// Helper function to establish and configure an FTP connection.
///
/// This function handles the common setup logic shared by all FTP tasklets:
/// - Connecting to the FTP server
/// - Logging in with credentials
/// - Setting timeouts for read/write operations
/// - Configuring transfer mode (active/passive)
///
/// # Arguments
///
/// * `host` - FTP server hostname or IP address
/// * `port` - FTP server port
/// * `username` - FTP username
/// * `password` - FTP password
/// * `passive_mode` - Whether to use passive mode
/// * `timeout` - Connection timeout duration
///
/// # Returns
///
/// Returns a configured `FtpStream` ready for file operations.
///
/// # Errors
///
/// Returns `BatchError` if connection, login, or configuration fails.
fn setup_ftp_connection(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    passive_mode: bool,
    timeout: Duration,
) -> Result<FtpStream, BatchError> {
    // Connect to FTP server
    let mut ftp_stream = FtpStream::connect(format!("{}:{}", host, port)).map_err(|e| {
        BatchError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Failed to connect to FTP server: {}", e),
        ))
    })?;

    // Login
    ftp_stream
        .login(username, password)
        .map_err(|e| BatchError::Configuration(format!("FTP login failed: {}", e)))?;

    // Set timeout for control channel commands
    ftp_stream
        .get_ref()
        .set_read_timeout(Some(timeout))
        .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
    ftp_stream
        .get_ref()
        .set_write_timeout(Some(timeout))
        .map_err(|e| BatchError::Configuration(format!("Failed to set write timeout: {}", e)))?;

    // Set transfer mode
    let mode = if passive_mode {
        Mode::Passive
    } else {
        Mode::Active
    };
    ftp_stream.set_mode(mode);

    Ok(ftp_stream)
}

/// Helper function to establish and configure an FTPS connection.
///
/// This function handles the setup logic for secure FTP connections:
/// - Connecting to the FTP server
/// - Switching to secure mode (explicit FTPS)
/// - Logging in with credentials
/// - Setting timeouts for read/write operations
/// - Configuring transfer mode (active/passive)
///
/// # Arguments
///
/// * `host` - FTP server hostname or IP address
/// * `port` - FTP server port
/// * `username` - FTP username
/// * `password` - FTP password
/// * `passive_mode` - Whether to use passive mode
/// * `timeout` - Connection timeout duration
///
/// # Returns
///
/// Returns a configured `NativeTlsFtpStream` ready for secure file operations.
///
/// # Errors
///
/// Returns `BatchError` if connection, TLS setup, login, or configuration fails.
#[cfg(feature = "ftp")]
fn setup_ftps_connection(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    passive_mode: bool,
    timeout: Duration,
) -> Result<NativeTlsFtpStream, BatchError> {
    // Connect to FTP server
    let plain_stream = NativeTlsFtpStream::connect(format!("{}:{}", host, port)).map_err(|e| {
        BatchError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Failed to connect to FTP server: {}", e),
        ))
    })?;

    // Switch to secure mode using explicit FTPS
    let tls_connector = TlsConnector::new()
        .map_err(|e| BatchError::Configuration(format!("Failed to create TLS connector: {}", e)))?;
    let mut ftp_stream = plain_stream
        .into_secure(NativeTlsConnector::from(tls_connector), host)
        .map_err(|e| {
            BatchError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Failed to establish FTPS connection: {}", e),
            ))
        })?;

    // Login
    ftp_stream
        .login(username, password)
        .map_err(|e| BatchError::Configuration(format!("FTPS login failed: {}", e)))?;

    // Set timeout for control channel commands
    ftp_stream
        .get_ref()
        .set_read_timeout(Some(timeout))
        .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
    ftp_stream
        .get_ref()
        .set_write_timeout(Some(timeout))
        .map_err(|e| BatchError::Configuration(format!("Failed to set write timeout: {}", e)))?;

    // Set transfer mode
    let mode = if passive_mode {
        Mode::Passive
    } else {
        Mode::Active
    };
    ftp_stream.set_mode(mode);

    Ok(ftp_stream)
}

/// A tasklet for uploading files to an FTP server.
///
/// This tasklet provides functionality for uploading local files to an FTP server
/// as part of a batch processing step. Supports both plain FTP and secure FTPS
/// (FTP over TLS) connections.
#[derive(Debug)]
pub struct FtpPutTasklet {
    /// FTP server hostname or IP address
    host: String,
    /// FTP server port (default: 21)
    port: u16,
    /// FTP username
    username: String,
    /// FTP password
    password: String,
    /// Local file path to upload
    local_file: PathBuf,
    /// Remote file path on FTP server
    remote_file: String,
    /// Whether to use passive mode (default: true)
    passive_mode: bool,
    /// Connection timeout in seconds
    timeout: Duration,
    /// Whether to use FTPS (FTP over TLS) for secure communication (default: false)
    secure: bool,
}

impl FtpPutTasklet {
    /// Creates a new FtpPutTasklet with the specified parameters.
    pub fn new<P: AsRef<Path>>(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        local_file: P,
        remote_file: &str,
    ) -> Result<Self, BatchError> {
        let local_path = local_file.as_ref().to_path_buf();

        // Validate local file exists
        if !local_path.exists() {
            return Err(BatchError::Configuration(format!(
                "Local file does not exist: {}",
                local_path.display()
            )));
        }

        Ok(Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            local_file: local_path,
            remote_file: remote_file.to_string(),
            passive_mode: true,
            timeout: Duration::from_secs(30),
            secure: false,
        })
    }

    /// Sets the passive mode for FTP connection.
    pub fn set_passive_mode(&mut self, passive: bool) {
        self.passive_mode = passive;
    }

    /// Sets the connection timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

impl Tasklet for FtpPutTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let protocol = if self.secure { "FTPS" } else { "FTP" };
        info!(
            "Starting {} PUT: {} -> {}:{}{}",
            protocol,
            self.local_file.display(),
            self.host,
            self.port,
            self.remote_file
        );

        let file = File::open(&self.local_file).map_err(BatchError::Io)?;
        let mut reader = BufReader::new(file);

        if self.secure {
            #[cfg(feature = "ftp")]
            {
                // Connect using FTPS
                let mut ftp_stream = setup_ftps_connection(
                    &self.host,
                    self.port,
                    &self.username,
                    &self.password,
                    self.passive_mode,
                    self.timeout,
                )?;

                // Upload file
                ftp_stream
                    .put_file(&self.remote_file, &mut reader)
                    .map_err(|e| {
                        BatchError::Io(std::io::Error::other(format!("FTPS upload failed: {}", e)))
                    })?;

                // Disconnect
                let _ = ftp_stream.quit();
            }
            #[cfg(not(feature = "ftp"))]
            {
                return Err(BatchError::Configuration(
                    "FTPS support requires the 'ftp' feature to be enabled".to_string(),
                ));
            }
        } else {
            // Connect using plain FTP
            let mut ftp_stream = setup_ftp_connection(
                &self.host,
                self.port,
                &self.username,
                &self.password,
                self.passive_mode,
                self.timeout,
            )?;

            // Upload file
            ftp_stream
                .put_file(&self.remote_file, &mut reader)
                .map_err(|e| {
                    BatchError::Io(std::io::Error::other(format!("FTP upload failed: {}", e)))
                })?;

            // Disconnect
            let _ = ftp_stream.quit();
        }

        info!(
            "{} PUT completed successfully: {} uploaded to {}:{}{}",
            protocol,
            self.local_file.display(),
            self.host,
            self.port,
            self.remote_file
        );

        Ok(RepeatStatus::Finished)
    }
}

/// A tasklet for downloading files from an FTP server.
///
/// This tasklet provides functionality for downloading files from an FTP server
/// to local storage as part of a batch processing step. Supports both plain FTP
/// and secure FTPS (FTP over TLS) connections.
#[derive(Debug)]
pub struct FtpGetTasklet {
    /// FTP server hostname or IP address
    host: String,
    /// FTP server port (default: 21)
    port: u16,
    /// FTP username
    username: String,
    /// FTP password
    password: String,
    /// Remote file path on FTP server
    remote_file: String,
    /// Local file path to save downloaded file
    local_file: PathBuf,
    /// Whether to use passive mode (default: true)
    passive_mode: bool,
    /// Connection timeout in seconds
    timeout: Duration,
    /// Whether to use FTPS (FTP over TLS) for secure communication (default: false)
    secure: bool,
}

impl FtpGetTasklet {
    /// Creates a new FtpGetTasklet with the specified parameters.
    pub fn new<P: AsRef<Path>>(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        remote_file: &str,
        local_file: P,
    ) -> Result<Self, BatchError> {
        let local_path = local_file.as_ref().to_path_buf();

        // Ensure local directory exists
        if let Some(parent) = local_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(BatchError::Io)?;
            }
        }

        Ok(Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            remote_file: remote_file.to_string(),
            local_file: local_path,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            secure: false,
        })
    }

    /// Sets the passive mode for FTP connection.
    pub fn set_passive_mode(&mut self, passive: bool) {
        self.passive_mode = passive;
    }

    /// Sets the connection timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

impl Tasklet for FtpGetTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let protocol = if self.secure { "FTPS" } else { "FTP" };
        info!(
            "Starting {} GET: {}:{}{} -> {}",
            protocol,
            self.host,
            self.port,
            self.remote_file,
            self.local_file.display()
        );

        let local_file_path = self.local_file.clone();

        if self.secure {
            #[cfg(feature = "ftp")]
            {
                // Connect using FTPS
                let mut ftp_stream = setup_ftps_connection(
                    &self.host,
                    self.port,
                    &self.username,
                    &self.password,
                    self.passive_mode,
                    self.timeout,
                )?;

                // Stream download directly to file for improved memory efficiency
                ftp_stream
                    .retr(&self.remote_file, |stream| {
                        // Create local file for writing
                        let mut file =
                            File::create(&local_file_path).map_err(io_error_to_ftp_error)?;

                        // Copy data from FTP stream to local file using streaming
                        std::io::copy(stream, &mut file).map_err(io_error_to_ftp_error)?;

                        // Ensure data is flushed to disk
                        file.sync_all().map_err(io_error_to_ftp_error)?;

                        Ok(())
                    })
                    .map_err(|e| {
                        BatchError::Io(std::io::Error::other(format!(
                            "FTPS streaming download failed: {}",
                            e
                        )))
                    })?;

                // Disconnect
                let _ = ftp_stream.quit();
            }
            #[cfg(not(feature = "ftp"))]
            {
                return Err(BatchError::Configuration(
                    "FTPS support requires the 'ftp' feature to be enabled".to_string(),
                ));
            }
        } else {
            // Connect using plain FTP
            let mut ftp_stream = setup_ftp_connection(
                &self.host,
                self.port,
                &self.username,
                &self.password,
                self.passive_mode,
                self.timeout,
            )?;

            // Stream download directly to file for improved memory efficiency
            ftp_stream
                .retr(&self.remote_file, |stream| {
                    // Create local file for writing
                    let mut file = File::create(&local_file_path).map_err(io_error_to_ftp_error)?;

                    // Copy data from FTP stream to local file using streaming
                    std::io::copy(stream, &mut file).map_err(io_error_to_ftp_error)?;

                    // Ensure data is flushed to disk
                    file.sync_all().map_err(io_error_to_ftp_error)?;

                    Ok(())
                })
                .map_err(|e| {
                    BatchError::Io(std::io::Error::other(format!(
                        "FTP streaming download failed: {}",
                        e
                    )))
                })?;

            // Disconnect
            let _ = ftp_stream.quit();
        }

        info!(
            "{} GET completed successfully: {}:{}{} downloaded to {}",
            protocol,
            self.host,
            self.port,
            self.remote_file,
            self.local_file.display()
        );

        Ok(RepeatStatus::Finished)
    }
}

/// A tasklet for uploading entire folder contents to an FTP server.
///
/// This tasklet provides functionality for uploading all files from a local folder
/// to a remote folder on an FTP server as part of a batch processing step. Supports
/// both plain FTP and secure FTPS (FTP over TLS) connections.
#[derive(Debug)]
pub struct FtpPutFolderTasklet {
    /// FTP server hostname or IP address
    host: String,
    /// FTP server port (default: 21)
    port: u16,
    /// FTP username
    username: String,
    /// FTP password
    password: String,
    /// Local folder path to upload
    local_folder: PathBuf,
    /// Remote folder path on FTP server
    remote_folder: String,
    /// Whether to use passive mode (default: true)
    passive_mode: bool,
    /// Connection timeout in seconds
    timeout: Duration,
    /// Whether to create remote directories if they don't exist
    create_directories: bool,
    /// Whether to upload subdirectories recursively
    recursive: bool,
    /// Whether to use FTPS (FTP over TLS) for secure communication (default: false)
    secure: bool,
}

impl FtpPutFolderTasklet {
    /// Creates a new FtpPutFolderTasklet with the specified parameters.
    pub fn new<P: AsRef<Path>>(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        local_folder: P,
        remote_folder: &str,
    ) -> Result<Self, BatchError> {
        let local_path = local_folder.as_ref().to_path_buf();

        // Validate local folder exists
        if !local_path.exists() {
            return Err(BatchError::Configuration(format!(
                "Local folder does not exist: {}",
                local_path.display()
            )));
        }

        if !local_path.is_dir() {
            return Err(BatchError::Configuration(format!(
                "Local path is not a directory: {}",
                local_path.display()
            )));
        }

        Ok(Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            local_folder: local_path,
            remote_folder: remote_folder.to_string(),
            passive_mode: true,
            timeout: Duration::from_secs(30),
            create_directories: true,
            recursive: false,
            secure: false,
        })
    }

    /// Sets the passive mode for FTP connection.
    pub fn set_passive_mode(&mut self, passive: bool) {
        self.passive_mode = passive;
    }

    /// Sets the connection timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Sets whether to create remote directories if they don't exist.
    pub fn set_create_directories(&mut self, create: bool) {
        self.create_directories = create;
    }

    /// Sets whether to upload subdirectories recursively.
    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    /// Recursively uploads files from a directory.
    fn upload_directory(
        &self,
        ftp_stream: &mut FtpStream,
        local_dir: &Path,
        remote_dir: &str,
    ) -> Result<(), BatchError> {
        let entries = fs::read_dir(local_dir).map_err(BatchError::Io)?;

        for entry in entries {
            let entry = entry.map_err(BatchError::Io)?;
            let local_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            let remote_path = if remote_dir.is_empty() {
                file_name_str.to_string()
            } else {
                format!("{}/{}", remote_dir, file_name_str)
            };

            if local_path.is_file() {
                info!(
                    "Uploading file: {} -> {}",
                    local_path.display(),
                    remote_path
                );

                let file = File::open(&local_path).map_err(BatchError::Io)?;
                let mut reader = BufReader::new(file);

                ftp_stream
                    .put_file(&remote_path, &mut reader)
                    .map_err(|e| {
                        BatchError::Io(std::io::Error::other(format!(
                            "FTP upload failed for {}: {}",
                            local_path.display(),
                            e
                        )))
                    })?;
            } else if local_path.is_dir() && self.recursive {
                info!("Creating remote directory: {}", remote_path);

                if self.create_directories {
                    // Try to create directory, ignore error if it already exists
                    let _ = ftp_stream.mkdir(&remote_path);
                }

                // Recursively upload subdirectory
                self.upload_directory(ftp_stream, &local_path, &remote_path)?;
            }
        }

        Ok(())
    }

    /// Recursively uploads files from a directory using FTPS.
    #[cfg(feature = "ftp")]
    fn upload_directory_secure(
        &self,
        ftp_stream: &mut NativeTlsFtpStream,
        local_dir: &Path,
        remote_dir: &str,
    ) -> Result<(), BatchError> {
        let entries = fs::read_dir(local_dir).map_err(BatchError::Io)?;

        for entry in entries {
            let entry = entry.map_err(BatchError::Io)?;
            let local_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            let remote_path = if remote_dir.is_empty() {
                file_name_str.to_string()
            } else {
                format!("{}/{}", remote_dir, file_name_str)
            };

            if local_path.is_file() {
                info!(
                    "Uploading file (FTPS): {} -> {}",
                    local_path.display(),
                    remote_path
                );

                let file = File::open(&local_path).map_err(BatchError::Io)?;
                let mut reader = BufReader::new(file);

                ftp_stream
                    .put_file(&remote_path, &mut reader)
                    .map_err(|e| {
                        BatchError::Io(std::io::Error::other(format!(
                            "FTPS upload failed for {}: {}",
                            local_path.display(),
                            e
                        )))
                    })?;
            } else if local_path.is_dir() && self.recursive {
                info!("Creating remote directory (FTPS): {}", remote_path);

                if self.create_directories {
                    // Try to create directory, ignore error if it already exists
                    let _ = ftp_stream.mkdir(&remote_path);
                }

                // Recursively upload subdirectory
                self.upload_directory_secure(ftp_stream, &local_path, &remote_path)?;
            }
        }

        Ok(())
    }
}

impl Tasklet for FtpPutFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let protocol = if self.secure { "FTPS" } else { "FTP" };
        info!(
            "Starting {} PUT FOLDER: {} -> {}:{}{}",
            protocol,
            self.local_folder.display(),
            self.host,
            self.port,
            self.remote_folder
        );

        if self.secure {
            #[cfg(feature = "ftp")]
            {
                // Connect using FTPS
                let mut ftp_stream = setup_ftps_connection(
                    &self.host,
                    self.port,
                    &self.username,
                    &self.password,
                    self.passive_mode,
                    self.timeout,
                )?;

                // Create remote base directory if needed
                if self.create_directories && !self.remote_folder.is_empty() {
                    let _ = ftp_stream.mkdir(&self.remote_folder);
                }

                // Upload folder contents using FTPS
                self.upload_directory_secure(
                    &mut ftp_stream,
                    &self.local_folder,
                    &self.remote_folder,
                )?;

                // Disconnect
                let _ = ftp_stream.quit();
            }
            #[cfg(not(feature = "ftp"))]
            {
                return Err(BatchError::Configuration(
                    "FTPS support requires the 'ftp' feature to be enabled".to_string(),
                ));
            }
        } else {
            // Connect using plain FTP
            let mut ftp_stream = setup_ftp_connection(
                &self.host,
                self.port,
                &self.username,
                &self.password,
                self.passive_mode,
                self.timeout,
            )?;

            // Create remote base directory if needed
            if self.create_directories && !self.remote_folder.is_empty() {
                let _ = ftp_stream.mkdir(&self.remote_folder);
            }

            // Upload folder contents
            self.upload_directory(&mut ftp_stream, &self.local_folder, &self.remote_folder)?;

            // Disconnect
            let _ = ftp_stream.quit();
        }

        info!(
            "{} PUT FOLDER completed successfully: {} uploaded to {}:{}{}",
            protocol,
            self.local_folder.display(),
            self.host,
            self.port,
            self.remote_folder
        );

        Ok(RepeatStatus::Finished)
    }
}

/// A tasklet for downloading entire folder contents from an FTP server.
///
/// This tasklet provides functionality for downloading all files from a remote folder
/// on an FTP server to a local folder as part of a batch processing step. Supports
/// both plain FTP and secure FTPS (FTP over TLS) connections.
#[derive(Debug)]
pub struct FtpGetFolderTasklet {
    /// FTP server hostname or IP address
    host: String,
    /// FTP server port (default: 21)
    port: u16,
    /// FTP username
    username: String,
    /// FTP password
    password: String,
    /// Remote folder path on FTP server
    remote_folder: String,
    /// Local folder path to save downloaded files
    local_folder: PathBuf,
    /// Whether to use passive mode (default: true)
    passive_mode: bool,
    /// Connection timeout in seconds
    timeout: Duration,
    /// Whether to create local directories if they don't exist
    create_directories: bool,
    /// Whether to download subdirectories recursively
    recursive: bool,
    /// Whether to use FTPS (FTP over TLS) for secure communication (default: false)
    secure: bool,
}

impl FtpGetFolderTasklet {
    /// Creates a new FtpGetFolderTasklet with the specified parameters.
    pub fn new<P: AsRef<Path>>(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        remote_folder: &str,
        local_folder: P,
    ) -> Result<Self, BatchError> {
        let local_path = local_folder.as_ref().to_path_buf();

        Ok(Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            remote_folder: remote_folder.to_string(),
            local_folder: local_path,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            create_directories: true,
            recursive: false,
            secure: false,
        })
    }

    /// Sets the passive mode for FTP connection.
    pub fn set_passive_mode(&mut self, passive: bool) {
        self.passive_mode = passive;
    }

    /// Sets the connection timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Sets whether to create local directories if they don't exist.
    pub fn set_create_directories(&mut self, create: bool) {
        self.create_directories = create;
    }

    /// Sets whether to download subdirectories recursively.
    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    /// Recursively downloads files from a directory.
    fn download_directory(
        &self,
        ftp_stream: &mut FtpStream,
        remote_dir: &str,
        local_dir: &Path,
    ) -> Result<(), BatchError> {
        // List remote directory contents
        let files = ftp_stream.nlst(Some(remote_dir)).map_err(|e| {
            BatchError::Io(std::io::Error::other(format!(
                "Failed to list remote directory {}: {}",
                remote_dir, e
            )))
        })?;

        for file_path in files {
            let file_name = Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_path);

            let local_path = local_dir.join(file_name);
            let remote_full_path = if remote_dir.is_empty() {
                file_path.clone()
            } else {
                format!("{}/{}", remote_dir, file_name)
            };

            // Try to determine if it's a file or directory by attempting to stream download
            let download_result = {
                let local_path_clone = local_path.clone();
                ftp_stream.retr(&remote_full_path, |stream| {
                    // Create local file for writing
                    let mut file =
                        File::create(&local_path_clone).map_err(io_error_to_ftp_error)?;

                    // Copy data from FTP stream to local file using streaming
                    std::io::copy(stream, &mut file).map_err(io_error_to_ftp_error)?;

                    // Ensure data is flushed to disk
                    file.sync_all().map_err(io_error_to_ftp_error)?;

                    Ok(())
                })
            };

            match download_result {
                Ok(_) => {
                    // It's a file, successfully downloaded using streaming
                    info!(
                        "Streaming downloaded file: {} -> {}",
                        remote_full_path,
                        local_path.display()
                    );

                    if self.create_directories {
                        if let Some(parent) = local_path.parent() {
                            fs::create_dir_all(parent).map_err(BatchError::Io)?;
                        }
                    }
                }
                Err(_) if self.recursive => {
                    // Might be a directory, try to recurse
                    info!("Attempting to download directory: {}", remote_full_path);

                    if self.create_directories {
                        fs::create_dir_all(&local_path).map_err(BatchError::Io)?;
                    }

                    // Recursively download subdirectory
                    if let Err(e) =
                        self.download_directory(ftp_stream, &remote_full_path, &local_path)
                    {
                        // If recursion fails, it might not be a directory, just log and continue
                        info!(
                            "Failed to download as directory, skipping: {} ({})",
                            remote_full_path, e
                        );
                    }
                }
                Err(e) => {
                    info!(
                        "Skipping item that couldn't be downloaded: {} ({})",
                        remote_full_path, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Recursively downloads files from a directory using FTPS.
    #[cfg(feature = "ftp")]
    fn download_directory_secure(
        &self,
        ftp_stream: &mut NativeTlsFtpStream,
        remote_dir: &str,
        local_dir: &Path,
    ) -> Result<(), BatchError> {
        // List remote directory contents
        let files = ftp_stream.nlst(Some(remote_dir)).map_err(|e| {
            BatchError::Io(std::io::Error::other(format!(
                "Failed to list remote directory {}: {}",
                remote_dir, e
            )))
        })?;

        for file_path in files {
            let file_name = Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_path);

            let local_path = local_dir.join(file_name);
            let remote_full_path = if remote_dir.is_empty() {
                file_path.clone()
            } else {
                format!("{}/{}", remote_dir, file_name)
            };

            // Try to determine if it's a file or directory by attempting to stream download
            let download_result = {
                let local_path_clone = local_path.clone();
                ftp_stream.retr(&remote_full_path, |stream| {
                    // Create local file for writing
                    let mut file =
                        File::create(&local_path_clone).map_err(io_error_to_ftp_error)?;

                    // Copy data from FTP stream to local file using streaming
                    std::io::copy(stream, &mut file).map_err(io_error_to_ftp_error)?;

                    // Ensure data is flushed to disk
                    file.sync_all().map_err(io_error_to_ftp_error)?;

                    Ok(())
                })
            };

            match download_result {
                Ok(_) => {
                    // It's a file, successfully downloaded using streaming
                    info!(
                        "Streaming downloaded file (FTPS): {} -> {}",
                        remote_full_path,
                        local_path.display()
                    );

                    if self.create_directories {
                        if let Some(parent) = local_path.parent() {
                            fs::create_dir_all(parent).map_err(BatchError::Io)?;
                        }
                    }
                }
                Err(_) if self.recursive => {
                    // Might be a directory, try to recurse
                    info!(
                        "Attempting to download directory (FTPS): {}",
                        remote_full_path
                    );

                    if self.create_directories {
                        fs::create_dir_all(&local_path).map_err(BatchError::Io)?;
                    }

                    // Recursively download subdirectory
                    if let Err(e) =
                        self.download_directory_secure(ftp_stream, &remote_full_path, &local_path)
                    {
                        // If recursion fails, it might not be a directory, just log and continue
                        info!(
                            "Failed to download as directory, skipping: {} ({})",
                            remote_full_path, e
                        );
                    }
                }
                Err(e) => {
                    info!(
                        "Skipping item that couldn't be downloaded (FTPS): {} ({})",
                        remote_full_path, e
                    );
                }
            }
        }

        Ok(())
    }
}

impl Tasklet for FtpGetFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let protocol = if self.secure { "FTPS" } else { "FTP" };
        info!(
            "Starting {} GET FOLDER: {}:{}{} -> {}",
            protocol,
            self.host,
            self.port,
            self.remote_folder,
            self.local_folder.display()
        );

        // Create local base directory if needed
        if self.create_directories {
            fs::create_dir_all(&self.local_folder).map_err(BatchError::Io)?;
        }

        if self.secure {
            #[cfg(feature = "ftp")]
            {
                // Connect using FTPS
                let mut ftp_stream = setup_ftps_connection(
                    &self.host,
                    self.port,
                    &self.username,
                    &self.password,
                    self.passive_mode,
                    self.timeout,
                )?;

                // Download folder contents using FTPS
                self.download_directory_secure(
                    &mut ftp_stream,
                    &self.remote_folder,
                    &self.local_folder,
                )?;

                // Disconnect
                let _ = ftp_stream.quit();
            }
            #[cfg(not(feature = "ftp"))]
            {
                return Err(BatchError::Configuration(
                    "FTPS support requires the 'ftp' feature to be enabled".to_string(),
                ));
            }
        } else {
            // Connect using plain FTP
            let mut ftp_stream = setup_ftp_connection(
                &self.host,
                self.port,
                &self.username,
                &self.password,
                self.passive_mode,
                self.timeout,
            )?;

            // Download folder contents
            self.download_directory(&mut ftp_stream, &self.remote_folder, &self.local_folder)?;

            // Disconnect
            let _ = ftp_stream.quit();
        }

        info!(
            "{} GET FOLDER completed successfully: {}:{}{} downloaded to {}",
            protocol,
            self.host,
            self.port,
            self.remote_folder,
            self.local_folder.display()
        );

        Ok(RepeatStatus::Finished)
    }
}

/// Builder for creating FtpPutTasklet instances with a fluent interface.
pub struct FtpPutTaskletBuilder {
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    local_file: Option<PathBuf>,
    remote_file: Option<String>,
    passive_mode: bool,
    timeout: Duration,
    secure: bool,
}

impl Default for FtpPutTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FtpPutTaskletBuilder {
    /// Creates a new FtpPutTaskletBuilder with default settings.
    pub fn new() -> Self {
        Self {
            host: None,
            port: 21,
            username: None,
            password: None,
            local_file: None,
            remote_file: None,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            secure: false,
        }
    }

    /// Sets the FTP server hostname or IP address.
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the FTP server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the FTP username.
    pub fn username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the FTP password.
    pub fn password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the local file path to upload.
    pub fn local_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the remote file path on the FTP server.
    pub fn remote_file<S: Into<String>>(mut self, path: S) -> Self {
        self.remote_file = Some(path.into());
        self
    }

    /// Sets whether to use passive mode.
    pub fn passive_mode(mut self, passive: bool) -> Self {
        self.passive_mode = passive;
        self
    }

    /// Sets the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to use FTPS (FTP over TLS) for secure communication.
    ///
    /// When enabled, the connection will use explicit FTPS (FTP over TLS) for secure
    /// file transfers. This is recommended when handling sensitive data.
    ///
    /// # Arguments
    ///
    /// * `secure` - true to enable FTPS, false for plain FTP (default: false)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builds the FtpPutTasklet instance.
    pub fn build(self) -> Result<FtpPutTasklet, BatchError> {
        let host = self
            .host
            .ok_or_else(|| BatchError::Configuration("FTP host is required".to_string()))?;
        let username = self
            .username
            .ok_or_else(|| BatchError::Configuration("FTP username is required".to_string()))?;
        let password = self
            .password
            .ok_or_else(|| BatchError::Configuration("FTP password is required".to_string()))?;
        let local_file = self
            .local_file
            .ok_or_else(|| BatchError::Configuration("Local file path is required".to_string()))?;
        let remote_file = self
            .remote_file
            .ok_or_else(|| BatchError::Configuration("Remote file path is required".to_string()))?;

        let mut tasklet = FtpPutTasklet::new(
            &host,
            self.port,
            &username,
            &password,
            &local_file,
            &remote_file,
        )?;

        tasklet.set_passive_mode(self.passive_mode);
        tasklet.set_timeout(self.timeout);
        tasklet.secure = self.secure;

        Ok(tasklet)
    }
}

/// Builder for creating FtpGetTasklet instances with a fluent interface.
pub struct FtpGetTaskletBuilder {
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    remote_file: Option<String>,
    local_file: Option<PathBuf>,
    passive_mode: bool,
    timeout: Duration,
    secure: bool,
}

impl Default for FtpGetTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FtpGetTaskletBuilder {
    /// Creates a new FtpGetTaskletBuilder with default settings.
    pub fn new() -> Self {
        Self {
            host: None,
            port: 21,
            username: None,
            password: None,
            remote_file: None,
            local_file: None,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            secure: false,
        }
    }

    /// Sets the FTP server hostname or IP address.
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the FTP server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the FTP username.
    pub fn username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the FTP password.
    pub fn password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the remote file path on the FTP server.
    pub fn remote_file<S: Into<String>>(mut self, path: S) -> Self {
        self.remote_file = Some(path.into());
        self
    }

    /// Sets the local file path to save the downloaded file.
    pub fn local_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets whether to use passive mode.
    pub fn passive_mode(mut self, passive: bool) -> Self {
        self.passive_mode = passive;
        self
    }

    /// Sets the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to use FTPS (FTP over TLS) for secure communication.
    ///
    /// When enabled, the connection will use explicit FTPS (FTP over TLS) for secure
    /// file transfers. This is recommended when handling sensitive data.
    ///
    /// # Arguments
    ///
    /// * `secure` - true to enable FTPS, false for plain FTP (default: false)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builds the FtpGetTasklet instance.
    pub fn build(self) -> Result<FtpGetTasklet, BatchError> {
        let host = self
            .host
            .ok_or_else(|| BatchError::Configuration("FTP host is required".to_string()))?;
        let username = self
            .username
            .ok_or_else(|| BatchError::Configuration("FTP username is required".to_string()))?;
        let password = self
            .password
            .ok_or_else(|| BatchError::Configuration("FTP password is required".to_string()))?;
        let remote_file = self
            .remote_file
            .ok_or_else(|| BatchError::Configuration("Remote file path is required".to_string()))?;
        let local_file = self
            .local_file
            .ok_or_else(|| BatchError::Configuration("Local file path is required".to_string()))?;

        let mut tasklet = FtpGetTasklet::new(
            &host,
            self.port,
            &username,
            &password,
            &remote_file,
            &local_file,
        )?;

        tasklet.set_passive_mode(self.passive_mode);
        tasklet.set_timeout(self.timeout);
        tasklet.secure = self.secure;

        Ok(tasklet)
    }
}

/// Builder for creating FtpPutFolderTasklet instances with a fluent interface.
pub struct FtpPutFolderTaskletBuilder {
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    local_folder: Option<PathBuf>,
    remote_folder: Option<String>,
    passive_mode: bool,
    timeout: Duration,
    create_directories: bool,
    recursive: bool,
    secure: bool,
}

impl Default for FtpPutFolderTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FtpPutFolderTaskletBuilder {
    /// Creates a new FtpPutFolderTaskletBuilder with default settings.
    pub fn new() -> Self {
        Self {
            host: None,
            port: 21,
            username: None,
            password: None,
            local_folder: None,
            remote_folder: None,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            create_directories: true,
            recursive: false,
            secure: false,
        }
    }

    /// Sets the FTP server hostname or IP address.
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the FTP server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the FTP username.
    pub fn username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the FTP password.
    pub fn password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the local folder path to upload.
    pub fn local_folder<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_folder = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the remote folder path on the FTP server.
    pub fn remote_folder<S: Into<String>>(mut self, path: S) -> Self {
        self.remote_folder = Some(path.into());
        self
    }

    /// Sets whether to use passive mode.
    pub fn passive_mode(mut self, passive: bool) -> Self {
        self.passive_mode = passive;
        self
    }

    /// Sets the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to create remote directories if they don't exist.
    pub fn create_directories(mut self, create: bool) -> Self {
        self.create_directories = create;
        self
    }

    /// Sets whether to upload subdirectories recursively.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Sets whether to use FTPS (FTP over TLS) for secure communication.
    ///
    /// When enabled, the connection will use explicit FTPS (FTP over TLS) for secure
    /// file transfers. This is recommended when handling sensitive data.
    ///
    /// # Arguments
    ///
    /// * `secure` - true to enable FTPS, false for plain FTP (default: false)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builds the FtpPutFolderTasklet instance.
    pub fn build(self) -> Result<FtpPutFolderTasklet, BatchError> {
        let host = self
            .host
            .ok_or_else(|| BatchError::Configuration("FTP host is required".to_string()))?;
        let username = self
            .username
            .ok_or_else(|| BatchError::Configuration("FTP username is required".to_string()))?;
        let password = self
            .password
            .ok_or_else(|| BatchError::Configuration("FTP password is required".to_string()))?;
        let local_folder = self.local_folder.ok_or_else(|| {
            BatchError::Configuration("Local folder path is required".to_string())
        })?;
        let remote_folder = self.remote_folder.ok_or_else(|| {
            BatchError::Configuration("Remote folder path is required".to_string())
        })?;

        let mut tasklet = FtpPutFolderTasklet::new(
            &host,
            self.port,
            &username,
            &password,
            &local_folder,
            &remote_folder,
        )?;

        tasklet.set_passive_mode(self.passive_mode);
        tasklet.set_timeout(self.timeout);
        tasklet.set_create_directories(self.create_directories);
        tasklet.set_recursive(self.recursive);
        tasklet.secure = self.secure;

        Ok(tasklet)
    }
}

/// Builder for creating FtpGetFolderTasklet instances with a fluent interface.
pub struct FtpGetFolderTaskletBuilder {
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    remote_folder: Option<String>,
    local_folder: Option<PathBuf>,
    passive_mode: bool,
    timeout: Duration,
    create_directories: bool,
    recursive: bool,
    secure: bool,
}

impl Default for FtpGetFolderTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FtpGetFolderTaskletBuilder {
    /// Creates a new FtpGetFolderTaskletBuilder with default settings.
    pub fn new() -> Self {
        Self {
            host: None,
            port: 21,
            username: None,
            password: None,
            remote_folder: None,
            local_folder: None,
            passive_mode: true,
            timeout: Duration::from_secs(30),
            create_directories: true,
            recursive: false,
            secure: false,
        }
    }

    /// Sets the FTP server hostname or IP address.
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the FTP server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the FTP username.
    pub fn username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the FTP password.
    pub fn password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the remote folder path on the FTP server.
    pub fn remote_folder<S: Into<String>>(mut self, path: S) -> Self {
        self.remote_folder = Some(path.into());
        self
    }

    /// Sets the local folder path to save the downloaded files.
    pub fn local_folder<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_folder = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets whether to use passive mode.
    pub fn passive_mode(mut self, passive: bool) -> Self {
        self.passive_mode = passive;
        self
    }

    /// Sets the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to create local directories if they don't exist.
    pub fn create_directories(mut self, create: bool) -> Self {
        self.create_directories = create;
        self
    }

    /// Sets whether to download subdirectories recursively.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Sets whether to use FTPS (FTP over TLS) for secure communication.
    ///
    /// When enabled, the connection will use explicit FTPS (FTP over TLS) for secure
    /// file transfers. This is recommended when handling sensitive data.
    ///
    /// # Arguments
    ///
    /// * `secure` - true to enable FTPS, false for plain FTP (default: false)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builds the FtpGetFolderTasklet instance.
    pub fn build(self) -> Result<FtpGetFolderTasklet, BatchError> {
        let host = self
            .host
            .ok_or_else(|| BatchError::Configuration("FTP host is required".to_string()))?;
        let username = self
            .username
            .ok_or_else(|| BatchError::Configuration("FTP username is required".to_string()))?;
        let password = self
            .password
            .ok_or_else(|| BatchError::Configuration("FTP password is required".to_string()))?;
        let remote_folder = self.remote_folder.ok_or_else(|| {
            BatchError::Configuration("Remote folder path is required".to_string())
        })?;
        let local_folder = self.local_folder.ok_or_else(|| {
            BatchError::Configuration("Local folder path is required".to_string())
        })?;

        let mut tasklet = FtpGetFolderTasklet::new(
            &host,
            self.port,
            &username,
            &password,
            &remote_folder,
            &local_folder,
        )?;

        tasklet.set_passive_mode(self.passive_mode);
        tasklet.set_timeout(self.timeout);
        tasklet.set_create_directories(self.create_directories);
        tasklet.set_recursive(self.recursive);
        tasklet.secure = self.secure;

        Ok(tasklet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::step::StepExecution;
    use mockall::{mock, predicate::*};
    use std::env::temp_dir;
    use std::fs;

    // Mock trait for FTP operations to enable proper unit testing
    #[cfg(test)]
    pub trait FtpOperations {
        fn connect(&self, host: &str, port: u16) -> Result<(), BatchError>;
        fn login(&self, username: &str, password: &str) -> Result<(), BatchError>;
        fn set_mode(&self, passive: bool) -> Result<(), BatchError>;
        fn put_file(&self, remote_path: &str, content: &[u8]) -> Result<(), BatchError>;
        fn get_file(&self, remote_path: &str) -> Result<Vec<u8>, BatchError>;
        fn mkdir(&self, path: &str) -> Result<(), BatchError>;
        fn list_files(&self, path: &str) -> Result<Vec<String>, BatchError>;
        fn quit(&self) -> Result<(), BatchError>;
    }

    // Mock implementation using mockall
    mock! {
        pub FtpClient {}

        impl FtpOperations for FtpClient {
            fn connect(&self, host: &str, port: u16) -> Result<(), BatchError>;
            fn login(&self, username: &str, password: &str) -> Result<(), BatchError>;
            fn set_mode(&self, passive: bool) -> Result<(), BatchError>;
            fn put_file(&self, remote_path: &str, content: &[u8]) -> Result<(), BatchError>;
            fn get_file(&self, remote_path: &str) -> Result<Vec<u8>, BatchError>;
            fn mkdir(&self, path: &str) -> Result<(), BatchError>;
            fn list_files(&self, path: &str) -> Result<Vec<String>, BatchError>;
            fn quit(&self) -> Result<(), BatchError>;
        }
    }

    // Helper function to create a mock FTP client with common expectations
    fn create_successful_mock() -> MockFtpClient {
        let mut mock = MockFtpClient::new();
        mock.expect_connect().returning(|_, _| Ok(()));
        mock.expect_login().returning(|_, _| Ok(()));
        mock.expect_set_mode().returning(|_| Ok(()));
        mock.expect_quit().returning(|| Ok(()));
        mock
    }

    #[test]
    fn test_mock_ftp_put_success() {
        let mut mock = create_successful_mock();
        mock.expect_put_file()
            .with(eq("/remote/test.txt"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        // Simulate successful FTP PUT operation
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());
        assert!(mock.put_file("/remote/test.txt", b"test content").is_ok());
        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_get_success() {
        let mut mock = create_successful_mock();
        let expected_data = b"downloaded content".to_vec();
        mock.expect_get_file()
            .with(eq("/remote/test.txt"))
            .times(1)
            .returning(move |_| Ok(expected_data.clone()));

        // Simulate successful FTP GET operation
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());
        let result = mock.get_file("/remote/test.txt").unwrap();
        assert_eq!(result, b"downloaded content");
        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_connection_failure() {
        let mut mock = MockFtpClient::new();
        mock.expect_connect()
            .with(eq("invalid.host"), eq(21))
            .times(1)
            .returning(|_, _| {
                Err(BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Connection refused",
                )))
            });

        // Test connection failure
        let result = mock.connect("invalid.host", 21);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Connection refused"));
    }

    #[test]
    fn test_mock_ftp_login_failure() {
        let mut mock = MockFtpClient::new();
        mock.expect_connect().returning(|_, _| Ok(()));
        mock.expect_login()
            .with(eq("invalid_user"), eq("wrong_pass"))
            .times(1)
            .returning(|_, _| Err(BatchError::Configuration("Login failed".to_string())));

        // Test login failure
        assert!(mock.connect("localhost", 21).is_ok());
        let result = mock.login("invalid_user", "wrong_pass");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Login failed"));
    }

    #[test]
    fn test_mock_ftp_upload_failure() {
        let mut mock = create_successful_mock();
        mock.expect_put_file()
            .with(eq("/protected/file.txt"), always())
            .times(1)
            .returning(|_, _| {
                Err(BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Permission denied",
                )))
            });

        // Test upload failure
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());
        let result = mock.put_file("/protected/file.txt", b"content");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Permission denied"));
    }

    #[test]
    fn test_mock_ftp_download_failure() {
        let mut mock = create_successful_mock();
        mock.expect_get_file()
            .with(eq("/nonexistent/file.txt"))
            .times(1)
            .returning(|_| {
                Err(BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                )))
            });

        // Test download failure
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());
        let result = mock.get_file("/nonexistent/file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_mock_ftp_folder_operations() {
        let mut mock = create_successful_mock();

        // Setup expectations for folder operations
        mock.expect_mkdir()
            .with(eq("/remote/new_folder"))
            .times(1)
            .returning(|_| Ok(()));

        mock.expect_list_files()
            .with(eq("/remote/folder"))
            .times(1)
            .returning(|_| {
                Ok(vec![
                    "file1.txt".to_string(),
                    "file2.txt".to_string(),
                    "subfolder".to_string(),
                ])
            });

        mock.expect_put_file().times(2).returning(|_, _| Ok(()));

        // Test folder operations
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());
        assert!(mock.mkdir("/remote/new_folder").is_ok());

        let files = mock.list_files("/remote/folder").unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));

        // Upload multiple files
        assert!(mock.put_file("/remote/file1.txt", b"content1").is_ok());
        assert!(mock.put_file("/remote/file2.txt", b"content2").is_ok());
        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_passive_active_mode() {
        let mut mock = MockFtpClient::new();
        mock.expect_connect().returning(|_, _| Ok(()));
        mock.expect_login().returning(|_, _| Ok(()));
        mock.expect_set_mode()
            .with(eq(true)) // passive mode
            .times(1)
            .returning(|_| Ok(()));
        mock.expect_set_mode()
            .with(eq(false)) // active mode
            .times(1)
            .returning(|_| Ok(()));
        mock.expect_quit().returning(|| Ok(()));

        // Test passive and active mode switching
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok()); // Set to passive
        assert!(mock.set_mode(false).is_ok()); // Set to active
        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_multiple_file_transfers() {
        let mut mock = create_successful_mock();

        // Expect multiple file uploads
        mock.expect_put_file()
            .with(eq("/remote/file1.txt"), always())
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_put_file()
            .with(eq("/remote/file2.txt"), always())
            .times(1)
            .returning(|_, _| Ok(()));
        mock.expect_put_file()
            .with(eq("/remote/file3.txt"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        // Test multiple file transfers
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());

        // Upload multiple files
        assert!(mock.put_file("/remote/file1.txt", b"content1").is_ok());
        assert!(mock.put_file("/remote/file2.txt", b"content2").is_ok());
        assert!(mock.put_file("/remote/file3.txt", b"content3").is_ok());

        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_binary_file_transfer() {
        let mut mock = create_successful_mock();

        // Create some binary data
        let binary_data = vec![0u8, 1, 2, 3, 255, 128, 64];
        let binary_data_clone = binary_data.clone();

        mock.expect_put_file()
            .with(eq("/remote/binary.dat"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        mock.expect_get_file()
            .with(eq("/remote/binary.dat"))
            .times(1)
            .returning(move |_| Ok(binary_data_clone.clone()));

        // Test binary file transfer
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());

        // Upload binary file
        assert!(mock.put_file("/remote/binary.dat", &binary_data).is_ok());

        // Download binary file
        let downloaded = mock.get_file("/remote/binary.dat").unwrap();
        assert_eq!(downloaded, binary_data);

        assert!(mock.quit().is_ok());
    }

    #[test]
    fn test_mock_ftp_error_recovery() {
        let mut mock = MockFtpClient::new();
        mock.expect_connect().returning(|_, _| Ok(()));
        mock.expect_login().returning(|_, _| Ok(()));
        mock.expect_set_mode().returning(|_| Ok(()));

        // First attempt fails, second succeeds
        mock.expect_put_file()
            .with(eq("/remote/retry.txt"), always())
            .times(1)
            .returning(|_, _| {
                Err(BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout",
                )))
            });

        mock.expect_put_file()
            .with(eq("/remote/retry.txt"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        mock.expect_quit().returning(|| Ok(()));

        // Test error recovery scenario
        assert!(mock.connect("localhost", 21).is_ok());
        assert!(mock.login("user", "pass").is_ok());
        assert!(mock.set_mode(true).is_ok());

        // First attempt fails
        let result1 = mock.put_file("/remote/retry.txt", b"content");
        assert!(result1.is_err());
        assert!(result1.unwrap_err().to_string().contains("Timeout"));

        // Second attempt succeeds
        let result2 = mock.put_file("/remote/retry.txt", b"content");
        assert!(result2.is_ok());

        assert!(mock.quit().is_ok());
    }

    // Original tests (keeping the existing ones for integration testing)
    #[test]
    fn test_ftp_put_tasklet_creation() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("test_upload.txt");
        fs::write(&test_file, "test content").unwrap();

        let tasklet = FtpPutTasklet::new(
            "localhost",
            21,
            "testuser",
            "testpass",
            &test_file,
            "/remote/test.txt",
        )?;

        assert_eq!(tasklet.host, "localhost");
        assert_eq!(tasklet.port, 21);
        assert_eq!(tasklet.username, "testuser");
        assert_eq!(tasklet.remote_file, "/remote/test.txt");
        assert!(tasklet.passive_mode);

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_tasklet_creation() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_file = temp_dir.join("downloaded.txt");

        let tasklet = FtpGetTasklet::new(
            "localhost",
            21,
            "testuser",
            "testpass",
            "/remote/test.txt",
            &local_file,
        )?;

        assert_eq!(tasklet.host, "localhost");
        assert_eq!(tasklet.port, 21);
        assert_eq!(tasklet.username, "testuser");
        assert_eq!(tasklet.remote_file, "/remote/test.txt");
        assert!(tasklet.passive_mode);

        Ok(())
    }

    #[test]
    fn test_ftp_put_builder() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("test_builder.txt");
        fs::write(&test_file, "test content").unwrap();

        let tasklet = FtpPutTaskletBuilder::new()
            .host("ftp.example.com")
            .port(2121)
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/upload/file.txt")
            .passive_mode(false)
            .timeout(Duration::from_secs(60))
            .build()?;

        assert_eq!(tasklet.host, "ftp.example.com");
        assert_eq!(tasklet.port, 2121);
        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(60));

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_builder() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_file = temp_dir.join("download_builder.txt");

        let tasklet = FtpGetTaskletBuilder::new()
            .host("ftp.example.com")
            .port(2121)
            .username("user")
            .password("pass")
            .remote_file("/download/file.txt")
            .local_file(&local_file)
            .passive_mode(false)
            .timeout(Duration::from_secs(60))
            .build()?;

        assert_eq!(tasklet.host, "ftp.example.com");
        assert_eq!(tasklet.port, 2121);
        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(60));

        Ok(())
    }

    #[test]
    fn test_builder_validation() {
        // Test missing host
        let result = FtpPutTaskletBuilder::new()
            .username("user")
            .password("pass")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FTP host is required"));

        // Test missing username
        let result = FtpGetTaskletBuilder::new()
            .host("localhost")
            .password("pass")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FTP username is required"));

        // Test missing password
        let result = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FTP password is required"));

        // Test missing local file for PUT
        let result = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .remote_file("/remote/file.txt")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local file path is required"));

        // Test missing remote file for GET
        let result = FtpGetTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file("/local/file.txt")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Remote file path is required"));
    }

    #[test]
    fn test_nonexistent_local_file() {
        let result = FtpPutTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            "/nonexistent/file.txt",
            "/remote/file.txt",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local file does not exist"));
    }

    #[test]
    fn test_ftp_put_tasklet_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("config_test.txt");
        fs::write(&test_file, "test content").unwrap();

        let mut tasklet = FtpPutTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            &test_file,
            "/remote/file.txt",
        )?;

        // Test default values
        assert!(tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(30));

        // Test configuration methods
        tasklet.set_passive_mode(false);
        tasklet.set_timeout(Duration::from_secs(60));

        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(60));

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_tasklet_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_file = temp_dir.join("config_test.txt");

        let mut tasklet = FtpGetTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            "/remote/file.txt",
            &local_file,
        )?;

        // Test default values
        assert!(tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(30));

        // Test configuration methods
        tasklet.set_passive_mode(false);
        tasklet.set_timeout(Duration::from_secs(120));

        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(120));

        Ok(())
    }

    #[test]
    fn test_ftp_put_tasklet_execution_with_connection_error() {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("connection_error_test.txt");
        fs::write(&test_file, "test content").unwrap();

        let tasklet = FtpPutTasklet::new(
            "nonexistent.host.invalid",
            21,
            "user",
            "pass",
            &test_file,
            "/remote/file.txt",
        )
        .unwrap();

        let step_execution = StepExecution::new("test-step");
        let result = tasklet.execute(&step_execution);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, BatchError::Io(_)));
        assert!(error
            .to_string()
            .contains("Failed to connect to FTP server"));

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_ftp_get_tasklet_execution_with_connection_error() {
        let temp_dir = temp_dir();
        let local_file = temp_dir.join("connection_error_test.txt");

        let tasklet = FtpGetTasklet::new(
            "nonexistent.host.invalid",
            21,
            "user",
            "pass",
            "/remote/file.txt",
            &local_file,
        )
        .unwrap();

        let step_execution = StepExecution::new("test-step");
        let result = tasklet.execute(&step_execution);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, BatchError::Io(_)));
        assert!(error
            .to_string()
            .contains("Failed to connect to FTP server"));
    }

    #[test]
    fn test_setup_ftp_connection_parameters() {
        // Test that setup_ftp_connection function exists and has correct signature
        // This is a compile-time test to ensure the function signature is correct
        let _: fn(&str, u16, &str, &str, bool, Duration) -> Result<FtpStream, BatchError> =
            setup_ftp_connection;
    }

    #[test]
    fn test_ftp_put_folder_tasklet_creation() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_folder = temp_dir.join("test_upload_folder");
        fs::create_dir_all(&test_folder).unwrap();
        fs::write(test_folder.join("file1.txt"), "content1").unwrap();
        fs::write(test_folder.join("file2.txt"), "content2").unwrap();

        let tasklet = FtpPutFolderTasklet::new(
            "localhost",
            21,
            "testuser",
            "testpass",
            &test_folder,
            "/remote/folder",
        )?;

        assert_eq!(tasklet.host, "localhost");
        assert_eq!(tasklet.port, 21);
        assert_eq!(tasklet.username, "testuser");
        assert_eq!(tasklet.remote_folder, "/remote/folder");
        assert!(tasklet.passive_mode);
        assert!(tasklet.create_directories);
        assert!(!tasklet.recursive);

        fs::remove_dir_all(&test_folder).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_folder_tasklet_creation() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_folder = temp_dir.join("download_folder");

        let tasklet = FtpGetFolderTasklet::new(
            "localhost",
            21,
            "testuser",
            "testpass",
            "/remote/folder",
            &local_folder,
        )?;

        assert_eq!(tasklet.host, "localhost");
        assert_eq!(tasklet.port, 21);
        assert_eq!(tasklet.username, "testuser");
        assert_eq!(tasklet.remote_folder, "/remote/folder");
        assert!(tasklet.passive_mode);
        assert!(tasklet.create_directories);
        assert!(!tasklet.recursive);

        Ok(())
    }

    #[test]
    fn test_ftp_put_folder_builder() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_folder = temp_dir.join("test_builder_folder");
        fs::create_dir_all(&test_folder).unwrap();
        fs::write(test_folder.join("file.txt"), "content").unwrap();

        let tasklet = FtpPutFolderTaskletBuilder::new()
            .host("ftp.example.com")
            .port(2121)
            .username("user")
            .password("pass")
            .local_folder(&test_folder)
            .remote_folder("/upload/folder")
            .passive_mode(false)
            .timeout(Duration::from_secs(60))
            .create_directories(false)
            .recursive(true)
            .build()?;

        assert_eq!(tasklet.host, "ftp.example.com");
        assert_eq!(tasklet.port, 2121);
        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(60));
        assert!(!tasklet.create_directories);
        assert!(tasklet.recursive);

        fs::remove_dir_all(&test_folder).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_folder_builder() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_folder = temp_dir.join("download_builder_folder");

        let tasklet = FtpGetFolderTaskletBuilder::new()
            .host("ftp.example.com")
            .port(2121)
            .username("user")
            .password("pass")
            .remote_folder("/download/folder")
            .local_folder(&local_folder)
            .passive_mode(false)
            .timeout(Duration::from_secs(60))
            .create_directories(false)
            .recursive(true)
            .build()?;

        assert_eq!(tasklet.host, "ftp.example.com");
        assert_eq!(tasklet.port, 2121);
        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(60));
        assert!(!tasklet.create_directories);
        assert!(tasklet.recursive);

        Ok(())
    }

    #[test]
    fn test_folder_builder_validation() {
        // Test missing host for folder upload
        let result = FtpPutFolderTaskletBuilder::new()
            .username("user")
            .password("pass")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FTP host is required"));

        // Test missing username for folder download
        let result = FtpGetFolderTaskletBuilder::new()
            .host("localhost")
            .password("pass")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("FTP username is required"));

        // Test missing local folder for PUT
        let result = FtpPutFolderTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .remote_folder("/remote/folder")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local folder path is required"));

        // Test missing remote folder for GET
        let result = FtpGetFolderTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_folder("/local/folder")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Remote folder path is required"));
    }

    #[test]
    fn test_nonexistent_local_folder() {
        let result = FtpPutFolderTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            "/nonexistent/folder",
            "/remote/folder",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local folder does not exist"));
    }

    #[test]
    fn test_local_file_not_directory() {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("not_a_directory.txt");
        fs::write(&test_file, "content").unwrap();

        let result = FtpPutFolderTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            &test_file,
            "/remote/folder",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local path is not a directory"));

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_ftp_put_folder_tasklet_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_folder = temp_dir.join("config_folder_test");
        fs::create_dir_all(&test_folder).unwrap();
        fs::write(test_folder.join("file.txt"), "content").unwrap();

        let mut tasklet = FtpPutFolderTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            &test_folder,
            "/remote/folder",
        )?;

        // Test default values
        assert!(tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(30));
        assert!(tasklet.create_directories);
        assert!(!tasklet.recursive);

        // Test configuration methods
        tasklet.set_passive_mode(false);
        tasklet.set_timeout(Duration::from_secs(90));
        tasklet.set_create_directories(false);
        tasklet.set_recursive(true);

        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(90));
        assert!(!tasklet.create_directories);
        assert!(tasklet.recursive);

        fs::remove_dir_all(&test_folder).ok();
        Ok(())
    }

    #[test]
    fn test_ftp_get_folder_tasklet_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let local_folder = temp_dir.join("config_folder_test");

        let mut tasklet = FtpGetFolderTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            "/remote/folder",
            &local_folder,
        )?;

        // Test default values
        assert!(tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(30));
        assert!(tasklet.create_directories);
        assert!(!tasklet.recursive);

        // Test configuration methods
        tasklet.set_passive_mode(false);
        tasklet.set_timeout(Duration::from_secs(180));
        tasklet.set_create_directories(false);
        tasklet.set_recursive(true);

        assert!(!tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(180));
        assert!(!tasklet.create_directories);
        assert!(tasklet.recursive);

        Ok(())
    }

    #[test]
    fn test_ftp_put_folder_tasklet_execution_with_connection_error() {
        let temp_dir = temp_dir();
        let test_folder = temp_dir.join("connection_error_folder_test");
        fs::create_dir_all(&test_folder).unwrap();
        fs::write(test_folder.join("file.txt"), "content").unwrap();

        let tasklet = FtpPutFolderTasklet::new(
            "nonexistent.host.invalid",
            21,
            "user",
            "pass",
            &test_folder,
            "/remote/folder",
        )
        .unwrap();

        let step_execution = StepExecution::new("test-step");
        let result = tasklet.execute(&step_execution);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, BatchError::Io(_)));
        assert!(error
            .to_string()
            .contains("Failed to connect to FTP server"));

        fs::remove_dir_all(&test_folder).ok();
    }

    #[test]
    fn test_ftp_get_folder_tasklet_execution_with_connection_error() {
        let temp_dir = temp_dir();
        let local_folder = temp_dir.join("connection_error_folder_test");

        let tasklet = FtpGetFolderTasklet::new(
            "nonexistent.host.invalid",
            21,
            "user",
            "pass",
            "/remote/folder",
            &local_folder,
        )
        .unwrap();

        let step_execution = StepExecution::new("test-step");
        let result = tasklet.execute(&step_execution);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, BatchError::Io(_)));
        assert!(error
            .to_string()
            .contains("Failed to connect to FTP server"));
    }

    #[test]
    fn test_builder_default_implementations() {
        // Test that all builders implement Default
        let _put_builder = FtpPutTaskletBuilder::default();
        let _get_builder = FtpGetTaskletBuilder::default();
        let _put_folder_builder = FtpPutFolderTaskletBuilder::default();
        let _get_folder_builder = FtpGetFolderTaskletBuilder::default();
    }

    #[test]
    fn test_builder_fluent_interface() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("fluent_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test method chaining works correctly
        let tasklet = FtpPutTaskletBuilder::new()
            .host("example.com")
            .port(2121)
            .username("testuser")
            .password("testpass")
            .local_file(&test_file)
            .remote_file("/remote/test.txt")
            .passive_mode(true)
            .timeout(Duration::from_secs(45))
            .build()?;

        assert_eq!(tasklet.host, "example.com");
        assert_eq!(tasklet.port, 2121);
        assert_eq!(tasklet.username, "testuser");
        assert_eq!(tasklet.password, "testpass");
        assert_eq!(tasklet.remote_file, "/remote/test.txt");
        assert!(tasklet.passive_mode);
        assert_eq!(tasklet.timeout, Duration::from_secs(45));

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_error_message_quality() {
        // Test that error messages are descriptive and helpful
        let result = FtpPutTaskletBuilder::new().build();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("FTP host is required"));

        let result = FtpPutTaskletBuilder::new().host("localhost").build();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("FTP username is required"));
    }

    #[test]
    fn test_path_handling() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("path_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test that different path types work
        let tasklet1 = FtpPutTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            &test_file,
            "/remote/file.txt",
        )?;

        let tasklet2 = FtpPutTasklet::new(
            "localhost",
            21,
            "user",
            "pass",
            test_file.as_path(),
            "/remote/file.txt",
        )?;

        assert_eq!(tasklet1.local_file, tasklet2.local_file);

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_timeout_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("timeout_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test various timeout values
        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .timeout(Duration::from_millis(500))
            .build()?;

        assert_eq!(tasklet.timeout, Duration::from_millis(500));

        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .timeout(Duration::from_secs(300))
            .build()?;

        assert_eq!(tasklet.timeout, Duration::from_secs(300));

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_port_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("port_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test various port values
        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .port(990) // FTPS port
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .build()?;

        assert_eq!(tasklet.port, 990);

        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .port(2121) // Alternative FTP port
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .build()?;

        assert_eq!(tasklet.port, 2121);

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_passive_mode_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("passive_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test passive mode true
        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .passive_mode(true)
            .build()?;

        assert!(tasklet.passive_mode);

        // Test passive mode false (active mode)
        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .passive_mode(false)
            .build()?;

        assert!(!tasklet.passive_mode);

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_secure_ftp_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_file = temp_dir.join("secure_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test secure mode disabled (default)
        let tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/remote/file.txt")
            .build()?;

        assert!(!tasklet.secure);

        // Test secure mode enabled (FTPS)
        let tasklet = FtpPutTaskletBuilder::new()
            .host("secure-ftp.example.com")
            .port(990)
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/secure/file.txt")
            .secure(true)
            .build()?;

        assert!(tasklet.secure);
        assert_eq!(tasklet.port, 990);

        // Test secure mode with FtpGetTasklet
        let local_file = temp_dir.join("downloaded_secure.txt");
        let get_tasklet = FtpGetTaskletBuilder::new()
            .host("secure-ftp.example.com")
            .port(990)
            .username("user")
            .password("pass")
            .remote_file("/secure/file.txt")
            .local_file(&local_file)
            .secure(true)
            .build()?;

        assert!(get_tasklet.secure);
        assert_eq!(get_tasklet.port, 990);

        fs::remove_file(&test_file).ok();
        Ok(())
    }

    #[test]
    fn test_secure_ftp_folder_configuration() -> Result<(), BatchError> {
        let temp_dir = temp_dir();
        let test_folder = temp_dir.join("secure_folder_test");
        fs::create_dir_all(&test_folder).unwrap();
        fs::write(test_folder.join("file.txt"), "test content").unwrap();

        // Test secure mode disabled (default) for folder upload
        let tasklet = FtpPutFolderTaskletBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass")
            .local_folder(&test_folder)
            .remote_folder("/remote/folder")
            .build()?;

        assert!(!tasklet.secure);

        // Test secure mode enabled (FTPS) for folder upload
        let tasklet = FtpPutFolderTaskletBuilder::new()
            .host("secure-ftp.example.com")
            .port(990)
            .username("user")
            .password("pass")
            .local_folder(&test_folder)
            .remote_folder("/secure/folder")
            .secure(true)
            .build()?;

        assert!(tasklet.secure);
        assert_eq!(tasklet.port, 990);

        // Test secure mode with FtpGetFolderTasklet
        let local_folder = temp_dir.join("downloaded_secure_folder");
        let get_tasklet = FtpGetFolderTaskletBuilder::new()
            .host("secure-ftp.example.com")
            .port(990)
            .username("user")
            .password("pass")
            .remote_folder("/secure/folder")
            .local_folder(&local_folder)
            .secure(true)
            .build()?;

        assert!(get_tasklet.secure);
        assert_eq!(get_tasklet.port, 990);

        fs::remove_dir_all(&test_folder).ok();
        Ok(())
    }
}
