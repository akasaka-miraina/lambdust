//! Token utilities and extended functionality.
//!
//! This module provides helper functions and utilities for working with tokens,
//! including precedence handling, arity checking, and token classification.

use super::{Token, TokenKind};
#[cfg(test)]
use crate::diagnostics::Span;
use crate::lexer::numeric::*;
use crate::lexer::string_utils::*;

impl Token {
    /// Creates a simple identifier token for testing.
    #[cfg(test)]
    pub fn identifier(text: &str, span: Span) -> Self {
        Self::new(TokenKind::Identifier, span, text.to_string())
    }

    /// Creates a simple integer token for testing.
    #[cfg(test)]
    pub fn integer(text: &str, span: Span) -> Self {
        Self::new(TokenKind::IntegerNumber, span, text.to_string())
    }
    
    /// Creates a simple real number token for testing.
    #[cfg(test)]
    pub fn real(text: &str, span: Span) -> Self {
        Self::new(TokenKind::RealNumber, span, text.to_string())
    }
    
    /// Creates a simple rational token for testing.
    #[cfg(test)]
    pub fn rational(text: &str, span: Span) -> Self {
        Self::new(TokenKind::RationalNumber, span, text.to_string())
    }
    
    /// Creates a simple complex token for testing.
    #[cfg(test)]
    pub fn complex(text: &str, span: Span) -> Self {
        Self::new(TokenKind::ComplexNumber, span, text.to_string())
    }

    /// Creates a simple string token for testing.
    #[cfg(test)]
    pub fn string(text: &str, span: Span) -> Self {
        Self::new(TokenKind::String, span, text.to_string())
    }
    
    /// Creates a simple character token for testing.
    #[cfg(test)]
    pub fn character(text: &str, span: Span) -> Self {
        Self::new(TokenKind::Character, span, text.to_string())
    }
    
    /// Creates a simple keyword token for testing.
    #[cfg(test)]
    pub fn keyword(text: &str, span: Span) -> Self {
        Self::new(TokenKind::Keyword, span, text.to_string())
    }

    /// Returns the precedence of this token when used as an operator.
    /// Higher numbers indicate higher precedence.
    pub fn precedence(&self) -> Option<u8> {
        if self.kind != TokenKind::Identifier {
            return None;
        }

        match self.text.as_str() {
            // Highest precedence (tightest binding)
            "^" | "expt" => Some(7),
            
            // Multiplicative
            "*" | "/" | "%" | "quotient" | "remainder" | "modulo" | "div" | "mod" => Some(6),
            
            // Additive
            "+" | "-" => Some(5),
            
            // Relational
            "<" | "<=" | ">" | ">=" | "=" | "eq?" | "eqv?" | "equal?" |
            "string<?" | "string<=?" | "string>?" | "string>=?" | "string=?" => Some(4),
            
            // Logical AND
            "and" => Some(3),
            
            // Logical OR
            "or" => Some(2),
            
            // No precedence for other identifiers (they are not operators)
            _ => None,
        }
    }

    /// Returns true if this token is a right-associative operator.
    pub fn is_right_associative(&self) -> bool {
        if self.kind != TokenKind::Identifier {
            return false;
        }

        matches!(self.text.as_str(), "^" | "expt" | "cons")
    }

    /// Returns true if this token represents a binary operator.
    pub fn is_binary_operator(&self) -> bool {
        self.precedence().is_some()
    }

    /// Returns true if this token is a delimiter (parenthesis, bracket, brace).
    pub fn is_delimiter(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::LeftParen | TokenKind::RightParen |
            TokenKind::LeftBracket | TokenKind::RightBracket |
            TokenKind::LeftBrace | TokenKind::RightBrace
        )
    }
    
    /// Returns true if this token is an opening delimiter.
    pub fn is_opening_delimiter(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace
        )
    }
    
    /// Returns true if this token is a closing delimiter.
    pub fn is_closing_delimiter(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace
        )
    }
    
    /// Returns the matching closing delimiter for an opening delimiter.
    pub fn matching_delimiter(&self) -> Option<TokenKind> {
        match self.kind {
            TokenKind::LeftParen => Some(TokenKind::RightParen),
            TokenKind::LeftBracket => Some(TokenKind::RightBracket),
            TokenKind::LeftBrace => Some(TokenKind::RightBrace),
            TokenKind::RightParen => Some(TokenKind::LeftParen),
            TokenKind::RightBracket => Some(TokenKind::LeftBracket),
            TokenKind::RightBrace => Some(TokenKind::LeftBrace),
            _ => None,
        }
    }
    
    /// Returns true if this token represents a quote-like form.
    pub fn is_quote_like(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Quote | TokenKind::Quasiquote | 
            TokenKind::Unquote | TokenKind::UnquoteSplicing
        )
    }
    
    /// Attempts to parse this token as a numeric value if it's a number token.
    pub fn parse_number(&self) -> Option<NumericValue> {
        match self.kind {
            TokenKind::IntegerNumber => {
                parse_integer(&self.text).map(NumericValue::Integer)
            }
            TokenKind::RealNumber => {
                parse_real(&self.text).map(NumericValue::Real)
            }
            TokenKind::RationalNumber => {
                parse_rational(&self.text).map(NumericValue::Rational)
            }
            TokenKind::ComplexNumber => {
                parse_complex(&self.text).map(NumericValue::Complex)
            }
            _ => None,
        }
    }
    
    /// Attempts to parse this token as a string value if it's a string token.
    pub fn parse_string(&self) -> crate::diagnostics::Result<String> {
        if self.kind != TokenKind::String {
            return Err(crate::diagnostics::Error::internal_error(
                "Attempted to parse non-string token as string"
            ).into())
        }
        
        // Remove surrounding quotes
        let content = &self.text[1..self.text.len()-1];
        unescape_string(content)
    }
    
    /// Attempts to parse this token as a character value if it's a character token.
    pub fn parse_character(&self) -> crate::diagnostics::Result<char> {
        if self.kind != TokenKind::Character {
            return Err(crate::diagnostics::Error::internal_error(
                "Attempted to parse non-character token as character"
            ).into())
        }
        
        // Remove #\ prefix
        let content = &self.text[2..];
        parse_character_literal(content)
    }

    /// Returns the expected arity (number of arguments) for this identifier
    /// if it's a known built-in function.
    pub fn expected_arity(&self) -> Option<Arity> {
        if self.kind != TokenKind::Identifier {
            return None;
        }

        match self.text.as_str() {
            // Arithmetic - variable arity
            "+" | "*" => Some(Arity::Variable(0)),
            "-" | "/" => Some(Arity::Variable(1)),
            "abs" | "floor" | "ceiling" | "truncate" | "round" | "sqrt" | "exp" | "log" |
            "sin" | "cos" | "tan" | "asin" | "acos" => Some(Arity::Exact(1)),
            "atan" => Some(Arity::Range(1, 2)),
            "expt" | "remainder" | "quotient" | "modulo" | "gcd" | "lcm" => Some(Arity::Variable(1)),
            "max" | "min" => Some(Arity::Variable(1)),
            
            // Comparison - variable arity
            "=" | "<" | "<=" | ">" | ">=" => Some(Arity::Variable(2)),
            
            // Logical
            "and" | "or" => Some(Arity::Variable(0)),
            "not" => Some(Arity::Exact(1)),
            
            // List operations
            "cons" => Some(Arity::Exact(2)),
            "car" | "cdr" | "null?" | "pair?" | "list?" | "length" => Some(Arity::Exact(1)),
            "list" | "append" => Some(Arity::Variable(0)),
            "reverse" => Some(Arity::Exact(1)),
            "list-ref" | "list-set!" => Some(Arity::Exact(2)),
            "member" | "assoc" => Some(Arity::Exact(2)),
            
            // String operations
            "string" => Some(Arity::Variable(0)),
            "string-length" | "string->symbol" | "symbol->string" => Some(Arity::Exact(1)),
            "string-ref" | "string-set!" | "string-append" => Some(Arity::Variable(1)),
            "substring" => Some(Arity::Range(2, 3)),
            "string=?" | "string<?" | "string<=?" | "string>?" | "string>=?" => Some(Arity::Variable(2)),
            
            // Type predicates
            "number?" | "complex?" | "real?" | "rational?" | "integer?" | "exact?" | "inexact?" |
            "string?" | "char?" | "symbol?" | "boolean?" | "procedure?" | "vector?" | "port?" |
            "input-port?" | "output-port?" | "eof-object?" => Some(Arity::Exact(1)),
            
            // Vector operations
            "vector" => Some(Arity::Variable(0)),
            "vector-length" => Some(Arity::Exact(1)),
            "vector-ref" | "vector-set!" => Some(Arity::Exact(2)),
            "make-vector" => Some(Arity::Range(1, 2)),
            
            // I/O
            "display" | "write" | "write-char" | "newline" => Some(Arity::Range(0, 1)),
            "read" | "read-char" | "peek-char" => Some(Arity::Range(0, 1)),
            "open-input-file" | "open-output-file" => Some(Arity::Exact(1)),
            "close-input-port" | "close-output-port" => Some(Arity::Exact(1)),
            
            // Control flow
            "if" => Some(Arity::Range(2, 3)),
            "cond" => Some(Arity::Variable(1)),
            "case" => Some(Arity::Variable(2)),
            
            // Conversion functions
            "char->integer" | "integer->char" => Some(Arity::Exact(1)),
            "string->number" => Some(Arity::Range(1, 2)),
            "number->string" => Some(Arity::Range(1, 2)),
            
            _ => None,
        }
    }
}

/// Represents the arity (number of arguments) expected by a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arity {
    /// Exactly n arguments
    Exact(usize),
    /// At least n arguments
    Variable(usize),
    /// Between min and max arguments (inclusive)
    Range(usize, usize),
}

impl Arity {
    /// Checks if the given number of arguments is valid for this arity.
    pub fn accepts(&self, n: usize) -> bool {
        match self {
            Arity::Exact(expected) => n == *expected,
            Arity::Variable(min) => n >= *min,
            Arity::Range(min, max) => n >= *min && n <= *max,
        }
    }

    /// Returns a human-readable description of this arity.
    pub fn description(&self) -> String {
        match self {
            Arity::Exact(n) => {
                if *n == 1 {
                    "1 argument".to_string()
                } else {
                    format!("{n} arguments")
                }
            }
            Arity::Variable(min) => {
                if *min == 0 {
                    "any number of arguments".to_string()
                } else if *min == 1 {
                    "at least 1 argument".to_string()
                } else {
                    format!("at least {min} arguments")
                }
            }
            Arity::Range(min, max) => {
                format!("{min}-{max} arguments")
            }
        }
    }
}

/// Represents the different types of numeric values that can be parsed.
#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    /// Integer value
    Integer(i64),
    /// Real (floating-point) value
    Real(f64),
    /// Rational value (exact fraction)
    Rational(Rational),
    /// Complex value
    Complex(Complex),
}

impl NumericValue {
    /// Converts the numeric value to a floating-point approximation.
    pub fn to_f64(&self) -> f64 {
        match self {
            NumericValue::Integer(i) => *i as f64,
            NumericValue::Real(r) => *r,
            NumericValue::Rational(rat) => rat.to_f64(),
            NumericValue::Complex(c) => c.real, // Real part only
        }
    }
    
    /// Returns true if this numeric value is exact (integer or rational).
    pub fn is_exact(&self) -> bool {
        matches!(self, NumericValue::Integer(_) | NumericValue::Rational(_))
    }
    
    /// Returns true if this numeric value is inexact (real or complex).
    pub fn is_inexact(&self) -> bool {
        !self.is_exact()
    }
    
    /// Returns true if this numeric value represents a real number.
    pub fn is_real(&self) -> bool {
        match self {
            NumericValue::Complex(c) => c.imag == 0.0,
            _ => true,
        }
    }
    
    /// Returns true if this numeric value is an integer.
    pub fn is_integer(&self) -> bool {
        match self {
            NumericValue::Integer(_) => true,
            NumericValue::Real(r) => r.fract() == 0.0 && r.is_finite(),
            NumericValue::Rational(rat) => rat.denominator == 1,
            NumericValue::Complex(c) => c.imag == 0.0 && c.real.fract() == 0.0 && c.real.is_finite(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence() {
        let span = Span::new(0, 1);
        
        let plus = Token::identifier("+", span);
        let mult = Token::identifier("*", span);
        let exp = Token::identifier("^", span);
        
        assert!(plus.precedence() < mult.precedence());
        assert!(mult.precedence() < exp.precedence());
    }

    #[test]
    fn test_arity_checking() {
        let exact_arity = Arity::Exact(2);
        assert!(exact_arity.accepts(2));
        assert!(!exact_arity.accepts(1));
        assert!(!exact_arity.accepts(3));

        let variable_arity = Arity::Variable(1);
        assert!(!variable_arity.accepts(0));
        assert!(variable_arity.accepts(1));
        assert!(variable_arity.accepts(10));

        let range_arity = Arity::Range(1, 3);
        assert!(!range_arity.accepts(0));
        assert!(range_arity.accepts(1));
        assert!(range_arity.accepts(2));
        assert!(range_arity.accepts(3));
        assert!(!range_arity.accepts(4));
    }

    #[test]
    fn test_binary_operators() {
        let span = Span::new(0, 1);
        
        let plus = Token::identifier("+", span);
        let ident = Token::identifier("foo", span);
        
        assert!(plus.is_binary_operator());
        assert!(!ident.is_binary_operator());
    }

    #[test]
    fn test_right_associative() {
        let span = Span::new(0, 1);
        
        let exp = Token::identifier("^", span);
        let plus = Token::identifier("+", span);
        
        assert!(exp.is_right_associative());
        assert!(!plus.is_right_associative());
    }
    
    #[test]
    fn test_delimiter_checking() {
        let span = Span::new(0, 1);
        
        let left_paren = Token::new(TokenKind::LeftParen, span, "(".to_string());
        let right_paren = Token::new(TokenKind::RightParen, span, ")".to_string());
        let ident = Token::identifier("foo", span);
        
        assert!(left_paren.is_delimiter());
        assert!(left_paren.is_opening_delimiter());
        assert!(!left_paren.is_closing_delimiter());
        
        assert!(right_paren.is_delimiter());
        assert!(!right_paren.is_opening_delimiter());
        assert!(right_paren.is_closing_delimiter());
        
        assert!(!ident.is_delimiter());
        
        assert_eq!(left_paren.matching_delimiter(), Some(TokenKind::RightParen));
        assert_eq!(right_paren.matching_delimiter(), Some(TokenKind::LeftParen));
        assert_eq!(ident.matching_delimiter(), None);
    }
    
    #[test]
    fn test_quote_like_checking() {
        let span = Span::new(0, 1);
        
        let quote = Token::new(TokenKind::Quote, span, "'".to_string());
        let quasiquote = Token::new(TokenKind::Quasiquote, span, "`".to_string());
        let ident = Token::identifier("foo", span);
        
        assert!(quote.is_quote_like());
        assert!(quasiquote.is_quote_like());
        assert!(!ident.is_quote_like());
    }
    
    #[test]
    fn test_number_parsing() {
        let span = Span::new(0, 3);
        
        let int_token = Token::integer("42", span);
        let real_token = Token::real("3.14", span);
        let rational_token = Token::rational("1/2", span);
        let complex_token = Token::complex("3+4i", span);
        
        assert!(int_token.parse_number().is_some());
        assert!(real_token.parse_number().is_some());
        assert!(rational_token.parse_number().is_some());
        assert!(complex_token.parse_number().is_some());
        
        let ident = Token::identifier("foo", span);
        assert!(ident.parse_number().is_none());
    }
    
    #[test]
    fn test_numeric_value_properties() {
        let int_val = NumericValue::Integer(42);
        let real_val = NumericValue::Real(3.05);
        let rat_val = NumericValue::Rational(Rational::new(1, 2).unwrap());
        let complex_val = NumericValue::Complex(Complex::new(3.0, 4.0));
        
        assert!(int_val.is_exact());
        assert!(int_val.is_integer());
        assert!(int_val.is_real());
        
        assert!(!real_val.is_exact());
        assert!(real_val.is_inexact());
        assert!(!real_val.is_integer());
        assert!(real_val.is_real());
        
        assert!(rat_val.is_exact());
        assert!(!rat_val.is_integer());
        assert!(rat_val.is_real());
        
        assert!(!complex_val.is_real());
        assert!(complex_val.is_inexact());
    }
}