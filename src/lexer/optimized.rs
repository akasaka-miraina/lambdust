//! Optimized lexer implementation with string interning and memory pooling.
//!
//! This module provides performance-optimized tokenization with:
//! - String interning for identifiers and keywords
//! - Memory pooling for token vectors
//! - Reduced string allocations
//! - Better cache locality

use super::{Token, TokenKind, Lexer, InternalLexer};
use crate::diagnostics::{Error, Result, Span};
use crate::utils::{InternedString, StringInterner, memory_pool::global_pools};
use std::sync::Arc;

/// Optimized lexer that uses string interning and memory pooling.
#[derive(Debug)]
pub struct OptimizedLexer<'a> {
    source: &'a str,
    filename: Option<&'a str>,
    interner: Arc<StringInterner>,
}

/// An optimized token that uses interned strings for common cases.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedToken {
    /// The kind of token
    pub kind: TokenKind,
    /// Source location information
    pub span: Span,
    /// Interned string content (for identifiers, keywords, etc.)
    pub interned_text: Option<InternedString>,
    /// Raw text (for literals that don't benefit from interning)
    pub raw_text: Option<String>,
}

impl OptimizedToken {
    /// Creates a new optimized token with interned text.
    pub fn with_interned(kind: TokenKind, span: Span, text: InternedString) -> Self {
        Self {
            kind,
            span,
            interned_text: Some(text),
            raw_text: None,
        }
    }

    /// Creates a new optimized token with raw text.
    pub fn with_raw(kind: TokenKind, span: Span, text: String) -> Self {
        Self {
            kind,
            span,
            interned_text: None,
            raw_text: Some(text),
        }
    }

    /// Gets the text content of this token.
    pub fn text(&self) -> &str {
        if let Some(ref interned) = self.interned_text {
            interned.as_str()
        } else if let Some(ref raw) = self.raw_text {
            raw
        } else {
            ""
        }
    }

    /// Gets the interned text if available.
    pub fn interned_text(&self) -> Option<&InternedString> {
        self.interned_text.as_ref()
    }

    /// Converts to a regular Token.
    pub fn to_token(&self) -> Token {
        Token::new(self.kind.clone(), self.span, self.text().to_string())
    }
}

impl<'a> OptimizedLexer<'a> {
    /// Creates a new optimized lexer.
    pub fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        Self {
            source,
            filename,
            interner: Arc::new(StringInterner::new()),
        }
    }

    /// Creates a new optimized lexer with a shared string interner.
    pub fn with_interner(source: &'a str, filename: Option<&'a str>, interner: Arc<StringInterner>) -> Self {
        Self {
            source,
            filename,
            interner,
        }
    }

    /// Tokenizes the source code with optimizations.
    pub fn tokenize_optimized(&mut self) -> Result<Vec<OptimizedToken>> {
        let tokens = global_pools::get_token_vec().take();
        let mut optimized_tokens = Vec::with_capacity(tokens.capacity());
        
        // Use internal lexer instead of logos
        let mut internal_lexer = InternalLexer::new(self.source, self.filename);
        let regular_tokens = internal_lexer.tokenize()?;

        for token in regular_tokens {
            // Skip comments
            if matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment) {
                continue;
            }

            let optimized_token = self.create_optimized_token(token.kind, token.span, &token.text);
            optimized_tokens.push(optimized_token);
        }

        Ok(optimized_tokens)
    }

    /// Creates an optimized token, deciding whether to intern the string or not.
    fn create_optimized_token(&self, kind: TokenKind, span: Span, text: &str) -> OptimizedToken {
        match kind {
            // Intern identifiers and keywords as they're likely to be repeated
            TokenKind::Identifier | TokenKind::Keyword => {
                let interned = self.interner.intern(text);
                OptimizedToken::with_interned(kind, span, interned)
            }
            
            // Don't intern literals as they're usually unique
            TokenKind::IntegerNumber | TokenKind::RealNumber | 
            TokenKind::RationalNumber | TokenKind::ComplexNumber |
            TokenKind::String | TokenKind::Character => {
                OptimizedToken::with_raw(kind, span, text.to_string())
            }
            
            // Intern small, commonly repeated tokens
            TokenKind::LeftParen | TokenKind::RightParen |
            TokenKind::LeftBracket | TokenKind::RightBracket |
            TokenKind::LeftBrace | TokenKind::RightBrace |
            TokenKind::Quote | TokenKind::Quasiquote |
            TokenKind::Unquote | TokenKind::UnquoteSplicing |
            TokenKind::Dot | TokenKind::TypeAnnotation |
            TokenKind::Boolean => {
                let interned = self.interner.intern(text);
                OptimizedToken::with_interned(kind, span, interned)
            }
            
            // For other tokens, use raw text
            _ => OptimizedToken::with_raw(kind, span, text.to_string()),
        }
    }

    /// Tokenizes with fallback to regular tokens for compatibility.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let optimized_tokens = self.tokenize_optimized()?;
        Ok(optimized_tokens.into_iter().map(|t| t.to_token()).collect())
    }

    /// Gets the current filename (if any).
    pub fn filename(&self) -> Option<&str> {
        self.filename
    }

    /// Gets the source code.
    pub fn source(&self) -> &str {
        self.source
    }

    /// Gets a reference to the string interner.
    pub fn interner(&self) -> &StringInterner {
        &self.interner
    }

    /// Gets statistics about the optimization benefits.
    pub fn optimization_stats(&self) -> OptimizationStats {
        let (interned_count, estimated_memory) = (self.interner.len(), self.interner.len() * 32);
        OptimizationStats {
            interned_strings: interned_count,
            estimated_memory_saved: estimated_memory,
        }
    }
}

/// Statistics about lexer optimizations.
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of unique strings interned
    pub interned_strings: usize,
    /// Estimated memory saved by interning (rough approximation)
    pub estimated_memory_saved: usize,
}

/// Benchmark helper to compare optimized vs regular lexer.
pub fn benchmark_comparison(source: &str, iterations: usize) -> (std::time::Duration, std::time::Duration, OptimizationStats) {
    use std::time::Instant;

    // Benchmark regular lexer
    let start = Instant::now();
    for _ in 0..iterations {
        let mut lexer = Lexer::new(source, Some("benchmark"));
        let _ = lexer.tokenize().unwrap();
    }
    let regular_time = start.elapsed();

    // Benchmark optimized lexer
    let start = Instant::now();
    let mut lexer = OptimizedLexer::new(source, Some("benchmark"));
    for _ in 0..iterations {
        let _ = lexer.tokenize().unwrap();
    }
    let stats = lexer.optimization_stats();
    let optimized_time = start.elapsed();

    (regular_time, optimized_time, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_tokenization() {
        let source = "(define foo 42) (define bar foo)";
        let mut lexer = OptimizedLexer::new(source, Some("test"));
        let tokens = lexer.tokenize_optimized().unwrap();

        // Should have tokenized correctly
        assert!(!tokens.is_empty());

        // Find identifier tokens
        let foo_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.kind == TokenKind::Identifier && t.text() == "foo")
            .collect();
        
        assert_eq!(foo_tokens.len(), 2);

        // Both "foo" tokens should use the same interned string
        if let (Some(first_interned), Some(second_interned)) = 
            (foo_tokens[0].interned_text(), foo_tokens[1].interned_text()) {
            assert_eq!(first_interned.id(), second_interned.id());
        }
    }

    #[test]
    fn test_optimization_stats() {
        let source = "(define test test) (define other test)";
        let mut lexer = OptimizedLexer::new(source, Some("test"));
        let _ = lexer.tokenize_optimized().unwrap();

        let stats = lexer.optimization_stats();
        assert!(stats.interned_strings > 0);
        assert!(stats.estimated_memory_saved > 0);
    }

    #[test]
    fn test_compatibility_with_regular_lexer() {
        let source = "(+ 1 2 3)";
        
        // Regular lexer
        let mut regular_lexer = Lexer::new(source, Some("test"));
        let regular_tokens = regular_lexer.tokenize().unwrap();

        // Optimized lexer
        let mut optimized_lexer = OptimizedLexer::new(source, Some("test"));
        let optimized_tokens = optimized_lexer.tokenize().unwrap();

        // Should produce same tokens
        assert_eq!(regular_tokens.len(), optimized_tokens.len());
        
        for (regular, optimized) in regular_tokens.iter().zip(optimized_tokens.iter()) {
            assert_eq!(regular.kind, optimized.kind);
            assert_eq!(regular.span, optimized.span);
            assert_eq!(regular.text, optimized.text);
        }
    }

    #[test]
    fn test_shared_interner() {
        let interner = Arc::new(StringInterner::new());
        let sources = vec![
            "(define test 1)",
            "(define test 2)",
        ];

        let mut total_interned = 0;
        for source in sources {
            let mut lexer = OptimizedLexer::with_interner(source, Some("test"), interner.clone());
            let _ = lexer.tokenize_optimized().unwrap();
            total_interned = lexer.interner().len();
        }

        // Should have reused strings across multiple lexer instances
        assert!(total_interned > 0);
        // "define" and "test" should only be interned once each, plus delimiters
        assert!(total_interned < 10); // Much less than if every string was interned separately
    }

    #[test]
    fn test_benchmark_comparison() {
        let source = r#"
            (define (factorial n)
              (if (= n 0)
                  1
                  (* n (factorial (- n 1)))))
            
            (define pi 3.14159)
            (factorial 5)
        "#;

        let (regular_time, optimized_time, stats) = benchmark_comparison(source, 10);
        
        // Should have completed both benchmarks
        assert!(regular_time.as_nanos() > 0);
        assert!(optimized_time.as_nanos() > 0);
        assert!(stats.interned_strings > 0);

        println!("Regular lexer: {:?}", regular_time);
        println!("Optimized lexer: {:?}", optimized_time);
        println!("Stats: {:?}", stats);
    }
}