# ItemProcessor Owned Input Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change `ItemProcessor::process` to take `I` by value instead of `&I`, eliminating the forced clone in `PassThroughProcessor` and making the API more idiomatic Rust.

**Architecture:** Single trait signature change cascades to all implementations. `PassThroughProcessor` drops its `T: Clone` bound and returns the item directly. `ChunkOrientedStep::process_chunk` consumes `Vec<I>` instead of borrowing `&[I]`, so items flow through one allocation end-to-end.

**Tech Stack:** Rust 2021, no new dependencies.

---

## File Map

| File | Change |
|---|---|
| `src/core/item.rs` | Trait signature, PassThrough impl, Composite impl, Box blanket impl, doc examples, unit tests |
| `src/core/step.rs` | `process_chunk` + `process_and_write_chunk` consume `Vec<I>`; mock signature; doc comments |
| `tests/csv_integration.rs` | `fn process` signature |
| `tests/xml_integration.rs` | Two `fn process` signatures |
| `tests/integration_test.rs` | Two `fn process` signatures |
| `tests/mongodb.rs` | `fn process` signature |
| `tests/error_cases.rs` | `fn process` signature |
| `tests/orm_integration.rs` | Two `fn process` signatures |
| `examples/csv_processing.rs` | `fn process` signature |
| `examples/xml_processing.rs` | `fn process` signature |
| `examples/mongodb_processing.rs` | Two `fn process` signatures |
| `examples/advanced_patterns.rs` | Two `fn process` signatures |
| `examples/database_processing.rs` | `fn process` signature |
| `examples/chaining_processors.rs` | Three `fn process` signatures |
| `examples/json_processing.rs` | Two `fn process` signatures |
| `examples/benchmark_csv_postgres_xml.rs` | `fn process` signature + six direct `.process()` call sites |
| `examples/filter_records_from_csv_with_processor.rs` | `fn process` signature |
| `examples/orm_processing.rs` | Two `fn process` signatures |

---

## Task 1: Update core trait and `src/core/item.rs`

**Files:**
- Modify: `src/core/item.rs`

- [ ] **Step 1.1: Change the `ItemProcessor` trait signature**

  In `src/core/item.rs` at line 129, change:
  ```rust
  fn process(&self, item: &I) -> ItemProcessorResult<O>;
  ```
  to:
  ```rust
  fn process(&self, item: I) -> ItemProcessorResult<O>;
  ```

- [ ] **Step 1.2: Update `PassThroughProcessor` rustdoc and impl**

  Replace the `# Performance` paragraph and all three doc examples (lines ~244-285), the impl bound, and the method body.

  The struct-level doc `# Performance` block should read:
  ```rust
  /// # Performance
  ///
  /// This processor takes ownership of the item and returns it directly,
  /// with no allocation or clone.
  ```

  The first doc example (simple string) becomes:
  ```rust
  /// ```
  /// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
  ///
  /// let processor = PassThroughProcessor::<String>::new();
  /// let result = processor.process("Hello, World!".to_string()).unwrap();
  /// assert_eq!(result, Some("Hello, World!".to_string()));
  /// ```
  ```

  The second doc example (`i32` + custom struct) becomes:
  ```rust
  /// ```
  /// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
  ///
  /// let int_processor = PassThroughProcessor::<i32>::new();
  /// let result = int_processor.process(42).unwrap();
  /// assert_eq!(result, Some(42));
  ///
  /// #[derive(PartialEq, Debug)]
  /// struct Person { name: String, age: u32 }
  ///
  /// let person_processor = PassThroughProcessor::<Person>::new();
  /// let result = person_processor.process(Person { name: "Alice".to_string(), age: 30 }).unwrap();
  /// assert_eq!(result, Some(Person { name: "Alice".to_string(), age: 30 }));
  /// ```
  ```
  Note: `Clone` is no longer required on `Person`.

  The impl block (line ~291):
  ```rust
  // Before
  impl<T: Clone> ItemProcessor<T, T> for PassThroughProcessor<T> {
  // After
  impl<T> ItemProcessor<T, T> for PassThroughProcessor<T> {
  ```

  The method doc example inside the impl (line ~303-308):
  ```rust
  /// ```
  /// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
  ///
  /// let processor = PassThroughProcessor::<Vec<i32>>::new();
  /// let result = processor.process(vec![1, 2, 3]).unwrap();
  /// assert_eq!(result, Some(vec![1, 2, 3]));
  /// ```
  ```

  The method signature and body (line ~310-312):
  ```rust
  // Before
  fn process(&self, item: &T) -> ItemProcessorResult<T> {
      Ok(Some(item.clone()))
  }
  // After
  fn process(&self, item: T) -> ItemProcessorResult<T> {
      Ok(Some(item))
  }
  ```

  The constructor impl bound (line ~315):
  ```rust
  // Before
  impl<T: Clone> PassThroughProcessor<T> {
  // After
  impl<T> PassThroughProcessor<T> {
  ```

- [ ] **Step 1.3: Update `CompositeItemProcessor::process`**

  Lines ~410-415:
  ```rust
  // Before
  fn process(&self, item: &I) -> ItemProcessorResult<O> {
      match self.first.process(item)? {
          Some(intermediate) => self.second.process(&intermediate),
          None => Ok(None),
      }
  }
  // After
  fn process(&self, item: I) -> ItemProcessorResult<O> {
      match self.first.process(item)? {
          Some(intermediate) => self.second.process(intermediate),
          None => Ok(None),
      }
  }
  ```

- [ ] **Step 1.4: Update the `Box<P>` blanket impl**

  Lines ~942-945:
  ```rust
  // Before
  impl<I, O, P: ItemProcessor<I, O> + ?Sized> ItemProcessor<I, O> for Box<P> {
      fn process(&self, item: &I) -> ItemProcessorResult<O> {
          (**self).process(item)
      }
  }
  // After
  impl<I, O, P: ItemProcessor<I, O> + ?Sized> ItemProcessor<I, O> for Box<P> {
      fn process(&self, item: I) -> ItemProcessorResult<O> {
          (**self).process(item)
      }
  }
  ```

- [ ] **Step 1.5: Update all doc examples in `CompositeItemProcessor` and `CompositeItemProcessorBuilder`**

  Search for every occurrence of `.process(&` in doc comments within `src/core/item.rs` and remove the `&`:
  ```
  composite.process(&21)  →  composite.process(21)
  composite.process(&5)   →  composite.process(5)
  composite.process(&4)   →  composite.process(4)
  composite.process(&41)  →  composite.process(41)
  composite.process(&"hello".to_string())  →  composite.process("hello".to_string())
  ```
  Also update inline doc example processor impls in the comments: every `fn process(&self, item: &i32)` → `fn process(&self, item: i32)` and bodies like `Ok(Some(item * 2))` stay correct since `i32` is `Copy`.

- [ ] **Step 1.6: Update unit tests in `#[cfg(test)]` module**

  **Test helper structs** (around line 1137). Change every `fn process(&self, item: &T)` in the test helpers:
  ```rust
  impl ItemProcessor<i32, i32> for DoubleProcessor {
      fn process(&self, item: i32) -> ItemProcessorResult<i32> {
          Ok(Some(item * 2))
      }
  }

  impl ItemProcessor<i32, i32> for AddTenProcessor {
      fn process(&self, item: i32) -> ItemProcessorResult<i32> {
          Ok(Some(item + 10))
      }
  }

  impl ItemProcessor<i32, String> for ToStringProcessor {
      fn process(&self, item: i32) -> ItemProcessorResult<String> {
          Ok(Some(item.to_string()))
      }
  }

  impl ItemProcessor<i32, i32> for FilterEvenProcessor {
      fn process(&self, item: i32) -> ItemProcessorResult<i32> {
          if item % 2 == 0 { Ok(None) } else { Ok(Some(item)) }
      }
  }

  impl ItemProcessor<i32, i32> for FailingProcessor {
      fn process(&self, _item: i32) -> ItemProcessorResult<i32> {
          Err(BatchError::ItemProcessor("forced failure".to_string()))
      }
  }
  ```

  **`AlwaysFailI32`** (around line 1250):
  ```rust
  impl ItemProcessor<i32, i32> for AlwaysFailI32 {
      fn process(&self, _: i32) -> ItemProcessorResult<i32> {
          Err(BatchError::ItemProcessor("fail".to_string()))
      }
  }
  ```

  **Call sites** — change `.process(&n)` to `.process(n)`:
  ```rust
  composite.process(5)?     // was process(&5)
  composite.process(21)?    // was process(&21)
  composite.process(5)?     // was process(&5)
  composite.process(3)?     // was process(&3)
  composite.process(4)?     // was process(&4)
  composite.process(1)      // was process(&1)
  composite.process(5)      // was process(&5)
  boxed.process(3)?         // was process(&3)
  boxed.process(7)?         // was process(&7)
  ```

  **PassThrough unit tests** — rewrite tests that used the owned value after processing:

  `should_pass_through_string_unchanged`:
  ```rust
  #[test]
  fn should_pass_through_string_unchanged() -> Result<(), BatchError> {
      let processor = PassThroughProcessor::new();
      let result = processor.process("Hello, World!".to_string())?;
      assert_eq!(result, Some("Hello, World!".to_string()));
      Ok(())
  }
  ```

  `should_pass_through_integer_unchanged`:
  ```rust
  #[test]
  fn should_pass_through_integer_unchanged() -> Result<(), BatchError> {
      let processor = PassThroughProcessor::new();
      let result = processor.process(42i32)?;
      assert_eq!(result, Some(42));
      Ok(())
  }
  ```

  `should_pass_through_vector_unchanged`:
  ```rust
  #[test]
  fn should_pass_through_vector_unchanged() -> Result<(), BatchError> {
      let processor = PassThroughProcessor::new();
      let result = processor.process(vec![1, 2, 3, 4, 5])?;
      assert_eq!(result, Some(vec![1, 2, 3, 4, 5]));
      Ok(())
  }
  ```

  `should_pass_through_custom_struct_unchanged`: remove `Clone` derive, update call:
  ```rust
  #[test]
  fn should_pass_through_custom_struct_unchanged() -> Result<(), BatchError> {
      #[derive(PartialEq, Debug)]
      struct TestData { id: u32, name: String, values: Vec<f64> }

      let processor = PassThroughProcessor::new();
      let result = processor.process(TestData {
          id: 123,
          name: "Test Item".to_string(),
          values: vec![1.1, 2.2, 3.3],
      })?;
      assert_eq!(result, Some(TestData {
          id: 123,
          name: "Test Item".to_string(),
          values: vec![1.1, 2.2, 3.3],
      }));
      Ok(())
  }
  ```

  `should_pass_through_option_unchanged`:
  ```rust
  #[test]
  fn should_pass_through_option_unchanged() -> Result<(), BatchError> {
      let processor = PassThroughProcessor::new();
      let result_some = processor.process(Some("test".to_string()))?;
      assert_eq!(result_some, Some(Some("test".to_string())));
      let result_none = processor.process(None::<String>)?;
      assert_eq!(result_none, Some(None::<String>));
      Ok(())
  }
  ```

  `should_handle_empty_collections`:
  ```rust
  #[test]
  fn should_handle_empty_collections() -> Result<(), BatchError> {
      let result_vec = PassThroughProcessor::new().process(Vec::<i32>::new())?;
      assert_eq!(result_vec, Some(vec![]));
      let result_string = PassThroughProcessor::new().process(String::new())?;
      assert_eq!(result_string, Some(String::new()));
      Ok(())
  }
  ```

  **Delete** `should_clone_input_not_move` entirely — it tested the old clone behaviour which no longer exists. Replace with a test that verifies ownership transfer:
  ```rust
  #[test]
  fn should_take_ownership_of_input() {
      let processor = PassThroughProcessor::new();
      let input = "original".to_string();
      // input is moved here; the result owns the value
      let result = processor.process(input).unwrap();
      assert_eq!(result, Some("original".to_string()));
      // `input` is no longer accessible — correct Rust ownership
  }
  ```

  `should_work_with_multiple_processors`:
  ```rust
  #[test]
  fn should_work_with_multiple_processors() -> Result<(), BatchError> {
      let processor1 = PassThroughProcessor::<String>::new();
      let processor2 = PassThroughProcessor::<String>::new();
      let inner = processor1.process("test data".to_string())?.unwrap();
      let result = processor2.process(inner)?;
      assert_eq!(result, Some("test data".to_string()));
      Ok(())
  }
  ```

  `should_handle_large_data_structures`:
  ```rust
  #[test]
  fn should_handle_large_data_structures() -> Result<(), BatchError> {
      let processor = PassThroughProcessor::new();
      let large_input: Vec<i32> = (0..10000).collect();
      let expected_len = large_input.len();
      let result = processor.process(large_input)?;
      assert_eq!(result.unwrap().len(), expected_len);
      Ok(())
  }
  ```

- [ ] **Step 1.7: Verify `src/` compiles cleanly**

  ```bash
  cargo build --all-features 2>&1 | grep -E "^error"
  ```
  Expected: no errors (warnings about step.rs doc comments are OK at this stage).

- [ ] **Step 1.8: Commit**

  ```bash
  git add src/core/item.rs
  git commit -m "refactor: ItemProcessor::process takes I by value, drop Clone bound on PassThrough"
  ```

---

## Task 2: Update `src/core/step.rs`

**Files:**
- Modify: `src/core/step.rs`

- [ ] **Step 2.1: Consume `Vec<I>` in `process_and_write_chunk`**

  Change signature and forward call:
  ```rust
  // Before (line ~690)
  fn process_and_write_chunk(
      &self,
      step_execution: &mut StepExecution,
      read_items: &[I],
  ) -> Result<(), BatchError> {
      let processed_items = match self.process_chunk(step_execution, read_items) {

  // After
  fn process_and_write_chunk(
      &self,
      step_execution: &mut StepExecution,
      read_items: Vec<I>,
  ) -> Result<(), BatchError> {
      let processed_items = match self.process_chunk(step_execution, read_items) {
  ```

- [ ] **Step 2.2: Consume `Vec<I>` in `process_chunk`**

  ```rust
  // Before (line ~785)
  fn process_chunk(
      &self,
      step_execution: &mut StepExecution,
      read_items: &[I],
  ) -> Result<Vec<O>, BatchError> {
      debug!("Processing chunk of {} items", read_items.len());
      let mut result = Vec::with_capacity(read_items.len());
      for item in read_items {
          match self.processor.process(item) {

  // After
  fn process_chunk(
      &self,
      step_execution: &mut StepExecution,
      read_items: Vec<I>,
  ) -> Result<Vec<O>, BatchError> {
      debug!("Processing chunk of {} items", read_items.len());
      let mut result = Vec::with_capacity(read_items.len());
      for item in read_items {
          match self.processor.process(item) {
  ```
  The rest of the method body is unchanged.

- [ ] **Step 2.3: Update the call site in the main loop**

  Line ~636:
  ```rust
  // Before
  if self
      .process_and_write_chunk(step_execution, &read_items)
      .is_err()

  // After
  if self
      .process_and_write_chunk(step_execution, read_items)
      .is_err()
  ```

- [ ] **Step 2.4: Update the mock in tests**

  Line ~1464:
  ```rust
  // Before
  mock! {
      pub TestProcessor {}
      impl ItemProcessor<Car, Car> for TestProcessor {
          fn process(&self, item: &Car) -> ItemProcessorResult<Car>;
      }
  }

  // After
  mock! {
      pub TestProcessor {}
      impl ItemProcessor<Car, Car> for TestProcessor {
          fn process(&self, item: Car) -> ItemProcessorResult<Car>;
      }
  }
  ```

- [ ] **Step 2.5: Update all doc comments in step.rs**

  Search for every `fn process(&self, item: &` pattern in doc comment lines (`///` and `//!`) and change to `fn process(&self, item: ` (remove the `&`). Also update bodies like `Ok(Some(item.clone()))` to `Ok(Some(item.to_string()))` or `Ok(Some(item.to_uppercase()))` as appropriate.

  Specific occurrences (line numbers are approximate — verify with grep):
  - `//!` module doc (line ~43): `fn process(&self, item: &String) -> ... { Ok(Some(item.clone())) }` → `fn process(&self, item: String) -> ... { Ok(Some(item)) }`
  - `///` on `ChunkOrientedStep` struct (line ~566): same
  - `///` on `StepBuilder::processor` (line ~1015): `fn process(&self, item: &String) -> ... { Ok(Some(item.to_uppercase())) }` → `fn process(&self, item: String) -> ... { Ok(Some(item.to_uppercase())) }`
  - `///` on `StepBuilder::writer` (line ~1047): same
  - `///` on `StepBuilder::chunk` (line ~1130): `fn process(&self, item: &String) -> ... { Ok(Some(item.clone())) }` → `fn process(&self, item: String) -> ... { Ok(Some(item)) }`
  - `///` on `StepBuilder` (line ~1190): same
  - `///` on `StepBuilder::build` (line ~1300): same
  - `///` on `StepBuilder` (line ~903-904): `fn process(&self, item: &i32) -> ... { Ok(Some(item.to_string())) }` → `fn process(&self, item: i32) -> ... { Ok(Some(item.to_string())) }`

- [ ] **Step 2.6: Verify `src/` compiles and unit tests pass**

  ```bash
  cargo test --lib --all-features 2>&1 | tail -20
  ```
  Expected: all tests pass, no compilation errors.

- [ ] **Step 2.7: Commit**

  ```bash
  git add src/core/step.rs
  git commit -m "refactor: ChunkOrientedStep consumes Vec<I> through process pipeline"
  ```

---

## Task 3: Update integration tests

**Files:**
- Modify: `tests/csv_integration.rs`, `tests/xml_integration.rs`, `tests/integration_test.rs`, `tests/mongodb.rs`, `tests/error_cases.rs`, `tests/orm_integration.rs`

For every `impl ItemProcessor<A, B> for Foo` in each test file, change:
```rust
fn process(&self, item: &A) -> ItemProcessorResult<B>
```
to:
```rust
fn process(&self, item: A) -> ItemProcessorResult<B>
```
and update the body to access `item` instead of `*item` (for `Copy` types the body is usually unchanged; for heap types, remove `.clone()` calls if any).

- [ ] **Step 3.1: `tests/csv_integration.rs` line 36-38**

  ```rust
  // Before
  impl ItemProcessor<Product, Product> for ProductProcessor {
      fn process(&self, item: &Product) -> ItemProcessorResult<Product> {
          Ok(Some(item.clone()))
      }
  }
  // After
  impl ItemProcessor<Product, Product> for ProductProcessor {
      fn process(&self, item: Product) -> ItemProcessorResult<Product> {
          Ok(Some(item))
      }
  }
  ```
  Also remove `Clone` from `#[derive(...)]` on `Product` if it is no longer needed elsewhere in the file. (Check with grep before removing.)

- [ ] **Step 3.2: `tests/xml_integration.rs` line 39-41**

  ```rust
  impl ItemProcessor<Product, Product> for ProductProcessor {
      fn process(&self, item: Product) -> ItemProcessorResult<Product> {
          Ok(Some(item))
      }
  }
  ```

- [ ] **Step 3.3: `tests/xml_integration.rs` line 189-191** (`CsvToEnhancedProductProcessor`)

  ```rust
  impl ItemProcessor<Vec<String>, EnhancedProduct> for CsvToEnhancedProductProcessor {
      fn process(&self, item: Vec<String>) -> ItemProcessorResult<EnhancedProduct> {
  ```
  The body accesses `item[0]`, `item[1]`, etc. — these index operations still work on owned `Vec`. No other changes needed in the body.

- [ ] **Step 3.4: `tests/integration_test.rs` line 54-56** (`UpperCaseProcessor`)

  ```rust
  impl ItemProcessor<Person, Person> for UpperCaseProcessor {
      fn process(&self, item: Person) -> ItemProcessorResult<Person> {
          Ok(Some(Person { name: item.name.to_uppercase(), ..item }))
      }
  }
  ```
  Using struct update syntax `..item` to move remaining fields without a full clone.

- [ ] **Step 3.5: `tests/integration_test.rs` line 71-73** (`CarProcessor`)

  Read the current body first, then update:
  ```rust
  impl ItemProcessor<Car, Car> for CarProcessor {
      fn process(&self, item: Car) -> ItemProcessorResult<Car> {
  ```
  If the body does `Ok(Some(item.clone()))`, change to `Ok(Some(item))`. If it transforms fields, update field accesses from `item.field` (already works on owned).

- [ ] **Step 3.6: `tests/mongodb.rs` line 49-51** (`FormatBookProcessor`)

  ```rust
  impl ItemProcessor<Book, FormattedBook> for FormatBookProcessor {
      fn process(&self, item: Book) -> ItemProcessorResult<FormattedBook> {
  ```
  Body accesses `&item.title` etc. — these work on owned `item` too. No body changes needed.

- [ ] **Step 3.7: `tests/error_cases.rs` line 63-65** (`CarProcessor`)

  Same pattern as Step 3.5. Change signature; update body if it clones.

- [ ] **Step 3.8: `tests/orm_integration.rs` line 59-61** (`ProductTransformProcessor`)

  ```rust
  impl ItemProcessor<Model, ProductDto> for ProductTransformProcessor {
      fn process(&self, item: Model) -> ItemProcessorResult<ProductDto> {
  ```

- [ ] **Step 3.9: `tests/orm_integration.rs` line 774-776** (`ProductDtoToActiveModelProcessor`)

  ```rust
  impl ItemProcessor<ProductInsertDto, ActiveModel> for ProductDtoToActiveModelProcessor {
      fn process(&self, item: ProductInsertDto) -> ItemProcessorResult<ActiveModel> {
  ```

- [ ] **Step 3.10: Verify all tests that don't need Docker pass**

  ```bash
  cargo test --test csv_integration --test xml_integration --test integration_test --test error_cases --all-features 2>&1 | tail -20
  ```
  Expected: all pass.

- [ ] **Step 3.11: Commit**

  ```bash
  git add tests/
  git commit -m "test: update ItemProcessor impls in integration tests for owned input"
  ```

---

## Task 4: Update examples

**Files:** All example files listed in the file map.

For each file, the change is the same pattern: `fn process(&self, item: &A) -> ...` → `fn process(&self, item: A) -> ...` and update body accordingly.

- [ ] **Step 4.1: `examples/csv_processing.rs` line 56-57** (`DiscountProcessor`)

  ```rust
  impl ItemProcessor<Product, Product> for DiscountProcessor {
      fn process(&self, item: Product) -> Result<Option<Product>, BatchError> {
          Ok(Some(Product { price: item.price * 0.9, ..item }))
      }
  }
  ```
  Check the actual body first; adapt struct update syntax if it matches.

- [ ] **Step 4.2: `examples/xml_processing.rs` line 79-80** (`HouseToCsvProcessor`)

  ```rust
  impl ItemProcessor<House, HouseCsv> for HouseToCsvProcessor {
      fn process(&self, item: House) -> Result<Option<HouseCsv>, BatchError> {
  ```
  Body builds a `HouseCsv` from `item` fields — owned access works the same.

- [ ] **Step 4.3: `examples/mongodb_processing.rs` lines 90-91 and 104-105**

  ```rust
  impl ItemProcessor<Book, BookCsv> for BookToCsvProcessor {
      fn process(&self, item: Book) -> Result<Option<BookCsv>, BatchError> {

  impl ItemProcessor<BookInput, Book> for BookFromCsvProcessor {
      fn process(&self, item: BookInput) -> Result<Option<Book>, BatchError> {
  ```

- [ ] **Step 4.4: `examples/advanced_patterns.rs` lines 85-86 and 123-124**

  ```rust
  impl ItemProcessor<RawTransaction, ValidTransaction> for ValidationProcessor {
      fn process(&self, item: RawTransaction) -> Result<Option<ValidTransaction>, BatchError> {

  impl ItemProcessor<ValidTransaction, EnrichedTransaction> for EnrichmentProcessor {
      fn process(&self, item: ValidTransaction) -> Result<Option<EnrichedTransaction>, BatchError> {
  ```

- [ ] **Step 4.5: `examples/database_processing.rs` line 86-87** (`ActivateUserProcessor`)

  ```rust
  impl ItemProcessor<User, User> for ActivateUserProcessor {
      fn process(&self, item: User) -> Result<Option<User>, BatchError> {
  ```

- [ ] **Step 4.6: `examples/chaining_processors.rs` lines 77-78, 100-101, 113-114**

  ```rust
  impl ItemProcessor<RawOrder, ParsedOrder> for ParseProcessor {
      fn process(&self, item: RawOrder) -> ItemProcessorResult<ParsedOrder> {

  impl ItemProcessor<ParsedOrder, ParsedOrder> for ValidateProcessor {
      fn process(&self, item: ParsedOrder) -> ItemProcessorResult<ParsedOrder> {

  impl ItemProcessor<ParsedOrder, EnrichedOrder> for EnrichProcessor {
      fn process(&self, item: ParsedOrder) -> ItemProcessorResult<EnrichedOrder> {
  ```

- [ ] **Step 4.7: `examples/json_processing.rs` lines 56-57 and 77-78**

  ```rust
  impl ItemProcessor<Order, OrderSummary> for OrderSummaryProcessor {
      fn process(&self, item: Order) -> Result<Option<OrderSummary>, BatchError> {

  impl ItemProcessor<Order, Order> for CompletedOrderProcessor {
      fn process(&self, item: Order) -> Result<Option<Order>, BatchError> {
  ```

- [ ] **Step 4.8: `examples/filter_records_from_csv_with_processor.rs` line 46-47**

  ```rust
  impl ItemProcessor<Person, Person> for AdultFilter {
      fn process(&self, item: Person) -> ItemProcessorResult<Person> {
          if item.age >= 18 { Ok(Some(item)) } else { Ok(None) }
      }
  }
  ```

- [ ] **Step 4.9: `examples/orm_processing.rs` lines 90-91 and 104-105**

  ```rust
  impl ItemProcessor<products::Model, ProductCsv> for ProductToCsvProcessor {
      fn process(&self, item: products::Model) -> Result<Option<ProductCsv>, BatchError> {

  impl ItemProcessor<ProductDto, products::ActiveModel> for DtoToActiveModelProcessor {
      fn process(&self, item: ProductDto) -> Result<Option<products::ActiveModel>, BatchError> {
  ```

- [ ] **Step 4.10: `examples/benchmark_csv_postgres_xml.rs` — impl + 6 call sites**

  Signature update (line 82-83):
  ```rust
  impl ItemProcessor<Transaction, Transaction> for TransactionProcessor {
      fn process(&self, item: Transaction) -> Result<Option<Transaction>, BatchError> {
  ```

  Each test function (lines ~229, 238, 246, 254, 266, 280) replaces `processor.process(&input)` with `processor.process(input)`. Since `input` is not used after the call, this compiles cleanly. The loop test (line ~264) creates `input` inside the loop body so consuming it is fine:
  ```rust
  for status in &["PENDING", "COMPLETED", "FAILED"] {
      let input = make_transaction("EUR", 100.0, status);
      let result = processor.process(input).unwrap();
      // ...
  }
  ```

- [ ] **Step 4.11: Verify all examples compile**

  ```bash
  cargo build --examples --all-features 2>&1 | grep "^error"
  ```
  Expected: no errors.

- [ ] **Step 4.12: Commit**

  ```bash
  git add examples/
  git commit -m "refactor: update all example ItemProcessor impls for owned input"
  ```

---

## Task 5: Final verification and PR

- [ ] **Step 5.1: Full quality check**

  ```bash
  make check
  ```
  Expected output: formatting OK, clippy clean (zero warnings), audit clean.

- [ ] **Step 5.2: Full test suite**

  ```bash
  make test
  ```
  Expected: all tests pass (DB integration tests require Docker).

- [ ] **Step 5.3: Doc build**

  ```bash
  cargo doc --no-deps --all-features 2>&1 | grep -E "warning|error"
  ```
  Expected: zero warnings, zero errors.

- [ ] **Step 5.4: Doc tests**

  ```bash
  cargo test --doc --all-features 2>&1 | tail -10
  ```
  Expected: all doc tests pass.

- [ ] **Step 5.5: Create PR**

  ```bash
  git push -u origin refactor/processor-owned-input
  gh pr create \
    --title "refactor: ItemProcessor::process takes I by value" \
    --body "$(cat <<'EOF'
  ## Summary

  - `ItemProcessor::process` signature changed from `fn process(&self, item: &I)` to `fn process(&self, item: I)`
  - `PassThroughProcessor` drops its `T: Clone` bound and returns the item directly with no allocation
  - `ChunkOrientedStep::process_chunk` and `process_and_write_chunk` now consume `Vec<I>`, so items move through one allocation end-to-end without an intermediate `&[I]` borrow
  - All implementations (tests, examples) updated to match

  ## Breaking change

  Any downstream code implementing `ItemProcessor` must update `fn process(&self, item: &I)` to `fn process(&self, item: I)`. Bodies that previously did `item.clone()` to pass through can now return `item` directly.

  ## Test plan

  - [ ] `make check` — format, clippy, audit
  - [ ] `make test` — full test suite (requires Docker for DB tests)
  - [ ] `cargo test --doc --all-features` — all doc tests

  🤖 Generated with [Claude Code](https://claude.com/claude-code)
  EOF
  )"
  ```
