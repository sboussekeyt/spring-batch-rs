# Processor Item Filtering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Permettre à un `ItemProcessor` de filtrer des items en retournant `Ok(None)`, en changeant `ItemProcessorResult<O>` de `Result<O, BatchError>` à `Result<Option<O>, BatchError>`, et en ajoutant un compteur `filter_count` dans `StepExecution`.

**Architecture:** Le changement est centré sur le type `ItemProcessorResult` dans `src/core/item.rs`. La logique de `process_chunk()` dans `src/core/step.rs` est mise à jour pour gérer `Ok(None)` comme un filtrage silencieux. Tous les processors existants et les exemples sont mis à jour pour retourner `Ok(Some(...))`.

**Tech Stack:** Rust 2021, mockall (tests), serde (exemples), feature flags `csv,json`

---

### Task 1 : Mettre à jour `ItemProcessorResult` et `PassThroughProcessor` dans `src/core/item.rs`

**Files:**
- Modify: `src/core/item.rs`

- [ ] **Step 1 : Mettre à jour le type et la doc de `ItemProcessorResult`**

Dans `src/core/item.rs`, remplacer les lignes 11-16 :

```rust
/// Represents the result of processing an item by the processor.
///
/// This type is a specialized `Result` that can be:
/// - `Ok(Some(O))` when an item is successfully processed and should be passed to the writer
/// - `Ok(None)` when an item is intentionally filtered out (not an error)
/// - `Err(BatchError)` when an error occurs during processing
pub type ItemProcessorResult<O> = Result<Option<O>, BatchError>;
```

- [ ] **Step 2 : Mettre à jour la doc du trait `ItemProcessor`**

Dans `src/core/item.rs`, remplacer le bloc de doc du trait (lignes 75-115) :

```rust
/// A trait for processing items.
///
/// This trait defines the contract for components that transform or process items
/// in a batch processing pipeline. It takes an input item of type `I` and produces
/// an output item of type `O`.
///
/// # Filtering
///
/// Returning `Ok(None)` filters the item silently: it is not passed to the writer
/// and is counted in [`StepExecution::filter_count`]. This is different from returning
/// `Err(BatchError)` which counts as a processing error and may trigger fault tolerance.
///
/// # Design Pattern
///
/// This follows the Strategy Pattern, allowing different processing strategies to be
/// interchangeable while maintaining a consistent interface.
///
/// # Type Parameters
///
/// - `I`: The input item type
/// - `O`: The output item type
///
/// # Example
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, ItemProcessorResult};
/// use spring_batch_rs::error::BatchError;
///
/// struct AdultFilter;
///
/// #[derive(Clone)]
/// struct Person { name: String, age: u32 }
///
/// impl ItemProcessor<Person, Person> for AdultFilter {
///     fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
///         if item.age >= 18 {
///             Ok(Some(item.clone())) // keep adults
///         } else {
///             Ok(None) // filter out minors
///         }
///     }
/// }
/// ```
pub trait ItemProcessor<I, O> {
    /// Processes an item and returns the processed result.
    ///
    /// # Parameters
    /// - `item`: The item to process
    ///
    /// # Returns
    /// - `Ok(Some(processed_item))` when the item is successfully processed
    /// - `Ok(None)` when the item is intentionally filtered out
    /// - `Err(BatchError)` when an error occurs during processing
    fn process(&self, item: &I) -> ItemProcessorResult<O>;
}
```

- [ ] **Step 3 : Mettre à jour `PassThroughProcessor::process`**

Remplacer la méthode `process` de `PassThroughProcessor` (ligne 295) :

```rust
fn process(&self, item: &T) -> ItemProcessorResult<T> {
    Ok(Some(item.clone()))
}
```

Et mettre à jour sa doc :

```rust
/// Processes an item by returning it unchanged.
///
/// # Parameters
/// - `item`: The item to process (will be cloned and returned unchanged)
///
/// # Returns
/// - `Ok(Some(cloned_item))` - Always succeeds and returns a clone of the input item
///
/// # Examples
///
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
///
/// let processor = PassThroughProcessor::<Vec<i32>>::new();
/// let input = vec![1, 2, 3];
/// let result = processor.process(&input).unwrap();
/// assert_eq!(result, Some(input));
/// ```
fn process(&self, item: &T) -> ItemProcessorResult<T> {
    Ok(Some(item.clone()))
}
```

- [ ] **Step 4 : Mettre à jour les doc-tests de `PassThroughProcessor` dans la doc de la struct**

Dans la doc de la struct `PassThroughProcessor` (ligne 239), remplacer les assertions :

```rust
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
///
/// let processor = PassThroughProcessor::<String>::new();
/// let input = "Hello, World!".to_string();
/// let result = processor.process(&input).unwrap();
/// assert_eq!(result, Some(input));
/// ```
```

Et le second exemple (ligne 248) :

```rust
/// ```
/// use spring_batch_rs::core::item::{ItemProcessor, PassThroughProcessor};
///
/// // With integers
/// let int_processor = PassThroughProcessor::<i32>::new();
/// let number = 42;
/// let result = int_processor.process(&number).unwrap();
/// assert_eq!(result, Some(number));
///
/// // With custom structs
/// #[derive(Clone, PartialEq, Debug)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person_processor = PassThroughProcessor::<Person>::new();
/// let person = Person {
///     name: "Alice".to_string(),
///     age: 30,
/// };
/// let result = person_processor.process(&person).unwrap();
/// assert_eq!(result, Some(person));
/// ```
```

- [ ] **Step 5 : Mettre à jour les tests inline de `PassThroughProcessor`**

Dans le bloc `#[cfg(test)]` (à partir de la ligne 320), mettre à jour tous les tests qui vérifient le résultat de `process()` :

```rust
#[test]
fn should_pass_through_string_unchanged() -> Result<(), BatchError> {
    let processor = PassThroughProcessor::new();
    let input = "Hello, World!".to_string();
    let expected = input.clone();

    let result = processor.process(&input)?;

    assert_eq!(result, Some(expected));
    Ok(())
}

#[test]
fn should_pass_through_integer_unchanged() -> Result<(), BatchError> {
    let processor = PassThroughProcessor::new();
    let input = 42i32;

    let result = processor.process(&input)?;

    assert_eq!(result, Some(input));
    Ok(())
}

#[test]
fn should_pass_through_vector_unchanged() -> Result<(), BatchError> {
    let processor = PassThroughProcessor::new();
    let input = vec![1, 2, 3, 4, 5];
    let expected = input.clone();

    let result = processor.process(&input)?;

    assert_eq!(result, Some(expected));
    Ok(())
}

#[test]
fn should_pass_through_custom_struct_unchanged() -> Result<(), BatchError> {
    #[derive(Clone, PartialEq, Debug)]
    struct TestData {
        id: u32,
        name: String,
        values: Vec<f64>,
    }

    let processor = PassThroughProcessor::new();
    let input = TestData {
        id: 123,
        name: "Test Item".to_string(),
        values: vec![1.1, 2.2, 3.3],
    };
    let expected = input.clone();

    let result = processor.process(&input)?;

    assert_eq!(result, Some(expected));
    Ok(())
}

#[test]
fn should_pass_through_option_unchanged() -> Result<(), BatchError> {
    let processor = PassThroughProcessor::new();

    let input_some = Some("test".to_string());
    let result_some = processor.process(&input_some)?;
    assert_eq!(result_some, Some(input_some));

    let input_none: Option<String> = None;
    let result_none = processor.process(&input_none)?;
    assert_eq!(result_none, Some(input_none));

    Ok(())
}

#[test]
fn should_handle_empty_collections() -> Result<(), BatchError> {
    let vec_processor = PassThroughProcessor::new();
    let empty_vec: Vec<i32> = vec![];
    let result_vec = vec_processor.process(&empty_vec)?;
    assert_eq!(result_vec, Some(empty_vec));

    let string_processor = PassThroughProcessor::new();
    let empty_string = String::new();
    let result_string = string_processor.process(&empty_string)?;
    assert_eq!(result_string, Some(empty_string));

    Ok(())
}

#[test]
fn should_clone_input_not_move() {
    let processor = PassThroughProcessor::new();
    let input = "original".to_string();
    let input_copy = input.clone();

    let _result = processor.process(&input).unwrap();

    assert_eq!(input, input_copy);
    assert_eq!(input, "original");
}

#[test]
fn should_work_with_multiple_processors() -> Result<(), BatchError> {
    let processor1 = PassThroughProcessor::<String>::new();
    let processor2 = PassThroughProcessor::<String>::new();

    let input = "test data".to_string();
    let result1 = processor1.process(&input)?;
    // result1 is Some(String), unwrap to pass to second processor
    let inner = result1.unwrap();
    let result2 = processor2.process(&inner)?;

    assert_eq!(result2, Some(input));
    Ok(())
}

#[test]
fn should_handle_large_data_structures() -> Result<(), BatchError> {
    let processor = PassThroughProcessor::new();

    let large_input: Vec<i32> = (0..10000).collect();
    let expected_len = large_input.len();

    let result = processor.process(&large_input)?;

    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), expected_len);
    Ok(())
}
```

- [ ] **Step 6 : Compiler et vérifier**

```bash
cargo build --all-features 2>&1 | head -40
```

Attendu : erreurs de compilation dans `step.rs` et les exemples (normal à ce stade — on a changé le type).

- [ ] **Step 7 : Commit**

```bash
git add src/core/item.rs
git commit -m "feat(core): change ItemProcessorResult to Result<Option<O>, BatchError>"
```

---

### Task 2 : Mettre à jour `StepExecution` et `process_chunk` dans `src/core/step.rs`

**Files:**
- Modify: `src/core/step.rs`

- [ ] **Step 1 : Ajouter `filter_count` à `StepExecution`**

Dans la struct `StepExecution` (ligne 331), ajouter après `process_error_count` :

```rust
/// Number of items filtered by the processor (processor returned Ok(None))
pub filter_count: usize,
```

Et dans `StepExecution::new()` (ligne 377), ajouter dans l'initialisation :

```rust
filter_count: 0,
```

Et mettre à jour la doc de la struct, en ajoutant dans l'example :

```rust
/// ```rust
/// use spring_batch_rs::core::step::{StepExecution, StepStatus};
///
/// let mut step_execution = StepExecution::new("data-processing-step");
/// assert_eq!(step_execution.status, StepStatus::Starting);
/// assert_eq!(step_execution.read_count, 0);
/// assert_eq!(step_execution.write_count, 0);
/// assert_eq!(step_execution.filter_count, 0);
/// ```
```

- [ ] **Step 2 : Mettre à jour `process_chunk` pour gérer `Ok(None)`**

Remplacer la méthode `process_chunk` (ligne 781-809) :

```rust
fn process_chunk(
    &self,
    step_execution: &mut StepExecution,
    read_items: &[I],
) -> Result<Vec<O>, BatchError> {
    debug!("Processing chunk of {} items", read_items.len());
    let mut result = Vec::with_capacity(read_items.len());

    for item in read_items {
        match self.processor.process(item) {
            Ok(Some(processed_item)) => {
                result.push(processed_item);
                step_execution.process_count += 1;
            }
            Ok(None) => {
                step_execution.filter_count += 1;
                debug!("Item filtered by processor");
            }
            Err(error) => {
                warn!("Error processing item: {}", error);
                step_execution.process_error_count += 1;

                if self.is_skip_limit_reached(step_execution) {
                    step_execution.status = StepStatus::ProcessorError;
                    return Err(error);
                }
            }
        }
    }

    Ok(result)
}
```

- [ ] **Step 3 : Mettre à jour `mock_process` dans les tests**

Dans le bloc `#[cfg(test)]` (ligne 1502), remplacer `mock_process` :

```rust
fn mock_process(i: &mut u16, error_at: &[u16]) -> ItemProcessorResult<Car> {
    *i += 1;
    if error_at.contains(i) {
        return Err(BatchError::ItemProcessor("mock process error".to_string()));
    }

    let car = Car {
        year: 1979,
        make: "make".to_owned(),
        model: "model".to_owned(),
        description: "description".to_owned(),
    };
    Ok(Some(car))
}
```

- [ ] **Step 4 : Mettre à jour les doc-tests inline de `step.rs` qui utilisent `process`**

Chercher toutes les occurrences de `-> Result<String, BatchError> { Ok(item.clone()) }` et `-> Result<String, BatchError> { Ok(item.to_uppercase()) }` et `-> Result<String, BatchError> { Ok(item.to_string()) }` dans les commentaires de `step.rs`, et les remplacer par :

```rust
// -> Result<Option<String>, BatchError> { Ok(Some(item.clone())) }
// -> Result<Option<String>, BatchError> { Ok(Some(item.to_uppercase())) }
// -> Result<Option<String>, BatchError> { Ok(Some(item.to_string())) }
```

Concrètement, rechercher avec :
```bash
grep -n "Result<String, BatchError> { Ok(" src/core/step.rs
grep -n "Result<Car, BatchError> { Ok(" src/core/step.rs
grep -n "Result<i32\|Result<u32" src/core/step.rs
```

Et remplacer chaque occurrence dans les commentaires `//!` et `///` de la forme :
- `-> Result<X, BatchError> { Ok(item.clone()) }` → `-> Result<Option<X>, BatchError> { Ok(Some(item.clone())) }`
- `-> Result<X, BatchError> { Ok(item.to_uppercase()) }` → `-> Result<Option<X>, BatchError> { Ok(Some(item.to_uppercase())) }`
- `-> Result<X, BatchError> { Ok(item.to_string()) }` → `-> Result<Option<X>, BatchError> { Ok(Some(item.to_string())) }`

- [ ] **Step 5 : Ajouter un test de filtrage**

Dans le bloc `#[cfg(test)]` de `step.rs`, ajouter après le dernier test existant :

```rust
#[test]
fn step_should_count_filtered_items() -> Result<()> {
    // Reader returns 4 items (items 0,1,2,3), ends at 4
    let mut i = 0u16;
    let mut reader = MockTestItemReader::default();
    reader
        .expect_read()
        .returning(move || mock_read(&mut i, 0, 4));

    // Processor filters item at position 2 (returns Ok(None))
    let mut j = 0u16;
    let mut processor = MockTestProcessor::default();
    processor.expect_process().returning(move |_| {
        j += 1;
        if j == 2 {
            return Ok(None); // filter this item
        }
        Ok(Some(Car {
            year: 1979,
            make: "make".to_owned(),
            model: "model".to_owned(),
            description: "description".to_owned(),
        }))
    });

    let mut writer = MockTestItemWriter::default();
    writer.expect_open().times(1).returning(|| Ok(()));
    // 3 items pass through (4 read - 1 filtered), written in one chunk
    writer
        .expect_write()
        .times(1)
        .returning(|items| {
            assert_eq!(items.len(), 3, "expected 3 items written after filtering");
            Ok(())
        });
    writer.expect_flush().returning(|| Ok(()));
    writer.expect_close().times(1).returning(|| Ok(()));

    let step = StepBuilder::new("test")
        .chunk(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let mut step_execution = StepExecution::new(&step.name);
    let result = step.execute(&mut step_execution);

    assert!(result.is_ok());
    assert_eq!(step_execution.read_count, 4, "should have read 4 items");
    assert_eq!(step_execution.filter_count, 1, "should have filtered 1 item");
    assert_eq!(step_execution.process_count, 3, "should have processed 3 items");
    assert_eq!(step_execution.write_count, 3, "should have written 3 items");

    Ok(())
}
```

- [ ] **Step 6 : Compiler**

```bash
cargo build --features csv,json 2>&1 | head -40
```

Attendu : erreurs dans les exemples uniquement (les exemples utilisent encore `Ok(item)` au lieu de `Ok(Some(item))`).

- [ ] **Step 7 : Lancer les tests du core**

```bash
cargo test --lib --all-features 2>&1 | tail -30
```

Attendu : tous les tests de `src/` passent.

- [ ] **Step 8 : Commit**

```bash
git add src/core/step.rs
git commit -m "feat(step): add filter_count to StepExecution and handle Ok(None) in process_chunk"
```

---

### Task 3 : Mettre à jour les exemples

**Files:**
- Modify: `examples/database_processing.rs`
- Modify: `examples/advanced_patterns.rs`
- Modify: `examples/json_processing.rs`
- Modify: `examples/xml_processing.rs`
- Modify: `examples/csv_processing.rs`
- Modify: `examples/mongodb_processing.rs`
- Modify: `examples/orm_processing.rs`
- Modify: `examples/benchmark_csv_postgres_xml.rs`

- [ ] **Step 1 : Mettre à jour tous les processors dans les exemples**

Pour chaque fichier d'exemple, remplacer `Ok(item)` ou `Ok(transformed_item)` par `Ok(Some(...))` dans toutes les méthodes `process()`. Faire la modification fichier par fichier.

**`examples/database_processing.rs` (ligne 87) :**
```bash
grep -n "fn process\|Ok(" examples/database_processing.rs | head -20
```
Remplacer `Ok(...)` dans `fn process` par `Ok(Some(...))`. La signature devient :
```rust
fn process(&self, item: &User) -> Result<Option<User>, BatchError> {
    Ok(Some(User { ... }))
}
```

**`examples/advanced_patterns.rs` (lignes 86, 124) :**
```rust
// Ligne ~93 :
fn process(&self, item: &RawTransaction) -> Result<Option<ValidTransaction>, BatchError> {
    Ok(Some(ValidTransaction { ... }))
}

// Ligne ~131 :
fn process(&self, item: &ValidTransaction) -> Result<Option<EnrichedTransaction>, BatchError> {
    Ok(Some(EnrichedTransaction { ... }))
}
```

**`examples/json_processing.rs` (lignes 57, 78) :**
```rust
fn process(&self, item: &Order) -> Result<Option<OrderSummary>, BatchError> {
    Ok(Some(OrderSummary { ... }))
}

fn process(&self, item: &Order) -> Result<Option<Order>, BatchError> {
    Ok(Some(item.clone()))
}
```

**`examples/xml_processing.rs` (ligne 80) :**
```rust
fn process(&self, item: &House) -> Result<Option<HouseCsv>, BatchError> {
    Ok(Some(HouseCsv { ... }))
}
```

**`examples/csv_processing.rs` (ligne 57) :**
```rust
fn process(&self, item: &Product) -> Result<Option<Product>, BatchError> {
    Ok(Some(item.clone()))
}
```

**`examples/mongodb_processing.rs` (lignes 91, 105) :**
```rust
fn process(&self, item: &Book) -> Result<Option<BookCsv>, BatchError> {
    Ok(Some(BookCsv { ... }))
}

fn process(&self, item: &BookInput) -> Result<Option<Book>, BatchError> {
    Ok(Some(Book { ... }))
}
```

**`examples/orm_processing.rs` (lignes 91, 105) :**
```rust
fn process(&self, item: &products::Model) -> Result<Option<ProductCsv>, BatchError> {
    Ok(Some(ProductCsv { ... }))
}

fn process(&self, item: &ProductDto) -> Result<Option<products::ActiveModel>, BatchError> {
    Ok(Some(products::ActiveModel { ... }))
}
```

**`examples/benchmark_csv_postgres_xml.rs` (ligne 83) :**
```rust
fn process(&self, item: &Transaction) -> Result<Option<Transaction>, BatchError> {
    Ok(Some(item.clone()))
}
```

- [ ] **Step 2 : Vérifier la compilation de tous les exemples**

```bash
cargo build --examples --features csv,json,xml,logger,fake 2>&1 | grep -E "^error" | head -20
```

Attendu : 0 erreurs pour les features non-database.

- [ ] **Step 3 : Commit**

```bash
git add examples/
git commit -m "fix(examples): update processors to return Ok(Some(...)) after ItemProcessorResult change"
```

---

### Task 4 : Créer l'exemple de filtrage

**Files:**
- Create: `examples/filter_records_from_csv_with_processor.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1 : Créer l'exemple**

Créer `examples/filter_records_from_csv_with_processor.rs` :

```rust
//! # Example: Filter Records from CSV with Processor
//!
//! Demonstrates how to filter items in a batch pipeline using a processor
//! that returns `Ok(None)` to silently discard items.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example filter_records_from_csv_with_processor --features csv,json
//! ```
//!
//! ## What It Does
//!
//! 1. Reads a list of persons (name, age) from an inline CSV string
//! 2. Filters out persons under 18 years old using a processor
//! 3. Writes the remaining adults to a JSON file in the temp directory
//! 4. Prints execution statistics including the filter count

use std::env::temp_dir;

use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult},
        job::{Job, JobBuilder},
        step::{Step, StepBuilder},
    },
    item::{
        csv::csv_reader::CsvItemReaderBuilder,
        json::json_writer::JsonItemWriterBuilder,
    },
    BatchError,
};

/// A person record read from the CSV source.
#[derive(Debug, Deserialize, Clone)]
struct Person {
    name: String,
    age: u32,
}

/// A processor that filters out persons under 18 years old.
///
/// Returns `Ok(None)` for minors, which causes the step to skip them
/// and increment `StepExecution::filter_count`.
#[derive(Default)]
struct AdultFilter;

impl ItemProcessor<Person, Person> for AdultFilter {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        if item.age >= 18 {
            Ok(Some(item.clone())) // keep adults
        } else {
            Ok(None) // filter out minors
        }
    }
}

const CSV_DATA: &str = "name,age\nAlice,30\nBob,16\nCharlie,25\nDiana,15\nEve,42\nFrank,17\n";

#[tokio::main]
async fn main() {
    // 1. Build reader from inline CSV string
    let reader = CsvItemReaderBuilder::<Person>::new()
        .has_headers(true)
        .from_reader(CSV_DATA.as_bytes());

    // 2. Build JSON writer to a temp file
    let output_path = temp_dir().join("adults.json");
    let writer = JsonItemWriterBuilder::<Person>::new()
        .from_path(output_path.clone());

    // 3. Build the filter processor
    let processor = AdultFilter::default();

    // 4. Build step with processor
    let step = StepBuilder::new("filter-adults")
        .chunk(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // 5. Build and run job
    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    // 6. Print execution statistics
    println!("Job status: {:?}", result.status);
    for step_execution in &result.step_executions {
        println!("Step: {}", step_execution.name);
        println!("  Read:     {}", step_execution.read_count);
        println!("  Filtered: {}", step_execution.filter_count);
        println!("  Written:  {}", step_execution.write_count);
    }
    println!("Output written to: {}", output_path.display());
}
```

- [ ] **Step 2 : Déclarer l'exemple dans `Cargo.toml`**

Ajouter dans la section `[[example]]` de `Cargo.toml` :

```toml
[[example]]
name = "filter_records_from_csv_with_processor"
required-features = ["csv", "json"]
```

- [ ] **Step 3 : Compiler l'exemple**

```bash
cargo build --example filter_records_from_csv_with_processor --features csv,json 2>&1
```

Attendu : compilation sans erreur.

- [ ] **Step 4 : Lancer l'exemple**

```bash
cargo run --example filter_records_from_csv_with_processor --features csv,json
```

Attendu :
```
Job status: Completed
Step: filter-adults
  Read:     6
  Filtered: 3
  Written:  3
Output written to: /tmp/adults.json
```

- [ ] **Step 5 : Commit**

```bash
git add examples/filter_records_from_csv_with_processor.rs Cargo.toml
git commit -m "feat(examples): add filter_records_from_csv_with_processor example"
```

---

### Task 5 : Mettre à jour la documentation du site

**Files:**
- Modify: `website/src/content/docs/api/item-processor.mdx`

- [ ] **Step 1 : Mettre à jour la définition du trait et le type alias**

Dans `website/src/content/docs/api/item-processor.mdx`, remplacer le bloc `## Trait Definition` (lignes 13-26) :

```mdx
## Trait Definition

```rust
pub trait ItemProcessor<I, O> {
    /// Processes an item and returns the transformed result
    ///
    /// # Returns
    /// - `Ok(Some(processed_item))` - Successfully processed, pass to writer
    /// - `Ok(None)` - Item filtered out, not passed to writer
    /// - `Err(BatchError)` - Processing failed
    fn process(&self, item: &I) -> Result<Option<O>, BatchError>;
}
```
```

- [ ] **Step 2 : Mettre à jour le type alias**

Remplacer le bloc `## Type Alias` :

```mdx
## Type Alias

```rust
pub type ItemProcessorResult<O> = Result<Option<O>, BatchError>;
```
```

- [ ] **Step 3 : Mettre à jour la card "Filtering"**

Remplacer la `<Card title="Filtering">` :

```mdx
<Card title="Filtering" icon="seti:todo">
  Return `Ok(None)` to silently discard items. Filtered items are counted in `StepExecution::filter_count`.
</Card>
```

- [ ] **Step 4 : Mettre à jour tous les exemples de code dans la page**

Parcourir toute la page et remplacer toutes les signatures de méthodes `process` et leurs retours :

- `fn process(&self, item: &X) -> ItemProcessorResult<Y>` ne change pas (le type alias est mis à jour)
- `Ok(item.clone())` → `Ok(Some(item.clone()))`
- `Ok(Product { ... })` → `Ok(Some(Product { ... }))`
- `Ok(EnrichedOrder { ... })` → `Ok(Some(EnrichedOrder { ... }))`
- `Ok(ProcessedTransaction { ... })` → `Ok(Some(ProcessedTransaction { ... }))`
- `Ok(cleaned)` → `Ok(Some(cleaned))`
- `Ok(intermediate)` dans `ProcessorChain` → `Ok(Some(intermediate))` (attention : la chaîne doit unwrap l'Option)
- `Ok(format!(...))` → `Ok(Some(format!(...)))`
- `Ok(response)` → `Ok(Some(response))`
- `Ok(TargetRecord { ... })` → `Ok(Some(TargetRecord { ... }))`
- `Ok(redacted)` → `Ok(Some(redacted))`

- [ ] **Step 5 : Ajouter un exemple de filtrage après la section "Data Validation"**

Après le bloc `### 2. Data Validation`, ajouter une nouvelle section :

```mdx
### 3. Item Filtering

Return `Ok(None)` to silently discard items — they are not passed to the writer and are counted in `StepExecution::filter_count`. This is different from returning `Err(BatchError)`, which counts as a processing error.

```rust
use spring_batch_rs::core::item::{ItemProcessor, ItemProcessorResult};

#[derive(Clone)]
struct Person {
    name: String,
    age: u32,
}

struct AdultFilter;

impl ItemProcessor<Person, Person> for AdultFilter {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        if item.age >= 18 {
            Ok(Some(item.clone())) // keep adults
        } else {
            Ok(None) // filter out minors — counted in filter_count
        }
    }
}
```

After job execution, check the filter count:

```rust
for step_execution in &result.step_executions {
    println!("Filtered: {}", step_execution.filter_count);
}
```

<Aside type="tip">
  Filtering with `Ok(None)` does **not** count toward `skip_limit`. Use it for intentional business filtering, not for error recovery.
</Aside>
```

- [ ] **Step 6 : Renuméroter les sections suivantes**

Les sections actuelles 3-6 deviennent 4-7 :
- `### 3. Data Enrichment` → `### 4. Data Enrichment`
- `### 4. Data Cleansing` → `### 5. Data Cleansing`
- `### 5. Conditional Processing` → `### 6. Conditional Processing`
- `### 6. String Transformations` → `### 7. String Transformations`

- [ ] **Step 7 : Vérifier que le site compile**

```bash
cd website && npm run build 2>&1 | tail -20
```

Attendu : build réussi sans erreur.

- [ ] **Step 8 : Commit**

```bash
git add website/src/content/docs/api/item-processor.mdx
git commit -m "docs(website): update ItemProcessor page for Ok(None) filtering and filter_count"
```

---

### Task 6 : Vérification finale

**Files:** aucun nouveau fichier

- [ ] **Step 1 : Lancer tous les tests**

```bash
cargo test --all-features 2>&1 | tail -30
```

Attendu : tous les tests passent, 0 failed.

- [ ] **Step 2 : Vérifier clippy**

```bash
cargo clippy --all-features -- -D warnings 2>&1 | grep -E "^error|^warning" | head -20
```

Attendu : 0 warnings, 0 errors.

- [ ] **Step 3 : Vérifier les doc-tests**

```bash
cargo test --doc --all-features 2>&1 | tail -20
```

Attendu : tous les doc-tests passent.

- [ ] **Step 4 : Lancer l'exemple de filtrage**

```bash
cargo run --example filter_records_from_csv_with_processor --features csv,json
```

Attendu :
```
Job status: Completed
Step: filter-adults
  Read:     6
  Filtered: 3
  Written:  3
```

- [ ] **Step 5 : Commit final**

```bash
git add -A
git commit -m "chore: final cleanup and verification of processor filtering feature"
```
