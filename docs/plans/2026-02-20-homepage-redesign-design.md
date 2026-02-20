# Homepage Redesign — Design Document

**Date:** 2026-02-20
**Approach:** Problem-first (Approach A)

## Context

The current homepage (`website/src/content/docs/index.mdx`) contains many sections — pipeline flow stats, tech specs cards, integration matrix, code showcase, capabilities grid, use cases section, and terminal CTA — but fails to clearly answer the fundamental question: *Why Spring Batch in Rust?*

## Goals

- Clearly explain **why Rust** for batch processing (the primary message, per user)
- Speak to **both audiences**: Rust developers unfamiliar with Spring Batch patterns, and Java/Spring developers discovering Rust
- Produce a page that is **structured but sober**: a clear role for each section, no redundancy
- Remove unverifiable marketing metrics ("10M+ Records/sec", "<50MB Memory")

## Target Audiences

1. **Rust developers** — know Rust, don't know Spring Batch; need to understand what problems the framework solves
2. **Java/Spring developers** — know Spring Batch patterns; need to understand why Rust and what translates

## Page Structure

### Section 1 — Hero

- **Title:** `Enterprise Batch Processing` / `Powered by Rust` (keeps existing gradient style)
- **Tagline:** `Spring Batch patterns you know. Rust performance you need.`
  - One line, speaks to both audiences immediately
- **Sub-tagline:** `Process millions of records with type safety, fault tolerance, and zero GC pauses.`
- **Actions:** `Get Started →` (primary) + `View Examples` (secondary)

**Why this tagline:** The current tagline ("High-performance data processing framework with type safety...") is generic. The new one answers "why Rust" AND "why Spring Batch" in one sentence.

### Section 2 — The Problem (new section)

Two-column text block, no complex styling:

**Left column:**
> Batch jobs fail silently. Memory grows until the process crashes. Errors get swallowed. Retries are manual. Monitoring is an afterthought.

**Right column:**
> Spring Batch RS brings proven patterns from the Java ecosystem — chunk-oriented processing, skip & retry policies, job lifecycle tracking — rewritten in Rust for memory safety and predictable performance.

Purpose: bridges the gap between the problem and the solution for both audiences.

### Section 3 — Code Example

**Title:** `From zero to pipeline in minutes`

Improved version of the existing CSV→JSON example with comments that explain *concepts*, not just syntax:

```rust
// 1. Define your data shape — compile-time type checking
#[derive(Deserialize, Serialize)]
struct Product { id: u32, name: String, price: f64 }

// 2. Configure source and destination
let reader = CsvItemReaderBuilder::<Product>::new()
    .has_headers(true)
    .from_path("products.csv");

let writer = JsonItemWriterBuilder::<Product>::new()
    .from_path("products.json");

// 3. Build the pipeline
//    chunk(100) = process 100 items per write transaction
//    skip_limit(10) = tolerate up to 10 bad records before failing
let step = StepBuilder::new("convert")
    .reader(&reader).writer(&writer)
    .chunk(100).skip_limit(10)
    .build();

JobBuilder::new().start(&step).build().run();
```

### Section 4 — Why Rust for Batch Processing?

**Title:** `Why Rust for batch processing?`

Four cards, concise, no fake metrics:

1. **No GC pauses** — Rust's ownership model means no garbage collector interrupting your batch jobs at runtime.
2. **Memory safety** — Buffer overflows and null pointer errors are compile-time failures, not production incidents.
3. **Predictable throughput** — Zero-cost abstractions add no overhead beyond what you explicitly write.
4. **Type-safe pipelines** — Reader, processor, and writer types must match at compile time. Wrong shape = won't compile.

**Replaces:** the current "Tech Specs" cards with invented performance numbers.

### Section 5 — Ecosystem (simplified)

Keep the concept of the integration matrix but simplified: three compact tag groups (Databases / Formats / Utilities) without large cards.

- **Databases:** PostgreSQL, MySQL, SQLite, MongoDB, SeaORM
- **Formats:** CSV, JSON, XML
- **Utilities:** Tokio Async, Fault Tolerance, ZIP, FTP, Fake Data

### Section 6 — CTA (simplified)

Keep the terminal `cargo add spring-batch-rs` widget but reduce excessive animations. Three buttons: **Read Docs** / **Browse Examples** / **GitHub**.

### Section 7 — Resource Links (unchanged)

Four `LinkCard` components at the bottom: API Reference, Architecture Guide, Quick Start, Examples Gallery.

## What is Removed

| Removed | Reason |
|---|---|
| "Production Use Cases" (4 cards) | Redundant with the problem section and code example |
| "Enterprise Capabilities" grid (6 cards) | Overlaps entirely with Section 4 (Why Rust) |
| Fake performance metrics (10M+ rec/s, <50MB) | Unverifiable, damages credibility |

## What is Preserved

- Dark theme with cyan/orange gradient accents
- Existing CSS animations (fadeInUp, shimmer, etc.) — kept but not added to new sections
- Existing `LinkCards` at the bottom
- Terminal CTA widget structure
- `template: splash` + Starlight hero YAML

## Files to Modify

- `website/src/content/docs/index.mdx` — primary change (content restructure)
- `website/src/styles/landing.css` — add styles for the new problem section (two-column text block)

## Success Criteria

- A developer unfamiliar with Spring Batch can understand the value proposition in under 10 seconds
- A Spring developer can immediately recognize the patterns and understand why Rust adds value
- No section is redundant with another
- No unverifiable marketing claims
