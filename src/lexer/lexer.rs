//! The main lexer for Lambdust source code.

use crate::diagnostics::{Error, Result, Span, SourceMap};
use logos::Logos;
use std::sync::Arc;

use super::{Token, TokenKind};

/// The main lexer for Lambdust source code.
#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    filename: Option<&'a str>,
    _position: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source code.
    pub fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        Self {
            source,
            filename,
            _position: 0,
        }
    }

    /// Tokenizes the entire source code.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut lex = TokenKind::lexer(self.source);

        while let Some(token_result) = lex.next() {
            let span = lex.span();
            let span = Span::new(span.start, span.len());

            match token_result {
                Ok(kind) => {
                    // Skip comments but preserve newlines
                    if matches!(kind, TokenKind::LineComment | TokenKind::BlockComment) {
                        continue;
                    }

                    let token = Token::new(kind, span, self.source[span.start..span.end()].to_owned());
                    tokens.push(token);
                }
                Err(()) => {
                    let text = &self.source[span.start..span.end()];
                    return Err(Box::new(Error::lex_error(
                        format!("Unexpected character: '{text}'"),
                        span,
                    )))
                }
            }
        }

        // Add EOF token
        let eof_span = Span::new(self.source.len(), 0);
        tokens.push(Token::new(TokenKind::Eof, eof_span, String::new()));

        Ok(tokens)
    }

    /// Gets the current filename (if any).
    pub fn filename(&self) -> Option<&str> {
        self.filename
    }

    /// Gets the source code.
    pub fn source(&self) -> &str {
        self.source
    }

    /// Gets the source map if available (placeholder implementation).
    pub fn get_source_map(&self) -> Option<Arc<SourceMap>> {
        // TODO: Implement source map integration
        None
    }

    /// Validates the entire source for common issues before tokenizing.
    pub fn validate_source(&self) -> Vec<Error> {
        let mut errors = Vec::new();
        let mut open_parens = 0;
        let mut open_quotes = false;
        let mut position = 0;

        for (i, ch) in self.source.char_indices() {
            position = i;
            match ch {
                '(' => open_parens += 1,
                ')' => {
                    if open_parens == 0 {
                        let span = Span::new(i, 1);
                        errors.push(Error::lex_error(
                            "Unmatched closing parenthesis".to_string(),
                            span,
                        ));
                    } else {
                        open_parens -= 1;
                    }
                }
                '"' => open_quotes = !open_quotes,
                '\n' if open_quotes => {
                    let span = Span::new(i, 1);
                    errors.push(Error::lex_error(
                        "Unterminated string literal".to_string(),
                        span,
                    ));
                    open_quotes = false;
                }
                _ => {}
            }
        }

        // Check for unmatched opening parentheses
        if open_parens > 0 {
            let span = Span::new(position, 0);
            errors.push(Error::lex_error(
                format!("{open_parens} unmatched opening parenthesis(es)"),
                span,
            ));
        }

        // Check for unterminated string
        if open_quotes {
            let span = Span::new(position, 0);
            errors.push(Error::lex_error(
                "Unterminated string literal at end of file".to_string(),
                span,
            ));
        }

        errors
    }

    /// Tokenizes with validation, returning both tokens and validation errors.
    pub fn tokenize_with_validation(&mut self) -> (Result<Vec<Token>>, Vec<Error>) {
        let validation_errors = self.validate_source();
        let tokenize_result = self.tokenize();
        (tokenize_result, validation_errors)
    }
}