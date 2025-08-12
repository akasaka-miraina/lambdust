//! Literal parsing utilities.

use super::Parser;
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Result, Spanned};

impl Parser {
    /// Parses a number literal with comprehensive error handling.
    /// 
    /// Supports all R7RS number formats:
    /// - Integers: 42, -17, +123
    /// - Reals: 3.14, -2.5, 1e10
    /// - Rationals: 1/2, -3/4
    /// - Complex: 1+2i, 3-4i, +i, -i
    pub fn parse_number(&mut self) -> Result<Spanned<Expr>> {
        let token = self.current_token();
        let span = token.span;
        let text = &token.text;
        
        // Use the token's built-in number parsing capability with enhanced error handling
        let parsed_number = token.parse_number().ok_or_else(|| {
            // Provide more specific error messages based on the token content
            let error_msg = if text.contains('/') {
                if text.ends_with('/') || text.starts_with('/') {
                    "Invalid rational number format: missing numerator or denominator".to_string()
                } else if text.contains("//") {
                    "Invalid rational number format: double slash not allowed".to_string()
                } else {
                    format!("Invalid rational number: {text}")
                }
            } else if text.to_lowercase().contains('i') {
                "Invalid complex number format".to_string()
            } else if text.contains('.') {
                "Invalid floating-point number format".to_string()
            } else {
                format!("Invalid number format: {text}")
            };
            
            crate::diagnostics::Error::parse_error(error_msg, span)
        })?;
        
        self.advance();
        
        let literal = match parsed_number {
            crate::lexer::NumericValue::Integer(i) => {
                // Check for integer overflow in extreme cases
                if i == i64::MIN { // Handle the special case where abs() would overflow
                    return Err(Box::new(crate::diagnostics::Error::parse_error(
                        "Integer too large", 
                        span
                    )))
                }
                // Integer literals are exact integers in R7RS
                Literal::integer(i)
            }
            crate::lexer::NumericValue::Real(r) => {
                // Check for special floating-point values
                // Allow infinity and NaN for R7RS-small special values
                if r.is_nan() || r.is_infinite() {
                    Literal::InexactReal(r) // +nan.0 and +/-inf.0 are allowed
                } else {
                    // Real literals are inexact in R7RS
                    Literal::float(r)
                }
            }
            crate::lexer::NumericValue::Rational(rat) => {
                // Validate rational number
                if rat.denominator == 0 {
                    return Err(Box::new(crate::diagnostics::Error::parse_error(
                        "Division by zero in rational number", 
                        span
                    )))
                }
                Literal::rational(rat.numerator, rat.denominator as i64)
            }
            crate::lexer::NumericValue::Complex(complex) => {
                // Validate complex number components
                if complex.real.is_nan() || complex.imag.is_nan() {
                    return Err(Box::new(crate::diagnostics::Error::parse_error(
                        "NaN components not allowed in complex numbers", 
                        span
                    )))
                }
                if complex.real.is_infinite() || complex.imag.is_infinite() {
                    return Err(Box::new(crate::diagnostics::Error::parse_error(
                        "Infinite components not allowed in complex numbers", 
                        span
                    )))
                }
                Literal::complex(complex.real, complex.imag)
            }
        };
        
        Ok(Spanned::new(Expr::Literal(literal), span))
    }

    /// Parses a string literal with comprehensive escape sequence handling.
    /// 
    /// Supports all R7RS string escape sequences:
    /// - \a (alarm), \b (backspace), \t (tab), \n (newline), \r (return)
    /// - \" (quote), \\ (backslash), \| (vertical bar)
    /// - \x<hex>; (Unicode escape)
    /// - \<octal> (octal escape)
    pub fn parse_string(&mut self) -> Result<Spanned<Expr>> {
        let token = self.current_token();
        let span = token.span;
        let text = &token.text;
        
        // Enhanced string parsing with better error messages
        let content = token.parse_string().map_err(|e| {
            // Provide more specific error messages
            let error_msg = if text.len() < 2 || !text.starts_with('"') || !text.ends_with('"') {
                "String must be enclosed in double quotes"
            } else if text.contains('\n') && !text.contains("\\n") {
                "Unescaped newline in string literal"
            } else if text.contains('\\') {
                "Invalid escape sequence in string"
            } else {
                "Invalid string literal"
            };
            
            crate::diagnostics::Error::parse_error(
                format!("{error_msg}: {e}"), 
                span
            )
        })?;
        
        self.advance();
        
        Ok(Spanned::new(Expr::Literal(Literal::String(content)), span))
    }

    /// Parses a character literal.
    /// 
    /// Supports all R7RS character formats:
    /// - Named characters: #\space, #\newline, #\tab, etc.
    /// - Unicode escapes: #\x<hex>
    /// - Single characters: #\a, #\A, #\1, etc.
    pub fn parse_character(&mut self) -> Result<Spanned<Expr>> {
        let token = self.current_token();
        let span = token.span;
        let text = &token.text;
        
        // Enhanced character parsing with better error messages
        let ch = token.parse_character().map_err(|e| {
            let error_msg = if !text.starts_with("#\\") {
                "Character literal must start with #\\"
            } else if text.len() == 2 {
                "Empty character literal"
            } else if text.starts_with("#\\x") && text.len() > 3 {
                "Invalid Unicode escape sequence"
            } else {
                "Invalid character literal"
            };
            
            crate::diagnostics::Error::parse_error(
                format!("{error_msg}: {e}"), 
                span
            )
        })?;
        
        self.advance();
        
        Ok(Spanned::new(Expr::Literal(Literal::Character(ch)), span))
    }

    /// Parses a boolean literal.
    /// 
    /// Supports both short and long forms:
    /// - #t, #f (short form)
    /// - #true, #false (long form)
    pub fn parse_boolean(&mut self) -> Result<Spanned<Expr>> {
        let token = self.current_token();
        let span = token.span;
        let text = token.text.clone();
        
        self.advance();
        
        let value = match text.as_str() {
            "#t" | "#true" => true,
            "#f" | "#false" => false,
            _ => {
                return Err(Box::new(crate::diagnostics::Error::parse_error(
                    format!(
                        "Invalid boolean literal '{text}'. Use #t/#true for true or #f/#false for false"
                    ), 
                    span
                )))
            }
        };
        
        Ok(Spanned::new(Expr::Literal(Literal::Boolean(value)), span))
    }
    
    /// Parses any literal value with appropriate error handling.
    /// 
    /// This is a convenience method that dispatches to the appropriate
    /// specific literal parsing method based on the token type.
    pub fn parse_any_literal(&mut self) -> Result<Spanned<Expr>> {
        let token_kind = self.current_token().kind.clone();
        
        match token_kind {
            crate::lexer::TokenKind::IntegerNumber | 
            crate::lexer::TokenKind::RealNumber | 
            crate::lexer::TokenKind::RationalNumber | 
            crate::lexer::TokenKind::ComplexNumber => self.parse_number(),
            
            crate::lexer::TokenKind::String => self.parse_string(),
            crate::lexer::TokenKind::Character => self.parse_character(),
            crate::lexer::TokenKind::Boolean => self.parse_boolean(),
            
            _ => Err(Box::new(crate::diagnostics::Error::parse_error(
                format!("Expected literal, found {}", self.current_token_text()),
                self.current_span(),
            )))
        }
    }
}