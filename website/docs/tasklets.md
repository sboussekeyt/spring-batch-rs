---
sidebar_position: 3
---

# Tasklets

Tasklets provide a simple way to execute single operations within a batch job. Unlike chunk-oriented processing, tasklets are designed for tasks that don't fit the read-process-write pattern, such as file operations, system commands, or cleanup tasks.

## Overview

A tasklet is a single unit of work that executes once per step. Common use cases include:

- File transfer operations (FTP, FTPS)
- Archive and compression tasks
- Database maintenance operations
- System cleanup tasks
- External service integrations

## Built-in Tasklets

### FTP Tasklets

Spring Batch RS provides comprehensive FTP and FTPS support for secure file transfer operations. All FTP tasklets support both plain FTP and secure FTPS (FTP over TLS) connections.

#### Security Overview

All FTP tasklets support secure connections through FTPS (FTP over TLS):

- **Plain FTP**: Traditional unencrypted FTP (default: `secure(false)`)
- **FTPS**: FTP over TLS for encrypted file transfers (`secure(true)`)
- **Memory Efficient**: Both modes use streaming downloads for optimal memory usage
- **Easy Configuration**: Simple boolean flag to enable/disable encryption

#### FTP File Upload (`FtpPutTasklet`)

Upload individual files to an FTP server with optional TLS encryption:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpPutTaskletBuilder,
    BatchError,
};

fn upload_file() -> Result<(), BatchError> {
    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host("ftp.example.com")
        .port(21)
        .username("user")
        .password("password")
        .local_file("./data/report.csv")
        .remote_file("/uploads/report.csv")
        .passive_mode(true)
        .secure(false)  // Use plain FTP
        .build()?;

    let step = StepBuilder::new("upload_report")
        .tasklet(&upload_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

**Configuration Options:**

- `host(host)` - FTP server hostname
- `port(port)` - FTP server port (default: 21)
- `username(user)` - FTP username
- `password(pass)` - FTP password
- `local_file(path)` - Local file path to upload
- `remote_file(path)` - Remote destination path
- `passive_mode(bool)` - Enable passive mode (recommended)
- `timeout(duration)` - Connection timeout
- `secure(bool)` - Enable FTPS (FTP over TLS) for encrypted transfers

#### FTPS Secure File Upload

For sensitive data, use FTPS with TLS encryption:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpPutTaskletBuilder,
    BatchError,
};

fn secure_upload() -> Result<(), BatchError> {
    let secure_upload_tasklet = FtpPutTaskletBuilder::new()
        .host("secure-ftp.example.com")
        .port(990)  // Common FTPS port
        .username("secure_user")
        .password("secure_password")
        .local_file("./sensitive/financial_report.xlsx")
        .remote_file("/secure/uploads/financial_report.xlsx")
        .passive_mode(true)
        .secure(true)  // Enable FTPS encryption
        .timeout(Duration::from_secs(60))
        .build()?;

    let step = StepBuilder::new("secure_upload")
        .tasklet(&secure_upload_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

#### FTP File Download (`FtpGetTasklet`)

Download individual files from an FTP server with memory-efficient streaming:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpGetTaskletBuilder,
    BatchError,
};

fn download_file() -> Result<(), BatchError> {
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host("ftp.example.com")
        .port(21)
        .username("user")
        .password("password")
        .remote_file("/data/input.csv")
        .local_file("./downloads/input.csv")
        .passive_mode(true)
        .secure(false)  // Plain FTP
        .build()?;

    let step = StepBuilder::new("download_input")
        .tasklet(&download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

#### FTPS Secure File Download

Download files securely using FTPS with streaming for large files:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpGetTaskletBuilder,
    BatchError,
};
use std::time::Duration;

fn secure_download() -> Result<(), BatchError> {
    let secure_download_tasklet = FtpGetTaskletBuilder::new()
        .host("secure-ftp.example.com")
        .port(990)  // FTPS port
        .username("secure_user")
        .password("secure_password")
        .remote_file("/secure/data/large_dataset.zip")  // Handles large files efficiently
        .local_file("./downloads/large_dataset.zip")
        .passive_mode(true)
        .secure(true)  // Enable FTPS encryption
        .timeout(Duration::from_secs(120))
        .build()?;

    let step = StepBuilder::new("secure_download")
        .tasklet(&secure_download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

#### FTP Folder Upload (`FtpPutFolderTasklet`)

Upload entire directories to an FTP server:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpPutFolderTaskletBuilder,
    BatchError,
};

fn upload_folder() -> Result<(), BatchError> {
    let upload_tasklet = FtpPutFolderTaskletBuilder::new()
        .host("ftp.example.com")
        .port(21)
        .username("user")
        .password("password")
        .local_folder("./reports")
        .remote_folder("/uploads/reports")
        .recursive(true)           // Include subdirectories
        .create_directories(true)  // Create remote directories
        .passive_mode(true)
        .secure(false)             // Plain FTP
        .build()?;

    let step = StepBuilder::new("upload_reports")
        .tasklet(&upload_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

#### FTPS Secure Folder Upload

Securely upload entire directory structures using FTPS:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpPutFolderTaskletBuilder,
    BatchError,
};
use std::time::Duration;

fn secure_folder_upload() -> Result<(), BatchError> {
    let secure_folder_tasklet = FtpPutFolderTaskletBuilder::new()
        .host("secure-ftp.example.com")
        .port(990)
        .username("secure_user")
        .password("secure_password")
        .local_folder("./confidential_reports")
        .remote_folder("/secure/uploads/reports")
        .recursive(true)           // Upload subdirectories
        .create_directories(true)  // Create remote structure
        .passive_mode(true)
        .secure(true)              // Enable FTPS encryption
        .timeout(Duration::from_secs(180))
        .build()?;

    let step = StepBuilder::new("secure_folder_upload")
        .tasklet(&secure_folder_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

**Folder-Specific Options:**

- `local_folder(path)` - Local directory to upload
- `remote_folder(path)` - Remote destination directory
- `recursive(bool)` - Include subdirectories
- `create_directories(bool)` - Create remote directories if needed

#### FTP Folder Download (`FtpGetFolderTasklet`)

Download entire directories from an FTP server:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpGetFolderTaskletBuilder,
    BatchError,
};

fn download_folder() -> Result<(), BatchError> {
    let download_tasklet = FtpGetFolderTaskletBuilder::new()
        .host("ftp.example.com")
        .port(21)
        .username("user")
        .password("password")
        .remote_folder("/data/reports")
        .local_folder("./downloads/reports")
        .recursive(true)
        .create_directories(true)
        .passive_mode(true)
        .secure(false)  // Plain FTP
        .build()?;

    let step = StepBuilder::new("download_reports")
        .tasklet(&download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

#### FTPS Secure Folder Download

Download directory structures securely with streaming for memory efficiency:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpGetFolderTaskletBuilder,
    BatchError,
};
use std::time::Duration;

fn secure_folder_download() -> Result<(), BatchError> {
    let secure_download_tasklet = FtpGetFolderTaskletBuilder::new()
        .host("secure-ftp.example.com")
        .port(990)
        .username("secure_user")
        .password("secure_password")
        .remote_folder("/secure/data/archives")
        .local_folder("./downloads/secure_archives")
        .recursive(true)
        .create_directories(true)
        .passive_mode(true)
        .secure(true)  // Enable FTPS encryption
        .timeout(Duration::from_secs(300))  // Longer timeout for large transfers
        .build()?;

    let step = StepBuilder::new("secure_folder_download")
        .tasklet(&secure_download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

### ZIP Archive Tasklet

Create compressed archives of files and directories:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::zip::ZipTaskletBuilder,
    BatchError,
};

fn create_archive() -> Result<(), BatchError> {
    let zip_tasklet = ZipTaskletBuilder::new()
        .source_path("./data")
        .target_path("./archive/data_backup.zip")
        .compression_level(6)
        .build()?;

    let step = StepBuilder::new("create_backup")
        .tasklet(&zip_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

## Advanced FTP/FTPS Patterns

### Environment-Based Configuration

Use environment variables for secure credential management:

```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder},
    tasklet::ftp::FtpPutTaskletBuilder,
    BatchError,
};
use std::{env, time::Duration};

fn secure_upload_with_env() -> Result<(), BatchError> {
    let ftp_host = env::var("SECURE_FTP_HOST")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_HOST not set".to_string()))?;
    let ftp_user = env::var("SECURE_FTP_USER")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_USER not set".to_string()))?;
    let ftp_pass = env::var("SECURE_FTP_PASS")
        .map_err(|_| BatchError::Configuration("SECURE_FTP_PASS not set".to_string()))?;

    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host(&ftp_host)
        .port(990)  // FTPS port
        .username(&ftp_user)
        .password(&ftp_pass)
        .local_file("./data/sensitive.csv")
        .remote_file("/secure/sensitive.csv")
        .passive_mode(true)
        .secure(true)  // Always use FTPS for sensitive data
        .timeout(Duration::from_secs(120))
        .build()?;

    let step = StepBuilder::new("secure_env_upload")
        .tasklet(&upload_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

### Memory Efficiency for Large Files

Both FTP and FTPS modes use streaming for memory-efficient transfers:

```rust
// This example works with files of any size without loading them into memory
fn download_large_file() -> Result<(), BatchError> {
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host("files.example.com")
        .port(990)
        .username("user")
        .password("password")
        .remote_file("/archives/huge_dataset_5gb.zip")  // 5GB file
        .local_file("./downloads/huge_dataset.zip")     // Streams directly to disk
        .secure(true)  // FTPS encryption
        .passive_mode(true)
        .timeout(Duration::from_secs(3600))  // 1 hour for large files
        .build()?;

    let step = StepBuilder::new("download_large_file")
        .tasklet(&download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

### Conditional FTPS Usage

Choose connection type based on environment or configuration:

```rust
fn conditional_secure_upload(use_secure: bool) -> Result<(), BatchError> {
    let port = if use_secure { 990 } else { 21 };

    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host("ftp.example.com")
        .port(port)
        .username("user")
        .password("password")
        .local_file("./data/report.csv")
        .remote_file("/uploads/report.csv")
        .passive_mode(true)
        .secure(use_secure)  // Conditional FTPS
        .timeout(Duration::from_secs(if use_secure { 120 } else { 60 }))
        .build()?;

    let step_name = if use_secure { "secure_upload" } else { "upload" };
    let step = StepBuilder::new(step_name)
        .tasklet(&upload_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

## Custom Tasklets

You can create custom tasklets by implementing the `Tasklet` trait:

```rust
use spring_batch_rs::{
    core::{
        job::JobBuilder,
        step::{StepBuilder, Tasklet, StepExecution, RepeatStatus}
    },
    BatchError,
};
use std::fs;
use log::info;

struct CleanupTasklet {
    directory: String,
    max_age_days: u64,
}

impl Tasklet for CleanupTasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        info!("Starting cleanup in directory: {}", self.directory);

        let cutoff_time = std::time::SystemTime::now()
            - std::time::Duration::from_secs(self.max_age_days * 24 * 60 * 60);

        let entries = fs::read_dir(&self.directory)
            .map_err(|e| BatchError::Tasklet(format!("Failed to read directory: {}", e)))?;

        let mut deleted_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| BatchError::Tasklet(e.to_string()))?;
            let path = entry.path();

            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        fs::remove_file(&path)
                            .map_err(|e| BatchError::Tasklet(format!("Failed to delete file: {}", e)))?;
                        deleted_count += 1;
                        info!("Deleted old file: {:?}", path);
                    }
                }
            }
        }

        info!("Cleanup completed. Deleted {} files", deleted_count);
        Ok(RepeatStatus::Finished)
    }
}

fn cleanup_old_files() -> Result<(), BatchError> {
    let cleanup_tasklet = CleanupTasklet {
        directory: "./temp".to_string(),
        max_age_days: 7,
    };

    let step = StepBuilder::new("cleanup_temp_files")
        .tasklet(&cleanup_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

## Best Practices

### Error Handling

Always handle errors appropriately in tasklets:

```rust
impl Tasklet for MyTasklet {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        match self.perform_operation() {
            Ok(_) => {
                info!("Operation completed successfully");
                Ok(RepeatStatus::Finished)
            }
            Err(e) => {
                error!("Operation failed: {}", e);
                Err(BatchError::Tasklet(format!("Operation failed: {}", e)))
            }
        }
    }
}
```

### Security Considerations

For FTP/FTPS operations, follow security best practices:

1. **Use FTPS for Sensitive Data**: Always enable encryption for confidential files:

```rust
.secure(true)  // Enable FTPS encryption
.port(990)     // Use secure port
```

2. **Use Environment Variables** for credentials:

```rust
let ftp_host = std::env::var("FTP_HOST")
    .map_err(|_| BatchError::Configuration("FTP_HOST not set".to_string()))?;
let ftp_user = std::env::var("FTP_USER")
    .map_err(|_| BatchError::Configuration("FTP_USER not set".to_string()))?;
let ftp_pass = std::env::var("FTP_PASS")
    .map_err(|_| BatchError::Configuration("FTP_PASS not set".to_string()))?;
```

3. **Enable Passive Mode** for better firewall compatibility:

```rust
.passive_mode(true)
```

4. **Set Appropriate Timeouts** for secure connections:

```rust
.timeout(Duration::from_secs(120))  // FTPS may need longer timeouts
```

5. **Use Secure Ports**: Common secure FTP ports include:
   - 990: FTPS (FTP over TLS)
   - 22: SFTP (SSH File Transfer Protocol) - not yet supported

### Performance Optimization

1. **Memory Efficiency**: All FTP tasklets use streaming for large files:

   - Files are transferred directly from source to destination
   - Memory usage remains constant regardless of file size
   - Suitable for files from bytes to gigabytes

2. **Connection Management**: Configure timeouts appropriately:
   - Shorter timeouts for small files
   - Longer timeouts for large transfers or FTPS handshakes
   - Consider network latency and file sizes

### Retry Logic

Implement retry logic for unreliable operations:

```rust
struct RetryableTasklet<T: Tasklet> {
    inner: T,
    max_retries: u32,
    delay: Duration,
}

impl<T: Tasklet> Tasklet for RetryableTasklet<T> {
    fn execute(&self, step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self.inner.execute(step_execution) {
                Ok(status) => return Ok(status),
                Err(e) if attempts >= self.max_retries => return Err(e),
                Err(e) => {
                    warn!("Attempt {} failed: {}. Retrying...", attempts, e);
                    std::thread::sleep(self.delay);
                }
            }
        }
    }
}
```

## Feature Flags

FTP/FTPS tasklets require the `ftp` feature flag:

```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["ftp"] }
```

Available tasklet features:

- `ftp` - FTP and FTPS file transfer operations
- `zip` - Archive and compression operations

## Security Matrix

| Feature                | Plain FTP         | FTPS                     |
| ---------------------- | ----------------- | ------------------------ |
| **Encryption**         | ❌ None           | ✅ TLS/SSL               |
| **Data Protection**    | ❌ Plaintext      | ✅ Encrypted             |
| **Memory Efficiency**  | ✅ Streaming      | ✅ Streaming             |
| **Large File Support** | ✅ Yes            | ✅ Yes                   |
| **Performance**        | ⚡ Fast           | 🔒 Secure + Fast         |
| **Recommended For**    | Internal networks | Internet, sensitive data |

## Next Steps

- See [Examples](examples.md) for complete workflow examples with FTPS
- Check [Processing Models](processing-models.md) for chunk vs tasklet comparison
- Visit [Architecture](architecture.md) for system design patterns
- Review [Getting Started](getting-started.md) for environment setup
