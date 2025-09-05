//! Common utilities and definitions for string operations.

use crate::ast::Literal;
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Helper macro to bind a primitive procedure with both normal and builtin: names
macro_rules! bind_primitive {
    ($env:expr, $name:expr, $arity_min:expr, $arity_max:expr, $implementation:expr, $effects:expr) => {
        let proc = Arc::new(PrimitiveProcedure {
            name: $name.to_owned(),
            arity_min: $arity_min,
            arity_max: $arity_max,
            implementation: PrimitiveImpl::RustFn($implementation),
            effects: $effects,
        });
        let name_owned = $name.to_owned();
        $env.define(name_owned.clone(), Value::Primitive(proc.clone()));
        $env.define(format!("builtin:{}", name_owned), Value::Primitive(proc));
    };
}

pub(crate) use bind_primitive;

/// Represents a character set for SRFI-13 operations
#[derive(Clone, Debug)]
pub enum CharacterSet {
    /// A predicate function that tests characters
    Predicate(fn(char) -> bool),
    /// A character literal
    Character(char),
    /// A string containing characters to match
    String(String),
    /// Default whitespace character set
    Whitespace,
}

impl CharacterSet {
    /// Test if a character is in this character set
    pub fn contains(&self, ch: char) -> bool {
        match self {
            CharacterSet::Predicate(f) => f(ch),
            CharacterSet::Character(c) => ch == *c,
            CharacterSet::String(s) => s.contains(ch),
            CharacterSet::Whitespace => ch.is_whitespace(),
        }
    }

    /// Create a character set from a Scheme value
    pub fn from_value(value: &Value) -> Result<CharacterSet> {
        match value {
            Value::Literal(Literal::Character(ch)) => Ok(CharacterSet::Character(*ch)),
            _ => {
                if let Some(s) = value.as_string() {
                    Ok(CharacterSet::String(s.to_string()))
                } else {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "Character set must be a character, string, or predicate".to_string(),
                        None,
                    )))
                }
            }
        }
    }
}

/// Default character sets for common operations
impl Default for CharacterSet {
    fn default() -> Self {
        CharacterSet::Whitespace
    }
}

/// Validate string bounds for operations
pub fn validate_string_bounds(
    s: &str,
    start: usize,
    end: Option<usize>,
    operation: &str,
) -> Result<usize> {
    let len = s.chars().count();

    if start > len {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation}: start index {start} out of bounds for string of length {len}"),
            None,
        )));
    }

    let actual_end = end.unwrap_or(len);
    if actual_end > len {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation}: end index {actual_end} out of bounds for string of length {len}"),
            None,
        )));
    }

    if start > actual_end {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation}: start index {start} greater than end index {actual_end}"),
            None,
        )));
    }

    Ok(actual_end)
}

/// Extract a character from a character position in a string (using char indices)
pub fn extract_char_at_index(s: &str, index: usize) -> Option<char> {
    s.chars().nth(index)
}

/// Get a substring using character indices
pub fn substring_by_chars(s: &str, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end - start).collect()
}

/// Extracts a string from a Value (returns a reference to immutable strings only).
pub fn extract_string<'a>(value: &'a Value, operation: &str) -> Result<&'a str> {
    value.as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))
    })
}

/// Extracts a string from a Value as an owned String (works with both mutable and immutable strings).
pub fn extract_string_owned(value: &Value, operation: &str) -> Result<String> {
    value.as_string_owned().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))
    })
}

/// Extracts a character from a Value.
pub fn extract_character(value: &Value, operation: &str) -> Result<char> {
    match value {
        Value::Literal(Literal::Character(c)) => Ok(*c),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires character arguments"),
            None,
        ))),
    }
}
