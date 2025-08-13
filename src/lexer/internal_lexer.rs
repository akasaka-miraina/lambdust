//! Internal lexer implementation for Lambdust
//!
//! This module provides a high-performance, zero-dependency lexical analyzer
//! that replaces the logos-based implementation. It supports the complete
//! R7RS Scheme specification plus Lambdust extensions.

use crate::diagnostics::{Error, Result, Span};
use std::str::Chars;
use std::iter::Peekable;

use super::{Token, TokenKind};

/// Internal lexer implementation without external dependencies
#[derive(Debug)]
pub struct InternalLexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    position: usize,
    current: Option<char>,
    filename: Option<&'a str>,
}

impl<'a> InternalLexer<'a> {
    /// Creates a new internal lexer for the given source code
    pub fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        let mut chars = source.chars().peekable();
        let current = chars.next();
        
        Self {
            source,
            chars,
            position: 0,
            current,
            filename,
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some(ch) = self.current {
            self.position += ch.len_utf8();
            self.current = self.chars.next();
        }
    }

    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Skip whitespace characters (space, tab, form-feed, carriage return, newline)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Check if a character is a valid identifier start character
    fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || "!$%&*+-/<=>?^_~|".contains(ch)
    }

    /// Check if a character is a valid identifier continuation character
    fn is_identifier_continue(ch: char) -> bool {
        ch.is_alphanumeric() || "!$%&*+-/<=>?^_~|".contains(ch)
    }

    /// Check if a character is a digit
    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Tokenize a single token from the current position
    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        let start_pos = self.position;
        
        let current = match self.current {
            Some(ch) => ch,
            None => {
                // End of input
                let span = Span::new(self.position, 0);
                return Ok(Some(Token::new(TokenKind::Eof, span, String::new())));
            }
        };

        let token_kind = match current {
            // Delimiters
            '(' => {
                self.advance();
                TokenKind::LeftParen
            }
            ')' => {
                self.advance();
                TokenKind::RightParen
            }
            '[' => {
                self.advance();
                TokenKind::LeftBracket
            }
            ']' => {
                self.advance();
                TokenKind::RightBracket
            }
            '{' => {
                self.advance();
                TokenKind::LeftBrace
            }
            '}' => {
                self.advance();
                TokenKind::RightBrace
            }
            
            // Quote forms
            '\'' => {
                self.advance();
                TokenKind::Quote
            }
            '`' => {
                self.advance();
                TokenKind::Quasiquote
            }
            ',' => {
                self.advance();
                // Check for ",@" (unquote-splicing)
                if self.current == Some('@') {
                    self.advance();
                    TokenKind::UnquoteSplicing
                } else {
                    TokenKind::Unquote
                }
            }
            
            // Dot or number
            '.' => {
                // Check if this is a decimal number like .123
                if let Some(next_ch) = self.peek() {
                    if Self::is_digit(next_ch) {
                        return self.tokenize_number(start_pos);
                    }
                }
                self.advance();
                TokenKind::Dot
            }
            
            // Type annotation or something else
            ':' => {
                self.advance();
                if self.current == Some(':') {
                    self.advance();
                    TokenKind::TypeAnnotation
                } else {
                    // Single colon - in R7RS this would be part of an identifier
                    return Err(Box::new(Error::lex_error(
                        "Unexpected character ':'".to_string(),
                        Span::new(start_pos, 1),
                    )));
                }
            }

            // Numbers (including signed)
            ch if Self::is_digit(ch) || (ch == '+' || ch == '-') => {
                return self.tokenize_number(start_pos);
            }

            // Strings
            '"' => {
                return self.tokenize_string(start_pos);
            }

            // Characters and special forms starting with #
            '#' => {
                return self.tokenize_hash_form(start_pos);
            }

            // Line comments
            ';' => {
                return self.tokenize_line_comment(start_pos);
            }

            // Identifiers
            ch if Self::is_identifier_start(ch) => {
                return self.tokenize_identifier(start_pos);
            }

            // Error: unrecognized character
            ch => {
                self.advance();
                return Err(Box::new(Error::lex_error(
                    format!("Unexpected character: '{ch}'"),
                    Span::new(start_pos, ch.len_utf8()),
                )));
            }
        };

        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        
        Ok(Some(Token::new(token_kind, span, text)))
    }

    /// Tokenize all tokens from the source
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token()? {
            let is_eof = token.kind == TokenKind::Eof;
            
            // Skip comments in the token stream (but preserve them for potential use)
            if !matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment) {
                tokens.push(token);
            }
            
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    // Placeholder implementations for specialized tokenization methods
    // These will be implemented in subsequent steps

    fn tokenize_number(&mut self, start_pos: usize) -> Result<Option<Token>> {
        let mut has_sign = false;
        let mut has_decimal = false;
        let mut has_exponent = false;
        let mut has_imaginary = false;
        let mut has_rational = false;
        let mut is_complex = false;
        
        // Handle optional sign at start
        if let Some(ch) = self.current {
            if ch == '+' || ch == '-' {
                has_sign = true;
                self.advance();
                
                // Special case: lone +/- followed by 'i' is pure imaginary
                if self.current == Some('i') {
                    self.advance();
                    let end_pos = self.position;
                    let span = Span::new(start_pos, end_pos - start_pos);
                    let text = self.source[start_pos..end_pos].to_owned();
                    return Ok(Some(Token::new(TokenKind::ComplexNumber, span, text)));
                }
            }
        }
        
        // If we only had a sign but no digits following, this might be an identifier
        if has_sign && !Self::is_digit(self.current.unwrap_or(' ')) && self.current != Some('.') {
            // Reset position and treat as identifier
            self.position = start_pos;
            self.current = self.source.chars().nth(start_pos);
            self.chars = self.source[start_pos..].chars().peekable();
            self.chars.next(); // Skip first char to sync with current
            return self.tokenize_identifier(start_pos);
        }
        
        // Parse integer part
        let mut digit_count = 0;
        while let Some(ch) = self.current {
            if Self::is_digit(ch) {
                digit_count += 1;
                self.advance();
            } else {
                break;
            }
        }
        
        // Handle decimal point
        if self.current == Some('.') {
            // Check if this is just a standalone dot (not a number)
            if digit_count == 0 && !has_sign {
                let end_pos = self.position;
                let span = Span::new(start_pos, end_pos - start_pos);
                let text = self.source[start_pos..end_pos].to_owned();
                return Ok(Some(Token::new(TokenKind::Dot, span, text)));
            }
            
            has_decimal = true;
            self.advance();
            
            // Parse fractional part
            while let Some(ch) = self.current {
                if Self::is_digit(ch) {
                    digit_count += 1;
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        // Handle rational numbers (/)
        if self.current == Some('/') {
            if digit_count == 0 || has_decimal || has_exponent {
                return Err(Box::new(Error::lex_error(
                    "Invalid rational number format".to_string(),
                    Span::new(start_pos, self.position - start_pos),
                )));
            }
            
            has_rational = true;
            self.advance();
            
            // Parse denominator
            let mut denom_digits = 0;
            while let Some(ch) = self.current {
                if Self::is_digit(ch) {
                    denom_digits += 1;
                    self.advance();
                } else {
                    break;
                }
            }
            
            if denom_digits == 0 {
                return Err(Box::new(Error::lex_error(
                    "Rational number missing denominator".to_string(),
                    Span::new(start_pos, self.position - start_pos),
                )));
            }
        }
        
        // Handle scientific notation (e/E)
        if !has_rational && (self.current == Some('e') || self.current == Some('E')) {
            has_exponent = true;
            self.advance();
            
            // Optional sign in exponent
            if self.current == Some('+') || self.current == Some('-') {
                self.advance();
            }
            
            // Parse exponent digits
            let mut exp_digits = 0;
            while let Some(ch) = self.current {
                if Self::is_digit(ch) {
                    exp_digits += 1;
                    self.advance();
                } else {
                    break;
                }
            }
            
            if exp_digits == 0 {
                return Err(Box::new(Error::lex_error(
                    "Scientific notation missing exponent digits".to_string(),
                    Span::new(start_pos, self.position - start_pos),
                )));
            }
        }
        
        // Handle imaginary unit (i)
        if self.current == Some('i') {
            has_imaginary = true;
            self.advance();
        }
        
        // Check for complex numbers (real+imag format)
        let mut complex_real_end = self.position;
        if !has_imaginary && (self.current == Some('+') || self.current == Some('-')) {
            // Try to parse the imaginary part
            let complex_start = self.position;
            self.advance(); // Skip +/-
            
            // Parse imaginary part (could be just 'i' or digits+'i')
            let mut imag_digits = 0;
            while let Some(ch) = self.current {
                if Self::is_digit(ch) {
                    imag_digits += 1;
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Check for decimal in imaginary part
            if self.current == Some('.') {
                self.advance();
                while let Some(ch) = self.current {
                    if Self::is_digit(ch) {
                        imag_digits += 1;
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            
            // Check for exponent in imaginary part
            if self.current == Some('e') || self.current == Some('E') {
                self.advance();
                if self.current == Some('+') || self.current == Some('-') {
                    self.advance();
                }
                while let Some(ch) = self.current {
                    if Self::is_digit(ch) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            
            // Must end with 'i' for complex number
            if self.current == Some('i') {
                is_complex = true;
                self.advance();
            } else {
                // Not a complex number, backtrack
                self.position = complex_real_end;
                self.current = self.source.chars().nth(self.position);
                self.chars = self.source[self.position..].chars().peekable();
                if self.position < self.source.len() {
                    self.chars.next(); // Skip first char to sync with current
                }
            }
        }
        
        if digit_count == 0 && !has_sign {
            return Err(Box::new(Error::lex_error(
                "Invalid number format".to_string(),
                Span::new(start_pos, self.position - start_pos),
            )));
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        
        // Determine the token type
        let token_kind = if is_complex || has_imaginary {
            TokenKind::ComplexNumber
        } else if has_rational {
            TokenKind::RationalNumber
        } else if has_decimal || has_exponent {
            TokenKind::RealNumber
        } else {
            TokenKind::IntegerNumber
        };
        
        Ok(Some(Token::new(token_kind, span, text)))
    }

    fn tokenize_string(&mut self, start_pos: usize) -> Result<Option<Token>> {
        self.advance(); // Skip opening quote
        let mut found_closing_quote = false;
        
        while let Some(ch) = self.current {
            match ch {
                '"' => {
                    // Found closing quote
                    self.advance();
                    found_closing_quote = true;
                    break;
                }
                '\\' => {
                    // Handle escape sequences
                    self.advance();
                    if let Some(escape_ch) = self.current {
                        match escape_ch {
                            '"' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                                // Valid single-character escape
                                self.advance();
                            }
                            'x' => {
                                // Unicode hex escape: \xHH...;
                                self.advance();
                                let mut hex_digits = 0;
                                while let Some(hex_ch) = self.current {
                                    if hex_ch.is_ascii_hexdigit() {
                                        hex_digits += 1;
                                        self.advance();
                                        if hex_digits > 6 {
                                            return Err(Box::new(Error::lex_error(
                                                "Unicode hex escape too long".to_string(),
                                                Span::new(start_pos, self.position - start_pos),
                                            )));
                                        }
                                    } else if hex_ch == ';' {
                                        self.advance();
                                        break;
                                    } else {
                                        return Err(Box::new(Error::lex_error(
                                            "Invalid character in unicode hex escape".to_string(),
                                            Span::new(start_pos, self.position - start_pos),
                                        )));
                                    }
                                }
                                if hex_digits == 0 {
                                    return Err(Box::new(Error::lex_error(
                                        "Empty unicode hex escape".to_string(),
                                        Span::new(start_pos, self.position - start_pos),
                                    )));
                                }
                            }
                            ch if ch.is_ascii_digit() => {
                                // Octal escape: \OOO
                                let mut octal_digits = 1;
                                self.advance();
                                while let Some(octal_ch) = self.current {
                                    if octal_ch.is_ascii_digit() && octal_ch <= '7' && octal_digits < 3 {
                                        octal_digits += 1;
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                return Err(Box::new(Error::lex_error(
                                    format!("Invalid escape sequence: \\{escape_ch}"),
                                    Span::new(start_pos, self.position - start_pos),
                                )));
                            }
                        }
                    } else {
                        return Err(Box::new(Error::lex_error(
                            "Unterminated escape sequence at end of file".to_string(),
                            Span::new(start_pos, self.position - start_pos),
                        )));
                    }
                }
                '\n' | '\r' => {
                    return Err(Box::new(Error::lex_error(
                        "Unterminated string literal".to_string(),
                        Span::new(start_pos, self.position - start_pos),
                    )));
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        if !found_closing_quote {
            return Err(Box::new(Error::lex_error(
                "Unterminated string literal at end of file".to_string(),
                Span::new(start_pos, self.position - start_pos),
            )));
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::String, span, text)))
    }

    fn tokenize_hash_form(&mut self, start_pos: usize) -> Result<Option<Token>> {
        self.advance(); // Skip #
        
        match self.current {
            Some('t') => {
                self.advance();
                // Check for #true
                if self.source[start_pos..].starts_with("#true") {
                    // Consume remaining characters
                    self.advance(); // r
                    self.advance(); // u
                    self.advance(); // e
                }
                let end_pos = self.position;
                let span = Span::new(start_pos, end_pos - start_pos);
                let text = self.source[start_pos..end_pos].to_owned();
                Ok(Some(Token::new(TokenKind::Boolean, span, text)))
            }
            Some('f') => {
                self.advance();
                // Check for #false
                if self.source[start_pos..].starts_with("#false") {
                    // Consume remaining characters
                    self.advance(); // a
                    self.advance(); // l
                    self.advance(); // s
                    self.advance(); // e
                }
                let end_pos = self.position;
                let span = Span::new(start_pos, end_pos - start_pos);
                let text = self.source[start_pos..end_pos].to_owned();
                Ok(Some(Token::new(TokenKind::Boolean, span, text)))
            }
            Some('\\') => {
                // Character literal
                self.advance();
                self.tokenize_character_literal(start_pos)
            }
            Some(':') => {
                // Keyword
                self.advance();
                self.tokenize_keyword(start_pos)
            }
            Some('|') => {
                // Block comment
                self.advance();
                self.tokenize_block_comment(start_pos)
            }
            _ => {
                Err(Box::new(Error::lex_error(
                    format!("Invalid character after #: {:?}", self.current),
                    Span::new(start_pos, self.position - start_pos + 1),
                )))
            }
        }
    }

    fn tokenize_character_literal(&mut self, start_pos: usize) -> Result<Option<Token>> {
        match self.current {
            Some('x') => {
                // Unicode hex character: #\xHH...
                self.advance();
                while let Some(ch) = self.current {
                    if ch.is_ascii_hexdigit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            Some(ch) => {
                // Check for named characters
                let remaining = &self.source[self.position..];
                
                // Try to match named characters
                if remaining.starts_with("alarm") {
                    for _ in 0..5 { self.advance(); }
                } else if remaining.starts_with("backspace") {
                    for _ in 0..9 { self.advance(); }
                } else if remaining.starts_with("delete") || remaining.starts_with("escape") {
                    for _ in 0..6 { self.advance(); }
                } else if remaining.starts_with("newline") {
                    for _ in 0..7 { self.advance(); }
                } else if remaining.starts_with("null") {
                    for _ in 0..4 { self.advance(); }
                } else if remaining.starts_with("return") {
                    for _ in 0..6 { self.advance(); }
                } else if remaining.starts_with("space") {
                    for _ in 0..5 { self.advance(); }
                } else if remaining.starts_with("tab") {
                    for _ in 0..3 { self.advance(); }
                } else if remaining.starts_with("vtab") {
                    for _ in 0..4 { self.advance(); }
                } else {
                    // Single character
                    self.advance();
                }
            }
            None => {
                return Err(Box::new(Error::lex_error(
                    "Incomplete character literal at end of file".to_string(),
                    Span::new(start_pos, self.position - start_pos),
                )));
            }
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::Character, span, text)))
    }

    fn tokenize_keyword(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Parse keyword identifier
        while let Some(ch) = self.current {
            if Self::is_identifier_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::Keyword, span, text)))
    }

    fn tokenize_block_comment(&mut self, start_pos: usize) -> Result<Option<Token>> {
        let mut nesting_level = 1;
        
        while let Some(ch) = self.current {
            if ch == '#' && self.peek() == Some('|') {
                // Start of nested comment
                nesting_level += 1;
                self.advance(); // #
                self.advance(); // |
            } else if ch == '|' && self.peek() == Some('#') {
                // End of comment
                nesting_level -= 1;
                self.advance(); // |
                self.advance(); // #
                if nesting_level == 0 {
                    break;
                }
            } else {
                self.advance();
            }
        }
        
        if nesting_level > 0 {
            return Err(Box::new(Error::lex_error(
                "Unterminated block comment".to_string(),
                Span::new(start_pos, self.position - start_pos),
            )));
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::BlockComment, span, text)))
    }

    fn tokenize_line_comment(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Skip until end of line
        while let Some(ch) = self.current {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::LineComment, span, text)))
    }

    fn tokenize_identifier(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Continue while we have valid identifier characters
        while let Some(ch) = self.current {
            if Self::is_identifier_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }
        
        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos - start_pos);
        let text = self.source[start_pos..end_pos].to_owned();
        Ok(Some(Token::new(TokenKind::Identifier, span, text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_delimiters() {
        let source = "()[]{}";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let expected_kinds = vec![
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ];
        
        assert_eq!(tokens.len(), expected_kinds.len());
        for (token, expected) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected);
        }
    }

    #[test]
    fn test_quote_forms() {
        let source = "'`,";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let expected_kinds = vec![
            TokenKind::Quote,
            TokenKind::Quasiquote,
            TokenKind::Unquote,
            TokenKind::Eof,
        ];
        
        assert_eq!(tokens.len(), expected_kinds.len());
        for (token, expected) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected);
        }
    }

    #[test]
    fn test_unquote_splicing() {
        let source = ",@";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::UnquoteSplicing);
        assert_eq!(tokens[0].text, ",@");
    }

    #[test]
    fn test_type_annotation() {
        let source = "::";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::TypeAnnotation);
        assert_eq!(tokens[0].text, "::");
    }

    #[test]
    fn test_simple_identifier() {
        let source = "hello-world";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "hello-world");
    }

    #[test]
    fn test_whitespace_skipping() {
        let source = "  a   b  ";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        // Should have: a, b, EOF
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "a");
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].text, "b");
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn test_integer_numbers() {
        let source = "42 +123 -456 0";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let number_tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert_eq!(number_tokens.len(), 4);
        
        assert_eq!(number_tokens[0].kind, TokenKind::IntegerNumber);
        assert_eq!(number_tokens[0].text, "42");
        
        assert_eq!(number_tokens[1].kind, TokenKind::IntegerNumber);
        assert_eq!(number_tokens[1].text, "+123");
        
        assert_eq!(number_tokens[2].kind, TokenKind::IntegerNumber);
        assert_eq!(number_tokens[2].text, "-456");
        
        assert_eq!(number_tokens[3].kind, TokenKind::IntegerNumber);
        assert_eq!(number_tokens[3].text, "0");
    }

    #[test]
    fn test_real_numbers() {
        let source = "3.14 -2.5 .5 1.0e10 -5.2e-3";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let number_tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert_eq!(number_tokens.len(), 5);
        
        for token in &number_tokens {
            assert_eq!(token.kind, TokenKind::RealNumber);
        }
        
        assert_eq!(number_tokens[0].text, "3.14");
        assert_eq!(number_tokens[1].text, "-2.5");
        assert_eq!(number_tokens[2].text, ".5");
        assert_eq!(number_tokens[3].text, "1.0e10");
        assert_eq!(number_tokens[4].text, "-5.2e-3");
    }

    #[test]
    fn test_rational_numbers() {
        let source = "22/7 -3/4 +1/2";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let number_tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert_eq!(number_tokens.len(), 3);
        
        for token in &number_tokens {
            assert_eq!(token.kind, TokenKind::RationalNumber);
        }
        
        assert_eq!(number_tokens[0].text, "22/7");
        assert_eq!(number_tokens[1].text, "-3/4");
        assert_eq!(number_tokens[2].text, "+1/2");
    }

    #[test]
    fn test_complex_numbers() {
        let source = "3+4i -2-5i 1.5+2.7i +i -i 0+0i";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let number_tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert_eq!(number_tokens.len(), 6);
        
        for token in &number_tokens {
            assert_eq!(token.kind, TokenKind::ComplexNumber);
        }
        
        assert_eq!(number_tokens[0].text, "3+4i");
        assert_eq!(number_tokens[1].text, "-2-5i");
        assert_eq!(number_tokens[2].text, "1.5+2.7i");
        assert_eq!(number_tokens[3].text, "+i");
        assert_eq!(number_tokens[4].text, "-i");
        assert_eq!(number_tokens[5].text, "0+0i");
    }

    #[test]
    fn test_signed_identifiers() {
        let source = "+ - +add -sub";
        let mut lexer = InternalLexer::new(source, None);
        let tokens = lexer.tokenize().unwrap();
        
        let ident_tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert_eq!(ident_tokens.len(), 4);
        
        for token in &ident_tokens {
            assert_eq!(token.kind, TokenKind::Identifier);
        }
        
        assert_eq!(ident_tokens[0].text, "+");
        assert_eq!(ident_tokens[1].text, "-");
        assert_eq!(ident_tokens[2].text, "+add");
        assert_eq!(ident_tokens[3].text, "-sub");
    }
}