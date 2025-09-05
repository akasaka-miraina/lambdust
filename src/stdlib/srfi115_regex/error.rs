//! Error handling for SRFI-115 regular expressions
//!
//! Comprehensive error types that cover all possible failure modes
//! in SRE parsing, compilation, and matching operations.

use crate::diagnostics::{Error, Result, Span};
use std::fmt;

/// SRFI-115 specific error types
#[derive(Debug, Clone)]
pub enum RegexError {
    /// Invalid SRE syntax or structure
    SyntaxError {
        message: String,
        position: Option<usize>,
        sre: String,
    },

    /// Invalid character class specification
    InvalidCharacterClass {
        class: String,
        reason: String,
    },

    /// Invalid Unicode property or script
    InvalidUnicodeProperty {
        property: String,
        reason: String,
    },

    /// Invalid quantifier specification
    InvalidQuantifier {
        min: Option<usize>,
        max: Option<usize>,
        reason: String,
    },

    /// Invalid backreference
    InvalidBackreference {
        index: usize,
        max_groups: usize,
    },

    /// Invalid named group reference
    InvalidNamedGroup {
        name: String,
        available_names: Vec<String>,
    },

    /// Compilation flags error
    InvalidFlag {
        flag: String,
    },

    /// Pattern too complex for compilation
    TooComplex {
        reason: String,
        suggestion: String,
    },

    /// Invalid range specification in match operation
    InvalidRange {
        start: usize,
        end: usize,
        text_len: usize,
    },

    /// Match object access error
    MatchAccessError {
        operation: String,
        reason: String,
    },

    /// Internal engine error
    InternalError {
        message: String,
        details: String,
    },

    /// Rust regex crate error (when using hybrid engine)
    RegexCrateError {
        source: String,
    },

    /// Unsupported SRE feature
    UnsupportedFeature {
        feature: String,
        alternative: Option<String>,
    },
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexError::SyntaxError { message, position, sre } => {
                if let Some(pos) = position {
                    write!(f, "SRE syntax error at position {}: {} in '{}'", pos, message, sre)
                } else {
                    write!(f, "SRE syntax error: {} in '{}'", message, sre)
                }
            }
            RegexError::InvalidCharacterClass { class, reason } => {
                write!(f, "Invalid character class '{}': {}", class, reason)
            }
            RegexError::InvalidUnicodeProperty { property, reason } => {
                write!(f, "Invalid Unicode property '{}': {}", property, reason)
            }
            RegexError::InvalidQuantifier { min, max, reason } => {
                match (min, max) {
                    (Some(min_val), Some(max_val)) => {
                        write!(f, "Invalid quantifier {{{},{}}}: {}", min_val, max_val, reason)
                    }
                    (Some(min_val), None) => {
                        write!(f, "Invalid quantifier {{{}+}}: {}", min_val, reason)
                    }
                    (None, Some(max_val)) => {
                        write!(f, "Invalid quantifier {{,{}}}: {}", max_val, reason)
                    }
                    (None, None) => {
                        write!(f, "Invalid quantifier: {}", reason)
                    }
                }
            }
            RegexError::InvalidBackreference { index, max_groups } => {
                write!(f, "Invalid backreference \\{}: only {} groups available", index, max_groups)
            }
            RegexError::InvalidNamedGroup { name, available_names } => {
                write!(f, "Invalid named group '{}': available names are [{}]", 
                       name, available_names.join(", "))
            }
            RegexError::InvalidFlag { flag } => {
                write!(f, "Invalid compilation flag: '{}'", flag)
            }
            RegexError::TooComplex { reason, suggestion } => {
                write!(f, "Pattern too complex: {}. Try: {}", reason, suggestion)
            }
            RegexError::InvalidRange { start, end, text_len } => {
                write!(f, "Invalid range [{}, {}] for text of length {}", start, end, text_len)
            }
            RegexError::MatchAccessError { operation, reason } => {
                write!(f, "Match access error in {}: {}", operation, reason)
            }
            RegexError::InternalError { message, details } => {
                write!(f, "Internal regex engine error: {} ({})", message, details)
            }
            RegexError::RegexCrateError { source } => {
                write!(f, "Rust regex error: {}", source)
            }
            RegexError::UnsupportedFeature { feature, alternative } => {
                if let Some(alt) = alternative {
                    write!(f, "Unsupported SRE feature '{}': use '{}' instead", feature, alt)
                } else {
                    write!(f, "Unsupported SRE feature '{}'", feature)
                }
            }
        }
    }
}

impl std::error::Error for RegexError {}

impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        match err {
            RegexError::SyntaxError { message, position, sre: _ } => {
                Error::runtime_error(
                    format!("SRFI-115 regex error: {}", message),
                    position.map(|pos| Span::new(pos, 1))
                )
            }
            _ => Error::runtime_error(
                format!("SRFI-115 regex error: {}", err),
                None
            )
        }
    }
}

/// Convenience type alias for SRFI-115 results
pub type RegexResult<T> = std::result::Result<T, RegexError>;

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub sre_text: String,
    pub position: Option<usize>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>, sre_text: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            sre_text: sre_text.into(),
            position: None,
        }
    }

    pub fn with_position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    pub fn syntax_error(self, message: impl Into<String>) -> RegexError {
        RegexError::SyntaxError {
            message: format!("{}: {}", self.operation, message.into()),
            position: self.position,
            sre: self.sre_text,
        }
    }

    pub fn invalid_character_class(self, class: impl Into<String>, reason: impl Into<String>) -> RegexError {
        RegexError::InvalidCharacterClass {
            class: class.into(),
            reason: format!("{}: {}", self.operation, reason.into()),
        }
    }
}

/// Helper trait for adding context to results
pub trait WithContext<T> {
    fn with_context(self, ctx: ErrorContext) -> RegexResult<T>;
    fn with_operation(self, operation: impl Into<String>) -> RegexResult<T>;
}

impl<T> WithContext<T> for RegexResult<T> {
    fn with_context(self, ctx: ErrorContext) -> RegexResult<T> {
        self.map_err(|err| match err {
            RegexError::SyntaxError { message, position, sre: _ } => {
                RegexError::SyntaxError {
                    message: format!("{}: {}", ctx.operation, message),
                    position: position.or(ctx.position),
                    sre: ctx.sre_text,
                }
            }
            other => other,
        })
    }

    fn with_operation(self, operation: impl Into<String>) -> RegexResult<T> {
        let op = operation.into();
        self.map_err(|err| match err {
            RegexError::SyntaxError { message, position, sre } => {
                RegexError::SyntaxError {
                    message: format!("{}: {}", op, message),
                    position,
                    sre,
                }
            }
            RegexError::InvalidCharacterClass { class, reason } => {
                RegexError::InvalidCharacterClass {
                    class,
                    reason: format!("{}: {}", op, reason),
                }
            }
            other => other,
        })
    }
}

/// Common error constructors
impl RegexError {
    /// Create a syntax error with context
    pub fn syntax(message: impl Into<String>, sre: impl Into<String>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            position: None,
            sre: sre.into(),
        }
    }

    /// Create a syntax error with position
    pub fn syntax_at(
        message: impl Into<String>, 
        position: usize, 
        sre: impl Into<String>
    ) -> Self {
        Self::SyntaxError {
            message: message.into(),
            position: Some(position),
            sre: sre.into(),
        }
    }

    /// Create an invalid character class error
    pub fn invalid_class(class: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCharacterClass {
            class: class.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid Unicode property error
    pub fn invalid_unicode(property: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidUnicodeProperty {
            property: property.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid quantifier error
    pub fn invalid_quantifier(
        min: Option<usize>, 
        max: Option<usize>, 
        reason: impl Into<String>
    ) -> Self {
        Self::InvalidQuantifier {
            min,
            max,
            reason: reason.into(),
        }
    }

    /// Create a too complex error
    pub fn too_complex(reason: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::TooComplex {
            reason: reason.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create an unsupported feature error
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
            alternative: None,
        }
    }

    /// Create an unsupported feature error with alternative
    pub fn unsupported_with_alt(
        feature: impl Into<String>, 
        alternative: impl Into<String>
    ) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
            alternative: Some(alternative.into()),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>, details: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            details: details.into(),
        }
    }

    /// Convert from regex crate error
    pub fn from_regex_error(err: regex::Error) -> Self {
        Self::RegexCrateError {
            source: err.to_string(),
        }
    }
}

/// Helper trait for converting regex crate errors
pub trait FromRegexError<T> {
    fn map_regex_error(self) -> RegexResult<T>;
}

impl<T> FromRegexError<T> for std::result::Result<T, regex::Error> {
    fn map_regex_error(self) -> RegexResult<T> {
        self.map_err(RegexError::from_regex_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error_display() {
        let err = RegexError::syntax_at("unexpected character", 5, "(foo bar)");
        let display = format!("{}", err);
        assert!(display.contains("position 5"));
        assert!(display.contains("unexpected character"));
        assert!(display.contains("(foo bar)"));
    }

    #[test]
    fn test_character_class_error() {
        let err = RegexError::invalid_class("[:invalid:]", "unknown POSIX class");
        let display = format!("{}", err);
        assert!(display.contains("Invalid character class"));
        assert!(display.contains("[:invalid:]"));
    }

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new("parsing alternation", "(or foo bar)")
            .with_position(10);
        
        let err = ctx.syntax_error("missing closing paren");
        
        if let RegexError::SyntaxError { message, position, sre } = err {
            assert!(message.contains("parsing alternation"));
            assert_eq!(position, Some(10));
            assert_eq!(sre, "(or foo bar)");
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_with_context_trait() {
        let result: RegexResult<()> = Err(RegexError::syntax("test", "pattern"));
        let ctx = ErrorContext::new("test operation", "test pattern");
        
        let contextual_result = result.with_context(ctx);
        assert!(contextual_result.is_err());
        
        if let Err(RegexError::SyntaxError { message, .. }) = contextual_result {
            assert!(message.contains("test operation"));
        }
    }

    #[test]
    fn test_quantifier_error_display() {
        let err = RegexError::invalid_quantifier(Some(5), Some(3), "min > max");
        let display = format!("{}", err);
        assert!(display.contains("{5,3}"));
        assert!(display.contains("min > max"));
    }

    #[test]
    fn test_unicode_property_error() {
        let err = RegexError::invalid_unicode("InvalidScript", "not a valid Unicode script");
        let display = format!("{}", err);
        assert!(display.contains("Unicode property"));
        assert!(display.contains("InvalidScript"));
    }
}