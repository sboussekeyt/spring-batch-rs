//! # S3 Integration Tests
//!
//! Integration tests for S3 tasklets using a LocalStack container as the S3 backend.
//! Tests require Docker to be running.

#[cfg(feature = "s3")]
mod s3_tests {
    use aws_config::BehaviorVersion;
    use aws_sdk_s3::config::Credentials;
    use spring_batch_rs::{
        core::{
            job::{Job, JobBuilder},
            step::StepBuilder,
        },
        tasklet::s3::{
            get::{S3GetFolderTaskletBuilder, S3GetTaskletBuilder},
            put::{S3PutFolderTaskletBuilder, S3PutTaskletBuilder},
        },
    };
    use std::{env::temp_dir, fs};
    use testcontainers_modules::{localstack::LocalStack, testcontainers::runners::AsyncRunner};

    const REGION: &str = "us-east-1";
    const ACCESS_KEY: &str = "test";
    const SECRET_KEY: &str = "test";

    static LOCALSTACK_ENDPOINT: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

    /// Return the shared LocalStack endpoint URL, starting the container on first call.
    /// All tests share one container to avoid port-binding races under parallel execution.
    async fn localstack_endpoint() -> &'static str {
        LOCALSTACK_ENDPOINT
            .get_or_init(|| async {
                let container = LocalStack::default().start().await.unwrap();
                let host = container.get_host().await.unwrap();
                let port = container.get_host_port_ipv4(4566).await.unwrap();
                let endpoint_url = format!("http://{}:{}", host, port);
                // Intentionally leak: the container must outlive all tests.
                std::mem::forget(container);
                endpoint_url
            })
            .await
    }

    /// Create a bucket in the LocalStack container.
    ///
    /// Must use `force_path_style` (same as `build_s3_client`) so that requests
    /// hit `http://host:port/bucket` instead of `http://bucket.host:port/`.
    async fn create_bucket(endpoint_url: &str, bucket: &str) {
        let creds = Credentials::new(ACCESS_KEY, SECRET_KEY, None, None, "test");
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(REGION)
            .credentials_provider(creds)
            .endpoint_url(endpoint_url)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(true)
            .build();
        let client = aws_sdk_s3::Client::from_conf(s3_config);
        client.create_bucket().bucket(bucket).send().await.unwrap();
    }

    /// Return a unique temp path to avoid cross-test interference.
    fn tmp(name: &str) -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir().join(format!("s3_test_{name}_{ts}"))
    }

    // -------------------------------------------------------------------------

    /// Upload a small CSV and download it back; verify the content is identical.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_put_and_get_single_file() {
        let endpoint_url = localstack_endpoint().await;
        let bucket = "test-put-get";
        create_bucket(&endpoint_url, bucket).await;

        let upload_path = tmp("upload.csv");
        let download_path = tmp("download.csv");
        let content = "id,name\n1,Alice\n2,Bob\n";
        fs::write(&upload_path, content).unwrap();

        let put_tasklet = S3PutTaskletBuilder::new()
            .bucket(bucket)
            .key("single/file.csv")
            .local_file(&upload_path)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let get_tasklet = S3GetTaskletBuilder::new()
            .bucket(bucket)
            .key("single/file.csv")
            .local_file(&download_path)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let upload_step = StepBuilder::new("s3-upload").tasklet(&put_tasklet).build();
        let download_step = StepBuilder::new("s3-download")
            .tasklet(&get_tasklet)
            .build();

        let job = JobBuilder::new()
            .start(&upload_step)
            .next(&download_step)
            .build();
        job.run().unwrap();

        let downloaded = fs::read_to_string(&download_path).unwrap();
        assert_eq!(
            downloaded, content,
            "downloaded content should match the uploaded content"
        );

        fs::remove_file(&upload_path).ok();
        fs::remove_file(&download_path).ok();
    }

    /// Upload a file that exceeds chunk_size (triggers multipart upload), then
    /// download it and verify size is preserved.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_multipart_upload_and_download() {
        let endpoint_url = localstack_endpoint().await;
        let bucket = "test-multipart";
        create_bucket(&endpoint_url, bucket).await;

        // 6 MiB file with 5 MiB chunk_size → 2 parts (1 full + 1 remainder)
        let chunk_size = 5 * 1024 * 1024_usize;
        let file_size = 6 * 1024 * 1024_usize;
        let upload_path = tmp("large_upload.bin");
        let download_path = tmp("large_download.bin");
        fs::write(&upload_path, vec![b'X'; file_size]).unwrap();

        let put_tasklet = S3PutTaskletBuilder::new()
            .bucket(bucket)
            .key("large/file.bin")
            .local_file(&upload_path)
            .chunk_size(chunk_size)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let get_tasklet = S3GetTaskletBuilder::new()
            .bucket(bucket)
            .key("large/file.bin")
            .local_file(&download_path)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let upload_step = StepBuilder::new("s3-multipart-upload")
            .tasklet(&put_tasklet)
            .build();
        let download_step = StepBuilder::new("s3-multipart-download")
            .tasklet(&get_tasklet)
            .build();

        let job = JobBuilder::new()
            .start(&upload_step)
            .next(&download_step)
            .build();
        job.run().unwrap();

        let downloaded = fs::read(&download_path).unwrap();
        assert_eq!(
            downloaded.len(),
            file_size,
            "downloaded file should be the same size as the uploaded file"
        );
        assert!(
            downloaded.iter().all(|&b| b == b'X'),
            "all bytes should be 'X'"
        );

        fs::remove_file(&upload_path).ok();
        fs::remove_file(&download_path).ok();
    }

    /// Upload a folder with multiple files and download it back; verify that
    /// every file is present and has the correct content.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_put_and_get_folder() {
        let endpoint_url = localstack_endpoint().await;
        let bucket = "test-folder";
        create_bucket(&endpoint_url, bucket).await;

        // Prepare a local folder with two files
        let upload_folder = tmp("upload_folder");
        fs::create_dir_all(&upload_folder).unwrap();
        fs::write(upload_folder.join("a.txt"), "file A content").unwrap();
        fs::write(upload_folder.join("b.txt"), "file B content").unwrap();

        let download_folder = tmp("download_folder");

        let put_tasklet = S3PutFolderTaskletBuilder::new()
            .bucket(bucket)
            .prefix("folder/")
            .local_folder(&upload_folder)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let get_tasklet = S3GetFolderTaskletBuilder::new()
            .bucket(bucket)
            .prefix("folder/")
            .local_folder(&download_folder)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let upload_step = StepBuilder::new("s3-folder-upload")
            .tasklet(&put_tasklet)
            .build();
        let download_step = StepBuilder::new("s3-folder-download")
            .tasklet(&get_tasklet)
            .build();

        let job = JobBuilder::new()
            .start(&upload_step)
            .next(&download_step)
            .build();
        job.run().unwrap();

        let a = fs::read_to_string(download_folder.join("a.txt")).unwrap();
        let b = fs::read_to_string(download_folder.join("b.txt")).unwrap();
        assert_eq!(a, "file A content", "a.txt content should match");
        assert_eq!(b, "file B content", "b.txt content should match");

        fs::remove_dir_all(&upload_folder).ok();
        fs::remove_dir_all(&download_folder).ok();
    }

    /// Upload a folder that contains a nested subdirectory; verify that the
    /// directory structure is preserved after download.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_put_and_get_nested_folder() {
        let endpoint_url = localstack_endpoint().await;
        let bucket = "test-nested";
        create_bucket(&endpoint_url, bucket).await;

        let upload_folder = tmp("upload_nested");
        let sub = upload_folder.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(upload_folder.join("root.csv"), "root").unwrap();
        fs::write(sub.join("child.csv"), "child").unwrap();

        let download_folder = tmp("download_nested");

        let put_tasklet = S3PutFolderTaskletBuilder::new()
            .bucket(bucket)
            .prefix("nested/")
            .local_folder(&upload_folder)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let get_tasklet = S3GetFolderTaskletBuilder::new()
            .bucket(bucket)
            .prefix("nested/")
            .local_folder(&download_folder)
            .endpoint_url(endpoint_url)
            .access_key_id(ACCESS_KEY)
            .secret_access_key(SECRET_KEY)
            .region(REGION)
            .build()
            .unwrap();

        let upload_step = StepBuilder::new("s3-nested-upload")
            .tasklet(&put_tasklet)
            .build();
        let download_step = StepBuilder::new("s3-nested-download")
            .tasklet(&get_tasklet)
            .build();

        let job = JobBuilder::new()
            .start(&upload_step)
            .next(&download_step)
            .build();
        job.run().unwrap();

        assert_eq!(
            fs::read_to_string(download_folder.join("root.csv")).unwrap(),
            "root"
        );
        assert_eq!(
            fs::read_to_string(download_folder.join("sub").join("child.csv")).unwrap(),
            "child"
        );

        fs::remove_dir_all(&upload_folder).ok();
        fs::remove_dir_all(&download_folder).ok();
    }
}
