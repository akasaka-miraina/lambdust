//! SRFI 135: Immutable Texts
//!
//! This SRFI defines a new data type of immutable texts that are similar to strings
//! but cannot be mutated. Immutability enables space-efficient representations
//! and efficient sharing of substructures.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Immutable text data structure using rope-like implementation for efficiency
#[derive(Debug, Clone, PartialEq)]
pub enum Text {
    /// Leaf node containing actual character data
    Leaf {
        /// UTF-8 encoded string data
        data: Arc<String>,
        /// Start index in characters (not bytes)
        start: usize,
        /// Length in characters
        length: usize,
    },
    /// Concatenation node for efficient append operations
    Concat {
        /// Left subtree
        left: Rc<Text>,
        /// Right subtree
        right: Rc<Text>,
        /// Cached total length in characters
        length: usize,
    },
}

impl Text {
    /// Create a new text from a string
    #[must_use] pub fn from_string(s: String) -> Self {
        let length = s.chars().count();
        Text::Leaf {
            data: Arc::new(s),
            start: 0,
            length,
        }
    }

    /// Create an empty text
    #[must_use] pub fn empty() -> Self {
        Text::Leaf {
            data: Arc::new(String::new()),
            start: 0,
            length: 0,
        }
    }

    /// Get the length of the text in characters
    #[must_use] pub fn length(&self) -> usize {
        match self {
            Text::Leaf { length, .. } => *length,
            Text::Concat { length, .. } => *length,
        }
    }

    /// Check if the text is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Get character at given index (O(log n) for rope structure)
    pub fn char_at(&self, index: usize) -> Result<char> {
        if index >= self.length() {
            return Err(LambdustError::runtime_error(
                "Text index out of bounds".to_string(),
            ));
        }

        self.char_at_internal(index)
    }

    fn char_at_internal(&self, index: usize) -> Result<char> {
        match self {
            Text::Leaf { data, start, .. } => {
                let chars: Vec<char> = data.chars().skip(*start).collect();
                if index < chars.len() {
                    Ok(chars[index])
                } else {
                    Err(LambdustError::runtime_error(
                        "Character index out of bounds".to_string(),
                    ))
                }
            }
            Text::Concat { left, right, .. } => {
                let left_length = left.length();
                if index < left_length {
                    left.char_at_internal(index)
                } else {
                    right.char_at_internal(index - left_length)
                }
            }
        }
    }

    /// Create a substring (subtext) from start to end indices
    pub fn subtext(&self, start: usize, end: usize) -> Result<Text> {
        if start > end || end > self.length() {
            return Err(LambdustError::runtime_error(
                "Invalid subtext bounds".to_string(),
            ));
        }

        if start == end {
            return Ok(Text::empty());
        }

        if start == 0 && end == self.length() {
            return Ok(self.clone());
        }

        self.subtext_internal(start, end - start)
    }

    fn subtext_internal(&self, start: usize, length: usize) -> Result<Text> {
        match self {
            Text::Leaf {
                data,
                start: leaf_start,
                ..
            } => Ok(Text::Leaf {
                data: data.clone(),
                start: leaf_start + start,
                length,
            }),
            Text::Concat { left, right, .. } => {
                let left_length = left.length();
                if start + length <= left_length {
                    // Entirely in left subtree
                    left.subtext_internal(start, length)
                } else if start >= left_length {
                    // Entirely in right subtree
                    right.subtext_internal(start - left_length, length)
                } else {
                    // Spans both subtrees
                    let left_part = left.subtext_internal(start, left_length - start)?;
                    let right_part = right.subtext_internal(0, length - (left_length - start))?;
                    Ok(Text::concat(left_part, right_part))
                }
            }
        }
    }

    /// Concatenate two texts efficiently
    #[must_use] pub fn concat(left: Text, right: Text) -> Text {
        if left.is_empty() {
            return right;
        }
        if right.is_empty() {
            return left;
        }

        let length = left.length() + right.length();
        Text::Concat {
            left: Rc::new(left),
            right: Rc::new(right),
            length,
        }
    }

    /// Convert text to a regular string
    #[must_use] pub fn text_to_string(&self) -> String {
        match self {
            Text::Leaf {
                data,
                start,
                length,
            } => {
                let chars: Vec<char> = data.chars().skip(*start).take(*length).collect();
                chars.into_iter().collect()
            }
            Text::Concat { left, right, .. } => {
                let mut result = left.text_to_string();
                result.push_str(&right.text_to_string());
                result
            }
        }
    }

    /// Take first n characters
    #[must_use] pub fn take(&self, n: usize) -> Text {
        if n == 0 {
            Text::empty()
        } else if n >= self.length() {
            self.clone()
        } else {
            self.subtext(0, n).unwrap_or_else(|_| Text::empty())
        }
    }

    /// Drop first n characters
    #[must_use] pub fn drop(&self, n: usize) -> Text {
        if n == 0 {
            self.clone()
        } else if n >= self.length() {
            Text::empty()
        } else {
            self.subtext(n, self.length())
                .unwrap_or_else(|_| Text::empty())
        }
    }

    /// Check if this text equals another text
    #[must_use] pub fn text_equal(&self, other: &Text) -> bool {
        if self.length() != other.length() {
            return false;
        }

        // Fast path for identical references
        if std::ptr::eq(self, other) {
            return true;
        }

        // Compare character by character
        for i in 0..self.length() {
            if let (Ok(c1), Ok(c2)) = (self.char_at(i), other.char_at(i)) {
                if c1 != c2 {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::empty()
    }
}

/// Check if a value is textual (either text or string)
#[must_use] pub fn is_textual(value: &Value) -> bool {
    matches!(value, Value::Text(_) | Value::String(_))
}

/// Convert a textual value to text
pub fn textual_to_text(value: &Value) -> Result<Text> {
    match value {
        Value::Text(text) => Ok((**text).clone()),
        Value::String(s) => Ok(Text::from_string(s.clone())),
        _ => Err(LambdustError::type_error(
            "Expected textual value".to_string(),
        )),
    }
}

/// Convert a textual value to string
pub fn textual_to_string(value: &Value) -> Result<String> {
    match value {
        Value::Text(text) => Ok(text.text_to_string()),
        Value::String(s) => Ok(s.clone()),
        _ => Err(LambdustError::type_error(
            "Expected textual value".to_string(),
        )),
    }
}

/// SRFI 135 implementation
pub struct Srfi135;

impl super::SrfiModule for Srfi135 {
    fn srfi_id(&self) -> u32 {
        135
    }

    fn name(&self) -> &'static str {
        "Immutable Texts"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Predicates
        exports.insert(
            "text?".to_string(),
            make_builtin_procedure("text?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Text(_))))
            }),
        );

        exports.insert(
            "textual?".to_string(),
            make_builtin_procedure("textual?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_textual(&args[0])))
            }),
        );

        exports.insert(
            "textual-null?".to_string(),
            make_builtin_procedure("textual-null?", Some(1), |args| {
                check_arity(args, 1)?;
                match &args[0] {
                    Value::Text(text) => Ok(Value::Boolean(text.is_empty())),
                    Value::String(s) => Ok(Value::Boolean(s.is_empty())),
                    _ => Err(LambdustError::type_error(
                        "Expected textual value".to_string(),
                    )),
                }
            }),
        );

        // Constructors
        exports.insert(
            "make-text".to_string(),
            make_builtin_procedure("make-text", Some(2), |args| {
                check_arity(args, 2)?;
                let len = args[0].as_number().ok_or_else(|| {
                    LambdustError::type_error("Expected number for length".to_string())
                })?;
                let ch = match &args[1] {
                    Value::Character(c) => *c,
                    _ => {
                        return Err(LambdustError::type_error("Expected character".to_string()));
                    }
                };

                let length = match len {
                    crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                    _ => {
                        return Err(LambdustError::type_error(
                            "Expected non-negative integer".to_string(),
                        ));
                    }
                };

                let s = ch.to_string().repeat(length);
                let text = Text::from_string(s);
                Ok(Value::Text(Rc::new(text)))
            }),
        );

        exports.insert(
            "text".to_string(),
            make_builtin_procedure("text", None, |args| {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Value::Character(c) => result.push(*c),
                        _ => {
                            return Err(LambdustError::type_error(
                                "Expected character".to_string(),
                            ));
                        }
                    }
                }
                let text = Text::from_string(result);
                Ok(Value::Text(Rc::new(text)))
            }),
        );

        // Conversion procedures
        exports.insert(
            "textual->text".to_string(),
            make_builtin_procedure("textual->text", Some(1), |args| {
                check_arity(args, 1)?;
                let text = textual_to_text(&args[0])?;
                Ok(Value::Text(Rc::new(text)))
            }),
        );

        exports.insert(
            "textual->string".to_string(),
            make_builtin_procedure("textual->string", Some(1), |args| {
                check_arity(args, 1)?;
                let s = textual_to_string(&args[0])?;
                Ok(Value::String(s))
            }),
        );

        exports.insert(
            "string->text".to_string(),
            make_builtin_procedure("string->text", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::String(s) = &args[0] {
                    let text = Text::from_string(s.clone());
                    Ok(Value::Text(Rc::new(text)))
                } else {
                    Err(LambdustError::type_error("Expected string".to_string()))
                }
            }),
        );

        // Selection procedures
        exports.insert(
            "text-length".to_string(),
            make_builtin_procedure("text-length", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Text(text) = &args[0] {
                    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                        text.length() as i64,
                    )))
                } else {
                    Err(LambdustError::type_error("Expected text".to_string()))
                }
            }),
        );

        exports.insert(
            "textual-length".to_string(),
            make_builtin_procedure("textual-length", Some(1), |args| {
                check_arity(args, 1)?;
                let length = match &args[0] {
                    Value::Text(text) => text.length(),
                    Value::String(s) => s.chars().count(),
                    _ => {
                        return Err(LambdustError::type_error(
                            "Expected textual value".to_string(),
                        ));
                    }
                };
                Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                    length as i64,
                )))
            }),
        );

        exports.insert(
            "text-ref".to_string(),
            make_builtin_procedure("text-ref", Some(2), |args| {
                check_arity(args, 2)?;
                if let Value::Text(text) = &args[0] {
                    let index = args[1].as_number().ok_or_else(|| {
                        LambdustError::type_error("Expected number for index".to_string())
                    })?;
                    let idx = match index {
                        crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                        _ => {
                            return Err(LambdustError::type_error(
                                "Expected non-negative integer".to_string(),
                            ));
                        }
                    };
                    let ch = text.char_at(idx)?;
                    Ok(Value::Character(ch))
                } else {
                    Err(LambdustError::type_error("Expected text".to_string()))
                }
            }),
        );

        exports.insert(
            "subtext".to_string(),
            make_builtin_procedure("subtext", Some(3), |args| {
                check_arity(args, 3)?;
                if let Value::Text(text) = &args[0] {
                    let start = args[1].as_number().ok_or_else(|| {
                        LambdustError::type_error("Expected number for start".to_string())
                    })?;
                    let end = args[2].as_number().ok_or_else(|| {
                        LambdustError::type_error("Expected number for end".to_string())
                    })?;

                    let start_idx = match start {
                        crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                        _ => {
                            return Err(LambdustError::type_error(
                                "Expected non-negative integer".to_string(),
                            ));
                        }
                    };
                    let end_idx = match end {
                        crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                        _ => {
                            return Err(LambdustError::type_error(
                                "Expected non-negative integer".to_string(),
                            ));
                        }
                    };

                    let subtext = text.subtext(start_idx, end_idx)?;
                    Ok(Value::Text(Rc::new(subtext)))
                } else {
                    Err(LambdustError::type_error("Expected text".to_string()))
                }
            }),
        );

        // Manipulation procedures
        exports.insert(
            "textual-take".to_string(),
            make_builtin_procedure("textual-take", Some(2), |args| {
                check_arity(args, 2)?;
                let text = textual_to_text(&args[0])?;
                let n = args[1]
                    .as_number()
                    .ok_or_else(|| LambdustError::type_error("Expected number".to_string()))?;
                let count = match n {
                    crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                    _ => {
                        return Err(LambdustError::type_error(
                            "Expected non-negative integer".to_string(),
                        ));
                    }
                };
                let result = text.take(count);
                Ok(Value::Text(Rc::new(result)))
            }),
        );

        exports.insert(
            "textual-drop".to_string(),
            make_builtin_procedure("textual-drop", Some(2), |args| {
                check_arity(args, 2)?;
                let text = textual_to_text(&args[0])?;
                let n = args[1]
                    .as_number()
                    .ok_or_else(|| LambdustError::type_error("Expected number".to_string()))?;
                let count = match n {
                    crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as usize,
                    _ => {
                        return Err(LambdustError::type_error(
                            "Expected non-negative integer".to_string(),
                        ));
                    }
                };
                let result = text.drop(count);
                Ok(Value::Text(Rc::new(result)))
            }),
        );

        // Comparison procedures
        exports.insert(
            "textual=?".to_string(),
            make_builtin_procedure("textual=?", None, |args| {
                if args.is_empty() {
                    return Ok(Value::Boolean(true));
                }

                let first_text = textual_to_text(&args[0])?;
                for arg in args.iter().skip(1) {
                    let other_text = textual_to_text(arg)?;
                    if !first_text.text_equal(&other_text) {
                        return Ok(Value::Boolean(false));
                    }
                }
                Ok(Value::Boolean(true))
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 135 has no parts
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_text_creation() {
        let text = Text::from_string("hello".to_string());
        assert_eq!(text.length(), 5);
        assert!(!text.is_empty());

        let empty_text = Text::empty();
        assert_eq!(empty_text.length(), 0);
        assert!(empty_text.is_empty());
    }

    #[test]
    fn test_text_char_access() {
        let text = Text::from_string("hello".to_string());

        assert_eq!(text.char_at(0).unwrap(), 'h');
        assert_eq!(text.char_at(1).unwrap(), 'e');
        assert_eq!(text.char_at(4).unwrap(), 'o');

        assert!(text.char_at(5).is_err()); // Out of bounds
    }

    #[test]
    fn test_text_subtext() {
        let text = Text::from_string("hello world".to_string());

        let sub = text.subtext(0, 5).unwrap();
        assert_eq!(sub.text_to_string(), "hello");

        let sub2 = text.subtext(6, 11).unwrap();
        assert_eq!(sub2.text_to_string(), "world");

        // Empty subtext
        let empty_sub = text.subtext(3, 3).unwrap();
        assert!(empty_sub.is_empty());

        // Full text
        let full = text.subtext(0, text.length()).unwrap();
        assert_eq!(full.text_to_string(), "hello world");
    }

    #[test]
    fn test_text_concat() {
        let left = Text::from_string("hello".to_string());
        let right = Text::from_string(" world".to_string());

        let combined = Text::concat(left, right);
        assert_eq!(combined.length(), 11);
        assert_eq!(combined.text_to_string(), "hello world");
    }

    #[test]
    fn test_text_take_drop() {
        let text = Text::from_string("hello world".to_string());

        let taken = text.take(5);
        assert_eq!(taken.text_to_string(), "hello");

        let dropped = text.drop(6);
        assert_eq!(dropped.text_to_string(), "world");

        // Edge cases
        let take_all = text.take(100);
        assert_eq!(take_all.text_to_string(), "hello world");

        let drop_all = text.drop(100);
        assert!(drop_all.is_empty());
    }

    #[test]
    fn test_text_equality() {
        let text1 = Text::from_string("hello".to_string());
        let text2 = Text::from_string("hello".to_string());
        let text3 = Text::from_string("world".to_string());

        assert!(text1.text_equal(&text2));
        assert!(!text1.text_equal(&text3));

        // Test with subtexts
        let full_text = Text::from_string("hello world".to_string());
        let sub_text = full_text.subtext(0, 5).unwrap();
        assert!(text1.text_equal(&sub_text));
    }

    #[test]
    fn test_unicode_support() {
        let text = Text::from_string("こんにちは".to_string());
        assert_eq!(text.length(), 5); // 5 Japanese characters

        assert_eq!(text.char_at(0).unwrap(), 'こ');
        assert_eq!(text.char_at(1).unwrap(), 'ん');
        assert_eq!(text.char_at(4).unwrap(), 'は');

        let sub = text.subtext(0, 3).unwrap();
        assert_eq!(sub.text_to_string(), "こんに");
    }

    #[test]
    fn test_textual_helper_functions() {
        let text_value = Value::Text(Rc::new(Text::from_string("hello".to_string())));
        let string_value = Value::String("world".to_string());
        let number_value = Value::Number(crate::lexer::SchemeNumber::Integer(42));

        assert!(is_textual(&text_value));
        assert!(is_textual(&string_value));
        assert!(!is_textual(&number_value));

        let converted_text = textual_to_text(&string_value).unwrap();
        assert_eq!(converted_text.text_to_string(), "world");

        let converted_string = textual_to_string(&text_value).unwrap();
        assert_eq!(converted_string, "hello");
    }

    #[test]
    fn test_srfi_135_exports() {
        let srfi = Srfi135;
        let exports = srfi.exports();

        // Check that all required exports exist
        assert!(exports.contains_key("text?"));
        assert!(exports.contains_key("textual?"));
        assert!(exports.contains_key("textual-null?"));
        assert!(exports.contains_key("make-text"));
        assert!(exports.contains_key("text"));
        assert!(exports.contains_key("textual->text"));
        assert!(exports.contains_key("textual->string"));
        assert!(exports.contains_key("string->text"));
        assert!(exports.contains_key("text-length"));
        assert!(exports.contains_key("textual-length"));
        assert!(exports.contains_key("text-ref"));
        assert!(exports.contains_key("subtext"));
        assert!(exports.contains_key("textual-take"));
        assert!(exports.contains_key("textual-drop"));
        assert!(exports.contains_key("textual=?"));
    }

    #[test]
    fn test_text_procedures() {
        use crate::value::Procedure;

        let srfi = Srfi135;
        let exports = srfi.exports();

        // Test text? predicate
        let text_pred = exports.get("text?").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = text_pred {
            let text_value = Value::Text(Rc::new(Text::from_string("hello".to_string())));
            let result = func(&[text_value]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            let not_text = Value::String("hello".to_string());
            let result = func(&[not_text]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }

        // Test textual? predicate
        let textual_pred = exports.get("textual?").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = textual_pred {
            let text_value = Value::Text(Rc::new(Text::from_string("hello".to_string())));
            let result = func(&[text_value]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            let string_value = Value::String("hello".to_string());
            let result = func(&[string_value]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            let not_textual = Value::Number(crate::lexer::SchemeNumber::Integer(42));
            let result = func(&[not_textual]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }

        // Test text constructor
        let text_proc = exports.get("text").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = text_proc {
            let chars = vec![
                Value::Character('h'),
                Value::Character('e'),
                Value::Character('l'),
                Value::Character('l'),
                Value::Character('o'),
            ];
            let result = func(&chars).unwrap();
            if let Value::Text(text) = result {
                assert_eq!(text.text_to_string(), "hello");
                assert_eq!(text.length(), 5);
            } else {
                panic!("Expected text result");
            }
        }
    }
}
