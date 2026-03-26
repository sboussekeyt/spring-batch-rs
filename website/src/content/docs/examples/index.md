---
title: Examples Gallery
description: Complete examples for all Spring Batch RS features
sidebar:
  order: 0
---

import { Card, CardGrid } from '@astrojs/starlight/components';

Browse examples organized by feature. Each page includes documentation and links to runnable code.

<CardGrid>
  <Card title="CSV Processing" icon="document">
    Read and write CSV files, transform data, handle headers and delimiters.

    [View CSV Examples →](/examples/csv/)
  </Card>

  <Card title="JSON Processing" icon="document">
    Stream JSON arrays, write with formatting, transform between formats.

    [View JSON Examples →](/examples/json/)
  </Card>

  <Card title="XML Processing" icon="document">
    Parse XML documents, handle attributes, nested elements.

    [View XML Examples →](/examples/xml/)
  </Card>

  <Card title="Database (RDBC)" icon="seti:db">
    PostgreSQL, MySQL, SQLite operations with batch inserts.

    [View Database Examples →](/examples/database/)
  </Card>

  <Card title="MongoDB" icon="seti:db">
    Query collections, batch inserts, cursor-based pagination.

    [View MongoDB Examples →](/examples/mongodb/)
  </Card>

  <Card title="ORM (SeaORM)" icon="seti:db">
    Type-safe ORM queries, entity mapping, pagination.

    [View ORM Examples →](/examples/orm/)
  </Card>

  <Card title="Tasklets" icon="rocket">
    ZIP compression, FTP transfers, single-task operations.

    [View Tasklet Examples →](/examples/tasklets/)
  </Card>

  <Card title="Advanced Patterns" icon="puzzle">
    Multi-step ETL pipelines, error handling, complex workflows.

    [View Advanced Patterns →](/examples/advanced-patterns/)
  </Card>
</CardGrid>

## Quick Links to Source Code

| Example | Features | Source |
|---------|----------|--------|
| CSV Processing | `csv`, `json` | [csv_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/csv_processing.rs) |
| JSON Processing | `json`, `csv`, `logger` | [json_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/json_processing.rs) |
| XML Processing | `xml`, `json`, `csv` | [xml_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/xml_processing.rs) |
| Database (SQLite) | `rdbc-sqlite`, `csv`, `json`, `logger` | [database_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/database_processing.rs) |
| MongoDB | `mongodb`, `csv`, `json` | [mongodb_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/mongodb_processing.rs) |
| ORM (SeaORM) | `orm`, `csv`, `json` | [orm_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/orm_processing.rs) |
| ZIP Tasklet | `zip` | [tasklet_zip.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_zip.rs) |
| FTP Tasklet | `ftp` | [tasklet_ftp.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_ftp.rs) |
| Advanced Patterns | `csv`, `json`, `logger` | [advanced_patterns.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/advanced_patterns.rs) |

## Running Examples

All examples can be run directly with cargo:

```bash
# List all available examples
cargo run --example

# Run a specific example with required features
cargo run --example csv_processing --features csv,json
cargo run --example json_processing --features json,csv,logger
cargo run --example xml_processing --features xml,json,csv
cargo run --example database_processing --features rdbc-sqlite,csv,json,logger
cargo run --example orm_processing --features orm,csv,json
cargo run --example tasklet_zip --features zip
cargo run --example advanced_patterns --features csv,json,logger
```

See the [Cargo.toml](https://github.com/sboussekeyt/spring-batch-rs/blob/main/Cargo.toml) file for all available examples and their required features.
