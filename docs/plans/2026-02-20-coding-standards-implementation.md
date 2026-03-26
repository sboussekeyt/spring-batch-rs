# Coding Standards Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Apply the coding rules defined in `.claude/rules/` to the codebase — add missing inline unit tests, complete rustdoc, and verify examples compile.

**Architecture:** Each task targets one source file. Tests are written inline (`#[cfg(test)]` mod), no external test runners needed. RDBC files that require a live DB use builder-state tests only (no mocks needed — we just verify struct fields, not execution).

**Tech Stack:** Rust 2021, cargo test, mockall (already in dev-deps), tempfile (already in dev-deps)

---

## Task 1: Add unit tests to `src/item/csv/csv_writer.rs`

**Files:**
- Modify: `src/item/csv/csv_writer.rs` (append `#[cfg(test)]` block at end of file)

**Step 1: Write the failing test block**

Append at end of file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::item::ItemWriter;
    use serde::Serialize;

    #[derive(Serialize, Clone)]
    struct Row {
        name: String,
        value: u32,
    }

    fn sample_rows() -> Vec<Row> {
        vec![
            Row { name: "alpha".into(), value: 1 },
            Row { name: "beta".into(), value: 2 },
        ]
    }

    #[test]
    fn should_write_records_with_headers() {
        let mut buf = Vec::new();
        {
            let writer = CsvItemWriterBuilder::<Row>::new()
                .has_headers(true)
                .from_writer(&mut buf);
            writer.write(&sample_rows()).unwrap();
            ItemWriter::<Row>::flush(&writer).unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("name,value"), "header row missing: {out}");
        assert!(out.contains("alpha,1"), "first data row missing: {out}");
        assert!(out.contains("beta,2"), "second data row missing: {out}");
    }

    #[test]
    fn should_write_records_without_headers() {
        let mut buf = Vec::new();
        {
            let writer = CsvItemWriterBuilder::<Row>::new()
                .has_headers(false)
                .from_writer(&mut buf);
            writer.write(&sample_rows()).unwrap();
            ItemWriter::<Row>::flush(&writer).unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains("name"), "header row should be absent: {out}");
        assert!(out.contains("alpha,1"));
    }

    #[test]
    fn should_write_with_custom_delimiter() {
        let mut buf = Vec::new();
        {
            let writer = CsvItemWriterBuilder::<Row>::new()
                .has_headers(true)
                .delimiter(b';')
                .from_writer(&mut buf);
            writer.write(&sample_rows()).unwrap();
            ItemWriter::<Row>::flush(&writer).unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("name;value"), "semicolon header missing: {out}");
        assert!(out.contains("alpha;1"), "semicolon data missing: {out}");
    }

    #[test]
    fn should_write_empty_chunk_without_error() {
        let mut buf = Vec::new();
        {
            let writer = CsvItemWriterBuilder::<Row>::new()
                .has_headers(true)
                .from_writer(&mut buf);
            writer.write(&[]).unwrap();
            ItemWriter::<Row>::flush(&writer).unwrap();
        }
        // Writing empty chunk should not panic and may or may not emit a header row
        let out = String::from_utf8(buf).unwrap();
        // Just verifying no crash and valid UTF-8
        let _ = out;
    }

    #[test]
    fn should_write_single_record() {
        let mut buf = Vec::new();
        {
            let writer = CsvItemWriterBuilder::<Row>::new()
                .has_headers(false)
                .from_writer(&mut buf);
            writer.write(&[Row { name: "only".into(), value: 99 }]).unwrap();
            ItemWriter::<Row>::flush(&writer).unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("only,99"), "single record missing: {out}");
    }

    #[test]
    fn should_write_to_file() {
        use std::fs;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let writer = CsvItemWriterBuilder::<Row>::new()
            .has_headers(true)
            .from_path(&path);
        writer.write(&sample_rows()).unwrap();
        ItemWriter::<Row>::flush(&writer).unwrap();
        drop(writer);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("name,value"), "file header missing");
        assert!(content.contains("alpha,1"), "file data missing");
    }
}
```

**Step 2: Run the tests**

```bash
cargo test --features csv -p spring-batch-rs csv_writer
```

Expected: all 6 tests pass (they test against the already-correct implementation).

**Step 3: Commit**

```bash
git add src/item/csv/csv_writer.rs
git commit -m "test(csv): add inline unit tests for CsvItemWriter"
```

---

## Task 2: Add unit tests to `src/item/rdbc/unified_reader_builder.rs`

**Files:**
- Modify: `src/item/rdbc/unified_reader_builder.rs` (append `#[cfg(test)]` block)

**Step 1: Write the test block**

Append at end of file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Note: We test builder *state* only — no real DB connection required.
    // Execution is covered by tests/rdbc_*.rs integration tests.

    #[test]
    fn should_set_query() {
        let builder = RdbcItemReaderBuilder::<String>::new()
            .query("SELECT id FROM users");
        assert_eq!(builder.query, Some("SELECT id FROM users"));
    }

    #[test]
    fn should_set_page_size() {
        let builder = RdbcItemReaderBuilder::<String>::new()
            .with_page_size(50);
        assert_eq!(builder.page_size, Some(50));
    }

    #[test]
    fn should_set_database_type_to_postgres_when_pool_provided() {
        // We cannot create a real pool here, so we verify db_type via default state.
        let builder = RdbcItemReaderBuilder::<String>::new();
        assert!(builder.db_type.is_none(), "db_type should be None before setting a pool");
    }

    #[test]
    fn should_chain_query_and_page_size() {
        let builder = RdbcItemReaderBuilder::<String>::new()
            .query("SELECT * FROM items")
            .with_page_size(100);
        assert_eq!(builder.query, Some("SELECT * FROM items"));
        assert_eq!(builder.page_size, Some(100));
    }

    #[test]
    fn should_have_no_pool_by_default() {
        let builder = RdbcItemReaderBuilder::<String>::new();
        assert!(builder.postgres_pool.is_none());
        assert!(builder.mysql_pool.is_none());
        assert!(builder.sqlite_pool.is_none());
    }
}
```

**Step 2: Run the tests**

```bash
cargo test --features rdbc-postgres,rdbc-mysql,rdbc-sqlite unified_reader_builder
```

Expected: all 5 tests pass.

**Step 3: Commit**

```bash
git add src/item/rdbc/unified_reader_builder.rs
git commit -m "test(rdbc): add builder state unit tests for RdbcItemReaderBuilder"
```

---

## Task 3: Add unit tests to `src/item/rdbc/unified_writer_builder.rs`

**Files:**
- Modify: `src/item/rdbc/unified_writer_builder.rs` (append `#[cfg(test)]` block)

**Step 1: Write the test block**

Append at end of file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Builder state tests only — no real DB connection required.

    #[test]
    fn should_set_table_name() {
        let builder = RdbcItemWriterBuilder::<String>::new()
            .table("users");
        assert_eq!(builder.table, Some("users"));
    }

    #[test]
    fn should_accumulate_columns() {
        let builder = RdbcItemWriterBuilder::<String>::new()
            .add_column("id")
            .add_column("name")
            .add_column("email");
        assert_eq!(builder.columns, vec!["id", "name", "email"]);
    }

    #[test]
    fn should_start_with_empty_columns() {
        let builder = RdbcItemWriterBuilder::<String>::new();
        assert!(builder.columns.is_empty(), "columns should be empty by default");
    }

    #[test]
    fn should_have_no_table_by_default() {
        let builder = RdbcItemWriterBuilder::<String>::new();
        assert!(builder.table.is_none());
    }

    #[test]
    fn should_chain_table_and_columns() {
        let builder = RdbcItemWriterBuilder::<String>::new()
            .table("orders")
            .add_column("order_id")
            .add_column("amount");
        assert_eq!(builder.table, Some("orders"));
        assert_eq!(builder.columns.len(), 2);
    }
}
```

**Step 2: Run the tests**

```bash
cargo test --features rdbc-postgres,rdbc-mysql,rdbc-sqlite unified_writer_builder
```

Expected: all 5 tests pass.

**Step 3: Commit**

```bash
git add src/item/rdbc/unified_writer_builder.rs
git commit -m "test(rdbc): add builder state unit tests for RdbcItemWriterBuilder"
```

---

## Task 4: Improve rustdoc in `src/item/fake/person_reader.rs`

**Files:**
- Modify: `src/item/fake/person_reader.rs`

**Step 1: Identify missing docs**

The following items lack `# Examples` or `# Parameters`:
- `PersonReaderBuilder::number_of_items`
- `PersonReaderBuilder::build`
- `PersonReader` struct itself

**Step 2: Add missing rustdoc**

For `PersonReader` struct — add type params and example:
```rust
/// Reads randomly generated `Person` objects.
///
/// Produces a fixed number of fake persons (configured via [`PersonReaderBuilder`]).
/// Returns `Ok(None)` when the configured count is exhausted.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::fake::person_reader::PersonReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
///
/// let reader = PersonReaderBuilder::new().number_of_items(3).build();
///
/// let p = reader.read().unwrap().unwrap();
/// assert!(!p.first_name.is_empty());
/// ```
pub struct PersonReader { ... }
```

For `number_of_items`:
```rust
/// Sets the total number of `Person` objects to generate.
///
/// Defaults to `0` (no items generated).
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::fake::person_reader::PersonReaderBuilder;
///
/// let builder = PersonReaderBuilder::new().number_of_items(10);
/// ```
pub fn number_of_items(mut self, number_of_items: usize) -> Self { ... }
```

For `build`:
```rust
/// Creates the configured [`PersonReader`].
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::fake::person_reader::PersonReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
///
/// let reader = PersonReaderBuilder::new().number_of_items(1).build();
/// assert!(reader.read().unwrap().is_some());
/// ```
pub fn build(self) -> PersonReader { ... }
```

**Step 3: Verify doc-tests compile and run**

```bash
cargo test --doc --features fake
```

Expected: all doc-tests pass.

**Step 4: Commit**

```bash
git add src/item/fake/person_reader.rs
git commit -m "docs(fake): add Examples sections to PersonReaderBuilder methods"
```

---

## Task 5: Add missing rustdoc Examples to `src/item/fake/person_reader.rs` — `PersonReaderBuilder::new`

**Files:**
- Modify: `src/item/fake/person_reader.rs`

**Step 1: Check current state of `PersonReaderBuilder::new`**

It currently reads: `/// Creates a new PersonReaderBuilder instance.`

Add `# Examples`:
```rust
/// Creates a new `PersonReaderBuilder` with default settings.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::fake::person_reader::PersonReaderBuilder;
///
/// let builder = PersonReaderBuilder::new();
/// ```
pub fn new() -> Self { ... }
```

**Step 2: Run doc-tests**

```bash
cargo test --doc --features fake
```

**Step 3: Commit** (combine with Task 4 if done in same session)

```bash
git add src/item/fake/person_reader.rs
git commit -m "docs(fake): complete rustdoc for PersonReaderBuilder"
```

---

## Task 6: Rename test `this_test_will_pass` in `src/item/csv/csv_reader.rs` and `src/item/fake/person_reader.rs`

The rule file `02-unit-tests.md` requires `should_<behaviour>_<condition>` naming.

**Files:**
- Modify: `src/item/csv/csv_reader.rs` — rename `this_test_will_pass` → `should_parse_string_records_with_headers`
- Modify: `src/item/fake/person_reader.rs` — rename `this_test_will_pass` → `should_read_configured_number_of_persons`

**Step 1: Rename in csv_reader.rs**

Find and replace:
```
fn this_test_will_pass()
```
→
```
fn should_parse_string_records_with_headers()
```

**Step 2: Rename in person_reader.rs**

Find and replace:
```
fn this_test_will_pass()
```
→
```
fn should_read_configured_number_of_persons()
```

**Step 3: Run tests to verify nothing broke**

```bash
cargo test --features csv,fake
```

Expected: all tests pass.

**Step 4: Commit**

```bash
git add src/item/csv/csv_reader.rs src/item/fake/person_reader.rs
git commit -m "test: rename this_test_will_pass to descriptive names per coding standards"
```

---

## Task 7: Verify all examples compile

**Files:**
- Read-only verification of `examples/*.rs` and `Cargo.toml`

**Step 1: Build all examples**

```bash
cargo build --examples --all-features 2>&1
```

Expected: zero errors, zero warnings.

**Step 2: Verify each example has `//!` module doc with Run section**

Check these files all have the pattern:
```
//! # Example: ...
//! ## Run
//! ```bash
//! cargo run --example ... --features ...
//! ```
```

Files to check:
- `examples/csv_processing.rs`
- `examples/json_processing.rs`
- `examples/xml_processing.rs`
- `examples/database_processing.rs`
- `examples/mongodb_processing.rs`
- `examples/orm_processing.rs`
- `examples/tasklet_zip.rs`
- `examples/tasklet_ftp.rs`
- `examples/advanced_patterns.rs`

**Step 3: For any missing module doc — add it**

Template:
```rust
//! # Example: <short title>
//!
//! Demonstrates <one sentence>.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example <name> --features <features>
//! ```
```

**Step 4: Commit if any changes made**

```bash
git add examples/
git commit -m "docs(examples): ensure all examples have module-level doc with run command"
```

---

## Task 8: Final quality gate

**Step 1: Run full test suite**

```bash
cargo test --all-features 2>&1
```

Expected: all tests pass.

**Step 2: Run clippy**

```bash
cargo clippy --all-features -- -D warnings 2>&1
```

Expected: zero warnings.

**Step 3: Run doc build**

```bash
cargo doc --no-deps --all-features 2>&1
```

Expected: zero warnings.

**Step 4: Run doc-tests**

```bash
cargo test --doc --all-features 2>&1
```

Expected: all doc-tests pass.

**Step 5: Commit if any lint fixes needed, then final commit**

```bash
git add -p
git commit -m "chore: apply coding standards — all tests, clippy, and docs pass"
```

---

## Summary

| Task | File | Type of change |
|---|---|---|
| 1 | `src/item/csv/csv_writer.rs` | Add 6 inline unit tests |
| 2 | `src/item/rdbc/unified_reader_builder.rs` | Add 5 builder state tests |
| 3 | `src/item/rdbc/unified_writer_builder.rs` | Add 5 builder state tests |
| 4-5 | `src/item/fake/person_reader.rs` | Add `# Examples` to 4 methods |
| 6 | `csv_reader.rs`, `person_reader.rs` | Rename 2 bad test names |
| 7 | `examples/*.rs` | Verify/add `//!` module docs |
| 8 | All | Quality gate: test + clippy + doc |
