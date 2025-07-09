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

impl Value {
    /// Create a new symbol value
    #[cfg(not(feature = "memory-pooling"))]
    pub fn new_symbol(symbol: String) -> Self {
        Value::Symbol(symbol)
    }

    /// Create a new symbol value from string reference
    pub fn new_symbol_ref(symbol: &str) -> Self {
        Value::Symbol(symbol.to_string())
    }

    /// Create a new string value
    pub fn new_string(string: String) -> Self {
        Value::String(string)
    }

    /// Create a new string value from string reference
    pub fn new_string_ref(string: &str) -> Self {
        Value::String(string.to_string())
    }

    /// Create a new character value
    pub fn new_character(character: char) -> Self {
        Value::Character(character)
    }

    /// Create a new boolean value
    #[cfg(not(feature = "memory-pooling"))]
    pub fn new_boolean(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }

    /// Create a new number value
    pub fn new_number(number: SchemeNumber) -> Self {
        Value::Number(number)
    }

    /// Create a new integer value
    #[cfg(not(feature = "memory-pooling"))]
    pub fn new_integer(integer: i64) -> Self {
        Value::Number(SchemeNumber::Integer(integer))
    }

    /// Create a new real value
    pub fn new_real(real: f64) -> Self {
        Value::Number(SchemeNumber::Real(real))
    }

    /// Create the nil value
    pub fn nil() -> Self {
        Value::Nil
    }

    /// Create an undefined value
    pub fn undefined() -> Self {
        Value::Undefined
    }

    /// Create a nil value (alias for nil)
    #[cfg(not(feature = "memory-pooling"))]
    pub fn new_nil() -> Self {
        Value::Nil
    }
}

/// Extension trait to add to_value method to String
pub trait ToValue {
    /// Convert this type to a Scheme Value
    fn to_value(&self) -> Value;
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
