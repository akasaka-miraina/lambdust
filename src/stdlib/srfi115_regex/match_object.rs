//! SRFI-115 Match Objects
//!
//! Rich match objects providing access to match results, submatches,
//! named groups, and context information according to SRFI-115 specification.

use crate::stdlib::srfi115_regex::{
    error::{RegexError, RegexResult},
};
use std::collections::HashMap;

/// Information about a submatch (capture group)
#[derive(Debug, Clone, PartialEq)]
pub struct SubmatchInfo {
    /// Start position in original string (0-based)
    pub start: usize,
    /// End position in original string (exclusive)
    pub end: usize,
    /// The matched text
    pub text: String,
}

impl SubmatchInfo {
    /// Create new submatch info
    pub fn new(start: usize, end: usize, text: String) -> Self {
        Self { start, end, text }
    }

    /// Length of the matched text
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if submatch is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get the matched text as a string slice
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// SRFI-115 match object containing match results and context
#[derive(Debug, Clone)]
pub struct MatchObject {
    /// Start position of main match (0-based)
    start: usize,
    /// End position of main match (exclusive)
    end: usize,
    /// The matched text
    matched_text: String,
    /// The complete input string
    input_text: String,
    /// Information about submatches (1-based indexing per SRFI-115)
    /// Index 0 represents the main match, indices 1+ are capture groups
    submatches: Vec<Option<SubmatchInfo>>,
    /// Named group mappings (name -> group index)
    named_groups: HashMap<String, usize>,
}

impl MatchObject {
    /// Create a new match object
    pub fn new(
        start: usize,
        end: usize,
        matched_text: String,
        input_text: String,
        submatches: Vec<Option<SubmatchInfo>>,
        named_groups: HashMap<String, usize>,
    ) -> Self {
        Self {
            start,
            end,
            matched_text,
            input_text,
            submatches,
            named_groups,
        }
    }

    /// Get the start position of the main match
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the end position of the main match
    pub fn end(&self) -> usize {
        self.end
    }

    /// Get the length of the main match
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if main match is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the matched text
    pub fn matched_text(&self) -> &str {
        &self.matched_text
    }

    /// Get the complete input text
    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    /// Get the number of capture groups (not including main match)
    pub fn group_count(&self) -> usize {
        self.submatches.len()
    }

    /// Get submatch by index (1-based indexing per SRFI-115)
    /// Index 0 returns the main match, indices 1+ return capture groups
    pub fn submatch(&self, index: usize) -> RegexResult<Option<SubmatchInfo>> {
        if index == 0 {
            // Main match - create SubmatchInfo on demand
            Ok(Some(SubmatchInfo {
                start: self.start,
                end: self.end,
                text: self.matched_text.clone(),
            }))
        } else if index <= self.submatches.len() {
            Ok(self.submatches[index - 1].clone())
        } else {
            Err(RegexError::MatchAccessError {
                operation: "submatch access".to_string(),
                reason: format!(
                    "submatch index {} out of range (0-{})",
                    index,
                    self.submatches.len()
                ),
            })
        }
    }

    /// Get submatch start position by index (1-based)
    pub fn submatch_start(&self, index: usize) -> RegexResult<Option<usize>> {
        if index == 0 {
            Ok(Some(self.start))
        } else if index <= self.submatches.len() {
            Ok(self.submatches[index - 1].as_ref().map(|s| s.start))
        } else {
            Err(RegexError::MatchAccessError {
                operation: "submatch start access".to_string(),
                reason: format!(
                    "submatch index {} out of range (0-{})",
                    index,
                    self.submatches.len()
                ),
            })
        }
    }

    /// Get submatch end position by index (1-based)
    pub fn submatch_end(&self, index: usize) -> RegexResult<Option<usize>> {
        if index == 0 {
            Ok(Some(self.end))
        } else if index <= self.submatches.len() {
            Ok(self.submatches[index - 1].as_ref().map(|s| s.end))
        } else {
            Err(RegexError::MatchAccessError {
                operation: "submatch end access".to_string(),
                reason: format!(
                    "submatch index {} out of range (0-{})",
                    index,
                    self.submatches.len()
                ),
            })
        }
    }

    /// Get submatch text by index (1-based)
    pub fn submatch_text(&self, index: usize) -> RegexResult<Option<String>> {
        if index == 0 {
            Ok(Some(self.matched_text.clone()))
        } else if index <= self.submatches.len() {
            Ok(self.submatches[index - 1]
                .as_ref()
                .map(|s| s.text.clone()))
        } else {
            Err(RegexError::MatchAccessError {
                operation: "submatch text access".to_string(),
                reason: format!(
                    "submatch index {} out of range (0-{})",
                    index,
                    self.submatches.len()
                ),
            })
        }
    }

    /// Get named submatch by name
    pub fn named_submatch(&self, name: &str) -> RegexResult<Option<SubmatchInfo>> {
        if let Some(&group_index) = self.named_groups.get(name) {
            if group_index == 0 {
                // This shouldn't happen - named groups should be >= 1
                Err(RegexError::MatchAccessError {
                    operation: "named submatch access".to_string(),
                    reason: "invalid group index 0 for named group".to_string(),
                })
            } else {
                Ok(self.submatches.get(group_index - 1).and_then(|opt| opt.clone()))
            }
        } else {
            Err(RegexError::InvalidNamedGroup {
                name: name.to_string(),
                available_names: self.named_groups.keys().cloned().collect(),
            })
        }
    }

    /// Get named submatch text by name
    pub fn named_submatch_text(&self, name: &str) -> RegexResult<Option<String>> {
        self.named_submatch(name)
            .map(|opt| opt.map(|s| s.text))
    }

    /// Get named submatch start position by name
    pub fn named_submatch_start(&self, name: &str) -> RegexResult<Option<usize>> {
        self.named_submatch(name)
            .map(|opt| opt.map(|s| s.start))
    }

    /// Get named submatch end position by name
    pub fn named_submatch_end(&self, name: &str) -> RegexResult<Option<usize>> {
        self.named_submatch(name)
            .map(|opt| opt.map(|s| s.end))
    }

    /// Get all named group names
    pub fn named_group_names(&self) -> Vec<String> {
        self.named_groups.keys().cloned().collect()
    }

    /// Get the text before the match
    pub fn pre_match(&self) -> &str {
        &self.input_text[..self.start]
    }

    /// Get the text after the match
    pub fn post_match(&self) -> &str {
        &self.input_text[self.end..]
    }

    /// Check if a submatch exists (captured something)
    pub fn submatch_exists(&self, index: usize) -> bool {
        if index == 0 {
            true // Main match always exists
        } else if index <= self.submatches.len() {
            self.submatches[index - 1].is_some()
        } else {
            false
        }
    }

    /// Check if a named submatch exists
    pub fn named_submatch_exists(&self, name: &str) -> bool {
        self.named_groups.contains_key(name)
            && self.named_groups.get(name)
                .map(|&idx| idx <= self.submatches.len() && self.submatches[idx - 1].is_some())
                .unwrap_or(false)
    }

    /// Get all submatch texts as a vector (including main match at index 0)
    pub fn all_submatches(&self) -> Vec<Option<String>> {
        let mut result = vec![Some(self.matched_text.clone())];
        result.extend(
            self.submatches
                .iter()
                .map(|opt| opt.as_ref().map(|s| s.text.clone())),
        );
        result
    }

    /// Get match context information for debugging
    pub fn debug_info(&self) -> MatchDebugInfo {
        MatchDebugInfo {
            main_match: SubmatchInfo {
                start: self.start,
                end: self.end,
                text: self.matched_text.clone(),
            },
            total_submatches: self.submatches.len(),
            active_submatches: self.submatches.iter().filter(|s| s.is_some()).count(),
            named_groups: self.named_groups.clone(),
            input_length: self.input_text.len(),
            pre_match_length: self.start,
            post_match_length: self.input_text.len() - self.end,
        }
    }
}

/// Debug information for match objects
#[derive(Debug, Clone)]
pub struct MatchDebugInfo {
    /// Information about the main match
    pub main_match: SubmatchInfo,
    /// Total number of capture groups
    pub total_submatches: usize,
    /// Number of capture groups that actually matched
    pub active_submatches: usize,
    /// Named group mappings
    pub named_groups: HashMap<String, usize>,
    /// Length of input text
    pub input_length: usize,
    /// Length of text before match
    pub pre_match_length: usize,
    /// Length of text after match
    pub post_match_length: usize,
}

/// Builder for creating match objects incrementally
pub struct MatchObjectBuilder {
    start: Option<usize>,
    end: Option<usize>,
    matched_text: Option<String>,
    input_text: String,
    submatches: Vec<Option<SubmatchInfo>>,
    named_groups: HashMap<String, usize>,
}

impl MatchObjectBuilder {
    /// Create a new builder with input text
    pub fn new(input_text: String) -> Self {
        Self {
            start: None,
            end: None,
            matched_text: None,
            input_text,
            submatches: Vec::new(),
            named_groups: HashMap::new(),
        }
    }

    /// Set the main match bounds
    pub fn set_main_match(mut self, start: usize, end: usize) -> RegexResult<Self> {
        if start > end {
            return Err(RegexError::InvalidRange {
                start,
                end,
                text_len: self.input_text.len(),
            });
        }
        if end > self.input_text.len() {
            return Err(RegexError::InvalidRange {
                start,
                end,
                text_len: self.input_text.len(),
            });
        }

        self.start = Some(start);
        self.end = Some(end);
        self.matched_text = Some(self.input_text[start..end].to_string());
        Ok(self)
    }

    /// Add a submatch (capture group)
    pub fn add_submatch(mut self, start: usize, end: usize) -> RegexResult<Self> {
        if start > end {
            return Err(RegexError::InvalidRange {
                start,
                end,
                text_len: self.input_text.len(),
            });
        }
        if end > self.input_text.len() {
            return Err(RegexError::InvalidRange {
                start,
                end,
                text_len: self.input_text.len(),
            });
        }

        let submatch = SubmatchInfo {
            start,
            end,
            text: self.input_text[start..end].to_string(),
        };
        self.submatches.push(Some(submatch));
        Ok(self)
    }

    /// Add an empty submatch (group that didn't capture)
    pub fn add_empty_submatch(mut self) -> Self {
        self.submatches.push(None);
        self
    }

    /// Add a named group mapping
    pub fn add_named_group(mut self, name: String, group_index: usize) -> RegexResult<Self> {
        if group_index == 0 {
            return Err(RegexError::MatchAccessError {
                operation: "add named group".to_string(),
                reason: "named groups cannot have index 0".to_string(),
            });
        }

        if self.named_groups.contains_key(&name) {
            return Err(RegexError::MatchAccessError {
                operation: "add named group".to_string(),
                reason: format!("named group '{}' already exists", name),
            });
        }

        self.named_groups.insert(name, group_index);
        Ok(self)
    }

    /// Reserve space for the expected number of submatches
    pub fn reserve_submatches(mut self, count: usize) -> Self {
        self.submatches.reserve(count);
        self
    }

    /// Build the final match object
    pub fn build(self) -> RegexResult<MatchObject> {
        let start = self.start.ok_or_else(|| {
            RegexError::MatchAccessError {
                operation: "build match object".to_string(),
                reason: "main match start position not set".to_string(),
            }
        })?;

        let end = self.end.ok_or_else(|| {
            RegexError::MatchAccessError {
                operation: "build match object".to_string(),
                reason: "main match end position not set".to_string(),
            }
        })?;

        let matched_text = self.matched_text.ok_or_else(|| {
            RegexError::MatchAccessError {
                operation: "build match object".to_string(),
                reason: "matched text not set".to_string(),
            }
        })?;

        // Validate named groups point to valid submatch indices
        for (name, &index) in &self.named_groups {
            if index > self.submatches.len() {
                return Err(RegexError::MatchAccessError {
                    operation: "build match object".to_string(),
                    reason: format!(
                        "named group '{}' references invalid submatch index {}",
                        name, index
                    ),
                });
            }
        }

        Ok(MatchObject::new(
            start,
            end,
            matched_text,
            self.input_text,
            self.submatches,
            self.named_groups,
        ))
    }
}

impl PartialEq for MatchObject {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.end == other.end
            && self.matched_text == other.matched_text
            && self.input_text == other.input_text
            && self.submatches == other.submatches
            && self.named_groups == other.named_groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submatch_info_creation() {
        let submatch = SubmatchInfo::new(5, 10, "hello".to_string());
        assert_eq!(submatch.start, 5);
        assert_eq!(submatch.end, 10);
        assert_eq!(submatch.text, "hello");
        assert_eq!(submatch.len(), 5);
        assert!(!submatch.is_empty());
        assert_eq!(submatch.as_str(), "hello");
    }

    #[test]
    fn test_match_object_basic() {
        let match_obj = MatchObject::new(
            5,
            10,
            "hello".to_string(),
            "test hello world".to_string(),
            vec![],
            HashMap::new(),
        );

        assert_eq!(match_obj.start(), 5);
        assert_eq!(match_obj.end(), 10);
        assert_eq!(match_obj.len(), 5);
        assert!(!match_obj.is_empty());
        assert_eq!(match_obj.matched_text(), "hello");
        assert_eq!(match_obj.input_text(), "test hello world");
        assert_eq!(match_obj.group_count(), 0);
    }

    #[test]
    fn test_match_object_submatches() {
        let submatches = vec![
            Some(SubmatchInfo::new(0, 4, "test".to_string())),
            Some(SubmatchInfo::new(11, 16, "world".to_string())),
            None, // Empty group
        ];

        let match_obj = MatchObject::new(
            0,
            16,
            "test hello world".to_string(),
            "test hello world".to_string(),
            submatches,
            HashMap::new(),
        );

        assert_eq!(match_obj.group_count(), 3);

        // Main match (index 0)
        assert!(match_obj.submatch_exists(0));
        let main_match = match_obj.submatch(0).unwrap().unwrap();
        assert_eq!(main_match.text, "test hello world");

        // First submatch (index 1)
        assert!(match_obj.submatch_exists(1));
        let sub1 = match_obj.submatch(1).unwrap().unwrap();
        assert_eq!(sub1.text, "test");

        // Second submatch (index 2)
        assert!(match_obj.submatch_exists(2));
        let sub2 = match_obj.submatch(2).unwrap().unwrap();
        assert_eq!(sub2.text, "world");

        // Third submatch (index 3) - empty
        assert!(!match_obj.submatch_exists(3));
        assert!(match_obj.submatch(3).unwrap().is_none());

        // Out of range
        assert!(!match_obj.submatch_exists(4));
        assert!(match_obj.submatch(4).is_err());
    }

    #[test]
    fn test_match_object_named_groups() {
        let submatches = vec![
            Some(SubmatchInfo::new(0, 4, "test".to_string())),
            Some(SubmatchInfo::new(11, 16, "world".to_string())),
        ];

        let mut named_groups = HashMap::new();
        named_groups.insert("first".to_string(), 1);
        named_groups.insert("second".to_string(), 2);

        let match_obj = MatchObject::new(
            0,
            16,
            "test hello world".to_string(),
            "test hello world".to_string(),
            submatches,
            named_groups,
        );

        // Test named group access
        assert!(match_obj.named_submatch_exists("first"));
        assert!(match_obj.named_submatch_exists("second"));
        assert!(!match_obj.named_submatch_exists("third"));

        let first = match_obj.named_submatch("first").unwrap().unwrap();
        assert_eq!(first.text, "test");

        let second = match_obj.named_submatch("second").unwrap().unwrap();
        assert_eq!(second.text, "world");

        // Test named group text access
        assert_eq!(
            match_obj.named_submatch_text("first").unwrap().unwrap(),
            "test"
        );
        assert_eq!(
            match_obj.named_submatch_text("second").unwrap().unwrap(),
            "world"
        );

        // Test invalid named group
        assert!(match_obj.named_submatch("invalid").is_err());

        // Test named group names
        let names = match_obj.named_group_names();
        assert!(names.contains(&"first".to_string()));
        assert!(names.contains(&"second".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_match_object_context() {
        let match_obj = MatchObject::new(
            5,
            10,
            "hello".to_string(),
            "test hello world".to_string(),
            vec![],
            HashMap::new(),
        );

        assert_eq!(match_obj.pre_match(), "test ");
        assert_eq!(match_obj.post_match(), " world");
    }

    #[test]
    fn test_match_object_builder() {
        let builder = MatchObjectBuilder::new("test hello world".to_string())
            .set_main_match(5, 10)
            .unwrap()
            .add_submatch(0, 4)
            .unwrap()
            .add_submatch(11, 16)
            .unwrap()
            .add_empty_submatch()
            .add_named_group("first".to_string(), 1)
            .unwrap()
            .add_named_group("second".to_string(), 2)
            .unwrap();

        let match_obj = builder.build().unwrap();

        assert_eq!(match_obj.matched_text(), "hello");
        assert_eq!(match_obj.group_count(), 3);
        assert!(match_obj.submatch_exists(1));
        assert!(match_obj.submatch_exists(2));
        assert!(!match_obj.submatch_exists(3));
        assert!(match_obj.named_submatch_exists("first"));
        assert!(match_obj.named_submatch_exists("second"));
    }

    #[test]
    fn test_match_object_builder_validation() {
        // Test invalid range
        let result = MatchObjectBuilder::new("test".to_string())
            .set_main_match(10, 5);
        assert!(result.is_err());

        // Test out of bounds range
        let result = MatchObjectBuilder::new("test".to_string())
            .set_main_match(0, 10);
        assert!(result.is_err());

        // Test building without setting main match
        let result = MatchObjectBuilder::new("test".to_string()).build();
        assert!(result.is_err());

        // Test invalid named group index
        let result = MatchObjectBuilder::new("test".to_string())
            .add_named_group("test".to_string(), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_info() {
        let submatches = vec![
            Some(SubmatchInfo::new(0, 4, "test".to_string())),
            None,
            Some(SubmatchInfo::new(11, 16, "world".to_string())),
        ];

        let match_obj = MatchObject::new(
            5,
            10,
            "hello".to_string(),
            "test hello world".to_string(),
            submatches,
            HashMap::new(),
        );

        let debug_info = match_obj.debug_info();
        assert_eq!(debug_info.main_match.text, "hello");
        assert_eq!(debug_info.total_submatches, 3);
        assert_eq!(debug_info.active_submatches, 2);
        assert_eq!(debug_info.input_length, 16);
        assert_eq!(debug_info.pre_match_length, 5);
        assert_eq!(debug_info.post_match_length, 6);
    }

    #[test]
    fn test_all_submatches() {
        let submatches = vec![
            Some(SubmatchInfo::new(0, 4, "test".to_string())),
            None,
            Some(SubmatchInfo::new(11, 16, "world".to_string())),
        ];

        let match_obj = MatchObject::new(
            5,
            10,
            "hello".to_string(),
            "test hello world".to_string(),
            submatches,
            HashMap::new(),
        );

        let all = match_obj.all_submatches();
        assert_eq!(all.len(), 4); // main + 3 submatches
        assert_eq!(all[0].as_ref().unwrap(), "hello"); // main match
        assert_eq!(all[1].as_ref().unwrap(), "test");  // submatch 1
        assert!(all[2].is_none());                      // submatch 2 (empty)
        assert_eq!(all[3].as_ref().unwrap(), "world");  // submatch 3
    }
}