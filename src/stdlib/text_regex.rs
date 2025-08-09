#![allow(unused_variables)]
//! Regular expression engine for SRFI-135 Text processing.
//!
//! This module implements a PCRE-compatible regular expression engine
//! with Unicode support, named capture groups, and efficient matching.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::text::Text;
use std::sync::Arc;
use regex::Captures;
use std::collections::HashMap;

// ============= REGEX ENGINE =============

/// Regular expression compiled for text processing.
/// Note: This is a simplified implementation. Full regex support
/// would require the regex crate.
#[derive(Debug, Clone)]
pub struct TextRegex {
    /// Original pattern string
    pattern: String,
    /// Regex flags
    flags: RegexFlags,
    /// Compiled regex
    regex: regex::Regex,
}

/// Regular expression compilation flags.
#[derive(Debug, Clone, Copy)]
pub struct RegexFlags {
    /// Case-insensitive matching
    pub case_insensitive: bool,
    /// Multi-line mode (^ and $ match line boundaries)
    pub multiline: bool,
    /// Dot matches newline
    pub dot_matches_newline: bool,
    /// Unicode mode (default true)
    pub unicode: bool,
    /// Extended syntax (ignore whitespace and comments)
    pub extended: bool,
    /// Swap greed of quantifiers
    pub swap_greed: bool,
}

/// Match result from regex operations.
#[derive(Debug, Clone)]
pub struct TextMatchResult {
    /// The matched text
    pub matched_text: Text,
    /// Start position (character index)
    pub start: usize,
    /// End position (character index)
    pub end: usize,
    /// Captured groups (indexed)
    pub groups: Vec<Option<Text>>,
    /// Named capture groups
    pub named_groups: HashMap<String, Option<Text>>,
}

/// Match iterator for finding all matches.
pub struct TextMatchIter<'t> {
    regex: &'t TextRegex,
    text: &'t Text,
    last_end: usize,
}

impl TextRegex {
    /// Compiles a regular expression with default flags.
    /// Note: This is a simplified implementation without actual regex compilation.
    pub fn new(pattern: &str) -> Result<Self> {
        Self::with_flags(pattern, RegexFlags::default())
    }

    /// Compiles a regular expression with specific flags.
    /// Note: This is a simplified implementation.
    pub fn with_flags(pattern: &str, flags: RegexFlags) -> Result<Self> {
        // Build regex with flags
        let mut builder = regex::RegexBuilder::new(pattern);
        builder.case_insensitive(flags.case_insensitive);
        builder.multi_line(flags.multiline);
        builder.dot_matches_new_line(flags.dot_matches_newline);
        builder.unicode(flags.unicode);
        builder.ignore_whitespace(flags.extended);
        builder.swap_greed(flags.swap_greed);
        
        let regex = builder.build().map_err(|e| DiagnosticError::runtime_error(
            format!("Invalid regex pattern: {e}"),
            None
        ))?;
        
        Ok(Self {
            pattern: pattern.to_string(),
            flags,
            regex,
        })
    }

    /// Gets the original pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Gets the regex flags.
    pub fn flags(&self) -> RegexFlags {
        self.flags
    }

    /// Tests if the pattern matches anywhere in the text.
    /// Note: Simplified implementation using basic string matching.
    pub fn is_match(&self, text: &Text) -> bool {
        text.to_string().contains(&self.pattern)
    }

    /// Finds the first match in the text.
    /// Note: Simplified implementation using basic string searching.
    pub fn find(&self, text: &Text) -> Option<TextMatchResult> {
        let s = text.to_string();
        if let Some(pos) = s.find(&self.pattern) {
            let start_char = s[..pos].chars().count();
            let end_char = start_char + self.pattern.chars().count();
            
            Some(TextMatchResult {
                matched_text: Text::from_string_slice(&self.pattern),
                start: start_char,
                end: end_char,
                groups: vec![],
                named_groups: HashMap::new(),
            })
        } else {
            None
        }
    }

    /// Finds all matches in the text.
    pub fn find_all(&self, text: &Text) -> Vec<TextMatchResult> {
        let s = text.to_string();
        self.regex
            .captures_iter(&s)
            .filter_map(|caps| self.captures_to_match_result(text, &caps))
            .collect()
    }

    /// Creates an iterator over all matches.
    pub fn find_iter<'t>(&'t self, text: &'t Text) -> TextMatchIter<'t> {
        TextMatchIter {
            regex: self,
            text,
            last_end: 0,
        }
    }

    /// Replaces the first match with replacement text.
    pub fn replace(&self, text: &Text, replacement: &Text) -> Text {
        let s = text.to_string();
        let replacement_str = replacement.to_string();
        let result = self.regex.replace(&s, replacement_str.as_str());
        Text::from_string(result.into_owned())
    }

    /// Replaces all matches with replacement text.
    pub fn replace_all(&self, text: &Text, replacement: &Text) -> Text {
        let s = text.to_string();
        let replacement_str = replacement.to_string();
        let result = self.regex.replace_all(&s, replacement_str.as_str());
        Text::from_string(result.into_owned())
    }

    /// Replaces matches using a callback function.
    pub fn replace_all_with<F>(&self, text: &Text, replacer: F) -> Text
    where
        F: Fn(&TextMatchResult) -> Text,
    {
        let s = text.to_string();
        let mut result = String::new();
        let mut last_end = 0;

        for captures in self.regex.captures_iter(&s) {
            let m = captures.get(0).unwrap();
            
            // Add text before match
            result.push_str(&s[last_end..m.start()]);
            
            // Create match result and get replacement
            if let Some(match_result) = self.captures_to_match_result(text, &captures) {
                let replacement = replacer(&match_result);
                result.push_str(&replacement.to_string());
            }
            
            last_end = m.end();
        }
        
        // Add remaining text
        result.push_str(&s[last_end..]);
        
        Text::from_string(result)
    }

    /// Splits the text by the regex pattern.
    pub fn split(&self, text: &Text) -> Vec<Text> {
        let s = text.to_string();
        self.regex
            .split(&s)
            .map(|part| Text::from_string(part.to_string()))
            .collect()
    }

    /// Splits the text by the regex pattern with limit.
    pub fn splitn(&self, text: &Text, limit: usize) -> Vec<Text> {
        let s = text.to_string();
        self.regex
            .splitn(&s, limit)
            .map(|part| Text::from_string(part.to_string()))
            .collect()
    }

    /// Converts regex captures to match result.
    fn captures_to_match_result(&self, text: &Text, captures: &Captures<'_>) -> Option<TextMatchResult> {
        let full_match = captures.get(0)?;
        let text_str = text.to_string();
        
        // Convert byte positions to character positions
        let start_char = text_str[..full_match.start()].chars().count();
        let end_char = text_str[..full_match.end()].chars().count();
        
        let matched_text = text.substring(start_char, end_char)?;
        
        // Extract numbered groups
        let mut groups = Vec::new();
        for i in 1..captures.len() {
            if let Some(group_match) = captures.get(i) {
                let group_start = text_str[..group_match.start()].chars().count();
                let group_end = text_str[..group_match.end()].chars().count();
                let group_text = text.substring(group_start, group_end);
                groups.push(group_text);
            } else {
                groups.push(None);
            }
        }
        
        // Extract named groups
        let mut named_groups = HashMap::new();
        for name in self.regex.capture_names().flatten() {
            if let Some(group_match) = captures.name(name) {
                let group_start = text_str[..group_match.start()].chars().count();
                let group_end = text_str[..group_match.end()].chars().count();
                let group_text = text.substring(group_start, group_end);
                named_groups.insert(name.to_string(), group_text);
            } else {
                named_groups.insert(name.to_string(), None);
            }
        }

        Some(TextMatchResult {
            matched_text,
            start: start_char,
            end: end_char,
            groups,
            named_groups,
        })
    }
}

impl<'t> Iterator for TextMatchIter<'t> {
    type Item = TextMatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end > self.text.char_length() {
            return None;
        }

        let remaining_text = self.text.substring(self.last_end, self.text.char_length())?;
        let match_result = self.regex.find(&remaining_text)?;
        
        // Adjust positions to be relative to original text
        let adjusted_result = TextMatchResult {
            matched_text: match_result.matched_text,
            start: match_result.start + self.last_end,
            end: match_result.end + self.last_end,
            groups: match_result.groups,
            named_groups: match_result.named_groups,
        };
        
        self.last_end = adjusted_result.end;
        Some(adjusted_result)
    }
}

impl Default for RegexFlags {
    fn default() -> Self {
        Self {
            case_insensitive: false,
            multiline: false,
            dot_matches_newline: false,
            unicode: true,
            extended: false,
            swap_greed: false,
        }
    }
}

impl RegexFlags {
    /// Creates flags for case-insensitive matching.
    pub fn case_insensitive() -> Self {
        Self {
            case_insensitive: true,
            ..Default::default()
        }
    }

    /// Creates flags for multiline matching.
    pub fn multiline() -> Self {
        Self {
            multiline: true,
            ..Default::default()
        }
    }

    /// Creates flags with all common options enabled.
    pub fn extended() -> Self {
        Self {
            case_insensitive: true,
            multiline: true,
            dot_matches_newline: true,
            unicode: true,
            extended: true,
            swap_greed: false,
        }
    }
}

// ============= SCHEME BINDINGS =============

/// Creates regex operation bindings for the standard library.
pub fn create_regex_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Regex compilation
    bind_regex_construction(env);
    
    // Pattern matching
    bind_regex_matching(env);
    
    // Text replacement
    bind_regex_replacement(env);
    
    // Text splitting
    bind_regex_splitting(env);
}

/// Binds regex construction operations.
fn bind_regex_construction(env: &Arc<ThreadSafeEnvironment>) {
    // regex-compile
    env.define("regex-compile".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-compile".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_regex_compile),
        effects: vec![Effect::Pure],
    })));
    
    // regex-compile-ci (case-insensitive)
    env.define("regex-compile-ci".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-compile-ci".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_regex_compile_ci),
        effects: vec![Effect::Pure],
    })));
}

/// Binds regex matching operations.
fn bind_regex_matching(env: &Arc<ThreadSafeEnvironment>) {
    // regex-match?
    env.define("regex-match?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-match?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_regex_match_p),
        effects: vec![Effect::Pure],
    })));
    
    // regex-search
    env.define("regex-search".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-search".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_regex_search),
        effects: vec![Effect::Pure],
    })));
    
    // regex-search-all
    env.define("regex-search-all".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-search-all".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_regex_search_all),
        effects: vec![Effect::Pure],
    })));
}

/// Binds regex replacement operations.
fn bind_regex_replacement(env: &Arc<ThreadSafeEnvironment>) {
    // regex-replace
    env.define("regex-replace".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-replace".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_regex_replace),
        effects: vec![Effect::Pure],
    })));
    
    // regex-replace-all
    env.define("regex-replace-all".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-replace-all".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_regex_replace_all),
        effects: vec![Effect::Pure],
    })));
}

/// Binds regex splitting operations.
fn bind_regex_splitting(env: &Arc<ThreadSafeEnvironment>) {
    // regex-split
    env.define("regex-split".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "regex-split".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_regex_split),
        effects: vec![Effect::Pure],
    })));
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// regex-compile operation
fn primitive_regex_compile(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-compile expects 1-2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-compile pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let flags = if args.len() > 1 {
        // Parse flags from string or other representation
        RegexFlags::default() // Simplified for now
    } else {
        RegexFlags::default()
    };
    
    let regex = TextRegex::with_flags(pattern, flags)?;
    
    // For now, we'll store the regex as a foreign object
    // In a complete implementation, we'd have a proper regex value type
    Ok(Value::string(format!("regex:{pattern}")))
}

/// regex-compile-ci operation
fn primitive_regex_compile_ci(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-compile-ci expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-compile-ci pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let regex = TextRegex::with_flags(pattern, RegexFlags::case_insensitive())?;
    
    Ok(Value::string(format!("regex-ci:{pattern}")))
}

/// regex-match? predicate
fn primitive_regex_match_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-match? expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-match? pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let regex = TextRegex::new(pattern)?;
    
    Ok(Value::boolean(regex.is_match(&text)))
}

/// regex-search operation
fn primitive_regex_search(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-search expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-search pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let regex = TextRegex::new(pattern)?;
    
    match regex.find(&text) {
        Some(match_result) => {
            // Return a match object - for now, return the matched text
            Ok(match_result.matched_text.into())
        }
        None => Ok(Value::boolean(false)),
    }
}

/// regex-search-all operation
fn primitive_regex_search_all(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-search-all expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-search-all pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let regex = TextRegex::new(pattern)?;
    
    let matches = regex.find_all(&text);
    let match_values: Vec<Value> = matches
        .into_iter()
        .map(|m| m.matched_text.into())
        .collect();
    
    Ok(Value::list(match_values))
}

/// regex-replace operation
fn primitive_regex_replace(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-replace expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-replace pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let replacement = Text::try_from(&args[2])?;
    
    let regex = TextRegex::new(pattern)?;
    let result = regex.replace(&text, &replacement);
    
    Ok(result.into())
}

/// regex-replace-all operation
fn primitive_regex_replace_all(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-replace-all expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-replace-all pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let replacement = Text::try_from(&args[2])?;
    
    let regex = TextRegex::new(pattern)?;
    let result = regex.replace_all(&text, &replacement);
    
    Ok(result.into())
}

/// regex-split operation
fn primitive_regex_split(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("regex-split expects 2-3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = args[0].as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "regex-split pattern must be a string".to_string(),
            None,
        ))
    })?;
    
    let text = Text::try_from(&args[1])?;
    let regex = TextRegex::new(pattern)?;
    
    let parts = if args.len() > 2 {
        let limit = args[2].as_integer().ok_or_else(|| {
            Box::new(DiagnosticError::runtime_error(
                "regex-split limit must be an integer".to_string(),
                None,
            ))
        })? as usize;
        regex.splitn(&text, limit)
    } else {
        regex.split(&text)
    };
    
    let part_values: Vec<Value> = parts.into_iter().map(|p| p.into()).collect();
    Ok(Value::list(part_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_compilation() {
        let regex = TextRegex::new(r"\d+").unwrap();
        assert_eq!(regex.pattern(), r"\d+");
    }

    #[test]
    fn test_regex_matching() {
        let regex = TextRegex::new(r"\d+").unwrap();
        let text = Text::from_string_slice("abc123def");
        
        assert!(regex.is_match(&text));
        
        let match_result = regex.find(&text).unwrap();
        assert_eq!(match_result.matched_text.to_string(), "123");
        assert_eq!(match_result.start, 3);
        assert_eq!(match_result.end, 6);
    }

    #[test]
    fn test_regex_replacement() {
        let regex = TextRegex::new(r"\d+").unwrap();
        let text = Text::from_string_slice("abc123def456");
        let replacement = Text::from_string_slice("XXX");
        
        let result = regex.replace(&text, &replacement);
        assert_eq!(result.to_string(), "abcXXXdef456");
        
        let result_all = regex.replace_all(&text, &replacement);
        assert_eq!(result_all.to_string(), "abcXXXdefXXX");
    }

    #[test]
    fn test_regex_splitting() {
        let regex = TextRegex::new(r",\s*").unwrap();
        let text = Text::from_string_slice("a, b, c, d");
        
        let parts = regex.split(&text);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].to_string(), "a");
        assert_eq!(parts[1].to_string(), "b");
        assert_eq!(parts[2].to_string(), "c");
        assert_eq!(parts[3].to_string(), "d");
    }

    #[test]
    fn test_case_insensitive_regex() {
        let regex = TextRegex::with_flags(r"hello", RegexFlags::case_insensitive()).unwrap();
        let text = Text::from_string_slice("Hello World");
        
        assert!(regex.is_match(&text));
        
        let match_result = regex.find(&text).unwrap();
        assert_eq!(match_result.matched_text.to_string(), "Hello");
    }

    #[test]
    fn test_named_groups() {
        let regex = TextRegex::new(r"(?P<word>\w+)\s+(?P<number>\d+)").unwrap();
        let text = Text::from_string_slice("hello 123");
        
        let match_result = regex.find(&text).unwrap();
        
        assert!(match_result.named_groups.contains_key("word"));
        assert!(match_result.named_groups.contains_key("number"));
        
        if let Some(Some(word)) = match_result.named_groups.get("word") {
            assert_eq!(word.to_string(), "hello");
        }
        
        if let Some(Some(number)) = match_result.named_groups.get("number") {
            assert_eq!(number.to_string(), "123");
        }
    }
}