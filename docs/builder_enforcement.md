# Builder Enforcement for RDBC Readers and Writers

This document explains the architectural decision to enforce the use of unified builders for creating RDBC readers and writers.

## Overview

All RDBC item readers and writers can **only** be created through the unified builder API (`RdbcItemReaderBuilder` and `RdbcItemWriterBuilder`). Direct instantiation is not allowed.

## Why Enforce Builders?

### 1. **Consistent API**
By forcing all users to use the same builder API, we ensure:
- Uniform code patterns across the codebase
- Easier onboarding for new developers
- Consistent documentation and examples

### 2. **Validation at Construction**
Builders can enforce:
- Required parameters are provided
- Parameters are in valid ranges
- Proper initialization order

### 3. **Future Flexibility**
If we need to add:
- Additional validation logic
- New configuration options
- Deprecation warnings
- Migration helpers

We can do so in one place (the builder) without breaking existing code.

### 4. **Type Safety**
The builder pattern ensures:
- Compile-time type checking
- Correct database-specific types
- Proper lifetime management

## Implementation Details

### Private Fields
All struct fields are marked `pub(crate)`:

```rust
pub struct PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Postgres>,
    pub(crate) query: &'a str,
    pub(crate) page_size: Option<i32>,
    pub(crate) offset: Cell<i32>,
    pub(crate) buffer: RefCell<Vec<I>>,
}
```

This prevents external code from:
- Accessing fields directly
- Creating instances with struct literal syntax
- Modifying internal state

### Private Constructors
All `new()` methods and builder methods are marked `pub(crate)`:

```rust
impl<'a, I> PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) fn new(pool: Pool<Postgres>, query: &'a str, page_size: Option<i32>) -> Self {
        // ...
    }
}
```

This prevents external code from:
- Calling constructors directly
- Bypassing the builder API

### Public Builder API
Only the unified builders are public:

```rust
// Public API - This is the ONLY way to create readers/writers
pub use unified_reader_builder::RdbcItemReaderBuilder;
pub use unified_writer_builder::RdbcItemWriterBuilder;

// Types are public for trait implementations and usage
pub use postgres_reader::PostgresRdbcItemReader;
pub use mysql_reader::MySqlRdbcItemReader;
pub use sqlite_reader::SqliteRdbcItemReader;
```

## Usage Examples

### Creating a Reader (The Only Way)

```rust
use spring_batch_rs::item::rdbc::RdbcItemReaderBuilder;

// This is the ONLY way to create a reader
let reader = RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)
    .query("SELECT * FROM users")
    .with_page_size(100)
    .build_postgres();
```

### What You CANNOT Do

```rust
use spring_batch_rs::item::rdbc::PostgresRdbcItemReader;

// ❌ This will NOT compile - new() is not accessible
let reader = PostgresRdbcItemReader::new(pool, query, Some(100));

// ❌ This will NOT compile - fields are not accessible
let reader = PostgresRdbcItemReader {
    pool: pool,
    query: "SELECT * FROM users",
    page_size: Some(100),
    offset: Cell::new(0),
    buffer: RefCell::new(vec![]),
};

// ❌ This will NOT compile - old builder was removed
let reader = PostgresRdbcItemReaderBuilder::new()
    .pool(pool)
    .query("SELECT * FROM users")
    .build();
```

## Benefits for Library Users

### 1. **Clear Intent**
```rust
// Database type is explicit and obvious
RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)  // ← Clearly PostgreSQL
    .query("...")
    .build_postgres();
```

### 2. **Compiler Guidance**
If you try to use the wrong build method:
```rust
RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)
    .build_mysql();  // ← Compiler error! Type mismatch
```

### 3. **IDE Autocomplete**
Modern IDEs will show:
- `.postgres()` - for PostgreSQL
- `.mysql()` - for MySQL
- `.sqlite()` - for SQLite

All from the same builder type!

### 4. **Guaranteed Validity**
Builders ensure:
- Pool is provided
- Query is provided
- Types match the database

## Migration Path

Users migrating from old code need to:

1. **Replace direct constructors:**
   ```rust
   // Old (no longer works)
   PostgresRdbcItemReader::new(pool, query, Some(100))

   // New (required)
   RdbcItemReaderBuilder::new()
       .postgres(pool)
       .query(query)
       .with_page_size(100)
       .build_postgres()
   ```

2. **Replace old builders:**
   ```rust
   // Old (removed)
   PostgresRdbcItemReaderBuilder::new()
       .pool(pool)
       .query(query)
       .build()

   // New (required)
   RdbcItemReaderBuilder::new()
       .postgres(pool)
       .query(query)
       .build_postgres()
   ```

## For Library Maintainers

### Adding New Features
To add a new configuration option:

1. Add field to reader/writer struct (keep it `pub(crate)`)
2. Add builder method in `RdbcItemReaderBuilder`/`RdbcItemWriterBuilder`
3. Update all three `build_*()` methods to pass the new parameter

### Adding New Database Types
To add support for a new database:

1. Create new `{database}_reader.rs` with `pub(crate)` fields and methods
2. Add database variant to `DatabaseType` enum
3. Add builder method `.{database}()` to `RdbcItemReaderBuilder`
4. Add `build_{database}()` method
5. Export the reader type in `mod.rs`

## Design Rationale

This approach follows Rust's principle of **"making invalid states unrepresentable"**. By:

- Making fields private → Invalid partial construction is impossible
- Making constructors crate-only → Bypassing the builder is impossible
- Providing only the builder API → Consistency is enforced

We create a **pit of success** where the easiest path (using the builder) is also the correct path.

## Compile-Time Guarantees

With this design, the compiler prevents:

✅ Creating readers without required parameters
✅ Mixing database types incorrectly
✅ Bypassing validation logic
✅ Using deprecated APIs
✅ Inconsistent construction patterns

All while maintaining:

✅ Zero runtime overhead
✅ Full type safety
✅ Excellent error messages
✅ IDE support and autocomplete

## Conclusion

Enforcing the builder pattern through visibility modifiers provides:

- **Consistency** - One way to create readers/writers
- **Safety** - Compile-time validation
- **Maintainability** - Single point of change
- **Usability** - Clear, discoverable API

This is a **breaking change** but provides long-term benefits for code quality and maintainability.
