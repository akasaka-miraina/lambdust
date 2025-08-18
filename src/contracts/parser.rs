//! Contract parser for define/contract syntax and contract expressions.
//!
//! This module extends the main parser to support contract syntax including:
//!
//! - define/contract function definitions
//! - Contract expression parsing
//! - Contract combinator syntax
//! - Higher-order contract syntax

use crate::contracts::ast::{
    ContractExpr, ComparisonOp, DependentBinding, FunctionCase
};
use crate::ast::{Expr, Formals, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use std::collections::HashMap;

/// Contract parser for parsing contract expressions and define/contract syntax.
#[derive(Debug)]
pub struct ContractParser {
    /// Token stream
    tokens: Vec<Token>,
    /// Current position in token stream
    position: usize,
    /// Collected errors
    errors: Vec<Error>,
}

impl ContractParser {
    /// Creates a new contract parser.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            errors: Vec::new(),
        }
    }

    /// Returns collected errors.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Returns whether there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Checks if we're at the end of the token stream.
    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Gets the current token.
    fn current_token(&self) -> &Token {
        static EOF_TOKEN: std::sync::OnceLock<Token> = std::sync::OnceLock::new();
        
        if self.at_end() {
            EOF_TOKEN.get_or_init(|| Token::eof(Span::new(0, 0)))
        } else {
            &self.tokens[self.position]
        }
    }

    /// Advances to the next token.
    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.position += 1;
        }
        self.previous_token()
    }

    /// Gets the previous token.
    fn previous_token(&self) -> &Token {
        static EOF_TOKEN: std::sync::OnceLock<Token> = std::sync::OnceLock::new();
        
        if self.position == 0 {
            EOF_TOKEN.get_or_init(|| Token::eof(Span::new(0, 0)))
        } else {
            &self.tokens[self.position - 1]
        }
    }

    /// Checks if the current token matches a specific kind.
    fn check(&self, kind: &TokenKind) -> bool {
        if self.at_end() {
            false
        } else {
            &self.current_token().kind == kind
        }
    }

    /// Consumes a token if it matches the expected kind.
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expects a specific token kind and returns an error if not found.
    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<&Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            let error = Error::new_spanned(
                format!("{}: expected {:?}, found {:?}", message, kind, self.current_token().kind),
                self.current_token().span,
            );
            self.errors.push(error.clone());
            Err(Box::new(error))
        }
    }

    /// Parses a define/contract expression.
    pub fn parse_define_contract(&mut self) -> Result<Spanned<Expr>> {
        let start_token = self.current_token().clone();
        self.expect(TokenKind::LeftParen, "Expected opening paren for define/contract")?;
        self.expect(TokenKind::Identifier, "Expected 'define/contract'")?;

        // Parse function signature: (name formals) or just name
        let (name, formals, return_type) = self.parse_function_signature()?;
        
        // Parse contract expression
        let contract = self.parse_contract_expression()?;
        
        // Parse body expressions
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            body.push(self.parse_expression()?);
        }
        
        self.expect(TokenKind::RightParen, "Expected closing paren for define/contract")?;
        
        let end_span = self.previous_token().span;
        let span = Span::new(start_token.span.start, end_span.end());
        
        // Create a DefineContract expression
        let define_contract = Expr::DefineContract {
            name,
            formals,
            contract: Box::new(contract),
            return_type,
            body,
        };
        
        Ok(Spanned::new(define_contract, span))
    }

    /// Parses a function signature for define/contract.
    fn parse_function_signature(&mut self) -> Result<(String, Option<Formals>, Option<Spanned<ContractExpr>>)> {
        if self.check(&TokenKind::LeftParen) {
            // Function definition: (name formals)
            self.advance(); // consume (
            
            let name = self.parse_identifier()?;
            let formals = Some(self.parse_formals()?);
            
            // Check for return type annotation
            let return_type = if self.match_token(TokenKind::Colon) {
                Some(self.parse_contract_expression()?)
            } else {
                None
            };
            
            self.expect(TokenKind::RightParen, "Expected closing paren for function signature")?;
            Ok((name, formals, return_type))
        } else {
            // Variable definition: just name
            let name = self.parse_identifier()?;
            
            // Check for type annotation
            let return_type = if self.match_token(TokenKind::Colon) {
                Some(self.parse_contract_expression()?)
            } else {
                None
            };
            
            Ok((name, None, return_type))
        }
    }

    /// Parses a contract expression.
    pub fn parse_contract_expression(&mut self) -> Result<Spanned<ContractExpr>> {
        let start_span = self.current_token().span;
        
        if self.check(&TokenKind::LeftParen) {
            self.parse_compound_contract()
        } else if self.check(&TokenKind::Identifier) {
            let token_text = self.current_token().lexeme().to_string();
            if token_text == "*" {
                // Dynamic type: * (use any/c for now)
                self.advance();
                let span = self.previous_token().span;
                Ok(Spanned::new(ContractExpr::Any { location: span }, span))
            } else if token_text == "?" {
                // Unknown type: ? (use any/c for now)
                self.advance();
                let span = self.previous_token().span;
                Ok(Spanned::new(ContractExpr::Any { location: span }, span))
            } else {
                self.parse_predicate_or_reference()
            }
        } else {
            Err(Box::new(Error::new_spanned(
                "Expected contract expression".to_string(),
                start_span,
            )))
        }
    }

    /// Parses a compound contract expression.
    fn parse_compound_contract(&mut self) -> Result<Spanned<ContractExpr>> {
        let start_token = self.current_token().clone();
        self.expect(TokenKind::LeftParen, "Expected opening paren")?;
        
        if !self.check(&TokenKind::Identifier) {
            return Err(Box::new(Error::new_spanned(
                "Expected contract combinator name".to_string(),
                self.current_token().span,
            )));
        }
        
        let combinator_name = self.advance().lexeme().to_string();
        let result = match combinator_name.as_str() {
            "->" => self.parse_function_contract(),
            "->i" => self.parse_dependent_function_contract(),
            "->*" => self.parse_case_function_contract(),
            "and/c" => self.parse_and_contract(),
            "or/c" => self.parse_or_contract(),
            "not/c" => self.parse_not_contract(),
            "one-of/c" => self.parse_one_of_contract(),
            "between/c" => self.parse_between_contract(),
            "</c" | "<=/c" | ">/c" | ">=/c" | "=/c" | "!=/c" => {
                self.parse_comparison_contract(&combinator_name)
            }
            "listof" => self.parse_listof_contract(),
            "vectorof" => self.parse_vectorof_contract(),
            "hash/c" => self.parse_hash_contract(),
            "tuple/c" => self.parse_tuple_contract(),
            "list/c" => self.parse_list_contract(),
            "vector/c" => self.parse_vector_contract(),
            "flat/c" => self.parse_flat_contract(),
            "rec/c" => self.parse_recursive_contract(),
            "with-message/c" => self.parse_with_message_contract(),
            _ => {
                // Try parsing as parametric contract
                self.parse_parametric_contract(&combinator_name)
            }
        };
        
        self.expect(TokenKind::RightParen, "Expected closing paren")?;
        
        let end_span = self.previous_token().span;
        let span = Span::new(start_token.span.start, end_span.end());
        
        result.map(|contract| Spanned::new(contract, span))
    }

    /// Parses a predicate or contract reference.
    fn parse_predicate_or_reference(&mut self) -> Result<Spanned<ContractExpr>> {
        let token = self.advance();
        let name = token.lexeme().to_string().clone();
        let span = token.span;
        
        // Check if it's a predicate (ends with ?) or any/c, none/c
        if name.ends_with('?') || name == "any/c" || name == "none/c" {
            Ok(Spanned::new(
                ContractExpr::Predicate { name, location: span },
                span,
            ))
        } else {
            // Contract reference
            Ok(Spanned::new(
                ContractExpr::Reference { name, location: span },
                span,
            ))
        }
    }

    /// Parses a function contract: (-> domain ... codomain)
    fn parse_function_contract(&mut self) -> Result<ContractExpr> {
        let mut domain = Vec::new();
        let mut codomain = None;
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            let contract = self.parse_contract_expression()?;
            if codomain.is_none() {
                domain.push(contract);
            } else {
                return Err(Box::new(Error::new_spanned(
                    "Too many arguments in function contract".to_string(),
                    contract.span,
                )));
            }
            
            // Last contract is the codomain
            if !domain.is_empty() && !self.check(&TokenKind::RightParen) {
                // If there's another contract coming, the current last one becomes codomain
                if let Some(last) = domain.pop() {
                    if codomain.is_none() {
                        codomain = Some(Box::new(last))
                    }
                }
            }
        }
        
        // If we only have one contract, it's the codomain (nullary function)
        if domain.is_empty() && codomain.is_none() {
            return Err(Box::new(Error::new_spanned(
                "Function contract requires at least a codomain".to_string(),
                self.current_token().span,
            )));
        }
        
        if domain.len() == 1 && codomain.is_none() {
            // Single contract is the codomain
            codomain = Some(Box::new(domain.pop().unwrap()))
        } else if codomain.is_none() && !domain.is_empty() {
            // Last domain contract becomes codomain
            codomain = Some(Box::new(domain.pop().unwrap()))
        }
        
        Ok(ContractExpr::Function {
            domain,
            codomain: codomain.unwrap_or_else(|| {
                Box::new(Spanned::new(
                    ContractExpr::Any { location: Span::new(0, 0) },
                    Span::new(0, 0),
                ))
            }),
            location: self.current_token().span,
        })
    }

    /// Parses a dependent function contract: (->i ([x pred] ...) codomain)
    fn parse_dependent_function_contract(&mut self) -> Result<ContractExpr> {
        self.expect(TokenKind::LeftParen, "Expected opening paren for bindings")?;
        
        let mut bindings = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            self.expect(TokenKind::LeftBracket, "Expected opening bracket for binding")?;
            let name = self.parse_identifier()?;
            let contract = self.parse_contract_expression()?;
            self.expect(TokenKind::RightBracket, "Expected closing bracket for binding")?;
            
            bindings.push(DependentBinding::new(
                name,
                contract,
                None, // TODO: Parse dependency expressions
                self.previous_token().span,
            ))
        }
        
        self.expect(TokenKind::RightParen, "Expected closing paren for bindings")?;
        
        let codomain = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::DependentFunction {
            bindings,
            codomain,
            location: self.current_token().span,
        })
    }

    /// Parses a case function contract: (->* (domain ... codomain) ...)
    fn parse_case_function_contract(&mut self) -> Result<ContractExpr> {
        let mut cases = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            self.expect(TokenKind::LeftParen, "Expected opening paren for case")?;
            
            let mut domain = Vec::new();
            let mut codomain = None;
            
            while !self.check(&TokenKind::RightParen) && !self.at_end() {
                let contract = self.parse_contract_expression()?;
                domain.push(contract);
            }
            
            // Last contract is the codomain
            if let Some(last) = domain.pop() {
                codomain = Some(last);
            }
            
            self.expect(TokenKind::RightParen, "Expected closing paren for case")?;
            
            if let Some(cod) = codomain {
                cases.push(FunctionCase::new(domain, cod, self.previous_token().span))
            }
        }
        
        Ok(ContractExpr::CaseFunction {
            cases,
            location: self.current_token().span,
        })
    }

    /// Parses an and contract: (and/c contract ...)
    fn parse_and_contract(&mut self) -> Result<ContractExpr> {
        let mut contracts = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            contracts.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::And {
            contracts,
            location: self.current_token().span,
        })
    }

    /// Parses an or contract: (or/c contract ...)
    fn parse_or_contract(&mut self) -> Result<ContractExpr> {
        let mut contracts = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            contracts.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::Or {
            contracts,
            location: self.current_token().span,
        })
    }

    /// Parses a not contract: (not/c contract)
    fn parse_not_contract(&mut self) -> Result<ContractExpr> {
        let contract = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::Not {
            contract,
            location: self.current_token().span,
        })
    }

    /// Parses a one-of contract: (one-of/c value ...)
    fn parse_one_of_contract(&mut self) -> Result<ContractExpr> {
        let mut values = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            values.push(Box::new(self.parse_expression()?))
        }
        
        Ok(ContractExpr::OneOf {
            values,
            location: self.current_token().span,
        })
    }

    /// Parses a between contract: (between/c min max)
    fn parse_between_contract(&mut self) -> Result<ContractExpr> {
        let min = Box::new(self.parse_expression()?);
        let max = Box::new(self.parse_expression()?);
        
        Ok(ContractExpr::Between {
            min,
            max,
            location: self.current_token().span,
        })
    }

    /// Parses a comparison contract: (</c value)
    fn parse_comparison_contract(&mut self, operator_str: &str) -> Result<ContractExpr> {
        let operator = match operator_str {
            "</c" => ComparisonOp::LessThan,
            "<=/c" => ComparisonOp::LessThanEqual,
            ">/c" => ComparisonOp::GreaterThan,
            ">=/c" => ComparisonOp::GreaterThanEqual,
            "=/c" => ComparisonOp::Equal,
            "!=/c" => ComparisonOp::NotEqual,
            _ => return Err(Box::new(Error::new_spanned(
                format!("Unknown comparison operator: {operator_str}"),
                self.current_token().span,
            ))),
        };
        
        let value = Box::new(self.parse_expression()?);
        
        Ok(ContractExpr::Comparison {
            operator,
            value,
            location: self.current_token().span,
        })
    }

    /// Parses a listof contract: (listof contract)
    fn parse_listof_contract(&mut self) -> Result<ContractExpr> {
        let element_contract = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::ListOf {
            element_contract,
            location: self.current_token().span,
        })
    }

    /// Parses a vectorof contract: (vectorof contract)
    fn parse_vectorof_contract(&mut self) -> Result<ContractExpr> {
        let element_contract = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::VectorOf {
            element_contract,
            location: self.current_token().span,
        })
    }

    /// Parses a hash contract: (hash/c key-contract value-contract)
    fn parse_hash_contract(&mut self) -> Result<ContractExpr> {
        let key_contract = Box::new(self.parse_contract_expression()?);
        let value_contract = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::Hash {
            key_contract,
            value_contract,
            location: self.current_token().span,
        })
    }

    /// Parses a tuple contract: (tuple/c contract ...)
    fn parse_tuple_contract(&mut self) -> Result<ContractExpr> {
        let mut element_contracts = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            element_contracts.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::Tuple {
            element_contracts,
            location: self.current_token().span,
        })
    }

    /// Parses a list contract: (list/c contract ...)
    fn parse_list_contract(&mut self) -> Result<ContractExpr> {
        let mut element_contracts = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            element_contracts.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::List {
            element_contracts,
            location: self.current_token().span,
        })
    }

    /// Parses a vector contract: (vector/c contract ...)
    fn parse_vector_contract(&mut self) -> Result<ContractExpr> {
        let mut element_contracts = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            element_contracts.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::Vector {
            element_contracts,
            location: self.current_token().span,
        })
    }

    /// Parses a flat contract: (flat/c expr)
    fn parse_flat_contract(&mut self) -> Result<ContractExpr> {
        let check_expr = Box::new(self.parse_expression()?);
        
        Ok(ContractExpr::Flat {
            check_expr,
            location: self.current_token().span,
        })
    }

    /// Parses a recursive contract: (rec/c name contract)
    fn parse_recursive_contract(&mut self) -> Result<ContractExpr> {
        let name = self.parse_identifier()?;
        let contract = Box::new(self.parse_contract_expression()?);
        
        Ok(ContractExpr::Recursive {
            name,
            contract,
            location: self.current_token().span,
        })
    }

    /// Parses a with-message contract: (with-message/c contract message)
    fn parse_with_message_contract(&mut self) -> Result<ContractExpr> {
        let contract = Box::new(self.parse_contract_expression()?);
        let message = self.parse_string_literal()?;
        
        Ok(ContractExpr::WithMessage {
            contract,
            message,
            location: self.current_token().span,
        })
    }

    /// Parses a parametric contract: (name arg ...)
    fn parse_parametric_contract(&mut self, name: &str) -> Result<ContractExpr> {
        let mut parameters = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            parameters.push(self.parse_contract_expression()?);
        }
        
        Ok(ContractExpr::Parametric {
            name: name.to_string(),
            parameters,
            location: self.current_token().span,
        })
    }

    // ============= UTILITY PARSING METHODS =============

    /// Parses an identifier.
    fn parse_identifier(&mut self) -> Result<String> {
        let token = self.expect(TokenKind::Identifier, "Expected identifier")?;
        Ok(token.lexeme().to_string().clone())
    }

    /// Parses a string literal.
    fn parse_string_literal(&mut self) -> Result<String> {
        let token = self.expect(TokenKind::String, "Expected string literal")?;
        Ok(token.lexeme().to_string().clone())
    }

    /// Parses formals (parameter list).
    fn parse_formals(&mut self) -> Result<Formals> {
        // Simplified formals parsing - just collect identifiers
        let mut params = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.at_end() {
            if self.check(&TokenKind::Identifier) {
                params.push(self.advance().lexeme().to_string().clone())
            } else {
                break;
            }
        }
        
        Ok(Formals::Fixed(params))
    }

    /// Parses an expression (simplified for this contract parser).
    fn parse_expression(&mut self) -> Result<Spanned<Expr>> {
        let start_span = self.current_token().span;
        
        if self.check(&TokenKind::Identifier) {
            let name = self.advance().lexeme().to_string().clone();
            Ok(Spanned::new(Expr::Identifier(name), start_span))
        } else if self.check(&TokenKind::RealNumber) || 
                  self.check(&TokenKind::IntegerNumber) ||
                  self.check(&TokenKind::RationalNumber) ||
                  self.check(&TokenKind::ComplexNumber) {
            let token = self.advance();
            if let Ok(num) = token.lexeme().to_string().parse::<f64>() {
                Ok(Spanned::new(Expr::Literal(Literal::Number(num)), token.span))
            } else {
                Err(Box::new(Error::new_spanned("Invalid number literal".to_string(), token.span)))
            }
        } else if self.check(&TokenKind::String) {
            let token = self.advance();
            Ok(Spanned::new(
                Expr::Literal(Literal::String(Box::new(token.lexeme().to_string()))),
                token.span,
            ))
        } else if self.check(&TokenKind::Boolean) {
            let token = self.advance();
            let value = token.lexeme() == "true";
            Ok(Spanned::new(Expr::Literal(Literal::Boolean(value)), token.span))
        } else {
            Err(Box::new(Error::new_spanned(
                "Expected expression".to_string(),
                start_span,
            )))
        }
    }
}

/// Extended Expr enum to include DefineContract.
/// This would be added to the main ast::Expr enum.
impl Expr {
    /// DefineContract expression: (define/contract (name formals) contract body ...)
    pub fn define_contract(
        name: String,
        formals: Option<Formals>,
        contract: Spanned<ContractExpr>,
        return_type: Option<Spanned<ContractExpr>>,
        body: Vec<Spanned<Expr>>,
    ) -> Self {
        // For now, we'll represent this as a regular Define with metadata
        // In a real implementation, this would be its own variant
        let metadata = HashMap::new(); // TODO: Store contract information
        
        let value = if let Some(formals) = formals {
            // Function definition
            Box::new(Spanned::new(
                Expr::Lambda {
                    formals,
                    return_type: None, // TODO: Convert ContractExpr to TypeExpr
                    metadata,
                    body,
                },
                contract.span,
            ))
        } else {
            // Variable definition - use first body expression or identifier
            if let Some(first_body) = body.into_iter().next() {
                Box::new(first_body)
            } else {
                Box::new(Spanned::new(
                    Expr::Identifier(name.clone()),
                    contract.span,
                ))
            }
        };
        
        Expr::Define {
            name,
            value,
            return_type: None, // TODO: Convert ContractExpr to TypeExpr
            metadata: HashMap::new(),
        }
    }
}

// Note: Additional TokenKind variants like Star and Question would be needed
// in the main lexer::TokenKind enum for full contract syntax support.
// For now, we use string matching on Identifier tokens.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, TokenKind};
    use crate::diagnostics::Span;

    fn create_test_tokens(input: &str) -> Vec<Token> {
        // Simplified tokenization for tests
        let mut tokens = Vec::new();
        let mut pos = 0;
        
        for word in input.split_whitespace() {
            let span = Span::new(pos, pos + word.len());
            let kind = match word {
                "(" => TokenKind::LeftParen,
                ")" => TokenKind::RightParen,
                "[" => TokenKind::LeftBracket,
                "]" => TokenKind::RightBracket,
                ":" => TokenKind::Colon,
                "*" => TokenKind::Identifier,
                "?" => TokenKind::Identifier,
                "true" => TokenKind::Boolean,
                "false" => TokenKind::Boolean,
                s if s.starts_with('"') => TokenKind::String,
                s if s.parse::<f64>().is_ok() => TokenKind::RealNumber,
                _ => TokenKind::Identifier,
            };
            
            tokens.push(Token {
                kind,
                lexeme: word.to_string(),
                span,
            });
            pos += word.len() + 1;
        }
        
        tokens
    }

    #[test]
    fn test_predicate_parsing() {
        let tokens = create_test_tokens("number?");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::Predicate { name, .. } => {
                assert_eq!(name, "number?");
            }
            _ => panic!("Expected predicate contract"),
        }
    }

    #[test]
    fn test_function_contract_parsing() {
        let tokens = create_test_tokens("( -> number? string? boolean? )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::Function { domain, codomain, .. } => {
                assert_eq!(domain.len(), 2);
                assert!(matches!(codomain.inner, ContractExpr::Predicate { .. }))
            }
            _ => panic!("Expected function contract"),
        }
    }

    #[test]
    fn test_and_contract_parsing() {
        let tokens = create_test_tokens("( and/c number? positive? )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::And { contracts, .. } => {
                assert_eq!(contracts.len(), 2);
            }
            _ => panic!("Expected and contract"),
        }
    }

    #[test]
    fn test_listof_contract_parsing() {
        let tokens = create_test_tokens("( listof number? )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::ListOf { element_contract, .. } => {
                match element_contract.inner {
                    ContractExpr::Predicate { name, .. } => {
                        assert_eq!(name, "number?");
                    }
                    _ => panic!("Expected predicate in listof"),
                }
            }
            _ => panic!("Expected listof contract"),
        }
    }

    #[test]
    fn test_comparison_contract_parsing() {
        let tokens = create_test_tokens("( </c 10 )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOp::LessThan);
            }
            _ => panic!("Expected comparison contract"),
        }
    }

    #[test]
    fn test_nested_contract_parsing() {
        let tokens = create_test_tokens("( and/c number? ( or/c positive? zero? ) )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression().unwrap();
        match result.inner {
            ContractExpr::And { contracts, .. } => {
                assert_eq!(contracts.len(), 2);
                match &contracts[1].inner {
                    ContractExpr::Or { contracts: or_contracts, .. } => {
                        assert_eq!(or_contracts.len(), 2);
                    }
                    _ => panic!("Expected or contract in second position"),
                }
            }
            _ => panic!("Expected and contract"),
        }
    }

    #[test]
    fn test_error_handling() {
        let tokens = create_test_tokens("( invalid-combinator )");
        let mut parser = ContractParser::new(tokens);
        
        let result = parser.parse_contract_expression();
        // Should parse as parametric contract or return error
        assert!(result.is_ok() || parser.has_errors())
    }
}