//! SRFI-180 JSON Lexical Analyzer
//!
//! RFC 8259 compliant JSON tokenizer with Unicode support,
//! proper error reporting, and streaming capabilities.

use super::error::{JsonError, JsonResult, JsonContext};
use std::str::Chars;
use std::iter::Peekable;

/// JSON token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// JSON string literal
    String(String),
    /// JSON number (integer or floating point)
    Number(f64),
    /// JSON true literal
    True,
    /// JSON false literal
    False,
    /// JSON null literal
    Null,
    /// Left brace {
    LeftBrace,
    /// Right brace }
    RightBrace,
    /// Left bracket [
    LeftBracket,
    /// Right bracket ]
    RightBracket,
    /// Comma ,
    Comma,
    /// Colon :
    Colon,
    /// End of input
    Eof,
}

impl Token {
    /// Get human-readable name for error messages
    pub fn name(&self) -> &'static str {
        match self {
            Token::String(_) => "string",
            Token::Number(_) => "number",
            Token::True => "true",
            Token::False => "false", 
            Token::Null => "null",
            Token::LeftBrace => "'{'",
            Token::RightBrace => "'}'",
            Token::LeftBracket => "'['",
            Token::RightBracket => "']'",
            Token::Comma => "','",
            Token::Colon => "':'",
            Token::Eof => "end of input",
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "string: {}", s),
            Token::Number(n) => write!(f, "number: {}", n),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// JSON lexer with position tracking and error reporting
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    context: JsonContext,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer with default options
    pub fn new(input: &'a str) -> Self {
        Self::with_options(input, super::error::JsonOptions::default())
    }
    
    /// Get current position information
    pub fn position(&self) -> &super::error::Position {
        &self.context.position
    }

    /// Create a new lexer with custom options
    pub fn with_options(input: &'a str, options: super::error::JsonOptions) -> Self {
        let mut lexer = Self {
            input: input.chars().peekable(),
            context: JsonContext::new(options),
            current_char: None,
        };
        lexer.advance(); // Load first character
        lexer
    }

    /// Get the current parsing context
    pub fn context(&self) -> &JsonContext {
        &self.context
    }

    /// Get mutable reference to context
    pub fn context_mut(&mut self) -> &mut JsonContext {
        &mut self.context
    }

    /// Advance to next character
    fn advance(&mut self) {
        self.current_char = self.input.next();
        if let Some(ch) = self.current_char {
            self.context.advance(ch);
        }
    }

    /// Peek at next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\n' | '\r' => self.advance(),
                _ => break,
            }
        }
    }

    /// Parse a JSON string literal
    fn parse_string(&mut self) -> JsonResult<String> {
        let mut result = String::new();
        
        // Skip opening quote
        self.advance();

        loop {
            match self.current_char {
                None => return Err(self.context.unexpected_eof("closing quote")),
                Some('"') => {
                    self.advance(); // Skip closing quote
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char {
                        None => return Err(self.context.unexpected_eof("escape sequence")),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('b') => result.push('\u{0008}'), // backspace
                        Some('f') => result.push('\u{000C}'), // form feed
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('u') => {
                            // Parse Unicode escape sequence \uXXXX
                            let unicode_char = self.parse_unicode_escape()?;
                            result.push(unicode_char);
                        }
                        Some(ch) => {
                            return Err(self.context.syntax_error(
                                format!("Invalid escape sequence: \\{}", ch)
                            ));
                        }
                    }
                    self.advance();
                }
                Some(ch) if ch.is_control() => {
                    return Err(self.context.syntax_error(
                        format!("Unescaped control character in string: U+{:04X}", ch as u32)
                    ));
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
            }
        }

        Ok(result)
    }

    /// Parse Unicode escape sequence \uXXXX
    fn parse_unicode_escape(&mut self) -> JsonResult<char> {
        let mut code_point = 0u32;
        
        for _ in 0..4 {
            self.advance();
            match self.current_char {
                None => return Err(self.context.unexpected_eof("Unicode escape sequence")),
                Some(ch) if ch.is_ascii_hexdigit() => {
                    code_point = code_point * 16 + ch.to_digit(16).unwrap();
                }
                Some(ch) => {
                    return Err(self.context.syntax_error(
                        format!("Invalid hex digit in Unicode escape: {}", ch)
                    ));
                }
            }
        }

        // Handle surrogate pairs (UTF-16 encoding)
        if (0xD800..0xDC00).contains(&code_point) {
            // High surrogate - expect low surrogate
            if self.peek() == Some('\\') {
                self.advance(); // consume '\'
                if self.peek() == Some('u') {
                    self.advance(); // consume 'u'
                    let low_surrogate = self.parse_unicode_escape_value()?;
                    if (0xDC00..0xE000).contains(&low_surrogate) {
                        // Valid surrogate pair
                        let combined = 0x10000 + ((code_point - 0xD800) << 10) + (low_surrogate - 0xDC00);
                        return std::char::from_u32(combined)
                            .ok_or_else(|| self.context.invalid_unicode("Invalid surrogate pair"));
                    }
                }
            }
            return Err(self.context.invalid_unicode("Unpaired high surrogate"));
        }

        std::char::from_u32(code_point)
            .ok_or_else(|| self.context.invalid_unicode(format!("Invalid code point: U+{:04X}", code_point)))
    }

    /// Parse 4 hex digits for Unicode escape (helper)
    fn parse_unicode_escape_value(&mut self) -> JsonResult<u32> {
        let mut code_point = 0u32;
        for _ in 0..4 {
            self.advance();
            match self.current_char {
                None => return Err(self.context.unexpected_eof("Unicode escape sequence")),
                Some(ch) if ch.is_ascii_hexdigit() => {
                    code_point = code_point * 16 + ch.to_digit(16).unwrap();
                }
                Some(ch) => {
                    return Err(self.context.syntax_error(
                        format!("Invalid hex digit in Unicode escape: {}", ch)
                    ));
                }
            }
        }
        Ok(code_point)
    }

    /// Parse a JSON number
    fn parse_number(&mut self) -> JsonResult<f64> {
        let mut number_str = String::new();
        
        // Handle negative sign
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }

        // Parse integer part
        match self.current_char {
            None => return Err(self.context.unexpected_eof("number")),
            Some('0') => {
                number_str.push('0');
                self.advance();
                // Leading zeros not allowed except for '0' itself
                if let Some(ch) = self.current_char {
                    if ch.is_ascii_digit() {
                        return Err(self.context.syntax_error("Leading zeros not allowed"));
                    }
                }
            }
            Some(ch) if ch.is_ascii_digit() => {
                while let Some(ch) = self.current_char {
                    if ch.is_ascii_digit() {
                        number_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            Some(ch) => {
                return Err(self.context.syntax_error(
                    format!("Expected digit, found '{}'", ch)
                ));
            }
        }

        // Parse decimal part
        if self.current_char == Some('.') {
            number_str.push('.');
            self.advance();
            
            if !self.current_char.map_or(false, |ch| ch.is_ascii_digit()) {
                return Err(self.context.syntax_error("Expected digit after decimal point"));
            }

            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    number_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse exponent part
        if let Some(ch) = self.current_char {
            if ch == 'e' || ch == 'E' {
                number_str.push(ch);
                self.advance();
                
                // Optional sign
                if let Some(sign) = self.current_char {
                    if sign == '+' || sign == '-' {
                        number_str.push(sign);
                        self.advance();
                    }
                }
                
                if !self.current_char.map_or(false, |ch| ch.is_ascii_digit()) {
                    return Err(self.context.syntax_error("Expected digit in exponent"));
                }

                while let Some(ch) = self.current_char {
                    if ch.is_ascii_digit() {
                        number_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        // Parse the complete number
        number_str.parse::<f64>()
            .map_err(|_| self.context.syntax_error(format!("Invalid number: {}", number_str)))
    }

    /// Parse a literal (true, false, null)
    fn parse_literal(&mut self, expected: &str, token: Token) -> JsonResult<Token> {
        for expected_char in expected.chars() {
            if self.current_char != Some(expected_char) {
                return Err(self.context.syntax_error(
                    format!("Expected '{}', found '{:?}'", expected, self.current_char)
                ));
            }
            self.advance();
        }
        Ok(token)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> JsonResult<Token> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),
            Some('"') => {
                let s = self.parse_string()?;
                Ok(Token::String(s))
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let n = self.parse_number()?;
                Ok(Token::Number(n))
            }
            Some('t') => self.parse_literal("true", Token::True),
            Some('f') => self.parse_literal("false", Token::False),
            Some('n') => self.parse_literal("null", Token::Null),
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some('[') => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RightBracket)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some(ch) => Err(self.context.syntax_error(
                format!("Unexpected character: '{}'", ch)
            )),
        }
    }

    /// Check if we're at end of input
    pub fn is_eof(&self) -> bool {
        self.current_char.is_none()
    }

    /// Create an iterator over tokens
    pub fn tokens(self) -> TokenIterator<'a> {
        TokenIterator { lexer: self }
    }
}

/// Iterator adapter for lexer tokens
pub struct TokenIterator<'a> {
    lexer: Lexer<'a>,
}

// Type aliases for compatibility with other modules
pub type JsonLexer<'a> = Lexer<'a>;
pub type JsonToken = Token;

impl<'a> Iterator for TokenIterator<'a> {
    type Item = JsonResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lexer.is_eof() {
            None
        } else {
            Some(self.lexer.next_token())
        }
    }
}