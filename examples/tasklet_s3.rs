//! # Example: S3 File Transfer with Tasklets
//!
//! Demonstrates uploading a local file to S3 and downloading it back,
//! using `S3PutTasklet` and `S3GetTasklet` as batch steps.
//!
//! ## Run (requires a running S3-compatible service)
//!
//! ```bash
//! # Start LocalStack
//! docker run --rm -p 4566:4566 localstack/localstack
//!
//! # Create a bucket (requires awscli or localstack tools)
//! aws --endpoint-url=http://localhost:4566 s3 mb s3://my-batch-bucket
//!
//! # Run the example
//! cargo run --example tasklet_s3 --features s3
//! ```
//!
//! ## What It Does
//!
//! 1. Writes a sample CSV file to a temp directory
//! 2. Uploads it to `s3://my-batch-bucket/exports/sample.csv` via `S3PutTasklet`
//! 3. Downloads it back to a different temp path via `S3GetTasklet`
//! 4. Prints job execution status

use spring_batch_rs::{
    core::{
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    tasklet::s3::{get::S3GetTaskletBuilder, put::S3PutTaskletBuilder},
};
use std::env::temp_dir;
use std::fs;

#[tokio::main]
async fn main() {
    // 1. Prepare a sample file to upload
    let upload_path = temp_dir().join("spring_batch_s3_sample.csv");
    fs::write(&upload_path, "id,name\n1,Alice\n2,Bob\n").unwrap(); // example setup — panics on error
    let download_path = temp_dir().join("spring_batch_s3_downloaded.csv");

    // 2. Build the upload tasklet (S3PutTasklet)
    let put_tasklet = S3PutTaskletBuilder::new()
        .bucket("my-batch-bucket")
        .key("exports/sample.csv")
        .local_file(&upload_path)
        .endpoint_url("http://localhost:4566") // LocalStack endpoint
        .access_key_id("test")
        .secret_access_key("test")
        .region("us-east-1")
        .build()
        .unwrap(); // panics on misconfiguration — intentional in examples

    // 3. Build the download tasklet (S3GetTasklet)
    let get_tasklet = S3GetTaskletBuilder::new()
        .bucket("my-batch-bucket")
        .key("exports/sample.csv")
        .local_file(&download_path)
        .endpoint_url("http://localhost:4566")
        .access_key_id("test")
        .secret_access_key("test")
        .region("us-east-1")
        .build()
        .unwrap(); // panics on misconfiguration — intentional in examples

    // 4. Build steps
    let upload_step = StepBuilder::new("s3-upload")
        .tasklet(&put_tasklet)
        .build();

    let download_step = StepBuilder::new("s3-download")
        .tasklet(&get_tasklet)
        .build();

    // 5. Build and run the job
    let job = JobBuilder::new()
        .start(&upload_step)
        .next(&download_step)
        .build();

    match job.run() {
        Ok(result) => {
            println!("Step 1: Uploaded sample.csv to S3");
            println!("Step 2: Downloaded sample.csv from S3");
            println!("Total duration: {:?}", result.duration);
        }
        Err(e) => {
            eprintln!("Job failed: {:?}", e);
        }
    }

    // Cleanup
    fs::remove_file(&upload_path).ok();
    fs::remove_file(&download_path).ok();
}
