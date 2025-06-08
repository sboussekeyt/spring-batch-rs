# Your First Batch Job

In this tutorial, you'll create your first Spring Batch RS application that reads CSV data and converts it to JSON format. This tutorial covers the fundamental concepts of jobs, steps, readers, writers, and processors.

## What You'll Learn

- How to set up a Spring Batch RS project
- Core concepts: Jobs, Steps, ItemReaders, ItemWriters, and ItemProcessors
- How to process CSV data and output JSON
- Basic error handling and debugging

## Prerequisites

- Rust 1.70+ installed
- Basic Rust programming knowledge
- Familiarity with cargo and Rust project structure

## Project Setup

### 1. Create a New Project

```bash
cargo new csv-to-json-batch
cd csv-to-json-batch
```

### 2. Add Dependencies

Edit your `Cargo.toml`:

```toml
[package]
name = "csv-to-json-batch"
version = "0.1.0"
edition = "2021"

[dependencies]
spring-batch-rs = { version = "0.3", features = ["csv", "json"] }
serde = { version = "1.0", features = ["derive"] }
```

### 3. Create Sample Data

Create a file called `products.csv` in your project root:

```csv
id,name,price,category,in_stock
1,Laptop Computer,999.99,Electronics,true
2,Coffee Mug,12.99,Kitchen,true
3,Wireless Mouse,29.99,Electronics,false
4,Notebook Set,15.99,Office,true
5,Desk Lamp,45.00,Office,true
```

## Implementation

### 1. Define Your Data Structure

Create `src/main.rs` and define the data structure:

```rust
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
    in_stock: bool,
}

fn main() -> Result<(), BatchError> {
    // Implementation will go here
    Ok(())
}
```

### 2. Create the CSV Reader

Add the CSV reader configuration:

```rust
fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // More code will go here...
    Ok(())
}
```

**Key points about the CSV reader:**

- `CsvItemReaderBuilder::<Product>` specifies the target type
- `has_headers(true)` tells the reader to skip the first line
- `from_path()` reads from a file (you can also use `from_reader()` for in-memory data)

### 3. Create the JSON Writer

Add the JSON writer:

```rust
fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // More code will go here...
    Ok(())
}
```

**Key points about the JSON writer:**

- `pretty_formatter(true)` creates nicely formatted JSON
- `from_path()` writes to a file
- The writer automatically handles JSON array formatting

### 4. Create a Processor

For this tutorial, we'll use a pass-through processor that doesn't modify the data:

```rust
fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // Create processor (pass-through)
    let processor = PassThroughProcessor::<Product>::new();

    // More code will go here...
    Ok(())
}
```

### 5. Build the Step

Now create a step that combines the reader, processor, and writer:

```rust
fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // Create processor (pass-through)
    let processor = PassThroughProcessor::<Product>::new();

    // Build the step
    let step = StepBuilder::new("csv-to-json-step")
        .chunk(10)  // Process 10 items at a time
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // More code will go here...
    Ok(())
}
```

**Key points about the step:**

- `chunk(10)` means we process items in batches of 10
- The step name "csv-to-json-step" is used for logging and debugging
- All three components (reader, processor, writer) are required

### 6. Create and Run the Job

Finally, create the job and execute it:

```rust
fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // Create processor (pass-through)
    let processor = PassThroughProcessor::<Product>::new();

    // Build the step
    let step = StepBuilder::new("csv-to-json-step")
        .chunk(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // Build and run the job
    let job = JobBuilder::new()
        .start(&step)
        .build();

    // Execute the job
    let result = job.run()?;

    println!("Job completed successfully!");
    println!("Steps executed: {}", result.get_step_executions().len());

    // Print step details
    for step_execution in result.get_step_executions() {
        println!("Step '{}' processed {} items",
                 step_execution.get_step_name(),
                 step_execution.get_read_count());
    }

    Ok(())
}
```

## Running Your Job

### 1. Execute the Program

```bash
cargo run
```

You should see output similar to:

```
Job completed successfully!
Steps executed: 1
Step 'csv-to-json-step' processed 5 items
```

### 2. Check the Output

Look at the generated `products.json` file:

```json
[
  {
    "id": 1,
    "name": "Laptop Computer",
    "price": 999.99,
    "category": "Electronics",
    "in_stock": true
  },
  {
    "id": 2,
    "name": "Coffee Mug",
    "price": 12.99,
    "category": "Kitchen",
    "in_stock": true
  },
  {
    "id": 3,
    "name": "Wireless Mouse",
    "price": 29.99,
    "category": "Electronics",
    "in_stock": false
  },
  {
    "id": 4,
    "name": "Notebook Set",
    "price": 15.99,
    "category": "Office",
    "in_stock": true
  },
  {
    "id": 5,
    "name": "Desk Lamp",
    "price": 45.0,
    "category": "Office",
    "in_stock": true
  }
]
```

## Understanding What Happened

Let's break down the execution flow:

1. **Job Started**: The JobBuilder created a job with one step
2. **Step Execution**: The step began processing with chunk size 10
3. **Reading Phase**: The CSV reader read each line and deserialized it to a `Product`
4. **Processing Phase**: The PassThroughProcessor passed each item unchanged
5. **Writing Phase**: The JSON writer collected items and wrote them as a JSON array
6. **Completion**: The job finished successfully

## Adding Custom Processing

Let's enhance the example by adding a custom processor that applies a discount to electronics:

```rust
use spring_batch_rs::core::item::ItemProcessor;

struct DiscountProcessor;

impl ItemProcessor<Product, Product> for DiscountProcessor {
    fn process(&self, item: Product) -> Result<Option<Product>, BatchError> {
        let mut product = item;

        // Apply 10% discount to electronics
        if product.category == "Electronics" {
            product.price *= 0.9;
            println!("Applied discount to {}: ${:.2}", product.name, product.price);
        }

        Ok(Some(product))
    }
}

fn main() -> Result<(), BatchError> {
    // ... reader and writer setup ...

    // Use custom processor instead of PassThroughProcessor
    let processor = DiscountProcessor;

    // ... rest of the code ...
}
```

## Error Handling

Add error handling with skip limits:

```rust
let step = StepBuilder::new("csv-to-json-step")
    .chunk(10)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .skip_limit(2)  // Skip up to 2 errors before failing
    .build();
```

## Best Practices

1. **Choose appropriate chunk sizes**: Smaller chunks use less memory but have more overhead
2. **Handle errors gracefully**: Use skip limits for fault tolerance
3. **Use meaningful step names**: They appear in logs and help with debugging
4. **Validate your data structures**: Ensure your Serde derives match your data format
5. **Test with small datasets first**: Verify your logic before processing large files

## Next Steps

Now that you've created your first batch job, explore these topics:

- **Working with Different Data Formats** - XML, databases, and more _(Coming soon)_
- **Error Handling and Fault Tolerance** - Robust error handling patterns _(Coming soon)_
- **Custom Processors** - Implement complex business logic _(Coming soon)_
- **Multi-Step Jobs** - Chain multiple processing steps _(Coming soon)_

## Troubleshooting

**Common issues and solutions:**

- **File not found**: Ensure `products.csv` is in your project root
- **Deserialization errors**: Check that your CSV headers match your struct fields
- **Permission errors**: Ensure you have write permissions for the output file
- **Type mismatches**: Verify your data types match the CSV content

## Complete Code

Here's the complete `src/main.rs` file:

```rust
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{job::JobBuilder, step::StepBuilder, item::PassThroughProcessor},
    item::{csv::CsvItemReaderBuilder, json::JsonItemWriterBuilder},
    BatchError,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
    in_stock: bool,
}

fn main() -> Result<(), BatchError> {
    // Create CSV reader
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv")?;

    // Create JSON writer
    let writer = JsonItemWriterBuilder::new()
        .pretty_formatter(true)
        .from_path("products.json")?;

    // Create processor (pass-through)
    let processor = PassThroughProcessor::<Product>::new();

    // Build the step
    let step = StepBuilder::new("csv-to-json-step")
        .chunk(10)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    // Build and run the job
    let job = JobBuilder::new()
        .start(&step)
        .build();

    // Execute the job
    let result = job.run()?;

    println!("Job completed successfully!");
    println!("Steps executed: {}", result.get_step_executions().len());

    for step_execution in result.get_step_executions() {
        println!("Step '{}' processed {} items",
                 step_execution.get_step_name(),
                 step_execution.get_read_count());
    }

    Ok(())
}
```

Congratulations! You've successfully created your first Spring Batch RS application. 🎉
