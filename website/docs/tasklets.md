---
sidebar_position: 3
---

# Tasklets

Tasklets provide a simple way to execute single operations within a batch job. Unlike chunk-oriented processing, tasklets are designed for tasks that don't fit the read-process-write pattern, such as file operations, system commands, or cleanup tasks.

## Overview

A tasklet is a single unit of work that executes once per step. Common use cases include:

- File transfer operations (FTP, SFTP)
- Archive and compression tasks
- Database maintenance operations
- System cleanup tasks
- External service integrations

## Built-in Tasklets

### FTP Tasklets

Spring Batch RS provides comprehensive FTP support for file transfer operations.

#### FTP File Upload (`FtpPutTasklet`)

Upload individual files to an FTP server:

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

#### FTP File Download (`FtpGetTasklet`)

Download individual files from an FTP server:

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
        .create_directories(true)  // Create local directories if needed
        .build()?;

    let step = StepBuilder::new("download_input")
        .tasklet(&download_tasklet)
        .build();

    let job = JobBuilder::new().start(&step).build();
    job.run()
}
```

**Additional Options:**

- `create_directories(bool)` - Create local directories if they don't exist

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
        .build()?;

    let step = StepBuilder::new("upload_reports")
        .tasklet(&upload_tasklet)
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
        .build()?;

    let step = StepBuilder::new("download_reports")
        .tasklet(&download_tasklet)
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

For FTP operations, consider security best practices:

1. **Use Environment Variables** for credentials:

```rust
let ftp_host = std::env::var("FTP_HOST")
    .map_err(|_| BatchError::Configuration("FTP_HOST not set".to_string()))?;
let ftp_user = std::env::var("FTP_USER")
    .map_err(|_| BatchError::Configuration("FTP_USER not set".to_string()))?;
let ftp_pass = std::env::var("FTP_PASS")
    .map_err(|_| BatchError::Configuration("FTP_PASS not set".to_string()))?;
```

2. **Enable Passive Mode** for better firewall compatibility:

```rust
.passive_mode(true)
```

3. **Set Appropriate Timeouts**:

```rust
.timeout(Duration::from_secs(60))
```

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

FTP tasklets require the `ftp` feature flag:

```toml
[dependencies]
spring-batch-rs = { version = "0.1", features = ["ftp"] }
```

Available tasklet features:

- `ftp` - FTP file transfer operations
- `zip` - Archive and compression operations

## Next Steps

- See [Examples](examples.md) for complete workflow examples
- Check [Processing Models](processing-models.md) for chunk vs tasklet comparison
- Visit [Architecture](architecture.md) for system design patterns
