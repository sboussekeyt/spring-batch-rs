# RDBC SelectBuilder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fluent `SelectBuilder<I>` that generates SQL SELECT strings as an alternative to raw `.query()` strings on `RdbcItemReaderBuilder`.

**Architecture:** A new database-agnostic `SelectBuilder<I>` struct generates the base SQL string from typed conditions. `RdbcItemReaderBuilder` gains a `.select()` method that extracts the SQL and propagates keyset config. Readers switch from `&'a str` to owned `String` to support the built-query path without lifetime issues.

**Tech Stack:** Rust 2021, sqlx, cargo test `--all-features`

---

## File Map

| File | Change |
|---|---|
| `src/item/rdbc/select_builder.rs` | **Create** — full `SelectBuilder<I>` implementation |
| `src/item/rdbc/postgres_reader.rs` | **Modify** — `query: &'a str` → `query: String`, remove `'a` lifetime |
| `src/item/rdbc/mysql_reader.rs` | **Modify** — same as postgres_reader |
| `src/item/rdbc/sqlite_reader.rs` | **Modify** — same as postgres_reader |
| `src/item/rdbc/unified_reader_builder.rs` | **Modify** — add `QuerySource`, `.select()`, update build methods |
| `src/item/rdbc/mod.rs` | **Modify** — declare and export `select_builder` module |

---

## Task 1: Create `select_builder.rs` with internal types and `build_sql`

**Files:**
- Create: `src/item/rdbc/select_builder.rs`

> Note: `ConditionValue`, `WhereClause`, and `OrderClause` are `pub(crate)` — users interact only via builder methods and `impl Into<ConditionValue>` conversions.

- [ ] **Step 1.1: Write the failing tests first**

Add this entire file to `src/item/rdbc/select_builder.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    #[test]
    fn should_generate_select_star_when_no_columns_given() {
        let sql = SelectBuilder::<Dummy>::from("users").build_sql();
        assert_eq!(sql, "SELECT * FROM users", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_column_list() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .columns(&["id", "amount", "status"])
            .build_sql();
        assert_eq!(sql, "SELECT id, amount, status FROM orders", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_eq_for_string() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("status", "ACTIVE")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE status = 'ACTIVE'", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_eq_for_integer() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("age", 30i32)
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE age = 30", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_eq_for_bool() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("active", true)
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE active = true", "unexpected: {sql}");
    }

    #[test]
    fn should_escape_single_quotes_in_string_values() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("name", "O'Brien")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE name = 'O''Brien'", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_not_eq() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_not_eq("role", "ADMIN")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE role != 'ADMIN'", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_gt() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gt("amount", 100i64)
            .build_sql();
        assert_eq!(sql, "SELECT * FROM orders WHERE amount > 100", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_gte() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gte("score", 0.5f64)
            .build_sql();
        assert!(sql.starts_with("SELECT * FROM orders WHERE score >= "), "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_lt() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lt("stock", 10i32)
            .build_sql();
        assert_eq!(sql, "SELECT * FROM items WHERE stock < 10", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_lte() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lte("rank", 100i32)
            .build_sql();
        assert_eq!(sql, "SELECT * FROM items WHERE rank <= 100", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_like() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_like("email", "%@corp.com")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE email LIKE '%@corp.com'", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_is_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_null("deleted_at")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE deleted_at IS NULL", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_where_is_not_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_not_null("confirmed_at")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE confirmed_at IS NOT NULL", "unexpected: {sql}");
    }

    #[test]
    fn should_join_multiple_conditions_with_and() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("status", "ACTIVE")
            .where_is_null("deleted_at")
            .where_gt("age", 18i32)
            .build_sql();
        assert_eq!(
            sql,
            "SELECT * FROM users WHERE status = 'ACTIVE' AND deleted_at IS NULL AND age > 18",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_order_by_asc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("created_at")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at ASC", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_order_by_desc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_desc("score")
            .build_sql();
        assert_eq!(sql, "SELECT * FROM users ORDER BY score DESC", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_multiple_order_by_clauses() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("last_name")
            .order_by_desc("score")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT * FROM users ORDER BY last_name ASC, score DESC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_full_select_with_columns_conditions_and_order() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .columns(&["id", "name", "email"])
            .where_eq("status", "ACTIVE")
            .where_is_null("deleted_at")
            .order_by_asc("id")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT id, name, email FROM users WHERE status = 'ACTIVE' AND deleted_at IS NULL ORDER BY id ASC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_set_keyset_column_and_key_fn_on_order_by_keyset() {
        let builder = SelectBuilder::<Dummy>::from("users")
            .order_by_keyset("id", |_: &Dummy| "0".to_string());
        assert_eq!(builder.keyset_column.as_deref(), Some("id"), "keyset column should be set");
        assert!(builder.keyset_key_fn.is_some(), "keyset key fn should be set");
    }

    #[test]
    fn should_replace_previous_order_by_on_keyset() {
        let builder = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("name")
            .order_by_keyset("id", |_: &Dummy| "0".to_string());
        let sql = builder.build_sql();
        assert_eq!(sql, "SELECT * FROM users ORDER BY id ASC", "previous order_by should be replaced: {sql}");
    }

    #[test]
    fn should_generate_order_by_asc_in_sql_for_keyset() {
        let sql = SelectBuilder::<Dummy>::from("events")
            .order_by_keyset("id", |_: &Dummy| "0".to_string())
            .build_sql();
        assert_eq!(sql, "SELECT * FROM events ORDER BY id ASC", "unexpected: {sql}");
    }
}
```

- [ ] **Step 1.2: Run the tests — expect compilation failure**

```bash
cargo test --features rdbc-sqlite -p sbrs-lib 2>&1 | head -20
```

Expected: compile error — `select_builder` module not found or `SelectBuilder` not defined.

- [ ] **Step 1.3: Write the full implementation**

Replace the contents of `src/item/rdbc/select_builder.rs` with:

```rust
//! Fluent SQL SELECT builder for RDBC item readers.
//!
//! Use [`SelectBuilder`] to construct a `SELECT` query without writing raw SQL.
//! Pass the result to [`RdbcItemReaderBuilder::select`] instead of `.query()`.
//!
//! # Examples
//!
//! ```
//! use spring_batch_rs::item::rdbc::SelectBuilder;
//!
//! struct Order { id: i64, amount: f64 }
//!
//! let sql = SelectBuilder::<Order>::from("orders")
//!     .columns(&["id", "amount"])
//!     .where_eq("status", "PENDING")
//!     .where_gt("amount", 0.0f64)
//!     .order_by_asc("id")
//!     .build_sql();
//!
//! assert!(sql.starts_with("SELECT id, amount FROM orders WHERE"));
//! ```

use std::marker::PhantomData;

pub(crate) enum ConditionValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

impl ConditionValue {
    fn to_sql(&self) -> String {
        match self {
            ConditionValue::Integer(n) => n.to_string(),
            ConditionValue::Float(f) => f.to_string(),
            ConditionValue::Bool(b) => b.to_string(),
            ConditionValue::Text(s) => format!("'{}'", s.replace('\'', "''")),
        }
    }
}

impl From<i32> for ConditionValue {
    fn from(v: i32) -> Self {
        ConditionValue::Integer(i64::from(v))
    }
}

impl From<i64> for ConditionValue {
    fn from(v: i64) -> Self {
        ConditionValue::Integer(v)
    }
}

impl From<f32> for ConditionValue {
    fn from(v: f32) -> Self {
        ConditionValue::Float(f64::from(v))
    }
}

impl From<f64> for ConditionValue {
    fn from(v: f64) -> Self {
        ConditionValue::Float(v)
    }
}

impl From<bool> for ConditionValue {
    fn from(v: bool) -> Self {
        ConditionValue::Bool(v)
    }
}

impl From<&str> for ConditionValue {
    fn from(v: &str) -> Self {
        ConditionValue::Text(v.to_string())
    }
}

impl From<String> for ConditionValue {
    fn from(v: String) -> Self {
        ConditionValue::Text(v)
    }
}

pub(crate) enum WhereClause {
    Eq(String, ConditionValue),
    NotEq(String, ConditionValue),
    Gt(String, ConditionValue),
    Gte(String, ConditionValue),
    Lt(String, ConditionValue),
    Lte(String, ConditionValue),
    Like(String, String),
    IsNull(String),
    IsNotNull(String),
}

impl WhereClause {
    fn to_sql(&self) -> String {
        match self {
            WhereClause::Eq(col, val) => format!("{} = {}", col, val.to_sql()),
            WhereClause::NotEq(col, val) => format!("{} != {}", col, val.to_sql()),
            WhereClause::Gt(col, val) => format!("{} > {}", col, val.to_sql()),
            WhereClause::Gte(col, val) => format!("{} >= {}", col, val.to_sql()),
            WhereClause::Lt(col, val) => format!("{} < {}", col, val.to_sql()),
            WhereClause::Lte(col, val) => format!("{} <= {}", col, val.to_sql()),
            WhereClause::Like(col, pat) => {
                format!("{} LIKE '{}'", col, pat.replace('\'', "''"))
            }
            WhereClause::IsNull(col) => format!("{} IS NULL", col),
            WhereClause::IsNotNull(col) => format!("{} IS NOT NULL", col),
        }
    }
}

pub(crate) enum OrderClause {
    Asc(String),
    Desc(String),
}

/// Fluent builder for SQL `SELECT` queries used with [`RdbcItemReaderBuilder::select`].
///
/// Generic over `I` (the item type) only when [`order_by_keyset`] is used to carry
/// the cursor extraction closure. In all other cases `I` is inferred from context.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::rdbc::SelectBuilder;
///
/// struct User { id: i64, name: String }
///
/// let sql = SelectBuilder::<User>::from("users")
///     .columns(&["id", "name"])
///     .where_eq("active", true)
///     .order_by_keyset("id", |u: &User| u.id.to_string())
///     .build_sql();
///
/// assert_eq!(sql, "SELECT id, name FROM users WHERE active = true ORDER BY id ASC");
/// ```
pub struct SelectBuilder<I> {
    table: String,
    columns: Vec<String>,
    conditions: Vec<WhereClause>,
    order_by: Vec<OrderClause>,
    pub(crate) keyset_column: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(crate) keyset_key_fn: Option<Box<dyn Fn(&I) -> String>>,
    _phantom: PhantomData<I>,
}

impl<I> SelectBuilder<I> {
    /// Creates a new builder targeting the given table.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    /// let builder = SelectBuilder::<Row>::from("users");
    /// assert_eq!(builder.build_sql(), "SELECT * FROM users");
    /// ```
    pub fn from(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            keyset_column: None,
            keyset_key_fn: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the columns to select. Omitting this call generates `SELECT *`.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    /// let sql = SelectBuilder::<Row>::from("t").columns(&["a", "b"]).build_sql();
    /// assert_eq!(sql, "SELECT a, b FROM t");
    /// ```
    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Adds a `col = value` condition.
    pub fn where_eq(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Eq(col.to_string(), val.into()));
        self
    }

    /// Adds a `col != value` condition.
    pub fn where_not_eq(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::NotEq(col.to_string(), val.into()));
        self
    }

    /// Adds a `col > value` condition.
    pub fn where_gt(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Gt(col.to_string(), val.into()));
        self
    }

    /// Adds a `col >= value` condition.
    pub fn where_gte(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Gte(col.to_string(), val.into()));
        self
    }

    /// Adds a `col < value` condition.
    pub fn where_lt(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Lt(col.to_string(), val.into()));
        self
    }

    /// Adds a `col <= value` condition.
    pub fn where_lte(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Lte(col.to_string(), val.into()));
        self
    }

    /// Adds a `col LIKE pattern` condition.
    pub fn where_like(mut self, col: &str, pat: &str) -> Self {
        self.conditions
            .push(WhereClause::Like(col.to_string(), pat.to_string()));
        self
    }

    /// Adds a `col IS NULL` condition.
    pub fn where_is_null(mut self, col: &str) -> Self {
        self.conditions.push(WhereClause::IsNull(col.to_string()));
        self
    }

    /// Adds a `col IS NOT NULL` condition.
    pub fn where_is_not_null(mut self, col: &str) -> Self {
        self.conditions
            .push(WhereClause::IsNotNull(col.to_string()));
        self
    }

    /// Adds an `ORDER BY col ASC` clause.
    pub fn order_by_asc(mut self, col: &str) -> Self {
        self.order_by.push(OrderClause::Asc(col.to_string()));
        self
    }

    /// Adds an `ORDER BY col DESC` clause.
    pub fn order_by_desc(mut self, col: &str) -> Self {
        self.order_by.push(OrderClause::Desc(col.to_string()));
        self
    }

    /// Configures keyset (cursor) pagination on `col`.
    ///
    /// Replaces all previous `order_by_*` calls. Sets `ORDER BY col ASC` and stores
    /// `key_fn` to extract the cursor value from each item. When passed to
    /// [`RdbcItemReaderBuilder::select`], keyset pagination is enabled automatically
    /// without a separate `.with_keyset()` call.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct User { id: i64 }
    ///
    /// let sql = SelectBuilder::<User>::from("users")
    ///     .order_by_keyset("id", |u: &User| u.id.to_string())
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users ORDER BY id ASC");
    /// ```
    pub fn order_by_keyset(mut self, col: &str, key_fn: impl Fn(&I) -> String + 'static) -> Self {
        self.order_by.clear();
        self.order_by.push(OrderClause::Asc(col.to_string()));
        self.keyset_column = Some(col.to_string());
        self.keyset_key_fn = Some(Box::new(key_fn));
        self
    }

    /// Generates the base SQL string (without `LIMIT`, `OFFSET`, or keyset `WHERE` clauses).
    ///
    /// Those clauses are appended by the reader at execution time.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    /// let sql = SelectBuilder::<Row>::from("items")
    ///     .columns(&["id", "name"])
    ///     .where_is_null("deleted_at")
    ///     .order_by_asc("id")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT id, name FROM items WHERE deleted_at IS NULL ORDER BY id ASC");
    /// ```
    pub fn build_sql(&self) -> String {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols, self.table);

        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self.conditions.iter().map(WhereClause::to_sql).collect();
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            let clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|o| match o {
                    OrderClause::Asc(col) => format!("{} ASC", col),
                    OrderClause::Desc(col) => format!("{} DESC", col),
                })
                .collect();
            sql.push_str(" ORDER BY ");
            sql.push_str(&clauses.join(", "));
        }

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    #[test]
    fn should_generate_select_star_when_no_columns_given() {
        let sql = SelectBuilder::<Dummy>::from("users").build_sql();
        assert_eq!(sql, "SELECT * FROM users", "unexpected: {sql}");
    }

    #[test]
    fn should_generate_column_list() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .columns(&["id", "amount", "status"])
            .build_sql();
        assert_eq!(
            sql, "SELECT id, amount, status FROM orders",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_eq_for_string() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("status", "ACTIVE")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE status = 'ACTIVE'",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_eq_for_integer() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("age", 30i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE age = 30",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_eq_for_bool() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("active", true)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE active = true",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_escape_single_quotes_in_string_values() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("name", "O'Brien")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE name = 'O''Brien'",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_not_eq() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_not_eq("role", "ADMIN")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE role != 'ADMIN'",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_gt() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gt("amount", 100i64)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM orders WHERE amount > 100",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_gte() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gte("score", 0.5f64)
            .build_sql();
        assert!(
            sql.starts_with("SELECT * FROM orders WHERE score >= "),
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_lt() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lt("stock", 10i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM items WHERE stock < 10",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_lte() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lte("rank", 100i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM items WHERE rank <= 100",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_like() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_like("email", "%@corp.com")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE email LIKE '%@corp.com'",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_is_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_null("deleted_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE deleted_at IS NULL",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_where_is_not_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_not_null("confirmed_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE confirmed_at IS NOT NULL",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_join_multiple_conditions_with_and() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("status", "ACTIVE")
            .where_is_null("deleted_at")
            .where_gt("age", 18i32)
            .build_sql();
        assert_eq!(
            sql,
            "SELECT * FROM users WHERE status = 'ACTIVE' AND deleted_at IS NULL AND age > 18",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_order_by_asc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("created_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users ORDER BY created_at ASC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_order_by_desc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_desc("score")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users ORDER BY score DESC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_multiple_order_by_clauses() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("last_name")
            .order_by_desc("score")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT * FROM users ORDER BY last_name ASC, score DESC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_generate_full_select_with_columns_conditions_and_order() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .columns(&["id", "name", "email"])
            .where_eq("status", "ACTIVE")
            .where_is_null("deleted_at")
            .order_by_asc("id")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT id, name, email FROM users WHERE status = 'ACTIVE' AND deleted_at IS NULL ORDER BY id ASC",
            "unexpected: {sql}"
        );
    }

    #[test]
    fn should_set_keyset_column_and_key_fn_on_order_by_keyset() {
        let builder = SelectBuilder::<Dummy>::from("users")
            .order_by_keyset("id", |_: &Dummy| "0".to_string());
        assert_eq!(
            builder.keyset_column.as_deref(),
            Some("id"),
            "keyset column should be set"
        );
        assert!(
            builder.keyset_key_fn.is_some(),
            "keyset key fn should be set"
        );
    }

    #[test]
    fn should_replace_previous_order_by_on_keyset() {
        let builder = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("name")
            .order_by_keyset("id", |_: &Dummy| "0".to_string());
        let sql = builder.build_sql();
        assert_eq!(
            sql, "SELECT * FROM users ORDER BY id ASC",
            "previous order_by should be replaced: {sql}"
        );
    }

    #[test]
    fn should_generate_order_by_asc_in_sql_for_keyset() {
        let sql = SelectBuilder::<Dummy>::from("events")
            .order_by_keyset("id", |_: &Dummy| "0".to_string())
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM events ORDER BY id ASC",
            "unexpected: {sql}"
        );
    }
}
```

- [ ] **Step 1.4: Declare the module in `mod.rs`**

In `src/item/rdbc/mod.rs`, add before the existing `pub mod postgres_reader;` line:

```rust
/// Fluent SQL SELECT builder for RDBC item readers.
pub mod select_builder;
```

And add to the re-exports at the bottom:

```rust
pub use select_builder::SelectBuilder;
```

- [ ] **Step 1.5: Run tests and verify they pass**

```bash
cargo test select_builder --features rdbc-sqlite -p sbrs-lib 2>&1
```

Expected: all `select_builder::tests::*` tests PASS, zero warnings.

- [ ] **Step 1.6: Commit**

```bash
git add src/item/rdbc/select_builder.rs src/item/rdbc/mod.rs
git commit -m "feat(rdbc): add SelectBuilder for fluent SQL SELECT generation"
```

---

## Task 2: Migrate readers from borrowed `&'a str` to owned `String`

The readers currently store `query: &'a str`, which prevents the builder from passing a `String` produced by `SelectBuilder::build_sql()`. Changing to `String` removes the `'a` lifetime from the reader structs entirely.

**Files:**
- Modify: `src/item/rdbc/postgres_reader.rs`
- Modify: `src/item/rdbc/mysql_reader.rs`
- Modify: `src/item/rdbc/sqlite_reader.rs`

- [ ] **Step 2.1: Migrate `postgres_reader.rs`**

Apply the following changes to `src/item/rdbc/postgres_reader.rs`:

**Change 1** — struct definition: remove `'a`, change `query` field type.

Old:
```rust
pub struct PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Postgres>,
    pub(crate) query: &'a str,
```

New:
```rust
pub struct PostgresRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
{
    pub(crate) pool: Pool<Postgres>,
    pub(crate) query: String,
```

**Change 2** — `impl` block signature:

Old:
```rust
impl<'a, I> PostgresRdbcItemReader<'a, I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
```

New:
```rust
impl<I> PostgresRdbcItemReader<I>
where
    for<'r> I: FromRow<'r, PgRow> + Send + Unpin + Clone,
```

**Change 3** — `new()` signature:

Old:
```rust
pub fn new(
    pool: Pool<Postgres>,
    query: &'a str,
    page_size: Option<i32>,
    keyset_column: Option<String>,
    keyset_key: Option<Box<dyn Fn(&I) -> String>>,
) -> Self {
    Self {
        pool,
        query,
```

New:
```rust
pub fn new(
    pool: Pool<Postgres>,
    query: String,
    page_size: Option<i32>,
    keyset_column: Option<String>,
    keyset_key: Option<Box<dyn Fn(&I) -> String>>,
) -> Self {
    Self {
        pool,
        query,
```

**Change 4** — `read_page()`: change `self.query` reference (it's already a `String`, so this works without change since `QueryBuilder::new` accepts `&str` and `String` deref to `&str`). No change needed here.

**Change 5** — `ItemReader` impl signature:

Old:
```rust
impl<I> ItemReader<I> for PostgresRdbcItemReader<'_, I>
```

New:
```rust
impl<I> ItemReader<I> for PostgresRdbcItemReader<I>
```

**Change 6** — update the test `reader_with_keyset` helper:

Old:
```rust
fn reader_with_keyset(keyset: bool) -> PostgresRdbcItemReader<'static, Dummy> {
    ...
    PostgresRdbcItemReader::new(pool, "SELECT 1", Some(10), col, key)
```

New:
```rust
fn reader_with_keyset(keyset: bool) -> PostgresRdbcItemReader<Dummy> {
    ...
    PostgresRdbcItemReader::new(pool, "SELECT 1".to_string(), Some(10), col, key)
```

- [ ] **Step 2.2: Migrate `mysql_reader.rs`**

Apply the identical set of changes to `src/item/rdbc/mysql_reader.rs` (replace `Postgres`/`PgRow` with `MySql`/`MySqlRow` in type names):

- `MySqlRdbcItemReader<'a, I>` → `MySqlRdbcItemReader<I>`
- `query: &'a str` → `query: String`
- `impl<'a, I> MySqlRdbcItemReader<'a, I>` → `impl<I> MySqlRdbcItemReader<I>`
- `new(pool, query: &'a str, ...)` → `new(pool, query: String, ...)`
- `impl<I> ItemReader<I> for MySqlRdbcItemReader<'_, I>` → `MySqlRdbcItemReader<I>`

- [ ] **Step 2.3: Migrate `sqlite_reader.rs`**

Apply the identical changes to `src/item/rdbc/sqlite_reader.rs` (replace with `Sqlite`/`SqliteRow`):

- `SqliteRdbcItemReader<'a, I>` → `SqliteRdbcItemReader<I>`
- `query: &'a str` → `query: String`
- `impl<'a, I> SqliteRdbcItemReader<'a, I>` → `impl<I> SqliteRdbcItemReader<I>`
- `new(pool, query: &'a str, ...)` → `new(pool, query: String, ...)`
- `impl<I> ItemReader<I> for SqliteRdbcItemReader<'_, I>` → `SqliteRdbcItemReader<I>`
- In the test helper `make_reader`: `SqliteRdbcItemReader::<Row>::new(pool, query, ...)` → add `.to_string()` to the query argument
- In all test calls that pass a `&str` query to `SqliteRdbcItemReader::new`, add `.to_string()`

Specifically in `sqlite_reader.rs` tests, these lines change:
```rust
// Old
fn make_reader(pool: SqlitePool, query: &str, page_size: Option<i32>) -> SqliteRdbcItemReader<'_, Row> {
    SqliteRdbcItemReader::<Row>::new(pool, query, page_size, None, None)
}

// New
fn make_reader(pool: SqlitePool, query: &str, page_size: Option<i32>) -> SqliteRdbcItemReader<Row> {
    SqliteRdbcItemReader::<Row>::new(pool, query.to_string(), page_size, None, None)
}
```

And the two tests that call `SqliteRdbcItemReader::new` directly with `"SELECT id, name FROM items"`:
```rust
// Old
let reader = SqliteRdbcItemReader::<Row>::new(
    pool,
    "SELECT id, name FROM items",
    Some(2),
    ...
);

// New
let reader = SqliteRdbcItemReader::<Row>::new(
    pool,
    "SELECT id, name FROM items".to_string(),
    Some(2),
    ...
);
```

- [ ] **Step 2.4: Run tests to verify no regressions**

```bash
cargo test --features rdbc-sqlite -p sbrs-lib 2>&1
```

Expected: all existing reader and builder tests PASS.

- [ ] **Step 2.5: Commit**

```bash
git add src/item/rdbc/postgres_reader.rs src/item/rdbc/mysql_reader.rs src/item/rdbc/sqlite_reader.rs
git commit -m "refactor(rdbc): readers own their query String, remove 'a lifetime"
```

---

## Task 3: Add `QuerySource` and `.select()` to `RdbcItemReaderBuilder`

**Files:**
- Modify: `src/item/rdbc/unified_reader_builder.rs`

- [ ] **Step 3.1: Write the failing tests first**

Add these tests to the `#[cfg(test)]` block in `unified_reader_builder.rs` (after the existing tests):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn should_build_sqlite_reader_from_select_builder() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let reader = RdbcItemReaderBuilder::<Dummy>::new()
        .sqlite(pool)
        .select(
            SelectBuilder::from("items")
                .columns(&["id"])
                .where_eq("active", true)
                .order_by_asc("id"),
        )
        .build_sqlite();
    assert_eq!(
        reader.query,
        "SELECT id FROM items WHERE active = true ORDER BY id ASC",
        "select builder SQL should be stored in reader"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn should_propagate_keyset_from_select_builder_to_sqlite_reader() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let reader = RdbcItemReaderBuilder::<Dummy>::new()
        .sqlite(pool)
        .select(
            SelectBuilder::from("items")
                .order_by_keyset("id", |d: &Dummy| d.id.to_string()),
        )
        .with_page_size(10)
        .build_sqlite();
    assert_eq!(
        reader.keyset_column.as_deref(),
        Some("id"),
        "keyset column should propagate from SelectBuilder"
    );
    assert!(
        reader.keyset_key.is_some(),
        "keyset key fn should propagate from SelectBuilder"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn should_prefer_select_over_query_when_called_last() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let reader = RdbcItemReaderBuilder::<Dummy>::new()
        .sqlite(pool.clone())
        .query("SELECT id FROM old_table")
        .select(SelectBuilder::from("new_table").columns(&["id"]))
        .build_sqlite();
    assert_eq!(
        reader.query, "SELECT id FROM new_table",
        "select() called last should win"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn should_prefer_query_over_select_when_called_last() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let reader = RdbcItemReaderBuilder::<Dummy>::new()
        .sqlite(pool)
        .select(SelectBuilder::from("old_table").columns(&["id"]))
        .query("SELECT id FROM new_table")
        .build_sqlite();
    assert_eq!(
        reader.query, "SELECT id FROM new_table",
        "query() called last should win"
    );
}
```

- [ ] **Step 3.2: Run the new tests — expect failure**

```bash
cargo test should_build_sqlite_reader_from_select_builder --features rdbc-sqlite -p sbrs-lib 2>&1
```

Expected: compile error — `.select()` method not found.

- [ ] **Step 3.3: Add `QuerySource` enum and update the builder struct**

At the top of `unified_reader_builder.rs`, add the import for `SelectBuilder`:

```rust
use super::select_builder::SelectBuilder;
```

Add the `QuerySource` enum after the imports:

```rust
enum QuerySource<'a> {
    Raw(&'a str),
    Built(String),
}
```

In `RdbcItemReaderBuilder`, replace:

```rust
query: Option<&'a str>,
```

with:

```rust
query_source: Option<QuerySource<'a>>,
```

In `RdbcItemReaderBuilder::new()`, replace:

```rust
query: None,
```

with:

```rust
query_source: None,
```

- [ ] **Step 3.4: Update `.query()` method and add `.select()` method**

Replace the existing `.query()` method:

```rust
pub fn query(mut self, query: &'a str) -> Self {
    self.query_source = Some(QuerySource::Raw(query));
    self
}
```

Add `.select()` right after `.query()`:

```rust
/// Sets the query using a [`SelectBuilder`] instead of a raw SQL string.
///
/// If the [`SelectBuilder`] was configured with [`SelectBuilder::order_by_keyset`],
/// keyset pagination is enabled automatically — no need to call `.with_keyset()`.
///
/// Calling `.select()` after `.query()` (or vice-versa) uses the last call.
///
/// # Examples
///
/// ```no_run
/// use spring_batch_rs::item::rdbc::{RdbcItemReaderBuilder, SelectBuilder};
/// use sqlx::SqlitePool;
///
/// # #[derive(sqlx::FromRow, Clone)]
/// # struct Item { id: i32, name: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = SqlitePool::connect("sqlite::memory:").await?;
///
/// let reader = RdbcItemReaderBuilder::<Item>::new()
///     .sqlite(pool)
///     .select(
///         SelectBuilder::from("items")
///             .columns(&["id", "name"])
///             .where_eq("active", true)
///             .order_by_keyset("id", |i: &Item| i.id.to_string()),
///     )
///     .with_page_size(100)
///     .build_sqlite();
/// # Ok(())
/// # }
/// ```
pub fn select(mut self, builder: SelectBuilder<I>) -> Self {
    if let Some(col) = builder.keyset_column {
        self.keyset_column = Some(col);
    }
    if let Some(key_fn) = builder.keyset_key_fn {
        self.keyset_key_fn = Some(key_fn);
    }
    self.query_source = Some(QuerySource::Built(builder.build_sql()));
    self
}
```

> Note: `builder.build_sql()` is called before `builder.keyset_column` and `builder.keyset_key_fn` are moved out, but since `build_sql` takes `&self` we need to call it on a reference or restructure. Move the `build_sql` call to happen on a reference before destructuring. Adjust to:

```rust
pub fn select(mut self, builder: SelectBuilder<I>) -> Self {
    let sql = builder.build_sql();
    if let Some(col) = builder.keyset_column {
        self.keyset_column = Some(col);
    }
    if let Some(key_fn) = builder.keyset_key_fn {
        self.keyset_key_fn = Some(key_fn);
    }
    self.query_source = Some(QuerySource::Built(sql));
    self
}
```

- [ ] **Step 3.5: Update `build_postgres`, `build_mysql`, `build_sqlite` to use `query_source`**

Add this helper at the top of each `impl` block that has a `build_*` method, or as a method on the builder:

In each `build_*` method, replace:

```rust
self.query.expect("Query is required"),
```

with:

```rust
match self.query_source.expect("Query is required — call .query() or .select()") {
    QuerySource::Raw(s) => s.to_string(),
    QuerySource::Built(s) => s,
},
```

Full updated `build_postgres`:

```rust
pub fn build_postgres(self) -> PostgresRdbcItemReader<I> {
    let query = match self.query_source.expect("Query is required — call .query() or .select()") {
        QuerySource::Raw(s) => s.to_string(),
        QuerySource::Built(s) => s,
    };
    PostgresRdbcItemReader::new(
        self.postgres_pool.expect("PostgreSQL pool is required"),
        query,
        self.page_size,
        self.keyset_column,
        self.keyset_key_fn,
    )
}
```

Full updated `build_mysql`:

```rust
pub fn build_mysql(self) -> MySqlRdbcItemReader<I> {
    let query = match self.query_source.expect("Query is required — call .query() or .select()") {
        QuerySource::Raw(s) => s.to_string(),
        QuerySource::Built(s) => s,
    };
    MySqlRdbcItemReader::new(
        self.mysql_pool.expect("MySQL pool is required"),
        query,
        self.page_size,
        self.keyset_column,
        self.keyset_key_fn,
    )
}
```

Full updated `build_sqlite`:

```rust
pub fn build_sqlite(self) -> SqliteRdbcItemReader<I> {
    let query = match self.query_source.expect("Query is required — call .query() or .select()") {
        QuerySource::Raw(s) => s.to_string(),
        QuerySource::Built(s) => s,
    };
    SqliteRdbcItemReader::new(
        self.sqlite_pool.expect("SQLite pool is required"),
        query,
        self.page_size,
        self.keyset_column,
        self.keyset_key_fn,
    )
}
```

- [ ] **Step 3.6: Update the existing panic tests**

The existing tests that use `.expect("Query is required")` in the panic message need to match the new message. Update these tests:

```rust
#[test]
#[should_panic(expected = "Query is required")]
fn should_panic_when_building_sqlite_without_pool() { ... }

#[test]
#[should_panic(expected = "Query is required")]
fn should_panic_when_building_postgres_without_pool() { ... }

#[test]
#[should_panic(expected = "Query is required")]
fn should_panic_when_building_mysql_without_pool() { ... }
```

These tests panic because of "PostgreSQL pool is required" / "MySQL pool is required" etc. — they don't reach the query check, so those tests are unaffected. The tests that test missing query (if any) may need their expected message updated. Verify by running the tests.

- [ ] **Step 3.7: Run all tests**

```bash
cargo test --features rdbc-sqlite -p sbrs-lib 2>&1
```

Expected: all tests PASS including the new `should_build_sqlite_reader_from_select_builder` and `should_propagate_keyset_from_select_builder_to_sqlite_reader`.

- [ ] **Step 3.8: Commit**

```bash
git add src/item/rdbc/unified_reader_builder.rs
git commit -m "feat(rdbc): add .select() method to RdbcItemReaderBuilder with QuerySource"
```

---

## Task 4: Final verification

- [ ] **Step 4.1: Run full test suite**

```bash
cargo test --all-features -p sbrs-lib 2>&1
```

Expected: all tests PASS.

- [ ] **Step 4.2: Check clippy**

```bash
cargo clippy --all-features -p sbrs-lib -- -D warnings 2>&1
```

Expected: zero warnings.

- [ ] **Step 4.3: Check doc tests**

```bash
cargo test --doc --all-features -p sbrs-lib 2>&1
```

Expected: all doc tests PASS.

- [ ] **Step 4.4: Commit**

```bash
git add .
git commit -m "chore(rdbc): verify SelectBuilder builds and tests clean"
```

---

## Self-Review Against Spec

| Spec requirement | Covered by |
|---|---|
| `SelectBuilder<I>` in `select_builder.rs` | Task 1 |
| `from()`, `columns()`, all `where_*` methods | Task 1 |
| `order_by_asc`, `order_by_desc` | Task 1 |
| `order_by_keyset` replaces prior order, stores keyset | Task 1 |
| `build_sql()` generates correct SQL | Task 1 tests |
| `ConditionValue` with `Into` impls for all primitives | Task 1 |
| `QuerySource` enum in builder | Task 3 |
| `.select()` on `RdbcItemReaderBuilder` | Task 3 |
| `.select()` propagates keyset column + key_fn | Task 3 |
| `.query()` still works (Raw path) | Task 3 |
| Last-call-wins for `.query()` vs `.select()` | Task 3 tests |
| Readers own their query (`String`) | Task 2 |
| `SelectBuilder` exported from `mod.rs` | Task 1 step 1.4 |
| Existing integration tests unbroken | Task 2 step 2.4, Task 4 |
