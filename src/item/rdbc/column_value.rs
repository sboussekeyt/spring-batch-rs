//! Type-erased column value for RDBC item writers.

/// A type-erased value that can be bound to a database column.
///
/// Used by RDBC item writers to carry field values from item extractor closures
/// to database parameter binding at write time. This enum allows storing values
/// of different types in a single collection without requiring static type dispatch.
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::rdbc::ColumnValue;
///
/// let v: ColumnValue = 42i32.into();
/// assert!(matches!(v, ColumnValue::Int(42)));
///
/// let v: ColumnValue = None::<i32>.into();
/// assert!(matches!(v, ColumnValue::Null));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnValue {
    /// Signed integer (covers i32, i64).
    Int(i64),
    /// Floating-point number (covers f32, f64).
    Float(f64),
    /// UTF-8 text (covers &str, String).
    Text(String),
    /// Boolean value.
    Bool(bool),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// SQL NULL — produced by Option::None.
    Null,
}

impl From<i32> for ColumnValue {
    fn from(v: i32) -> Self {
        ColumnValue::Int(v as i64)
    }
}

impl From<i64> for ColumnValue {
    fn from(v: i64) -> Self {
        ColumnValue::Int(v)
    }
}

impl From<f32> for ColumnValue {
    fn from(v: f32) -> Self {
        ColumnValue::Float(v as f64)
    }
}

impl From<f64> for ColumnValue {
    fn from(v: f64) -> Self {
        ColumnValue::Float(v)
    }
}

impl From<bool> for ColumnValue {
    fn from(v: bool) -> Self {
        ColumnValue::Bool(v)
    }
}

impl From<&str> for ColumnValue {
    fn from(v: &str) -> Self {
        ColumnValue::Text(v.to_string())
    }
}

impl From<String> for ColumnValue {
    fn from(v: String) -> Self {
        ColumnValue::Text(v)
    }
}

impl From<Vec<u8>> for ColumnValue {
    fn from(v: Vec<u8>) -> Self {
        ColumnValue::Bytes(v)
    }
}

impl<T: Into<ColumnValue>> From<Option<T>> for ColumnValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => inner.into(),
            None => ColumnValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_i32_to_int() {
        assert!(matches!(ColumnValue::from(42i32), ColumnValue::Int(42)));
    }

    #[test]
    fn should_convert_i64_to_int() {
        assert!(matches!(ColumnValue::from(100i64), ColumnValue::Int(100)));
    }

    #[test]
    fn should_convert_f32_to_float() {
        let v = ColumnValue::from(1.5f32);
        assert!(matches!(v, ColumnValue::Float(_)));
        if let ColumnValue::Float(f) = v {
            assert!((f - 1.5f64).abs() < 1e-5, "f32 should be widened to f64");
        }
    }

    #[test]
    fn should_convert_f64_to_float() {
        assert!(matches!(ColumnValue::from(3.14f64), ColumnValue::Float(_)));
    }

    #[test]
    fn should_convert_bool_to_bool() {
        assert!(matches!(ColumnValue::from(true), ColumnValue::Bool(true)));
        assert!(matches!(ColumnValue::from(false), ColumnValue::Bool(false)));
    }

    #[test]
    fn should_convert_str_to_text() {
        assert!(matches!(ColumnValue::from("hello"), ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_string_to_text() {
        assert!(matches!(
            ColumnValue::from("world".to_string()),
            ColumnValue::Text(_)
        ));
    }

    #[test]
    fn should_convert_bytes_to_bytes() {
        let v = ColumnValue::from(vec![1u8, 2, 3]);
        assert!(matches!(v, ColumnValue::Bytes(_)));
    }

    #[test]
    fn should_convert_some_i32_to_int() {
        assert!(matches!(
            ColumnValue::from(Some(7i32)),
            ColumnValue::Int(7)
        ));
    }

    #[test]
    fn should_convert_none_i32_to_null() {
        assert!(matches!(ColumnValue::from(None::<i32>), ColumnValue::Null));
    }

    #[test]
    fn should_convert_some_string_to_text() {
        let v = ColumnValue::from(Some("abc".to_string()));
        assert!(matches!(v, ColumnValue::Text(_)));
    }

    #[test]
    fn should_convert_none_string_to_null() {
        assert!(matches!(
            ColumnValue::from(None::<String>),
            ColumnValue::Null
        ));
    }
}
