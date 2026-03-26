# Rustdoc Rules — spring-batch-rs

## Obligation

Every `pub` item (struct, enum, trait, fn, type alias, const) MUST have a rustdoc comment. Items without documentation are a CI failure (`cargo doc --no-deps --all-features`).

## Standard Structure

```rust
/// One-sentence summary, ending with a period.
///
/// Optional paragraph explaining context, design choices, or caveats.
/// Keep it factual, no filler words.
///
/// # Type Parameters
///
/// - `I`: Input item type. Must implement `DeserializeOwned`.
/// - `R`: Underlying reader type. Must implement `Read`.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use spring_batch_rs::core::item::ItemReader;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Record { name: String }
///
/// let reader = CsvItemReaderBuilder::<Record>::new()
///     .has_headers(true)
///     .from_reader("name\nAlice".as_bytes());
///
/// let item = reader.read().unwrap().unwrap();
/// assert_eq!(item.name, "Alice");
/// ```
///
/// # Errors
///
/// Returns [`BatchError::ItemReader`] if parsing fails.
///
/// # Panics
///
/// Panics if the file path does not exist (only for `from_path` constructors).
pub struct MyReader<I, R: Read> { ... }
```

## Section Rules

| Section | When required | Content |
|---|---|---|
| `# Examples` | Always on public structs, traits, builders | Runnable code, not `compile_fail` unless truly necessary |
| `# Errors` | When `Result` is returned | List `BatchError` variants that can be returned |
| `# Panics` | When `unwrap`/`expect` or explicit `panic!` exists | Describe the condition |
| `# Type Parameters` | Generic structs/fns with non-obvious bounds | One bullet per type param |
| `# Implementation Note` | Non-obvious internals (e.g. `RefCell`, `Cell`) | Brief, factual |

## Trait Implementations

Document each method in a trait `impl` only if the behaviour deviates from the trait's own docs:

```rust
impl<I: DeserializeOwned, R: Read> ItemReader<I> for CsvItemReader<R> {
    /// Reads the next CSV row and deserializes it into `I`.
    ///
    /// Returns `Ok(None)` at end-of-file, `Err` on malformed input.
    fn read(&self) -> ItemReaderResult<I> { ... }
}
```

## Builder Methods

Every builder method must document:
1. What the parameter does
2. The default value
3. A minimal `# Examples` snippet

```rust
/// Sets the field delimiter byte.
///
/// Defaults to `b','`.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::csv::csv_reader::CsvItemReaderBuilder;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Row { a: String, b: String }
///
/// let reader = CsvItemReaderBuilder::<Row>::new()
///     .delimiter(b';')
///     .from_reader("a;b\n1;2".as_bytes());
/// ```
pub fn delimiter(mut self, delimiter: u8) -> Self { ... }
```

## Doc-test Rules

- Use `///` code fences (triple backtick) — they are compiled and run.
- Use `compile_fail` only when demonstrating intentional compile errors.
- Use `no_run` only for examples that require external resources (files, network, Docker).
- Every doc-test must `assert!` something — never leave assertions out.
- Features required by the example must be gated with `#[cfg(feature = "...")]` or documented in the prose.

### File Output in Doc-tests

**NEVER use a relative path for file output in a runnable doc-test.** `cargo test --doc` runs from the project root, so any relative `from_path("file.ext")` will create a file there.

Always use `std::env::temp_dir()`:

```rust
/// ```
/// # use std::env::temp_dir;
/// # use spring_batch_rs::item::json::json_writer::JsonItemWriterBuilder;
/// # use serde::Serialize;
/// # #[derive(Serialize)] struct Row { id: u32 }
///
/// let writer = JsonItemWriterBuilder::<Row>::new()
///     .from_path(temp_dir().join("output.json")); // ✅ temp dir
/// ```
```

```rust
// WRONG — creates a file in the project root during `cargo test --doc`
/// let writer = JsonItemWriterBuilder::<Row>::new()
///     .from_path("output.json"); // ❌ relative path
```

## Forbidden Patterns

```rust
// WRONG: empty
pub struct Foo;

// WRONG: vague
/// Does stuff.
pub fn bar() {}

// WRONG: restates the name
/// CsvItemReader reads CSV items.
pub struct CsvItemReader {}

// WRONG: missing # Errors
pub fn read(&self) -> Result<Option<T>, BatchError> {}
```

## Module-level Docs

Each `mod.rs` must have a `//!` comment block describing:
- What the module does
- Key types exported
- At least one full usage example

```rust
//! CSV support for reading and writing tabular data.
//!
//! # Key Types
//!
//! - [`CsvItemReader`] — reads CSV rows as typed structs
//! - [`CsvItemWriter`] — writes typed structs as CSV rows
//!
//! # Examples
//!
//! ```
//! // full working snippet
//! ```
```
