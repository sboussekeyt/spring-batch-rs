# README Redesign — Design Spec

**Date:** 2026-03-25
**Status:** Approved (v2 — post-review fixes)

## Context

The current README.md is displayed on crates.io. It underperforms as a marketing document: the intro is generic, a large "Development" section targets contributors instead of users, and the "why" section lists vague bullets without emotional resonance.

## Goals

- Make the README compelling for Rust developers discovering the crate on crates.io
- Clearly communicate what problem the crate solves
- Get a developer to their first working batch job as fast as possible
- Remove contributor-focused content that dilutes the user message

## Target Audience

**Primary:** Rust developers who need to process large volumes of data (CSV, JSON, XML, databases) and don't want to write the infrastructure plumbing themselves. No Java/Spring background assumed.

## Approach: Problem/Solution

Open with an emotional hook, follow immediately with a minimal working example, then justify the choice with concrete technical benefits.

---

## Structure

### 1. Header + Hook

```
# spring-batch-rs

> Stop writing batch boilerplate. Start processing data.

[badges: crate version, docs.rs, build status, Discord, CodeCov, license]
```

**Problem/Solution block (3-4 lines):**

> Processing a large CSV into a database? You end up writing readers, chunk logic, error loops, retry handling — just to move data. Spring Batch RS handles the plumbing: you define what to read, what to transform, where to write. The rest is taken care of.

### 2. Quick Start

Two sub-sections:

**Add to Cargo.toml:**
```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json"] }
serde = { version = "1.0", features = ["derive"] }
```

> Note for implementation: database features (rdbc-*, orm, mongodb) additionally require
> `tokio = { version = "1", features = ["full"] }`. A note must appear near the examples
> table in Section 5 pointing async users to the Getting Started Guide.

**Minimal working example** (CSV → JSON, synchronous):

Correct import paths (verified against actual source):
```rust
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{
        csv::csv_reader::CsvItemReaderBuilder,   // csv::csv_reader sub-module
        json::json_writer::JsonItemWriterBuilder, // json::json_writer sub-module
    },
};
```

Structure:
- Define a typed struct with `Deserialize + Serialize`
- Build a `CsvItemReaderBuilder::<T>::new().has_headers(true).from_reader(...)`
- Build a `JsonItemWriterBuilder::<T>::new().from_path(...)`
- Wire with `StepBuilder::new("step-name").chunk(100).reader(&reader).processor(&processor).writer(&writer).skip_limit(5).build()`
- Run with `JobBuilder::new().start(&step).build().run()`
- `fn main()` returns `Result<(), BatchError>` — synchronous, no async runtime needed for CSV/JSON

One comment per logical block. Show `skip_limit` to hint at fault tolerance without a separate section.

**Sync vs async note:** Add a one-line note after the example:
> `// Database readers/writers require an async runtime — see the documentation.`

### 3. Conceptual bridge (2 sentences max)

Before or after the quick start, include a minimal two-sentence conceptual intro to ensure
developers without Spring Batch background understand the structure they just copy-pasted:

> A **Job** is composed of one or more **Steps**. Each Step reads items one by one, buffers
> them into a chunk, then writes the whole chunk — balancing throughput with memory usage.

This replaces the current six-bullet "Core Concepts" section with something scannable in 5 seconds.

### 4. Why spring-batch-rs

Four bullets, each with a bolded concept + concrete technical explanation:

- **Chunk-oriented processing** — reads one item at a time, writes in batches. Constant memory usage regardless of dataset size.
- **Fault tolerance built-in** — configure a `skip_limit` to keep processing when bad rows appear. No manual try/catch loops.
- **Type-safe pipelines** — reader, processor, and writer types are checked at compile time. Wrong types don't compile.
- **Modular by design** — enable only what you need via feature flags. No unused dependencies pulled in.

### 5. Features Table

Reorganized by category. Use sub-headings above each group (not blank-cell header rows)
to avoid rendering artifacts on crates.io:

```markdown
**Formats**

| Feature | Description |
|---------|-------------|
| `csv`   | CSV ItemReader and ItemWriter |
| `json`  | JSON ItemReader and ItemWriter |
| `xml`   | XML ItemReader and ItemWriter |

**Databases** *(require `tokio` async runtime)*

| Feature | Description |
|---------|-------------|
| `rdbc-postgres` | PostgreSQL ItemReader and ItemWriter |
| ...

**Utilities**

| Feature | Description |
| ...
```

### 6. Examples Table

Each row includes the `cargo run --example` command for immediate runnability:

| Use case | Run command |
|----------|-------------|
| CSV → JSON | `cargo run --example csv_processing --features csv,json` |
| CSV → SQLite | `cargo run --example database_processing --features rdbc-sqlite,csv,json,logger` |
| XML processing | `cargo run --example xml_processing --features xml,json,csv` |
| Generate fake data | `cargo run --example advanced_patterns --features csv,json,logger` |
| ZIP tasklet | `cargo run --example tasklet_zip --features zip` |

Link: → Full examples gallery (website URL)
Note: Add one-liner "Database examples require Docker (testcontainers)."

### 7. Documentation Links

- Getting Started Guide
- Item Readers & Writers
- API Reference (docs.rs)
- Architecture Overview

### 8. Community

- Discord
- GitHub Issues
- GitHub Discussions

### 9. License

MIT OR Apache-2.0 (unchanged)

---

## What Gets Removed

| Section | Reason |
|---------|--------|
| "Development" (Makefile commands) | Contributor content, not user content |
| "Build Tools" section | Same |
| Six-bullet "Core Concepts" section | Replaced by a 2-sentence conceptual bridge (Section 3) |

## Success Criteria

- A Rust developer landing on crates.io can understand what the crate does in under 10 seconds
- They can copy-paste the quick start and have a working batch job (imports verified against source)
- The README is ≤ 150 lines (vs current ~200 with less signal)
- No blank-cell category header rows in tables (crates.io rendering safe)
