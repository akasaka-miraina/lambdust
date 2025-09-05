//! SRFI-180 JSON ↔ Scheme Value Conversion
//!
//! Bidirectional conversion between JSON values and Scheme values
//! according to SRFI-180 specifications with proper error handling.

use super::error::{JsonError, JsonResult};
use crate::eval::Value;
use crate::ast::Literal;
use crate::utils::intern_symbol;
use std::collections::HashMap;
use std::rc::Rc;

/// Convert JSON value to Scheme Value according to SRFI-180 mappings
///
/// SRFI-180 Mappings:
/// - null → 'null (symbol)
/// - true → #t
/// - false → #f  
/// - Number → Number
/// - String → String
/// - Array → Vector
/// - Object → Association list with symbol keys
pub fn json_to_scheme(json_value: &JsonValue) -> JsonResult<Value> {
    match json_value {
        JsonValue::Null => Ok(Value::Symbol(intern_symbol("null".to_string()))),
        JsonValue::boolean(true) => Ok(Value::Literal(Literal::Boolean(true))),
        JsonValue::boolean(false) => Ok(Value::Literal(Literal::Boolean(false))),
        JsonValue::number(n) => {
            match n {
                JsonNumber::Integer(i) => Ok(Value::Literal(Literal::Number(*i as f64))),
                JsonNumber::Float(f) => Ok(Value::Literal(Literal::Number(*f))),
                JsonNumber::Exact(num, den) => {
                    // Create rational number if possible
                    if *den == 0 {
                        return Err(JsonError::ConversionError {
                            message: "Division by zero in exact rational".to_string(),
                            value: format!("{}/{}", num, den),
                        });
                    }
                    Ok(Value::Literal(Literal::Number(*num as f64 / *den as f64)))
                }
                JsonNumber::Raw(s) => {
                    // Try to parse the raw string
                    if let Ok(i) = s.parse::<i64>() {
                        Ok(Value::Literal(Literal::Number(i as f64)))
                    } else if let Ok(f) = s.parse::<f64>() {
                        Ok(Value::Literal(Literal::Number(f)))
                    } else {
                        Err(JsonError::ConversionError {
                            message: "Invalid number format".to_string(),
                            value: s.clone(),
                        })
                    }
                }
            }
        }
        JsonValue::String(s) => Ok(Value::Literal(Literal::String(Box::new(s.clone())))),
        JsonValue::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(json_to_scheme(item)?);
            }
            Ok(Value::Vector(Rc::new(vec.into())))
        }
        JsonValue::Object(obj) => {
            let mut alist = Vec::new();
            for (key, value) in obj {
                let scheme_key = Value::Symbol(intern_symbol(key.clone()));
                let scheme_value = json_to_scheme(value)?;
                // Create cons pair (key . value)
                let pair = Value::Pair(
                    Box::new(scheme_key),
                    Box::new(scheme_value)
                );
                alist.push(pair);
            }
            // Convert to proper list
            Ok(list_from_vec(alist))
        }
    }
}

/// Convert Scheme Value to JSON value according to SRFI-180 mappings
pub fn scheme_to_json(scheme_value: &Value) -> JsonResult<JsonValue> {
    match scheme_value {
        // SRFI-180: 'null symbol → null
        Value::Symbol(sym) if intern_symbol("null".to_string()) == *sym => {
            Ok(JsonValue::Null)
        }
        Value::Symbol(sym) => {
            // Convert other symbols to strings
            let symbol_name = crate::utils::symbol_name(*sym)
                .unwrap_or_else(|| format!("symbol-{}", sym.0));
            Ok(JsonValue::String(symbol_name))
        }
        Value::Literal(Literal::Boolean(b)) => Ok(JsonValue::boolean(*b)),
        Value::Literal(Literal::Number(n)) => {
            // Try to preserve integer precision if possible
            if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                Ok(JsonValue::number(JsonNumber::Integer(*n as i64)))
            } else {
                Ok(JsonValue::number(JsonNumber::Float(*n)))
            }
        }
        Value::Literal(Literal::String(s)) => Ok(JsonValue::String((**s).clone())),
        Value::Literal(Literal::Character(c)) => {
            // Convert character to single-character string
            Ok(JsonValue::String(c.to_string()))
        }
        Value::Vector(vec) => {
            let mut json_array = Vec::new();
            if let Ok(vec_guard) = vec.try_borrow() {
                for item in vec_guard.iter() {
                    json_array.push(scheme_to_json(item)?);
                }
            }
            Ok(JsonValue::Array(json_array))
        }
        // TODO: Convert association lists to JSON objects
        // This feature is disabled pending proper association list detection
        
        // Convert regular lists to JSON arrays
        value if value.is_list() => {
            let mut json_array = Vec::new();
            let mut current = value;
            
            while let Value::Pair(head_box, tail_box) = current {
                json_array.push(scheme_to_json(head_box.as_ref())?);
                current = tail_box.as_ref();
            }
            Ok(JsonValue::Array(json_array))
        }
        Value::Nil => Ok(JsonValue::Array(Vec::new())), // Empty list → empty array
        _ => Err(JsonError::ConversionError {
            message: "Value type cannot be converted to JSON".to_string(),
            value: format!("{:?}", scheme_value),
        }),
    }
}

/// JSON number representation that preserves exact/inexact distinction
#[derive(Debug, Clone, PartialEq)]
pub enum JsonNumber {
    /// Exact integer
    Integer(i64),
    /// Inexact floating point
    Float(f64),
    /// Exact rational (for completeness)
    Exact(i64, i64),
    /// Raw string representation (for edge cases)
    Raw(String),
}

/// Parse a JSON number string into JsonNumber
pub fn parse_json_number(s: &str) -> JsonResult<JsonNumber> {
    // Try integer first
    if let Ok(i) = s.parse::<i64>() {
        return Ok(JsonNumber::Integer(i));
    }
    
    // Try float
    if let Ok(f) = s.parse::<f64>() {
        return Ok(JsonNumber::Float(f));
    }
    
    // Fallback to raw string
    Ok(JsonNumber::Raw(s.to_string()))
}

/// Internal JSON value representation for processing
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            JsonValue::Null => "null",
            JsonValue::boolean(_) => "boolean",
            JsonValue::number(_) => "number",
            JsonValue::String(_) => "string",
            JsonValue::Array(_) => "array",
            JsonValue::Object(_) => "object",
        }
    }
}

/// Check if a list is an association list (list of pairs)
fn is_association_list(list: &[Value]) -> bool {
    if list.is_empty() {
        return true; // Empty list is valid association list
    }
    
    for item in list {
        if !matches!(item, Value::Pair(_, _)) {
            return false;
        }
    }
    true
}

/// Helper function to extract head and tail from a list
fn list_head_tail(list: &[Value]) -> Option<(&Value, &[Value])> {
    if list.is_empty() {
        None
    } else {
        Some((&list[0], &list[1..]))
    }
}

/// Convert a Vec<Value> to a proper Scheme list
fn list_from_vec(vec: Vec<Value>) -> Value {
    if vec.is_empty() {
        Value::Nil
    } else {
        Value::list(vec)
    }
}

/// Pretty-print JSON value for debugging
impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::boolean(b) => write!(f, "{}", b),
            JsonValue::number(n) => {
                match n {
                    JsonNumber::Integer(i) => write!(f, "{}", i),
                    JsonNumber::Float(fl) => {
                        if fl.fract() == 0.0 && fl.is_finite() {
                            write!(f, "{}", *fl as i64)
                        } else {
                            write!(f, "{}", fl)
                        }
                    }
                    JsonNumber::Exact(num, den) => {
                        if *den == 1 {
                            write!(f, "{}", num)
                        } else {
                            write!(f, "{}", (*num as f64 / *den as f64))
                        }
                    }
                    JsonNumber::Raw(s) => write!(f, "{}", s),
                }
            }
            JsonValue::String(s) => write!(f, "\"{}\"", escape_json_string(s)),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in obj {
                    if !first { write!(f, ", ")?; }
                    first = false;
                    write!(f, "\"{}\": {}", escape_json_string(key), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Escape string for JSON output
fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{0008}' => result.push_str("\\b"), // backspace
            '\u{000C}' => result.push_str("\\f"), // form feed
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => {
                result.push_str(&format!("\\u{:04X}", ch as u32));
            }
            ch => result.push(ch),
        }
    }
    result
}

/// Validation helpers for JSON values
impl JsonValue {
    /// Check if this JSON value is valid (no NaN, Infinity, etc.)
    pub fn is_valid(&self) -> bool {
        match self {
            JsonValue::number(n) => match n {
                JsonNumber::Integer(_) => true,
                JsonNumber::Float(f) => f.is_finite(),
                JsonNumber::Exact(_, den) => *den != 0,
                JsonNumber::Raw(_) => true, // Assume valid if parsed
            },
            JsonValue::Array(arr) => arr.iter().all(|v| v.is_valid()),
            JsonValue::Object(obj) => obj.values().all(|v| v.is_valid()),
            _ => true,
        }
    }

    /// Get the size of this JSON value (for memory limits)
    pub fn size_estimate(&self) -> usize {
        match self {
            JsonValue::Null => 4,
            JsonValue::boolean(_) => 5, // "true" or "false"
            JsonValue::number(n) => match n {
                JsonNumber::Integer(_) => 16,
                JsonNumber::Float(_) => 16,
                JsonNumber::Exact(_, _) => 32, // Two i64s
                JsonNumber::Raw(s) => s.len(),
            },
            JsonValue::String(s) => s.len() + 2, // Plus quotes
            JsonValue::Array(arr) => {
                2 + arr.iter().map(|v| v.size_estimate() + 1).sum::<usize>() // brackets + commas
            }
            JsonValue::Object(obj) => {
                2 + obj.iter().map(|(k, v)| k.len() + 3 + v.size_estimate() + 1).sum::<usize>() // braces + quotes + colons + commas
            }
        }
    }
}

/// Convert multiple JSON values to Scheme values
pub fn json_array_to_scheme(json_array: &[JsonValue]) -> JsonResult<Vec<Value>> {
    let mut result = Vec::new();
    for json_val in json_array {
        result.push(json_to_scheme(json_val)?);
    }
    Ok(result)
}

/// Convert multiple Scheme values to JSON values
pub fn scheme_array_to_json(scheme_array: &[Value]) -> JsonResult<Vec<JsonValue>> {
    let mut result = Vec::new();
    for scheme_val in scheme_array {
        result.push(scheme_to_json(scheme_val)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_scheme_primitives() {
        assert_eq!(json_to_scheme(&JsonValue::Null).unwrap(), 
                   Value::Symbol(intern_symbol("null".to_string())));
        assert_eq!(json_to_scheme(&JsonValue::boolean(true)).unwrap(),
                   Value::Literal(Literal::Boolean(true)));
        assert_eq!(json_to_scheme(&JsonValue::boolean(false)).unwrap(),
                   Value::Literal(Literal::Boolean(false)));
        assert_eq!(json_to_scheme(&JsonValue::number(JsonNumber::Integer(42))).unwrap(),
                   Value::Literal(Literal::Number(42.0)));
        assert_eq!(json_to_scheme(&JsonValue::String("hello".to_string())).unwrap(),
                   Value::Literal(Literal::String(Box::new("hello".to_string()))));
    }

    #[test]
    fn test_scheme_to_json_primitives() {
        assert_eq!(scheme_to_json(&Value::Symbol(intern_symbol("null".to_string()))).unwrap(),
                   JsonValue::Null);
        assert_eq!(scheme_to_json(&Value::Literal(Literal::Boolean(true))).unwrap(),
                   JsonValue::boolean(true));
        assert_eq!(scheme_to_json(&Value::Literal(Literal::Boolean(false))).unwrap(),
                   JsonValue::boolean(false));
        assert_eq!(scheme_to_json(&Value::Literal(Literal::Number(42.0))).unwrap(),
                   JsonValue::number(JsonNumber::Integer(42)));
        assert_eq!(scheme_to_json(&Value::Literal(Literal::String(Box::new("hello".to_string())))).unwrap(),
                   JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_json_string_escaping() {
        let test_cases = vec![
            ("hello", "hello"),
            ("hello\"world", "hello\\\"world"),
            ("hello\nworld", "hello\\nworld"),
            ("hello\tworld", "hello\\tworld"),
            ("hello\\world", "hello\\\\world"),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(escape_json_string(input), expected);
        }
    }
}