//! Type predicate methods for Value

use super::Value;
use crate::lexer::SchemeNumber;

impl Value {
    /// Check if this value is truthy (everything except #f is truthy in Scheme)
    #[must_use] pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Boolean(false))
    }

    /// Check if this value is a number
    #[must_use] pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Get the number if this is a number
    #[must_use] pub fn as_number(&self) -> Option<&SchemeNumber> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Check if this value is a string
    #[must_use] pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_) | Value::ShortString(_))
    }

    /// Get the string if this is a string
    #[must_use] pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::ShortString(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Check if this value is a symbol
    #[must_use] pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_) | Value::ShortSymbol(_))
    }

    /// Get the symbol if this is a symbol
    #[must_use] pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Value::Symbol(s) => Some(s),
            Value::ShortSymbol(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Check if this value is a character
    #[must_use] pub fn is_character(&self) -> bool {
        matches!(self, Value::Character(_))
    }

    /// Get the character if this is a character
    #[must_use] pub fn as_character(&self) -> Option<char> {
        match self {
            Value::Character(c) => Some(*c),
            _ => None,
        }
    }

    /// Check if this value is a vector
    #[must_use] pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_) | Value::LazyVector(_))
    }

    /// Get the vector if this is a vector (only works for materialized vectors)
    #[must_use] pub fn as_vector(&self) -> Option<&[Value]> {
        match self {
            Value::Vector(v) => Some(v),
            Value::LazyVector(_) => None, // Lazy vectors cannot be directly converted to slice
            _ => None,
        }
    }

    /// Check if this value is null (alias for `is_nil` for compatibility)
    #[must_use] pub fn is_null(&self) -> bool {
        self.is_nil()
    }

    /// Check if this value is a boolean
    #[must_use] pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Get the boolean if this is a boolean
    #[must_use] pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if this value is a hash table
    #[must_use] pub fn is_hash_table(&self) -> bool {
        matches!(self, Value::HashTable(_))
    }

    /// Check if this value is a box
    #[must_use] pub fn is_box(&self) -> bool {
        matches!(self, Value::Box(_))
    }

    /// Check if this value is a comparator
    #[must_use] pub fn is_comparator(&self) -> bool {
        matches!(self, Value::Comparator(_))
    }

    /// Check if this value is a string cursor
    #[must_use] pub fn is_string_cursor(&self) -> bool {
        matches!(self, Value::StringCursor(_))
    }

    /// Check if this value is an ideque
    #[must_use] pub fn is_ideque(&self) -> bool {
        matches!(self, Value::Ideque(_))
    }

    /// Check if this value is a text
    #[must_use] pub fn is_text(&self) -> bool {
        matches!(self, Value::Text(_))
    }

    /// Get the text if this is a text
    #[must_use] pub fn as_text(&self) -> Option<&crate::srfi::srfi_135::Text> {
        match self {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }
}
