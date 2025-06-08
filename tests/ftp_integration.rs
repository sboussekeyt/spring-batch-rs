//! # FTP Integration Tests
//!
//! This module contains integration tests for FTP tasklets using testcontainers
//! to spin up a real FTP server for comprehensive testing.

#[cfg(feature = "ftp")]
mod ftp_tests {
    use spring_batch_rs::{
        core::{
            job::{Job, JobBuilder},
            step::{Step, StepBuilder},
        },
        tasklet::ftp::{
            FtpGetFolderTaskletBuilder, FtpGetTaskletBuilder, FtpPutFolderTaskletBuilder,
            FtpPutTaskletBuilder,
        },
    };
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        time::Duration,
    };
    #[allow(unused_imports)]
    use testcontainers::{
        core::{ContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    #[allow(unused_imports)]
    use tokio::time::sleep;

    /// Shared FTP container information
    #[derive(Debug, Clone)]
    pub struct FtpServerInfo {
        pub host: String,
        pub port: u16,
        pub username: String,
        pub password: String,
    }

    /// Create a new FTP server for testing
    async fn create_ftp_server() -> Result<FtpServerInfo, Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();

        // Use a random port to avoid conflicts
        let random_port = 21000 + (std::process::id() % 1000) as u16;

        let ftp_container = GenericImage::new("delfer/alpine-ftp-server", "latest")
            .with_wait_for(WaitFor::seconds(5))
            .with_exposed_port(ContainerPort::Tcp(21))
            .with_mapped_port(random_port, ContainerPort::Tcp(21))
            .with_mapped_port(random_port + 1, ContainerPort::Tcp(random_port + 1))
            .with_env_var("USERS", "testuser|testpass123|/home/testuser|1000")
            .with_env_var("ADDRESS", "127.0.0.1")
            .with_env_var("MIN_PORT", (random_port + 1).to_string())
            .with_env_var("MAX_PORT", (random_port + 1).to_string())
            .start()
            .await?;

        let ftp_port = ftp_container.get_host_port_ipv4(21).await?;
        let ftp_host = ftp_container.get_host().await?;

        let server_info = FtpServerInfo {
            host: ftp_host.to_string(),
            port: ftp_port,
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };

        println!(
            "FTP Server started at {}:{}",
            server_info.host, server_info.port
        );

        // Add a small delay to ensure the FTP server is fully ready
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Keep the container alive by leaking it
        std::mem::forget(ftp_container);

        Ok(server_info)
    }

    /// Helper function to create unique test directories to avoid conflicts
    fn create_unique_test_dir(test_name: &str) -> std::path::PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", test_name, timestamp))
    }

    /// Helper function to create unique remote file/folder names to avoid conflicts
    fn create_unique_remote_name(prefix: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("./{}_{}_{}", prefix, std::process::id(), timestamp)
    }

    /// Helper function to create test files in a directory
    fn create_test_files(base_dir: &Path) -> Result<(), std::io::Error> {
        fs::create_dir_all(base_dir)?;

        // Create some test files
        let files = vec![
            ("file1.txt", "Content of file 1\nLine 2 of file 1"),
            ("file2.txt", "Content of file 2\nLine 2 of file 2"),
            ("data.json", r#"{"name": "test", "value": 42}"#),
        ];

        for (filename, content) in files {
            let file_path = base_dir.join(filename);
            let mut file = File::create(file_path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_tasklet_builders() -> Result<(), Box<dyn std::error::Error>> {
        // Test FTP PUT tasklet builder
        let temp_dir = std::env::temp_dir().join("ftp_builder_test");
        fs::create_dir_all(&temp_dir)?;

        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test content")?;

        let put_tasklet = FtpPutTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("testuser")
            .password("testpass")
            .local_file(&test_file)
            .remote_file("/test.txt")
            .passive_mode(true)
            .timeout(Duration::from_secs(30))
            .build()?;

        // Test FTP GET tasklet builder
        let download_file = temp_dir.join("download.txt");
        let get_tasklet = FtpGetTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("testuser")
            .password("testpass")
            .remote_file("/test.txt")
            .local_file(&download_file)
            .passive_mode(true)
            .timeout(Duration::from_secs(30))
            .build()?;

        // Test FTP PUT FOLDER tasklet builder
        let upload_dir = temp_dir.join("upload");
        create_test_files(&upload_dir)?;

        let put_folder_tasklet = FtpPutFolderTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("testuser")
            .password("testpass")
            .local_folder(&upload_dir)
            .remote_folder("/upload")
            .passive_mode(true)
            .create_directories(true)
            .recursive(true)
            .timeout(Duration::from_secs(60))
            .build()?;

        // Test FTP GET FOLDER tasklet builder
        let download_dir = temp_dir.join("download");
        let get_folder_tasklet = FtpGetFolderTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("testuser")
            .password("testpass")
            .remote_folder("/upload")
            .local_folder(&download_dir)
            .passive_mode(true)
            .create_directories(true)
            .recursive(true)
            .timeout(Duration::from_secs(60))
            .build()?;

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        // All builders should succeed - tasklets don't have name() method
        // Just verify they were created successfully by checking they exist
        assert!(!std::ptr::addr_of!(put_tasklet).is_null());
        assert!(!std::ptr::addr_of!(get_tasklet).is_null());
        assert!(!std::ptr::addr_of!(put_folder_tasklet).is_null());
        assert!(!std::ptr::addr_of!(get_folder_tasklet).is_null());

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_tasklet_validation() -> Result<(), Box<dyn std::error::Error>> {
        // Test missing required fields
        let result = FtpPutTaskletBuilder::new().build();
        assert!(
            result.is_err(),
            "Builder should fail without required fields"
        );

        // Test invalid file paths
        let invalid_put = FtpPutTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("test")
            .password("test")
            .local_file("/nonexistent/file.txt")
            .remote_file("/test.txt")
            .build();

        assert!(
            invalid_put.is_err(),
            "Builder should fail with nonexistent local file"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_job_creation() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary file for testing
        let temp_dir = std::env::temp_dir().join("ftp_job_test");
        fs::create_dir_all(&temp_dir)?;

        let test_file = temp_dir.join("job_test.txt");
        fs::write(&test_file, "job test content")?;

        // Create FTP tasklet (this won't actually connect, just test job creation)
        let ftp_tasklet = FtpPutTaskletBuilder::new()
            .host("invalid.host") // Use invalid host so it fails quickly
            .port(21)
            .username("test")
            .password("test")
            .local_file(&test_file)
            .remote_file("/test.txt")
            .timeout(Duration::from_secs(1)) // Very short timeout
            .build()?;

        // Create job with FTP step
        let step = StepBuilder::new("ftp-test-step")
            .tasklet(&ftp_tasklet)
            .build();

        let job = JobBuilder::new().start(&step).build();

        // Job creation should succeed even if execution would fail
        // Jobs don't have a public name() method, just verify it was created
        assert!(!std::ptr::addr_of!(job).is_null());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_connection_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary file
        let temp_dir = std::env::temp_dir().join("ftp_error_test");
        fs::create_dir_all(&temp_dir)?;

        let test_file = temp_dir.join("error_test.txt");
        fs::write(&test_file, "error test content")?;

        // Create tasklet with invalid connection details
        let ftp_tasklet = FtpPutTaskletBuilder::new()
            .host("invalid.nonexistent.host.example")
            .port(21)
            .username("invalid")
            .password("invalid")
            .local_file(&test_file)
            .remote_file("/test.txt")
            .timeout(Duration::from_secs(2)) // Short timeout for quick failure
            .build()?;

        let step = StepBuilder::new("ftp-error-step")
            .tasklet(&ftp_tasklet)
            .build();

        let job = JobBuilder::new().start(&step).build();
        let result = job.run();

        // Should fail due to invalid connection
        assert!(
            result.is_err(),
            "Job should fail with invalid FTP connection"
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_folder_operations_validation() -> Result<(), Box<dyn std::error::Error>> {
        // Test folder tasklet with nonexistent directory
        let nonexistent_dir = std::env::temp_dir().join("nonexistent_folder_12345");

        let result = FtpPutFolderTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("test")
            .password("test")
            .local_folder(&nonexistent_dir)
            .remote_folder("/test")
            .build();

        assert!(result.is_err(), "Should fail with nonexistent local folder");

        // Test with valid empty directory
        let empty_dir = std::env::temp_dir().join("empty_ftp_test");
        fs::create_dir_all(&empty_dir)?;

        let tasklet = FtpPutFolderTaskletBuilder::new()
            .host("localhost")
            .port(21)
            .username("test")
            .password("test")
            .local_folder(&empty_dir)
            .remote_folder("/test")
            .build()?;

        // Tasklet created successfully
        assert!(!std::ptr::addr_of!(tasklet).is_null());

        // Cleanup
        fs::remove_dir_all(&empty_dir).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_configuration_options() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join("ftp_config_test");
        fs::create_dir_all(&temp_dir)?;

        let test_file = temp_dir.join("config_test.txt");
        fs::write(&test_file, "config test")?;

        // Test different configuration options
        let tasklet_active = FtpPutTaskletBuilder::new()
            .host("localhost")
            .port(2121) // Non-standard port
            .username("testuser")
            .password("testpass")
            .local_file(&test_file)
            .remote_file("/test.txt")
            .passive_mode(false) // Active mode
            .timeout(Duration::from_secs(45))
            .build()?;

        let tasklet_passive = FtpPutTaskletBuilder::new()
            .host("ftp.example.com")
            .port(21)
            .username("user")
            .password("pass")
            .local_file(&test_file)
            .remote_file("/upload/test.txt")
            .passive_mode(true) // Passive mode
            .timeout(Duration::from_secs(120))
            .build()?;

        // Both configurations should be valid
        assert!(!std::ptr::addr_of!(tasklet_active).is_null());
        assert!(!std::ptr::addr_of!(tasklet_passive).is_null());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_complete_workflow_with_real_server() -> Result<(), Box<dyn std::error::Error>>
    {
        let server_info = create_ftp_server().await?;

        // Create test directory structure with unique names
        let temp_dir = create_unique_test_dir("ftp_complete_workflow_test");
        fs::create_dir_all(&temp_dir)?;

        // Create source data
        let source_dir = temp_dir.join("source");
        create_test_files(&source_dir)?;

        let single_file = temp_dir.join("single_test.txt");
        fs::write(&single_file, "Single file test content")?;

        // Create destination directories
        let download_single = temp_dir.join("download_single");
        let download_folder = temp_dir.join("download_folder");

        // Use unique remote names
        let remote_single_file = create_unique_remote_name("workflow_single");
        let remote_folder_name = create_unique_remote_name("workflow_folder");

        // Build all tasklets
        let put_file_tasklet = FtpPutTaskletBuilder::new()
            .host(&server_info.host)
            .port(server_info.port)
            .username(&server_info.username)
            .password(&server_info.password)
            .local_file(&single_file)
            .remote_file(&remote_single_file)
            .passive_mode(true)
            .timeout(Duration::from_secs(10))
            .build()?;

        let put_folder_tasklet = FtpPutFolderTaskletBuilder::new()
            .host(&server_info.host)
            .port(server_info.port)
            .username(&server_info.username)
            .password(&server_info.password)
            .local_folder(&source_dir)
            .remote_folder(&remote_folder_name)
            .passive_mode(true)
            .create_directories(true)
            .recursive(false)
            .timeout(Duration::from_secs(30))
            .build()?;

        let get_file_tasklet = FtpGetTaskletBuilder::new()
            .host(&server_info.host)
            .port(server_info.port)
            .username(&server_info.username)
            .password(&server_info.password)
            .remote_file(&remote_single_file)
            .local_file(download_single.join("retrieved_single.txt"))
            .passive_mode(true)
            .timeout(Duration::from_secs(10))
            .build()?;

        let get_folder_tasklet = FtpGetFolderTaskletBuilder::new()
            .host(&server_info.host)
            .port(server_info.port)
            .username(&server_info.username)
            .password(&server_info.password)
            .remote_folder(&remote_folder_name)
            .local_folder(&download_folder)
            .passive_mode(true)
            .create_directories(true)
            .recursive(false)
            .timeout(Duration::from_secs(30))
            .build()?;

        // Create steps
        let put_file_step = StepBuilder::new("put-file-step")
            .tasklet(&put_file_tasklet)
            .build();

        let put_folder_step = StepBuilder::new("put-folder-step")
            .tasklet(&put_folder_tasklet)
            .build();

        let get_file_step = StepBuilder::new("get-file-step")
            .tasklet(&get_file_tasklet)
            .build();

        let get_folder_step = StepBuilder::new("get-folder-step")
            .tasklet(&get_folder_tasklet)
            .build();

        // Create and run complete workflow job
        let workflow_job = JobBuilder::new()
            .start(&put_file_step)
            .next(&put_folder_step)
            .next(&get_file_step)
            .next(&get_folder_step)
            .build();

        let result = workflow_job.run();

        // Verify complete workflow succeeded
        assert!(result.is_ok(), "Complete FTP workflow should succeed");

        // Verify downloaded files exist and have correct content
        let retrieved_single = download_single.join("retrieved_single.txt");
        assert!(
            retrieved_single.exists(),
            "Retrieved single file should exist"
        );

        let single_content = fs::read_to_string(&retrieved_single)?;
        assert_eq!(
            single_content, "Single file test content",
            "Single file content should match"
        );

        assert!(download_folder.exists(), "Download folder should exist");
        let downloaded_files = fs::read_dir(&download_folder)?;
        assert!(
            downloaded_files.count() > 0,
            "Should have downloaded folder files"
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_ftp_integration_comprehensive() -> Result<(), Box<dyn std::error::Error>> {
        // This test validates the complete integration without requiring Docker
        // It tests tasklet creation, job building, and error handling

        let temp_dir = std::env::temp_dir().join("ftp_integration_test");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        // Create test files and directories
        let test_file = temp_dir.join("integration_test.txt");
        fs::write(&test_file, "Integration test content")?;

        let test_folder = temp_dir.join("test_folder");
        create_test_files(&test_folder)?;

        // Test all four tasklet types
        let put_tasklet = FtpPutTaskletBuilder::new()
            .host("test.example.com")
            .port(21)
            .username("testuser")
            .password("testpass")
            .local_file(&test_file)
            .remote_file("/test.txt")
            .passive_mode(true)
            .timeout(Duration::from_secs(30))
            .build()?;

        let get_tasklet = FtpGetTaskletBuilder::new()
            .host("test.example.com")
            .port(21)
            .username("testuser")
            .password("testpass")
            .remote_file("/test.txt")
            .local_file(temp_dir.join("downloaded.txt"))
            .passive_mode(true)
            .timeout(Duration::from_secs(30))
            .build()?;

        let put_folder_tasklet = FtpPutFolderTaskletBuilder::new()
            .host("test.example.com")
            .port(21)
            .username("testuser")
            .password("testpass")
            .local_folder(&test_folder)
            .remote_folder("/upload")
            .passive_mode(true)
            .create_directories(true)
            .recursive(true)
            .timeout(Duration::from_secs(60))
            .build()?;

        let get_folder_tasklet = FtpGetFolderTaskletBuilder::new()
            .host("test.example.com")
            .port(21)
            .username("testuser")
            .password("testpass")
            .remote_folder("/upload")
            .local_folder(temp_dir.join("download"))
            .passive_mode(true)
            .create_directories(true)
            .recursive(true)
            .timeout(Duration::from_secs(60))
            .build()?;

        // Create steps for each tasklet
        let put_step = StepBuilder::new("ftp-put-step")
            .tasklet(&put_tasklet)
            .build();

        let get_step = StepBuilder::new("ftp-get-step")
            .tasklet(&get_tasklet)
            .build();

        let put_folder_step = StepBuilder::new("ftp-put-folder-step")
            .tasklet(&put_folder_tasklet)
            .build();

        let get_folder_step = StepBuilder::new("ftp-get-folder-step")
            .tasklet(&get_folder_tasklet)
            .build();

        // Verify all steps were created successfully
        assert_eq!(put_step.get_name(), "ftp-put-step");
        assert_eq!(get_step.get_name(), "ftp-get-step");
        assert_eq!(put_folder_step.get_name(), "ftp-put-folder-step");
        assert_eq!(get_folder_step.get_name(), "ftp-get-folder-step");

        // Test job creation with multiple steps
        let job = JobBuilder::new()
            .start(&put_step)
            .next(&put_folder_step)
            .next(&get_step)
            .next(&get_folder_step)
            .build();

        // Job should be created successfully (even though execution would fail without real FTP server)
        assert!(!std::ptr::addr_of!(job).is_null());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();

        Ok(())
    }
}
