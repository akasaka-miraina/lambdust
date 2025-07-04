//! Type conversions for Value

use super::Value;
use crate::lexer::SchemeNumber;

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Number(SchemeNumber::Integer(i))
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::Number(SchemeNumber::Integer(u as i64))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(SchemeNumber::Real(f))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<char> for Value {
    fn from(c: char) -> Self {
        Value::Character(c)
    }
}

impl From<SchemeNumber> for Value {
    fn from(n: SchemeNumber) -> Self {
        Value::Number(n)
    }
}
