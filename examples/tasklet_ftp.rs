//! # FTP Tasklet Examples
//!
//! Demonstrates FTP file transfer operations with Spring Batch RS tasklets.
//!
//! **Note**: These examples require a running FTP server.
//!
//! ## Features Demonstrated
//! - FTP PUT (upload single file)
//! - FTP GET (download single file)
//! - FTP PUT FOLDER (upload directory)
//! - FTP GET FOLDER (download directory)
//! - Secure FTPS connections
//! - Passive/Active mode configuration
//!
//! ## Prerequisites
//! ```bash
//! # Start a local FTP server (using Docker)
//! docker run -d -p 21:21 -p 21000-21010:21000-21010 \
//!   -e USERS="user|password" \
//!   --name ftp-server delfer/alpine-ftp-server
//! ```
//!
//! ## Run
//! ```bash
//! cargo run --example tasklet_ftp --features ftp
//! ```

use spring_batch_rs::{
    BatchError,
    core::{
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    tasklet::ftp::{
        FtpGetFolderTaskletBuilder, FtpGetTaskletBuilder, FtpPutFolderTaskletBuilder,
        FtpPutTaskletBuilder,
    },
};
use std::{
    env::temp_dir,
    fs::{self, File},
    io::Write,
    path::Path,
};

// =============================================================================
// FTP Configuration
// =============================================================================

const FTP_HOST: &str = "localhost";
const FTP_PORT: u16 = 21;
const FTP_USER: &str = "user";
const FTP_PASS: &str = "password";

// =============================================================================
// Setup: Create Test Data
// =============================================================================

/// Creates sample files for upload testing.
fn create_sample_files(base_path: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(base_path.join("upload_folder"))?;

    // Single file for upload
    let mut file = File::create(base_path.join("upload_file.txt"))?;
    file.write_all(b"This file will be uploaded to FTP server.\n")?;

    // Folder with multiple files
    for i in 1..=3 {
        let mut file = File::create(base_path.join(format!("upload_folder/file{}.txt", i)))?;
        file.write_all(format!("Content of file {}\n", i).as_bytes())?;
    }

    println!("Sample files created at: {}", base_path.display());
    Ok(())
}

// =============================================================================
// Example 1: FTP PUT (Upload Single File)
// =============================================================================

/// Uploads a single file to the FTP server.
fn example_ftp_put(local_path: &Path) -> Result<(), BatchError> {
    println!("=== Example 1: FTP PUT (Upload File) ===");

    let put_tasklet = FtpPutTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .local_file(local_path.join("upload_file.txt"))
        .remote_file("/uploaded_file.txt")
        .passive_mode(true)
        .build()?;

    let step = StepBuilder::new("ftp-put").tasklet(&put_tasklet).build();
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    println!("  Uploaded: upload_file.txt -> /uploaded_file.txt");
    println!("  Duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 2: FTP GET (Download Single File)
// =============================================================================

/// Downloads a single file from the FTP server.
fn example_ftp_get(local_path: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 2: FTP GET (Download File) ===");

    let get_tasklet = FtpGetTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .remote_file("/uploaded_file.txt")
        .local_file(local_path.join("downloaded_file.txt"))
        .passive_mode(true)
        .build()?;

    let step = StepBuilder::new("ftp-get").tasklet(&get_tasklet).build();
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    println!("  Downloaded: /uploaded_file.txt -> downloaded_file.txt");
    println!("  Duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 3: FTP PUT FOLDER (Upload Directory)
// =============================================================================

/// Uploads an entire folder to the FTP server.
fn example_ftp_put_folder(local_path: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 3: FTP PUT FOLDER ===");

    let put_folder_tasklet = FtpPutFolderTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .local_folder(local_path.join("upload_folder"))
        .remote_folder("/uploaded_folder")
        .passive_mode(true)
        .build()?;

    let step = StepBuilder::new("ftp-put-folder")
        .tasklet(&put_folder_tasklet)
        .build();
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    println!("  Uploaded folder: upload_folder -> /uploaded_folder");
    println!("  Duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 4: FTP GET FOLDER (Download Directory)
// =============================================================================

/// Downloads an entire folder from the FTP server.
fn example_ftp_get_folder(local_path: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 4: FTP GET FOLDER ===");

    let download_path = local_path.join("downloaded_folder");
    fs::create_dir_all(&download_path).map_err(BatchError::Io)?;

    let get_folder_tasklet = FtpGetFolderTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .remote_folder("/uploaded_folder")
        .local_folder(&download_path)
        .passive_mode(true)
        .build()?;

    let step = StepBuilder::new("ftp-get-folder")
        .tasklet(&get_folder_tasklet)
        .build();
    let job = JobBuilder::new().start(&step).build();
    let result = job.run()?;

    println!("  Downloaded folder: /uploaded_folder -> downloaded_folder");
    println!("  Duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 5: Multi-Step FTP Job
// =============================================================================

/// Demonstrates a complete upload/download workflow.
fn example_multi_step_ftp(local_path: &Path) -> Result<(), BatchError> {
    println!("\n=== Example 5: Multi-Step FTP Job ===");

    // Step 1: Upload file
    let upload_tasklet = FtpPutTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .local_file(local_path.join("upload_file.txt"))
        .remote_file("/workflow/data.txt")
        .passive_mode(true)
        .build()?;

    // Step 2: Download it back
    let download_tasklet = FtpGetTaskletBuilder::new()
        .host(FTP_HOST)
        .port(FTP_PORT)
        .username(FTP_USER)
        .password(FTP_PASS)
        .remote_file("/workflow/data.txt")
        .local_file(local_path.join("workflow_result.txt"))
        .passive_mode(true)
        .build()?;

    let upload_step = StepBuilder::new("upload").tasklet(&upload_tasklet).build();
    let download_step = StepBuilder::new("download")
        .tasklet(&download_tasklet)
        .build();

    let job = JobBuilder::new()
        .start(&upload_step)
        .next(&download_step)
        .build();

    let result = job.run()?;

    println!("  Step 1: Uploaded to /workflow/data.txt");
    println!("  Step 2: Downloaded to workflow_result.txt");
    println!("  Total duration: {:?}", result.duration);
    Ok(())
}

// =============================================================================
// Example 6: Secure FTPS Connection
// =============================================================================

/// Demonstrates secure FTP over TLS.
fn example_secure_ftps() -> Result<(), BatchError> {
    println!("\n=== Example 6: Secure FTPS (Configuration Only) ===");

    // Note: This example shows the configuration but won't run without
    // an FTPS server. Uncomment the build() and run to test with a real server.

    let _secure_upload = FtpPutTaskletBuilder::new()
        .host("secure-ftp.example.com")
        .port(990) // Implicit FTPS port
        .username("secure_user")
        .password("secure_pass")
        .local_file("/path/to/sensitive_data.txt")
        .remote_file("/secure/data.txt")
        .secure(true) // Enable FTPS
        .passive_mode(true);
    // .build()?;

    println!("  FTPS configuration:");
    println!("  - Port: 990 (implicit FTPS)");
    println!("  - TLS: Enabled");
    println!("  - Mode: Passive");

    Ok(())
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<(), BatchError> {
    println!("FTP Tasklet Examples");
    println!("====================\n");
    println!("Note: Requires FTP server at {}:{}\n", FTP_HOST, FTP_PORT);

    // Setup directories
    let temp_base = temp_dir().join("spring_batch_ftp_examples");

    // Clean up existing
    if temp_base.exists() {
        fs::remove_dir_all(&temp_base).map_err(|e| {
            BatchError::Io(std::io::Error::other(format!("Failed to clean up: {}", e)))
        })?;
    }

    fs::create_dir_all(&temp_base).map_err(BatchError::Io)?;

    // Create sample files
    create_sample_files(&temp_base).map_err(BatchError::Io)?;
    println!();

    // Run examples (comment out if no FTP server available)
    example_ftp_put(&temp_base)?;
    example_ftp_get(&temp_base)?;
    example_ftp_put_folder(&temp_base)?;
    example_ftp_get_folder(&temp_base)?;
    example_multi_step_ftp(&temp_base)?;

    // This example shows configuration only
    example_secure_ftps()?;

    println!("\n✓ All FTP examples completed successfully!");

    Ok(())
}
