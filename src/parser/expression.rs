//! Expression parsing utilities and helper functions.
//!
//! This module provides reusable helper functions for common parsing patterns
//! used throughout the Lambdust parser.

use super::Parser;
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::TokenKind;
use std::collections::HashMap;

impl Parser {
    /// Parses a sequence of expressions until a closing delimiter.
    /// 
    /// This is useful for parsing bodies, argument lists, etc.
    pub fn parse_expression_sequence(&mut self, delimiter: TokenKind) -> Result<Vec<Spanned<Expr>>> {
        let mut expressions = Vec::new();
        
        while !self.check(&delimiter) && !self.is_at_end() {
            match self.parse_expression() {
                Ok(expr) => {
                    expressions.push(expr);
                    self.panic_mode = false;
                }
                Err(err) => {
                    self.errors.push(*err);
                    if self.aggressive_recovery {
                        // Try to recover by skipping the problematic token
                        self.advance();
                        if self.errors.len() >= self.max_errors {
                            break;
                        }
                    } else {
                        return Err(Box::new(self.errors.last().unwrap().clone().into()))
                    }
                }
            }
            self.skip_whitespace();
        }
        
        Ok(expressions)
    }
    
    /// Parses a delimited sequence (like parentheses, brackets, etc.).
    /// 
    /// Automatically handles opening/closing delimiters and provides context.
    pub fn parse_delimited_sequence<T, F>(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        context: &str,
        mut parser_fn: F,
    ) -> Result<T>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        self.consume(&open, &format!("Expected {}", Self::token_kind_name(&open)))?;
        self.nesting_depth += 1;
        
        let result = self.with_context(context, |parser| {
            parser.skip_whitespace();
            parser_fn(parser)
        });
        
        self.nesting_depth -= 1;
        
        match result {
            Ok(value) => {
                self.consume(&close, &format!("Expected {}", Self::token_kind_name(&close)))?;
                Ok(value)
            }
            Err(err) => {
                // Try to recover by finding the closing delimiter
                self.synchronize_to_closing_paren();
                Err(err)
            }
        }
    }
    
    /// Parses an optional expression (returns None if next token isn't an expression start).
    pub fn parse_optional_expression(&mut self) -> Result<Option<Spanned<Expr>>> {
        if self.looks_like_expression_start() {
            Ok(Some(self.parse_expression()?))
        } else {
            Ok(None)
        }
    }
    
    /// Checks if the current token looks like the start of an expression.
    pub fn looks_like_expression_start(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        matches!(self.current_token().kind, 
            TokenKind::LeftParen | 
            TokenKind::IntegerNumber | TokenKind::RealNumber | 
            TokenKind::RationalNumber | TokenKind::ComplexNumber |
            TokenKind::String | TokenKind::Character | TokenKind::Boolean |
            TokenKind::Identifier | TokenKind::Keyword |
            TokenKind::Quote | TokenKind::Quasiquote)
    }
    
    /// Parses a comma-separated list of expressions.
    /// 
    /// Note: This is for potential future extensions - standard Scheme doesn't use commas.
    pub fn parse_comma_separated_expressions(&mut self) -> Result<Vec<Spanned<Expr>>> {
        let mut expressions = Vec::new();
        
        // Parse first expression
        if self.looks_like_expression_start() {
            expressions.push(self.parse_expression()?);
            
            // Parse remaining expressions
            while self.check(&TokenKind::Unquote) { // Using Unquote as comma for now
                self.advance(); // consume comma
                self.skip_whitespace();
                
                if !self.looks_like_expression_start() {
                    return Err(Box::new(Error::parse_error(
                        "Expected expression after comma",
                        self.current_span(),
                    ).into()))
                }
                
                expressions.push(self.parse_expression()?);
            }
        }
        
        Ok(expressions)
    }
    
    /// Validates that an expression is a valid identifier.
    pub fn expect_identifier<'a>(&self, expr: &'a Spanned<Expr>) -> Result<&'a str> {
        match &expr.inner {
            Expr::Identifier(name) => {
                Self::validate_identifier(name, expr.span)?;
                Ok(name)
            }
            _ => Err(Box::new(Error::parse_error(
                "Expected identifier",
                expr.span,
            ).into()))
        }
    }
    
    /// Validates that an expression is a valid literal.
    pub fn expect_literal<'a>(&self, expr: &'a Spanned<Expr>) -> Result<&'a Literal> {
        match &expr.inner {
            Expr::Literal(lit) => Ok(lit),
            _ => Err(Box::new(Error::parse_error(
                "Expected literal value",
                expr.span,
            ).into()))
        }
    }
    
    /// Creates a nil expression at the given span.
    pub fn make_nil(&self, span: Span) -> Spanned<Expr> {
        Spanned::new(Expr::Literal(Literal::Nil), span)
    }
    
    /// Creates a boolean expression.
    pub fn make_boolean(&self, value: bool, span: Span) -> Spanned<Expr> {
        Spanned::new(Expr::Literal(Literal::Boolean(value)), span)
    }
    
    /// Creates an identifier expression with validation.
    pub fn make_identifier(&self, name: String, span: Span) -> Result<Spanned<Expr>> {
        Self::validate_identifier(&name, span)?;
        Ok(Spanned::new(Expr::Identifier(name), span))
    }
    
    /// Tries to parse an expression with error recovery.
    /// 
    /// If parsing fails, attempts to skip to a recovery point and returns None.
    pub fn try_parse_expression(&mut self) -> Option<Spanned<Expr>> {
        match self.parse_expression() {
            Ok(expr) => {
                self.panic_mode = false;
                Some(expr)
            }
            Err(err) => {
                self.errors.push(*err);
                
                if self.aggressive_recovery && self.can_recover_at_current_position() {
                    // Skip problematic token and try to continue
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
                
                None
            }
        }
    }
    
    /// Returns a human-readable name for a token kind.
    pub fn token_kind_name(kind: &TokenKind) -> &'static str {
        match kind {
            TokenKind::LeftParen => "opening parenthesis '('",
            TokenKind::RightParen => "closing parenthesis ')'",
            TokenKind::LeftBracket => "opening bracket '['",
            TokenKind::RightBracket => "closing bracket ']'",
            TokenKind::LeftBrace => "opening brace '{'",
            TokenKind::RightBrace => "closing brace '}'",
            TokenKind::Quote => "quote '",
            TokenKind::Quasiquote => "quasiquote `",
            TokenKind::Unquote => "unquote ,",
            TokenKind::UnquoteSplicing => "unquote-splicing ,@",
            TokenKind::Dot => "dot",
            TokenKind::TypeAnnotation => "type annotation ::",
            TokenKind::IntegerNumber => "integer",
            TokenKind::RealNumber => "real number",
            TokenKind::RationalNumber => "rational number",
            TokenKind::ComplexNumber => "complex number",
            TokenKind::String => "string",
            TokenKind::Character => "character",
            TokenKind::Boolean => "boolean",
            TokenKind::Keyword => "keyword",
            TokenKind::Identifier => "identifier",
            TokenKind::LineComment => "line comment",
            TokenKind::BlockComment => "block comment",
            TokenKind::Eof => "end of file",
            TokenKind::Error => "error token",
        }
    }
    
    /// Parses a binding pattern (identifier or destructuring pattern).
    /// 
    /// For now, only supports simple identifiers, but could be extended
    /// for destructuring in the future.
    pub fn parse_binding_pattern(&mut self) -> Result<String> {
        if !self.check(&TokenKind::Identifier) {
            return Err(Box::new(Error::parse_error(
                "Expected identifier in binding pattern",
                self.current_span(),
            ).into()))
        }
        
        let name = self.current_token().text.clone();
        Self::validate_identifier(&name, self.current_span())?;
        self.advance();
        
        Ok(name)
    }
    
    /// Checks if a name conflicts with existing bindings in a scope.
    /// 
    /// This is a placeholder for future scope analysis.
    pub fn check_binding_conflict(&self, _name: &str, _span: Span) -> Result<()> {
        // TODO: Implement proper scope checking
        Ok(())
    }
    
    /// Parses optional metadata (keywords and their values).
    /// 
    /// Returns empty map if no metadata is found.
    pub fn parse_optional_metadata(&mut self) -> Result<HashMap<String, Spanned<Expr>>> {
        if self.check(&TokenKind::Keyword) {
            self.parse_metadata_exprs()
        } else {
            Ok(HashMap::new())
        }
    }
    
    /// Attempts to parse a specific token kind, providing helpful error messages.
    pub fn expect_token(&mut self, expected: TokenKind, context: &str) -> Result<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(Box::new(Error::parse_error(
                format!(
                    "Expected {} in {}, found {}",
                    Self::token_kind_name(&expected),
                    context,
                    self.current_token_text()
                ),
                self.current_span(),
            ).into()))
        }
    }
    
    /// Skips tokens until a synchronization point is found.
    /// 
    /// This is more sophisticated than the basic synchronize method.
    pub fn skip_to_synchronization_point(&mut self, targets: &[TokenKind]) {
        while !self.is_at_end() {
            if targets.contains(&self.current_token().kind) {
                break;
            }
            
            // Also stop at common synchronization points
            match self.current_token().kind {
                TokenKind::LeftParen | TokenKind::RightParen | TokenKind::Eof => break,
                _ => { self.advance(); },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::lexer::{Lexer, TokenKind};

    fn create_test_parser(source: &str) -> Parser {
        let mut lexer = Lexer::new(source, Some("test").into());
        let tokens = lexer.tokenize().unwrap();
        Parser::new(tokens)
    }

    #[test]
    fn test_looks_like_expression_start() {
        let mut parser = create_test_parser("42 foo (+ 1 2)");
        
        assert!(parser.looks_like_expression_start()); // 42
        parser.advance();
        
        assert!(parser.looks_like_expression_start()); // foo
        parser.advance();
        
        assert!(parser.looks_like_expression_start()); // (+ 1 2)
    }

    #[test]
    fn test_token_kind_name() {
        assert_eq!(Parser::token_kind_name(&TokenKind::LeftParen), "opening parenthesis '('");
        assert_eq!(Parser::token_kind_name(&TokenKind::Identifier), "identifier");
        assert_eq!(Parser::token_kind_name(&TokenKind::String), "string");
    }

    #[test]
    fn test_expect_identifier() {
        let parser = create_test_parser("test");
        let span = Span::new(0, 4);
        let expr = Spanned::new(Expr::Identifier("test".to_string()), span);
        
        assert_eq!(parser.expect_identifier(&expr).unwrap(), "test");
        
        let non_id = Spanned::new(Expr::Literal(Literal::Number(42.0)), span);
        assert!(parser.expect_identifier(&non_id).is_err());
    }

    #[test]
    fn test_make_helper_functions() {
        let parser = create_test_parser("");
        let span = Span::new(0, 1);
        
        let nil = parser.make_nil(span);
        assert!(matches!(nil.inner, Expr::Literal(Literal::Nil)));
        
        let bool_true = parser.make_boolean(true, span);
        assert!(matches!(bool_true.inner, Expr::Literal(Literal::Boolean(true))));
        
        let ident = parser.make_identifier("test".to_string(), span).unwrap();
        assert!(matches!(ident.inner, Expr::Identifier(ref name) if name == "test"));
    }
}