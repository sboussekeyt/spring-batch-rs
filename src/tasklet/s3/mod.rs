//! # S3 Tasklet
//!
//! This module provides tasklets for Amazon S3 file transfer operations (put and get).
//! It is designed to be similar to Spring Batch's S3 capabilities for batch file transfers.
//!
//! ## Features
//!
//! - S3 PUT operations (upload local files to S3)
//! - S3 GET operations (download S3 objects to local files, streaming)
//! - S3 PUT FOLDER operations (upload entire local folder to an S3 prefix)
//! - S3 GET FOLDER operations (download all objects under an S3 prefix to a local folder)
//! - Explicit credential configuration (access key / secret key)
//! - AWS default credential chain (environment variables, `~/.aws/credentials`, IAM role)
//! - Custom endpoint URL for S3-compatible services (MinIO, LocalStack)
//! - Configurable multipart upload chunk size
//!
//! ## Examples
//!
//! ### S3 PUT Operation
//!
//! ```rust,no_run
//! use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let tasklet = S3PutTaskletBuilder::new()
//!     .bucket("my-bucket")
//!     .key("exports/file.csv")
//!     .local_file("./output/file.csv")
//!     .region("eu-west-1")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### S3 GET Operation
//!
//! ```rust,no_run
//! use spring_batch_rs::tasklet::s3::get::S3GetTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let tasklet = S3GetTaskletBuilder::new()
//!     .bucket("my-bucket")
//!     .key("imports/file.csv")
//!     .local_file("./input/file.csv")
//!     .region("eu-west-1")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### S3 with MinIO (custom endpoint)
//!
//! ```rust,no_run
//! use spring_batch_rs::tasklet::s3::put::S3PutTaskletBuilder;
//!
//! # fn example() -> Result<(), spring_batch_rs::BatchError> {
//! let tasklet = S3PutTaskletBuilder::new()
//!     .bucket("my-bucket")
//!     .key("file.csv")
//!     .local_file("./output/file.csv")
//!     .endpoint_url("http://localhost:9000")
//!     .access_key_id("minioadmin")
//!     .secret_access_key("minioadmin")
//!     .build()?;
//! # Ok(())
//! # }
//! ```

pub mod get;
pub mod put;

use crate::BatchError;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::config::Credentials;

/// Configuration for connecting to an S3-compatible service.
///
/// All fields are optional. When `access_key_id` and `secret_access_key` are both
/// `None`, the AWS default credential chain is used (environment variables,
/// `~/.aws/credentials`, IAM instance role).
///
/// Set `endpoint_url` to connect to S3-compatible services such as MinIO or LocalStack.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::tasklet::s3::S3ClientConfig;
///
/// let config = S3ClientConfig {
///     region: Some("eu-west-1".to_string()),
///     endpoint_url: None,
///     access_key_id: None,
///     secret_access_key: None,
/// };
/// assert_eq!(config.region.as_deref(), Some("eu-west-1"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct S3ClientConfig {
    /// AWS region (e.g. `"eu-west-1"`). Falls back to `AWS_DEFAULT_REGION` env var when `None`.
    pub region: Option<String>,
    /// Custom endpoint URL for S3-compatible services (e.g. `"http://localhost:9000"` for MinIO).
    pub endpoint_url: Option<String>,
    /// AWS access key ID. Uses default credential chain when `None`.
    pub access_key_id: Option<String>,
    /// AWS secret access key. Uses default credential chain when `None`.
    pub secret_access_key: Option<String>,
}

/// Builds an [`aws_sdk_s3::Client`] from the given [`S3ClientConfig`].
///
/// When both `access_key_id` and `secret_access_key` are set, explicit static
/// credentials are used. Otherwise the AWS default credential chain applies.
///
/// # Errors
///
/// Returns [`BatchError::Configuration`] if the AWS SDK configuration cannot be loaded.
pub(crate) async fn build_s3_client(
    config: &S3ClientConfig,
) -> Result<aws_sdk_s3::Client, BatchError> {
    let region_provider =
        RegionProviderChain::first_try(config.region.clone().map(aws_sdk_s3::config::Region::new))
            .or_default_provider();

    let sdk_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let mut builder = S3ConfigBuilder::from(&sdk_config);

    if let Some(url) = &config.endpoint_url {
        builder = builder.endpoint_url(url).force_path_style(true);
    }

    if let (Some(key_id), Some(secret)) = (&config.access_key_id, &config.secret_access_key) {
        let creds = Credentials::new(key_id, secret, None, None, "static");
        builder = builder.credentials_provider(creds);
    }

    Ok(aws_sdk_s3::Client::from_conf(builder.build()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_to_none_fields() {
        let config = S3ClientConfig::default();
        assert!(config.region.is_none(), "region should default to None");
        assert!(
            config.endpoint_url.is_none(),
            "endpoint_url should default to None"
        );
        assert!(
            config.access_key_id.is_none(),
            "access_key_id should default to None"
        );
        assert!(
            config.secret_access_key.is_none(),
            "secret_access_key should default to None"
        );
    }

    #[test]
    fn should_store_region() {
        let config = S3ClientConfig {
            region: Some("us-east-1".to_string()),
            ..Default::default()
        };
        assert_eq!(config.region.as_deref(), Some("us-east-1"));
    }

    #[test]
    fn should_store_endpoint_url() {
        let config = S3ClientConfig {
            endpoint_url: Some("http://localhost:9000".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.endpoint_url.as_deref(),
            Some("http://localhost:9000")
        );
    }

    #[test]
    fn should_store_explicit_credentials() {
        let config = S3ClientConfig {
            access_key_id: Some("AKID".to_string()),
            secret_access_key: Some("SECRET".to_string()),
            ..Default::default()
        };
        assert_eq!(config.access_key_id.as_deref(), Some("AKID"));
        assert_eq!(config.secret_access_key.as_deref(), Some("SECRET"));
    }
}
