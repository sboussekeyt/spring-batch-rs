# CompositeItemWriter Design

**Date:** 2026-04-13  
**Branch:** 116-feat-chaining-itemprocessors  
**Status:** Approved

## Summary

Add `CompositeItemWriter<W1, W2>` and `CompositeItemWriterBuilder<W>` to `src/core/item.rs`.  
Implements a fan-out pattern: the same item chunk is written to all delegates in order.  
Uses static dispatch, mirroring the existing `CompositeItemProcessor` pattern exactly.

---

## Architecture

Both new types live in `src/core/item.rs` alongside `CompositeItemProcessor`.

```rust
pub struct CompositeItemWriter<W1, W2> {
    first: W1,
    second: W2,
}

pub struct CompositeItemWriterBuilder<W> {
    writer: W,
}
```

`CompositeItemWriter<W1, W2>` stores two writers by value — no heap allocation inside  
the struct. Chains are encoded in the type:  
`CompositeItemWriter<CompositeItemWriter<W1, W2>, W3>` for three writers.

---

## ItemWriter Implementation

`impl<T, W1, W2> ItemWriter<T> for CompositeItemWriter<W1, W2>`  
where `W1: ItemWriter<T>`, `W2: ItemWriter<T>`.

All four lifecycle methods delegate to `first` then `second`, short-circuiting on the  
first `Err`:

| Method  | Behaviour |
|---------|-----------|
| `write` | `first.write(items)?; second.write(items)` |
| `flush` | `first.flush()?; second.flush()` |
| `open`  | `first.open()?; second.open()` |
| `close` | `first.close()?; second.close()` |

If `open()` on `first` fails, `second.open()` is never called and `first.close()` is  
not called automatically — lifecycle management is the step's responsibility, consistent  
with the rest of the framework.

---

## Builder API

```rust
impl<W> CompositeItemWriterBuilder<W> {
    /// Creates a new builder with the given writer as the first delegate.
    pub fn new(first: W) -> Self;

    /// Appends a writer to the fan-out chain.
    pub fn add<W2>(self, next: W2)
        -> CompositeItemWriterBuilder<CompositeItemWriter<W, W2>>;

    /// Returns the built composite writer.
    pub fn build(self) -> W;
}
```

Usage:

```rust
let composite = CompositeItemWriterBuilder::new(csv_writer)
    .add(json_writer)
    .add(db_writer)
    .build();

step_builder.writer(&composite);
```

---

## Box Blanket Implementation

```rust
impl<T, W: ItemWriter<T> + ?Sized> ItemWriter<T> for Box<W> { ... }
```

Mirrors the existing `Box<P: ItemProcessor>` blanket. Allows `Box<dyn ItemWriter<T>>`  
and `Box<ConcreteWriter>` to be used wherever `&dyn ItemWriter<T>` is expected.

---

## Error Handling

All methods return `ItemWriterResult = Result<(), BatchError>`.  
Errors propagate via `?` — no swallowing, no retry logic at this layer.

---

## Testing

Inline `#[cfg(test)]` module added to `src/core/item.rs`.  
Local test structs with `std::cell::Cell` counters — no external mocking deps needed.

| Test name | Verifies |
|-----------|----------|
| `should_write_to_both_writers` | both writers receive the same items |
| `should_open_both_writers_in_order` | `open()` called on first then second |
| `should_close_both_writers_in_order` | `close()` called on first then second |
| `should_flush_both_writers` | `flush()` forwarded to both |
| `should_short_circuit_on_write_error` | first `write()` error stops chain |
| `should_short_circuit_on_open_error` | first `open()` error stops chain |
| `should_chain_three_writers` | three-writer builder, all receive items |
| `should_use_box_blanket_impl_as_item_writer` | `Box<dyn ItemWriter<T>>` via blanket impl |

---

## Example

New file: `examples/chaining_writers.rs`  
Required features: `logger`  
Demonstrates fan-out to `LoggerItemWriter` + a second in-memory writer, prints item  
count and status at the end.

`Cargo.toml` gets a new `[[example]]` entry with `required-features = ["logger"]`.

---

## Documentation Sync

Per `04-documentation.md` sync checklist:

- [ ] Rustdoc on `CompositeItemWriter`, `CompositeItemWriterBuilder`, and all public methods
- [ ] Module-level `//!` in `src/core/item.rs` updated (if present)
- [ ] At least one doc-test per public method that runs
- [ ] Website page `website/src/content/docs/examples/advanced-patterns.mdx` updated
- [ ] `examples/chaining_writers.rs` created
- [ ] `Cargo.toml` `[[example]]` entry added

---

## Files Changed

| File | Change |
|------|--------|
| `src/core/item.rs` | Add `CompositeItemWriter`, `CompositeItemWriterBuilder`, `Box<W>` blanket impl, tests |
| `examples/chaining_writers.rs` | New example |
| `Cargo.toml` | New `[[example]]` entry |
| `website/src/content/docs/examples/advanced-patterns.mdx` | Document new example |
