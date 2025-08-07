//! Token structure definition for Lambdust.

use crate::diagnostics::Span;
use std::fmt;

use super::TokenKind;

/// A token in the Lambdust language.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// Source location information.
    pub span: Span,
    /// The raw text of the token.
    pub text: String,
    /// Alias for text to maintain compatibility.
    pub lexeme: String,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { 
            kind, 
            span, 
            lexeme: text.clone(),
            text 
        }
    }

    /// Creates an EOF token.
    pub fn eof(span: Span) -> Self {
        Self::new(TokenKind::Eof, span, "".to_string())
    }

    /// Returns true if this token is an opening parenthesis.
    pub fn is_open_paren(&self) -> bool {
        matches!(self.kind, TokenKind::LeftParen)
    }

    /// Returns true if this token is a closing parenthesis.
    pub fn is_close_paren(&self) -> bool {
        matches!(self.kind, TokenKind::RightParen)
    }

    /// Returns true if this token is EOF.
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// Returns true if this token is a literal value.
    pub fn is_literal(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::IntegerNumber | TokenKind::RealNumber | TokenKind::RationalNumber | TokenKind::ComplexNumber |
            TokenKind::String | TokenKind::Character | TokenKind::Boolean
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text.is_empty() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{}", self.text)
        }
    }
}