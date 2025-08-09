//! Error suggestions and fix recommendations.

#![allow(dead_code)]

use super::{Span, Error};
use std::collections::HashMap;

/// Represents a suggestion for fixing an error.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// Description of the suggestion
    pub message: String,
    /// Optional replacement text
    pub replacement: Option<String>,
    /// Span to replace (if providing replacement text)
    pub span: Option<Span>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Category of suggestion
    pub category: SuggestionCategory,
}

/// Categories of error suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuggestionCategory {
    /// Syntax fix (typos, missing punctuation)
    Syntax,
    /// Semantic fix (wrong function, incorrect usage)
    Semantic,
    /// Style improvement
    Style,
    /// Performance optimization
    Performance,
    /// Best practice recommendation
    BestPractice,
}

impl Suggestion {
    /// Creates a new suggestion.
    pub fn new(message: impl Into<String>, category: SuggestionCategory) -> Self {
        Self {
            message: message.into(),
            replacement: None,
            span: None,
            confidence: 0.8,
            category,
        }
    }
    
    /// Creates a suggestion with replacement text.
    pub fn with_replacement(
        message: impl Into<String>,
        replacement: impl Into<String>,
        span: Span,
        category: SuggestionCategory,
    ) -> Self {
        Self {
            message: message.into(),
            replacement: Some(replacement.into()),
            span: Some(span),
            confidence: 0.9,
            category,
        }
    }
    
    /// Sets the confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Returns true if this suggestion has replacement text.
    pub fn has_replacement(&self) -> bool {
        self.replacement.is_some() && self.span.is_some()
    }
    
    /// Returns true if this is a high-confidence suggestion.
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.8
    }
}

/// Generator for creating error suggestions.
#[derive(Debug)]
pub struct SuggestionGenerator {
    /// Common typo corrections
    typo_corrections: HashMap<String, String>,
    /// Function name suggestions
    function_suggestions: HashMap<String, Vec<String>>,
    /// Syntax pattern fixes
    syntax_fixes: Vec<SyntaxFix>,
}

#[derive(Debug, Clone)]
struct SyntaxFix {
    pattern: String,
    replacement: String,
    message: String,
}

impl SuggestionGenerator {
    /// Creates a new suggestion generator with common patterns.
    pub fn new() -> Self {
        let mut generator = Self {
            typo_corrections: HashMap::new(),
            function_suggestions: HashMap::new(),
            syntax_fixes: Vec::new(),
        };
        
        generator.load_common_patterns();
        generator
    }
    
    /// Loads common error patterns and their fixes.
    fn load_common_patterns(&mut self) {
        // Common typos in Scheme/Lambdust
        self.typo_corrections.extend([
            ("defien".to_string(), "define".to_string()),
            ("lamda".to_string(), "lambda".to_string()),
            ("lenght".to_string(), "length".to_string()),
            ("consturct".to_string(), "construct".to_string()),
            ("beginn".to_string(), "begin".to_string()),
            ("conditon".to_string(), "condition".to_string()),
        ]);
        
        // Function name suggestions
        self.function_suggestions.extend([
            ("len".to_string(), vec!["length".to_string(), "string-length".to_string(), "vector-length".to_string()]),
            ("print".to_string(), vec!["display".to_string(), "write".to_string(), "newline".to_string()]),
            ("size".to_string(), vec!["length".to_string(), "vector-length".to_string()]),
            ("append".to_string(), vec!["string-append".to_string(), "vector-append".to_string()]),
            ("map".to_string(), vec!["map".to_string(), "for-each".to_string()]),
        ]);
        
        // Syntax fixes
        self.syntax_fixes.extend([
            SyntaxFix {
                pattern: "(".to_string(),
                replacement: "()".to_string(),
                message: "Add closing parenthesis".to_string(),
            },
            SyntaxFix {
                pattern: "\"".to_string(),
                replacement: "\"\"".to_string(),
                message: "Add closing quote".to_string(),
            },
        ]);
    }
    
    /// Generates suggestions for a lexical error.
    pub fn suggest_for_lex_error(&self, token: &str, span: Span) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Check for typos
        if let Some(correction) = self.typo_corrections.get(token) {
            suggestions.push(
                Suggestion::with_replacement(
                    format!("Did you mean '{correction}'?"),
                    correction.clone(),
                    span,
                    SuggestionCategory::Syntax,
                ).with_confidence(0.9)
            );
        }
        
        // Check for similar function names
        let similar = self.find_similar_names(token);
        for name in similar.into_iter().take(3) {
            suggestions.push(
                Suggestion::new(
                    format!("Did you mean '{name}'?"),
                    SuggestionCategory::Semantic,
                ).with_confidence(0.7)
            );
        }
        
        // Character-specific suggestions
        if token.len() == 1 {
            let ch = token.chars().next().unwrap();
            match ch {
                '@' => suggestions.push(
                    Suggestion::new(
                        "The '@' character is not valid in Lambdust. Use identifiers or keywords instead",
                        SuggestionCategory::Syntax,
                    )
                ),
                '#' => suggestions.push(
                    Suggestion::new(
                        "Use '#t' or '#f' for booleans, '#\\' for characters, or '#:' for keywords",
                        SuggestionCategory::Syntax,
                    )
                ),
                _ => {}
            }
        }
        
        suggestions
    }
    
    /// Generates suggestions for a parse error.
    pub fn suggest_for_parse_error(
        &self,
        message: &str,
        span: Span,
        expected: &Option<Vec<String>>,
        got: &Option<String>,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Handle missing closing parenthesis
        if message.contains("closing parenthesis") {
            suggestions.push(
                Suggestion::with_replacement(
                    "Add missing closing parenthesis",
                    ")".to_string(),
                    span,
                    SuggestionCategory::Syntax,
                ).with_confidence(0.95)
            );
        }
        
        // Handle unexpected tokens
        if message.contains("Unexpected") {
            if let Some(got_token) = got {
                match got_token.as_str() {
                    ")" => suggestions.push(
                        Suggestion::new(
                            "Remove extra closing parenthesis or add matching opening parenthesis",
                            SuggestionCategory::Syntax,
                        )
                    ),
                    "." => suggestions.push(
                        Suggestion::new(
                            "Dots are only valid in dotted pairs like (a . b)",
                            SuggestionCategory::Syntax,
                        )
                    ),
                    _ => {}
                }
            }
        }
        
        // Handle expected tokens
        if let Some(expected_tokens) = expected {
            if expected_tokens.len() == 1 {
                suggestions.push(
                    Suggestion::with_replacement(
                        format!("Add missing '{}'", expected_tokens[0]),
                        expected_tokens[0].clone(),
                        span,
                        SuggestionCategory::Syntax,
                    ).with_confidence(0.8)
                );
            } else if expected_tokens.len() <= 3 {
                let options = expected_tokens.join("', '");
                suggestions.push(
                    Suggestion::new(
                        format!("Expected one of: '{options}'"),
                        SuggestionCategory::Syntax,
                    )
                );
            }
        }
        
        suggestions
    }
    
    /// Generates suggestions for a runtime error.
    pub fn suggest_for_runtime_error(&self, message: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Common runtime error patterns
        if message.contains("unbound variable") || message.contains("undefined") {
            suggestions.push(
                Suggestion::new(
                    "Check if the variable is spelled correctly and defined in scope",
                    SuggestionCategory::Semantic,
                ).with_confidence(0.8)
            );
            
            suggestions.push(
                Suggestion::new(
                    "Use 'define' to bind a variable before using it",
                    SuggestionCategory::Semantic,
                ).with_confidence(0.7)
            );
        }
        
        if message.contains("not a procedure") || message.contains("cannot apply") {
            suggestions.push(
                Suggestion::new(
                    "Make sure the first element of the list is a function or procedure",
                    SuggestionCategory::Semantic,
                ).with_confidence(0.9)
            );
        }
        
        if message.contains("wrong number of arguments") {
            suggestions.push(
                Suggestion::new(
                    "Check the function documentation for the correct number of arguments",
                    SuggestionCategory::Semantic,
                ).with_confidence(0.8)
            );
        }
        
        if message.contains("division by zero") {
            suggestions.push(
                Suggestion::new(
                    "Add a check to ensure the divisor is not zero before division",
                    SuggestionCategory::BestPractice,
                ).with_confidence(0.9)
            );
        }
        
        if message.contains("type") && message.contains("mismatch") {
            suggestions.push(
                Suggestion::new(
                    "Check that all arguments are of the expected type",
                    SuggestionCategory::Semantic,
                ).with_confidence(0.8)
            );
        }
        
        suggestions
    }
    
    /// Generates suggestions for a type error.
    pub fn suggest_for_type_error(
        &self,
        expected_type: &Option<String>,
        actual_type: &Option<String>,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if let (Some(expected), Some(actual)) = (expected_type, actual_type) {
            // Specific type conversion suggestions
            match (expected.as_str(), actual.as_str()) {
                ("number", "string") => {
                    suggestions.push(
                        Suggestion::new(
                            "Use 'string->number' to convert string to number",
                            SuggestionCategory::Semantic,
                        ).with_confidence(0.9)
                    );
                }
                ("string", "number") => {
                    suggestions.push(
                        Suggestion::new(
                            "Use 'number->string' to convert number to string",
                            SuggestionCategory::Semantic,
                        ).with_confidence(0.9)
                    );
                }
                ("list", "vector") => {
                    suggestions.push(
                        Suggestion::new(
                            "Use 'vector->list' to convert vector to list",
                            SuggestionCategory::Semantic,
                        ).with_confidence(0.9)
                    );
                }
                ("vector", "list") => {
                    suggestions.push(
                        Suggestion::new(
                            "Use 'list->vector' to convert list to vector",
                            SuggestionCategory::Semantic,
                        ).with_confidence(0.9)
                    );
                }
                _ => {
                    suggestions.push(
                        Suggestion::new(
                            format!("Expected {expected}, but got {actual}. Check your types"),
                            SuggestionCategory::Semantic,
                        ).with_confidence(0.7)
                    );
                }
            }
        }
        
        suggestions
    }
    
    /// Finds similar names using basic string similarity.
    fn find_similar_names(&self, target: &str) -> Vec<String> {
        let mut similar = Vec::new();
        
        // Check function suggestions
        for (key, suggestions) in &self.function_suggestions {
            if self.is_similar(target, key) {
                similar.extend(suggestions.clone());
            }
        }
        
        // Check typo corrections
        for (typo, correction) in &self.typo_corrections {
            if self.is_similar(target, typo) {
                similar.push(correction.clone());
            }
        }
        
        // Remove duplicates and sort by similarity
        similar.sort();
        similar.dedup();
        similar
    }
    
    /// Basic string similarity check (can be improved with better algorithms).
    fn is_similar(&self, a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        
        // Simple edit distance heuristic
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return true;
        }
        
        let distance = self.edit_distance(a, b);
        let similarity = 1.0 - (distance as f64 / max_len as f64);
        
        similarity >= 0.6 // 60% similarity threshold
    }
    
    /// Calculates edit distance between two strings.
    fn edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();
        
        let mut dp = vec![vec![0; b_len + 1]; a_len + 1];
        
        // Initialize base cases
        for (i, row) in dp.iter_mut().enumerate() {
            row[0] = i;
        }
        for j in 0..=b_len {
            dp[0][j] = j;
        }
        
        // Fill the DP table
        for i in 1..=a_len {
            for j in 1..=b_len {
                if a_chars[i - 1] == b_chars[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
                }
            }
        }
        
        dp[a_len][b_len]
    }
    
    /// Adds a custom typo correction.
    pub fn add_typo_correction(&mut self, typo: String, correction: String) {
        self.typo_corrections.insert(typo, correction);
    }
    
    /// Adds function name suggestions.
    pub fn add_function_suggestions(&mut self, key: String, suggestions: Vec<String>) {
        self.function_suggestions.insert(key, suggestions);
    }
}

impl Default for SuggestionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to generate suggestions for any error.
pub fn generate_suggestions_for_error(error: &Error) -> Vec<Suggestion> {
    let generator = SuggestionGenerator::new();
    
    match error {
        Error::LexError { span, .. } => {
            // For lex errors, we need the actual token text
            // This would typically come from the lexer context
            generator.suggest_for_lex_error("", *span)
        }
        Error::ParseError { message, span } => {
            generator.suggest_for_parse_error(message, *span, &None, &None)
        }
        Error::RuntimeError { message, .. } => {
            generator.suggest_for_runtime_error(message)
        }
        Error::TypeError { .. } => {
            generator.suggest_for_type_error(&None, &None)
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    
    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new("Test suggestion", SuggestionCategory::Syntax);
        assert_eq!(suggestion.message, "Test suggestion");
        assert_eq!(suggestion.category, SuggestionCategory::Syntax);
        assert!(!suggestion.has_replacement());
        assert_eq!(suggestion.confidence, 0.8);
    }
    
    #[test]
    fn test_suggestion_with_replacement() {
        let span = Span::new(10, 5);
        let suggestion = Suggestion::with_replacement(
            "Fix typo",
            "correct",
            span,
            SuggestionCategory::Syntax,
        );
        
        assert!(suggestion.has_replacement());
        assert_eq!(suggestion.replacement, Some("correct".to_string()));
        assert_eq!(suggestion.span, Some(span));
        assert!(suggestion.is_high_confidence());
    }
    
    #[test]
    fn test_suggestion_generator_typos() {
        let generator = SuggestionGenerator::new();
        let span = Span::new(0, 6);
        
        let suggestions = generator.suggest_for_lex_error("defien", span);
        assert!(!suggestions.is_empty());
        
        let first = &suggestions[0];
        assert!(first.message.contains("define"));
        assert!(first.has_replacement());
        assert!(first.is_high_confidence());
    }
    
    #[test]
    fn test_suggestion_generator_parse_errors() {
        let generator = SuggestionGenerator::new();
        let span = Span::new(10, 1);
        
        let suggestions = generator.suggest_for_parse_error(
            "Expected closing parenthesis",
            span,
            &Some(vec![")".to_string()]),
            &Some("EOF".to_string()),
        );
        
        assert!(!suggestions.is_empty());
        let first = &suggestions[0];
        assert!(first.message.contains("closing parenthesis"));
    }
    
    #[test]
    fn test_suggestion_generator_runtime_errors() {
        let generator = SuggestionGenerator::new();
        
        let suggestions = generator.suggest_for_runtime_error("unbound variable: x");
        assert!(!suggestions.is_empty());
        
        let first = &suggestions[0];
        assert!(first.message.contains("variable"));
        assert_eq!(first.category, SuggestionCategory::Semantic);
    }
    
    #[test]
    fn test_suggestion_generator_type_errors() {
        let generator = SuggestionGenerator::new();
        
        let suggestions = generator.suggest_for_type_error(
            &Some("number".to_string()),
            &Some("string".to_string()),
        );
        
        assert!(!suggestions.is_empty());
        let first = &suggestions[0];
        assert!(first.message.contains("string->number"));
        assert!(first.is_high_confidence());
    }
    
    #[test]
    fn test_edit_distance() {
        let generator = SuggestionGenerator::new();
        
        assert_eq!(generator.edit_distance("", ""), 0);
        assert_eq!(generator.edit_distance("a", ""), 1);
        assert_eq!(generator.edit_distance("", "a"), 1);
        assert_eq!(generator.edit_distance("abc", "abc"), 0);
        assert_eq!(generator.edit_distance("abc", "ab"), 1);
        assert_eq!(generator.edit_distance("abc", "axc"), 1);
    }
    
    #[test]
    fn test_similarity_check() {
        let generator = SuggestionGenerator::new();
        
        assert!(generator.is_similar("define", "defien"));
        assert!(generator.is_similar("lambda", "lamda"));
        assert!(!generator.is_similar("define", "xyz"));
    }
    
    #[test]
    fn test_custom_corrections() {
        let mut generator = SuggestionGenerator::new();
        generator.add_typo_correction("custm".to_string(), "custom".to_string());
        
        let span = Span::new(0, 5);
        let suggestions = generator.suggest_for_lex_error("custm", span);
        
        let custom_suggestion = suggestions.iter()
            .find(|s| s.message.contains("custom"));
        assert!(custom_suggestion.is_some());
    }
}