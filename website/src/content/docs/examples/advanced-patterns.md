---
title: Advanced Patterns
description: Advanced batch processing patterns and techniques with Spring Batch RS
sidebar:
  order: 8
---

Learn advanced patterns for building robust, production-ready batch applications with Spring Batch RS.

## Multi-Step ETL Pipelines

Chain multiple steps to create complex data transformation pipelines.

### Quick Start

```rust
use spring_batch_rs::core::job::JobBuilder;
use spring_batch_rs::core::step::StepBuilder;

let job = JobBuilder::new()
    .start(&extract_step)    // Step 1: Extract
    .next(&validate_step)    // Step 2: Validate
    .next(&transform_step)   // Step 3: Transform
    .next(&load_step)        // Step 4: Load
    .build();

let result = job.run()?;
```

## Complete Example

The [`advanced_patterns`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/advanced_patterns.rs) example demonstrates:

1. **Multi-step ETL**: Extract -> Validate -> Enrich -> Load
2. **Multi-format export**: Same data to JSON and CSV
3. **Aggregation pipeline**: Compute summaries from detail records
4. **Error handling**: Skip policies and execution monitoring

### Run the Example

```bash
cargo run --example advanced_patterns --features csv,json,logger
```

## Pattern 1: Validation Pipeline

Filter and validate records before processing:

```rust
struct ValidationProcessor;

impl ItemProcessor<RawTransaction, ValidTransaction> for ValidationProcessor {
    fn process(&self, item: &RawTransaction) -> ItemProcessorResult<ValidTransaction> {
        // Skip non-completed transactions
        if item.status != "completed" {
            return Err(BatchError::ItemProcessor(
                format!("Invalid status: {}", item.status)
            ));
        }

        // Validate amount
        if item.amount <= 0.0 {
            return Err(BatchError::ItemProcessor(
                format!("Invalid amount: {}", item.amount)
            ));
        }

        Ok(Some(ValidTransaction {
            id: item.id,
            account: item.account.clone(),
            amount: item.amount,
        }))
    }
}

let step = StepBuilder::new("validate")
    .chunk::<RawTransaction, ValidTransaction>(100)
    .reader(&reader)
    .processor(&ValidationProcessor)
    .writer(&writer)
    .skip_limit(10)  // Allow up to 10 validation failures
    .build();
```

## Pattern 2: Enrichment Pipeline

Add computed fields and business logic:

```rust
struct EnrichmentProcessor {
    fee_rate: f64,
}

impl ItemProcessor<Transaction, EnrichedTransaction> for EnrichmentProcessor {
    fn process(&self, item: &Transaction) -> ItemProcessorResult<EnrichedTransaction> {
        let fee = item.amount * self.fee_rate;

        let category = match item.amount {
            a if a >= 10000.0 => "large",
            a if a >= 1000.0 => "medium",
            _ => "small",
        };

        Ok(Some(EnrichedTransaction {
            transaction_id: format!("TXN-{:06}", item.id),
            gross_amount: item.amount,
            fee,
            net_amount: item.amount - fee,
            category: category.to_string(),
        }))
    }
}
```

## Pattern 3: Conditional Processing

Process records differently based on conditions:

```rust
struct ConditionalProcessor;

impl ItemProcessor<Order, ProcessedOrder> for ConditionalProcessor {
    fn process(&self, item: &Order) -> ItemProcessorResult<ProcessedOrder> {
        let (discount, priority) = match item.customer_type.as_str() {
            "premium" => (0.15, "high"),
            "regular" => (0.05, "normal"),
            _ => (0.0, "low"),
        };

        Ok(Some(ProcessedOrder {
            order_id: item.id,
            final_amount: item.amount * (1.0 - discount),
            priority: priority.to_string(),
        }))
    }
}
```

## Pattern 4: Multi-Format Export

Export the same data to multiple formats:

```rust
// Create readers for each output (data must be cloned or re-read)
let json_reader = InMemoryReader::new(data.clone());
let csv_reader = InMemoryReader::new(data);

// JSON export step
let json_step = StepBuilder::new("export-json")
    .chunk::<Record, Record>(100)
    .reader(&json_reader)
    .processor(&PassThroughProcessor::new())
    .writer(&json_writer)
    .build();

// CSV export step
let csv_step = StepBuilder::new("export-csv")
    .chunk::<Record, Record>(100)
    .reader(&csv_reader)
    .processor(&PassThroughProcessor::new())
    .writer(&csv_writer)
    .build();

// Run both exports
let job = JobBuilder::new()
    .start(&json_step)
    .next(&csv_step)
    .build();
```

## Pattern 5: Error Handling and Monitoring

Monitor execution and handle errors gracefully:

```rust
let step = StepBuilder::new("monitored-step")
    .chunk::<Input, Output>(50)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .skip_limit(100)  // Skip up to 100 errors
    .build();

let job = JobBuilder::new().start(&step).build();
let result = job.run()?;

// Access execution metrics
let step_exec = job.get_step_execution("monitored-step").unwrap();

println!("Execution Summary:");
println!("  Status: {:?}", step_exec.status);
println!("  Read count: {}", step_exec.read_count);
println!("  Write count: {}", step_exec.write_count);
println!("  Read errors: {}", step_exec.read_error_count);
println!("  Process errors: {}", step_exec.process_error_count);
println!("  Duration: {:?}", result.duration);
```

## Pattern 6: Intermediate Files

Use temporary files between steps:

```rust
let intermediate_path = temp_dir().join("intermediate.json");

// Step 1: CSV to JSON
let step1 = StepBuilder::new("csv-to-json")
    .chunk::<CsvRecord, JsonRecord>(100)
    .reader(&csv_reader)
    .processor(&csv_to_json_processor)
    .writer(&JsonItemWriterBuilder::new().from_path(&intermediate_path))
    .build();

// Step 2: JSON to final output
let json_file = File::open(&intermediate_path)?;
let step2 = StepBuilder::new("json-to-output")
    .chunk::<JsonRecord, FinalRecord>(100)
    .reader(&JsonItemReaderBuilder::new().from_reader(json_file))
    .processor(&final_processor)
    .writer(&final_writer)
    .build();

let job = JobBuilder::new()
    .start(&step1)
    .next(&step2)
    .build();
```

## Pattern 7: Aggregation

Collect and summarize records:

```rust
use std::collections::HashMap;

// Read all transactions
let transactions: Vec<Transaction> = read_all_transactions()?;

// Aggregate by account
let mut accounts: HashMap<String, AccountSummary> = HashMap::new();

for txn in &transactions {
    let entry = accounts.entry(txn.account.clone())
        .or_insert(AccountSummary::default());
    
    if txn.transaction_type == "credit" {
        entry.total_credits += txn.amount;
    } else {
        entry.total_debits += txn.amount;
    }
    entry.count += 1;
}

// Write summaries
let summaries: Vec<AccountSummary> = accounts.into_values().collect();
let reader = InMemoryReader::new(summaries);

let processor = PassThroughProcessor::<AccountSummary>::new();

let step = StepBuilder::new("write-summaries")
    .chunk::<AccountSummary, AccountSummary>(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&summary_writer)
    .build();
```

## In-Memory Reader Helper

For testing and intermediate data:

```rust
use std::cell::RefCell;
use std::collections::VecDeque;

struct InMemoryReader<T> {
    items: RefCell<VecDeque<T>>,
}

impl<T: Clone> InMemoryReader<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            items: RefCell::new(items.into()),
        }
    }
}

impl<T: Clone> ItemReader<T> for InMemoryReader<T> {
    fn read(&self) -> ItemReaderResult<T> {
        Ok(self.items.borrow_mut().pop_front())
    }
}
```

## Best Practices

### 1. Choose Appropriate Chunk Sizes

```rust
// Small chunks for complex processing
.chunk::<Input, Output>(10)

// Larger chunks for simple pass-through
.chunk::<Input, Output>(1000)

// Consider memory usage for large items
.chunk::<LargeItem, LargeItem>(50)
```

### 2. Use Skip Limits Wisely

```rust
// Strict: fail on any error
.skip_limit(0)

// Tolerant: allow some bad records
.skip_limit(100)

// Very tolerant: for dirty data
.skip_limit(10000)
```

### 3. Monitor Execution

Always check step execution results:

```rust
let step_exec = job.get_step_execution("my-step").unwrap();
if step_exec.read_error_count > 0 {
    log::warn!("Skipped {} bad records", step_exec.read_error_count);
}
```

### 4. Clean Up Intermediate Files

```rust
use std::fs;

// After job completion
if intermediate_path.exists() {
    fs::remove_file(&intermediate_path)?;
}
```

## See Also

- [CSV Processing](/spring-batch-rs/examples/csv/) - File format basics
- [JSON Processing](/spring-batch-rs/examples/json/) - JSON operations
- [Database Processing](/spring-batch-rs/examples/database/) - Database integration
- [Tasklet Examples](/spring-batch-rs/examples/tasklets/) - Single-task operations
