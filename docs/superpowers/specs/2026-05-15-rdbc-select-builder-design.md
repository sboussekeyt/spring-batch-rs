# Design: RDBC SelectBuilder

**Date:** 2026-05-15  
**Status:** Approved  
**Scope:** `sbrs-lib` — `src/item/rdbc/`

---

## Problem

The RDBC reader currently requires a raw SQL string via `.query("SELECT ...")`. This means:

- Users write SQL inline in Rust builder chains
- Pagination clauses (LIMIT/OFFSET, keyset WHERE/ORDER BY) are appended by string concatenation inside the reader
- Keyset pagination requires two separate calls: `.query(...)` + `.with_keyset(col, key_fn)`, which are conceptually one concern

## Goal

Provide an optional fluent `SelectBuilder` that generates the base SQL string for simple queries. The raw `.query()` path remains fully supported for complex cases (JOINs, subqueries, CTEs).

---

## Design

### 1. `SelectBuilder<I>`

New file: `src/item/rdbc/select_builder.rs`

A **database-agnostic** struct that generates a SQL `SELECT` string. Generic over `I` (the item type) only to carry the keyset `key_fn` closure when `order_by_keyset` is used.

```rust
pub struct SelectBuilder<I> {
    table: String,
    columns: Vec<String>,          // empty → SELECT *
    conditions: Vec<WhereClause>,  // joined with AND
    order_by: Vec<OrderClause>,
    keyset_column: Option<String>,
    keyset_key_fn: Option<Box<dyn Fn(&I) -> String>>,
}
```

#### Constructor

```rust
SelectBuilder::from("users")   // sets table
```

#### Column selection

```rust
.columns(&["id", "name", "email"])  // omit → SELECT *
```

#### WHERE conditions

All values go through `impl Into<ConditionValue>`. Blanket impls cover `i32`, `i64`, `f64`, `bool`, `&str`, `String`.

| Method | SQL fragment |
|---|---|
| `.where_eq("col", val)` | `col = 'val'` / `col = 42` |
| `.where_not_eq("col", val)` | `col != 'val'` |
| `.where_gt("col", val)` | `col > val` |
| `.where_gte("col", val)` | `col >= val` |
| `.where_lt("col", val)` | `col < val` |
| `.where_lte("col", val)` | `col <= val` |
| `.where_like("col", pat)` | `col LIKE 'pat'` |
| `.where_is_null("col")` | `col IS NULL` |
| `.where_is_not_null("col")` | `col IS NOT NULL` |

Multiple conditions are combined with `AND`.

String values are escaped via `replace('\'', "''")` before inlining. Integer, float, and bool values are inlined directly (no injection risk).

#### Ordering / keyset

```rust
// Simple ordering (no keyset)
.order_by_asc("created_at")
.order_by_desc("score")

// Keyset ordering — sets ORDER BY + keyset column + key extractor
.order_by_keyset("id", |u: &User| u.id.to_string())
```

`order_by_keyset` replaces the need to separately call `.with_keyset()` on the reader builder. Only one keyset column is supported. Calling `order_by_keyset` after another `order_by_*` replaces all previous order clauses.

#### SQL generation

```rust
pub fn build_sql(&self) -> String
```

Produces: `SELECT col1, col2 FROM table [WHERE ...] [ORDER BY ...]`

Does **not** append LIMIT/OFFSET or keyset WHERE — those remain the reader's responsibility.

#### `ConditionValue` enum (internal)

```rust
enum ConditionValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}
```

---

### 2. `RdbcItemReaderBuilder` changes

File: `src/item/rdbc/unified_reader_builder.rs`

#### New `QuerySource` enum (internal)

```rust
enum QuerySource<'a> {
    Raw(&'a str),
    Built(String),
}
```

Replaces the current `query: Option<&'a str>` field. Both paths produce the same SQL string at build time.

#### New `.select()` method

```rust
pub fn select(mut self, builder: SelectBuilder<I>) -> Self
```

- Calls `builder.build_sql()`, stores result as `QuerySource::Built`
- If `builder.keyset_column` is `Some`, propagates `keyset_column` and `keyset_key_fn` to the reader (overrides any prior `.with_keyset()` call)

#### `.query()` unchanged

```rust
pub fn query(mut self, query: &'a str) -> Self
```

Still works, stores `QuerySource::Raw`. `.with_keyset()` on the builder remains available for `.query()` users.

---

### 3. Module exports

`src/item/rdbc/mod.rs` re-exports `SelectBuilder`:

```rust
pub use select_builder::SelectBuilder;
```

---

## Usage examples

### Simple filter + keyset (new path)

```rust
let reader = RdbcItemReaderBuilder::<User>::new()
    .postgres(pool)
    .select(
        SelectBuilder::from("users")
            .columns(&["id", "name", "email"])
            .where_eq("status", "ACTIVE")
            .where_is_null("deleted_at")
            .order_by_keyset("id", |u: &User| u.id.to_string())
    )
    .with_page_size(1_000)
    .build_postgres();
```

Generated SQL before pagination: `SELECT id, name, email FROM users WHERE status = 'ACTIVE' AND deleted_at IS NULL ORDER BY id ASC`

### Complex join (existing path, unchanged)

```rust
let reader = RdbcItemReaderBuilder::<UserWithRole>::new()
    .postgres(pool)
    .query("SELECT u.id, u.name, r.name AS role FROM users u JOIN roles r ON u.role_id = r.id")
    .with_page_size(500)
    .with_keyset("id", |u: &UserWithRole| u.id.to_string())
    .build_postgres();
```

---

## Files to create / modify

| File | Action |
|---|---|
| `src/item/rdbc/select_builder.rs` | Create |
| `src/item/rdbc/unified_reader_builder.rs` | Modify — add `QuerySource`, `.select()` |
| `src/item/rdbc/mod.rs` | Modify — export `SelectBuilder` |

Writers and the three database-specific readers (`postgres_reader.rs`, `mysql_reader.rs`, `sqlite_reader.rs`) are **not touched**.

---

## Testing

- `select_builder.rs` gets a `#[cfg(test)]` module covering: `build_sql` output for each condition type, empty columns (SELECT *), multiple conditions, keyset + order interaction
- `unified_reader_builder.rs` tests extended: `.select()` propagates keyset, `.select()` and `.query()` are mutually exclusive (last wins)
- Existing integration tests (`tests/rdbc_*.rs`) continue to pass unchanged

---

## Non-goals (explicit)

- No parameterized bindings (`push_bind`) in this iteration — strings are escaped inline
- No OR conditions, no nested conditions
- No JOIN support in `SelectBuilder`
- Writer-side improvements are out of scope for this spec
