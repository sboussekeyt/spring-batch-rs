// placeholder — full implementation in Task 4
use crate::BatchError;
use super::S3ClientConfig;

/// Builder for [`S3PutTasklet`].
///
/// # Examples
///
/// ```rust,no_run
/// use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
///
/// # fn example() -> Result<(), spring_batch_rs::BatchError> {
/// let _tasklet = S3PutTaskletBuilder::new()
///     .bucket("my-bucket")
///     .key("file.csv")
///     .local_file("./output/file.csv")
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct S3PutTaskletBuilder {
    config: S3ClientConfig,
    bucket: Option<String>,
    key: Option<String>,
    local_file: Option<String>,
}

impl S3PutTaskletBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self {
            config: S3ClientConfig::default(),
            bucket: None,
            key: None,
            local_file: None,
        }
    }

    /// Sets the S3 bucket name.
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Sets the S3 object key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Sets the local file path to upload.
    pub fn local_file(mut self, path: impl Into<String>) -> Self {
        self.local_file = Some(path.into());
        self
    }

    /// Sets the AWS region.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.config.region = Some(region.into());
        self
    }

    /// Sets a custom endpoint URL.
    pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.config.endpoint_url = Some(url.into());
        self
    }

    /// Sets the AWS access key ID.
    pub fn access_key_id(mut self, key: impl Into<String>) -> Self {
        self.config.access_key_id = Some(key.into());
        self
    }

    /// Sets the AWS secret access key.
    pub fn secret_access_key(mut self, secret: impl Into<String>) -> Self {
        self.config.secret_access_key = Some(secret.into());
        self
    }

    /// Builds the tasklet.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Configuration`] if required fields are missing.
    pub fn build(self) -> Result<S3PutTasklet, BatchError> {
        let bucket = self.bucket.ok_or_else(|| BatchError::Configuration("bucket is required".to_string()))?;
        let key = self.key.ok_or_else(|| BatchError::Configuration("key is required".to_string()))?;
        let local_file = self.local_file.ok_or_else(|| BatchError::Configuration("local_file is required".to_string()))?;
        Ok(S3PutTasklet { config: self.config, bucket, key, local_file })
    }
}

impl Default for S3PutTaskletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Tasklet that uploads a local file to S3. (Placeholder — full implementation in Task 4.)
#[allow(dead_code)]
pub struct S3PutTasklet {
    config: S3ClientConfig,
    bucket: String,
    key: String,
    local_file: String,
}
