//! S3 GET tasklets for downloading files and folders from Amazon S3.

use crate::{
    core::step::{RepeatStatus, StepExecution, Tasklet},
    tasklet::s3::{build_s3_client, S3ClientConfig},
    BatchError,
};
use log::{debug, info};
use std::path::{Path, PathBuf};
use tokio::runtime::Handle;

/// A tasklet that downloads a single S3 object to a local file.
///
/// The object body is collected into memory before being written to the local file.
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3GetTaskletBuilder::new()
///     .bucket("my-bucket")
///     .key("imports/file.csv")
///     .local_file("./input/file.csv")
///     .region("eu-west-1")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::ItemReader`] if the S3 download fails.
/// Returns [`BatchError::Io`] if the local file cannot be written.
#[derive(Debug)]
pub struct S3GetTasklet {
    bucket: String,
    key: String,
    local_file: PathBuf,
    config: S3ClientConfig,
}

impl S3GetTasklet {
    async fn execute_async(&self) -> Result<RepeatStatus, BatchError> {
        info!(
            "Downloading s3://{}/{} -> {}",
            self.bucket,
            self.key,
            self.local_file.display()
        );

        let client = build_s3_client(&self.config).await?;

        if let Some(parent) = self.local_file.parent() {
            std::fs::create_dir_all(parent).map_err(BatchError::Io)?;
        }

        let resp = client
            .get_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
            .map_err(|e| {
                BatchError::ItemReader(format!("S3 get_object failed for {}: {}", self.key, e))
            })?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| {
                BatchError::ItemReader(format!("Failed to read S3 body for {}: {}", self.key, e))
            })?
            .into_bytes();

        std::fs::write(&self.local_file, &bytes).map_err(BatchError::Io)?;

        info!(
            "Download complete: {} bytes written to {}",
            bytes.len(),
            self.local_file.display()
        );
        Ok(RepeatStatus::Finished)
    }
}

impl Tasklet for S3GetTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        tokio::task::block_in_place(|| Handle::current().block_on(self.execute_async()))
    }
}

/// Builder for [`S3GetTasklet`].
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3GetTaskletBuilder::new()
///     .bucket("my-bucket")
///     .key("imports/file.csv")
///     .local_file("./input/file.csv")
///     .region("eu-west-1")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::Configuration`] if `bucket`, `key`, or `local_file` are not set.
#[derive(Debug, Default)]
pub struct S3GetTaskletBuilder {
    bucket: Option<String>,
    key: Option<String>,
    local_file: Option<PathBuf>,
    config: S3ClientConfig,
}

impl S3GetTaskletBuilder {
    /// Creates a new builder with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the S3 bucket name.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().bucket("my-bucket");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().key("imports/file.csv");
    /// ```
    pub fn key<S: Into<String>>(mut self, key: S) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Sets the local file path to write the downloaded object to.
    ///
    /// Parent directories are created automatically during execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().local_file("./input/file.csv");
    /// ```
    pub fn local_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the AWS region.
    ///
    /// Falls back to the `AWS_REGION` environment variable (or `AWS_DEFAULT_REGION`)
    /// when not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().region("eu-west-1");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().endpoint_url("http://localhost:9000");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().access_key_id("AKIAIOSFODNN7EXAMPLE");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// let builder = S3GetTaskletBuilder::new().secret_access_key("wJalrXUtnFEMI/K7MDENG");
    /// ```
    pub fn secret_access_key<S: Into<String>>(mut self, secret: S) -> Self {
        self.config.secret_access_key = Some(secret.into());
        self
    }

    /// Builds the [`S3GetTasklet`].
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Configuration`] if `bucket`, `key`, or `local_file` are not set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// let tasklet = S3GetTaskletBuilder::new()
    ///     .bucket("my-bucket")
    ///     .key("file.csv")
    ///     .local_file("./input/file.csv")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<S3GetTasklet, BatchError> {
        let bucket = self.bucket.ok_or_else(|| {
            BatchError::Configuration("S3GetTasklet: 'bucket' is required".to_string())
        })?;
        let key = self.key.ok_or_else(|| {
            BatchError::Configuration("S3GetTasklet: 'key' is required".to_string())
        })?;
        let local_file = self.local_file.ok_or_else(|| {
            BatchError::Configuration("S3GetTasklet: 'local_file' is required".to_string())
        })?;

        Ok(S3GetTasklet {
            bucket,
            key,
            local_file,
            config: self.config,
        })
    }
}

// ---------------------------------------------------------------------------
// S3GetFolderTasklet
// ---------------------------------------------------------------------------

/// A tasklet that downloads all S3 objects under a given prefix to a local folder.
///
/// Objects are listed with `list_objects_v2` (with pagination support) and downloaded
/// sequentially. Parent directories are created automatically. If the prefix matches
/// no objects, the tasklet completes successfully with 0 files downloaded.
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3GetFolderTaskletBuilder::new()
///     .bucket("my-bucket")
///     .prefix("backups/2026-04-10/")
///     .local_folder("./imports/")
///     .region("eu-west-1")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::ItemReader`] if listing or downloading any object fails.
/// Returns [`BatchError::Io`] if writing any local file fails.
#[derive(Debug)]
pub struct S3GetFolderTasklet {
    bucket: String,
    prefix: String,
    local_folder: PathBuf,
    config: S3ClientConfig,
}

impl S3GetFolderTasklet {
    async fn execute_async(&self) -> Result<RepeatStatus, BatchError> {
        info!(
            "Downloading s3://{}/{} -> {}",
            self.bucket,
            self.prefix,
            self.local_folder.display()
        );

        let client = build_s3_client(&self.config).await?;
        std::fs::create_dir_all(&self.local_folder).map_err(BatchError::Io)?;

        let mut continuation_token: Option<String> = None;
        let mut total_files = 0usize;

        loop {
            let mut req = client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&self.prefix);

            if let Some(token) = continuation_token {
                req = req.continuation_token(token);
            }

            let list_resp = req
                .send()
                .await
                .map_err(|e| BatchError::ItemReader(format!("list_objects_v2 failed: {}", e)))?;

            for object in list_resp.contents() {
                let key = object.key().unwrap_or_default();
                // Strip prefix to get relative path within the local folder
                let relative = key.strip_prefix(self.prefix.as_str()).unwrap_or(key);
                if relative.is_empty() {
                    continue; // skip the prefix "directory" placeholder object
                }
                let local_path = self.local_folder.join(relative);

                if let Some(parent) = local_path.parent() {
                    std::fs::create_dir_all(parent).map_err(BatchError::Io)?;
                }

                debug!(
                    "Downloading s3://{}/{} -> {}",
                    self.bucket,
                    key,
                    local_path.display()
                );

                let resp = client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| {
                        BatchError::ItemReader(format!("get_object failed for {}: {}", key, e))
                    })?;

                let bytes = resp
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        BatchError::ItemReader(format!("Failed to read body for {}: {}", key, e))
                    })?
                    .into_bytes();

                std::fs::write(&local_path, &bytes).map_err(BatchError::Io)?;
                total_files += 1;
            }

            if list_resp.is_truncated().unwrap_or(false) {
                continuation_token = list_resp.next_continuation_token().map(str::to_string);
            } else {
                break;
            }
        }

        info!(
            "Folder download complete: {} files downloaded to {}",
            total_files,
            self.local_folder.display()
        );
        Ok(RepeatStatus::Finished)
    }
}

impl Tasklet for S3GetFolderTasklet {
    fn execute(&self, _step_execution: &StepExecution) -> Result<RepeatStatus, BatchError> {
        tokio::task::block_in_place(|| Handle::current().block_on(self.execute_async()))
    }
}

/// Builder for [`S3GetFolderTasklet`].
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let tasklet = S3GetFolderTaskletBuilder::new()
///     .bucket("my-bucket")
///     .prefix("backups/2026-04-10/")
///     .local_folder("./imports/")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`BatchError::Configuration`] if `bucket`, `prefix`, or `local_folder` are not set.
#[derive(Debug, Default)]
pub struct S3GetFolderTaskletBuilder {
    bucket: Option<String>,
    prefix: Option<String>,
    local_folder: Option<PathBuf>,
    config: S3ClientConfig,
}

impl S3GetFolderTaskletBuilder {
    /// Creates a new builder with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the S3 bucket name.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().bucket("my-bucket");
    /// ```
    pub fn bucket<S: Into<String>>(mut self, bucket: S) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Sets the S3 key prefix to list and download.
    ///
    /// All objects whose key starts with this prefix will be downloaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().prefix("backups/2026-04-10/");
    /// ```
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets the local folder path to write downloaded objects to.
    ///
    /// Created automatically if it does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().local_folder("./imports/");
    /// ```
    pub fn local_folder<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.local_folder = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the AWS region.
    ///
    /// Falls back to the `AWS_REGION` environment variable (or `AWS_DEFAULT_REGION`)
    /// when not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().region("eu-west-1");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().endpoint_url("http://localhost:9000");
    /// ```
    pub fn endpoint_url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.endpoint_url = Some(url.into());
        self
    }

    /// Sets the AWS access key ID for explicit credential configuration.
    ///
    /// Must be combined with [`secret_access_key`](Self::secret_access_key).
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().access_key_id("AKIAIOSFODNN7EXAMPLE");
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
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// let builder = S3GetFolderTaskletBuilder::new().secret_access_key("wJalrXUtnFEMI/K7MDENG");
    /// ```
    pub fn secret_access_key<S: Into<String>>(mut self, secret: S) -> Self {
        self.config.secret_access_key = Some(secret.into());
        self
    }

    /// Builds the [`S3GetFolderTasklet`].
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Configuration`] if `bucket`, `prefix`, or `local_folder` are not set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use spring_batch_rs::tasklet::s3::get::S3GetFolderTaskletBuilder;
    ///
    /// # fn example() -> Result<(), spring_batch_rs::BatchError> {
    /// let tasklet = S3GetFolderTaskletBuilder::new()
    ///     .bucket("my-bucket")
    ///     .prefix("backups/")
    ///     .local_folder("./imports/")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<S3GetFolderTasklet, BatchError> {
        let bucket = self.bucket.ok_or_else(|| {
            BatchError::Configuration("S3GetFolderTasklet: 'bucket' is required".to_string())
        })?;
        let prefix = self.prefix.ok_or_else(|| {
            BatchError::Configuration("S3GetFolderTasklet: 'prefix' is required".to_string())
        })?;
        let local_folder = self.local_folder.ok_or_else(|| {
            BatchError::Configuration("S3GetFolderTasklet: 'local_folder' is required".to_string())
        })?;

        Ok(S3GetFolderTasklet {
            bucket,
            prefix,
            local_folder,
            config: self.config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- S3GetTaskletBuilder tests ---

    #[test]
    fn should_fail_build_when_bucket_missing() {
        let result = S3GetTaskletBuilder::new()
            .key("file.csv")
            .local_file("/tmp/file.csv")
            .build();
        assert!(result.is_err(), "build should fail without bucket");
        assert!(result.unwrap_err().to_string().contains("bucket"));
    }

    #[test]
    fn should_fail_build_when_key_missing() {
        let result = S3GetTaskletBuilder::new()
            .bucket("my-bucket")
            .local_file("/tmp/file.csv")
            .build();
        assert!(result.is_err(), "build should fail without key");
        assert!(result.unwrap_err().to_string().contains("key"));
    }

    #[test]
    fn should_fail_build_when_local_file_missing() {
        let result = S3GetTaskletBuilder::new()
            .bucket("my-bucket")
            .key("file.csv")
            .build();
        assert!(result.is_err(), "build should fail without local_file");
        assert!(result.unwrap_err().to_string().contains("local_file"));
    }

    #[test]
    fn should_build_with_required_fields() {
        let result = S3GetTaskletBuilder::new()
            .bucket("my-bucket")
            .key("file.csv")
            .local_file("/tmp/file.csv")
            .build();
        assert!(
            result.is_ok(),
            "build should succeed with required fields: {:?}",
            result.err()
        );
    }

    #[test]
    fn should_store_optional_config_fields() {
        let tasklet = S3GetTaskletBuilder::new()
            .bucket("b")
            .key("k")
            .local_file("/tmp/f")
            .region("eu-west-1")
            .endpoint_url("http://localhost:9000")
            .access_key_id("AKID")
            .secret_access_key("SECRET")
            .build()
            .unwrap(); // required fields set — cannot fail
        assert_eq!(tasklet.config.region.as_deref(), Some("eu-west-1"));
        assert_eq!(
            tasklet.config.endpoint_url.as_deref(),
            Some("http://localhost:9000")
        );
        assert_eq!(tasklet.config.access_key_id.as_deref(), Some("AKID"));
        assert_eq!(tasklet.config.secret_access_key.as_deref(), Some("SECRET"));
    }

    // --- S3GetFolderTaskletBuilder tests ---

    #[test]
    fn should_fail_folder_build_when_bucket_missing() {
        let result = S3GetFolderTaskletBuilder::new()
            .prefix("backups/")
            .local_folder("/tmp/imports")
            .build();
        assert!(result.is_err(), "build should fail without bucket");
        assert!(result.unwrap_err().to_string().contains("bucket"));
    }

    #[test]
    fn should_fail_folder_build_when_prefix_missing() {
        let result = S3GetFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .local_folder("/tmp/imports")
            .build();
        assert!(result.is_err(), "build should fail without prefix");
        assert!(result.unwrap_err().to_string().contains("prefix"));
    }

    #[test]
    fn should_fail_folder_build_when_local_folder_missing() {
        let result = S3GetFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .prefix("backups/")
            .build();
        assert!(result.is_err(), "build should fail without local_folder");
        assert!(result.unwrap_err().to_string().contains("local_folder"));
    }

    #[test]
    fn should_build_folder_with_required_fields() {
        let result = S3GetFolderTaskletBuilder::new()
            .bucket("my-bucket")
            .prefix("backups/")
            .local_folder("/tmp/imports")
            .build();
        assert!(result.is_ok(), "build should succeed: {:?}", result.err());
    }
}
