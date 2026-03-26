# README Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite README.md so that a Rust developer landing on crates.io immediately understands what the crate solves and can get to a working batch job in under 5 minutes.

**Architecture:** Single-file edit (README.md). Problem/Solution hook → minimal quick start → 2-sentence conceptual bridge → Why section → Features table → Examples table → Docs links → Community → License. Remove all contributor-focused content (Makefile commands, build tools).

**Tech Stack:** Markdown, crates.io CommonMark renderer constraints.

**Spec:** `docs/superpowers/specs/2026-03-25-readme-redesign-design.md`

---

## Files

- Modify: `README.md` — full rewrite per spec

---

### Task 1: Write the Header + Badges + Hook

**Files:**
- Modify: `README.md` (lines 1–11 of current file)

- [ ] **Step 1: Replace the header block**

Replace the current header section with:

```markdown
# spring-batch-rs

> Stop writing batch boilerplate. Start processing data.

[![crate](https://img.shields.io/crates/v/spring-batch-rs.svg)](https://crates.io/crates/spring-batch-rs)
[![docs](https://docs.rs/spring-batch-rs/badge.svg)](https://docs.rs/spring-batch-rs)
[![build status](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml/badge.svg)](https://github.com/sboussekeyt/spring-batch-rs/actions/workflows/test.yml)
[![Discord chat](https://img.shields.io/discord/1097536141617528966.svg?logo=discord&style=flat-square)](https://discord.gg/9FNhawNsG6)
[![CodeCov](https://codecov.io/gh/sboussekeyt/spring-batch-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sboussekeyt/spring-batch-rs)
![license](https://shields.io/badge/license-MIT%2FApache--2.0-blue)

Processing a large CSV into a database? You end up writing readers, chunk logic, error
loops, retry handling — just to move data. **Spring Batch RS** handles the plumbing: you
define what to read, what to transform, where to write. The rest is taken care of.
```

- [ ] **Step 2: Verify rendering mentally**

Check: tagline is short and punchy, badges are on one line, problem/solution is ≤ 4 lines.

---

### Task 2: Write the Quick Start section

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write the Cargo.toml block**

```markdown
## Quick Start

### 1. Add to `Cargo.toml`

```toml
[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json"] }
serde = { version = "1.0", features = ["derive"] }
```
```

- [ ] **Step 2: Write the code example**

Use verified import paths from `examples/csv_processing.rs`. The example is synchronous (no `#[tokio::main]`) — CSV/JSON don't need async:

```markdown
### 2. Your first batch job (CSV → JSON)

```rust
use spring_batch_rs::{
    core::{job::{Job, JobBuilder}, step::StepBuilder, item::PassThroughProcessor},
    item::{
        csv::csv_reader::CsvItemReaderBuilder,
        json::json_writer::JsonItemWriterBuilder,
    },
    BatchError,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Order {
    id: u32,
    amount: f64,
    status: String,
}

fn main() -> Result<(), BatchError> {
    let csv = "id,amount,status\n1,99.5,pending\n2,14.0,complete\n3,bad,pending";

    // Read from CSV
    let reader = CsvItemReaderBuilder::<Order>::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());

    // Write to JSON
    let writer = JsonItemWriterBuilder::<Order>::new()
        .from_path("orders.json");

    // Wire together: read 100 items at a time, tolerate up to 5 bad rows
    let step = StepBuilder::new("csv-to-json")
        .chunk(100)
        .reader(&reader)
        .processor(&PassThroughProcessor::<Order>::new())
        .writer(&writer)
        .skip_limit(5)
        .build();

    JobBuilder::new().start(&step).build().run().map(|_| ())
}
```

> Database readers/writers (PostgreSQL, SQLite, MongoDB…) require an async runtime.
> See the [Getting Started guide](https://sboussekeyt.github.io/spring-batch-rs/getting-started/) for the full setup.
```

- [ ] **Step 3: Verify import paths match source**

Open `src/item/csv/mod.rs` — confirm `pub mod csv_reader` exists.
Open `src/item/json/mod.rs` — confirm `pub mod json_writer` exists.
Open `src/core/item.rs` — confirm `pub struct PassThroughProcessor` exists.

---

### Task 3: Write the conceptual bridge + Why section

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write the conceptual bridge**

```markdown
## How It Works

A **Job** contains one or more **Steps**. Each Step reads items one by one from a source,
buffers them into a configurable chunk, then writes the whole chunk at once — balancing
throughput with memory usage.

```
Read item → Read item → ... → [chunk full] → Write chunk → repeat
```
```

- [ ] **Step 2: Write the Why section**

```markdown
## Why spring-batch-rs

- **Chunk-oriented processing** — reads one item at a time, writes in batches. Memory usage stays constant regardless of dataset size.
- **Fault tolerance built-in** — set a `skip_limit` to keep processing when bad rows appear. No manual try/catch loops.
- **Type-safe pipelines** — reader, processor, and writer types are verified at compile time. Mismatched types don't compile.
- **Modular by design** — enable only what you need via feature flags. No unused dependencies.
```

---

### Task 4: Write the Features table

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write the table with prose sub-headings (crates.io safe)**

Use bold prose sub-headings above each group to avoid blank-cell rendering artifacts:

```markdown
## Features

**Formats**

| Feature | Description |
| ------- | ----------- |
| `csv`   | CSV `ItemReader` and `ItemWriter` |
| `json`  | JSON `ItemReader` and `ItemWriter` |
| `xml`   | XML `ItemReader` and `ItemWriter` |

**Databases** *(require `tokio` — see [Getting Started](https://sboussekeyt.github.io/spring-batch-rs/getting-started/))*

| Feature         | Description |
| --------------- | ----------- |
| `rdbc-postgres` | PostgreSQL `ItemReader` and `ItemWriter` |
| `rdbc-mysql`    | MySQL / MariaDB `ItemReader` and `ItemWriter` |
| `rdbc-sqlite`   | SQLite `ItemReader` and `ItemWriter` |
| `mongodb`       | MongoDB `ItemReader` and `ItemWriter` (sync) |
| `orm`           | SeaORM `ItemReader` and `ItemWriter` |

**Utilities**

| Feature  | Description |
| -------- | ----------- |
| `zip`    | ZIP compression `Tasklet` |
| `ftp`    | FTP / FTPS `Tasklet` |
| `fake`   | Fake data `ItemReader` for generating test datasets |
| `logger` | Logger `ItemWriter` for debugging pipelines |
| `full`   | All of the above |
```

---

### Task 5: Write the Examples table

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write the examples table with cargo run commands**

```markdown
## Examples

| Use case | Run |
| -------- | --- |
| CSV → JSON | `cargo run --example csv_processing --features csv,json` |
| CSV → SQLite | `cargo run --example database_processing --features rdbc-sqlite,csv,json,logger` |
| XML processing | `cargo run --example xml_processing --features xml,json,csv` |
| Advanced ETL pipeline | `cargo run --example advanced_patterns --features csv,json,logger` |
| ZIP tasklet | `cargo run --example tasklet_zip --features zip` |

> Database examples require Docker to be running (used by testcontainers for PostgreSQL, MySQL, MongoDB).

For the full examples gallery, tutorials, and advanced patterns:
**[https://sboussekeyt.github.io/spring-batch-rs/quick-examples/](https://sboussekeyt.github.io/spring-batch-rs/quick-examples/)**
```

---

### Task 6: Write the Documentation, Community, and License sections

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write Documentation section**

```markdown
## Documentation

| Resource | Link |
| -------- | ---- |
| Getting Started | [sboussekeyt.github.io/…/getting-started](https://sboussekeyt.github.io/spring-batch-rs/getting-started/) |
| Item Readers & Writers | [sboussekeyt.github.io/…/item-readers-writers](https://sboussekeyt.github.io/spring-batch-rs/item-readers-writers/overview/) |
| API Reference | [docs.rs/spring-batch-rs](https://docs.rs/spring-batch-rs) |
| Architecture | [sboussekeyt.github.io/…/architecture](https://sboussekeyt.github.io/spring-batch-rs/architecture/) |
```

- [ ] **Step 2: Write Community section**

```markdown
## Community

- [Discord](https://discord.gg/9FNhawNsG6) — Chat with the community
- [GitHub Issues](https://github.com/sboussekeyt/spring-batch-rs/issues) — Bug reports and feature requests
- [GitHub Discussions](https://github.com/sboussekeyt/spring-batch-rs/discussions) — Questions and ideas
```

- [ ] **Step 3: Write License section**

```markdown
## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
```

---

### Task 7: Final review and commit

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Count lines**

Run:
```bash
wc -l README.md
```
Expected: ≤ 150 lines. If over, trim verbose prose.

- [ ] **Step 2: Check that removed sections are gone**

Verify these are absent in the new README:
```bash
grep -n "Makefile\|make dev\|make check\|Build Tools\|build-tools\|cargo-check.sh" README.md
```
Expected: no matches.

- [ ] **Step 3: Compile-check the quick start example**

The closest real example exercises the same code path:
```bash
cargo check --example csv_processing --features csv,json
```
Expected: no errors.

- [ ] **Step 4: Verify no broken links (spot check)**

Confirm these URLs appear correctly:
- `https://sboussekeyt.github.io/spring-batch-rs/getting-started/`
- `https://docs.rs/spring-batch-rs`
- `https://discord.gg/9FNhawNsG6`

- [ ] **Step 5: Commit**

```bash
git add README.md docs/superpowers/specs/2026-03-25-readme-redesign-design.md docs/superpowers/plans/2026-03-25-readme-redesign.md
git commit -m "docs: rewrite README for crates.io — problem/solution framing, verified quick start, remove contributor content"
```
