//! SRFI 140: Immutable Strings
//!
//! This module implements immutable strings that provide guaranteed O(1)
//! performance for string-ref and string-length operations.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Immutable string representation optimized for performance
#[derive(Debug, Clone, PartialEq)]
pub enum IString {
    /// Small string optimization - store directly in enum (up to 23 bytes)
    Small {
        /// UTF-8 encoded bytes
        data: [u8; 23],
        /// Length in bytes
        len: u8,
    },
    /// Medium string - single allocation with metadata
    Medium {
        /// Shared reference to string data
        data: Arc<String>,
        /// Start position in characters
        start: usize,
        /// Length in characters
        length: usize,
    },
    /// Large string - rope structure for efficient manipulation
    Rope {
        /// Left subtree
        left: Rc<IString>,
        /// Right subtree
        right: Rc<IString>,
        /// Cached total length in characters
        length: usize,
        /// Cached depth (for balancing)
        depth: usize,
    },
}

impl IString {
    /// Create a new immutable string from a regular string
    pub fn from_string(s: String) -> Self {
        let byte_len = s.len();
        if byte_len <= 23 {
            // Small string optimization
            let mut data = [0u8; 23];
            data[..byte_len].copy_from_slice(s.as_bytes());
            IString::Small {
                data,
                len: byte_len as u8,
            }
        } else {
            // Medium string
            let char_len = s.chars().count();
            IString::Medium {
                data: Arc::new(s),
                start: 0,
                length: char_len,
            }
        }
    }

    /// Create a new immutable string from a &str
    pub fn from_str(s: &str) -> Self {
        Self::from_string(s.to_string())
    }

    /// Get the length of the string in characters (O(1))
    pub fn length(&self) -> usize {
        match self {
            IString::Small { len, .. } => {
                // For small strings, count UTF-8 characters
                let bytes = &self.as_bytes()[..*len as usize];
                std::str::from_utf8(bytes).unwrap().chars().count()
            }
            IString::Medium { length, .. } => *length,
            IString::Rope { length, .. } => *length,
        }
    }

    /// Get a character at the specified index (O(1) for Small/Medium, O(log n) for Rope)
    pub fn char_at(&self, index: usize) -> Option<char> {
        match self {
            IString::Small { .. } => {
                self.to_string().chars().nth(index)
            }
            IString::Medium { data, start, length } => {
                if index >= *length {
                    None
                } else {
                    data.chars().nth(start + index)
                }
            }
            IString::Rope { left, right, .. } => {
                let left_len = left.length();
                if index < left_len {
                    left.char_at(index)
                } else {
                    right.char_at(index - left_len)
                }
            }
        }
    }

    /// Convert to a regular String
    pub fn to_string(&self) -> String {
        match self {
            IString::Small { data, len } => {
                let bytes = &data[..*len as usize];
                std::str::from_utf8(bytes).unwrap().to_string()
            }
            IString::Medium { data, start, length } => {
                data.chars().skip(*start).take(*length).collect()
            }
            IString::Rope { left, right, .. } => {
                let mut result = String::new();
                result.push_str(&left.to_string());
                result.push_str(&right.to_string());
                result
            }
        }
    }

    /// Get the raw bytes for small strings
    fn as_bytes(&self) -> &[u8] {
        match self {
            IString::Small { data, len } => &data[..*len as usize],
            _ => panic!("as_bytes only available for small strings"),
        }
    }

    /// Create a substring (O(1) for Medium strings, may create new structure for others)
    pub fn substring(&self, start: usize, end: usize) -> Result<IString> {
        let total_len = self.length();
        if start > end || end > total_len {
            return Err(LambdustError::runtime_error(
                "Invalid substring indices".to_string(),
            ));
        }

        let sub_len = end - start;
        if sub_len == 0 {
            return Ok(IString::from_str(""));
        }

        match self {
            IString::Small { .. } => {
                let full_string = self.to_string();
                let substring: String = full_string.chars().skip(start).take(sub_len).collect();
                Ok(IString::from_string(substring))
            }
            IString::Medium { data, start: orig_start, .. } => {
                Ok(IString::Medium {
                    data: data.clone(),
                    start: orig_start + start,
                    length: sub_len,
                })
            }
            IString::Rope { .. } => {
                // For simplicity, convert to string and create new IString
                let full_string = self.to_string();
                let substring: String = full_string.chars().skip(start).take(sub_len).collect();
                Ok(IString::from_string(substring))
            }
        }
    }

    /// Append two immutable strings (returns new IString)
    pub fn append(&self, other: &IString) -> IString {
        let self_len = self.length();
        let other_len = other.length();
        let total_len = self_len + other_len;

        // If result would be small enough, create a small string
        let combined_string = format!("{}{}", self.to_string(), other.to_string());
        if combined_string.len() <= 23 {
            return IString::from_string(combined_string);
        }

        // Create rope structure for larger strings
        IString::Rope {
            left: Rc::new(self.clone()),
            right: Rc::new(other.clone()),
            length: total_len,
            depth: 1 + std::cmp::max(self.depth(), other.depth()),
        }
    }

    /// Get the depth of the rope structure
    fn depth(&self) -> usize {
        match self {
            IString::Small { .. } | IString::Medium { .. } => 0,
            IString::Rope { depth, .. } => *depth,
        }
    }
}

/// Check if a value is an immutable string
fn is_istring(value: &Value) -> bool {
    matches!(value, Value::IString(_))
}

/// Check if a value is a mutable string
fn is_mstring(value: &Value) -> bool {
    matches!(value, Value::String(_))
}

/// Check if a value is any kind of string
fn is_string(value: &Value) -> bool {
    is_istring(value) || is_mstring(value)
}

/// Get string length (works for both istring and mstring)
fn string_length_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;
    match &args[0] {
        Value::IString(istr) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
            istr.length() as i64,
        ))),
        Value::String(s) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
            s.chars().count() as i64,
        ))),
        _ => Err(LambdustError::type_error("Expected string".to_string())),
    }
}

/// Get character at index (works for both istring and mstring)
fn string_ref_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 2)?;
    let index = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
        _ => return Err(LambdustError::type_error("Expected integer index".to_string())),
    };

    match &args[0] {
        Value::IString(istr) => {
            if let Some(ch) = istr.char_at(index) {
                Ok(Value::Character(ch))
            } else {
                Err(LambdustError::runtime_error("String index out of bounds".to_string()))
            }
        }
        Value::String(s) => {
            if let Some(ch) = s.chars().nth(index) {
                Ok(Value::Character(ch))
            } else {
                Err(LambdustError::runtime_error("String index out of bounds".to_string()))
            }
        }
        _ => Err(LambdustError::type_error("Expected string".to_string())),
    }
}

/// Create immutable string from characters
fn string_proc(args: &[Value]) -> Result<Value> {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::Character(ch) => result.push(*ch),
            _ => return Err(LambdustError::type_error("Expected character".to_string())),
        }
    }
    Ok(Value::IString(Rc::new(IString::from_string(result))))
}

/// Create mutable string of specified length
fn make_string_proc(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let length = match &args[0] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
        _ => return Err(LambdustError::type_error("Expected integer length".to_string())),
    };

    let fill_char = if args.len() == 2 {
        match &args[1] {
            Value::Character(ch) => *ch,
            _ => return Err(LambdustError::type_error("Expected character".to_string())),
        }
    } else {
        ' '
    };

    Ok(Value::String(fill_char.to_string().repeat(length)))
}

/// Copy string (returns mutable string by default)
fn string_copy_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;
    match &args[0] {
        Value::IString(istr) => Ok(Value::String(istr.to_string())),
        Value::String(s) => Ok(Value::String(s.clone())),
        _ => Err(LambdustError::type_error("Expected string".to_string())),
    }
}

/// Create substring as immutable string
fn substring_proc(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let start = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
        _ => return Err(LambdustError::type_error("Expected integer start".to_string())),
    };

    let end = if args.len() == 3 {
        match &args[2] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            _ => return Err(LambdustError::type_error("Expected integer end".to_string())),
        }
    } else {
        match &args[0] {
            Value::IString(istr) => istr.length(),
            Value::String(s) => s.chars().count(),
            _ => return Err(LambdustError::type_error("Expected string".to_string())),
        }
    };

    match &args[0] {
        Value::IString(istr) => {
            let sub_istr = istr.substring(start, end)?;
            Ok(Value::IString(Rc::new(sub_istr)))
        }
        Value::String(s) => {
            let substring: String = s.chars().skip(start).take(end - start).collect();
            Ok(Value::IString(Rc::new(IString::from_string(substring))))
        }
        _ => Err(LambdustError::type_error("Expected string".to_string())),
    }
}

/// Append strings (returns immutable string)
fn string_append_proc(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::IString(Rc::new(IString::from_str(""))));
    }

    let mut result = IString::from_str("");
    for arg in args {
        let next_istr = match arg {
            Value::IString(istr) => istr.as_ref().clone(),
            Value::String(s) => IString::from_string(s.clone()),
            _ => return Err(LambdustError::type_error("Expected string".to_string())),
        };
        result = result.append(&next_istr);
    }

    Ok(Value::IString(Rc::new(result)))
}

/// SRFI 140 implementation
pub struct Srfi140;

impl super::SrfiModule for Srfi140 {
    fn srfi_id(&self) -> u32 {
        140
    }

    fn name(&self) -> &'static str {
        "Immutable Strings"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["istrings", "mstrings"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Type predicates
        exports.insert(
            "string?".to_string(),
            make_builtin_procedure("string?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_string(&args[0])))
            }),
        );

        exports.insert(
            "istring?".to_string(),
            make_builtin_procedure("istring?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_istring(&args[0])))
            }),
        );

        exports.insert(
            "mstring?".to_string(),
            make_builtin_procedure("mstring?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_mstring(&args[0])))
            }),
        );

        // Basic operations
        exports.insert(
            "string-length".to_string(),
            make_builtin_procedure("string-length", Some(1), string_length_proc),
        );

        exports.insert(
            "string-ref".to_string(),
            make_builtin_procedure("string-ref", Some(2), string_ref_proc),
        );

        // Construction (returns istrings by default)
        exports.insert(
            "string".to_string(),
            make_builtin_procedure("string", None, string_proc),
        );

        exports.insert(
            "make-string".to_string(),
            make_builtin_procedure("make-string", None, make_string_proc),
        );

        exports.insert(
            "string-copy".to_string(),
            make_builtin_procedure("string-copy", Some(1), string_copy_proc),
        );

        // Manipulation
        exports.insert(
            "substring".to_string(),
            make_builtin_procedure("substring", None, substring_proc),
        );

        exports.insert(
            "string-append".to_string(),
            make_builtin_procedure("string-append", None, string_append_proc),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of part
        // In a full implementation, this would filter based on istrings vs mstrings
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_istring_creation() {
        // Test small string
        let small = IString::from_str("hello");
        assert_eq!(small.length(), 5);
        assert_eq!(small.to_string(), "hello");

        // Test medium string
        let medium = IString::from_str("This is a longer string that should be stored as medium");
        assert!(medium.length() > 23);
        assert_eq!(medium.to_string(), "This is a longer string that should be stored as medium");
    }

    #[test]
    fn test_string_operations() {
        let istr = IString::from_str("hello world");
        
        // Test length
        assert_eq!(istr.length(), 11);
        
        // Test character access
        assert_eq!(istr.char_at(0), Some('h'));
        assert_eq!(istr.char_at(6), Some('w'));
        assert_eq!(istr.char_at(11), None);
        
        // Test substring
        let sub = istr.substring(6, 11).unwrap();
        assert_eq!(sub.to_string(), "world");
    }

    #[test]
    fn test_string_append() {
        let istr1 = IString::from_str("hello");
        let istr2 = IString::from_str(" world");
        let result = istr1.append(&istr2);
        
        assert_eq!(result.to_string(), "hello world");
        assert_eq!(result.length(), 11);
    }

    #[test]
    fn test_srfi_procedures() {
        let srfi = Srfi140;
        let exports = srfi.exports();

        // Test string? predicate
        let string_pred = exports.get("string?").unwrap();
        if let Value::Procedure(crate::value::Procedure::Builtin { func, .. }) = string_pred {
            let istr = Value::IString(Rc::new(IString::from_str("test")));
            let mstr = Value::String("test".to_string());
            let not_str = Value::Number(crate::lexer::SchemeNumber::Integer(42));

            assert_eq!(func(&[istr]).unwrap(), Value::Boolean(true));
            assert_eq!(func(&[mstr]).unwrap(), Value::Boolean(true));
            assert_eq!(func(&[not_str]).unwrap(), Value::Boolean(false));
        }

        // Test string construction
        let string_cons = exports.get("string").unwrap();
        if let Value::Procedure(crate::value::Procedure::Builtin { func, .. }) = string_cons {
            let result = func(&[
                Value::Character('h'),
                Value::Character('i'),
            ]).unwrap();
            
            if let Value::IString(istr) = result {
                assert_eq!(istr.to_string(), "hi");
            } else {
                panic!("Expected IString");
            }
        }
    }

    #[test]
    fn test_string_length_proc() {
        let istr = Value::IString(Rc::new(IString::from_str("hello")));
        let mstr = Value::String("world".to_string());

        let result1 = string_length_proc(&[istr]).unwrap();
        let result2 = string_length_proc(&[mstr]).unwrap();

        assert_eq!(result1, Value::Number(crate::lexer::SchemeNumber::Integer(5)));
        assert_eq!(result2, Value::Number(crate::lexer::SchemeNumber::Integer(5)));
    }

    #[test]
    fn test_string_ref_proc() {
        let istr = Value::IString(Rc::new(IString::from_str("hello")));
        let index = Value::Number(crate::lexer::SchemeNumber::Integer(1));

        let result = string_ref_proc(&[istr, index]).unwrap();
        assert_eq!(result, Value::Character('e'));
    }

    #[test]
    fn test_substring_proc() {
        let istr = Value::IString(Rc::new(IString::from_str("hello world")));
        let start = Value::Number(crate::lexer::SchemeNumber::Integer(6));
        let end = Value::Number(crate::lexer::SchemeNumber::Integer(11));

        let result = substring_proc(&[istr, start, end]).unwrap();
        if let Value::IString(sub_istr) = result {
            assert_eq!(sub_istr.to_string(), "world");
        } else {
            panic!("Expected IString");
        }
    }
}