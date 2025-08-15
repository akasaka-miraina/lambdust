//! Token structure definition for Lambdust.

use crate::diagnostics::Span;
use crate::utils::InternedString;
use std::fmt;
use std::sync::Arc;

use super::TokenKind;

/// Optimized text storage for tokens.
/// Uses different representations based on text length and commonality.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenText {
    /// Small text stored inline (up to 15 chars)
    Inline(SmallString),
    /// Interned string for common tokens
    Interned(InternedString),
    /// Regular Arc<str> for larger strings
    Shared(Arc<str>),
}

/// Small string storage for inline text (16 bytes total)
#[derive(Debug, Clone, PartialEq)]
pub struct SmallString {
    data: [u8; 15],
    len: u8,
}

impl SmallString {
    /// Creates a new SmallString from a string slice.
    pub fn new(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() > 15 {
            return None;
        }
        
        let mut data = [0u8; 15];
        data[..bytes.len()].copy_from_slice(bytes);
        
        Some(Self {
            data,
            len: bytes.len() as u8,
        })
    }
    
    /// Returns the string contents.
    pub fn as_str(&self) -> &str {
        let bytes = &self.data[..self.len as usize];
        std::str::from_utf8(bytes).unwrap_or("")
    }
}

impl TokenText {
    /// Creates optimized token text from a string.
    pub fn new(s: impl Into<String>) -> Self {
        let string = s.into();
        
        // Try inline storage first
        if let Some(small) = SmallString::new(&string) {
            return Self::Inline(small);
        }
        
        // Check if it's a common token that should be interned
        if is_common_token(&string) {
            return Self::Interned(crate::utils::intern(&string));
        }
        
        // Use shared storage for larger strings
        Self::Shared(string.into())
    }
    
    /// Returns the string contents.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Inline(small) => small.as_str(),
            Self::Interned(interned) => interned.as_str(),
            Self::Shared(arc) => arc,
        }
    }
    
    /// Returns the string length.
    pub fn len(&self) -> usize {
        match self {
            Self::Inline(small) => small.len as usize,
            Self::Interned(interned) => interned.len(),
            Self::Shared(arc) => arc.len(),
        }
    }
    
    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Checks if a token string is common enough to warrant interning.
fn is_common_token(s: &str) -> bool {
    matches!(s,
        // Common Scheme keywords and forms
        "define" | "lambda" | "let" | "let*" | "letrec" | "if" | "cond" | "case" |
        "and" | "or" | "not" | "begin" | "do" | "when" | "unless" | "quote" |
        "quasiquote" | "unquote" | "unquote-splicing" | "syntax" | "syntax-case" |
        
        // Common operators
        "+" | "-" | "*" | "/" | "=" | "<" | ">" | "<=" | ">=" | "eq?" | "eqv?" | "equal?" |
        
        // Common predicates
        "null?" | "pair?" | "list?" | "number?" | "string?" | "symbol?" | "boolean?" |
        "procedure?" | "vector?" | "char?" | "port?" | "input-port?" | "output-port?" |
        
        // Common list operations
        "car" | "cdr" | "cons" | "list" | "append" | "reverse" | "length" | "member" |
        "assoc" | "map" | "for-each" | "filter" |
        
        // Common I/O
        "read" | "write" | "display" | "newline" | "open-input-file" | "open-output-file" |
        "close-input-port" | "close-output-port" |
        
        // Common values and literals
        "#t" | "#f" | "()" | "else"
    )
}

/// A token in the Lambdust language.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// Source location information.
    pub span: Span,
    /// The optimized text content.
    text: TokenText,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { 
            kind, 
            span, 
            text: TokenText::new(text),
        }
    }

    /// Creates an EOF token.
    pub fn eof(span: Span) -> Self {
        Self::new(TokenKind::Eof, span, "".to_string())
    }

    /// Returns the text content of this token.
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
    
    /// Returns the text content as a string (for compatibility).
    pub fn lexeme(&self) -> &str {
        self.text.as_str()
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
    
    /// Returns the memory footprint of this token in bytes (approximate).
    pub fn memory_footprint(&self) -> usize {
        std::mem::size_of::<TokenKind>() + 
        std::mem::size_of::<Span>() + 
        match &self.text {
            TokenText::Inline(_) => std::mem::size_of::<SmallString>(),
            TokenText::Interned(_) => std::mem::size_of::<InternedString>(),
            TokenText::Shared(arc) => std::mem::size_of::<Arc<str>>() + arc.len(),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text.is_empty() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{}", self.text.as_str())
        }
    }
}

impl std::ops::Deref for Token {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        self.text.as_str()
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.text.as_str()
    }
}