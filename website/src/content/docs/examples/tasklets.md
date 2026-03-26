---
title: Tasklet Examples
description: Examples for ZIP compression and FTP operations with Spring Batch RS
sidebar:
  order: 7
---

Tasklets handle single-task operations that don't fit the chunk-oriented read-process-write pattern. Spring Batch RS provides built-in tasklets for ZIP compression and FTP file transfers.

## ZIP Compression

Create ZIP archives from files and directories with configurable compression and filtering.

### Quick Start

```rust
use spring_batch_rs::tasklet::zip::ZipTaskletBuilder;
use spring_batch_rs::core::step::StepBuilder;

let zip_tasklet = ZipTaskletBuilder::new()
    .source_path("./data")
    .target_path("./archive.zip")
    .compression_level(6)
    .build()?;

let step = StepBuilder::new("create-archive")
    .tasklet(&zip_tasklet)
    .build();
```

### Features

- **Directory compression**: Compress entire directory trees
- **File filtering**: Include/exclude patterns for selective archiving
- **Compression levels**: 0 (none) to 9 (maximum)
- **Structure control**: Preserve or flatten directory hierarchy
- **Single file support**: Compress individual files

### Complete Example

The [`tasklet_zip`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_zip.rs) example demonstrates:

1. **Basic compression**: Compress a directory with defaults
2. **Filtered compression**: Include only specific file types
3. **Flattened archive**: No subdirectories in ZIP
4. **Single file**: Compress one file
5. **Multi-step job**: Create multiple archives in one job

```bash
cargo run --example tasklet_zip --features zip
```

### ZipTaskletBuilder API

| Method | Description |
|--------|-------------|
| `source_path(path)` | File or directory to compress (required) |
| `target_path(path)` | Output ZIP file path (required) |
| `compression_level(0-9)` | Compression level (default: 6) |
| `include_pattern(glob)` | Include files matching pattern |
| `exclude_pattern(glob)` | Exclude files matching pattern |
| `preserve_structure(bool)` | Keep directory structure (default: true) |

### Pattern Examples

```rust
// Only text files
let tasklet = ZipTaskletBuilder::new()
    .source_path("./logs")
    .target_path("./logs.zip")
    .include_pattern("*.txt")
    .build()?;

// Exclude temporary files
let tasklet = ZipTaskletBuilder::new()
    .source_path("./data")
    .target_path("./data.zip")
    .exclude_pattern("*.tmp")
    .build()?;
```

## FTP Operations

Transfer files to and from FTP servers with support for single files, directories, and secure FTPS.

:::note
FTP examples require a running FTP server.
:::

### Quick Start

```rust
use spring_batch_rs::tasklet::ftp::{FtpPutTaskletBuilder, FtpGetTaskletBuilder};

// Upload file
let upload = FtpPutTaskletBuilder::new()
    .host("ftp.example.com")
    .username("user")
    .password("pass")
    .local_file("./data.txt")
    .remote_file("/uploads/data.txt")
    .build()?;

// Download file
let download = FtpGetTaskletBuilder::new()
    .host("ftp.example.com")
    .username("user")
    .password("pass")
    .remote_file("/files/report.csv")
    .local_file("./report.csv")
    .build()?;
```

### Features

- **PUT/GET operations**: Upload and download files
- **Folder operations**: Transfer entire directories
- **FTPS support**: Secure FTP over TLS
- **Active/Passive mode**: Configurable transfer mode
- **Streaming downloads**: Memory-efficient for large files

### Complete Example

The [`tasklet_ftp`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_ftp.rs) example demonstrates:

1. **FTP PUT**: Upload single file
2. **FTP GET**: Download single file
3. **PUT FOLDER**: Upload entire directory
4. **GET FOLDER**: Download entire directory
5. **Multi-step workflow**: Upload then download
6. **FTPS configuration**: Secure connections

```bash
# Start FTP server first
docker run -d -p 21:21 -p 21000-21010:21000-21010 \
  -e USERS="user|password" --name ftp-server delfer/alpine-ftp-server

cargo run --example tasklet_ftp --features ftp
```

### FTP Builder APIs

#### FtpPutTaskletBuilder

| Method | Description |
|--------|-------------|
| `host(hostname)` | FTP server address (required) |
| `port(u16)` | Server port (default: 21) |
| `username(user)` | FTP username (required) |
| `password(pass)` | FTP password (required) |
| `local_file(path)` | Local file to upload (required) |
| `remote_file(path)` | Remote destination path (required) |
| `passive_mode(bool)` | Use passive mode (default: false) |
| `secure(bool)` | Enable FTPS (default: false) |

#### FtpGetTaskletBuilder

| Method | Description |
|--------|-------------|
| `host(hostname)` | FTP server address (required) |
| `port(u16)` | Server port (default: 21) |
| `username(user)` | FTP username (required) |
| `password(pass)` | FTP password (required) |
| `remote_file(path)` | Remote file to download (required) |
| `local_file(path)` | Local destination path (required) |
| `passive_mode(bool)` | Use passive mode (default: false) |
| `secure(bool)` | Enable FTPS (default: false) |

### Folder Operations

```rust
use spring_batch_rs::tasklet::ftp::{
    FtpPutFolderTaskletBuilder,
    FtpGetFolderTaskletBuilder
};

// Upload entire folder
let upload_folder = FtpPutFolderTaskletBuilder::new()
    .host("ftp.example.com")
    .username("user")
    .password("pass")
    .local_folder("./reports")
    .remote_folder("/archive/reports")
    .passive_mode(true)
    .build()?;

// Download entire folder
let download_folder = FtpGetFolderTaskletBuilder::new()
    .host("ftp.example.com")
    .username("user")
    .password("pass")
    .remote_folder("/data/export")
    .local_folder("./downloaded")
    .build()?;
```

### Secure FTPS

```rust
let secure_upload = FtpPutTaskletBuilder::new()
    .host("secure-ftp.example.com")
    .port(990)  // Implicit FTPS port
    .username("user")
    .password("pass")
    .local_file("./sensitive.dat")
    .remote_file("/secure/sensitive.dat")
    .secure(true)
    .build()?;
```

## Multi-Step Tasklet Jobs

Combine multiple tasklets in a single job:

```rust
// Step 1: Create archive
let zip_tasklet = ZipTaskletBuilder::new()
    .source_path("./data")
    .target_path("./archive.zip")
    .build()?;

// Step 2: Upload archive
let ftp_tasklet = FtpPutTaskletBuilder::new()
    .host("ftp.example.com")
    .username("user")
    .password("pass")
    .local_file("./archive.zip")
    .remote_file("/backups/archive.zip")
    .build()?;

let step1 = StepBuilder::new("compress")
    .tasklet(&zip_tasklet)
    .build();

let step2 = StepBuilder::new("upload")
    .tasklet(&ftp_tasklet)
    .build();

let job = JobBuilder::new()
    .start(&step1)
    .next(&step2)
    .build();

job.run()?;
```

## See Also

- [Advanced Patterns](/spring-batch-rs/examples/advanced-patterns/) - Complex job workflows
- [CSV Processing](/spring-batch-rs/examples/csv/) - Data to archive
- [Database Processing](/spring-batch-rs/examples/database/) - Export and archive
