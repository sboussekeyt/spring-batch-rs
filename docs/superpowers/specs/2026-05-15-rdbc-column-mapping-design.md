# Design: RDBC Column Mapping API

**Date:** 2026-05-15  
**Status:** Approved  
**Scope:** `sbrs-lib` — `src/item/rdbc/`

---

## Problem

The RDBC writer requires users to implement the `DatabaseItemBinder<O, DB>` trait manually for each item type. This means:

- Boilerplate per struct: implement `bind()` for each database backend separately
- The builder has three binder fields (`postgres_binder`, `mysql_binder`, `sqlite_binder`) and three setter methods, one per database
- Adding or removing a column requires modifying the trait impl
- The trait is generic over `DB` (the sqlx database type), which leaks sqlx internals into user code

## Goal

Replace `DatabaseItemBinder` entirely with a fluent `.column(name, extractor)` method on `RdbcItemWriterBuilder`. The extractor closure maps an item to a `ColumnValue` — a type-erased enum covering all primitive sqlx types plus `Null` for `Option<T>`.

---

## Design

### 1. `ColumnValue` enum

New file: `src/item/rdbc/column_value.rs`

```rust
pub enum ColumnValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    Bytes(Vec<u8>),
    Null,
}
```

#### `From` implementations

| Source type | `ColumnValue` variant |
|---|---|
| `i32`, `i64` | `Int` |
| `f32`, `f64` | `Float` |
| `bool` | `Bool` |
| `&str`, `String` | `Text` |
| `Vec<u8>` | `Bytes` |
| `Option<T: Into<ColumnValue>>` | `Null` (when `None`) or delegates to `T` (when `Some`) |

String values are stored as-is (no escaping needed — values go through `push_bind`, not string concatenation).

---

### 2. `RdbcItemWriterBuilder` changes

File: `src/item/rdbc/unified_writer_builder.rs`

#### Removed

- Fields: `postgres_binder`, `mysql_binder`, `sqlite_binder`
- Methods: `.postgres_binder()`, `.mysql_binder()`, `.sqlite_binder()`

#### Added

```rust
column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>
```

```rust
pub fn column(mut self, name: &str, extractor: impl Fn(&O) -> ColumnValue + 'static) -> Self {
    self.column_bindings.push((name.to_string(), Box::new(extractor)));
    self
}
```

The accumulated `column_bindings` list is passed to all three writer constructors (Postgres, MySQL, SQLite) at build time. Column order in the INSERT matches the order of `.column()` calls.

---

### 3. Writer struct changes

Files: `postgres_writer.rs`, `mysql_writer.rs`, `sqlite_writer.rs`

#### Removed

- `item_binder: Option<&'a dyn DatabaseItemBinder<O, DB>>` field and its lifetime `'a`
- The `'a` lifetime parameter on the writer struct and its `impl` blocks

#### Added

```rust
column_bindings: Vec<(String, Box<dyn Fn(&O) -> ColumnValue>)>
```

#### `write()` implementation

```rust
push_values(items, |mut b, item| {
    for (_, extractor) in &self.column_bindings {
        match extractor(item) {
            ColumnValue::Int(v)   => { b.push_bind(v); }
            ColumnValue::Float(v) => { b.push_bind(v); }
            ColumnValue::Text(v)  => { b.push_bind(v); }
            ColumnValue::Bool(v)  => { b.push_bind(v); }
            ColumnValue::Bytes(v) => { b.push_bind(v); }
            ColumnValue::Null     => { b.push_bind(Option::<String>::None); }
        }
    }
})
```

The INSERT column list is built from the `column_bindings` names:
```sql
INSERT INTO table (col1, col2, col3) VALUES (?, ?, ?), ...
```

---

### 4. `DatabaseItemBinder` removal

File: `src/item/rdbc/writer_common.rs`

The `DatabaseItemBinder<O, DB>` trait is deleted entirely. No deprecation shim — this is a clean break.

---

### 5. Module exports

`src/item/rdbc/mod.rs`:

```rust
pub use column_value::ColumnValue;
// Remove: pub use writer_common::DatabaseItemBinder;
```

---

## Usage example

```rust
RdbcItemWriterBuilder::<User>::new()
    .postgres(pool)
    .table("users")
    .column("id", |u: &User| u.id.into())
    .column("name", |u: &User| u.name.as_str().into())
    .column("score", |u: &User| u.score.map(|s| s as f64).into())
    .column("avatar", |u: &User| u.avatar.clone().map(ColumnValue::Bytes).unwrap_or(ColumnValue::Null))
    .build_postgres();
```

---

## Files to create / modify

| File | Action |
|---|---|
| `src/item/rdbc/column_value.rs` | Create |
| `src/item/rdbc/unified_writer_builder.rs` | Modify — add `.column()`, remove binder fields/methods |
| `src/item/rdbc/postgres_writer.rs` | Modify — replace `item_binder` with `column_bindings` |
| `src/item/rdbc/mysql_writer.rs` | Modify — replace `item_binder` with `column_bindings` |
| `src/item/rdbc/sqlite_writer.rs` | Modify — replace `item_binder` with `column_bindings` |
| `src/item/rdbc/writer_common.rs` | Modify — delete `DatabaseItemBinder` trait |
| `src/item/rdbc/mod.rs` | Modify — export `ColumnValue`, remove `DatabaseItemBinder` |
| `tests/rdbc_postgres.rs` | Modify — rewrite writer tests with `.column()` |
| `tests/rdbc_mysql.rs` | Modify — rewrite writer tests with `.column()` |
| `tests/rdbc_sqlite.rs` | Modify — rewrite writer tests with `.column()` |
| `examples/` | Modify — update examples that use `DatabaseItemBinder` |

Writers and the three database-specific readers are not touched beyond removing the binder lifetime.

---

## Testing

### `column_value.rs` unit tests (`#[cfg(test)]`)

- Each `From` impl: `i32 → Int`, `f64 → Float`, `&str → Text`, `bool → Bool`, `Vec<u8> → Bytes`
- `Option<T>`: `Some(42i32)` → `Int(42)`, `None::<i32>` → `Null`

### Writer struct unit tests (inline, no DB)

- `should_store_column_bindings_in_order` — `.column()` calls accumulate in order
- `should_return_null_for_none_option` — `Option::None` → `ColumnValue::Null`

### Integration tests (`tests/rdbc_*.rs`)

- Existing write tests rewritten to use `.column()` instead of `DatabaseItemBinder` — same assertions, new API
- Add `should_write_row_with_null_optional_column` — inserts a row with `Option::None`, reads it back, asserts NULL in DB

---

## Non-goals (explicit)

- No parameterized binding for reads (`SelectBuilder` handles that separately)
- No `OR` conditions or composite column expressions
- No upsert / conflict resolution support
- No batch size tuning (already handled by chunk size)
- No backwards compatibility with `DatabaseItemBinder`
