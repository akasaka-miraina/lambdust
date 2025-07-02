//! Lexical analyzer for Scheme source code

use crate::error::{LambdustError, Result};
use std::fmt;

/// Token types in Scheme
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Left parenthesis '('
    LeftParen,
    /// Right parenthesis ')'
    RightParen,
    /// Quote '
    Quote,
    /// Quasiquote `
    Quasiquote,
    /// Unquote ,
    Unquote,
    /// Unquote-splicing ,@
    UnquoteSplicing,
    /// Dot for dotted pairs
    Dot,
    /// Boolean literal (#t or #f)
    Boolean(bool),
    /// Number literal
    Number(SchemeNumber),
    /// String literal
    String(String),
    /// Character literal
    Character(char),
    /// Symbol/identifier
    Symbol(String),
}

/// Number types in Scheme
///
/// Represents the different numeric types supported by the Scheme language
/// according to the R7RS specification.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemeNumber {
    /// Exact integer values
    Integer(i64),
    /// Exact rational numbers (numerator, denominator)
    Rational(i64, i64),
    /// Inexact real numbers (floating point)
    Real(f64),
    /// Complex numbers (real part, imaginary part)
    Complex(f64, f64),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Quote => write!(f, "'"),
            Token::Quasiquote => write!(f, "`"),
            Token::Unquote => write!(f, ","),
            Token::UnquoteSplicing => write!(f, ",@"),
            Token::Dot => write!(f, "."),
            Token::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Token::Number(n) => write!(f, "{n}"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Character(c) => write!(f, "#\\{c}"),
            Token::Symbol(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for SchemeNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeNumber::Integer(i) => write!(f, "{i}"),
            SchemeNumber::Rational(n, d) => write!(f, "{n}/{d}"),
            SchemeNumber::Real(r) => write!(f, "{r}"),
            SchemeNumber::Complex(r, i) => write!(f, "{r}+{i}i"),
        }
    }
}

/// Lexer for tokenizing Scheme source code
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
        };
        lexer.current_char = lexer.input.chars().next();
        lexer
    }

    /// Advance to the next character
    fn advance(&mut self) {
        self.position += self.current_char.map_or(0, |c| c.len_utf8());
        self.current_char = self.input[self.position..].chars().next();
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().nth(1)
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == ';' {
                // Skip line comment
                while let Some(ch) = self.current_char {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Read a number token
    fn read_number(&mut self) -> Result<Token> {
        let mut number_str = String::new();
        let mut has_dot = false;
        let mut has_slash = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '+' || ch == '-' {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot && !has_slash {
                has_dot = true;
                number_str.push(ch);
                self.advance();
            } else if ch == '/' && !has_dot && !has_slash {
                has_slash = true;
                number_str.push(ch);
                self.advance();
            } else if ch == 'i' && !number_str.is_empty() {
                // Complex number
                number_str.push(ch);
                self.advance();
                break;
            } else {
                break;
            }
        }

        self.parse_number(&number_str)
    }

    /// Parse a number string into a SchemeNumber
    fn parse_number(&self, s: &str) -> Result<Token> {
        // Handle complex numbers
        if let Some(real_part) = s.strip_suffix('i') {
            if let Ok(r) = real_part.parse::<f64>() {
                return Ok(Token::Number(SchemeNumber::Complex(0.0, r)));
            }
        }

        // Handle rational numbers
        if let Some(slash_pos) = s.find('/') {
            let numerator = &s[..slash_pos];
            let denominator = &s[slash_pos + 1..];
            if let (Ok(n), Ok(d)) = (numerator.parse::<i64>(), denominator.parse::<i64>()) {
                if d == 0 {
                    return Err(LambdustError::lexer_error(
                        "Division by zero in rational".to_string(),
                    ));
                }
                return Ok(Token::Number(SchemeNumber::Rational(n, d)));
            }
        }

        // Handle real numbers
        if s.contains('.') {
            if let Ok(r) = s.parse::<f64>() {
                return Ok(Token::Number(SchemeNumber::Real(r)));
            }
        }

        // Handle integers
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Token::Number(SchemeNumber::Integer(i)));
        }

        Err(LambdustError::lexer_error(format!("Invalid number: {s}")))
    }

    /// Read a string token
    fn read_string(&mut self) -> Result<Token> {
        let mut string_value = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(Token::String(string_value));
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string_value.push('\n'),
                    Some('t') => string_value.push('\t'),
                    Some('r') => string_value.push('\r'),
                    Some('\\') => string_value.push('\\'),
                    Some('"') => string_value.push('"'),
                    Some(c) => string_value.push(c),
                    None => {
                        return Err(LambdustError::lexer_error(
                            "Unterminated string".to_string(),
                        ));
                    }
                }
                self.advance();
            } else {
                string_value.push(ch);
                self.advance();
            }
        }

        Err(LambdustError::lexer_error(
            "Unterminated string".to_string(),
        ))
    }

    /// Read a character token
    fn read_character(&mut self) -> Result<Token> {
        self.advance(); // Skip #
        self.advance(); // Skip \

        match self.current_char {
            Some('s') if self.input[self.position..].starts_with("space") => {
                // Skip "pace"
                for _ in 0..4 {
                    self.advance();
                }
                self.advance();
                Ok(Token::Character(' '))
            }
            Some('n') if self.input[self.position..].starts_with("newline") => {
                // Skip "ewline"
                for _ in 0..6 {
                    self.advance();
                }
                self.advance();
                Ok(Token::Character('\n'))
            }
            Some('t') if self.input[self.position..].starts_with("tab") => {
                // Skip "ab"
                for _ in 0..2 {
                    self.advance();
                }
                self.advance();
                Ok(Token::Character('\t'))
            }
            Some(ch) => {
                self.advance();
                Ok(Token::Character(ch))
            }
            None => Err(LambdustError::lexer_error(
                "Incomplete character literal".to_string(),
            )),
        }
    }

    /// Read a symbol or boolean token
    fn read_symbol(&mut self) -> Result<Token> {
        let mut symbol = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_whitespace() || "()[]{}\",'`;".contains(ch) {
                break;
            }
            symbol.push(ch);
            self.advance();
        }

        // Check for boolean literals
        match symbol.as_str() {
            "#t" | "#true" => Ok(Token::Boolean(true)),
            "#f" | "#false" => Ok(Token::Boolean(false)),
            _ => Ok(Token::Symbol(symbol)),
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace_and_comments();

        match self.current_char {
            None => Ok(None),
            Some('(') => {
                self.advance();
                Ok(Some(Token::LeftParen))
            }
            Some(')') => {
                self.advance();
                Ok(Some(Token::RightParen))
            }
            Some('\'') => {
                self.advance();
                Ok(Some(Token::Quote))
            }
            Some('`') => {
                self.advance();
                Ok(Some(Token::Quasiquote))
            }
            Some(',') => {
                self.advance();
                if self.current_char == Some('@') {
                    self.advance();
                    Ok(Some(Token::UnquoteSplicing))
                } else {
                    Ok(Some(Token::Unquote))
                }
            }
            Some('.') => {
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.read_number().map(Some)
                } else {
                    self.advance();
                    Ok(Some(Token::Dot))
                }
            }
            Some('"') => self.read_string().map(Some),
            Some('#') => {
                if self.peek() == Some('\\') {
                    self.read_character().map(Some)
                } else {
                    self.read_symbol().map(Some)
                }
            }
            Some(ch) if ch.is_ascii_digit() || ch == '+' || ch == '-' => {
                // Check if it's a number or a symbol
                if ch.is_ascii_digit()
                    || (ch == '+' || ch == '-') && self.peek().is_some_and(|c| c.is_ascii_digit())
                {
                    self.read_number().map(Some)
                } else {
                    self.read_symbol().map(Some)
                }
            }
            Some(_) => self.read_symbol().map(Some),
        }
    }
}

/// Tokenize a string into a vector of tokens
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("()").unwrap();
        assert_eq!(tokens, vec![Token::LeftParen, Token::RightParen]);
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("42 3.14 1/2").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(SchemeNumber::Integer(42)),
                Token::Number(SchemeNumber::Real(3.14)),
                Token::Number(SchemeNumber::Rational(1, 2)),
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
    }

    #[test]
    fn test_symbols() {
        let tokens = tokenize("+ define lambda").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("+".to_string()),
                Token::Symbol("define".to_string()),
                Token::Symbol("lambda".to_string()),
            ]
        );
    }

    #[test]
    fn test_booleans() {
        let tokens = tokenize("#t #f").unwrap();
        assert_eq!(tokens, vec![Token::Boolean(true), Token::Boolean(false)]);
    }

    #[test]
    fn test_quote_tokens() {
        let tokens = tokenize("'x `(,y ,@z)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Quote,
                Token::Symbol("x".to_string()),
                Token::Quasiquote,
                Token::LeftParen,
                Token::Unquote,
                Token::Symbol("y".to_string()),
                Token::UnquoteSplicing,
                Token::Symbol("z".to_string()),
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("; This is a comment\n(+ 1 2)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("+".to_string()),
                Token::Number(SchemeNumber::Integer(1)),
                Token::Number(SchemeNumber::Integer(2)),
                Token::RightParen,
            ]
        );
    }
}
