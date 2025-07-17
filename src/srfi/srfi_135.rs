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

