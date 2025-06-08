# API Reference

The complete API documentation for Spring Batch RS is hosted on docs.rs, providing detailed information about all public APIs, traits, and modules.

## 📖 Complete API Documentation

**[View Full API Documentation on docs.rs →](https://docs.rs/spring-batch-rs)**

The docs.rs documentation includes:

- **Complete API reference** for all public types and functions
- **Detailed examples** for each module and function
- **Source code links** for implementation details
- **Feature flag documentation** showing what's available with each feature
- **Cross-references** between related types and traits

## 🗂️ Module Overview

Here's a quick overview of the main modules and their purposes:

### Core Modules

| Module                                                                            | Description                     | Key Types                                   |
| --------------------------------------------------------------------------------- | ------------------------------- | ------------------------------------------- |
| [`core::job`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/core/job/)   | Job orchestration and execution | `Job`, `JobBuilder`, `JobInstance`          |
| [`core::step`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/core/step/) | Step execution and management   | `Step`, `StepBuilder`, `StepExecution`      |
| [`core::item`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/core/item/) | Core processing interfaces      | `ItemReader`, `ItemProcessor`, `ItemWriter` |

### Item Processing

| Module                                                                            | Description          | Key Types                          |
| --------------------------------------------------------------------------------- | -------------------- | ---------------------------------- |
| [`item::csv`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/csv/)   | CSV file processing  | `CsvItemReader`, `CsvItemWriter`   |
| [`item::json`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/json/) | JSON file processing | `JsonItemReader`, `JsonItemWriter` |
| [`item::xml`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/xml/)   | XML file processing  | `XmlItemReader`, `XmlItemWriter`   |

### Database Integration

| Module                                                                                  | Description          | Key Types                            |
| --------------------------------------------------------------------------------------- | -------------------- | ------------------------------------ |
| [`item::orm`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/orm/)         | SeaORM integration   | `OrmItemReader`, `OrmItemWriter`     |
| [`item::rdbc`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/rdbc/)       | RDBC database access | `RdbcItemReader`, `RdbcItemWriter`   |
| [`item::mongodb`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/mongodb/) | MongoDB integration  | `MongoItemReader`, `MongoItemWriter` |

### Tasklets

| Module                                                                                | Description                | Key Types                         |
| ------------------------------------------------------------------------------------- | -------------------------- | --------------------------------- |
| [`tasklet::zip`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/tasklet/zip/) | ZIP compression operations | `ZipTasklet`, `ZipTaskletBuilder` |
| [`tasklet::ftp`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/tasklet/ftp/) | FTP file operations        | `FtpPutTasklet`, `FtpGetTasklet`  |

### Utilities

| Module                                                                                | Description          | Key Types      |
| ------------------------------------------------------------------------------------- | -------------------- | -------------- |
| [`item::logger`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/logger/) | Debug logging writer | `LoggerWriter` |
| [`item::fake`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/item/fake/)     | Mock data generation | `PersonReader` |

## 🔍 Quick API Lookup

### Common Patterns

**Creating a Job:**

```rust
use spring_batch_rs::core::job::JobBuilder;

let job = JobBuilder::new()
    .start(&step)
    .build();
```

**Building a Step:**

```rust
use spring_batch_rs::core::step::StepBuilder;

let step = StepBuilder::new("my-step")
    .chunk(100)
    .reader(&reader)
    .processor(&processor)
    .writer(&writer)
    .build();
```

**Implementing ItemProcessor:**

```rust
use spring_batch_rs::core::item::ItemProcessor;
use spring_batch_rs::BatchError;

impl ItemProcessor<InputType, OutputType> for MyProcessor {
    fn process(&self, item: InputType) -> Result<Option<OutputType>, BatchError> {
        // Your processing logic here
        Ok(Some(transformed_item))
    }
}
```

## 📋 Error Types

All Spring Batch RS operations use the [`BatchError`](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/enum.BatchError.html) enum for error handling:

```rust
pub enum BatchError {
    ItemReader(String),
    ItemProcessor(String),
    ItemWriter(String),
    Step(String),
    Job(String),
    Io(std::io::Error),
    Configuration(String),
    // ... other variants
}
```

## 🏷️ Feature Flags

Spring Batch RS uses feature flags to keep dependencies minimal. See the [feature documentation](https://docs.rs/spring-batch-rs/latest/spring_batch_rs/#features) for details on what each feature enables.

## 📚 Documentation Tips

When browsing the API documentation:

1. **Use the search box** to quickly find specific types or functions
2. **Check the examples** in each module for usage patterns
3. **Look at the source code** links for implementation details
4. **Follow the cross-references** to understand relationships between types
5. **Check feature requirements** for each module

## 🔗 Related Resources

- **[Getting Started Guide](./getting-started)** - Learn the basics
- **[Examples](./examples)** - Ready-to-run code examples
- **[Tutorials](./tutorials)** - Step-by-step guides
- **[Architecture](./architecture)** - Framework design and concepts

## 📝 Contributing to Documentation

Found an issue with the API documentation or have suggestions for improvement?

- **API docs issues**: [Report on docs.rs](https://github.com/rust-lang/docs.rs/issues)
- **Code documentation**: [Open an issue](https://github.com/sboussekeyt/spring-batch-rs/issues) or submit a PR
- **Website documentation**: [Contribute to our docs](https://github.com/sboussekeyt/spring-batch-rs/tree/main/website)

---

**[📖 Browse the complete API documentation on docs.rs →](https://docs.rs/spring-batch-rs)**
