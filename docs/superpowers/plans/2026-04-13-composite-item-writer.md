# CompositeItemWriter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `CompositeItemWriter<W1, W2>` and `CompositeItemWriterBuilder<W>` to `src/core/item.rs` so the same item chunk can be fan-out written to multiple writers using static dispatch, mirroring the existing `CompositeItemProcessor` pattern.

**Architecture:** Two new types in `src/core/item.rs`: a struct that stores two writers by value and fans out all four `ItemWriter` lifecycle methods, and a builder that wraps the accumulated type with each `.add()` call. A `Box<W: ItemWriter<T> + ?Sized>` blanket impl is also added. All logic is self-contained in the existing file.

**Tech Stack:** Rust 2021, `std::cell::Cell` for test recording, `spring_batch_rs::item::logger::LoggerWriterBuilder` for the example.

---

### Task 1: `CompositeItemWriter` struct + `ItemWriter` impl

**Files:**
- Modify: `src/core/item.rs` (add struct, impl, and tests)

- [ ] **Step 1: Write the failing tests**

  Add this block at the end of the existing `#[cfg(test)] mod tests` in `src/core/item.rs`, after the last existing test (`should_use_box_concrete_type_as_item_processor`):

  ```rust
  // --- CompositeItemWriter ---

  use std::cell::Cell;

  struct RecordingWriter {
      write_calls: Cell<usize>,
      items_written: Cell<usize>,
      open_calls: Cell<usize>,
      close_calls: Cell<usize>,
      flush_calls: Cell<usize>,
      fail_write: bool,
      fail_open: bool,
  }

  impl RecordingWriter {
      fn new() -> Self {
          Self {
              write_calls: Cell::new(0),
              items_written: Cell::new(0),
              open_calls: Cell::new(0),
              close_calls: Cell::new(0),
              flush_calls: Cell::new(0),
              fail_write: false,
              fail_open: false,
          }
      }
      fn failing_write() -> Self {
          Self { fail_write: true, ..Self::new() }
      }
      fn failing_open() -> Self {
          Self { fail_open: true, ..Self::new() }
      }
  }

  impl ItemWriter<i32> for RecordingWriter {
      fn write(&self, items: &[i32]) -> ItemWriterResult {
          if self.fail_write {
              return Err(BatchError::ItemWriter("forced write failure".to_string()));
          }
          self.write_calls.set(self.write_calls.get() + 1);
          self.items_written.set(self.items_written.get() + items.len());
          Ok(())
      }
      fn open(&self) -> ItemWriterResult {
          if self.fail_open {
              return Err(BatchError::ItemWriter("forced open failure".to_string()));
          }
          self.open_calls.set(self.open_calls.get() + 1);
          Ok(())
      }
      fn close(&self) -> ItemWriterResult {
          self.close_calls.set(self.close_calls.get() + 1);
          Ok(())
      }
      fn flush(&self) -> ItemWriterResult {
          self.flush_calls.set(self.flush_calls.get() + 1);
          Ok(())
      }
  }

  #[test]
  fn should_write_to_both_writers() -> Result<(), BatchError> {
      let w1 = RecordingWriter::new();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      composite.write(&[1, 2, 3])?;
      assert_eq!(composite.first.write_calls.get(), 1, "first writer should be called");
      assert_eq!(composite.first.items_written.get(), 3, "first writer should receive 3 items");
      assert_eq!(composite.second.write_calls.get(), 1, "second writer should be called");
      assert_eq!(composite.second.items_written.get(), 3, "second writer should receive 3 items");
      Ok(())
  }

  #[test]
  fn should_open_both_writers_in_order() -> Result<(), BatchError> {
      let w1 = RecordingWriter::new();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      composite.open()?;
      assert_eq!(composite.first.open_calls.get(), 1, "first writer should be opened");
      assert_eq!(composite.second.open_calls.get(), 1, "second writer should be opened");
      Ok(())
  }

  #[test]
  fn should_close_both_writers_in_order() -> Result<(), BatchError> {
      let w1 = RecordingWriter::new();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      composite.close()?;
      assert_eq!(composite.first.close_calls.get(), 1, "first writer should be closed");
      assert_eq!(composite.second.close_calls.get(), 1, "second writer should be closed");
      Ok(())
  }

  #[test]
  fn should_flush_both_writers() -> Result<(), BatchError> {
      let w1 = RecordingWriter::new();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      composite.flush()?;
      assert_eq!(composite.first.flush_calls.get(), 1, "first writer should be flushed");
      assert_eq!(composite.second.flush_calls.get(), 1, "second writer should be flushed");
      Ok(())
  }

  #[test]
  fn should_short_circuit_on_write_error() {
      let w1 = RecordingWriter::failing_write();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      let result = composite.write(&[1, 2, 3]);
      assert!(result.is_err(), "error should propagate");
      assert_eq!(composite.second.write_calls.get(), 0, "second writer should not be called after first fails");
  }

  #[test]
  fn should_short_circuit_on_open_error() {
      let w1 = RecordingWriter::failing_open();
      let w2 = RecordingWriter::new();
      let composite = CompositeItemWriter { first: w1, second: w2 };
      let result = composite.open();
      assert!(result.is_err(), "error should propagate");
      assert_eq!(composite.second.open_calls.get(), 0, "second writer should not be opened after first fails");
  }
  ```

- [ ] **Step 2: Run tests to verify they fail**

  ```bash
  cargo test --all-features should_write_to_both_writers 2>&1 | head -20
  ```

  Expected: `error[E0412]: cannot find type 'CompositeItemWriter'`

- [ ] **Step 3: Add the struct and impl**

  In `src/core/item.rs`, after the closing `}` of `CompositeItemProcessorBuilder`'s `impl` block (around line 626) and before the `Box<P>` blanket impl for processors, insert:

  ```rust
  /// A composite writer that fans out the same chunk to two writers sequentially using static dispatch.
  ///
  /// Both writers receive identical slices on every `write` call. All four lifecycle
  /// methods (`write`, `flush`, `open`, `close`) are forwarded to `first` then `second`,
  /// short-circuiting on the first `Err`. If `open()` on `first` fails, `second.open()`
  /// is never called — lifecycle management is the step's responsibility.
  ///
  /// Both writers are stored by value — no heap allocation occurs inside the struct.
  /// The type encodes the full chain:
  /// `CompositeItemWriter<CompositeItemWriter<W1, W2>, W3>` for three writers.
  ///
  /// Construct chains using [`CompositeItemWriterBuilder`] rather than instantiating
  /// this struct directly.
  ///
  /// # Type Parameters
  ///
  /// - `W1`: The first writer type. Must implement `ItemWriter<T>`.
  /// - `W2`: The second writer type. Must implement `ItemWriter<T>`.
  ///
  /// # Examples
  ///
  /// ```
  /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
  ///
  /// struct CountingWriter { count: std::cell::Cell<usize> }
  /// impl CountingWriter { fn new() -> Self { Self { count: std::cell::Cell::new(0) } } }
  /// impl ItemWriter<i32> for CountingWriter {
  ///     fn write(&self, items: &[i32]) -> Result<(), spring_batch_rs::BatchError> {
  ///         self.count.set(self.count.get() + items.len());
  ///         Ok(())
  ///     }
  /// }
  ///
  /// let composite = CompositeItemWriterBuilder::new(CountingWriter::new())
  ///     .add(CountingWriter::new())
  ///     .build();
  ///
  /// composite.write(&[1, 2, 3]).unwrap();
  /// assert_eq!(composite.first.count.get(), 3);
  /// assert_eq!(composite.second.count.get(), 3);
  /// ```
  ///
  /// # Errors
  ///
  /// Returns [`BatchError`] if any writer in the chain returns an error.
  pub struct CompositeItemWriter<W1, W2> {
      /// The first writer in the fan-out chain.
      pub first: W1,
      /// The second writer in the fan-out chain.
      pub second: W2,
  }

  impl<T, W1, W2> ItemWriter<T> for CompositeItemWriter<W1, W2>
  where
      W1: ItemWriter<T>,
      W2: ItemWriter<T>,
  {
      /// Writes `items` to `first`, then to `second`. Short-circuits on the first error.
      ///
      /// # Errors
      ///
      /// Returns [`BatchError::ItemWriter`] if either writer fails.
      fn write(&self, items: &[T]) -> ItemWriterResult {
          self.first.write(items)?;
          self.second.write(items)
      }

      /// Flushes `first`, then `second`. Short-circuits on the first error.
      ///
      /// # Errors
      ///
      /// Returns [`BatchError::ItemWriter`] if either flush fails.
      fn flush(&self) -> ItemWriterResult {
          self.first.flush()?;
          self.second.flush()
      }

      /// Opens `first`, then `second`. Short-circuits on the first error.
      ///
      /// # Errors
      ///
      /// Returns [`BatchError::ItemWriter`] if either open fails.
      fn open(&self) -> ItemWriterResult {
          self.first.open()?;
          self.second.open()
      }

      /// Closes `first`, then `second`. Short-circuits on the first error.
      ///
      /// # Errors
      ///
      /// Returns [`BatchError::ItemWriter`] if either close fails.
      fn close(&self) -> ItemWriterResult {
          self.first.close()?;
          self.second.close()
      }
  }
  ```

- [ ] **Step 4: Run the new tests to verify they pass**

  ```bash
  cargo test --all-features should_write_to_both_writers should_open_both_writers_in_order should_close_both_writers_in_order should_flush_both_writers should_short_circuit_on_write_error should_short_circuit_on_open_error 2>&1 | tail -10
  ```

  Expected: `6 passed`

- [ ] **Step 5: Commit**

  ```bash
  git add src/core/item.rs
  git commit -m "feat: add CompositeItemWriter with fan-out ItemWriter impl"
  ```

---

### Task 2: `CompositeItemWriterBuilder`

**Files:**
- Modify: `src/core/item.rs` (add builder struct, impl, and tests)

- [ ] **Step 1: Write the failing tests**

  Append these tests to the `#[cfg(test)] mod tests` block after the tests from Task 1:

  ```rust
  #[test]
  fn should_chain_two_writers_via_builder() -> Result<(), BatchError> {
      let composite = CompositeItemWriterBuilder::new(RecordingWriter::new())
          .add(RecordingWriter::new())
          .build();
      composite.write(&[10, 20])?;
      assert_eq!(composite.first.items_written.get(), 2, "first writer should receive 2 items");
      assert_eq!(composite.second.items_written.get(), 2, "second writer should receive 2 items");
      Ok(())
  }

  #[test]
  fn should_chain_three_writers() -> Result<(), BatchError> {
      let composite = CompositeItemWriterBuilder::new(RecordingWriter::new())
          .add(RecordingWriter::new())
          .add(RecordingWriter::new())
          .build();
      composite.write(&[1, 2, 3, 4])?;
      // composite: CompositeItemWriter<CompositeItemWriter<W1, W2>, W3>
      // composite.first is CompositeItemWriter<W1, W2>
      // composite.second is W3
      assert_eq!(composite.first.first.items_written.get(), 4, "writer 1 should receive 4 items");
      assert_eq!(composite.first.second.items_written.get(), 4, "writer 2 should receive 4 items");
      assert_eq!(composite.second.items_written.get(), 4, "writer 3 should receive 4 items");
      Ok(())
  }
  ```

- [ ] **Step 2: Run tests to verify they fail**

  ```bash
  cargo test --all-features should_chain_two_writers_via_builder 2>&1 | head -20
  ```

  Expected: `error[E0412]: cannot find type 'CompositeItemWriterBuilder'`

- [ ] **Step 3: Add the builder struct and impl**

  In `src/core/item.rs`, directly after the closing `}` of the `CompositeItemWriter` impl block, insert:

  ```rust
  /// Builder for creating a fan-out chain of [`ItemWriter`]s using static dispatch.
  ///
  /// Start the chain with [`new`](CompositeItemWriterBuilder::new), append writers
  /// with [`add`](CompositeItemWriterBuilder::add), and finalise with
  /// [`build`](CompositeItemWriterBuilder::build). Each call to `add` wraps the
  /// accumulated chain in a [`CompositeItemWriter`]. The built chain stores all
  /// writers by value — no heap allocations occur inside the chain itself.
  ///
  /// # Type Parameters
  ///
  /// - `W`: The accumulated writer type. Starts as the first writer and is wrapped
  ///   in [`CompositeItemWriter`] with each [`add`](CompositeItemWriterBuilder::add) call.
  ///
  /// # Examples
  ///
  /// Two writers:
  ///
  /// ```
  /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
  ///
  /// struct CountingWriter { count: std::cell::Cell<usize> }
  /// impl CountingWriter { fn new() -> Self { Self { count: std::cell::Cell::new(0) } } }
  /// impl ItemWriter<i32> for CountingWriter {
  ///     fn write(&self, items: &[i32]) -> Result<(), spring_batch_rs::BatchError> {
  ///         self.count.set(self.count.get() + items.len());
  ///         Ok(())
  ///     }
  /// }
  ///
  /// let composite = CompositeItemWriterBuilder::new(CountingWriter::new())
  ///     .add(CountingWriter::new())
  ///     .build();
  ///
  /// composite.write(&[1, 2, 3]).unwrap();
  /// assert_eq!(composite.first.count.get(), 3);
  /// assert_eq!(composite.second.count.get(), 3);
  /// ```
  ///
  /// Three writers:
  ///
  /// ```
  /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
  ///
  /// struct CountingWriter { count: std::cell::Cell<usize> }
  /// impl CountingWriter { fn new() -> Self { Self { count: std::cell::Cell::new(0) } } }
  /// impl ItemWriter<i32> for CountingWriter {
  ///     fn write(&self, items: &[i32]) -> Result<(), spring_batch_rs::BatchError> {
  ///         self.count.set(self.count.get() + items.len());
  ///         Ok(())
  ///     }
  /// }
  ///
  /// let composite = CompositeItemWriterBuilder::new(CountingWriter::new())
  ///     .add(CountingWriter::new())
  ///     .add(CountingWriter::new())
  ///     .build();
  ///
  /// composite.write(&[1, 2]).unwrap();
  /// assert_eq!(composite.first.first.count.get(), 2);
  /// assert_eq!(composite.first.second.count.get(), 2);
  /// assert_eq!(composite.second.count.get(), 2);
  /// ```
  pub struct CompositeItemWriterBuilder<W> {
      writer: W,
  }

  impl<W> CompositeItemWriterBuilder<W> {
      /// Creates a new builder with the given writer as the first delegate.
      ///
      /// # Parameters
      ///
      /// - `first`: The first writer in the fan-out chain.
      ///
      /// # Examples
      ///
      /// ```
      /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
      ///
      /// struct NoOpWriter;
      /// impl ItemWriter<i32> for NoOpWriter {
      ///     fn write(&self, _items: &[i32]) -> Result<(), spring_batch_rs::BatchError> { Ok(()) }
      /// }
      ///
      /// let builder = CompositeItemWriterBuilder::new(NoOpWriter);
      /// let writer = builder.build();
      /// assert!(writer.write(&[]).is_ok());
      /// ```
      pub fn new(first: W) -> Self {
          Self { writer: first }
      }

      /// Appends a writer to the fan-out chain.
      ///
      /// Returns a new builder whose accumulated type is `CompositeItemWriter<W, W2>`.
      /// Both writers must implement `ItemWriter<T>` for the same `T` — this is
      /// verified at compile time.
      ///
      /// # Parameters
      ///
      /// - `next`: The writer to add to the chain.
      ///
      /// # Examples
      ///
      /// ```
      /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
      ///
      /// struct NoOpWriter;
      /// impl ItemWriter<i32> for NoOpWriter {
      ///     fn write(&self, _items: &[i32]) -> Result<(), spring_batch_rs::BatchError> { Ok(()) }
      /// }
      ///
      /// let composite = CompositeItemWriterBuilder::new(NoOpWriter)
      ///     .add(NoOpWriter)
      ///     .build();
      ///
      /// assert!(composite.write(&[1, 2, 3]).is_ok());
      /// ```
      pub fn add<W2>(self, next: W2) -> CompositeItemWriterBuilder<CompositeItemWriter<W, W2>> {
          CompositeItemWriterBuilder {
              writer: CompositeItemWriter {
                  first: self.writer,
                  second: next,
              },
          }
      }

      /// Builds and returns the composite writer.
      ///
      /// Returns the accumulated writer value `W`. When chained via `add`, `W` will
      /// be a nested `CompositeItemWriter` such as
      /// `CompositeItemWriter<W1, CompositeItemWriter<W2, W3>>`.
      ///
      /// Pass `&composite` to the step builder's `.writer()` method.
      ///
      /// # Examples
      ///
      /// ```
      /// use spring_batch_rs::core::item::{ItemWriter, CompositeItemWriterBuilder};
      ///
      /// struct NoOpWriter;
      /// impl ItemWriter<i32> for NoOpWriter {
      ///     fn write(&self, _items: &[i32]) -> Result<(), spring_batch_rs::BatchError> { Ok(()) }
      /// }
      ///
      /// let composite = CompositeItemWriterBuilder::new(NoOpWriter)
      ///     .add(NoOpWriter)
      ///     .build();
      ///
      /// assert!(composite.write(&[]).is_ok());
      /// ```
      pub fn build(self) -> W {
          self.writer
      }
  }
  ```

- [ ] **Step 4: Run the new tests to verify they pass**

  ```bash
  cargo test --all-features should_chain_two_writers_via_builder should_chain_three_writers 2>&1 | tail -10
  ```

  Expected: `2 passed`

- [ ] **Step 5: Run all tests to check for regressions**

  ```bash
  cargo test --all-features 2>&1 | tail -15
  ```

  Expected: all tests pass.

- [ ] **Step 6: Commit**

  ```bash
  git add src/core/item.rs
  git commit -m "feat: add CompositeItemWriterBuilder for static-dispatch fan-out chains"
  ```

---

### Task 3: `Box<W: ItemWriter<T>>` blanket impl

**Files:**
- Modify: `src/core/item.rs` (add blanket impl and test)

- [ ] **Step 1: Write the failing test**

  Append to the `#[cfg(test)] mod tests` block:

  ```rust
  #[test]
  fn should_use_box_blanket_impl_as_item_writer() -> Result<(), BatchError> {
      let composite = CompositeItemWriterBuilder::new(RecordingWriter::new())
          .add(RecordingWriter::new())
          .build();
      let boxed: Box<dyn ItemWriter<i32>> = Box::new(composite);
      boxed.write(&[5, 6, 7])?;
      // The test verifies that Box<dyn ItemWriter<T>> can be used as an ItemWriter<T>.
      // We can't inspect the inner writers through Box<dyn>, so asserting Ok is sufficient.
      Ok(())
  }

  #[test]
  fn should_use_box_concrete_writer_as_item_writer() -> Result<(), BatchError> {
      let boxed: Box<RecordingWriter> = Box::new(RecordingWriter::new());
      boxed.write(&[1, 2])?;
      assert_eq!(boxed.items_written.get(), 2, "boxed concrete writer should delegate write");
      Ok(())
  }
  ```

- [ ] **Step 2: Run tests to verify they fail**

  ```bash
  cargo test --all-features should_use_box_blanket_impl_as_item_writer 2>&1 | head -20
  ```

  Expected: `error[E0277]: the trait bound 'Box<...>: ItemWriter<i32>' is not satisfied`

- [ ] **Step 3: Add the blanket impl**

  In `src/core/item.rs`, after the existing `Box<P: ItemProcessor>` blanket impl (around line 634), add:

  ```rust
  /// Allows any `Box<W>` where `W: ItemWriter<T>` to be used wherever
  /// `&dyn ItemWriter<T>` is expected — including boxed concrete types
  /// (`Box<MyWriter>`) and boxed trait objects (`Box<dyn ItemWriter<T>>`).
  ///
  /// The `?Sized` bound makes this cover trait objects: `dyn Trait` is
  /// unsized, so without `?Sized` the impl would not apply to them.
  impl<T, W: ItemWriter<T> + ?Sized> ItemWriter<T> for Box<W> {
      fn write(&self, items: &[T]) -> ItemWriterResult {
          (**self).write(items)
      }
      fn flush(&self) -> ItemWriterResult {
          (**self).flush()
      }
      fn open(&self) -> ItemWriterResult {
          (**self).open()
      }
      fn close(&self) -> ItemWriterResult {
          (**self).close()
      }
  }
  ```

- [ ] **Step 4: Run the new tests to verify they pass**

  ```bash
  cargo test --all-features should_use_box_blanket_impl_as_item_writer should_use_box_concrete_writer_as_item_writer 2>&1 | tail -10
  ```

  Expected: `2 passed`

- [ ] **Step 5: Run all tests and verify doc build**

  ```bash
  cargo test --all-features 2>&1 | tail -5
  cargo doc --no-deps --all-features 2>&1 | grep -E "warning|error"
  ```

  Expected: all tests pass, no doc warnings.

- [ ] **Step 6: Commit**

  ```bash
  git add src/core/item.rs
  git commit -m "feat: add Box<W: ItemWriter> blanket impl for dynamic dispatch support"
  ```

---

### Task 4: Example `chaining_writers.rs`

**Files:**
- Create: `examples/chaining_writers.rs`
- Modify: `Cargo.toml` (add `[[example]]` entry)

- [ ] **Step 1: Create the example file**

  Create `examples/chaining_writers.rs` with this content:

  ```rust
  //! # Example: Chaining Item Writers (Fan-out)
  //!
  //! Demonstrates how to write the same chunk of items to multiple destinations
  //! simultaneously using [`CompositeItemWriterBuilder`].
  //!
  //! Each writer in the chain receives an identical slice of items. Writers are
  //! called in order; if any writer fails the chain short-circuits.
  //!
  //! This example models a product ingestion pipeline that simultaneously:
  //! 1. Logs each product to the console (audit trail)
  //! 2. Writes all products to a JSON file (persistence)
  //!
  //! ## Run
  //!
  //! ```bash
  //! cargo run --example chaining_writers --features csv,json,logger
  //! ```
  //!
  //! ## What It Does
  //!
  //! 1. Reads product records from an inline CSV string
  //! 2. Fans out each chunk to two writers in parallel:
  //!    - `LoggerWriter` — logs every item via the `log` crate
  //!    - `JsonItemWriter` — writes all items to a temp JSON file
  //! 3. Prints the output path and item counts

  use serde::{Deserialize, Serialize};
  use spring_batch_rs::{
      core::{
          item::CompositeItemWriterBuilder,
          job::{Job, JobBuilder},
          step::StepBuilder,
      },
      item::{
          csv::csv_reader::CsvItemReaderBuilder,
          json::json_writer::JsonItemWriterBuilder,
          logger::LoggerWriterBuilder,
      },
  };
  use std::env::temp_dir;

  /// A product record read from CSV and written to both destinations.
  #[derive(Debug, Deserialize, Serialize, Clone)]
  struct Product {
      id: u32,
      name: String,
      price: f64,
  }

  fn main() {
      let csv = "\
  id,name,price
  1,Widget,9.99
  2,Gadget,24.50
  3,Doohickey,4.75
  4,Thingamajig,14.00
  5,Whatsit,2.99";

      // 1. Build reader
      let reader = CsvItemReaderBuilder::<Product>::new()
          .has_headers(true)
          .from_reader(csv.as_bytes());

      // 2. Build individual writers
      let output = temp_dir().join("products.json");
      let json_writer = JsonItemWriterBuilder::<Product>::new().from_path(&output);
      let logger_writer = LoggerWriterBuilder::<Product>::new().build();

      // 3. Build composite fan-out writer: same items go to logger AND json file
      let composite = CompositeItemWriterBuilder::new(logger_writer)
          .add(json_writer)
          .build();

      // 4. Build step (no processor — items pass through unchanged)
      let step = StepBuilder::new("fan-out-products")
          .chunk::<Product, Product>(10)
          .reader(&reader)
          .writer(&composite)
          .build();

      // 5. Run job
      let job = JobBuilder::new().start(&step).build();
      job.run().expect("job failed"); // unwrap is intentional in examples — panics on error

      // 6. Report results
      let exec = job.get_step_execution("fan-out-products").unwrap();
      println!("JSON output: {}", output.display());
      println!("Read:    {}", exec.read_count);  // 5
      println!("Written: {}", exec.write_count); // 5 (to both writers)
  }
  ```

- [ ] **Step 2: Add the `[[example]]` entry to `Cargo.toml`**

  In `Cargo.toml`, find the `[[example]]` block for `chaining_processors` and add the new entry directly after it:

  ```toml
  [[example]]
  name = "chaining_writers"
  required-features = ["csv", "json", "logger"]
  ```

- [ ] **Step 3: Build the example to verify it compiles**

  ```bash
  cargo build --example chaining_writers --features csv,json,logger 2>&1
  ```

  Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

  ```bash
  git add examples/chaining_writers.rs Cargo.toml
  git commit -m "feat: add chaining_writers example demonstrating CompositeItemWriterBuilder fan-out"
  ```

---

### Task 5: Website update

**Files:**
- Modify: `website/src/content/docs/examples/advanced-patterns.mdx`

- [ ] **Step 1: Add the "Chaining Item Writers" section**

  In `website/src/content/docs/examples/advanced-patterns.mdx`, find the line:

  ```
  ---

  ## Multi-Step ETL Job
  ```

  Insert the following block **before** that line (between "Chaining Item Processors" and "Multi-Step ETL Job"):

  ````mdx
  ---

  ## Chaining Item Writers (Fan-out)

  <Aside type="tip">
    View the complete source: [examples/chaining_writers.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/chaining_writers.rs)
  </Aside>

  `CompositeItemWriterBuilder` lets you send the same chunk of items to multiple writers
  simultaneously. Writers are called in order; if any writer fails the chain short-circuits
  and returns the error.

  ### How It Works

  ```
  chunk → w1 → w2 → ... → wN   (all receive the same slice)
            ↓ Err?
           stop — error propagated
  ```

  ### Fan-out to Logger and JSON File

  ```rust
  use spring_batch_rs::core::item::{
      ItemWriter, CompositeItemWriterBuilder,
  };
  use spring_batch_rs::item::{
      json::json_writer::JsonItemWriterBuilder,
      logger::LoggerWriterBuilder,
  };
  use serde::{Deserialize, Serialize};
  use std::env::temp_dir;

  #[derive(Debug, Deserialize, Serialize, Clone)]
  struct Product { id: u32, name: String, price: f64 }

  let json_writer = JsonItemWriterBuilder::<Product>::new()
      .from_path(temp_dir().join("products.json"));
  let logger_writer = LoggerWriterBuilder::<Product>::new().build();

  // Both writers receive identical item slices on every chunk.
  let composite = CompositeItemWriterBuilder::new(logger_writer)
      .add(json_writer)
      .build();

  let step = StepBuilder::new("fan-out-products")
      .chunk::<Product, Product>(10)
      .reader(&reader)
      .writer(&composite)
      .build();
  ```

  ```bash
  cargo run --example chaining_writers --features csv,json,logger
  ```

  ### Error Behaviour

  | Scenario | Result |
  |---|---|
  | All writers succeed | `Ok(())` |
  | Writer N returns `Err(e)` | Chain stops — error propagated, writers N+1…M not called |

  <Aside type="tip">
    Chains of any length are supported. Each `.add()` call wraps the chain in a new
    `CompositeItemWriter`, encoding the full structure in the type at zero runtime cost.
    Use `Box&lt;dyn ItemWriter&lt;T&gt;&gt;` when you need dynamic dispatch instead.
  </Aside>
  ````

- [ ] **Step 2: Verify the website builds**

  ```bash
  cd website && npm run build 2>&1 | tail -20
  ```

  Expected: `Complete!` with no errors.

- [ ] **Step 3: Commit**

  ```bash
  git add website/src/content/docs/examples/advanced-patterns.mdx
  git commit -m "docs: add Chaining Item Writers section to advanced-patterns page"
  ```

---

### Task 6: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Run the full test suite**

  ```bash
  cargo test --all-features 2>&1 | tail -15
  ```

  Expected: all tests pass, no failures.

- [ ] **Step 2: Run clippy**

  ```bash
  cargo clippy --all-features -- -D warnings 2>&1 | grep -E "warning|error"
  ```

  Expected: no output (zero warnings policy).

- [ ] **Step 3: Verify doc build**

  ```bash
  cargo doc --no-deps --all-features 2>&1 | grep -E "warning|error"
  ```

  Expected: no output.

- [ ] **Step 4: Run doc tests**

  ```bash
  cargo test --doc --all-features 2>&1 | tail -10
  ```

  Expected: all doc tests pass.
