//! Syntax highlighting system for the enhanced REPL.

#![allow(dead_code, missing_docs)]

use crate::{Result, eval::Value};

#[cfg(feature = "enhanced-repl")]
use {
    nu_ansi_term::{Color, Style},
    crossterm::style::{Stylize, ContentStyle},
};

/// Color scheme for syntax highlighting
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub keyword: Style,
    pub function: Style,
    pub macro_name: Style,
    pub string: Style,
    pub number: Style,
    pub boolean: Style,
    pub comment: Style,
    pub paren: Style,
    pub bracket: Style,
    pub quote: Style,
    pub error: Style,
    pub warning: Style,
    pub success: Style,
}

impl Default for ColorScheme {
    fn default() -> Self {
        #[cfg(feature = "enhanced-repl")]
        {
            Self {
                keyword: Style::new().fg(Color::Cyan).bold(),
                function: Style::new().fg(Color::Blue),
                macro_name: Style::new().fg(Color::Magenta),
                string: Style::new().fg(Color::Green),
                number: Style::new().fg(Color::Yellow),
                boolean: Style::new().fg(Color::Red),
                comment: Style::new().fg(Color::DarkGray).italic(),
                paren: Style::new().fg(Color::White).bold(),
                bracket: Style::new().fg(Color::Cyan),
                quote: Style::new().fg(Color::Purple),
                error: Style::new().fg(Color::Red).bold(),
                warning: Style::new().fg(Color::Yellow).bold(),
                success: Style::new().fg(Color::Green).bold(),
            }
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            // Fallback for when enhanced REPL features are not available
            Self {
                keyword: Style::new(),
                function: Style::new(),
                macro_name: Style::new(),
                string: Style::new(),
                number: Style::new(),
                boolean: Style::new(),
                comment: Style::new(),
                paren: Style::new(),
                bracket: Style::new(),
                quote: Style::new(),
                error: Style::new(),
                warning: Style::new(),
                success: Style::new(),
            }
        }
    }
}

/// Represents a syntax token for highlighting
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxToken {
    pub text: String,
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

/// Types of syntax tokens
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,
    Function,
    Macro,
    String,
    Number,
    Boolean,
    Comment,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Quote,
    QuasiQuote,
    Unquote,
    UnquoteSplicing,
    Symbol,
    Whitespace,
    Error,
}

/// The main syntax highlighter
pub struct SyntaxHighlighter {
    color_scheme: ColorScheme,
    keywords: std::collections::HashSet<String>,
    builtin_functions: std::collections::HashSet<String>,
    macros: std::collections::HashSet<String>,
}

impl SyntaxHighlighter {
    pub fn new() -> Result<Self> {
        let mut highlighter = Self {
            color_scheme: ColorScheme::default(),
            keywords: std::collections::HashSet::new(),
            builtin_functions: std::collections::HashSet::new(),
            macros: std::collections::HashSet::new(),
        };

        highlighter.initialize_keywords();
        highlighter.initialize_builtin_functions();
        highlighter.initialize_macros();

        Ok(highlighter)
    }

    pub fn with_color_scheme(color_scheme: ColorScheme) -> Result<Self> {
        let mut highlighter = Self::new()?;
        highlighter.color_scheme = color_scheme;
        Ok(highlighter)
    }

    fn initialize_keywords(&mut self) {
        let keywords = vec![
            "define", "lambda", "if", "cond", "case", "and", "or", "not",
            "let", "let*", "letrec", "begin", "quote", "quasiquote", 
            "unquote", "unquote-splicing", "set!", "else", "=>",
            "call/cc", "call-with-current-continuation", "values", 
            "call-with-values", "dynamic-wind", "eval", "apply",
            "import", "export", "library", "include", "include-ci",
        ];

        for keyword in keywords {
            self.keywords.insert(keyword.to_string());
        }
    }

    fn initialize_builtin_functions(&mut self) {
        let functions = vec![
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
            "car", "cdr", "cons", "list", "length", "append", "reverse",
            "map", "filter", "fold-left", "fold-right", "for-each",
            "null?", "pair?", "list?", "number?", "string?", "symbol?", 
            "boolean?", "procedure?", "vector?", "bytevector?",
            "display", "write", "newline", "read", "open-input-file",
            "open-output-file", "close-input-port", "close-output-port",
            "string-length", "string-append", "substring", "string=?",
            "string<?", "string>?", "string<=?", "string>=?",
            "char=?", "char<?", "char>?", "char<=?", "char>=?",
            "vector-length", "vector-ref", "vector-set!", "make-vector",
            "error", "raise", "with-exception-handler", "guard",
        ];

        for function in functions {
            self.builtin_functions.insert(function.to_string());
        }
    }

    fn initialize_macros(&mut self) {
        let macros = vec![
            "syntax-rules", "define-syntax", "let-syntax", "letrec-syntax",
            "syntax-case", "syntax", "quasisyntax", "unsyntax", "unsyntax-splicing",
            "with-syntax", "parameterize", "unless", "when", "do",
        ];

        for macro_name in macros {
            self.macros.insert(macro_name.to_string());
        }
    }

    /// Tokenizes input for syntax highlighting
    pub fn tokenize(&self, input: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let start = i;
            
            match chars[i] {
                // Whitespace
                c if c.is_whitespace() => {
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    tokens.push(SyntaxToken {
                        text: chars[start..i].iter().collect(),
                        token_type: TokenType::Whitespace,
                        start,
                        end: i,
                    });
                }
                
                // Comments
                ';' => {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    tokens.push(SyntaxToken {
                        text: chars[start..i].iter().collect(),
                        token_type: TokenType::Comment,
                        start,
                        end: i,
                    });
                }
                
                // Strings
                '"' => {
                    i += 1; // Skip opening quote
                    while i < chars.len() && chars[i] != '"' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 2; // Skip escaped character
                        } else {
                            i += 1;
                        }
                    }
                    if i < chars.len() {
                        i += 1; // Skip closing quote
                    }
                    tokens.push(SyntaxToken {
                        text: chars[start..i].iter().collect(),
                        token_type: TokenType::String,
                        start,
                        end: i,
                    });
                }
                
                // Parentheses and brackets
                '(' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: "(".to_string(),
                        token_type: TokenType::LeftParen,
                        start,
                        end: i,
                    });
                }
                ')' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: ")".to_string(),
                        token_type: TokenType::RightParen,
                        start,
                        end: i,
                    });
                }
                '[' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: "[".to_string(),
                        token_type: TokenType::LeftBracket,
                        start,
                        end: i,
                    });
                }
                ']' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: "]".to_string(),
                        token_type: TokenType::RightBracket,
                        start,
                        end: i,
                    });
                }
                
                // Quote forms
                '\'' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: "'".to_string(),
                        token_type: TokenType::Quote,
                        start,
                        end: i,
                    });
                }
                '`' => {
                    i += 1;
                    tokens.push(SyntaxToken {
                        text: "`".to_string(),
                        token_type: TokenType::QuasiQuote,
                        start,
                        end: i,
                    });
                }
                ',' => {
                    i += 1;
                    if i < chars.len() && chars[i] == '@' {
                        i += 1;
                        tokens.push(SyntaxToken {
                            text: ",@".to_string(),
                            token_type: TokenType::UnquoteSplicing,
                            start,
                            end: i,
                        });
                    } else {
                        tokens.push(SyntaxToken {
                            text: ",".to_string(),
                            token_type: TokenType::Unquote,
                            start,
                            end: i,
                        });
                    }
                }
                
                // Numbers and symbols
                _ => {
                    let token_text = self.read_atom(&chars, &mut i);
                    let token_type = self.classify_atom(&token_text);
                    
                    tokens.push(SyntaxToken {
                        text: token_text,
                        token_type,
                        start,
                        end: i,
                    });
                }
            }
        }

        tokens
    }

    fn read_atom(&self, chars: &[char], i: &mut usize) -> String {
        let start = *i;
        
        while *i < chars.len() {
            match chars[*i] {
                c if c.is_whitespace() => break,
                '(' | ')' | '[' | ']' | '\'' | '`' | ',' | ';' | '"' => break,
                _ => *i += 1,
            }
        }
        
        chars[start..*i].iter().collect()
    }

    fn classify_atom(&self, atom: &str) -> TokenType {
        // Check for booleans
        if atom == "#t" || atom == "#f" || atom == "#true" || atom == "#false" {
            return TokenType::Boolean;
        }

        // Check for numbers
        if self.is_number(atom) {
            return TokenType::Number;
        }

        // Check for keywords
        if self.keywords.contains(atom) {
            return TokenType::Keyword;
        }

        // Check for builtin functions
        if self.builtin_functions.contains(atom) {
            return TokenType::Function;
        }

        // Check for macros
        if self.macros.contains(atom) {
            return TokenType::Macro;
        }

        // Default to symbol
        TokenType::Symbol
    }

    fn is_number(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        // Simple number detection - could be more sophisticated
        s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')
            && s.chars().any(|c| c.is_ascii_digit())
    }

    /// Highlights the given input text
    pub fn highlight(&self, input: &str) -> String {
        #[cfg(feature = "enhanced-repl")]
        {
            let tokens = self.tokenize(input);
            let mut result = String::new();

            for token in tokens {
                let styled_text = match token.token_type {
                    TokenType::Keyword => self.color_scheme.keyword.paint(&token.text).to_string(),
                    TokenType::Function => self.color_scheme.function.paint(&token.text).to_string(),
                    TokenType::Macro => self.color_scheme.macro_name.paint(&token.text).to_string(),
                    TokenType::String => self.color_scheme.string.paint(&token.text).to_string(),
                    TokenType::Number => self.color_scheme.number.paint(&token.text).to_string(),
                    TokenType::Boolean => self.color_scheme.boolean.paint(&token.text).to_string(),
                    TokenType::Comment => self.color_scheme.comment.paint(&token.text).to_string(),
                    TokenType::LeftParen | TokenType::RightParen => {
                        self.color_scheme.paren.paint(&token.text).to_string()
                    },
                    TokenType::LeftBracket | TokenType::RightBracket => {
                        self.color_scheme.bracket.paint(&token.text).to_string()
                    },
                    TokenType::Quote | TokenType::QuasiQuote | TokenType::Unquote | TokenType::UnquoteSplicing => {
                        self.color_scheme.quote.paint(&token.text).to_string()
                    },
                    _ => token.text,
                };
                result.push_str(&styled_text);
            }

            result
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            // Fallback: return input unchanged
            input.to_string()
        }
    }

    /// Highlights a value for output display
    pub fn highlight_value(&self, value: &Value) -> String {
        #[cfg(feature = "enhanced-repl")]
        {
            let value_str = value.to_string();
            match value {
                Value::Literal(crate::ast::Literal::Number(_)) => 
                    self.color_scheme.number.paint(&value_str).to_string(),
                Value::Literal(crate::ast::Literal::String(_)) => 
                    self.color_scheme.string.paint(&format!("\"{value_str}\"")).to_string(),
                Value::Literal(crate::ast::Literal::Boolean(_)) => 
                    self.color_scheme.boolean.paint(&value_str).to_string(),
                Value::Symbol(_) => value_str,
                Value::Pair(_, _) | Value::MutablePair(_, _) => 
                    self.highlight(&value_str),
                _ => value_str,
            }
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            value.to_string()
        }
    }

    /// Highlights an error message
    pub fn highlight_error(&self, error: &str) -> String {
        #[cfg(feature = "enhanced-repl")]
        {
            self.color_scheme.error.paint(error).to_string()
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            error.to_string()
        }
    }

    /// Highlights a warning message
    pub fn highlight_warning(&self, warning: &str) -> String {
        #[cfg(feature = "enhanced-repl")]
        {
            self.color_scheme.warning.paint(warning).to_string()
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            warning.to_string()
        }
    }

    /// Highlights a success message
    pub fn highlight_success(&self, message: &str) -> String {
        #[cfg(feature = "enhanced-repl")]
        {
            self.color_scheme.success.paint(message).to_string()
        }
        #[cfg(not(feature = "enhanced-repl"))]
        {
            message.to_string()
        }
    }

    /// Provides bracket matching information
    pub fn find_matching_bracket(&self, input: &str, position: usize) -> Option<usize> {
        let chars: Vec<char> = input.chars().collect();
        if position >= chars.len() {
            return None;
        }

        let current_char = chars[position];
        let (open_char, close_char, direction) = match current_char {
            '(' => ('(', ')', 1),
            ')' => ('(', ')', -1),
            '[' => ('[', ']', 1),
            ']' => ('[', ']', -1),
            _ => return None,
        };

        let mut depth = 0;
        let mut i = position as isize;

        loop {
            if chars[i as usize] == open_char {
                depth += direction;
            } else if chars[i as usize] == close_char {
                depth -= direction;
            }

            if depth == 0 && i as usize != position {
                return Some(i as usize);
            }

            i += direction;
            if i < 0 || i >= chars.len() as isize {
                break;
            }
        }

        None
    }

    /// Updates the color scheme
    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Adds custom keywords for highlighting
    pub fn add_keywords(&mut self, keywords: Vec<String>) {
        for keyword in keywords {
            self.keywords.insert(keyword);
        }
    }

    /// Adds custom functions for highlighting
    pub fn add_functions(&mut self, functions: Vec<String>) {
        for function in functions {
            self.builtin_functions.insert(function);
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            color_scheme: ColorScheme::default(),
            keywords: std::collections::HashSet::new(),
            builtin_functions: std::collections::HashSet::new(),
            macros: std::collections::HashSet::new(),
        })
    }
}

// Fallback Style implementation for when enhanced-repl feature is not enabled
#[cfg(not(feature = "enhanced-repl"))]
#[derive(Debug, Clone, Default)]
pub struct Style;

#[cfg(not(feature = "enhanced-repl"))]
impl Style {
    pub fn new() -> Self { Self }
    pub fn fg(self, _color: ()) -> Self { self }
    pub fn bold(self) -> Self { self }
    pub fn italic(self) -> Self { self }
    pub fn paint(&self, text: &str) -> String { text.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let tokens = highlighter.tokenize("(define (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))");
        
        // Should tokenize into various types
        assert!(!tokens.is_empty());
        
        // Check that we have some parentheses
        let paren_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == TokenType::LeftParen || t.token_type == TokenType::RightParen)
            .collect();
        assert!(!paren_tokens.is_empty());
        
        // Check that we have keywords
        let keyword_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == TokenType::Keyword)
            .collect();
        assert!(!keyword_tokens.is_empty());
    }

    #[test]
    fn test_atom_classification() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        
        assert_eq!(highlighter.classify_atom("define"), TokenType::Keyword);
        assert_eq!(highlighter.classify_atom("+"), TokenType::Function);
        assert_eq!(highlighter.classify_atom("#t"), TokenType::Boolean);
        assert_eq!(highlighter.classify_atom("#f"), TokenType::Boolean);
        assert_eq!(highlighter.classify_atom("42"), TokenType::Number);
        assert_eq!(highlighter.classify_atom("3.14"), TokenType::Number);
        assert_eq!(highlighter.classify_atom("foo"), TokenType::Symbol);
    }

    #[test]
    fn test_bracket_matching() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let input = "(+ 1 (- 3 2))";
        
        // Find matching bracket for opening paren at position 0
        let matching = highlighter.find_matching_bracket(input, 0);
        assert_eq!(matching, Some(12)); // Should match the last closing paren
        
        // Find matching bracket for opening paren at position 5
        let matching = highlighter.find_matching_bracket(input, 5);
        assert_eq!(matching, Some(11)); // Should match the corresponding closing paren
    }

    #[test]
    fn test_number_detection() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        
        assert!(highlighter.is_number("42"));
        assert!(highlighter.is_number("3.14"));
        assert!(highlighter.is_number("-17"));
        assert!(highlighter.is_number("+23"));
        assert!(highlighter.is_number("1e10"));
        assert!(highlighter.is_number("2.5e-3"));
        
        assert!(!highlighter.is_number(""));
        assert!(!highlighter.is_number("abc"));
        assert!(!highlighter.is_number("12abc"));
    }
}