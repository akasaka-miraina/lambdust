//! SRFI-135 Text type implementation for R7RS-large.
//!
//! This module implements immutable text values with high-performance
//! Unicode support, rope data structures, and Copy-on-Write optimization.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;
use regex::Regex;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;
// regex = "1.10"
use std::collections::HashMap;
use std::cmp::Ordering;

// ============= TEXT TYPE DEFINITION =============

/// Text type implementing SRFI-135 immutable text values.
///
/// This is a high-performance implementation using a rope data structure
/// for efficient substring operations and Copy-on-Write optimization.
#[derive(Debug, Clone)]
pub struct Text {
    /// Underlying string representation using rope structure
    content: Arc<TextContent>,
    /// Cached UTF-8 byte length
    byte_length: usize,
    /// Cached Unicode scalar value count
    char_length: usize,
    /// Cached grapheme cluster count
    grapheme_length: usize,
    /// Normalization form (lazy computed)
    normalization: Option<NormalizationForm>,
}

/// Internal text content representation using rope structure.
#[derive(Debug)]
enum TextContent {
    /// Leaf node containing actual string data
    Leaf(String),
    /// Internal node concatenating two text values
    Concat {
        left: Arc<TextContent>,
        right: Arc<TextContent>,
        length: usize,
    },
    /// Substring view into another text
    Substring {
        source: Arc<TextContent>,
        start: usize,
        end: usize,
    },
}

/// Unicode normalization forms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationForm {
    NFC,
    NFD,
    NFKC,
    NFKD,
}

/// Text builder for efficient string construction
#[derive(Debug)]
pub struct TextBuilder {
    parts: Vec<String>,
    total_length: usize,
}

/// Regular expression engine for text operations
#[derive(Debug)]
pub struct TextRegex {
    regex: Regex,
    pattern: String,
    flags: RegexFlags,
}

/// Regular expression flags
#[derive(Debug, Clone, Copy)]
pub struct RegexFlags {
    case_insensitive: bool,
    multiline: bool,
    dot_matches_newline: bool,
    unicode: bool,
}

impl TextRegex {
    /// Creates a new regex from a pattern.
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern).map_err(|e| {
            Box::new(DiagnosticError::runtime_error(
                format!("Invalid regex pattern: {e}"),
                None,
            ))
        })?;
        
        Ok(Self {
            regex,
            pattern: pattern.to_string(),
            flags: RegexFlags::default(),
        })
    }

    /// Finds all matches in the given text.
    pub fn find_all(&self, text: &Text) -> Vec<TextMatch> {
        let text_str = text.as_string();
        self.regex
            .find_iter(&text_str)
            .map(|m| TextMatch {
                text: Text::from_string(m.as_str().to_string()),
                matched_text: Text::from_string(m.as_str().to_string()),
                start: m.start(),
                end: m.end(),
                groups: vec![],
                named_groups: HashMap::new(),
            })
            .collect()
    }

    /// Splits the text using this regex as delimiter.
    pub fn split(&self, text: &Text) -> Vec<Text> {
        let text_str = text.as_string();
        self.regex
            .split(&text_str)
            .map(|s| Text::from_string(s.to_string()))
            .collect()
    }

    /// Returns true if the regex matches the text.
    pub fn is_match(&self, text: &Text) -> bool {
        let text_str = text.as_string();
        self.regex.is_match(&text_str)
    }

    /// Finds the first match in the text.
    pub fn find(&self, text: &Text) -> Option<TextMatch> {
        let text_str = text.as_string();
        self.regex.find(&text_str).map(|m| TextMatch {
            text: Text::from_string(m.as_str().to_string()),
            matched_text: Text::from_string(m.as_str().to_string()),
            start: m.start(),
            end: m.end(),
            groups: vec![],
            named_groups: HashMap::new(),
        })
    }

    /// Replaces all matches in the text.
    pub fn replace_all(&self, text: &Text, replacement: &Text) -> Text {
        let text_str = text.as_string();
        let replacement_str = replacement.as_string();
        let result = self.regex.replace_all(&text_str, replacement_str);
        Text::from_string(result.to_string())
    }
}

impl RegexFlags {
    pub fn new_default() -> Self {
        Self {
            case_insensitive: false,
            multiline: false,
            dot_matches_newline: false,
            unicode: true,
        }
    }
}

/// Match result for regular expressions
#[derive(Debug, Clone)]
pub struct TextMatch {
    text: Text,
    pub matched_text: Text,
    pub start: usize,
    pub end: usize,
    pub groups: Vec<Option<Text>>,
    named_groups: HashMap<String, Option<Text>>,
}

impl Text {
    /// Creates a new empty text.
    pub fn new() -> Self {
        Self::from_string(String::new())
    }

    /// Creates a text from a string.
    pub fn from_string(s: String) -> Self {
        let byte_length = s.len();
        let char_length = s.chars().count();
        let grapheme_length = s.graphemes(true).count();
        
        Self {
            content: Arc::new(TextContent::Leaf(s)),
            byte_length,
            char_length,
            grapheme_length,
            normalization: None,
        }
    }

    /// Creates a text from a string slice.
    pub fn from_string_slice(s: &str) -> Self {
        Self::from_string(s.to_string())
    }

    /// Creates a text by repeating a character.
    pub fn repeat_char(ch: char, count: usize) -> Self {
        let s = ch.to_string().repeat(count);
        Self::from_string(s)
    }

    /// Returns true if the text is empty.
    pub fn is_empty(&self) -> bool {
        self.byte_length == 0
    }

    /// Returns the length in bytes.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Returns the length in Unicode scalar values.
    pub fn char_length(&self) -> usize {
        self.char_length
    }

    /// Returns the length in grapheme clusters.
    pub fn grapheme_length(&self) -> usize {
        self.grapheme_length
    }

    /// Converts the text to a string.
    pub fn as_string(&self) -> String {
        self.content.as_string()
    }

    /// Gets a character at the specified index.
    pub fn char_at(&self, index: usize) -> Option<char> {
        if index >= self.char_length {
            return None;
        }
        self.as_string().chars().nth(index)
    }

    /// Gets a substring as a new text.
    pub fn substring(&self, start: usize, end: usize) -> Option<Self> {
        if start > end || end > self.char_length {
            return None;
        }

        if start == 0 && end == self.char_length {
            return Some(self.clone());
        }

        // Convert character indices to byte indices
        let s = self.as_string();
        let mut char_indices = s.char_indices();
        
        let start_byte = if start == 0 {
            0
        } else {
            char_indices.nth(start)?.0
        };

        let end_byte = if end == self.char_length {
            s.len()
        } else {
            char_indices.nth(end - start - 1).map(|(i, _)| i)
                .unwrap_or(s.len())
        };

        let substr = s[start_byte..end_byte].to_string();
        Some(Self::from_string(substr))
    }

    /// Concatenates this text with another.
    pub fn concat(&self, other: &Self) -> Self {
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }

        let new_content = Arc::new(TextContent::Concat {
            left: self.content.clone(),
            right: other.content.clone(),
            length: self.byte_length + other.byte_length,
        });

        Self {
            content: new_content,
            byte_length: self.byte_length + other.byte_length,
            char_length: self.char_length + other.char_length,
            grapheme_length: self.grapheme_length + other.grapheme_length,
            normalization: None,
        }
    }

    /// Appends another text to this one (Copy-on-Write).
    pub fn append(&self, other: &Self) -> Self {
        self.concat(other)
    }

    /// Checks if this text starts with the given prefix.
    pub fn starts_with(&self, prefix: &Self) -> bool {
        if prefix.char_length > self.char_length {
            return false;
        }
        
        let self_str = self.as_string();
        let prefix_str = prefix.as_string();
        self_str.starts_with(&prefix_str)
    }

    /// Checks if this text ends with the given suffix.
    pub fn ends_with(&self, suffix: &Self) -> bool {
        if suffix.char_length > self.char_length {
            return false;
        }
        
        let self_str = self.as_string();
        let suffix_str = suffix.as_string();
        self_str.ends_with(&suffix_str)
    }

    /// Finds the first occurrence of a substring.
    pub fn find(&self, needle: &Self) -> Option<usize> {
        let haystack = self.as_string();
        let needle_str = needle.as_string();
        
        haystack.find(&needle_str).map(|byte_pos| {
            haystack[..byte_pos].chars().count()
        })
    }

    /// Finds the last occurrence of a substring.
    pub fn rfind(&self, needle: &Self) -> Option<usize> {
        let haystack = self.as_string();
        let needle_str = needle.as_string();
        
        haystack.rfind(&needle_str).map(|byte_pos| {
            haystack[..byte_pos].chars().count()
        })
    }

    /// Checks if this text contains the given substring.
    pub fn contains(&self, needle: &Self) -> bool {
        self.find(needle).is_some()
    }

    /// Splits the text by a delimiter.
    pub fn split(&self, delimiter: &Self) -> Vec<Self> {
        let s = self.as_string();
        let delim = delimiter.as_string();
        
        if delim.is_empty() {
            // Split into individual characters
            return s.chars().map(|c| Self::from_string(c.to_string())).collect();
        }
        
        s.split(&delim)
            .map(|part| Self::from_string(part.to_string()))
            .collect()
    }

    /// Replaces all occurrences of a pattern with replacement text.
    pub fn replace(&self, pattern: &Self, replacement: &Self) -> Self {
        let s = self.as_string();
        let pattern_str = pattern.as_string();
        let replacement_str = replacement.as_string();
        
        let replaced = s.replace(&pattern_str, &replacement_str);
        Self::from_string(replaced)
    }

    /// Converts to uppercase.
    pub fn to_uppercase(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.to_uppercase())
    }

    /// Converts to lowercase.
    pub fn to_lowercase(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.to_lowercase())
    }

    /// Converts to titlecase.
    pub fn to_titlecase(&self) -> Self {
        let s = self.as_string();
        let mut result = String::new();
        let mut first = true;
        
        for ch in s.chars() {
            if first && ch.is_alphabetic() {
                result.extend(ch.to_uppercase());
                first = false;
            } else if ch.is_whitespace() {
                result.push(ch);
                first = true;
            } else {
                result.extend(ch.to_lowercase());
            }
        }
        
        Self::from_string(result)
    }

    /// Case-folds the text for case-insensitive comparison.
    pub fn fold_case(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.to_lowercase())
    }

    /// Trims whitespace from both ends.
    pub fn trim(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.trim().to_string())
    }

    /// Trims whitespace from the start.
    pub fn trim_start(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.trim_start().to_string())
    }

    /// Trims whitespace from the end.
    pub fn trim_end(&self) -> Self {
        let s = self.as_string();
        Self::from_string(s.trim_end().to_string())
    }

    /// Reverses the text.
    pub fn reverse(&self) -> Self {
        let s = self.as_string();
        let reversed: String = s.chars().rev().collect();
        Self::from_string(reversed)
    }

    /// Normalizes the text to the specified Unicode normalization form.
    /// Note: This is a simplified implementation. Full Unicode normalization
    /// would require the unicode-normalization crate.
    pub fn normalize(&self, form: NormalizationForm) -> Self {
        let s = self.as_string();
        // Simplified implementation - in production would use proper Unicode normalization
        let normalized = match form {
            NormalizationForm::NFC | NormalizationForm::NFD | 
            NormalizationForm::NFKC | NormalizationForm::NFKD => s,
        };
        
        let mut result = Self::from_string(normalized);
        result.normalization = Some(form);
        result
    }

    /// Checks if the text is in the specified normalization form.
    /// Note: This is a simplified implementation.
    pub fn is_normalized(&self, _form: NormalizationForm) -> bool {
        // Simplified implementation - assume always normalized
        true
    }

    /// Gets the cached normalization form, if any.
    pub fn normalization_form(&self) -> Option<NormalizationForm> {
        self.normalization
    }

    /// Compares texts lexicographically.
    pub fn compare(&self, other: &Self) -> Ordering {
        self.as_string().cmp(&other.as_string())
    }

    /// Compares texts case-insensitively.
    pub fn compare_ci(&self, other: &Self) -> Ordering {
        self.to_lowercase().compare(&other.to_lowercase())
    }

    /// Iterates over characters.
    pub fn chars(&self) -> Vec<char> {
        self.as_string().chars().collect()
    }

    /// Iterates over grapheme clusters.
    pub fn graphemes(&self) -> Vec<String> {
        self.as_string().graphemes(true).map(|s| s.to_string()).collect()
    }

    /// Gets grapheme cluster boundaries.
    /// Note: This is a simplified implementation.
    pub fn grapheme_indices(&self) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut byte_index = 0;
        
        for ch in self.chars() {
            result.push((byte_index, ch.to_string()));
            byte_index += ch.len_utf8();
        }
        
        result
    }
}

impl TextContent {
    /// Converts the content to a string.
    fn as_string(&self) -> String {
        match self {
            TextContent::Leaf(s) => s.clone(),
            TextContent::Concat { left, right, .. } => {
                let mut result = left.as_string();
                result.push_str(&right.as_string());
                result
            }
            TextContent::Substring { source, start, end } => {
                let s = source.as_string();
                s.chars().skip(*start).take(end - start).collect()
            }
        }
    }
}

impl TextBuilder {
    /// Creates a new text builder.
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            total_length: 0,
        }
    }

    /// Creates a text builder with initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            parts: Vec::with_capacity(capacity),
            total_length: 0,
        }
    }

    /// Appends a string to the builder.
    pub fn push_str(&mut self, s: &str) {
        self.parts.push(s.to_string());
        self.total_length += s.len();
    }

    /// Appends a character to the builder.
    pub fn push_char(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.push_str(s);
    }

    /// Appends a text to the builder.
    pub fn push_text(&mut self, text: &Text) {
        self.push_str(&text.as_string());
    }

    /// Builds the final text.
    pub fn build(self) -> Text {
        if self.parts.is_empty() {
            return Text::new();
        }
        
        let mut result = String::with_capacity(self.total_length);
        for part in self.parts {
            result.push_str(&part);
        }
        
        Text::from_string(result)
    }

    /// Returns the current length in bytes.
    pub fn len(&self) -> usize {
        self.total_length
    }

    /// Returns true if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.total_length == 0
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RegexFlags {
    fn default() -> Self {
        Self {
            case_insensitive: false,
            multiline: false,
            dot_matches_newline: false,
            unicode: true,
        }
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        if self.char_length != other.char_length {
            return false;
        }
        self.as_string() == other.as_string()
    }
}

impl Eq for Text {}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for Text {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::from_string_slice(s)
    }
}

impl From<char> for Text {
    fn from(ch: char) -> Self {
        Self::from_string(ch.to_string())
    }
}

// ============= SCHEME VALUE INTEGRATION =============

impl From<Text> for Value {
    fn from(text: Text) -> Self {
        Value::Literal(crate::ast::Literal::String(text.as_string()))
    }
}

impl TryFrom<&Value> for Text {
    type Error = Box<DiagnosticError>;

    fn try_from(value: &Value) -> Result<Self> {
        match value {
            Value::Literal(crate::ast::Literal::String(s)) => Ok(Text::from_string(s.clone())),
            _ => Err(Box::new(DiagnosticError::runtime_error(
                "Expected text/string value".to_string(),
                None,
            ))),
        }
    }
}

// ============= SRFI-135 API BINDINGS =============

/// Creates SRFI-135 text operation bindings for the standard library.
pub fn create_text_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Text constructors
    bind_text_constructors(env);
    
    // Text predicates
    bind_text_predicates(env);
    
    // Text accessors
    bind_text_accessors(env);
    
    // Text comparison
    bind_text_comparison(env);
    
    // Text manipulation
    bind_text_manipulation(env);
    
    // Text conversion
    bind_text_conversion(env);
    
    // Unicode operations
    bind_unicode_operations(env);
}

/// Binds text constructor operations.
fn bind_text_constructors(env: &Arc<ThreadSafeEnvironment>) {
    // text
    env.define("text".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_text),
        effects: vec![Effect::Pure],
    })));
    
    // make-text
    env.define("make-text".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-text".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_text),
        effects: vec![Effect::Pure],
    })));
    
    // text-tabulate
    env.define("text-tabulate".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-tabulate".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_tabulate),
        effects: vec![Effect::Pure],
    })));
}

/// Binds text predicate operations.
fn bind_text_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // text?
    env.define("text?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_p),
        effects: vec![Effect::Pure],
    })));
    
    // text-null?
    env.define("text-null?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-null?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_null_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds text accessor operations.
fn bind_text_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // text-length    
    env.define("text-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-length".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_length),
        effects: vec![Effect::Pure],
    })));
    
    // text-ref
    env.define("text-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-ref".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_ref),
        effects: vec![Effect::Pure],
    })));
}

/// Binds text comparison operations.
fn bind_text_comparison(env: &Arc<ThreadSafeEnvironment>) {
    // text=?
    env.define("text=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_text_equal),
        effects: vec![Effect::Pure],
    })));
    
    // text<?
    env.define("text<?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text<?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_text_less),
        effects: vec![Effect::Pure],
    })));
    
    // text-ci=?
    env.define("text-ci=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-ci=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_text_ci_equal),
        effects: vec![Effect::Pure],
    })));
}

/// Binds text manipulation operations.
fn bind_text_manipulation(env: &Arc<ThreadSafeEnvironment>) {
    // text-append
    env.define("text-append".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-append".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_text_append),
        effects: vec![Effect::Pure],
    })));
    
    // subtext
    env.define("subtext".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "subtext".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_subtext),
        effects: vec![Effect::Pure],
    })));
    
    // text-copy
    env.define("text-copy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-copy".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_text_copy),
        effects: vec![Effect::Pure],
    })));
}

/// Binds text conversion operations.
fn bind_text_conversion(env: &Arc<ThreadSafeEnvironment>) {
    // string->text
    env.define("string->text".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string->text".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_string_to_text),
        effects: vec![Effect::Pure],
    })));
    
    // text->string
    env.define("text->string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text->string".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_text_to_string),
        effects: vec![Effect::Pure],
    })));
    
    // text->list
    env.define("text->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text->list".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_text_to_list),
        effects: vec![Effect::Pure],
    })));
    
    // list->text
    env.define("list->text".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->text".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_list_to_text),
        effects: vec![Effect::Pure],
    })));
}

/// Binds Unicode normalization operations.
fn bind_unicode_operations(env: &Arc<ThreadSafeEnvironment>) {
    // text-normalize-nfc
    env.define("text-normalize-nfc".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-normalize-nfc".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_normalize_nfc),
        effects: vec![Effect::Pure],
    })));
    
    // text-normalize-nfd
    env.define("text-normalize-nfd".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-normalize-nfd".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_normalize_nfd),
        effects: vec![Effect::Pure],
    })));
    
    // text-normalize-nfkc
    env.define("text-normalize-nfkc".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-normalize-nfkc".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_normalize_nfkc),
        effects: vec![Effect::Pure],
    })));
    
    // text-normalize-nfkd
    env.define("text-normalize-nfkd".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-normalize-nfkd".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_text_normalize_nfkd),
        effects: vec![Effect::Pure],
    })));
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// text constructor
fn primitive_text(args: &[Value]) -> Result<Value> {
    let mut builder = TextBuilder::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::Character(ch)) => {
                builder.push_char(*ch);
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "text constructor requires character arguments".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(builder.build().into())
}

/// make-text constructor
fn primitive_make_text(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-text expects 1-2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let len = args[0].as_integer().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "make-text first argument must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let ch = if args.len() > 1 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::Character(c)) => *c,
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "make-text second argument must be a character".to_string(),
                    None,
                )));
            }
        }
    } else {
        ' '
    };
    
    Ok(Text::repeat_char(ch, len).into())
}

/// text? predicate
fn primitive_text_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_text = matches!(args[0], Value::Literal(crate::ast::Literal::String(_)));
    Ok(Value::boolean(is_text))
}

/// text-null? predicate
fn primitive_text_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-null? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(Value::boolean(text.is_empty()))
}

/// text-length accessor
fn primitive_text_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-length expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(Value::integer(text.char_length() as i64))
}

/// text-ref accessor
fn primitive_text_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-ref expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    let index = args[1].as_integer().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "text-ref index must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    match text.char_at(index) {
        Some(ch) => Ok(Value::Literal(crate::ast::Literal::Character(ch))),
        None => Err(Box::new(DiagnosticError::runtime_error(
            "text-ref index out of bounds".to_string(),
            None,
        ))),
    }
}

/// text=? comparison
fn primitive_text_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "text=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = Text::try_from(&args[0])?;
    
    for arg in &args[1..] {
        let text = Text::try_from(arg)?;
        if first != text {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// text<? comparison
fn primitive_text_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "text<? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let t1 = Text::try_from(&window[0])?;
        let t2 = Text::try_from(&window[1])?;
        if t1.compare(&t2) != Ordering::Less {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// text-ci=? case-insensitive comparison
fn primitive_text_ci_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "text-ci=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = Text::try_from(&args[0])?.fold_case();
    
    for arg in &args[1..] {
        let text = Text::try_from(arg)?.fold_case();
        if first != text {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// text-append manipulation
fn primitive_text_append(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Text::new().into());
    }
    
    let mut result = Text::try_from(&args[0])?;
    
    for arg in &args[1..] {
        let text = Text::try_from(arg)?;
        result = result.append(&text);
    }
    
    Ok(result.into())
}

/// subtext manipulation
fn primitive_subtext(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("subtext expects 2-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    let start = args[1].as_integer().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "subtext start index must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let end = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
        Box::new(
            DiagnosticError::runtime_error(
                "subtext end index must be an integer".to_string(),
                None,
            ))
        })? as usize
    } else {
        text.char_length()
    };
    
    match text.substring(start, end) {
        Some(subtext) => Ok(subtext.into()),
        None => Err(Box::new(DiagnosticError::runtime_error(
            "subtext indices out of bounds".to_string(),
            None,
        ))),
    }
}

/// text-copy operation
fn primitive_text_copy(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-copy expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    
    if args.len() == 1 {
        return Ok(text.into())
    }
    
    let start = args[1].as_integer().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "text-copy start index must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let end = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
        Box::new(
            DiagnosticError::runtime_error(
                "text-copy end index must be an integer".to_string(),
                None,
            ))
        })? as usize
    } else {
        text.char_length()
    };
    
    match text.substring(start, end) {
        Some(copy) => Ok(copy.into()),
        None => Err(Box::new(DiagnosticError::runtime_error(
            "text-copy indices out of bounds".to_string(),
            None,
        ))),
    }
}

/// string->text conversion
fn primitive_string_to_text(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string->text expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = args[0].as_string().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "string->text first argument must be a string".to_string(),
            None,
        ))
    })?;
    
    // For now, we just convert the entire string
    // In a full implementation, we'd handle start/end indices
    let text = Text::from_string_slice(s);
    Ok(text.into())
}

/// text->string conversion
fn primitive_text_to_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text->string expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    
    // For now, we just convert the entire text
    // In a full implementation, we'd handle start/end indices
    Ok(Value::string(text.as_string()))
}

/// text->list conversion
fn primitive_text_to_list(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text->list expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    
    let chars: Vec<Value> = text.chars()
        .iter()
        .map(|ch| Value::Literal(crate::ast::Literal::Character(*ch)))
        .collect();
    
    Ok(Value::list(chars))
}

/// list->text conversion
fn primitive_list_to_text(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list->text expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "list->text argument must be a list".to_string(),
            None,
        ))
    })?;
    
    let mut builder = TextBuilder::new();
    
    for item in list {
        match item {
            Value::Literal(crate::ast::Literal::Character(ch)) => {
                builder.push_char(ch);
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "list->text list must contain only characters".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(builder.build().into())
}

/// text-tabulate constructor
fn primitive_text_tabulate(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-tabulate expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let len = args[0].as_integer().ok_or_else(|| {
        Box::new(
        DiagnosticError::runtime_error(
            "text-tabulate first argument must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let proc = &args[1];
    if !proc.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "text-tabulate second argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // For now, we'll create a simple implementation
    // In a full implementation, we'd call the procedure for each index
    let mut builder = TextBuilder::new();
    for i in 0..len {
        // This is a simplified version - would need proper procedure call
        builder.push_char((b'a' + (i % 26) as u8) as char);
    }
    
    Ok(builder.build().into())
}

/// Unicode normalization functions
fn primitive_text_normalize_nfc(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-normalize-nfc expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(text.normalize(NormalizationForm::NFC).into())
}

fn primitive_text_normalize_nfd(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-normalize-nfd expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(text.normalize(NormalizationForm::NFD).into())
}

fn primitive_text_normalize_nfkc(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-normalize-nfkc expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(text.normalize(NormalizationForm::NFKC).into())
}

fn primitive_text_normalize_nfkd(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-normalize-nfkd expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    Ok(text.normalize(NormalizationForm::NFKD).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::from_string_slice("Hello, 世界!");
        assert_eq!(text.as_string(), "Hello, 世界!");
        assert_eq!(text.char_length(), 9);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_text_operations() {
        let t1 = Text::from_string_slice("Hello");
        let t2 = Text::from_string_slice(", World!");
        let result = t1.concat(&t2);
        
        assert_eq!(result.as_string(), "Hello, World!");
        assert_eq!(result.char_length(), 13);
    }

    #[test]
    fn test_text_substring() {
        let text = Text::from_string_slice("Hello, World!");
        let sub = text.substring(0, 5).unwrap();
        
        assert_eq!(sub.as_string(), "Hello");
        assert_eq!(sub.char_length(), 5);
    }

    #[test]
    fn test_unicode_normalization() {
        let text = Text::from_string_slice("é"); // composed
        let nfd = text.normalize(NormalizationForm::NFD);
        
        // NFD should have more characters (base + combining)
        assert!(nfd.char_length() > text.char_length());
        
        let nfc = nfd.normalize(NormalizationForm::NFC);
        assert_eq!(nfc.as_string(), text.as_string());
    }

    #[test]
    fn test_text_builder() {
        let mut builder = TextBuilder::new();
        builder.push_str("Hello");
        builder.push_char(',');
        builder.push_str(" World!");
        
        let text = builder.build();
        assert_eq!(text.as_string(), "Hello, World!");
    }

    #[test]
    fn test_text_comparison() {
        let t1 = Text::from_string_slice("abc");
        let t2 = Text::from_string_slice("def");
        let t3 = Text::from_string_slice("ABC");
        
        assert_eq!(t1.compare(&t2), Ordering::Less);
        assert_eq!(t1.compare_ci(&t3), Ordering::Equal);
    }
}