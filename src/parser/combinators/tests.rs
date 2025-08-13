//! Unit tests for parser combinators
//!
//! This module contains comprehensive unit tests for all parser combinators,
//! ensuring correctness, performance, and R7RS compliance.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::combinators::{
        primitive,
        combinator::ParserCombinator,
        scheme,
        types::*,
    };
    
    // Helper functions from the primitive module
    use primitive::{char, tag, digit, satisfy};
    // Helper functions from the scheme module  
    use scheme::{scheme_number, scheme_string, scheme_character, scheme_symbol, scheme_sexp};
    // Helper functions from the combinator module
    use combinator::{whitespace0, whitespace1};

    /// Test basic character parsing
    #[test]
    fn test_char_parsing() {
        let parser = char('a');
        
        // Successful parsing
        match parser.parse("abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "bc");
                assert_eq!(parsed, 'a');
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Failed parsing - wrong character
        match parser.parse("xyz") {
            Ok(_) => panic!("Expected parse error for wrong character"),
            Err(e) => {
                assert_eq!(e.actual, "x".to_string());
                assert!(e.expected.contains(&"a".to_string()));
            }
        }
        
        // Failed parsing - empty input
        match parser.parse("") {
            Ok(_) => panic!("Expected parse error for empty input"),
            Err(e) => {
                assert_eq!(e.actual, "EOF".to_string());
                assert!(e.expected.contains(&"a".to_string()));
            }
        }
    }

    /// Test string tag parsing
    #[test]
    fn test_tag_parsing() {
        let parser = tag("hello");
        
        // Successful parsing
        match parser.parse("hello world") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " world");
                assert_eq!(parsed, "hello");
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Failed parsing - partial match
        match parser.parse("help") {
            Ok(_) => panic!("Expected parse error for partial match"),
            Err(e) => {
                assert_eq!(e.actual, "help".to_string());
                assert!(e.expected.contains(&"hello".to_string()));
            }
        }
    }

    /// Test digit parsing
    #[test]
    fn test_digit_parsing() {
        let parser = digit();
        
        // Successful parsing
        match parser.parse("5abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "abc");
                assert_eq!(parsed, '5');
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Failed parsing - non-digit
        match parser.parse("abc") {
            Ok(_) => panic!("Expected parse error for non-digit"),
            Err(e) => {
                assert_eq!(e.actual, "a".to_string());
                assert!(e.expected.contains(&"digit".to_string()));
            }
        }
    }

    /// Test satisfy predicate parsing
    #[test]
    fn test_satisfy_parsing() {
        let parser = satisfy(|c: &char| c.is_alphabetic());
        
        // Successful parsing
        match parser.parse("abc123") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "bc123");
                assert_eq!(parsed, 'a');
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Failed parsing - predicate fails
        match parser.parse("123abc") {
            Ok(_) => panic!("Expected parse error when predicate fails"),
            Err(e) => {
                assert_eq!(e.actual, "1".to_string());
                assert!(e.message.contains("predicate"));
            }
        }
    }

    /// Test parser mapping
    #[test]
    fn test_map_combinator() {
        let parser = digit().map(|c| c.to_digit(10).unwrap() as i32);
        
        match parser.parse("7xyz") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "xyz");
                assert_eq!(parsed, 7);
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
    }

    /// Test parser alternatives (or combinator)
    #[test]
    fn test_or_combinator() {
        let parser = char('a').or(char('b'));
        
        // First alternative succeeds
        match parser.parse("abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "bc");
                assert_eq!(parsed, 'a');
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Second alternative succeeds
        match parser.parse("bac") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "ac");
                assert_eq!(parsed, 'b');
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Both alternatives fail
        match parser.parse("xyz") {
            Ok(_) => panic!("Expected parse error when both alternatives fail"),
            Err(e) => {
                assert_eq!(e.actual, "x".to_string());
                assert!(e.expected.len() >= 1); // Should have multiple expected values
            }
        }
    }

    /// Test many combinator (zero or more)
    #[test]
    fn test_many_combinator() {
        let parser = digit().many();
        
        // Multiple matches
        match parser.parse("123abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "abc");
                assert_eq!(parsed, vec!['1', '2', '3']);
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Zero matches (should still succeed)
        match parser.parse("abc123") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "abc123");
                assert_eq!(parsed, Vec::<char>::new());
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
    }

    /// Test many1 combinator (one or more)
    #[test]
    fn test_many1_combinator() {
        let parser = digit().many1();
        
        // Multiple matches
        match parser.parse("123abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "abc");
                assert_eq!(parsed, vec!['1', '2', '3']);
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Zero matches (should fail)
        match parser.parse("abc123") {
            Ok(_) => panic!("Expected parse error when no matches for many1"),
            Err(e) => {
                assert!(e.message.contains("at least one"));
            }
        }
    }

    /// Test optional combinator
    #[test]
    fn test_optional_combinator() {
        let parser = char('a').optional();
        
        // Present value
        match parser.parse("abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "bc");
                assert_eq!(parsed, Some('a'));
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
        
        // Absent value (should still succeed with None)
        match parser.parse("xyz") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "xyz");
                assert_eq!(parsed, None);
            }
            Err(e) => panic!("Expected successful parse, got error: {:?}", e),
        }
    }

    /// Test Scheme number parsing
    #[test]
    fn test_scheme_number_parsing() {
        let parser = scheme_number();
        
        // Integer
        match parser.parse("42 ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                // Verify it's parsed as a number (exact type depends on implementation)
            }
            Err(e) => panic!("Expected successful integer parse, got error: {:?}", e),
        }
        
        // Negative integer
        match parser.parse("-123 ") {
            Ok((remaining, _parsed)) => {
                assert_eq!(remaining, " ");
            }
            Err(e) => panic!("Expected successful negative integer parse, got error: {:?}", e),
        }
        
        // Floating point
        match parser.parse("3.14159 ") {
            Ok((remaining, _parsed)) => {
                assert_eq!(remaining, " ");
            }
            Err(e) => panic!("Expected successful float parse, got error: {:?}", e),
        }
        
        // Rational number (if supported)
        match parser.parse("22/7 ") {
            Ok((remaining, _parsed)) => {
                assert_eq!(remaining, " ");
            }
            Err(e) => panic!("Expected successful rational parse, got error: {:?}", e),
        }
    }

    /// Test Scheme string parsing
    #[test]
    fn test_scheme_string_parsing() {
        let parser = scheme_string();
        
        // Simple string
        match parser.parse("\"hello world\" ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "hello world");
            }
            Err(e) => panic!("Expected successful string parse, got error: {:?}", e),
        }
        
        // String with escape sequences
        match parser.parse("\"line1\\nline2\\t\\\"quoted\\\"\" ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "line1\nline2\t\"quoted\"");
            }
            Err(e) => panic!("Expected successful escaped string parse, got error: {:?}", e),
        }
        
        // Empty string
        match parser.parse("\"\" ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "");
            }
            Err(e) => panic!("Expected successful empty string parse, got error: {:?}", e),
        }
        
        // Unterminated string (should fail)
        match parser.parse("\"unterminated") {
            Ok(_) => panic!("Expected parse error for unterminated string"),
            Err(e) => {
                assert!(e.message.contains("unterminated") || e.message.contains("expected"));
            }
        }
    }

    /// Test Scheme character parsing
    #[test]
    fn test_scheme_character_parsing() {
        let parser = scheme_character();
        
        // Simple character
        match parser.parse("#\\a ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, 'a');
            }
            Err(e) => panic!("Expected successful character parse, got error: {:?}", e),
        }
        
        // Named character - newline
        match parser.parse("#\\newline ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, '\n');
            }
            Err(e) => panic!("Expected successful newline parse, got error: {:?}", e),
        }
        
        // Named character - space
        match parser.parse("#\\space ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, ' ');
            }
            Err(e) => panic!("Expected successful space parse, got error: {:?}", e),
        }
        
        // Named character - tab
        match parser.parse("#\\tab ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, '\t');
            }
            Err(e) => panic!("Expected successful tab parse, got error: {:?}", e),
        }
    }

    /// Test Scheme symbol parsing
    #[test]
    fn test_scheme_symbol_parsing() {
        let parser = scheme_symbol();
        
        // Simple symbol
        match parser.parse("hello ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "hello");
            }
            Err(e) => panic!("Expected successful symbol parse, got error: {:?}", e),
        }
        
        // Symbol with special characters
        match parser.parse("my-var? ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "my-var?");
            }
            Err(e) => panic!("Expected successful symbol with special chars parse, got error: {:?}", e),
        }
        
        // Symbol with numbers (not at start)
        match parser.parse("var123 ") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, " ");
                assert_eq!(parsed, "var123");
            }
            Err(e) => panic!("Expected successful symbol with numbers parse, got error: {:?}", e),
        }
        
        // Symbol starting with number (should fail)
        match parser.parse("123var") {
            Ok(_) => panic!("Expected parse error for symbol starting with number"),
            Err(_e) => {
                // Expected to fail
            }
        }
    }

    /// Test comment skipping
    #[test]
    fn test_comment_parsing() {
        let parser = CommentSkipper::new();
        
        // Line comment
        match parser.parse("; this is a comment\nrest") {
            Ok((remaining, _)) => {
                assert_eq!(remaining, "rest");
            }
            Err(e) => panic!("Expected successful comment skip, got error: {:?}", e),
        }
        
        // Block comment
        match parser.parse("#| block comment |# rest") {
            Ok((remaining, _)) => {
                assert_eq!(remaining, " rest");
            }
            Err(e) => panic!("Expected successful block comment skip, got error: {:?}", e),
        }
    }

    /// Test whitespace handling
    #[test]
    fn test_whitespace_parsing() {
        let parser = whitespace1();
        
        // Multiple whitespace characters
        match parser.parse("   \t\n  abc") {
            Ok((remaining, parsed)) => {
                assert_eq!(remaining, "abc");
                assert_eq!(parsed.len(), 7); // Should capture all whitespace
            }
            Err(e) => panic!("Expected successful whitespace parse, got error: {:?}", e),
        }
        
        // No whitespace (should fail for whitespace1)
        match parser.parse("abc") {
            Ok(_) => panic!("Expected parse error when no whitespace for whitespace1"),
            Err(_e) => {
                // Expected to fail
            }
        }
    }

    /// Test error span information
    #[test]
    fn test_error_span_information() {
        let parser = tag("hello");
        
        match parser.parse("help") {
            Ok(_) => panic!("Expected parse error"),
            Err(e) => {
                assert_eq!(e.span.start, 0);
                assert_eq!(e.span.end, 4); // Should span the entire input
                assert!(e.message.len() > 0);
                assert!(e.expected.len() > 0);
                assert_eq!(e.actual, "help");
            }
        }
    }

    /// Integration test: Parse a simple S-expression
    #[test]
    fn test_simple_sexp_integration() {
        let parser = whitespace0().and_then(|_| scheme_sexp());
        
        // Simple list
        match parser.parse("(+ 1 2)") {
            Ok((remaining, _parsed)) => {
                assert_eq!(remaining.trim(), "");
                // S-expression should be parsed correctly
            }
            Err(e) => panic!("Expected successful S-exp parse, got error: {:?}", e),
        }
        
        // Nested list
        match parser.parse("(if (> x 0) x (- x))") {
            Ok((remaining, _parsed)) => {
                assert_eq!(remaining.trim(), "");
            }
            Err(e) => panic!("Expected successful nested S-exp parse, got error: {:?}", e),
        }
    }

    /// Performance test: Parse large input efficiently
    #[test]
    fn test_performance_large_input() {
        let parser = digit().many();
        let large_input = "1".repeat(10000);
        
        let start_time = std::time::Instant::now();
        match parser.parse(&large_input) {
            Ok((remaining, parsed)) => {
                let duration = start_time.elapsed();
                assert_eq!(remaining, "");
                assert_eq!(parsed.len(), 10000);
                
                // Should complete in reasonable time (adjust threshold as needed)
                assert!(duration.as_millis() < 100, "Parse took too long: {:?}", duration);
            }
            Err(e) => panic!("Expected successful large input parse, got error: {:?}", e),
        }
    }
}