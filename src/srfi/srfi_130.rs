//! SRFI 130: Cursor-based String Library
//!
//! This SRFI extends SRFI 13 with string cursor abstractions for high-performance
//! string processing without intermediate string allocation.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// String cursor - an index into a string with optional bounds
#[derive(Debug, Clone)]
pub struct StringCursor {
    /// The string being indexed
    string: String,
    /// Current position in the string (byte index)
    position: usize,
    /// Start bound (byte index)
    start: usize,
    /// End bound (byte index)
    end: usize,
}

impl StringCursor {
    /// Create a new string cursor for the entire string
    pub fn new(string: String) -> Self {
        let end = string.len();
        Self {
            string,
            position: 0,
            start: 0,
            end,
        }
    }

    /// Create a string cursor with specific bounds
    pub fn with_bounds(string: String, start: usize, end: usize) -> Result<Self> {
        if start > end || end > string.len() {
            return Err(LambdustError::runtime_error(
                "Invalid string cursor bounds".to_string(),
            ));
        }
        Ok(Self {
            string,
            position: start,
            start,
            end,
        })
    }

    /// Get the string this cursor references
    pub fn string(&self) -> &str {
        &self.string
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get start bound
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get end bound
    pub fn end(&self) -> usize {
        self.end
    }

    /// Check if at start of bounds
    pub fn at_start(&self) -> bool {
        self.position == self.start
    }

    /// Check if at end of bounds
    pub fn at_end(&self) -> bool {
        self.position >= self.end
    }

    /// Move cursor forward by one character
    pub fn advance(&mut self) -> Result<()> {
        if self.at_end() {
            return Err(LambdustError::runtime_error(
                "Cannot advance cursor past end".to_string(),
            ));
        }

        // Find next character boundary
        while self.position < self.end {
            self.position += 1;
            if self.string.is_char_boundary(self.position) {
                break;
            }
        }
        Ok(())
    }

    /// Move cursor backward by one character
    pub fn retreat(&mut self) -> Result<()> {
        if self.at_start() {
            return Err(LambdustError::runtime_error(
                "Cannot retreat cursor past start".to_string(),
            ));
        }

        // Find previous character boundary
        while self.position > self.start {
            self.position -= 1;
            if self.string.is_char_boundary(self.position) {
                break;
            }
        }
        Ok(())
    }

    /// Get character at current position
    pub fn current_char(&self) -> Result<char> {
        if self.at_end() {
            return Err(LambdustError::runtime_error(
                "Cursor is at end of string".to_string(),
            ));
        }

        self.string[self.position..]
            .chars()
            .next()
            .ok_or_else(|| LambdustError::runtime_error("Invalid character position".to_string()))
    }

    /// Get substring from current position to end of bounds
    pub fn rest(&self) -> &str {
        &self.string[self.position..self.end]
    }

    /// Get substring from start of bounds to current position
    pub fn prefix(&self) -> &str {
        &self.string[self.start..self.position]
    }

    /// Clone cursor
    pub fn clone_cursor(&self) -> Self {
        self.clone()
    }
}

impl PartialEq for StringCursor {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
            && self.position == other.position
            && self.start == other.start
            && self.end == other.end
    }
}

/// SRFI 130 implementation
pub struct Srfi130;

impl super::SrfiModule for Srfi130 {
    fn srfi_id(&self) -> u32 {
        130
    }

    fn name(&self) -> &'static str {
        "Cursor-based String Library"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // String cursor constructors

        // string-cursor-start
        exports.insert(
            "string-cursor-start".to_string(),
            make_builtin_procedure("string-cursor-start", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::String(s) = &args[0] {
                    let cursor = StringCursor::new(s.clone());
                    Ok(Value::StringCursor(Rc::new(cursor)))
                } else {
                    Err(LambdustError::type_error("Expected string".to_string()))
                }
            }),
        );

        // string-cursor-end
        exports.insert(
            "string-cursor-end".to_string(),
            make_builtin_procedure("string-cursor-end", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::String(s) = &args[0] {
                    let mut cursor = StringCursor::new(s.clone());
                    cursor.position = cursor.end;
                    Ok(Value::StringCursor(Rc::new(cursor)))
                } else {
                    Err(LambdustError::type_error("Expected string".to_string()))
                }
            }),
        );

        // string-cursor? predicate
        exports.insert(
            "string-cursor?".to_string(),
            make_builtin_procedure("string-cursor?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::StringCursor(_))))
            }),
        );

        // String cursor navigation

        // string-cursor-next
        exports.insert(
            "string-cursor-next".to_string(),
            make_builtin_procedure("string-cursor-next", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::StringCursor(cursor_rc) = &args[0] {
                    let mut new_cursor = cursor_rc.clone_cursor();
                    new_cursor.advance()?;
                    Ok(Value::StringCursor(Rc::new(new_cursor)))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursor".to_string(),
                    ))
                }
            }),
        );

        // string-cursor-prev
        exports.insert(
            "string-cursor-prev".to_string(),
            make_builtin_procedure("string-cursor-prev", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::StringCursor(cursor_rc) = &args[0] {
                    let mut new_cursor = cursor_rc.clone_cursor();
                    new_cursor.retreat()?;
                    Ok(Value::StringCursor(Rc::new(new_cursor)))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursor".to_string(),
                    ))
                }
            }),
        );

        // String cursor predicates

        // string-cursor=?
        exports.insert(
            "string-cursor=?".to_string(),
            make_builtin_procedure("string-cursor=?", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::StringCursor(c1), Value::StringCursor(c2)) = (&args[0], &args[1]) {
                    Ok(Value::Boolean(
                        c1.position() == c2.position()
                            && c1.string() == c2.string()
                            && c1.start() == c2.start()
                            && c1.end() == c2.end(),
                    ))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursors".to_string(),
                    ))
                }
            }),
        );

        // string-cursor<?
        exports.insert(
            "string-cursor<?".to_string(),
            make_builtin_procedure("string-cursor<?", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::StringCursor(c1), Value::StringCursor(c2)) = (&args[0], &args[1]) {
                    if c1.string() != c2.string() {
                        return Err(LambdustError::runtime_error(
                            "Cursors must reference the same string".to_string(),
                        ));
                    }
                    Ok(Value::Boolean(c1.position() < c2.position()))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursors".to_string(),
                    ))
                }
            }),
        );

        // String cursor accessors

        // string-cursor-ref
        exports.insert(
            "string-cursor-ref".to_string(),
            make_builtin_procedure("string-cursor-ref", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::StringCursor(cursor) = &args[0] {
                    let ch = cursor.current_char()?;
                    Ok(Value::Character(ch))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursor".to_string(),
                    ))
                }
            }),
        );

        // substring/cursors
        exports.insert(
            "substring/cursors".to_string(),
            make_builtin_procedure("substring/cursors", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::StringCursor(start), Value::StringCursor(end)) = (&args[0], &args[1])
                {
                    if start.string() != end.string() {
                        return Err(LambdustError::runtime_error(
                            "Cursors must reference the same string".to_string(),
                        ));
                    }
                    if start.position() > end.position() {
                        return Err(LambdustError::runtime_error(
                            "Start cursor must not be after end cursor".to_string(),
                        ));
                    }
                    let substring = &start.string()[start.position()..end.position()];
                    Ok(Value::String(substring.to_string()))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursors".to_string(),
                    ))
                }
            }),
        );

        // String operations with cursors

        // string-take-cursor
        exports.insert(
            "string-take-cursor".to_string(),
            make_builtin_procedure("string-take-cursor", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::String(s), Value::Number(n)) = (&args[0], &args[1]) {
                    let count = n.to_i64() as usize;
                    let mut result = String::new();
                    let mut chars = s.chars();
                    for _ in 0..count {
                        if let Some(ch) = chars.next() {
                            result.push(ch);
                        } else {
                            break;
                        }
                    }
                    Ok(Value::String(result))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string and number".to_string(),
                    ))
                }
            }),
        );

        // string-drop-cursor
        exports.insert(
            "string-drop-cursor".to_string(),
            make_builtin_procedure("string-drop-cursor", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::String(s), Value::Number(n)) = (&args[0], &args[1]) {
                    let count = n.to_i64() as usize;
                    let result: String = s.chars().skip(count).collect();
                    Ok(Value::String(result))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string and number".to_string(),
                    ))
                }
            }),
        );

        // String searching with cursors

        // string-index-cursor
        exports.insert(
            "string-index-cursor".to_string(),
            make_builtin_procedure("string-index-cursor", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::String(s), Value::Character(target_char)) = (&args[0], &args[1]) {
                    for (i, ch) in s.char_indices() {
                        if ch == *target_char {
                            let mut cursor = StringCursor::new(s.clone());
                            cursor.position = i;
                            return Ok(Value::StringCursor(Rc::new(cursor)));
                        }
                    }
                    Ok(Value::Boolean(false)) // Not found
                } else {
                    Err(LambdustError::type_error(
                        "Expected string and character".to_string(),
                    ))
                }
            }),
        );

        // string-contains-cursor
        exports.insert(
            "string-contains-cursor".to_string(),
            make_builtin_procedure("string-contains-cursor", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::String(haystack), Value::String(needle)) = (&args[0], &args[1]) {
                    if let Some(pos) = haystack.find(needle) {
                        let mut cursor = StringCursor::new(haystack.clone());
                        cursor.position = pos;
                        Ok(Value::StringCursor(Rc::new(cursor)))
                    } else {
                        Ok(Value::Boolean(false)) // Not found
                    }
                } else {
                    Err(LambdustError::type_error("Expected strings".to_string()))
                }
            }),
        );

        // string-length/cursors (enhanced version with cursor support)
        exports.insert(
            "string-length/cursors".to_string(),
            make_builtin_procedure("string-length/cursors", Some(2), |args| {
                check_arity(args, 2)?;
                if let (Value::StringCursor(start), Value::StringCursor(end)) = (&args[0], &args[1])
                {
                    if start.string() != end.string() {
                        return Err(LambdustError::runtime_error(
                            "Cursors must reference the same string".to_string(),
                        ));
                    }
                    let char_count = start.string()[start.position()..end.position()]
                        .chars()
                        .count();
                    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                        char_count as i64,
                    )))
                } else {
                    Err(LambdustError::type_error(
                        "Expected string cursors".to_string(),
                    ))
                }
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 130 has no parts, return all exports
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_string_cursor_creation() {
        let cursor = StringCursor::new("hello".to_string());
        assert_eq!(cursor.string(), "hello");
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.start(), 0);
        assert_eq!(cursor.end(), 5);
        assert!(cursor.at_start());
        assert!(!cursor.at_end());
    }

    #[test]
    fn test_string_cursor_navigation() {
        let mut cursor = StringCursor::new("hello".to_string());

        // Advance through string
        cursor.advance().unwrap();
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.current_char().unwrap(), 'e');

        cursor.advance().unwrap();
        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.current_char().unwrap(), 'l');

        // Retreat
        cursor.retreat().unwrap();
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.current_char().unwrap(), 'e');
    }

    #[test]
    fn test_string_cursor_bounds() {
        let cursor = StringCursor::with_bounds("hello world".to_string(), 2, 7).unwrap();
        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.start(), 2);
        assert_eq!(cursor.end(), 7);
        assert_eq!(cursor.rest(), "llo w");
    }

    #[test]
    fn test_srfi_130_exports() {
        let srfi = Srfi130;
        let exports = srfi.exports();

        assert!(exports.contains_key("string-cursor-start"));
        assert!(exports.contains_key("string-cursor-end"));
        assert!(exports.contains_key("string-cursor?"));
        assert!(exports.contains_key("string-cursor-next"));
        assert!(exports.contains_key("string-cursor-prev"));
        assert!(exports.contains_key("string-cursor=?"));
        assert!(exports.contains_key("string-cursor<?"));
        assert!(exports.contains_key("string-cursor-ref"));
        assert!(exports.contains_key("substring/cursors"));
        assert!(exports.contains_key("string-index-cursor"));
        assert!(exports.contains_key("string-contains-cursor"));
    }

    #[test]
    fn test_unicode_support() {
        let mut cursor = StringCursor::new("こんにちは".to_string());

        // Each Japanese character is 3 bytes in UTF-8
        assert_eq!(cursor.string().len(), 15); // 5 chars * 3 bytes each

        cursor.advance().unwrap();
        assert_eq!(cursor.position(), 3); // First character boundary
        assert_eq!(cursor.current_char().unwrap(), 'ん');

        cursor.advance().unwrap();
        assert_eq!(cursor.position(), 6); // Second character boundary  
        assert_eq!(cursor.current_char().unwrap(), 'に');
    }
}
