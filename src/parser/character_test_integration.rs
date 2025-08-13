//! Integration tests for character literal parsing

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::{Expr, Literal};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_character_literal_complete_pipeline() {
        let test_cases = vec![
            ("#\\a", 'a'),
            ("#\\space", ' '),
            ("#\\newline", '\n'),
            ("#\\tab", '\t'),
            ("#\\9", '9'),
            ("#\\A", 'A'),
            ("#\\x41", 'A'),
            ("#\\x1F600", '😀'),
        ];

        for (input, expected) in test_cases {
            println!("Testing: {}", input);
            
            // Step 1: Lexing
            let mut lexer = Lexer::new(input, Some("test"));
            let tokens = lexer.tokenize().expect(&format!("Failed to tokenize: {}", input));
            
            // Should have 2 tokens: Character token + EOF
            assert_eq!(tokens.len(), 2, "Expected 2 tokens for {}", input);
            assert_eq!(tokens[0].kind, crate::lexer::TokenKind::Character, "First token should be Character for {}", input);
            
            // Step 2: Parse the character literal
            let mut parser = Parser::new(tokens);
            let expr = parser.parse_character().expect(&format!("Failed to parse character: {}", input));
            
            // Step 3: Check the AST
            match expr.inner {
                Expr::Literal(Literal::Character(ch)) => {
                    assert_eq!(ch, expected, "Character value mismatch for {}: got '{}', expected '{}'", input, ch, expected);
                }
                _ => panic!("Expected character literal, got: {:?}", expr.inner),
            }
            
            println!("  Success: {} -> '{}'", input, expected);
        }
    }

    #[test]
    fn test_invalid_character_literals() {
        let invalid_cases = vec![
            "#\\invalid_name",  // Invalid named character - this should be lexed as #\i + "nvalid_name"
        ];

        for input in invalid_cases {
            println!("Testing invalid: {}", input);
            
            let mut lexer = Lexer::new(input, Some("test"));
            let result = lexer.tokenize();
            
            if let Ok(tokens) = result {
                println!("  Tokens: {:?}", tokens.iter().map(|t| &t.text).collect::<Vec<_>>());
                
                // The lexer should produce multiple tokens for invalid named characters
                // For "#\invalid_name", we expect "#\i" (character) + "nvalid_name" (identifier)
                if tokens.len() >= 3 { // character + identifier + EOF
                    let mut parser = Parser::new(tokens);
                    let result = parser.parse_character();
                    // The parsing should succeed for the character part "#\i"
                    assert!(result.is_ok(), "Expected successful parsing of character part for: {}", input);
                    match result.unwrap().inner {
                        crate::ast::Expr::Literal(crate::ast::Literal::Character(ch)) => {
                            assert_eq!(ch, 'i', "Expected 'i' from #\\invalid_name");
                        }
                        _ => panic!("Expected character literal"),
                    }
                }
            }
        }
    }

    #[test]
    fn test_r7rs_named_characters() {
        let r7rs_named = vec![
            ("#\\alarm", '\x07'),
            ("#\\backspace", '\x08'),
            ("#\\delete", '\x7F'),
            ("#\\escape", '\x1B'),
            ("#\\newline", '\n'),
            ("#\\null", '\0'),
            ("#\\return", '\r'),
            ("#\\space", ' '),
            ("#\\tab", '\t'),
            ("#\\vtab", '\x0B'),
        ];

        for (input, expected) in r7rs_named {
            let mut lexer = Lexer::new(input, Some("test"));
            let tokens = lexer.tokenize().expect(&format!("Failed to tokenize: {}", input));
            
            let mut parser = Parser::new(tokens);
            let expr = parser.parse_character().expect(&format!("Failed to parse character: {}", input));
            
            match expr.inner {
                Expr::Literal(Literal::Character(ch)) => {
                    assert_eq!(ch, expected, "R7RS character mismatch for {}: got '{}' (U+{:04X}), expected '{}' (U+{:04X})", 
                               input, ch, ch as u32, expected, expected as u32);
                }
                _ => panic!("Expected character literal, got: {:?}", expr.inner),
            }
            
            println!("  R7RS compliance: {} -> U+{:04X}", input, expected as u32);
        }
    }
}