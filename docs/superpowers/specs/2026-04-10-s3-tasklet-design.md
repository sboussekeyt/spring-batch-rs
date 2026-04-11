# S3 Tasklet Design

**Date:** 2026-04-10  
**Status:** Approved  
**Feature flag:** `s3`

## Overview

Add four S3 tasklets to `spring-batch-rs` for pure file transfer operations (upload/download), following the same pattern as the existing FTP tasklets. These tasklets are distinct from future S3 item readers/writers — they handle bulk file movement, not chunk-oriented processing.

## File Structure

```
src/tasklet/
├── mod.rs                    # add s3 module declaration
└── s3/
    ├── mod.rs                # public exports, S3ClientConfig, build_s3_client() helper
    ├── put.rs                # S3PutTasklet, S3PutTaskletBuilder
    │                         # S3PutFolderTasklet, S3PutFolderTaskletBuilder
    └── get.rs                # S3GetTasklet, S3GetTaskletBuilder
                              # S3GetFolderTasklet, S3GetFolderTaskletBuilder

Cargo.toml                    # feature "s3", aws-sdk-s3, aws-config, tokio-util deps
examples/tasklet_s3.rs        # upload + download example (no_run, LocalStack)
```

## Dependencies

```toml
[dependencies]
aws-sdk-s3 = { version = "1", optional = true }
aws-config = { version = "1", optional = true }
tokio-util = { version = "0.7", features = ["io"], optional = true }

[features]
s3 = ["dep:aws-sdk-s3", "dep:aws-config", "dep:tokio-util"]
full = [...existing..., "s3"]

[[example]]
name = "tasklet_s3"
required-features = ["s3"]
```

`tokio-util` is required to convert a local `AsyncRead` file into an AWS `ByteStream` for streaming uploads.

## Shared Infrastructure (`s3/mod.rs`)

```rust
pub struct S3ClientConfig {
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

pub(crate) async fn build_s3_client(config: &S3ClientConfig) -> aws_sdk_s3::Client { ... }
```

All four builders use `build_s3_client()` to avoid credential setup duplication.

## Public API

### `S3PutTasklet` — local file → S3

```rust
S3PutTaskletBuilder::new()
    .bucket("my-bucket")
    .key("prefix/file.csv")
    .local_file("./output/file.csv")
    .region("eu-west-1")                    // optional, falls back to AWS_DEFAULT_REGION
    .endpoint_url("http://localhost:9000")  // optional, for MinIO/LocalStack
    .access_key_id("...")                   // optional, falls back to AWS default chain
    .secret_access_key("...")               // optional
    .chunk_size(8 * 1024 * 1024)           // optional, default 8 MiB
    .build()?
```

### `S3GetTasklet` — S3 → local file (streaming)

```rust
S3GetTaskletBuilder::new()
    .bucket("my-bucket")
    .key("prefix/file.csv")
    .local_file("./input/file.csv")
    .region("eu-west-1")
    .endpoint_url("http://localhost:9000")
    .access_key_id("...")
    .secret_access_key("...")
    .build()?
```

### `S3PutFolderTasklet` — local folder → S3 prefix

```rust
S3PutFolderTaskletBuilder::new()
    .bucket("my-bucket")
    .prefix("backups/2026-04-10/")
    .local_folder("./exports/")
    .region("eu-west-1")
    .endpoint_url("http://localhost:9000")
    .access_key_id("...")
    .secret_access_key("...")
    .chunk_size(8 * 1024 * 1024)
    .build()?
```

### `S3GetFolderTasklet` — S3 prefix → local folder

```rust
S3GetFolderTaskletBuilder::new()
    .bucket("my-bucket")
    .prefix("backups/2026-04-10/")
    .local_folder("./imports/")
    .region("eu-west-1")
    .endpoint_url("http://localhost:9000")
    .access_key_id("...")
    .secret_access_key("...")
    .build()?
```

All four implement the `Tasklet` trait and return `RepeatStatus::Finished` on success or `BatchError` on failure.

## Upload Strategy (Put tasklets)

- Files **smaller than** `chunk_size`: single `put_object` call.
- Files **equal to or larger than** `chunk_size`: multipart upload:
  1. `create_multipart_upload`
  2. `upload_part` × N parts
  3. `complete_multipart_upload`
  4. On failure at any step: `abort_multipart_upload` to avoid orphaned S3 parts and unexpected costs.

Default `chunk_size`: **8 MiB** (AWS minimum for multipart parts is 5 MiB).

## Download Strategy (Get tasklets)

- `get_object` returns a `ByteStream` streamed chunk-by-chunk to the local file.
- `create_dir_all` is called before writing to ensure parent directories exist.
- For folder downloads, `list_objects_v2` with the given prefix enumerates all objects, then each is downloaded sequentially.

## Error Handling

No new `BatchError` variants. Existing variants are mapped as follows:

| Scenario | `BatchError` variant |
|---|---|
| Missing bucket/key, invalid credentials, bad endpoint | `Configuration` |
| Local file read/write failure, directory creation | `Io` |
| S3 upload failure (put, multipart part, complete) | `ItemWriter` |
| S3 download failure (get object, list objects) | `ItemReader` |

## Logging

All logging uses `log` macros — no `println!`:

```rust
info!("Uploading {} to s3://{}/{}", local_file, bucket, key);
debug!("Multipart upload: part {} ({} bytes)", part_num, bytes);
info!("Upload complete: s3://{}/{}", bucket, key);
info!("Downloading s3://{}/{} -> {}", bucket, key, local_file);
info!("Download complete: {} bytes written", total_bytes);
```

## Testing Strategy

- **Unit tests** (inline `#[cfg(test)]`): verify builder configuration — required fields, defaults, error on missing bucket/key. No real AWS calls.
- **Doc-tests**: marked `no_run` as they require an external S3-compatible service.
- **Example** (`tasklet_s3.rs`): demonstrates upload + download with LocalStack, marked `no_run`.
- No testcontainers integration in this version (LocalStack container can be added in a follow-up).

## Out of Scope

- `S3DeleteTasklet` — future follow-up
- S3 item readers/writers (chunk-oriented) — separate feature
- Presigned URLs
- Server-side encryption configuration
- Object tagging/metadata
