use crate::ast::{Expr, Formals, KeywordParam, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use super::ParserConfig;
use std::collections::HashMap;

/// The main parser for Lambdust.
/// 
/// This parser implements recursive descent parsing with robust error recovery.
/// It supports the full R7RS grammar plus Lambdust extensions.
#[derive(Debug)]
pub struct Parser {
    /// Token stream being parsed
    pub(crate) tokens: Vec<Token>,
    /// Current position in the token stream
    pub(crate) position: usize,
    /// Collected errors for error recovery - allows continuing after parse errors
    pub(crate) errors: Vec<Error>,
    /// Current nesting depth for tracking context (parentheses, etc.)
    pub(crate) nesting_depth: usize,
    /// Whether we're in panic mode recovery
    pub(crate) panic_mode: bool,
    /// Maximum number of errors to collect before stopping
    pub(crate) max_errors: usize,
    /// Context stack for better error messages
    pub(crate) context_stack: Vec<String>,
    /// Whether to enable aggressive error recovery
    pub(crate) aggressive_recovery: bool,
    /// EOF token for when we're past the end of input
    eof_token: Token,
}

impl Parser {
    /// Creates a new parser with the given tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { 
            tokens, 
            position: 0,
            errors: Vec::new(),
            nesting_depth: 0,
            panic_mode: false,
            max_errors: 10,
            context_stack: Vec::new(),
            aggressive_recovery: true,
            eof_token: Token::eof(Span::new(0, 0)),
        }
    }
    
    /// Creates a new parser with custom error handling settings.
    pub fn with_settings(tokens: Vec<Token>, max_errors: usize, aggressive_recovery: bool) -> Self {
        Self {
            tokens,
            position: 0,
            errors: Vec::new(),
            nesting_depth: 0,
            panic_mode: false,
            max_errors,
            context_stack: Vec::new(),
            aggressive_recovery,
            eof_token: Token::eof(Span::new(0, 0)),
        }
    }
    
    /// Creates a new parser from configuration.
    pub fn from_config(tokens: Vec<Token>, config: &ParserConfig) -> Self {
        Self::with_settings(tokens, config.max_errors, config.aggressive_recovery)
    }
    
    /// Returns all collected errors.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
    
    /// Returns whether the parser has encountered errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Gets the current position in the token stream.
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Gets the current nesting depth.
    pub fn nesting_depth(&self) -> usize {
        self.nesting_depth
    }
    
    /// Returns whether the parser is in panic mode.
    pub fn is_panic_mode(&self) -> bool {
        self.panic_mode
    }
    
    /// Adds an error to the error list.
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }
    
    /// Sets panic mode.
    pub fn set_panic_mode(&mut self, panic: bool) {
        self.panic_mode = panic;
    }
    
    /// Increments nesting depth.
    pub fn increment_nesting(&mut self) {
        self.nesting_depth += 1;
    }
    
    /// Decrements nesting depth.
    pub fn decrement_nesting(&mut self) {
        if self.nesting_depth > 0 {
            self.nesting_depth -= 1;
        }
    }
    
    /// Pushes a context onto the context stack.
    pub fn push_context(&mut self, context: String) {
        self.context_stack.push(context);
    }
    
    /// Pops a context from the context stack.
    pub fn pop_context(&mut self) -> Option<String> {
        self.context_stack.pop()
    }
    
    /// Gets the current context stack.
    pub fn context_stack(&self) -> &[String] {
        &self.context_stack
    }

    /// Main parsing method - parses tokens into a Program AST.
    pub fn parse(&mut self) -> Result<Program> {
        self.try_parse_program()
    }

    /// Attempts to parse a program from the token stream.
    pub fn try_parse_program(&mut self) -> Result<Program> {
        let mut expressions = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            if !self.is_at_end() {
                match self.parse_single_expression() {
                    Ok(expr) => expressions.push(expr),
                    Err(err) => {
                        let cloned_err = err.as_ref().clone();
                        self.add_error(cloned_err);
                        if !self.aggressive_recovery || self.errors.len() >= self.max_errors {
                            return Err(err);
                        }
                        // Skip the problematic token and continue
                        self.advance();
                    }
                }
            }
        }
        
        Ok(Program { expressions })
    }

    /// Parses a single expression from the token stream.
    pub fn parse_single_expression(&mut self) -> Result<Spanned<Expr>> {
        if self.is_at_end() {
            return Err(Error::unexpected_eof(self.current_span()).boxed())
        }

        let start_pos = self.position();
        let token = &self.tokens[start_pos];
        
        match &token.kind {
            TokenKind::LeftParen => self.parse_parenthesized_expression(),
            TokenKind::IntegerNumber | TokenKind::RealNumber | 
            TokenKind::RationalNumber | TokenKind::ComplexNumber => self.parse_number(),
            TokenKind::String => self.parse_string(),
            TokenKind::Character => self.parse_character(),
            TokenKind::Boolean => self.parse_boolean(),
            TokenKind::Identifier => {
                let name = self.current_token().text.clone();
                let span = self.current_span();
                self.advance();
                self.make_identifier(name, span)
            },
            TokenKind::Quote => {
                let start_span = self.current_span();
                self.advance(); // consume quote
                self.parse_quote_form(start_span)
            },
            TokenKind::Quasiquote => self.parse_quasiquote_expression(),
            TokenKind::Unquote => self.parse_unquote_expression(),
            TokenKind::UnquoteSplicing => self.parse_unquote_splicing_expression(),
            _ => Err(Box::new(Error::unexpected_token(token, "expression"))),
        }
    }

    /// Alias for parse_single_expression for backward compatibility.
    pub fn parse_expression(&mut self) -> Result<Spanned<Expr>> {
        self.parse_single_expression()
    }

    /// Checks if current token matches the given kind.
    pub fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.current_token().kind == kind
        }
    }

    /// Checks if we're at the end of the token stream.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || 
        matches!(self.current_token().kind, TokenKind::Eof)
    }

    /// Advances to the next token and returns the consumed token.
    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous_token()
    }

    /// Consumes a token of the expected kind or returns an error.
    pub fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(Box::new(Error::expected_token(self.current_token(), kind, message)))
        }
    }

    /// Gets the current token without consuming it.
    pub fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() {
            &self.eof_token
        } else {
            &self.tokens[self.position]
        }
    }

    /// Gets the previous token.
    pub fn previous_token(&self) -> &Token {
        if self.position == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.position - 1]
        }
    }

    /// Gets the current span position.
    pub fn current_span(&self) -> Span {
        if self.position >= self.tokens.len() {
            // Create an EOF span at the end
            Span::new(self.position, self.position)
        } else {
            self.current_token().span
        }
    }

    /// Skips whitespace and comment tokens.
    /// Note: Newlines are now handled as whitespace at lexer level (R7RS compliant).
    pub fn skip_whitespace(&mut self) {
        while !self.is_at_end() && matches!(self.current_token().kind, 
            TokenKind::LineComment | TokenKind::BlockComment) {
            self.advance();
        }
    }

    /// Executes a function with a context pushed onto the stack.
    pub fn with_context<T, F>(&mut self, context: &str, mut f: F) -> Result<T>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        self.push_context(context.to_string());
        let result = f(self);
        self.pop_context();
        result
    }

    /// Synchronizes parser state by finding the next closing parenthesis.
    pub fn synchronize_to_closing_paren(&mut self) {
        let mut paren_count = 1;
        
        while !self.is_at_end() && paren_count > 0 {
            match self.current_token().kind {
                TokenKind::LeftParen => paren_count += 1,
                TokenKind::RightParen => paren_count -= 1,
                _ => {}
            }
            self.advance();
        }
    }


    // Note: Literal parsing methods (parse_number, parse_string, parse_character, parse_boolean)
    // are implemented in literals.rs as extensions to the Parser impl.
    // Core parsing method implementations are provided above
    // with additional specialized methods in (literals.rs, expression.rs, special_forms.rs)

    /// Parses metadata as string key-value pairs (for simple cases).
    pub fn parse_metadata(&mut self) -> Result<HashMap<String, String>> {
        // For now, just return empty metadata
        Ok(HashMap::new())
    }

    /// Parses metadata with expression values (for complex cases).
    pub fn parse_metadata_exprs(&mut self) -> Result<HashMap<String, Spanned<Expr>>> {
        // For now, just return empty metadata
        Ok(HashMap::new())
    }

    /// Validates that an identifier name is valid.
    pub fn validate_identifier(name: &str, span: Span) -> Result<()> {
        if name.is_empty() {
            return Err(Box::new(Error::parse_error("Identifier cannot be empty", span)))
        }
        
        // Basic validation - could be expanded with more Scheme identifier rules
        if name.starts_with(|c: char| c.is_numeric()) {
            return Err(Box::new(Error::parse_error("Identifier cannot start with a number", span)))
        }
        
        Ok(())
    }

    /// Checks if the parser can recover at the current position.
    pub fn can_recover_at_current_position(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        // Can recover at common recovery points like parentheses, etc.
        matches!(
            self.current_token().kind,
            TokenKind::LeftParen | TokenKind::RightParen | TokenKind::Eof
        )
    }

    /// Gets the text of the current token.
    pub fn current_token_text(&self) -> String {
        self.current_token().text.clone()
    }

    /// Validates formals for correctness.
    pub fn validate_formals(formals: &Formals, span: Span) -> Result<()> {
        // Basic validation - check for duplicate parameter names
        use std::collections::HashSet;
        let mut seen_names = HashSet::new();
        
        match formals {
            Formals::Fixed(params) => {
                for param in params {
                    if !seen_names.insert(param) {
                        return Err(Box::new(Error::parse_error(
                            format!("Duplicate parameter name: {param}"),
                            span,
                        )))
                    }
                }
            }
            Formals::Variable(param) => {
                // Just one parameter, no duplicates possible
                if param.is_empty() {
                    return Err(Box::new(Error::parse_error(
                        "Parameter name cannot be empty", 
                        span
                    )))
                }
            }
            Formals::Mixed { fixed, rest } => {
                // Check fixed parameters
                for param in fixed {
                    if !seen_names.insert(param) {
                        return Err(Box::new(Error::parse_error(
                            format!("Duplicate parameter name: {param}"),
                            span,
                        )))
                    }
                }
                
                // Check rest parameter
                if !seen_names.insert(rest) {
                    return Err(Box::new(Error::parse_error(
                        format!("Duplicate parameter name: {rest}"),
                        span,
                    )))
                }
            }
            Formals::Keyword { fixed, rest, keywords } => {
                // Check fixed parameters
                for param in fixed {
                    if !seen_names.insert(param) {
                        return Err(Box::new(Error::parse_error(
                            format!("Duplicate parameter name: {param}"),
                            span,
                        )))
                    }
                }
                
                // Check rest parameter
                if let Some(rest) = rest {
                    if !seen_names.insert(rest) {
                        return Err(Box::new(Error::parse_error(
                            format!("Duplicate parameter name: {rest}"),
                            span,
                        )))
                    }
                }
                
                // Check keyword parameters
                for keyword in keywords {
                    if !seen_names.insert(&keyword.name) {
                        return Err(Box::new(Error::parse_error(
                            format!("Duplicate parameter name: {}", keyword.name),
                            span,
                        )))
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Parses a parenthesized expression (list, call, or special form).
    pub fn parse_parenthesized_expression(&mut self) -> Result<Spanned<Expr>> {
        let start_span = self.current_span();
        
        // Consume the opening parenthesis
        self.consume(&TokenKind::LeftParen, "Expected opening parenthesis")?;
        self.increment_nesting();
        self.skip_whitespace();
        
        // Handle empty list
        if self.check(&TokenKind::RightParen) {
            let end_span = self.current_span();
            self.advance(); // consume ')'
            self.decrement_nesting();
            let span = start_span.combine(end_span);
            return Ok(Spanned::new(Expr::List(Vec::new()), span))
        }
        
        // Parse the first element to determine what kind of expression this is
        let first_element = self.parse_expression()?;
        self.skip_whitespace();
        
        // Check if this is a special form by examining the first element
        if let Expr::Identifier(ref name) = first_element.inner {
            let result = match name.as_str() {
                // Core special forms
                "quote" => self.parse_quote_form(start_span),
                "lambda" => self.parse_lambda_form(start_span),
                "if" => self.parse_if_form(start_span),
                "define" => self.parse_define_form(start_span),
                "set!" => self.parse_set_form(start_span),
                "define-syntax" => self.parse_define_syntax_form(start_span),
                "syntax-rules" => self.parse_syntax_rules_form(start_span),
                "call-with-current-continuation" | "call/cc" => self.parse_call_cc_form(start_span),
                "primitive" => self.parse_primitive_form(start_span),
                "::" => self.parse_type_annotation_form(start_span),
                "parameterize" => self.parse_parameterize_form(start_span),
                "import" => self.parse_import_form(start_span),
                "define-library" => self.parse_define_library_form(start_span),
                
                // Derived forms
                "begin" => self.parse_begin_form(start_span),
                "let" => self.parse_let_form(start_span),
                "let*" => self.parse_let_star_form(start_span),
                "letrec" => self.parse_letrec_form(start_span),
                "cond" => self.parse_cond_form(start_span),
                "case" => self.parse_case_form(start_span),
                "and" => self.parse_and_form(start_span),
                "or" => self.parse_or_form(start_span),
                "when" => self.parse_when_form(start_span),
                "unless" => self.parse_unless_form(start_span),
                "guard" => self.parse_guard_form(start_span),
                "case-lambda" => self.parse_case_lambda_form(start_span),
                
                // Not a special form - parse as application
                _ => {
                    // Parse remaining operands
                    let mut operands = Vec::new();
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        operands.push(self.parse_expression()?);
                        self.skip_whitespace();
                    }
                    
                    let end_span = self.current_span();
                    self.consume(&TokenKind::RightParen, "Expected closing parenthesis")?;
                    self.decrement_nesting();
                    let span = start_span.combine(end_span);
                    
                    Ok(Spanned::new(Expr::Application {
                        operator: Box::new(first_element),
                        operands,
                    }, span))
                }
            };
            
            // Decrement nesting depth for special forms (they handle their own closing paren)
            match result {
                Ok(_) => {
                    self.decrement_nesting();
                    result
                }
                Err(e) => {
                    self.decrement_nesting();
                    // Try to synchronize to closing paren on error
                    if self.aggressive_recovery {
                        self.synchronize_to_closing_paren();
                    }
                    Err(e)
                }
            }
        } else {
            // Not an identifier - treat as regular application or list
            let mut elements = vec![first_element];
            
            // Parse remaining elements
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                elements.push(self.parse_expression()?);
                self.skip_whitespace();
            }
            
            let end_span = self.current_span();
            self.consume(&TokenKind::RightParen, "Expected closing parenthesis")?;
            self.decrement_nesting();
            let span = start_span.combine(end_span);
            
            // If first element is not a procedure, treat as a list
            // Otherwise, treat as application
            if elements.len() == 1 {
                Ok(Spanned::new(Expr::List(elements), span))
            } else {
                let operator = Box::new(elements.remove(0));
                Ok(Spanned::new(Expr::Application {
                    operator,
                    operands: elements,
                }, span))
            }
        }
    }

    /// Parses a quasiquote expression.
    pub fn parse_quasiquote_expression(&mut self) -> Result<Spanned<Expr>> {
        let start_span = self.current_span();
        self.advance(); // consume '`'
        
        let expr = self.parse_expression()?;
        let span = start_span.combine(expr.span);
        
        // For now, treat quasiquote similar to quote
        // TODO: Implement proper quasiquote semantics with unquote handling
        Ok(Spanned::new(Expr::Quote(Box::new(expr)), span))
    }

    /// Parses an unquote expression.
    pub fn parse_unquote_expression(&mut self) -> Result<Spanned<Expr>> {
        let start_span = self.current_span();
        self.advance(); // consume ','
        
        let expr = self.parse_expression()?;
        let span = start_span.combine(expr.span);
        
        // For now, create an application of 'unquote'
        // TODO: Implement proper quasiquote/unquote semantics
        let unquote_id = Spanned::new(Expr::Identifier("unquote".to_string()), start_span);
        Ok(Spanned::new(Expr::Application {
            operator: Box::new(unquote_id),
            operands: vec![expr],
        }, span))
    }

    /// Parses an unquote-splicing expression.
    pub fn parse_unquote_splicing_expression(&mut self) -> Result<Spanned<Expr>> {
        let start_span = self.current_span();
        self.advance(); // consume ',@'
        
        let expr = self.parse_expression()?;
        let span = start_span.combine(expr.span);
        
        // For now, create an application of 'unquote-splicing'
        // TODO: Implement proper quasiquote/unquote-splicing semantics
        let unquote_splicing_id = Spanned::new(Expr::Identifier("unquote-splicing".to_string()), start_span);
        Ok(Spanned::new(Expr::Application {
            operator: Box::new(unquote_splicing_id),
            operands: vec![expr],
        }, span))
    }
}