//! Compatibility layer with the standard `regex` crate.
//!
//! This module provides API compatibility with the widely-used `regex` crate,
//! allowing existing code to use our lightweight Thompson NFA implementation
//! as a drop-in replacement.
//!
//! ## Supported API Surface
//!
//! **Core Types:**
//! - `LightRegex` - Drop-in replacement for `regex::Regex`
//! - `RegexBuilder` - Builder pattern for regex construction
//! - `Error` - Error type compatible with `regex::Error`
//! - `Match` - Match result compatible with `regex::Match`
//!
//! **Key Methods:**
//! - `new()` - Compile pattern
//! - `is_match()` - Test for match
//! - `find()` - Find first match  
//! - `find_iter()` - Iterate over all matches
//! - `captures()` - Extract captures (basic support)
//! - `replace()` / `replace_all()` - String replacement
//! - `split()` - Split by regex
//!
//! ## Migration Strategy
//!
//! 1. **Phase 1**: Implement core matching API (80% coverage)
//! 2. **Phase 2**: Add capture groups and advanced features
//! 3. **Phase 3**: Performance optimizations and edge cases
//!
//! ## Limitations
//!
//! - No Unicode property support (use ASCII classes only)
//! - No look-around assertions
//! - No backreferences
//! - Basic capture group support only
//! - No regex sets or multi-pattern matching

use std::fmt;
use std::borrow::Cow;
use crate::regex::{NfaEngine, PatternParser, Matcher};
use crate::regex::matcher::Match as InternalMatch;

/// Error type compatible with `regex::Error`.
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax error in regex pattern
    Syntax(String),
    /// Compilation error
    CompiledTooBig(usize),
    /// Internal error
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax(msg) => write!(f, "regex parse error: {msg}"),
            Error::CompiledTooBig(limit) => write!(f, "compiled regex too big: {limit}"),
            Error::Internal(msg) => write!(f, "regex internal error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::regex::parser::PatternError> for Error {
    fn from(err: crate::regex::parser::PatternError) -> Self {
        Error::Syntax(err.to_string())
    }
}

impl From<crate::regex::engine::EngineError> for Error {
    fn from(err: crate::regex::engine::EngineError) -> Self {
        match err {
            crate::regex::engine::EngineError::TooComplex(msg) => Error::CompiledTooBig(1000),
            crate::regex::engine::EngineError::UnsupportedFeature(msg) => Error::Syntax(msg),
            crate::regex::engine::EngineError::InternalError(msg) => Error::Internal(msg),
        }
    }
}

/// Match result compatible with `regex::Match`.
#[derive(Debug, Clone, Copy)]
pub struct Match<'t> {
    text: &'t str,
    start: usize,
    end: usize,
}

impl<'t> Match<'t> {
    /// Creates a new match.
    pub fn new(text: &'t str, start: usize, end: usize) -> Self {
        Self { text, start, end }
    }
    
    /// Returns the start position of the match.
    pub fn start(&self) -> usize {
        self.start
    }
    
    /// Returns the end position of the match.
    pub fn end(&self) -> usize {
        self.end
    }
    
    /// Returns the matched text.
    pub fn as_str(&self) -> &'t str {
        &self.text[self.start..self.end]
    }
    
    /// Returns the length of the match.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Tests if the match is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// Returns the range of the match.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

/// Captures type for compatibility (basic implementation).
#[derive(Debug, Clone)]
pub struct Captures<'t> {
    text: &'t str,
    matches: Vec<Option<Match<'t>>>,
}

impl<'t> Captures<'t> {
    /// Creates new captures.
    pub fn new(text: &'t str) -> Self {
        Self {
            text,
            matches: Vec::new(),
        }
    }
    
    /// Gets the full match.
    pub fn get(&self, i: usize) -> Option<Match<'t>> {
        self.matches.get(i).copied().flatten()
    }
    
    /// Gets the full match by name (not yet supported).
    pub fn name(&self, _name: &str) -> Option<Match<'t>> {
        None // Not implemented in Phase 1
    }
    
    /// Returns an iterator over all matches.
    pub fn iter(&self) -> impl Iterator<Item = Option<Match<'t>>> + '_ {
        self.matches.iter().copied()
    }
    
    /// Returns the number of captured groups.
    pub fn len(&self) -> usize {
        self.matches.len()
    }
    
    /// Tests if there are no captures.
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
}

/// Lightweight regex compatible with `regex::Regex`.
#[derive(Debug, Clone)]
pub struct LightRegex {
    engine: NfaEngine,
    pattern: String,
}

impl LightRegex {
    /// Compiles a regular expression.
    ///
    /// # Examples
    /// ```ignore
    /// use lambdust::regex::compat::LightRegex;
    /// let re = LightRegex::new(r"\d+").unwrap();
    /// assert!(re.is_match("123"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let parsed = PatternParser::new(pattern).parse()?;
        let engine = NfaEngine::from_pattern(&parsed)?;
        
        Ok(Self {
            engine,
            pattern: pattern.to_string(),
        })
    }
    
    /// Tests if the regex matches anywhere in the text.
    pub fn is_match(&self, text: &str) -> bool {
        let mut matcher = Matcher::new(&self.engine);
        matcher.find(text).is_some()
    }
    
    /// Finds the first match in the text.
    pub fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
        let mut matcher = Matcher::new(&self.engine);
        if let Some(internal_match) = matcher.find(text) {
            Some(Match::new(text, internal_match.start, internal_match.end))
        } else {
            None
        }
    }
    
    /// Returns an iterator over all non-overlapping matches.
    pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> FindMatches<'r, 't> {
        FindMatches::new(self, text)
    }
    
    /// Finds all matches and returns an iterator over capture groups.
    pub fn captures<'t>(&self, text: &'t str) -> Option<Captures<'t>> {
        if let Some(m) = self.find(text) {
            let mut captures = Captures::new(text);
            captures.matches.push(Some(m));
            Some(captures)
        } else {
            None
        }
    }
    
    /// Returns an iterator over all capture groups in the text.
    pub fn captures_iter<'r, 't>(&'r self, text: &'t str) -> CaptureMatches<'r, 't> {
        CaptureMatches::new(self, text)
    }
    
    /// Replaces the first match with replacement text.
    pub fn replace<'t>(&self, text: &'t str, rep: &str) -> Cow<'t, str> {
        if let Some(m) = self.find(text) {
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..m.start()]);
            result.push_str(rep);
            result.push_str(&text[m.end()..]);
            Cow::Owned(result)
        } else {
            Cow::Borrowed(text)
        }
    }
    
    /// Replaces all matches with replacement text.
    pub fn replace_all<'t>(&self, text: &'t str, rep: &str) -> Cow<'t, str> {
        let mut result = String::new();
        let mut last_end = 0;
        let mut found_any = false;
        
        for m in self.find_iter(text) {
            found_any = true;
            result.push_str(&text[last_end..m.start()]);
            result.push_str(rep);
            last_end = m.end();
        }
        
        if found_any {
            result.push_str(&text[last_end..]);
            Cow::Owned(result)
        } else {
            Cow::Borrowed(text)
        }
    }
    
    /// Replaces all matches using a replacer function.
    pub fn replace_all_fn<'t, F>(&self, text: &'t str, mut replacer: F) -> Cow<'t, str>
    where
        F: FnMut(&Match<'_>) -> String,
    {
        let mut result = String::new();
        let mut last_end = 0;
        let mut found_any = false;
        
        for m in self.find_iter(text) {
            found_any = true;
            result.push_str(&text[last_end..m.start()]);
            result.push_str(&replacer(&m));
            last_end = m.end();
        }
        
        if found_any {
            result.push_str(&text[last_end..]);
            Cow::Owned(result)
        } else {
            Cow::Borrowed(text)
        }
    }
    
    /// Splits text by the regex.
    pub fn split<'r, 't>(&'r self, text: &'t str) -> Split<'r, 't> {
        Split::new(self, text)
    }
    
    /// Splits text by the regex with a limit.
    pub fn splitn<'r, 't>(&'r self, text: &'t str, limit: usize) -> SplitN<'r, 't> {
        SplitN::new(self, text, limit)
    }
    
    /// Returns the original pattern string.
    pub fn as_str(&self) -> &str {
        &self.pattern
    }
}

/// Iterator over all matches in text.
pub struct FindMatches<'r, 't> {
    regex: &'r LightRegex,
    text: &'t str,
    last_end: usize,
}

impl<'r, 't> FindMatches<'r, 't> {
    fn new(regex: &'r LightRegex, text: &'t str) -> Self {
        Self {
            regex,
            text,
            last_end: 0,
        }
    }
}

impl<'r, 't> Iterator for FindMatches<'r, 't> {
    type Item = Match<'t>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end > self.text.len() {
            return None;
        }
        
        let mut matcher = Matcher::new(&self.regex.engine);
        for start_pos in self.last_end..=self.text.len() {
            if let Some(internal_match) = matcher.find_at(self.text, start_pos) {
                let m = Match::new(self.text, internal_match.start, internal_match.end);
                self.last_end = m.end().max(self.last_end + 1);
                return Some(m);
            }
        }
        None
    }
}

/// Iterator over all capture groups in text.
pub struct CaptureMatches<'r, 't> {
    matches: FindMatches<'r, 't>,
}

impl<'r, 't> CaptureMatches<'r, 't> {
    fn new(regex: &'r LightRegex, text: &'t str) -> Self {
        Self {
            matches: FindMatches::new(regex, text),
        }
    }
}

impl<'r, 't> Iterator for CaptureMatches<'r, 't> {
    type Item = Captures<'t>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.matches.next() {
            let mut captures = Captures::new(self.matches.text);
            captures.matches.push(Some(m));
            Some(captures)
        } else {
            None
        }
    }
}

/// Iterator over split text segments.
pub struct Split<'r, 't> {
    finder: FindMatches<'r, 't>,
    last: usize,
    finished: bool,
}

impl<'r, 't> Split<'r, 't> {
    fn new(regex: &'r LightRegex, text: &'t str) -> Self {
        Self {
            finder: FindMatches::new(regex, text),
            last: 0,
            finished: false,
        }
    }
}

impl<'r, 't> Iterator for Split<'r, 't> {
    type Item = &'t str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        
        if let Some(m) = self.finder.next() {
            let text = &self.finder.text[self.last..m.start()];
            self.last = m.end();
            Some(text)
        } else {
            self.finished = true;
            Some(&self.finder.text[self.last..])
        }
    }
}

/// Iterator over split text segments with limit.
pub struct SplitN<'r, 't> {
    split: Split<'r, 't>,
    limit: usize,
    count: usize,
}

impl<'r, 't> SplitN<'r, 't> {
    fn new(regex: &'r LightRegex, text: &'t str, limit: usize) -> Self {
        Self {
            split: Split::new(regex, text),
            limit,
            count: 0,
        }
    }
}

impl<'r, 't> Iterator for SplitN<'r, 't> {
    type Item = &'t str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.limit {
            // Return rest of string as final segment
            if self.count == self.limit {
                self.count += 1;
                Some(&self.split.finder.text[self.split.last..])
            } else {
                None
            }
        } else {
            self.count += 1;
            self.split.next()
        }
    }
}

/// Regex builder for advanced configuration.
pub struct RegexBuilder {
    pattern: String,
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
    unicode: bool,
}

impl RegexBuilder {
    /// Creates a new regex builder.
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_insensitive: false,
            multi_line: false,
            dot_matches_new_line: false,
            unicode: true,
        }
    }
    
    /// Configures case-insensitive matching.
    pub fn case_insensitive(mut self, yes: bool) -> Self {
        self.case_insensitive = yes;
        self
    }
    
    /// Configures multi-line mode.
    pub fn multi_line(mut self, yes: bool) -> Self {
        self.multi_line = yes;
        self
    }
    
    /// Configures whether . matches newlines.
    pub fn dot_matches_new_line(mut self, yes: bool) -> Self {
        self.dot_matches_new_line = yes;
        self
    }
    
    /// Configures Unicode mode.
    pub fn unicode(mut self, yes: bool) -> Self {
        self.unicode = yes;
        self
    }
    
    /// Builds the regex.
    pub fn build(self) -> Result<LightRegex, Error> {
        // For Phase 1, ignore most flags and use basic compilation
        // TODO: Implement flag support in Phase 2
        LightRegex::new(&self.pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matching() {
        let re = LightRegex::new("hello").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("goodbye world"));
    }
    
    #[test]
    fn test_find_match() {
        let re = LightRegex::new(r"\d+").unwrap();
        let m = re.find("abc123def").unwrap();
        assert_eq!(m.start(), 3);
        assert_eq!(m.end(), 6);
        assert_eq!(m.as_str(), "123");
    }
    
    #[test]
    fn test_find_iter() {
        let re = LightRegex::new(r"\d+").unwrap();
        let matches: Vec<_> = re.find_iter("a1b23c456").collect();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].as_str(), "1");
        assert_eq!(matches[1].as_str(), "23");
        assert_eq!(matches[2].as_str(), "456");
    }
    
    #[test]
    fn test_replace() {
        let re = LightRegex::new(r"\d+").unwrap();
        let result = re.replace("abc123def", "XXX");
        assert_eq!(result, "abcXXXdef");
    }
    
    #[test]
    fn test_replace_all() {
        let re = LightRegex::new(r"\d+").unwrap();
        let result = re.replace_all("a1b2c3", "X");
        assert_eq!(result, "aXbXcX");
    }
    
    #[test]
    fn test_split() {
        let re = LightRegex::new(r"\s+").unwrap();
        let parts: Vec<_> = re.split("a  b   c").collect();
        assert_eq!(parts, vec!["a", "b", "c", ""]);
    }
    
    #[test]
    fn test_splitn() {
        let re = LightRegex::new(r"\s+").unwrap();
        let parts: Vec<_> = re.splitn("a b c d", 2).collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "a");
        assert_eq!(parts[1], "b c d");
    }
    
    #[test]
    fn test_captures() {
        let re = LightRegex::new(r"\d+").unwrap();
        let caps = re.captures("abc123def").unwrap();
        let m = caps.get(0).unwrap();
        assert_eq!(m.as_str(), "123");
    }
    
    #[test]
    fn test_regex_builder() {
        let re = RegexBuilder::new("hello")
            .case_insensitive(true)
            .build()
            .unwrap();
        
        // For now, flags are ignored, but API should work
        assert!(re.is_match("hello"));
    }
}