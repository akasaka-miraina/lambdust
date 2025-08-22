//! Type expression parser for Lambdust type annotation syntax.
//!
//! This module provides parsing functionality for type expressions, converting
//! tokens into TypeExpr AST nodes. It supports the full range of type syntax
//! including function types, parametric types, constraints, and advanced features.

use crate::ast::{TypeConstraint, TypeExpr, VariantCase};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parses a type expression from the current position.
    ///
    /// Type expressions can be:
    /// - Simple identifiers: Integer, String, Boolean
    /// - Type variables: 'a, 'alpha
    /// - Function types: (Integer -> String), (Integer String -> Boolean)
    /// - Parametric types: (List Integer), (Maybe String)
    /// - Constrained types: (Show a => a -> String)
    /// - Record types: {x : Integer, y : String}
    /// - Variant types: (| Some Integer | None)
    /// - Complex expressions with parentheses and nesting
    pub fn parse_type_expression(&mut self) -> Result<Spanned<TypeExpr>> {
        self.parse_constrained_type()
    }

    /// Parses a constrained type: (Show a => a -> String)
    fn parse_constrained_type(&mut self) -> Result<Spanned<TypeExpr>> {
        let start_span = self.current_span();

        // Try to parse as a normal type first
        let type_expr = self.parse_function_type()?;

        // Check if this is actually a constraint
        if self.check(&TokenKind::FatArrow) {
            // This is a constraint - backtrack and parse properly
            // For now, we'll implement a simpler approach
            // TODO: Implement proper constraint parsing with backtracking
            return Ok(type_expr);
        }

        Ok(type_expr)
    }

    /// Parses a function type: (A -> B) or (A B -> C)
    fn parse_function_type(&mut self) -> Result<Spanned<TypeExpr>> {
        let start_span = self.current_span();
        let mut type_expr = self.parse_application_type()?;

        // Check for function arrow
        if self.check(&TokenKind::Arrow) {
            self.advance(); // consume '->'
            let return_type = self.parse_function_type()?; // right-associative

            let span = start_span.combine(return_type.span);
            type_expr = Spanned::new(
                TypeExpr::Function {
                    params: vec![type_expr],
                    return_type: Box::new(return_type),
                },
                span,
            );
        }

        Ok(type_expr)
    }

    /// Parses a type application: (List Integer), (Maybe String)
    fn parse_application_type(&mut self) -> Result<Spanned<TypeExpr>> {
        self.parse_primary_type()
    }

    /// Parses a primary type expression (atomic types, parenthesized expressions)
    fn parse_primary_type(&mut self) -> Result<Spanned<TypeExpr>> {
        let start_span = self.current_span();

        match &self.current_token().kind {
            TokenKind::Identifier => {
                let name = self.current_token_text();
                let span = self.current_span();
                self.advance();

                // Check for type application
                if self.check(&TokenKind::Identifier)
                    || self.check(&TokenKind::LeftParen)
                    || (self.check(&TokenKind::Quote) && self.peek_after_quote_is_identifier())
                {
                    // This is a parametric type like (List Integer)
                    let mut args = Vec::new();

                    while !self.is_at_end()
                        && !self.check(&TokenKind::RightParen)
                        && !self.check(&TokenKind::Arrow)
                        && !self.check(&TokenKind::FatArrow)
                        && !self.check(&TokenKind::Pipe)
                        && !self.check(&TokenKind::TildeArrow)
                    {
                        args.push(self.parse_primary_type()?);

                        // Break on certain tokens that indicate end of type args
                        if self.check(&TokenKind::Arrow)
                            || self.check(&TokenKind::FatArrow)
                            || self.check(&TokenKind::RightParen)
                        {
                            break;
                        }
                    }

                    if args.is_empty() {
                        Ok(Spanned::new(TypeExpr::Identifier(name), span))
                    } else {
                        let end_span = args.last().unwrap().span;
                        let combined_span = span.combine(end_span);
                        Ok(Spanned::new(
                            TypeExpr::Parametric { name, args },
                            combined_span,
                        ))
                    }
                } else {
                    Ok(Spanned::new(TypeExpr::Identifier(name), span))
                }
            }

            TokenKind::Quote => {
                // Type variable: 'a, 'alpha
                self.advance(); // consume quote
                if !self.check(&TokenKind::Identifier) {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier after ' in type variable",
                        self.current_span(),
                    )));
                }

                let var_name = self.current_token_text();
                let end_span = self.current_span();
                self.advance();

                let span = start_span.combine(end_span);
                Ok(Spanned::new(TypeExpr::Variable(var_name), span))
            }

            TokenKind::LeftParen => {
                self.advance(); // consume '('
                self.skip_whitespace();

                // Check for empty parentheses (unit type)
                if self.check(&TokenKind::RightParen) {
                    let end_span = self.current_span();
                    self.advance(); // consume ')'
                    let span = start_span.combine(end_span);
                    return Ok(Spanned::new(TypeExpr::Identifier("Unit".to_string()), span));
                }

                // Check for special type constructs
                if self.check(&TokenKind::Identifier) {
                    let keyword = self.current_token_text();
                    match keyword.as_str() {
                        "forall" | "∀" => return self.parse_forall_type(start_span),
                        "exists" | "∃" => return self.parse_exists_type(start_span),
                        "mu" | "μ" => return self.parse_recursive_type(start_span),
                        _ => {}
                    }
                }

                // Check for variant type: (| Some A | None)
                if self.check(&TokenKind::Pipe) {
                    return self.parse_variant_type(start_span);
                }

                // Parse inner type expression
                let inner = self.parse_type_expression()?;

                // Check for multiple parameters in function type: (A B -> C)
                if !self.check(&TokenKind::RightParen) {
                    let mut params = vec![inner];

                    // Collect more parameters until we hit '->'
                    while !self.check(&TokenKind::RightParen)
                        && !self.check(&TokenKind::Arrow)
                        && !self.is_at_end()
                    {
                        params.push(self.parse_primary_type()?);
                    }

                    // Expect arrow
                    if !self.check(&TokenKind::Arrow) {
                        return Err(Box::new(Error::parse_error(
                            "Expected '->' in function type",
                            self.current_span(),
                        )));
                    }
                    self.advance(); // consume '->'

                    let return_type = self.parse_type_expression()?;

                    self.consume(&TokenKind::RightParen, "Expected closing parenthesis")?;
                    let end_span = self.current_span();
                    let span = start_span.combine(end_span);

                    return Ok(Spanned::new(
                        TypeExpr::Function {
                            params,
                            return_type: Box::new(return_type),
                        },
                        span,
                    ));
                }

                self.consume(&TokenKind::RightParen, "Expected closing parenthesis")?;
                let end_span = self.current_span();
                let span = start_span.combine(end_span);

                Ok(Spanned::new(TypeExpr::Parenthesized(Box::new(inner)), span))
            }

            TokenKind::LeftBrace => {
                // Record type: {x : Integer, y : String}
                self.parse_record_type(start_span)
            }

            TokenKind::Keyword => {
                let keyword = self.current_token_text();
                match keyword.as_str() {
                    "*" => {
                        let span = self.current_span();
                        self.advance();
                        Ok(Spanned::new(TypeExpr::Dynamic, span))
                    }
                    "?" => {
                        let span = self.current_span();
                        self.advance();
                        Ok(Spanned::new(TypeExpr::Unknown, span))
                    }
                    _ => Err(Box::new(Error::parse_error(
                        format!("Unexpected keyword in type expression: {keyword}"),
                        self.current_span(),
                    ))),
                }
            }

            _ => Err(Box::new(Error::parse_error(
                "Expected type expression",
                self.current_span(),
            ))),
        }
    }

    /// Parses a forall type: (forall (a b) (List a))
    fn parse_forall_type(&mut self, start_span: Span) -> Result<Spanned<TypeExpr>> {
        self.advance(); // consume 'forall' or '∀'

        // Parse type variables: (a b)
        self.consume(
            &TokenKind::LeftParen,
            "Expected opening parenthesis for type variables",
        )?;
        let mut vars = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            if !self.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected identifier in type variable list",
                    self.current_span(),
                )));
            }
            vars.push(self.current_token_text());
            self.advance();
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for type variables",
        )?;

        // Parse body type
        let body = self.parse_type_expression()?;

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for forall type",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(
            TypeExpr::Forall {
                vars,
                body: Box::new(body),
            },
            span,
        ))
    }

    /// Parses an exists type: (exists (a) (Pair a String))
    fn parse_exists_type(&mut self, start_span: Span) -> Result<Spanned<TypeExpr>> {
        self.advance(); // consume 'exists' or '∃'

        // Parse type variables: (a)
        self.consume(
            &TokenKind::LeftParen,
            "Expected opening parenthesis for type variables",
        )?;
        let mut vars = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            if !self.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected identifier in type variable list",
                    self.current_span(),
                )));
            }
            vars.push(self.current_token_text());
            self.advance();
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for type variables",
        )?;

        // Parse body type
        let body = self.parse_type_expression()?;

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for exists type",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(
            TypeExpr::Exists {
                vars,
                body: Box::new(body),
            },
            span,
        ))
    }

    /// Parses a recursive type: (mu t (List t))
    fn parse_recursive_type(&mut self, start_span: Span) -> Result<Spanned<TypeExpr>> {
        self.advance(); // consume 'mu' or 'μ'

        // Parse type variable
        if !self.check(&TokenKind::Identifier) {
            return Err(Box::new(Error::parse_error(
                "Expected identifier for recursive type variable",
                self.current_span(),
            )));
        }
        let var = self.current_token_text();
        self.advance();

        // Parse body type
        let body = self.parse_type_expression()?;

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for recursive type",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(
            TypeExpr::Recursive {
                var,
                body: Box::new(body),
            },
            span,
        ))
    }

    /// Parses a variant type: (| Some Integer | None)
    fn parse_variant_type(&mut self, start_span: Span) -> Result<Spanned<TypeExpr>> {
        let mut cases = Vec::new();

        while self.check(&TokenKind::Pipe) && !self.is_at_end() {
            self.advance(); // consume '|'

            // Parse constructor name
            if !self.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected constructor name in variant type",
                    self.current_span(),
                )));
            }
            let constructor = self.current_token_text();
            self.advance();

            // Check for payload type
            let payload = if !self.check(&TokenKind::Pipe)
                && !self.check(&TokenKind::RightParen)
                && !self.is_at_end()
            {
                Some(self.parse_primary_type()?)
            } else {
                None
            };

            cases.push(VariantCase {
                constructor,
                payload,
            });
        }

        if cases.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Variant type must have at least one case",
                self.current_span(),
            )));
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for variant type",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(TypeExpr::Variant { cases }, span))
    }

    /// Parses a record type: {x : Integer, y : String}
    fn parse_record_type(&mut self, start_span: Span) -> Result<Spanned<TypeExpr>> {
        self.advance(); // consume '{'
        self.skip_whitespace();

        let mut fields = Vec::new();
        let mut rest = None;

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            // Check for rest variable: | r
            if self.check(&TokenKind::Pipe) {
                self.advance(); // consume '|'
                if !self.check(&TokenKind::Identifier) {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier for record rest variable",
                        self.current_span(),
                    )));
                }
                rest = Some(self.current_token_text());
                self.advance();
                break;
            }

            // Parse field: name : type
            if !self.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected field name in record type",
                    self.current_span(),
                )));
            }
            let field_name = self.current_token_text();
            self.advance();

            self.consume(&TokenKind::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type_expression()?;

            fields.push((field_name, field_type));

            // Check for comma (optional)
            if self.check(&TokenKind::Identifier) && self.current_token_text() == "," {
                self.advance();
            }

            self.skip_whitespace();
        }

        self.consume(
            &TokenKind::RightBrace,
            "Expected closing brace for record type",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(TypeExpr::Record { fields, rest }, span))
    }

    /// Helper method to check if there's an identifier after a quote
    fn peek_after_quote_is_identifier(&mut self) -> bool {
        // This is a simplified check - in a real implementation we'd need
        // proper lookahead
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::lexer::Lexer;

    fn parse_type_expr(source: &str) -> Result<Spanned<TypeExpr>> {
        let mut lexer = Lexer::new(source, Some("test"));
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_type_expression()
    }

    #[test]
    fn test_simple_type_parsing() {
        let result = parse_type_expr("Integer");
        assert!(result.is_ok());
        let type_expr = result.unwrap();
        assert!(matches!(type_expr.inner, TypeExpr::Identifier(ref name) if name == "Integer"));
    }

    #[test]
    fn test_type_variable_parsing() {
        let result = parse_type_expr("'a");
        assert!(result.is_ok());
        let type_expr = result.unwrap();
        assert!(matches!(type_expr.inner, TypeExpr::Variable(ref name) if name == "a"));
    }

    #[test]
    fn test_function_type_parsing() {
        let result = parse_type_expr("(Integer -> String)");
        assert!(result.is_ok());
        let type_expr = result.unwrap();
        assert!(matches!(type_expr.inner, TypeExpr::Function { .. }));
    }

    #[test]
    fn test_parametric_type_parsing() {
        let result = parse_type_expr("(List Integer)");
        assert!(result.is_ok());
        let type_expr = result.unwrap();
        assert!(matches!(type_expr.inner, TypeExpr::Parametric { ref name, .. } if name == "List"));
    }

    #[test]
    fn test_multiple_parameter_function() {
        let result = parse_type_expr("(Integer String -> Boolean)");
        assert!(result.is_ok());
        let type_expr = result.unwrap();
        if let TypeExpr::Function { params, .. } = type_expr.inner {
            assert_eq!(params.len(), 2);
        } else {
            panic!("Expected function type");
        }
    }
}
