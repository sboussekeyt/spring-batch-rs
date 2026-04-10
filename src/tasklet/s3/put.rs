//! S3 PUT tasklets for uploading files and folders to Amazon S3.

use crate::{
    core::step::{RepeatStatus, StepExecution, Tasklet},
    tasklet::s3::{build_s3_client, S3ClientConfig},
    BatchError,
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use log::{debug, info};
use std::path::{Path, PathBuf};
use tokio::runtime::Handle;

const DEFAULT_CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

/// A tasklet that uploads a single local file to an S3 object.
///
/// Files smaller than `chunk_size` are uploaded with a single `put_object` call.
/// Files equal to or larger than `chunk_size` use multipart upload. If multipart
/// upload fails mid-way, `abort_multipart_upload` is called to avoid orphaned parts.
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3PutTaskletBuilder::new()
///     .bucket("my-bucket")
///     .key("exports/file.csv")
///     .local_file("./output/file.csv")
///     .region("eu-west-1")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::ItemWriter`] if the S3 upload fails.
/// Returns [`BatchError::Io`] if the local file cannot be read.
#[derive(Debug)]
pub struct S3PutTasklet {
    bucket: String,
    key: String,
    local_file: PathBuf,
    chunk_size: usize,
    config: S3ClientConfig,
}

impl S3PutTasklet {
    async fn execute_async(&self) -> Result<RepeatStatus, BatchError> {
        info!(
            "Uploading {} to s3://{}/{}",
            self.local_file.display(),
            self.bucket,
            self.key
        );

        let client = build_s3_client(&self.config).await?;
        let file_size = std::fs::metadata(&self.local_file)
            .map_err(BatchError::Io)?
            .len() as usize;

        if file_size < self.chunk_size {
            // Simple single-part upload
            let body = ByteStream::from_path(&self.local_file)
                .await
                .map_err(|e| BatchError::ItemWriter(format!("Failed to read file for upload: {}", e)))?;

            client
                .put_object()
                .bucket(&self.bucket)
                .key(&self.key)
                .body(body)
                .send()
                .await
                .map_err(|e| BatchError::ItemWriter(format!("S3 put_object failed: {}", e)))?;
        } else {
            // Multipart upload
            upload_multipart(&client, &self.bucket, &self.key, &self.local_file, self.chunk_size).await?;
        }

        info!("Upload complete: s3://{}/{}", self.bucket, self.key);
        Ok(RepeatStatus::Finished)
    }
}

impl Tasklet for S3PutTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        tokio::task::block_in_place(|| Handle::current().block_on(self.execute_async()))
    }
}

/// Builder for [`S3PutTasklet`].
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3PutTaskletBuilder::new()
///     .bucket("my-bucket")
///     .key("exports/file.csv")
///     .local_file("./output/file.csv")
///     .region("eu-west-1")
///     .chunk_size(16 * 1024 * 1024)
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::Configuration`] if `bucket`, `key`, or `local_file` are not set.
#[derive(Debug, Default)]
pub struct S3PutTaskletBuilder {
    bucket: Option<String>,
    key: Option<String>,
    local_file: Option<PathBuf>,
    chunk_size: usize,
    config: S3ClientConfig,
}

impl S3PutTaskletBuilder {
    /// Creates a new builder with default settings.
    ///
    /// Default `chunk_size` is 8 MiB.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            ..Default::default()
        }
    }

    /// Sets the S3 bucket name.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().bucket("my-bucket");
    /// ```
    pub fn bucket<S: Into<String>>(mut self, bucket: S) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Sets the S3 object key (path within the bucket).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().key("prefix/file.csv");
    /// ```
    pub fn key<S: Into<String>>(mut self, key: S) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Sets the local file path to upload.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().local_file("./output/file.csv");
    /// ```
    pub fn local_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the AWS region.
    ///
    /// Defaults to the `AWS_REGION` environment variable (or `AWS_DEFAULT_REGION` as fallback)
    /// when not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().region("eu-west-1");
    /// ```
    pub fn region<S: Into<String>>(mut self, region: S) -> Self {
        self.config.region = Some(region.into());
        self
    }

    /// Sets a custom endpoint URL for S3-compatible services (MinIO, LocalStack).
    ///
    /// When set, path-style addressing is enabled automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().endpoint_url("http://localhost:9000");
    /// ```
    pub fn endpoint_url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.endpoint_url = Some(url.into());
        self
    }

    /// Sets the AWS access key ID for explicit credential configuration.
    ///
    /// Must be combined with [`secret_access_key`](Self::secret_access_key).
    /// Falls back to the AWS default credential chain when not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().access_key_id("AKIAIOSFODNN7EXAMPLE");
    /// ```
    pub fn access_key_id<S: Into<String>>(mut self, key_id: S) -> Self {
        self.config.access_key_id = Some(key_id.into());
        self
    }

    /// Sets the AWS secret access key for explicit credential configuration.
    ///
    /// Must be combined with [`access_key_id`](Self::access_key_id).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().secret_access_key("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
    /// ```
    pub fn secret_access_key<S: Into<String>>(mut self, secret: S) -> Self {
        self.config.secret_access_key = Some(secret.into());
        self
    }

    /// Sets the multipart upload chunk size in bytes.
    ///
    /// Files smaller than this value are uploaded in a single request. Files
    /// equal to or larger are split into parts of this size. Defaults to `8 MiB`.
    /// Minimum value is 5 MiB (AWS requirement for multipart parts).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// let builder = S3PutTaskletBuilder::new().chunk_size(16 * 1024 * 1024);
    /// ```
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Builds the [`S3PutTasklet`].
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Configuration`] if `bucket`, `key`, or `local_file` are not set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// let tasklet = S3PutTaskletBuilder::new()
    ///     .bucket("my-bucket")
    ///     .key("file.csv")
    ///     .local_file("./output/file.csv")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<S3PutTasklet, BatchError> {
        let bucket = self
            .bucket
            .ok_or_else(|| BatchError::Configuration("S3PutTasklet: 'bucket' is required".to_string()))?;
        let key = self
            .key
            .ok_or_else(|| BatchError::Configuration("S3PutTasklet: 'key' is required".to_string()))?;
        let local_file = self
            .local_file
            .ok_or_else(|| BatchError::Configuration("S3PutTasklet: 'local_file' is required".to_string()))?;

        Ok(S3PutTasklet {
            bucket,
            key,
            local_file,
            chunk_size: self.chunk_size,
            config: self.config,
        })
    }
}

// ---------------------------------------------------------------------------
// S3PutFolderTasklet
// ---------------------------------------------------------------------------

/// A tasklet that uploads all files from a local folder to an S3 prefix.
///
/// Each file under `local_folder` is uploaded as `<prefix><relative_path>`.
/// Files are uploaded one at a time. Files equal to or larger than `chunk_size`
/// use multipart upload.
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3PutFolderTaskletBuilder::new()
///     .bucket("my-bucket")
///     .prefix("backups/2026-04-10/")
///     .local_folder("./exports/")
///     .region("eu-west-1")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::ItemWriter`] if any S3 upload fails.
/// Returns [`BatchError::Io`] if local folder traversal or file reads fail.
#[derive(Debug)]
pub struct S3PutFolderTasklet {
    bucket: String,
    prefix: String,
    local_folder: PathBuf,
    chunk_size: usize,
    config: S3ClientConfig,
}

impl S3PutFolderTasklet {
    async fn execute_async(&self) -> Result<RepeatStatus, BatchError> {
        info!(
            "Uploading folder {} to s3://{}/{}",
            self.local_folder.display(),
            self.bucket,
            self.prefix
        );

        let client = build_s3_client(&self.config).await?;
        let entries = collect_files(&self.local_folder)?;

        for local_path in &entries {
            let relative = local_path
                .strip_prefix(&self.local_folder)
                .map_err(|e| BatchError::Io(std::io::Error::other(e.to_string())))?;
            let key = format!("{}{}", self.prefix, relative.to_string_lossy().replace('\\', "/"));

            let file_size = std::fs::metadata(local_path).map_err(BatchError::Io)?.len() as usize;

            debug!("Uploading {} -> s3://{}/{}", local_path.display(), self.bucket, key);

            if file_size < self.chunk_size {
                let body = ByteStream::from_path(local_path)
                    .await
                    .map_err(|e| BatchError::ItemWriter(format!("Failed to read {}: {}", local_path.display(), e)))?;

                client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(&key)
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| BatchError::ItemWriter(format!("S3 put_object failed for {}: {}", key, e)))?;
            } else {
                upload_multipart(&client, &self.bucket, &key, local_path, self.chunk_size).await?;
            }
        }

        info!(
            "Folder upload complete: {} files uploaded to s3://{}/{}",
            entries.len(),
            self.bucket,
            self.prefix
        );
        Ok(RepeatStatus::Finished)
    }
}

impl Tasklet for S3PutFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        tokio::task::block_in_place(|| Handle::current().block_on(self.execute_async()))
    }
}

/// Builder for [`S3PutFolderTasklet`].
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3PutFolderTaskletBuilder::new()
///     .bucket("my-bucket")
///     .prefix("backups/2026-04-10/")
///     .local_folder("./exports/")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::Configuration`] if `bucket`, `prefix`, or `local_folder` are not set.
#[derive(Debug, Default)]
pub struct S3PutFolderTaskletBuilder {
    bucket: Option<String>,
    prefix: Option<String>,
    local_folder: Option<PathBuf>,
    chunk_size: usize,
    config: S3ClientConfig,
}

impl S3PutFolderTaskletBuilder {
    /// Creates a new builder with default settings.
    ///
    /// Default `chunk_size` is 8 MiB.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            ..Default::default()
        }
    }

    /// Sets the S3 bucket name.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().bucket("my-bucket");
    /// ```
    pub fn bucket<S: Into<String>>(mut self, bucket: S) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Sets the S3 key prefix for uploaded objects.
    ///
    /// All uploaded files will be stored under this prefix. Defaults to `""` (bucket root).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().prefix("backups/2026-04-10/");
    /// ```
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets the local folder path to upload.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().local_folder("./exports/");
    /// ```
    pub fn local_folder<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_folder = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the AWS region.
    ///
    /// Defaults to `AWS_REGION` (or `AWS_DEFAULT_REGION`) when not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().region("eu-west-1");
    /// ```
    pub fn region<S: Into<String>>(mut self, region: S) -> Self {
        self.config.region = Some(region.into());
        self
    }

    /// Sets a custom endpoint URL for S3-compatible services (MinIO, LocalStack).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().endpoint_url("http://localhost:9000");
    /// ```
    pub fn endpoint_url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.endpoint_url = Some(url.into());
        self
    }

    /// Sets the AWS access key ID for explicit credential configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().access_key_id("AKIAIOSFODNN7EXAMPLE");
    /// ```
    pub fn access_key_id<S: Into<String>>(mut self, key_id: S) -> Self {
        self.config.access_key_id = Some(key_id.into());
        self
    }

    /// Sets the AWS secret access key for explicit credential configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().secret_access_key("wJalrXUtnFEMI/K7MDENG");
    /// ```
    pub fn secret_access_key<S: Into<String>>(mut self, secret: S) -> Self {
        self.config.secret_access_key = Some(secret.into());
        self
    }

    /// Sets the multipart upload chunk size in bytes. Defaults to `8 MiB`.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// let builder = S3PutFolderTaskletBuilder::new().chunk_size(16 * 1024 * 1024);
    /// ```
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Builds the [`S3PutFolderTasklet`].
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Configuration`] if `bucket`, `prefix`, or `local_folder` are not set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use spring_batch_rs::tasklet::s3::put::S3PutFolderTaskletBuilder;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// let tasklet = S3PutFolderTaskletBuilder::new()
    ///     .bucket("my-bucket")
    ///     .prefix("backups/")
    ///     .local_folder("./exports/")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<S3PutFolderTasklet, BatchError> {
        let bucket = self
            .bucket
            .ok_or_else(|| BatchError::Configuration("S3PutFolderTasklet: 'bucket' is required".to_string()))?;
        let prefix = self
            .prefix
            .ok_or_else(|| BatchError::Configuration("S3PutFolderTasklet: 'prefix' is required".to_string()))?;
        let local_folder = self
            .local_folder
            .ok_or_else(|| BatchError::Configuration("S3PutFolderTasklet: 'local_folder' is required".to_string()))?;

        Ok(S3PutFolderTasklet {
            bucket,
            prefix,
            local_folder,
            chunk_size: self.chunk_size,
            config: self.config,
        })
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Performs a multipart upload of a local file to S3.
///
/// Aborts the upload if any part fails to avoid orphaned S3 parts.
async fn upload_multipart(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    local_file: &Path,
    chunk_size: usize,
) -> Result<(), BatchError> {
    let create_resp = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| BatchError::ItemWriter(format!("create_multipart_upload failed for {}: {}", key, e)))?;

    let upload_id = create_resp
        .upload_id()
        .ok_or_else(|| BatchError::ItemWriter("create_multipart_upload returned no upload_id".to_string()))?
        .to_string();

    let result = upload_parts(client, bucket, key, &upload_id, local_file, chunk_size).await;

    if let Err(e) = result {
        // Abort to clean up orphaned parts
        let _ = client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(key)
            .upload_id(&upload_id)
            .send()
            .await;
        return Err(e);
    }

    Ok(())
}

/// Uploads all parts and completes the multipart upload.
async fn upload_parts(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    local_file: &Path,
    chunk_size: usize,
) -> Result<(), BatchError> {
    use std::io::Read;

    let file = std::fs::File::open(local_file).map_err(BatchError::Io)?;
    let mut reader = std::io::BufReader::new(file);
    let mut part_number = 1i32;
    let mut completed_parts = Vec::new();

    loop {
        let mut buffer = Vec::with_capacity(chunk_size);
        let bytes_read = reader
            .by_ref()
            .take(chunk_size as u64)
            .read_to_end(&mut buffer)
            .map_err(BatchError::Io)?;
        if bytes_read == 0 {
            break;
        }

        debug!("Multipart upload: part {} ({} bytes) -> s3://{}/{}", part_number, bytes_read, bucket, key);

        let body = ByteStream::from(buffer);
        let part_resp = client
            .upload_part()
            .bucket(bucket)
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(body)
            .send()
            .await
            .map_err(|e| BatchError::ItemWriter(format!("upload_part {} failed: {}", part_number, e)))?;

        let etag = part_resp
            .e_tag()
            .ok_or_else(|| BatchError::ItemWriter(format!("upload_part {} returned no ETag", part_number)))?
            .to_string();

        completed_parts.push(
            CompletedPart::builder()
                .part_number(part_number)
                .e_tag(etag)
                .build(),
        );

        part_number += 1;
    }

    let completed = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts))
        .build();

    client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(completed)
        .send()
        .await
        .map_err(|e| BatchError::ItemWriter(format!("complete_multipart_upload failed for {}: {}", key, e)))?;

    Ok(())
}

/// Recursively collects all file paths under a directory.
pub(crate) fn collect_files(dir: &Path) -> Result<Vec<PathBuf>, BatchError> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(BatchError::Io)? {
        let entry = entry.map_err(BatchError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    // --- S3PutTaskletBuilder tests ---

    #[test]
    fn should_fail_build_when_bucket_missing() {
        let result = S3PutTaskletBuilder::new()
            .key("file.csv")
            .local_file("/tmp/file.csv")
            .build();
        assert!(result.is_err(), "build should fail without bucket");
        assert!(
            result.unwrap_err().to_string().contains("bucket"),
            "error message should mention 'bucket'"
        );
    }

    #[test]
    fn should_fail_build_when_key_missing() {
        let result = S3PutTaskletBuilder::new()
            .bucket("my-bucket")
            .local_file("/tmp/file.csv")
            .build();
        assert!(result.is_err(), "build should fail without key");
        assert!(
            result.unwrap_err().to_string().contains("key"),
            "error message should mention 'key'"
        );
    }

    #[test]
    fn should_fail_build_when_local_file_missing() {
        let result = S3PutTaskletBuilder::new()
            .bucket("my-bucket")
            .key("file.csv")
            .build();
        assert!(result.is_err(), "build should fail without local_file");
        assert!(
            result.unwrap_err().to_string().contains("local_file"),
            "error message should mention 'local_file'"
        );
    }

    #[test]
    fn should_build_with_required_fields() {
        let result = S3PutTaskletBuilder::new()
            .bucket("my-bucket")
            .key("file.csv")
            .local_file("/tmp/file.csv")
            .build();
        assert!(result.is_ok(), "build should succeed with required fields: {:?}", result.err());
    }

    #[test]
    fn should_apply_default_chunk_size() {
        let tasklet = S3PutTaskletBuilder::new()
            .bucket("b")
            .key("k")
            .local_file("/tmp/f")
            .build()
            .unwrap(); // required fields are set — cannot fail
        assert_eq!(tasklet.chunk_size, DEFAULT_CHUNK_SIZE, "default chunk_size should be 8 MiB");
    }

    #[test]
    fn should_override_chunk_size() {
        let tasklet = S3PutTaskletBuilder::new()
            .bucket("b")
            .key("k")
            .local_file("/tmp/f")
            .chunk_size(16 * 1024 * 1024)
            .build()
            .unwrap(); // required fields are set — cannot fail
        assert_eq!(tasklet.chunk_size, 16 * 1024 * 1024);
    }

    #[test]
    fn should_store_optional_config_fields() {
        let tasklet = S3PutTaskletBuilder::new()
            .bucket("b")
            .key("k")
            .local_file("/tmp/f")
            .region("us-east-1")
            .endpoint_url("http://localhost:9000")
            .access_key_id("AKID")
            .secret_access_key("SECRET")
            .build()
            .unwrap(); // required fields are set — cannot fail
        assert_eq!(tasklet.config.region.as_deref(), Some("us-east-1"));
        assert_eq!(tasklet.config.endpoint_url.as_deref(), Some("http://localhost:9000"));
        assert_eq!(tasklet.config.access_key_id.as_deref(), Some("AKID"));
        assert_eq!(tasklet.config.secret_access_key.as_deref(), Some("SECRET"));
    }

    // --- S3PutFolderTaskletBuilder tests ---

    #[test]
    fn should_fail_folder_build_when_bucket_missing() {
        let result = S3PutFolderTaskletBuilder::new()
            .prefix("backups/")
            .local_folder("/tmp/exports")
            .build();
        assert!(result.is_err(), "build should fail without bucket");
        assert!(result.unwrap_err().to_string().contains("bucket"));
    }

    #[test]
    fn should_fail_folder_build_when_prefix_missing() {
        let result = S3PutFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .local_folder("/tmp/exports")
            .build();
        assert!(result.is_err(), "build should fail without prefix");
        assert!(result.unwrap_err().to_string().contains("prefix"));
    }

    #[test]
    fn should_fail_folder_build_when_local_folder_missing() {
        let result = S3PutFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .prefix("backups/")
            .build();
        assert!(result.is_err(), "build should fail without local_folder");
        assert!(result.unwrap_err().to_string().contains("local_folder"));
    }

    #[test]
    fn should_build_folder_with_required_fields() {
        let result = S3PutFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .prefix("backups/")
            .local_folder("/tmp/exports")
            .build();
        assert!(result.is_ok(), "build should succeed with required fields: {:?}", result.err());
    }

    // --- collect_files helper tests ---

    #[test]
    fn should_collect_files_from_directory() {
        let dir = temp_dir().join("spring_batch_rs_test_collect");
        fs::remove_dir_all(&dir).ok(); // clean up any previous run
        fs::create_dir_all(&dir).unwrap(); // test setup — cannot fail in temp dir
        fs::write(dir.join("a.txt"), "a").unwrap(); // test setup
        fs::write(dir.join("b.txt"), "b").unwrap(); // test setup

        let files = collect_files(&dir).unwrap(); // dir exists — cannot fail
        assert_eq!(files.len(), 2, "should collect 2 files, got: {:?}", files);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn should_collect_files_from_nested_directories() {
        let dir = temp_dir().join("spring_batch_rs_test_collect_nested");
        let sub = dir.join("sub");
        fs::remove_dir_all(&dir).ok(); // clean up any previous run
        fs::create_dir_all(&sub).unwrap(); // test setup — cannot fail in temp dir
        fs::write(dir.join("root.txt"), "r").unwrap(); // test setup
        fs::write(sub.join("child.txt"), "c").unwrap(); // test setup

        let files = collect_files(&dir).unwrap(); // dir exists — cannot fail
        assert_eq!(files.len(), 2, "should collect files from nested dirs: {:?}", files);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn should_return_error_for_missing_directory() {
        let result = collect_files(Path::new("/nonexistent/path/xyz"));
        assert!(result.is_err(), "should return error for missing directory");
    }
}
