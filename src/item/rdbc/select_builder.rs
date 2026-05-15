//! Fluent SQL SELECT builder for RDBC item readers.
//!
//! This module provides [`SelectBuilder`], a type-safe fluent API for constructing
//! SQL `SELECT` statements with optional `WHERE` conditions and `ORDER BY` clauses.
//!
//! # Key Types
//!
//! - [`SelectBuilder`] — the public builder; configure via method chaining and call
//!   [`SelectBuilder::build_sql`] to get the final SQL string.
//!
//! # Examples
//!
//! ```
//! use spring_batch_rs::item::rdbc::SelectBuilder;
//!
//! struct Row;
//!
//! let sql = SelectBuilder::<Row>::from("users")
//!     .columns(&["id", "name", "email"])
//!     .where_eq("active", true)
//!     .order_by_asc("name")
//!     .build_sql();
//!
//! assert_eq!(sql, "SELECT id, name, email FROM users WHERE active = true ORDER BY name ASC");
//! ```

use std::marker::PhantomData;

// ──────────────────────────────────────────────────────────────────────────────
// Internal types
// ──────────────────────────────────────────────────────────────────────────────

/// A typed SQL literal value used inside WHERE conditions.
///
/// Users never construct this directly; instead they pass any type that
/// implements `Into<ConditionValue>` (e.g. `i32`, `bool`, `&str`).
pub(crate) enum ConditionValue {
    /// A 64-bit signed integer.
    Integer(i64),
    /// A 64-bit floating-point number.
    Float(f64),
    /// A text string.
    Text(String),
    /// A boolean.
    Bool(bool),
}

impl ConditionValue {
    /// Renders the value as a SQL literal string.
    pub(crate) fn to_sql(&self) -> String {
        match self {
            ConditionValue::Integer(n) => n.to_string(),
            ConditionValue::Float(f) => f.to_string(),
            ConditionValue::Bool(b) => b.to_string(),
            ConditionValue::Text(s) => format!("'{}'", s.replace('\'', "''")),
        }
    }
}

// `From` implementations — cover the most common primitive types.

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
        ConditionValue::Text(v.to_owned())
    }
}

impl From<String> for ConditionValue {
    fn from(v: String) -> Self {
        ConditionValue::Text(v)
    }
}

// ──────────────────────────────────────────────────────────────────────────────

/// A single WHERE predicate.
pub(crate) enum WhereClause {
    /// `col = val`
    Eq(String, ConditionValue),
    /// `col != val`
    NotEq(String, ConditionValue),
    /// `col > val`
    Gt(String, ConditionValue),
    /// `col >= val`
    Gte(String, ConditionValue),
    /// `col < val`
    Lt(String, ConditionValue),
    /// `col <= val`
    Lte(String, ConditionValue),
    /// `col LIKE 'pattern'`
    Like(String, String),
    /// `col IS NULL`
    IsNull(String),
    /// `col IS NOT NULL`
    IsNotNull(String),
}

impl WhereClause {
    /// Renders the clause as a SQL fragment.
    pub(crate) fn to_sql(&self) -> String {
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

// ──────────────────────────────────────────────────────────────────────────────

/// A single ORDER BY directive.
pub(crate) enum OrderClause {
    /// `col ASC`
    Asc(String),
    /// `col DESC`
    Desc(String),
}

impl OrderClause {
    fn to_sql(&self) -> String {
        match self {
            OrderClause::Asc(col) => format!("{} ASC", col),
            OrderClause::Desc(col) => format!("{} DESC", col),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Public struct
// ──────────────────────────────────────────────────────────────────────────────

/// Fluent builder for SQL `SELECT` statements used by RDBC item readers.
///
/// Call [`SelectBuilder::from`] to create a builder for a given table, chain
/// optional filter/order methods, then call [`SelectBuilder::build_sql`] to
/// obtain the final SQL string.
///
/// # Type Parameters
///
/// * `I` — The item type that will be read from the database. Used only for
///   the keyset key function; when keyset pagination is not needed `I` can be
///   any type (e.g. a unit struct).
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::rdbc::SelectBuilder;
///
/// struct Product;
///
/// let sql = SelectBuilder::<Product>::from("products")
///     .columns(&["id", "name", "price"])
///     .where_gte("price", 10.0_f64)
///     .where_eq("active", true)
///     .order_by_desc("price")
///     .build_sql();
///
/// assert!(sql.starts_with("SELECT id, name, price FROM products WHERE"));
/// assert!(sql.contains("ORDER BY price DESC"));
/// ```
pub struct SelectBuilder<I> {
    table: String,
    columns: Vec<String>,
    conditions: Vec<WhereClause>,
    order_by: Vec<OrderClause>,
    /// Column name used as the keyset cursor.
    pub(crate) keyset_column: Option<String>,
    /// Extracts the keyset cursor value from the last-read item.
    #[allow(clippy::type_complexity)]
    pub(crate) keyset_key_fn: Option<Box<dyn Fn(&I) -> String>>,
    _phantom: PhantomData<I>,
}

#[allow(private_bounds)]
impl<I> SelectBuilder<I> {
    /// Creates a new `SelectBuilder` targeting the given table.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("orders").build_sql();
    /// assert_eq!(sql, "SELECT * FROM orders");
    /// ```
    pub fn from(table: impl Into<String>) -> Self {
        SelectBuilder {
            table: table.into(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            keyset_column: None,
            keyset_key_fn: None,
            _phantom: PhantomData,
        }
    }

    /// Specifies the columns to select.
    ///
    /// When not called (or called with an empty slice), the query uses `SELECT *`.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .columns(&["id", "email"])
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT id, email FROM users");
    /// ```
    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Adds a `col = val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .where_eq("status", "ACTIVE")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users WHERE status = 'ACTIVE'");
    /// ```
    pub fn where_eq(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Eq(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col != val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .where_not_eq("role", "ADMIN")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users WHERE role != 'ADMIN'");
    /// ```
    pub fn where_not_eq(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::NotEq(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col > val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("orders")
    ///     .where_gt("amount", 100_i32)
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM orders WHERE amount > 100");
    /// ```
    pub fn where_gt(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Gt(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col >= val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("orders")
    ///     .where_gte("score", 4.5_f64)
    ///     .build_sql();
    ///
    /// assert!(sql.starts_with("SELECT * FROM orders WHERE score >= "));
    /// ```
    pub fn where_gte(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Gte(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col < val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("items")
    ///     .where_lt("stock", 10_i32)
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM items WHERE stock < 10");
    /// ```
    pub fn where_lt(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Lt(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col <= val` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("items")
    ///     .where_lte("rank", 100_i32)
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM items WHERE rank <= 100");
    /// ```
    pub fn where_lte(mut self, col: &str, val: impl Into<ConditionValue>) -> Self {
        self.conditions
            .push(WhereClause::Lte(col.to_owned(), val.into()));
        self
    }

    /// Adds a `col LIKE 'pattern'` condition.
    ///
    /// Single quotes in `pat` are escaped automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .where_like("email", "%@corp.com")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users WHERE email LIKE '%@corp.com'");
    /// ```
    pub fn where_like(mut self, col: &str, pat: &str) -> Self {
        self.conditions
            .push(WhereClause::Like(col.to_owned(), pat.to_owned()));
        self
    }

    /// Adds a `col IS NULL` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .where_is_null("deleted_at")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users WHERE deleted_at IS NULL");
    /// ```
    pub fn where_is_null(mut self, col: &str) -> Self {
        self.conditions.push(WhereClause::IsNull(col.to_owned()));
        self
    }

    /// Adds a `col IS NOT NULL` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .where_is_not_null("confirmed_at")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users WHERE confirmed_at IS NOT NULL");
    /// ```
    pub fn where_is_not_null(mut self, col: &str) -> Self {
        self.conditions.push(WhereClause::IsNotNull(col.to_owned()));
        self
    }

    /// Appends an `ORDER BY col ASC` clause.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .order_by_asc("created_at")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users ORDER BY created_at ASC");
    /// ```
    pub fn order_by_asc(mut self, col: &str) -> Self {
        self.order_by.push(OrderClause::Asc(col.to_owned()));
        self
    }

    /// Appends an `ORDER BY col DESC` clause.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("users")
    ///     .order_by_desc("score")
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM users ORDER BY score DESC");
    /// ```
    pub fn order_by_desc(mut self, col: &str) -> Self {
        self.order_by.push(OrderClause::Desc(col.to_owned()));
        self
    }

    /// Configures keyset (cursor-based) pagination on `col`.
    ///
    /// Clears any previously configured `ORDER BY` clauses, sets `ORDER BY col ASC`,
    /// and stores `col` and `key_fn` for use by the reader at runtime.
    ///
    /// `key_fn` extracts the cursor value (as a `String`) from the last item
    /// returned by a page, so the next page can request `WHERE col > last_cursor`.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Event { id: i64 }
    ///
    /// let sql = SelectBuilder::<Event>::from("events")
    ///     .order_by_keyset("id", |e: &Event| e.id.to_string())
    ///     .build_sql();
    ///
    /// assert_eq!(sql, "SELECT * FROM events ORDER BY id ASC");
    /// ```
    pub fn order_by_keyset(mut self, col: &str, key_fn: impl Fn(&I) -> String + 'static) -> Self {
        self.order_by.clear();
        self.order_by.push(OrderClause::Asc(col.to_owned()));
        self.keyset_column = Some(col.to_owned());
        self.keyset_key_fn = Some(Box::new(key_fn));
        self
    }

    /// Builds and returns the SQL `SELECT` statement.
    ///
    /// - If no columns were specified, emits `SELECT *`.
    /// - Multiple WHERE conditions are joined with `AND`.
    /// - Multiple ORDER BY clauses are joined with `, `.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::rdbc::SelectBuilder;
    ///
    /// struct Row;
    ///
    /// let sql = SelectBuilder::<Row>::from("orders")
    ///     .columns(&["id", "total"])
    ///     .where_eq("status", "OPEN")
    ///     .where_gt("total", 0_i32)
    ///     .order_by_asc("id")
    ///     .build_sql();
    ///
    /// assert_eq!(
    ///     sql,
    ///     "SELECT id, total FROM orders WHERE status = 'OPEN' AND total > 0 ORDER BY id ASC"
    /// );
    /// ```
    pub fn build_sql(&self) -> String {
        let col_part = if self.columns.is_empty() {
            "*".to_owned()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", col_part, self.table);

        if !self.conditions.is_empty() {
            let where_part = self
                .conditions
                .iter()
                .map(WhereClause::to_sql)
                .collect::<Vec<_>>()
                .join(" AND ");
            sql.push_str(" WHERE ");
            sql.push_str(&where_part);
        }

        if !self.order_by.is_empty() {
            let order_part = self
                .order_by
                .iter()
                .map(OrderClause::to_sql)
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(" ORDER BY ");
            sql.push_str(&order_part);
        }

        sql
    }

    /// Generates the base SQL string without the `ORDER BY` clause.
    ///
    /// Used internally when keyset pagination is active, since the reader
    /// constructs the `ORDER BY` clause itself. Calling this method on a builder
    /// that has no `ORDER BY` configured produces the same result as
    /// [`SelectBuilder::build_sql`].
    pub(crate) fn build_sql_no_order(&self) -> String {
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

        sql
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    // ── SELECT column list ────────────────────────────────────────────────────

    #[test]
    fn should_generate_select_star_when_no_columns_given() {
        let sql = SelectBuilder::<Dummy>::from("users").build_sql();
        assert_eq!(
            sql, "SELECT * FROM users",
            "expected SELECT * when no columns specified"
        );
    }

    #[test]
    fn should_generate_column_list() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .columns(&["id", "amount", "status"])
            .build_sql();
        assert_eq!(
            sql, "SELECT id, amount, status FROM orders",
            "column list was not rendered correctly"
        );
    }

    // ── WHERE conditions ─────────────────────────────────────────────────────

    #[test]
    fn should_generate_where_eq_for_string() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("status", "ACTIVE")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE status = 'ACTIVE'",
            "string equality condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_eq_for_integer() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("age", 30_i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE age = 30",
            "integer equality condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_eq_for_bool() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("active", true)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE active = true",
            "boolean equality condition was not rendered correctly"
        );
    }

    #[test]
    fn should_escape_single_quotes_in_string_values() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_eq("name", "O'Brien")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE name = 'O''Brien'",
            "single quotes in string values were not escaped"
        );
    }

    #[test]
    fn should_generate_where_not_eq() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_not_eq("role", "ADMIN")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE role != 'ADMIN'",
            "not-equal condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_gt() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gt("amount", 100_i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM orders WHERE amount > 100",
            "greater-than condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_gte() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gte("score", 4.5_f64)
            .build_sql();
        assert!(
            sql.starts_with("SELECT * FROM orders WHERE score >= "),
            "greater-than-or-equal condition was not rendered correctly; got: {sql}"
        );
    }

    #[test]
    fn should_generate_where_lt() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lt("stock", 10_i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM items WHERE stock < 10",
            "less-than condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_lte() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_lte("rank", 100_i32)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM items WHERE rank <= 100",
            "less-than-or-equal condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_like() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_like("email", "%@corp.com")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE email LIKE '%@corp.com'",
            "LIKE condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_is_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_null("deleted_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE deleted_at IS NULL",
            "IS NULL condition was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_where_is_not_null() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .where_is_not_null("confirmed_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users WHERE confirmed_at IS NOT NULL",
            "IS NOT NULL condition was not rendered correctly"
        );
    }

    #[test]
    fn should_join_multiple_conditions_with_and() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_eq("status", "OPEN")
            .where_gt("amount", 50_i32)
            .where_is_null("deleted_at")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT * FROM orders WHERE status = 'OPEN' AND amount > 50 AND deleted_at IS NULL",
            "multiple conditions were not joined with AND"
        );
    }

    // ── ORDER BY ─────────────────────────────────────────────────────────────

    #[test]
    fn should_generate_order_by_asc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("created_at")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users ORDER BY created_at ASC",
            "ORDER BY ASC was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_order_by_desc() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_desc("score")
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM users ORDER BY score DESC",
            "ORDER BY DESC was not rendered correctly"
        );
    }

    #[test]
    fn should_generate_multiple_order_by_clauses() {
        let sql = SelectBuilder::<Dummy>::from("users")
            .order_by_asc("last_name")
            .order_by_desc("score")
            .build_sql();
        assert!(
            sql.contains("last_name ASC, score DESC"),
            "multiple ORDER BY clauses were not rendered correctly; got: {sql}"
        );
    }

    // ── Full query ────────────────────────────────────────────────────────────

    #[test]
    fn should_generate_full_select_with_columns_conditions_and_order() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .columns(&["id", "total", "status"])
            .where_eq("status", "OPEN")
            .where_gt("total", 0_i32)
            .order_by_asc("id")
            .build_sql();
        assert_eq!(
            sql,
            "SELECT id, total, status FROM orders WHERE status = 'OPEN' AND total > 0 ORDER BY id ASC",
            "full SELECT query was not rendered correctly"
        );
    }

    // ── Keyset pagination ─────────────────────────────────────────────────────

    #[test]
    fn should_set_keyset_column_and_key_fn_on_order_by_keyset() {
        let builder = SelectBuilder::<Dummy>::from("users")
            .order_by_keyset("id", |_: &Dummy| "42".to_owned());
        assert_eq!(
            builder.keyset_column.as_deref(),
            Some("id"),
            "keyset_column was not set correctly"
        );
        assert!(
            builder.keyset_key_fn.is_some(),
            "keyset_key_fn should be Some after order_by_keyset"
        );
    }

    #[test]
    fn should_replace_previous_order_by_on_keyset() {
        let sql = SelectBuilder::<Dummy>::from("events")
            .order_by_desc("created_at")
            .order_by_keyset("id", |_: &Dummy| "1".to_owned())
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM events ORDER BY id ASC",
            "previous ORDER BY clauses should be cleared when keyset is configured"
        );
    }

    #[test]
    fn should_generate_order_by_asc_in_sql_for_keyset() {
        let sql = SelectBuilder::<Dummy>::from("events")
            .order_by_keyset("id", |_: &Dummy| "1".to_owned())
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM events ORDER BY id ASC",
            "keyset pagination should produce ORDER BY id ASC"
        );
    }

    #[test]
    fn should_generate_where_gte_for_float() {
        let sql = SelectBuilder::<Dummy>::from("orders")
            .where_gte("score", 4.5_f64)
            .build_sql();
        assert_eq!(
            sql, "SELECT * FROM orders WHERE score >= 4.5",
            "unexpected: {sql}"
        );
    }

    // ── build_sql_no_order ────────────────────────────────────────────────────

    #[test]
    fn should_omit_order_by_in_build_sql_no_order() {
        let sql = SelectBuilder::<Dummy>::from("events")
            .columns(&["id", "name"])
            .order_by_keyset("id", |_: &Dummy| "1".to_owned())
            .build_sql_no_order();
        assert_eq!(
            sql, "SELECT id, name FROM events",
            "build_sql_no_order should not include ORDER BY clause"
        );
    }

    #[test]
    fn should_preserve_where_conditions_in_build_sql_no_order() {
        let sql = SelectBuilder::<Dummy>::from("items")
            .where_eq("active", true)
            .order_by_keyset("id", |_: &Dummy| "1".to_owned())
            .build_sql_no_order();
        assert_eq!(
            sql, "SELECT * FROM items WHERE active = true",
            "build_sql_no_order should keep WHERE conditions but drop ORDER BY"
        );
    }

    #[test]
    fn should_match_build_sql_when_no_order_by_configured() {
        let without_order = SelectBuilder::<Dummy>::from("users")
            .columns(&["id"])
            .where_eq("status", "ACTIVE")
            .build_sql_no_order();
        let with_build_sql = SelectBuilder::<Dummy>::from("users")
            .columns(&["id"])
            .where_eq("status", "ACTIVE")
            .build_sql();
        assert_eq!(
            without_order, with_build_sql,
            "build_sql_no_order should equal build_sql when no ORDER BY is set"
        );
    }
}
