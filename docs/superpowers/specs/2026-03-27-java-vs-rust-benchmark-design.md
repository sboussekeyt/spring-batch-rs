# Design Spec — Spring Batch Java vs Rust Benchmark

**Date:** 2026-03-27
**Status:** Approved
**Author:** Brainstorming session

---

## Overview

Add a production-grade benchmark comparing Spring Batch (Java) and Spring Batch RS (Rust) on a realistic ETL pipeline processing 10 million financial transaction records. The goal is to provide credible, reproducible evidence of Rust's performance advantages (no GC pauses, lower memory, faster startup) while giving visitors runnable code for both implementations.

---

## Pipeline

Two-step batch job:

```
Step 1:  transactions.csv (10M rows)
            ↓ CsvItemReader
         TransactionProcessor (currency conversion + status normalization)
            ↓ PostgresItemWriter (bulk insert, chunk=1000)
         table `transactions` (PostgreSQL)

Step 2:  table `transactions` (PostgreSQL)
            ↓ RdbcItemReader (paginated, page_size=1000)
         PassThroughProcessor
            ↓ XmlItemWriter
         transactions_export.xml
```

---

## Data Model

### Transaction record

| Field          | Type   | Example                              |
|----------------|--------|--------------------------------------|
| transaction_id | String | UUID v4                              |
| amount         | f64    | 1234.56                              |
| currency       | String | "USD", "EUR", "GBP"                  |
| timestamp      | String | ISO 8601 ("2024-01-15T10:30:00Z")    |
| account_from   | String | "ACC-00001"                          |
| account_to     | String | "ACC-99999"                          |
| status         | String | "PENDING", "COMPLETED", "FAILED", "CANCELLED" |

### Transformation (Step 1 processor)
- Currency conversion to EUR: USD × 0.92, GBP × 1.17, EUR × 1.0
- Status normalization: "CANCELLED" → "FAILED"
- Added field: `amount_eur` (converted amount)

### Data generator
- Integrated into the Rust example using the `fake` feature
- Generates `transactions.csv` before running the job
- Java equivalent: `DataGenerator.java` using `java.util.Random`
- Both generators produce the same distribution of currencies and statuses

---

## Rust Implementation

### File: `examples/benchmark_csv_postgres_xml.rs`

Three logical sections:
1. **DataGenerator** — writes `transactions.csv` (10M rows) using `fake` crate
2. **Step 1** — CSV → PostgreSQL with optimized settings
3. **Step 2** — PostgreSQL → XML with optimized settings

### Performance settings
- `BufReader` with 64KB buffer on CSV input
- `PgPoolOptions::max_connections(10)`
- Bulk insert via `push_values` in the writer binder
- Chunk size: 1000 for both steps
- Page size: 1000 for RDBC reader
- Compiled with `--release` (`opt-level = 3`)

### Required features
```toml
[[example]]
name = "benchmark_csv_postgres_xml"
required-features = ["csv", "xml", "rdbc-postgres", "fake"]
```

### Run command
```bash
cargo run --release --example benchmark_csv_postgres_xml \
  --features csv,xml,rdbc-postgres,fake \
  -- --db-url postgresql://postgres:postgres@localhost:5432/benchmark
```

---

## Java Implementation

### Directory: `benchmark/java/`

### Stack
- Java 21
- Spring Batch 5.x
- Spring Boot 3.x
- HikariCP connection pool
- JAXB for XML output
- Maven build

### Key files
```
benchmark/java/
  pom.xml
  src/main/java/com/example/benchmark/
    BenchmarkApplication.java        ← main + job runner
    Transaction.java                 ← entity (JPA + JAXB annotated)
    TransactionProcessor.java        ← currency conversion + status normalization
    DataGenerator.java               ← CSV generator (10M rows)
    config/
      BatchConfig.java               ← Step 1: CSV → PostgreSQL
      XmlExportConfig.java           ← Step 2: PostgreSQL → XML
  src/main/resources/
    application.properties           ← datasource, HikariCP, batch settings
    schema.sql                       ← CREATE TABLE transactions
```

### Performance settings (Java)
- HikariCP: `maximumPoolSize=10`
- Chunk size: 1000 for both steps
- JVM: `-Xms512m -Xmx4g -XX:+UseG1GC -Xlog:gc*:gc.log`
- FlatFileItemReader with BufferedReader (default Spring Batch)
- JdbcBatchItemWriter for bulk inserts

### Run command
```bash
cd benchmark/java
mvn spring-boot:run -Dspring-boot.run.jvmArguments="\
  -Xms512m -Xmx4g -XX:+UseG1GC -Xlog:gc*:gc.log" \
  -Dspring-boot.run.arguments="\
  --spring.datasource.url=jdbc:postgresql://localhost:5432/benchmark"
```

---

## Metrics

| Category    | Metric                          | Collection method              |
|-------------|---------------------------------|--------------------------------|
| Time        | Wall time total                 | `time` command / `Instant`     |
| Time        | Step 1 duration                 | `StepExecution` / `JobExecution` |
| Time        | Step 2 duration                 | `StepExecution` / `JobExecution` |
| Startup     | Time to first record processed  | internal instrumentation       |
| Throughput  | records/sec Step 1              | `write_count / duration`       |
| Throughput  | records/sec Step 2              | `read_count / duration`        |
| Memory      | Peak RSS                        | `/usr/bin/time -v`             |
| Memory      | Heap peak                       | JVM: `-Xlog:gc*` / `sysinfo`   |
| CPU         | Average CPU %                   | `top` snapshot                 |
| GC (Java)   | Total GC pause time             | parsed from `gc.log`           |
| GC (Java)   | Number of GC events             | parsed from `gc.log`           |
| Artifact    | Deployable size                 | `ls -lh target/` / JAR size   |

---

## Website Page

### File: `website/src/content/docs/reference/java-vs-rust-benchmark.mdx`

### Structure
1. **Introduction** — why this benchmark, what it tests
2. **Test environment** — hardware, OS, Java version, Rust version, PostgreSQL version
3. **Pipeline description** — schema + data model
4. **Code side by side** — Rust vs Java for key parts (processor, step config)
5. **Results** — metrics table with measured values
6. **Analysis** — GC pauses deep dive, memory profile, startup time
7. **Conclusions & recommendations** — when to choose each
8. **How to reproduce** — step-by-step commands

### Components used
- `<Tabs>` / `<TabItem>` for Rust vs Java code blocks
- `<Aside type="note">` for environment disclaimer
- `<Aside type="tip">` for reproduction instructions
- Tables for metrics results

---

## Files Created / Modified

| Action | File |
|--------|------|
| CREATE | `examples/benchmark_csv_postgres_xml.rs` |
| MODIFY | `Cargo.toml` — add `[[example]]` entry |
| CREATE | `benchmark/java/pom.xml` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/BenchmarkApplication.java` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/Transaction.java` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/TransactionProcessor.java` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/DataGenerator.java` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/config/BatchConfig.java` |
| CREATE | `benchmark/java/src/main/java/com/example/benchmark/config/XmlExportConfig.java` |
| CREATE | `benchmark/java/src/main/resources/application.properties` |
| CREATE | `benchmark/java/src/main/resources/schema.sql` |
| CREATE | `website/src/content/docs/reference/java-vs-rust-benchmark.mdx` |

---

## Constraints

- Java code must compile with `mvn package` (no optional dependencies)
- Rust example must compile with `cargo build --release --features csv,xml,rdbc-postgres,fake`
- Both implementations use identical chunk size (1000), pool size (10), same data schema
- Benchmark results in the doc page are measured on a reference machine and clearly labeled as such
- An `Aside` note warns that results vary by hardware/environment
- No Docker orchestration required — user starts PostgreSQL independently
