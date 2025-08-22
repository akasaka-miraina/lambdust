//! Partial parsing for incomplete code and real-time IDE support
//!
//! This module handles parsing of incomplete or syntactically invalid code,
//! providing best-effort AST construction for IDE features like syntax highlighting,
//! autocompletion, and error reporting during code editing.

use crate::ast::{Expr, Formals, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use crate::parser::{
    Parser,
    error_recovery::{ErrorRecoveryEngine, RecoveryStrategy},
};
use std::collections::{HashMap, VecDeque};

/// Parser mode for different parsing scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingMode {
    /// Complete parsing - requires syntactically correct input
    Complete,
    /// Partial parsing - handles incomplete code
    Partial,
    /// Interactive parsing - optimized for real-time feedback
    Interactive,
    /// Error-tolerant parsing - maximum recovery effort
    ErrorTolerant,
}

/// Configuration for partial parsing behavior
#[derive(Debug, Clone)]
pub struct PartialParsingConfig {
    /// Parsing mode
    pub mode: ParsingMode,
    /// Maximum number of errors to tolerate
    pub max_errors: usize,
    /// Whether to insert missing tokens automatically
    pub auto_insert_tokens: bool,
    /// Whether to repair unbalanced delimiters
    pub repair_delimiters: bool,
    /// Whether to provide placeholder expressions
    pub use_placeholders: bool,
    /// Timeout for parsing (in milliseconds)
    pub timeout_ms: u64,
    /// Whether to preserve comments in partial AST
    pub preserve_comments: bool,
    /// Whether to track incomplete expressions
    pub track_incomplete: bool,
}

impl Default for PartialParsingConfig {
    fn default() -> Self {
        Self {
            mode: ParsingMode::Partial,
            max_errors: 50,
            auto_insert_tokens: true,
            repair_delimiters: true,
            use_placeholders: true,
            timeout_ms: 1000,
            preserve_comments: true,
            track_incomplete: true,
        }
    }
}

/// Result of partial parsing with additional metadata
#[derive(Debug, Clone)]
pub struct PartialParsingResult {
    /// Parsed program (may be incomplete)
    pub program: Program,
    /// Parse errors encountered
    pub errors: Vec<Error>,
    /// Incomplete expressions with context
    pub incomplete_expressions: Vec<IncompleteExpression>,
    /// Inserted tokens for repair
    pub inserted_tokens: Vec<InsertedToken>,
    /// Comments preserved during parsing
    pub comments: Vec<CommentInfo>,
    /// Parsing statistics
    pub stats: ParsingStatistics,
}

/// Information about incomplete expressions
#[derive(Debug, Clone)]
pub struct IncompleteExpression {
    /// Type of incompleteness
    pub kind: IncompleteKind,
    /// Span where incompleteness was detected
    pub span: Span,
    /// Expected tokens to complete
    pub expected: Vec<TokenKind>,
    /// Context information
    pub context: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Suggested completions
    pub suggestions: Vec<String>,
}

/// Types of incomplete expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompleteKind {
    /// Missing closing delimiter
    MissingClosing,
    /// Missing opening delimiter
    MissingOpening,
    /// Incomplete function call
    IncompleteCall,
    /// Incomplete definition
    IncompleteDefinition,
    /// Incomplete lambda expression
    IncompleteLambda,
    /// Incomplete conditional
    IncompleteConditional,
    /// Incomplete binding form
    IncompleteBinding,
    /// Unknown/malformed expression
    Malformed,
}

/// Information about tokens inserted during repair
#[derive(Debug, Clone)]
pub struct InsertedToken {
    /// The inserted token
    pub token: Token,
    /// Position where inserted
    pub position: usize,
    /// Reason for insertion
    pub reason: String,
    /// Confidence in the repair (0.0 - 1.0)
    pub confidence: f32,
}

/// Comment information preserved during parsing
#[derive(Debug, Clone)]
pub struct CommentInfo {
    /// Comment content
    pub content: String,
    /// Span of the comment
    pub span: Span,
    /// Type of comment
    pub comment_type: CommentType,
    /// Associated expression (if any)
    pub associated_expr: Option<Span>,
}

/// Types of comments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentType {
    /// Line comment (;; or ;)
    Line,
    /// Block comment (#| ... |#)
    Block,
    /// Documentation comment (;;; or ;!; etc.)
    Documentation,
}

/// Statistics about the parsing process
#[derive(Debug, Clone, Default)]
pub struct ParsingStatistics {
    /// Total tokens processed
    pub tokens_processed: usize,
    /// Number of expressions parsed
    pub expressions_parsed: usize,
    /// Number of errors encountered
    pub errors_encountered: usize,
    /// Number of repairs attempted
    pub repairs_attempted: usize,
    /// Number of successful repairs
    pub repairs_successful: usize,
    /// Time taken for parsing (in microseconds)
    pub parse_time_us: u64,
    /// Memory usage (approximate bytes)
    pub memory_usage: usize,
}

/// Partial parser that can handle incomplete code
pub struct PartialParser {
    /// Base parser
    base_parser: Parser,
    /// Configuration
    config: PartialParsingConfig,
    /// Error recovery engine
    recovery_engine: ErrorRecoveryEngine,
    /// Placeholder counter for unique names
    placeholder_counter: usize,
    /// Current parsing context
    context_stack: Vec<ParsingContext>,
    /// Token stream buffer
    token_buffer: VecDeque<Token>,
    /// Parsing statistics
    stats: ParsingStatistics,
}

/// Parsing context for nested structures
#[derive(Debug, Clone)]
pub struct ParsingContext {
    /// Type of context
    pub context_type: ContextType,
    /// Starting position
    pub start_position: usize,
    /// Expected closing tokens
    pub expected_closing: Vec<TokenKind>,
    /// Nesting depth at context start
    pub nesting_depth: usize,
}

/// Types of parsing contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// Top-level expression context
    TopLevel,
    /// Function application context
    Application,
    /// Special form context
    SpecialForm(SpecialFormType),
    /// List literal context
    List,
    /// Vector literal context
    Vector,
    /// Quote context
    Quote,
    /// Quasiquote context
    Quasiquote,
}

/// Types of special forms for context tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialFormType {
    /// Variable or function definition form (`define`)
    Define,
    /// Lambda expression form (`lambda`)
    Lambda,
    /// Conditional form (`if`)
    If,
    /// Local binding form (`let`, `let*`, `letrec`)
    Let,
    /// Multi-branch conditional form (`cond`)
    Cond,
    /// Pattern matching form (`case`)
    Case,
    /// Sequential execution form (`begin`)
    Begin,
    /// Logical AND form with short-circuiting (`and`)
    And,
    /// Logical OR form with short-circuiting (`or`)
    Or,
    /// Conditional execution form (`when`)
    When,
    /// Conditional execution form (`unless`)
    Unless,
    /// Guard expression in pattern matching
    Guard,
}

impl PartialParser {
    /// Creates a new partial parser
    pub fn new(tokens: Vec<Token>, config: PartialParsingConfig) -> Self {
        let base_parser = Parser::with_settings(
            tokens.clone(),
            config.max_errors,
            true, // aggressive recovery
        );

        let recovery_strategy = match config.mode {
            ParsingMode::Complete => RecoveryStrategy::PanicMode,
            ParsingMode::Partial => RecoveryStrategy::LocalRepair,
            ParsingMode::Interactive => RecoveryStrategy::Contextual,
            ParsingMode::ErrorTolerant => RecoveryStrategy::ErrorProductions,
        };

        Self {
            base_parser,
            config,
            recovery_engine: ErrorRecoveryEngine::new(recovery_strategy),
            placeholder_counter: 0,
            context_stack: Vec::new(),
            token_buffer: tokens.into(),
            stats: ParsingStatistics::default(),
        }
    }

    /// Creates a partial parser with default configuration
    pub fn with_default_config(tokens: Vec<Token>) -> Self {
        Self::new(tokens, PartialParsingConfig::default())
    }

    /// Parses tokens with partial parsing support
    pub fn parse_partial(&mut self) -> PartialParsingResult {
        let start_time = std::time::Instant::now();
        let mut expressions = Vec::new();
        let mut errors = Vec::new();
        let mut incomplete_expressions = Vec::new();
        let mut inserted_tokens = Vec::new();
        let mut comments = Vec::new();

        self.stats = ParsingStatistics::default();
        self.enter_context(ContextType::TopLevel, 0);

        // Main parsing loop with error tolerance
        while !self.is_at_end() {
            self.skip_whitespace_and_comments(&mut comments);

            if self.is_at_end() {
                break;
            }

            match self.parse_expression_with_recovery() {
                Ok(expr) => {
                    expressions.push(expr);
                    self.stats.expressions_parsed += 1;
                }
                Err(error) => {
                    errors.push(error.as_ref().clone());
                    self.stats.errors_encountered += 1;

                    // Attempt recovery
                    if let Ok(recovery_info) = self.attempt_recovery() {
                        if let Some(incomplete) = recovery_info.incomplete {
                            incomplete_expressions.push(incomplete);
                        }
                        if let Some(inserted) = recovery_info.inserted_token {
                            inserted_tokens.push(inserted);
                        }
                    }

                    // Check error limit
                    if errors.len() >= self.config.max_errors {
                        break;
                    }
                }
            }
        }

        // Handle remaining incomplete contexts
        self.finalize_incomplete_contexts(&mut incomplete_expressions);

        self.stats.parse_time_us = start_time.elapsed().as_micros() as u64;
        self.stats.tokens_processed = self.base_parser.position();

        PartialParsingResult {
            program: Program { expressions },
            errors,
            incomplete_expressions,
            inserted_tokens,
            comments,
            stats: self.stats.clone(),
        }
    }

    /// Parses a single expression with recovery
    fn parse_expression_with_recovery(&mut self) -> Result<Spanned<Expr>> {
        let start_pos = self.base_parser.position();

        match self.base_parser.parse_single_expression() {
            Ok(expr) => Ok(expr),
            Err(error) => {
                // Try recovery strategies
                if self.config.auto_insert_tokens {
                    if let Some(repaired) = self.try_token_insertion(start_pos)? {
                        return Ok(repaired);
                    }
                }

                if self.config.use_placeholders {
                    if let Some(placeholder) = self.create_placeholder_expression(start_pos) {
                        return Ok(placeholder);
                    }
                }

                Err(error)
            }
        }
    }

    /// Attempts to recover from parse errors
    fn attempt_recovery(&mut self) -> Result<RecoveryInfo> {
        let current_pos = self.base_parser.position();
        let tokens = &self.base_parser.tokens;

        // Create a dummy error for recovery context
        let error = Error::parse_error("Recovery attempt", Span::new(current_pos, 1));

        let recovery_result = self
            .recovery_engine
            .recover_from_error(tokens, current_pos, &error);

        match recovery_result {
            crate::parser::error_recovery::RecoveryResult::Synchronized {
                new_position, ..
            } => {
                // Update parser position
                self.base_parser.position = new_position;
                self.stats.repairs_successful += 1;

                Ok(RecoveryInfo {
                    success: true,
                    new_position: Some(new_position),
                    incomplete: None,
                    inserted_token: None,
                })
            }
            crate::parser::error_recovery::RecoveryResult::Repaired {
                repair,
                new_position,
                ..
            } => {
                self.base_parser.position = new_position;
                self.stats.repairs_successful += 1;

                let inserted_token = repair.map(|r| match r {
                    crate::parser::error_recovery::TokenRepair::Insert { position, token } => {
                        InsertedToken {
                            token,
                            position,
                            reason: "Auto-repair".to_string(),
                            confidence: 0.7,
                        }
                    }
                    _ => InsertedToken {
                        token: Token::eof(Span::new(0, 0)),
                        position: current_pos,
                        reason: "Complex repair".to_string(),
                        confidence: 0.5,
                    },
                });

                Ok(RecoveryInfo {
                    success: true,
                    new_position: Some(new_position),
                    incomplete: None,
                    inserted_token,
                })
            }
            crate::parser::error_recovery::RecoveryResult::Failed { reason } => {
                // Create incomplete expression info
                let incomplete = IncompleteExpression {
                    kind: self.determine_incomplete_kind(current_pos),
                    span: Span::new(current_pos, 1),
                    expected: self.determine_expected_tokens(),
                    context: self.get_current_context_name(),
                    confidence: 0.3,
                    suggestions: self.generate_suggestions(),
                };

                Ok(RecoveryInfo {
                    success: false,
                    new_position: None,
                    incomplete: Some(incomplete),
                    inserted_token: None,
                })
            }
        }
    }

    /// Tries to insert missing tokens
    fn try_token_insertion(&mut self, position: usize) -> Result<Option<Spanned<Expr>>> {
        let current_context = self.current_context();

        if let Some(context) = current_context {
            if context.context_type == ContextType::Application {
                // Try to insert closing parenthesis
                if self.config.repair_delimiters {
                    let closing_token = Token::new(
                        TokenKind::RightParen,
                        Span::new(position, 1),
                        ")".to_string(),
                    );

                    // Insert token and retry parsing
                    self.insert_token_at_position(closing_token.clone(), position);

                    // Try parsing again
                    if let Ok(expr) = self.base_parser.parse_single_expression() {
                        return Ok(Some(expr));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Creates a placeholder expression for incomplete code
    fn create_placeholder_expression(&mut self, position: usize) -> Option<Spanned<Expr>> {
        if !self.config.use_placeholders {
            return None;
        }

        self.placeholder_counter += 1;
        let placeholder_name = format!("__placeholder_{}", self.placeholder_counter);
        let span = Span::new(position, 1);

        Some(Spanned::new(Expr::Identifier(placeholder_name), span))
    }

    /// Determines the type of incompleteness
    fn determine_incomplete_kind(&self, position: usize) -> IncompleteKind {
        if let Some(context) = self.current_context() {
            match context.context_type {
                ContextType::Application => IncompleteKind::IncompleteCall,
                ContextType::SpecialForm(SpecialFormType::Define) => {
                    IncompleteKind::IncompleteDefinition
                }
                ContextType::SpecialForm(SpecialFormType::Lambda) => {
                    IncompleteKind::IncompleteLambda
                }
                ContextType::SpecialForm(SpecialFormType::If) => {
                    IncompleteKind::IncompleteConditional
                }
                ContextType::SpecialForm(SpecialFormType::Let) => IncompleteKind::IncompleteBinding,
                _ => IncompleteKind::Malformed,
            }
        } else {
            // Check for unbalanced delimiters
            if self.base_parser.nesting_depth() > 0 {
                IncompleteKind::MissingClosing
            } else {
                IncompleteKind::Malformed
            }
        }
    }

    /// Determines expected tokens for completion
    fn determine_expected_tokens(&self) -> Vec<TokenKind> {
        let mut expected = Vec::new();

        if let Some(context) = self.current_context() {
            expected.extend(context.expected_closing.iter().cloned());

            match context.context_type {
                ContextType::Application => {
                    expected.extend(vec![
                        TokenKind::Identifier,
                        TokenKind::LeftParen,
                        TokenKind::IntegerNumber,
                        TokenKind::String,
                    ]);
                }
                ContextType::SpecialForm(SpecialFormType::Define) => {
                    expected.extend(vec![TokenKind::Identifier, TokenKind::LeftParen]);
                }
                _ => {
                    expected.push(TokenKind::Identifier);
                }
            }
        }

        expected
    }

    /// Gets the current context name as a string
    fn get_current_context_name(&self) -> String {
        if let Some(context) = self.current_context() {
            match context.context_type {
                ContextType::TopLevel => "top-level".to_string(),
                ContextType::Application => "function application".to_string(),
                ContextType::SpecialForm(form_type) => format!("{form_type:?} form"),
                ContextType::List => "list".to_string(),
                ContextType::Vector => "vector".to_string(),
                ContextType::Quote => "quoted expression".to_string(),
                ContextType::Quasiquote => "quasiquoted expression".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }

    /// Generates completion suggestions
    fn generate_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if let Some(context) = self.current_context() {
            match context.context_type {
                ContextType::TopLevel => {
                    suggestions.extend(vec![
                        "(define name value)".to_string(),
                        "(lambda (args) body)".to_string(),
                        "(if condition then else)".to_string(),
                    ]);
                }
                ContextType::Application => {
                    suggestions.extend(vec!["argument".to_string(), "expression".to_string()]);
                }
                ContextType::SpecialForm(SpecialFormType::Define) => {
                    suggestions.extend(vec![
                        "variable-name".to_string(),
                        "(function-name args)".to_string(),
                    ]);
                }
                _ => {}
            }
        }

        suggestions
    }

    /// Finalizes incomplete contexts at end of parsing
    fn finalize_incomplete_contexts(
        &mut self,
        incomplete_expressions: &mut Vec<IncompleteExpression>,
    ) {
        for context in &self.context_stack {
            if context.context_type != ContextType::TopLevel {
                incomplete_expressions.push(IncompleteExpression {
                    kind: IncompleteKind::MissingClosing,
                    span: Span::new(context.start_position, 1),
                    expected: context.expected_closing.clone(),
                    context: format!("Unclosed {:?}", context.context_type),
                    confidence: 0.8,
                    suggestions: vec!["Add closing delimiter".to_string()],
                });
            }
        }
    }

    /// Helper methods for context management
    fn enter_context(&mut self, context_type: ContextType, position: usize) {
        let expected_closing = match context_type {
            ContextType::Application | ContextType::List => vec![TokenKind::RightParen],
            ContextType::Vector => vec![TokenKind::RightParen], // Simplified
            _ => vec![],
        };

        self.context_stack.push(ParsingContext {
            context_type,
            start_position: position,
            expected_closing,
            nesting_depth: self.base_parser.nesting_depth(),
        });
    }

    fn exit_context(&mut self) -> Option<ParsingContext> {
        self.context_stack.pop()
    }

    fn current_context(&self) -> Option<&ParsingContext> {
        self.context_stack.last()
    }

    fn insert_token_at_position(&mut self, token: Token, position: usize) {
        // This would require modifying the token stream
        // For now, just record the insertion
        self.stats.repairs_attempted += 1;
    }

    fn skip_whitespace_and_comments(&mut self, comments: &mut Vec<CommentInfo>) {
        while !self.is_at_end() {
            let current = self.base_parser.current_token();
            match current.kind {
                TokenKind::LineComment => {
                    comments.push(CommentInfo {
                        content: current.text().to_string(),
                        span: current.span,
                        comment_type: CommentType::Line,
                        associated_expr: None,
                    });
                    self.base_parser.advance();
                }
                TokenKind::BlockComment => {
                    comments.push(CommentInfo {
                        content: current.text().to_string(),
                        span: current.span,
                        comment_type: CommentType::Block,
                        associated_expr: None,
                    });
                    self.base_parser.advance();
                }
                _ => break,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.base_parser.is_at_end()
    }
}

/// Information about recovery attempts
#[derive(Debug, Clone)]
pub struct RecoveryInfo {
    /// Whether recovery was successful
    pub success: bool,
    /// New position after recovery
    pub new_position: Option<usize>,
    /// Incomplete expression information
    pub incomplete: Option<IncompleteExpression>,
    /// Token that was inserted (if any)
    pub inserted_token: Option<InsertedToken>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_partial_parser_creation() {
        let tokens = vec![
            Token::new(TokenKind::LeftParen, Span::new(0, 1), "(".to_string()),
            Token::new(TokenKind::Identifier, Span::new(1, 6), "define".to_string()),
            Token::eof(Span::new(7, 0)),
        ];

        let parser = PartialParser::with_default_config(tokens);
        assert_eq!(parser.config.mode, ParsingMode::Partial);
        assert!(parser.config.use_placeholders);
    }

    #[test]
    fn test_incomplete_expression_detection() {
        let input = "(define x";
        let mut lexer = Lexer::new(input, None);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = PartialParser::with_default_config(tokens);
        let result = parser.parse_partial();

        assert!(!result.incomplete_expressions.is_empty());
        let incomplete = &result.incomplete_expressions[0];
        assert_eq!(incomplete.kind, IncompleteKind::IncompleteDefinition);
    }

    #[test]
    fn test_placeholder_generation() {
        let mut parser = PartialParser::with_default_config(vec![]);
        let placeholder = parser.create_placeholder_expression(0);

        assert!(placeholder.is_some());
        if let Some(Spanned {
            inner: Expr::Identifier(name),
            ..
        }) = placeholder
        {
            assert!(name.starts_with("__placeholder_"));
        }
    }

    #[test]
    fn test_parsing_statistics() {
        let input = "(+ 1 2) (define x 3)";
        let mut lexer = Lexer::new(input, None);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = PartialParser::with_default_config(tokens);
        let result = parser.parse_partial();

        assert!(result.stats.tokens_processed > 0);
        assert_eq!(result.stats.expressions_parsed, 2);
        assert!(result.stats.parse_time_us > 0);
    }
}
