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
//! ### FTP GET Operation
//!
//! ```rust
//! use spring_batch_rs::tasklet::ftp::FtpGetTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let ftp_get_tasklet = FtpGetTaskletBuilder::new()
//!     .host("ftp.example.com")
//!     .username("user")
//!     .password("password")
//!     .remote_file("/remote/path/file.txt")
//!     .local_file("./downloaded_file.txt")
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
use suppaftp::{FtpStream, Mode};

/// A tasklet for uploading files to an FTP server.
///
/// This tasklet provides functionality for uploading local files to an FTP server
/// as part of a batch processing step.
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
        info!(
            "Starting FTP PUT: {} -> {}:{}{}",
            self.local_file.display(),
            self.host,
            self.port,
            self.remote_file
        );

        // Connect to FTP server
        let mut ftp_stream =
            FtpStream::connect(format!("{}:{}", self.host, self.port)).map_err(|e| {
                BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Failed to connect to FTP server: {}", e),
                ))
            })?;

        // Login
        ftp_stream
            .login(&self.username, &self.password)
            .map_err(|e| BatchError::Configuration(format!("FTP login failed: {}", e)))?;

        // Set timeout for control channel commands
        ftp_stream
            .get_ref()
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
        ftp_stream
            .get_ref()
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| {
                BatchError::Configuration(format!("Failed to set write timeout: {}", e))
            })?;

        // Set transfer mode
        let mode = if self.passive_mode {
            Mode::Passive
        } else {
            Mode::Active
        };
        ftp_stream.set_mode(mode);

        // Upload file
        let file = File::open(&self.local_file).map_err(BatchError::Io)?;
        let mut reader = BufReader::new(file);

        ftp_stream
            .put_file(&self.remote_file, &mut reader)
            .map_err(|e| {
                BatchError::Io(std::io::Error::other(format!("FTP upload failed: {}", e)))
            })?;

        // Disconnect
        let _ = ftp_stream.quit();

        info!(
            "FTP PUT completed successfully: {} uploaded to {}:{}{}",
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
/// to local storage as part of a batch processing step.
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
        info!(
            "Starting FTP GET: {}:{}{} -> {}",
            self.host,
            self.port,
            self.remote_file,
            self.local_file.display()
        );

        // Connect to FTP server
        let mut ftp_stream =
            FtpStream::connect(format!("{}:{}", self.host, self.port)).map_err(|e| {
                BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Failed to connect to FTP server: {}", e),
                ))
            })?;

        // Login
        ftp_stream
            .login(&self.username, &self.password)
            .map_err(|e| BatchError::Configuration(format!("FTP login failed: {}", e)))?;

        // Set timeout for control channel commands
        ftp_stream
            .get_ref()
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
        ftp_stream
            .get_ref()
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| {
                BatchError::Configuration(format!("Failed to set write timeout: {}", e))
            })?;

        // Set transfer mode
        let mode = if self.passive_mode {
            Mode::Passive
        } else {
            Mode::Active
        };
        ftp_stream.set_mode(mode);

        // Download file
        let data = ftp_stream.retr_as_buffer(&self.remote_file).map_err(|e| {
            BatchError::Io(std::io::Error::other(format!("FTP download failed: {}", e)))
        })?;

        // Write data to local file
        std::fs::write(&self.local_file, data.into_inner()).map_err(BatchError::Io)?;

        // Disconnect
        let _ = ftp_stream.quit();

        info!(
            "FTP GET completed successfully: {}:{}{} downloaded to {}",
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
/// to a remote folder on an FTP server as part of a batch processing step.
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
}

impl Tasklet for FtpPutFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        info!(
            "Starting FTP PUT FOLDER: {} -> {}:{}{}",
            self.local_folder.display(),
            self.host,
            self.port,
            self.remote_folder
        );

        // Connect to FTP server
        let mut ftp_stream =
            FtpStream::connect(format!("{}:{}", self.host, self.port)).map_err(|e| {
                BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Failed to connect to FTP server: {}", e),
                ))
            })?;

        // Login
        ftp_stream
            .login(&self.username, &self.password)
            .map_err(|e| BatchError::Configuration(format!("FTP login failed: {}", e)))?;

        // Set timeout for control channel commands
        ftp_stream
            .get_ref()
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
        ftp_stream
            .get_ref()
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| {
                BatchError::Configuration(format!("Failed to set write timeout: {}", e))
            })?;

        // Set transfer mode
        let mode = if self.passive_mode {
            Mode::Passive
        } else {
            Mode::Active
        };
        ftp_stream.set_mode(mode);

        // Create remote base directory if needed
        if self.create_directories && !self.remote_folder.is_empty() {
            let _ = ftp_stream.mkdir(&self.remote_folder);
        }

        // Upload folder contents
        self.upload_directory(&mut ftp_stream, &self.local_folder, &self.remote_folder)?;

        // Disconnect
        let _ = ftp_stream.quit();

        info!(
            "FTP PUT FOLDER completed successfully: {} uploaded to {}:{}{}",
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
/// on an FTP server to a local folder as part of a batch processing step.
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

            // Try to determine if it's a file or directory by attempting to download
            match ftp_stream.retr_as_buffer(&remote_full_path) {
                Ok(data) => {
                    // It's a file, save it
                    info!(
                        "Downloading file: {} -> {}",
                        remote_full_path,
                        local_path.display()
                    );

                    if self.create_directories {
                        if let Some(parent) = local_path.parent() {
                            fs::create_dir_all(parent).map_err(BatchError::Io)?;
                        }
                    }

                    // Extract the data from the cursor
                    let bytes = data.into_inner();
                    fs::write(&local_path, bytes).map_err(BatchError::Io)?;
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
}

impl Tasklet for FtpGetFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        info!(
            "Starting FTP GET FOLDER: {}:{}{} -> {}",
            self.host,
            self.port,
            self.remote_folder,
            self.local_folder.display()
        );

        // Connect to FTP server
        let mut ftp_stream =
            FtpStream::connect(format!("{}:{}", self.host, self.port)).map_err(|e| {
                BatchError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Failed to connect to FTP server: {}", e),
                ))
            })?;

        // Login
        ftp_stream
            .login(&self.username, &self.password)
            .map_err(|e| BatchError::Configuration(format!("FTP login failed: {}", e)))?;

        // Set timeout for control channel commands
        ftp_stream
            .get_ref()
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| BatchError::Configuration(format!("Failed to set read timeout: {}", e)))?;
        ftp_stream
            .get_ref()
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| {
                BatchError::Configuration(format!("Failed to set write timeout: {}", e))
            })?;

        // Set transfer mode
        let mode = if self.passive_mode {
            Mode::Passive
        } else {
            Mode::Active
        };
        ftp_stream.set_mode(mode);

        // Create local base directory if needed
        if self.create_directories {
            fs::create_dir_all(&self.local_folder).map_err(BatchError::Io)?;
        }

        // Download folder contents
        self.download_directory(&mut ftp_stream, &self.remote_folder, &self.local_folder)?;

        // Disconnect
        let _ = ftp_stream.quit();

        info!(
            "FTP GET FOLDER completed successfully: {}:{}{} downloaded to {}",
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

        Ok(tasklet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

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

        // Test missing username
        let result = FtpGetTaskletBuilder::new()
            .host("localhost")
            .password("pass")
            .build();
        assert!(result.is_err());
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

        // Test missing username for folder download
        let result = FtpGetFolderTaskletBuilder::new()
            .host("localhost")
            .password("pass")
            .build();
        assert!(result.is_err());
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

        fs::remove_file(&test_file).ok();
    }
}
