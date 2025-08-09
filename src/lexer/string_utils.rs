//! String and character literal parsing utilities for Lambdust.
//!
//! This module provides comprehensive parsing and validation for string literals
//! and character literals according to R7RS Scheme specification, including:
//! - String escape sequences (basic and Unicode)
//! - Character literals (named and Unicode)
//! - Proper error handling with detailed diagnostics

use crate::diagnostics::{Error, Result, Span};

/// Validates string escape sequences in a string literal content.
/// The input should be the string content without the surrounding quotes.
pub fn validate_string_escapes(content: &str, span: Span) -> Result<()> {
    let mut chars = content.char_indices();
    
    while let Some((_i, ch)) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some((_, escape_ch)) => {
                    match escape_ch {
                        // Standard escape sequences
                        '"' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                            // Valid escape sequence
                        }
                        // Unicode hex escape: \xHHHH... (terminated by ;)
                        'x' => {
                            let mut hex_digits = String::new();
                            let mut found_semicolon = false;
                            
                            for (_, hex_ch) in chars.by_ref() {
                                if hex_ch == ';' {
                                    found_semicolon = true;
                                    break;
                                } else if hex_ch.is_ascii_hexdigit() {
                                    hex_digits.push(hex_ch);
                                    if hex_digits.len() > 6 {
                                        return Err(Box::new(Error::lex_error(
                                            "Unicode escape sequence too long (max 6 digits)",
                                            span,
                                        )))
                                    }
                                } else {
                                    return Err(Box::new(Error::lex_error(
                                        format!("Invalid character in Unicode escape: '{hex_ch}'"),
                                        span,
                                    )))
                                }
                            }
                            
                            if !found_semicolon {
                                return Err(Box::new(Error::lex_error(
                                    "Unicode escape sequence must end with ';'",
                                    span,
                                )))
                            }
                            
                            if hex_digits.is_empty() {
                                return Err(Box::new(Error::lex_error(
                                    "Unicode escape sequence must have at least one digit",
                                    span,
                                )))
                            }
                            
                            // Validate the Unicode code point
                            if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                                if code_point > 0x10FFFF {
                                    return Err(Box::new(Error::lex_error(
                                        "Unicode code point out of range (max 0x10FFFF)",
                                        span,
                                    )))
                                }
                                // Check for surrogate pairs (invalid in Unicode)
                                if (0xD800..=0xDFFF).contains(&code_point) {
                                    return Err(Box::new(Error::lex_error(
                                        "Unicode surrogate code points are not allowed",
                                        span,
                                    )))
                                }
                            } else {
                                return Err(Box::new(Error::lex_error(
                                    "Invalid Unicode escape sequence",
                                    span,
                                )))
                            }
                        }
                        // Octal escape: \NNN (1-3 octal digits)
                        ch if ch.is_ascii_digit() && ('0'..='7').contains(&ch) => {
                            let mut octal_digits = String::new();
                            octal_digits.push(ch);
                            
                            // Look ahead for up to 2 more octal digits
                            let mut peek_chars = chars.clone();
                            for _ in 0..2 {
                                if let Some((_, next_ch)) = peek_chars.next() {
                                    if ('0'..='7').contains(&next_ch) {
                                        octal_digits.push(next_ch);
                                        chars.next(); // Consume the character
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            // Validate octal value
                            if let Ok(_octal_value) = u8::from_str_radix(&octal_digits, 8) {
                                // u8 values are always <= 255, so no additional check needed
                            } else {
                                return Err(Box::new(Error::lex_error(
                                    "Invalid octal escape sequence",
                                    span,
                                )))
                            }
                        }
                        _ => {
                            return Err(Box::new(Error::lex_error(
                                format!("Invalid escape sequence: '\\{escape_ch}'"),
                                span,
                            )))
                        }
                    }
                },
                None => {
                    return Err(Box::new(Error::lex_error(
                        "String literal ends with incomplete escape sequence",
                        span,
                    )))
                }
            }
        }
    }
    
    Ok(())
}

/// Validates a character literal (without the #\\ prefix).
pub fn validate_character_literal(content: &str, span: Span) -> Result<()> {
    if content.is_empty() {
        return Err(Box::new(Error::lex_error("Empty character literal", span)))
    }
    
    // Check for named characters
    match content {
        "alarm" | "backspace" | "delete" | "escape" | "newline" |
        "null" | "return" | "space" | "tab" | "vtab" => {
            return Ok(());
        }
        _ => {}
    }
    
    // Check for Unicode hex escape
    if let Some(hex_part) = content.strip_prefix('x') {
        if hex_part.is_empty() {
            return Err(Box::new(Error::lex_error(
                "Unicode character literal must have hex digits after 'x'",
                span,
            )))
        }
        
        for ch in hex_part.chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid hex digit in character literal: '{ch}'"),
                    span,
                )))
            }
        }
        
        // Validate the Unicode code point
        if let Ok(code_point) = u32::from_str_radix(hex_part, 16) {
            if code_point > 0x10FFFF {
                return Err(Box::new(Error::lex_error(
                    "Unicode code point out of range (max 0x10FFFF)",
                    span,
                )))
            }
            // Check for surrogate pairs
            if (0xD800..=0xDFFF).contains(&code_point) {
                return Err(Box::new(Error::lex_error(
                    "Unicode surrogate code points are not allowed",
                    span,
                )))
            }
        } else {
            return Err(Box::new(Error::lex_error(
                "Invalid Unicode character literal",
                span,
            )))
        }
        
        return Ok(());
    }
    
    // Single character
    let mut chars = content.chars();
    let first_char = chars.next().unwrap();
    
    if chars.next().is_some() {
        return Err(Box::new(Error::lex_error(
            "Character literal can only contain one character or a named character",
            span,
        )))
    }
    
    // Check for valid character
    if first_char.is_control() && !matches!(first_char, '\t' | '\n' | '\r') {
        return Err(Box::new(Error::lex_error(
            "Control characters in character literals must use named forms",
            span,
        )))
    }
    
    Ok(())
}

/// Unescapes a string literal content, converting escape sequences to their actual values.
pub fn unescape_string(content: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = content.char_indices();
    
    while let Some((_, ch)) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some((_, escape_ch)) => {
                    match escape_ch {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'a' => result.push('\x07'), // Alert (bell)
                        'b' => result.push('\x08'), // Backspace
                        'f' => result.push('\x0C'), // Form feed
                        'n' => result.push('\n'),   // Newline
                        'r' => result.push('\r'),   // Carriage return
                        't' => result.push('\t'),   // Tab
                        'v' => result.push('\x0B'), // Vertical tab
                        'x' => {
                            // Unicode hex escape
                            let mut hex_digits = String::new();
                            for (_, hex_ch) in chars.by_ref() {
                                if hex_ch == ';' {
                                    break;
                                } else {
                                    hex_digits.push(hex_ch);
                                }
                            }
                            
                            let code_point = u32::from_str_radix(&hex_digits, 16)
                                .map_err(|_| Error::internal_error("Invalid Unicode escape during unescaping"))?;
                            let unicode_char = char::from_u32(code_point)
                                .ok_or_else(|| Error::internal_error("Invalid Unicode code point during unescaping"))?;
                            result.push(unicode_char);
                        }
                        ch if ch.is_ascii_digit() && ('0'..='7').contains(&ch) => {
                            // Octal escape
                            let mut octal_digits = String::new();
                            octal_digits.push(ch);
                            
                            // Look for up to 2 more octal digits
                            let mut peek_chars = chars.clone();
                            for _ in 0..2 {
                                if let Some((_, next_ch)) = peek_chars.next() {
                                    if ('0'..='7').contains(&next_ch) {
                                        octal_digits.push(next_ch);
                                        chars.next(); // Consume
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            let octal_value = u8::from_str_radix(&octal_digits, 8)
                                .map_err(|_| Error::internal_error("Invalid octal escape during unescaping"))?;
                            result.push(octal_value as char);
                        }
                        _ => {
                            return Err(Box::new(Error::internal_error(
                                format!("Unknown escape sequence during unescaping: \\{escape_ch}")
                            )))
                        }
                    }
                }
                None => {
                    return Err(Box::new(Error::internal_error(
                        "Incomplete escape sequence during unescaping"
                    )))
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}

/// Converts a character literal to its character value.
pub fn parse_character_literal(content: &str) -> Result<char> {
    // Handle named characters
    match content {
        "alarm" => return Ok('\x07'),
        "backspace" => return Ok('\x08'),
        "delete" => return Ok('\x7F'),
        "escape" => return Ok('\x1B'),
        "newline" => return Ok('\n'),
        "null" => return Ok('\0'),
        "return" => return Ok('\r'),
        "space" => return Ok(' '),
        "tab" => return Ok('\t'),
        "vtab" => return Ok('\x0B'),
        _ => {}
    }
    
    // Handle Unicode hex escape
    if let Some(hex_part) = content.strip_prefix('x') {
        let code_point = u32::from_str_radix(hex_part, 16)
            .map_err(|_| Error::internal_error("Invalid Unicode character literal during parsing"))?;
        return char::from_u32(code_point)
            .ok_or_else(|| Error::internal_error("Invalid Unicode code point during character parsing")).map_err(Box::new)
    }
    
    // Single character
    let mut chars = content.chars();
    let ch = chars.next()
        .ok_or_else(|| Error::internal_error("Empty character literal during parsing"))?;
    
    if chars.next().is_some() {
        return Err(Box::new(Error::internal_error("Multiple characters in character literal during parsing")));
    }
    
    Ok(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_escape_validation() {
        let span = Span::new(0, 10);
        
        // Valid escape sequences
        assert!(validate_string_escapes(r#"hello\nworld"#, span).is_ok());
        assert!(validate_string_escapes(r#"quote: \""#, span).is_ok());
        assert!(validate_string_escapes(r#"backslash: \\"#, span).is_ok());
        assert!(validate_string_escapes(r#"unicode: \x41;"#, span).is_ok());
        assert!(validate_string_escapes(r#"octal: \101"#, span).is_ok());
        
        // Invalid escape sequences
        assert!(validate_string_escapes(r#"invalid: \z"#, span).is_err());
        assert!(validate_string_escapes(r#"incomplete: \"#, span).is_err());
        assert!(validate_string_escapes(r#"bad unicode: \x;"#, span).is_err());
        assert!(validate_string_escapes(r#"bad unicode: \xGG;"#, span).is_err());
    }

    #[test]
    fn test_character_literal_validation() {
        let span = Span::new(0, 5);
        
        // Valid character literals
        assert!(validate_character_literal("a", span).is_ok());
        assert!(validate_character_literal("space", span).is_ok());
        assert!(validate_character_literal("newline", span).is_ok());
        assert!(validate_character_literal("x41", span).is_ok());
        
        // Invalid character literals
        assert!(validate_character_literal("", span).is_err());
        assert!(validate_character_literal("ab", span).is_err());
        assert!(validate_character_literal("invalid_name", span).is_err());
        assert!(validate_character_literal("xGG", span).is_err());
    }

    #[test]
    fn test_string_unescaping() {
        assert_eq!(unescape_string("hello world").unwrap(), "hello world");
        assert_eq!(unescape_string(r#"hello\nworld"#).unwrap(), "hello\nworld");
        assert_eq!(unescape_string(r#"quote: \""#).unwrap(), r#"quote: ""#);
        assert_eq!(unescape_string(r#"\x41;"#).unwrap(), "A");
        assert_eq!(unescape_string(r#"\101"#).unwrap(), "A");
        
        // Invalid escape sequences should error
        assert!(unescape_string(r#"\z"#).is_err());
    }

    #[test]
    fn test_character_parsing() {
        assert_eq!(parse_character_literal("a").unwrap(), 'a');
        assert_eq!(parse_character_literal("space").unwrap(), ' ');
        assert_eq!(parse_character_literal("newline").unwrap(), '\n');
        assert_eq!(parse_character_literal("x41").unwrap(), 'A');
        assert_eq!(parse_character_literal("x1F600").unwrap(), '😀');
        
        // Invalid literals should error
        assert!(parse_character_literal("").is_err());
        assert!(parse_character_literal("ab").is_err());
    }

    #[test]
    fn test_unicode_validation() {
        let span = Span::new(0, 10);
        
        // Valid Unicode
        assert!(validate_string_escapes(r#"\x1F600;"#, span).is_ok()); // Emoji
        assert!(validate_character_literal("x1F600", span).is_ok());
        
        // Invalid Unicode (too large)
        assert!(validate_string_escapes(r#"\x110000;"#, span).is_err());
        assert!(validate_character_literal("x110000", span).is_err());
        
        // Surrogate pairs (invalid)
        assert!(validate_string_escapes(r#"\xD800;"#, span).is_err());
        assert!(validate_character_literal("xD800", span).is_err());
    }
}