//! Parser for converting tokens into an Abstract Syntax Tree

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use crate::lexer::Token;
pub use crate::parser::loop_detection::LoopDetectionConfig;
use crate::parser::loop_detection::{
    check_for_infinite_loops, check_for_infinite_loops_with_config,
};

/// Parser for Scheme tokens
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    /// Configuration for infinite loop detection
    loop_detection_config: LoopDetectionConfig,
}

impl Parser {
    /// Create a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            loop_detection_config: LoopDetectionConfig::default(),
        }
    }

    /// Create a new parser with custom loop detection configuration
    pub fn with_loop_detection_config(tokens: Vec<Token>, config: LoopDetectionConfig) -> Self {
        Parser {
            tokens,
            position: 0,
            loop_detection_config: config,
        }
    }

    /// Get the current token without consuming it
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Consume and return the current token
    fn consume_token(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Parse all tokens into a list of expressions
    pub fn parse_all(&mut self) -> Result<Vec<Expr>> {
        let mut expressions = Vec::new();

        while self.position < self.tokens.len() {
            expressions.push(self.parse_expression()?);
        }

        // Check for infinite loops after parsing all expressions
        if self.loop_detection_config.enable_cycle_detection {
            check_for_infinite_loops(&expressions)?;
        }

        Ok(expressions)
    }

    /// Parse a single expression
    pub fn parse_expression(&mut self) -> Result<Expr> {
        match self.current_token() {
            Some(Token::LeftParen) => self.parse_list(),
            Some(Token::VectorStart) => self.parse_vector(),
            Some(Token::Quote) => self.parse_quote(),
            Some(Token::Quasiquote) => self.parse_quasiquote(),
            Some(Token::Unquote) => self.parse_unquote(),
            Some(Token::UnquoteSplicing) => self.parse_unquote_splicing(),
            Some(token) => self.parse_atom(token.clone()),
            None => Err(LambdustError::parse_error(
                "Unexpected end of input".to_string(),
            )),
        }
    }

    /// Parse a list expression
    fn parse_list(&mut self) -> Result<Expr> {
        self.consume_token(); // consume '('
        let mut elements = Vec::new();
        let mut has_dot = false;
        let mut tail = None;

        while let Some(token) = self.current_token() {
            match token {
                Token::RightParen => {
                    self.consume_token(); // consume ')'
                    if has_dot {
                        if let Some(tail_expr) = tail {
                            return Ok(Expr::DottedList(elements, Box::new(tail_expr)));
                        }
                        return Err(LambdustError::parse_error(
                            "Missing tail after dot".to_string(),
                        ));
                    }
                    return Ok(Expr::List(elements));
                }
                Token::Dot => {
                    if has_dot {
                        return Err(LambdustError::parse_error(
                            "Multiple dots in list".to_string(),
                        ));
                    }
                    if elements.is_empty() {
                        return Err(LambdustError::parse_error(
                            "Dot at beginning of list".to_string(),
                        ));
                    }
                    has_dot = true;
                    self.consume_token(); // consume '.'
                    tail = Some(self.parse_expression()?);
                }
                _ => {
                    if has_dot && tail.is_some() {
                        return Err(LambdustError::parse_error(
                            "Multiple expressions after dot".to_string(),
                        ));
                    }
                    elements.push(self.parse_expression()?);
                }
            }
        }

        Err(LambdustError::parse_error("Unterminated list".to_string()))
    }

    /// Parse a quoted expression
    fn parse_quote(&mut self) -> Result<Expr> {
        self.consume_token(); // consume '
        let expr = self.parse_expression()?;
        Ok(Expr::Quote(Box::new(expr)))
    }

    /// Parse a quasiquoted expression
    fn parse_quasiquote(&mut self) -> Result<Expr> {
        self.consume_token(); // consume `
        let expr = self.parse_expression()?;
        Ok(Expr::Quasiquote(Box::new(expr)))
    }

    /// Parse an unquoted expression
    fn parse_unquote(&mut self) -> Result<Expr> {
        self.consume_token(); // consume ,
        let expr = self.parse_expression()?;
        Ok(Expr::Unquote(Box::new(expr)))
    }

    /// Parse an unquote-splicing expression
    fn parse_unquote_splicing(&mut self) -> Result<Expr> {
        self.consume_token(); // consume ,@
        let expr = self.parse_expression()?;
        Ok(Expr::UnquoteSplicing(Box::new(expr)))
    }

    /// Parse a vector expression
    fn parse_vector(&mut self) -> Result<Expr> {
        self.consume_token(); // consume #(
        let mut elements = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::RightParen => {
                    self.consume_token(); // consume )
                    return Ok(Expr::Vector(elements));
                }
                _ => {
                    elements.push(self.parse_expression()?);
                }
            }
        }

        Err(LambdustError::parse_error(
            "Expected closing parenthesis for vector".to_string(),
        ))
    }

    /// Parse an atomic expression (literal or symbol)
    fn parse_atom(&mut self, token: Token) -> Result<Expr> {
        self.consume_token(); // consume the token

        match token {
            Token::Boolean(b) => Ok(Expr::Literal(Literal::Boolean(b))),
            Token::Number(n) => Ok(Expr::Literal(Literal::Number(n))),
            Token::String(s) => Ok(Expr::Literal(Literal::String(s))),
            Token::Character(c) => Ok(Expr::Literal(Literal::Character(c))),
            Token::Symbol(s) => Ok(Expr::Variable(s)),
            Token::RightParen => Err(LambdustError::parse_error(
                "Unexpected right parenthesis".to_string(),
            )),
            Token::Dot => Err(LambdustError::parse_error(
                "Unexpected dot outside of list".to_string(),
            )),
            _ => Err(LambdustError::parse_error(format!(
                "Unexpected token: {token}"
            ))),
        }
    }
}

/// Parse a vector of tokens into a single expression (for REPL use)
pub fn parse(tokens: Vec<Token>) -> Result<Expr> {
    if tokens.is_empty() {
        return Err(LambdustError::parse_error("Unexpected end of input"));
    }

    let mut parser = Parser::new(tokens);
    let expressions = parser.parse_all()?;

    if expressions.len() == 1 {
        Ok(expressions.into_iter().next().unwrap())
    } else if expressions.is_empty() {
        Ok(Expr::Literal(Literal::Nil))
    } else {
        // Multiple expressions - wrap in begin
        Ok(Expr::List({
            let mut list = vec![Expr::Variable("begin".to_string())];
            list.extend(expressions);
            list
        }))
    }
}

/// Parse a vector of tokens into multiple expressions
pub fn parse_multiple(tokens: Vec<Token>) -> Result<Vec<Expr>> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let mut parser = Parser::new(tokens);
    parser.parse_all()
}

/// Parse with custom loop detection configuration
pub fn parse_with_loop_detection(tokens: Vec<Token>, config: LoopDetectionConfig) -> Result<Expr> {
    if tokens.is_empty() {
        return Err(LambdustError::parse_error("Unexpected end of input"));
    }

    let mut parser = Parser::with_loop_detection_config(
        tokens,
        LoopDetectionConfig {
            enable_cycle_detection: false,
            ..Default::default()
        },
    );
    let expressions = parser.parse_all()?;

    // Check for infinite loops with the given configuration
    if config.enable_cycle_detection {
        if config.warn_only {
            // In warn-only mode, we just collect the results but don't error
            let _loops = check_for_infinite_loops_with_config(&expressions, config)?;
        } else {
            check_for_infinite_loops(&expressions)?;
        }
    }

    if expressions.len() == 1 {
        Ok(expressions.into_iter().next().unwrap())
    } else if expressions.is_empty() {
        Ok(Expr::Literal(Literal::Nil))
    } else {
        // Multiple expressions - wrap in begin
        Ok(Expr::List({
            let mut list = vec![Expr::Variable("begin".to_string())];
            list.extend(expressions);
            list
        }))
    }
}

// Include the infinite loop detection modules
pub mod cycle_detector;
pub mod dependency_analyzer;
pub mod loop_detection;
