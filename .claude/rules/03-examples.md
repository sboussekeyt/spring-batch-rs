# Example Rules — spring-batch-rs

## Location

All examples live in `examples/`. Each file is a self-contained binary (declared in `Cargo.toml` with the correct `required-features`).

## File Naming Convention

```
<verb>_<output>_from_<input>_<detail>.rs

Examples:
  generate_json_file_from_csv_string.rs
  log_records_from_postgres_database.rs
  compress_files_with_zip_tasklet.rs
```

## Cargo.toml Declaration (required)

Every example MUST be declared with its feature flags:

```toml
[[example]]
name = "generate_json_file_from_csv_string"
required-features = ["csv", "json"]

[[example]]
name = "log_records_from_postgres_database"
required-features = ["rdbc-postgres", "logger"]
```

## File Structure

```rust
//! # Example: <Short title>
//!
//! Demonstrates <one sentence what this shows>.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example <name> --features <features>
//! ```
//!
//! ## What It Does
//!
//! 1. Step one
//! 2. Step two
//! 3. Step three

use spring_batch_rs::...;

// Data structures used in this example
#[derive(Debug, Serialize, Deserialize)]
struct MyRecord { ... }

// Optional: processor if transformation is shown
#[derive(Default)]
struct MyProcessor;

impl ItemProcessor<MyRecord, MyRecord> for MyProcessor {
    fn process(&self, item: &MyRecord) -> ItemProcessorResult<MyRecord> {
        Ok(item.clone())
    }
}

#[tokio::main]
async fn main() {
    // 1. Build reader
    let reader = ...;

    // 2. Build writer
    let writer = ...;

    // 3. Build step
    let step = StepBuilder::new("step-name")
        .chunk(100)
        .reader(&reader)
        .writer(&writer)
        .build();

    // 4. Build and run job
    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    println!("Status: {:?}", result.status);
}
```

## Content Requirements

| Requirement | Detail |
|---|---|
| Module-level doc (`//!`) | Title + run command + what-it-does steps |
| Data struct | Always define a `#[derive(Debug, Serialize/Deserialize)]` struct |
| Comments | One inline comment per logical block (build reader, build writer, etc.) |
| Output | Always print status or item count at the end |
| Error handling | Use `unwrap()` only in examples — add a comment noting it panics on error |
| Feature gate | Use `#[cfg(feature = "...")]` if mixing optional features |

## Categories

Maintain one example per meaningful combination:

| Category | Required examples |
|---|---|
| CSV | reader, writer, reader→writer, with fault-tolerance |
| JSON | reader, writer, reader→writer |
| XML | reader, writer, reader→writer |
| Database | postgres reader, postgres writer, sqlite reader, sqlite writer |
| Tasklets | zip, ftp |
| Fake data | fake→logger, fake→csv |
| ORM | orm writer |
| Advanced | processor chain, skip/retry, multiple steps |

## Forbidden Patterns

```rust
// WRONG: no module doc
fn main() { ... }

// WRONG: no comments in the body
let step = StepBuilder::new("step")
    .chunk(10)
    .reader(&r)
    .writer(&w)
    .build();

// WRONG: no output — user cannot tell if it worked
let result = job.run();
// (nothing printed)

// WRONG: unnecessary complexity — keep examples focused
struct MyFancyProcessor {
    counter: Arc<Mutex<u64>>,
    config: HashMap<String, String>,
}
```

## Running All Examples

```bash
# Build all examples to verify they compile
make examples

# Run a specific example
cargo run --example <name> --features <features>
```
