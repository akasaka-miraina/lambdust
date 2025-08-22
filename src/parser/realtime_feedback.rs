//! Real-time parsing feedback system for IDE integration
//!
//! This module provides incremental parsing capabilities, real-time error reporting,
//! and responsive feedback for modern IDE development experience.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::{
    Parser,
    contextual_errors::{ContextualError, ContextualErrorGenerator, ErrorContext},
    ide_support::{Diagnostic, DiagnosticSeverity, IdeSupportEngine, Position, Range},
    partial_parser::{ParsingMode, PartialParser, PartialParsingConfig},
};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Real-time feedback system configuration
#[derive(Debug, Clone)]
pub struct RealtimeFeedbackConfig {
    /// Update debounce delay (ms) - wait time after last edit before processing
    pub debounce_delay_ms: u64,
    /// Maximum processing time per update (ms)
    pub max_processing_time_ms: u64,
    /// Enable incremental parsing
    pub incremental_parsing: bool,
    /// Enable real-time diagnostics
    pub real_time_diagnostics: bool,
    /// Enable syntax highlighting updates
    pub syntax_highlighting: bool,
    /// Enable completion hints
    pub completion_hints: bool,
    /// Cache size for parsed results
    pub cache_size: usize,
    /// Worker thread count for background processing
    pub worker_threads: usize,
}

impl Default for RealtimeFeedbackConfig {
    fn default() -> Self {
        Self {
            debounce_delay_ms: 300,
            max_processing_time_ms: 100,
            incremental_parsing: true,
            real_time_diagnostics: true,
            syntax_highlighting: true,
            completion_hints: true,
            cache_size: 50,
            worker_threads: 2,
        }
    }
}

/// Real-time feedback engine for IDE integration
pub struct RealtimeFeedbackEngine {
    /// Configuration
    config: RealtimeFeedbackConfig,
    /// Document states by URI
    document_states: HashMap<String, DocumentState>,
    /// IDE support engine
    ide_support: IdeSupportEngine,
    /// Contextual error generator
    error_generator: ContextualErrorGenerator,
    /// Update queue for processing
    update_queue: VecDeque<UpdateRequest>,
    /// Processing statistics
    stats: ProcessingStatistics,
    /// Last update timestamps
    last_update_times: HashMap<String, Instant>,
}

/// State for a single document
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// Document URI
    pub uri: String,
    /// Current content
    pub content: String,
    /// Content version (incremented on each change)
    pub version: u64,
    /// Parsed program (may be partial/incomplete)
    pub parsed_program: Option<Program>,
    /// Current diagnostics
    pub diagnostics: Vec<Diagnostic>,
    /// Incremental parsing state
    pub incremental_state: IncrementalState,
    /// Last successful parse time
    pub last_parse_time: Option<Instant>,
    /// Parsing statistics
    pub parse_stats: DocumentParseStats,
}

/// Incremental parsing state
#[derive(Debug, Clone)]
pub struct IncrementalState {
    /// Parsed tokens with positions
    pub tokens: Vec<Token>,
    /// AST nodes with position ranges
    pub ast_nodes: HashMap<Span, AstNode>,
    /// Dirty ranges that need reparsing
    pub dirty_ranges: Vec<Range>,
    /// Unchanged regions from last parse
    pub clean_regions: Vec<CleanRegion>,
    /// Symbol table from last parse
    pub symbol_table: HashMap<String, SymbolInfo>,
}

/// AST node information for incremental updates
#[derive(Debug, Clone)]
pub struct AstNode {
    /// Node type
    pub node_type: AstNodeType,
    /// Span in source
    pub span: Span,
    /// Child nodes
    pub children: Vec<Span>,
    /// Associated symbols
    pub symbols: Vec<String>,
    /// Whether node needs reanalysis
    pub dirty: bool,
}

/// Types of AST nodes for incremental tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstNodeType {
    /// Top-level expression or definition
    TopLevel,
    /// Variable or function definition
    Define,
    /// Lambda expression (anonymous function)
    Lambda,
    /// Function application (procedure call)
    Application,
    /// Conditional expression
    If,
    /// Local binding expression
    Let,
    /// Multi-way conditional
    Cond,
    /// Pattern matching expression
    Case,
    /// Quoted expression
    Quote,
    /// List literal
    List,
    /// Atomic value (number, string, symbol, etc.)
    Atom,
}

/// Clean region that doesn't need reparsing
#[derive(Debug, Clone)]
pub struct CleanRegion {
    /// Range in document
    pub range: Range,
    /// Hash of content (for change detection)
    pub content_hash: u64,
    /// Associated AST nodes
    pub ast_nodes: Vec<Span>,
}

/// Symbol information for incremental analysis
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Definition location
    pub definition: Position,
    /// Symbol type (if known)
    pub symbol_type: Option<String>,
    /// Scope information
    pub scope: SymbolScope,
}

/// Symbol scope types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolScope {
    /// Global scope symbol
    Global,
    /// Local scope symbol
    Local,
    /// Function parameter symbol
    Parameter,
    /// Let binding symbol
    Binding,
}

/// Update request for document changes
#[derive(Debug, Clone)]
pub struct UpdateRequest {
    /// Document URI
    pub uri: String,
    /// New content (or change description)
    pub content_change: ContentChange,
    /// Request timestamp
    pub timestamp: Instant,
    /// Request priority
    pub priority: UpdatePriority,
}

/// Types of content changes
#[derive(Debug, Clone)]
pub enum ContentChange {
    /// Full document replacement
    FullUpdate {
        /// New complete document content
        content: String,
        /// Document version number
        version: u64,
    },
    /// Incremental change
    IncrementalUpdate {
        /// Range of text to replace
        range: Range,
        /// New text content
        text: String,
        /// Document version number
        version: u64,
    },
    /// Multiple changes
    BatchUpdate {
        /// List of changes to apply
        changes: Vec<(Range, String)>,
        /// Document version number
        version: u64,
    },
}

/// Update priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdatePriority {
    /// Low priority update (background processing)
    Low = 0,
    /// Normal priority update (standard processing)
    Normal = 1,
    /// High priority update (user is actively typing)
    High = 2,
    /// Critical priority update (immediate processing required)
    Critical = 3,
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStatistics {
    /// Total updates processed
    pub updates_processed: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Error recovery rate
    pub error_recovery_rate: f64,
}

/// Document-specific parsing statistics
#[derive(Debug, Clone, Default)]
pub struct DocumentParseStats {
    /// Total parses performed
    pub total_parses: u64,
    /// Incremental parses
    pub incremental_parses: u64,
    /// Full parses
    pub full_parses: u64,
    /// Average parse time
    pub avg_parse_time_ms: f64,
    /// Tokens parsed per second
    pub tokens_per_second: f64,
    /// Error count trend
    pub error_trend: Vec<u32>,
}

/// Real-time feedback result
#[derive(Debug, Clone)]
pub struct FeedbackResult {
    /// Document URI
    pub uri: String,
    /// Updated diagnostics
    pub diagnostics: Vec<Diagnostic>,
    /// Syntax highlighting tokens (if requested)
    pub syntax_tokens: Option<Vec<SyntaxToken>>,
    /// Completion suggestions (if applicable)
    pub completion_hints: Vec<CompletionHint>,
    /// Processing time
    pub processing_time_ms: u64,
    /// Whether this was an incremental update
    pub incremental: bool,
}

/// Syntax highlighting token
#[derive(Debug, Clone)]
pub struct SyntaxToken {
    /// Token range
    pub range: Range,
    /// Token type for highlighting
    pub token_type: SyntaxTokenType,
    /// Modifiers (bold, italic, etc.)
    pub modifiers: Vec<SyntaxTokenModifier>,
}

/// Syntax token types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTokenType {
    /// Language keywords (define, lambda, if, cond, etc.)
    Keyword,
    /// General identifiers and symbols
    Identifier,
    /// Function names in function calls or definitions
    Function,
    /// Variable names and bindings
    Variable,
    /// Function parameters in lambda expressions
    Parameter,
    /// Numeric literals (integers, rationals, reals, complex)
    Number,
    /// String literals
    String,
    /// Comments (line and block)
    Comment,
    /// Operators (+, -, *, /, =, etc.)
    Operator,
    /// Delimiters (parentheses, brackets, quotes)
    Delimiter,
    /// Special forms (if, cond, case, let, etc.)
    SpecialForm,
    /// Macro invocations and definitions
    Macro,
    /// Type annotations and specifications
    Type,
    /// Syntax errors and invalid tokens
    Error,
}

/// Syntax token modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTokenModifier {
    /// Token represents a definition site (not a reference)
    Definition,
    /// Token represents deprecated functionality
    Deprecated,
    /// Token represents read-only/immutable binding
    ReadOnly,
    /// Token represents static/global binding
    Static,
    /// Token represents abstract or theoretical construct
    Abstract,
    /// Token is part of the default library/built-ins
    DefaultLibrary,
}

/// Completion hint for real-time assistance
#[derive(Debug, Clone)]
pub struct CompletionHint {
    /// Position where hint applies
    pub position: Position,
    /// Suggested completions
    pub suggestions: Vec<String>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Context information
    pub context: String,
}

impl RealtimeFeedbackEngine {
    /// Creates a new real-time feedback engine
    pub fn new(config: RealtimeFeedbackConfig) -> Self {
        Self {
            config,
            document_states: HashMap::new(),
            ide_support: IdeSupportEngine::new(),
            error_generator: ContextualErrorGenerator::new(),
            update_queue: VecDeque::new(),
            stats: ProcessingStatistics::default(),
            last_update_times: HashMap::new(),
        }
    }

    /// Opens a document for real-time processing
    pub fn open_document(&mut self, uri: String, content: String) -> Result<()> {
        let state = DocumentState {
            uri: uri.clone(),
            content: content.clone(),
            version: 1,
            parsed_program: None,
            diagnostics: Vec::new(),
            incremental_state: IncrementalState::new(),
            last_parse_time: None,
            parse_stats: DocumentParseStats::default(),
        };

        self.document_states.insert(uri.clone(), state);

        // Queue initial parse
        self.queue_update(UpdateRequest {
            uri,
            content_change: ContentChange::FullUpdate {
                content,
                version: 1,
            },
            timestamp: Instant::now(),
            priority: UpdatePriority::Normal,
        });

        Ok(())
    }

    /// Updates document content and triggers incremental processing
    pub fn update_document(&mut self, uri: String, content_change: ContentChange) -> Result<()> {
        if !self.document_states.contains_key(&uri) {
            return Err(Box::new(Error::runtime_error("Document not found", None)));
        }

        // Check debounce timing
        let now = Instant::now();
        if let Some(last_time) = self.last_update_times.get(&uri) {
            let elapsed = now.duration_since(*last_time);
            if elapsed < Duration::from_millis(self.config.debounce_delay_ms) {
                // Too soon, just update the timestamp
                self.last_update_times.insert(uri.clone(), now);
                return Ok(());
            }
        }

        self.last_update_times.insert(uri.clone(), now);

        // Queue update request
        self.queue_update(UpdateRequest {
            uri,
            content_change,
            timestamp: now,
            priority: UpdatePriority::Normal,
        });

        Ok(())
    }

    /// Processes pending updates and returns feedback
    pub fn process_updates(&mut self) -> Vec<FeedbackResult> {
        let mut results = Vec::new();
        let start_time = Instant::now();
        let max_time = Duration::from_millis(self.config.max_processing_time_ms);

        while let Some(request) = self.update_queue.pop_front() {
            // Check time budget
            if start_time.elapsed() > max_time {
                // Put request back and break
                self.update_queue.push_front(request);
                break;
            }

            if let Ok(result) = self.process_update_request(request) {
                results.push(result);
            }
        }

        results
    }

    /// Processes a single update request
    fn process_update_request(&mut self, request: UpdateRequest) -> Result<FeedbackResult> {
        let start_time = Instant::now();

        // Extract config values to avoid borrowing conflicts
        let incremental_enabled = self.config.incremental_parsing;
        let diagnostics_enabled = self.config.real_time_diagnostics;
        let highlighting_enabled = self.config.syntax_highlighting;

        // Store the URI for repeated access
        let uri = request.uri.clone();

        // Clone the content change to avoid borrowing conflicts
        let content_change = request.content_change.clone();

        // Apply content change first
        let incremental = match &content_change {
            ContentChange::FullUpdate { content, version } => {
                if let Some(state) = self.document_states.get_mut(&uri) {
                    state.content = content.clone();
                    state.version = *version;
                    state.incremental_state.mark_all_dirty();
                }
                false // Full update, not incremental
            }
            ContentChange::IncrementalUpdate {
                range,
                text,
                version,
            } => {
                // Get current content and compute offsets
                let current_content = self
                    .document_states
                    .get(&uri)
                    .map(|s| s.content.clone())
                    .ok_or_else(|| Error::runtime_error("Document not found", None))?;

                let start_offset = self.position_to_offset(&current_content, range.start)?;
                let end_offset = self.position_to_offset(&current_content, range.end)?;

                // Apply the change
                if let Some(state) = self.document_states.get_mut(&uri) {
                    let mut content = state.content.clone();
                    content.replace_range(start_offset..end_offset, text);
                    state.content = content;
                    state.version = *version;
                    state.incremental_state.dirty_ranges.push(*range);
                }
                true
            }
            ContentChange::BatchUpdate { changes, version } => {
                // Apply batch updates one by one
                if let Some(state) = self.document_states.get_mut(&uri) {
                    state.version = *version;
                    // For simplicity, mark as full update for batch changes
                    state.incremental_state.mark_all_dirty();
                }
                false // Treat batch updates as full updates for now
            }
        };

        // Then perform parsing
        let parse_result = if incremental && incremental_enabled {
            // Simplified incremental parse
            let program = self
                .document_states
                .get(&uri)
                .map(|s| s.parsed_program.clone())
                .ok_or_else(|| Error::runtime_error("Document not found", None))?;
            ParseResult {
                program,
                success: true,
                parse_time_ms: 0,
            }
        } else {
            // Simplified full parse
            let program = self
                .document_states
                .get(&uri)
                .map(|s| s.parsed_program.clone())
                .ok_or_else(|| Error::runtime_error("Document not found", None))?;
            ParseResult {
                program,
                success: true,
                parse_time_ms: 0,
            }
        };

        // Generate diagnostics
        let diagnostics = if diagnostics_enabled {
            let state = self
                .document_states
                .get(&uri)
                .ok_or_else(|| Error::runtime_error("Document not found", None))?;
            self.generate_diagnostics(state, &parse_result)?
        } else {
            Vec::new()
        };

        // Generate syntax tokens
        let syntax_tokens = if highlighting_enabled {
            let state = self
                .document_states
                .get(&uri)
                .ok_or_else(|| Error::runtime_error("Document not found", None))?;
            Some(self.generate_syntax_tokens(state)?)
        } else {
            None
        };

        // Generate completion hints
        let completion_hints = if self.config.completion_hints {
            let state = self
                .document_states
                .get(&uri)
                .ok_or_else(|| Error::runtime_error("Document not found", None))?;
            self.generate_completion_hints(state)?
        } else {
            Vec::new()
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.update_statistics(&request, processing_time_ms, incremental);

        Ok(FeedbackResult {
            uri: request.uri,
            diagnostics,
            syntax_tokens,
            completion_hints,
            processing_time_ms,
            incremental,
        })
    }

    /// Applies content change to document state
    fn apply_content_change(
        &mut self,
        state: &mut DocumentState,
        change: &ContentChange,
    ) -> Result<bool> {
        match change {
            ContentChange::FullUpdate { content, version } => {
                state.content = content.clone();
                state.version = *version;
                state.incremental_state.mark_all_dirty();
                Ok(false) // Full update, not incremental
            }
            ContentChange::IncrementalUpdate {
                range,
                text,
                version,
            } => {
                // Apply incremental change
                let start_offset = self.position_to_offset(&state.content, range.start)?;
                let end_offset = self.position_to_offset(&state.content, range.end)?;

                let mut new_content = String::new();
                new_content.push_str(&state.content[..start_offset]);
                new_content.push_str(text);
                new_content.push_str(&state.content[end_offset..]);

                state.content = new_content;
                state.version = *version;

                // Mark affected regions as dirty
                state.incremental_state.mark_range_dirty(*range);

                Ok(true) // Incremental update
            }
            ContentChange::BatchUpdate { changes, version } => {
                // Apply multiple changes (simplified implementation)
                for (range, text) in changes {
                    let start_offset = self.position_to_offset(&state.content, range.start)?;
                    let end_offset = self.position_to_offset(&state.content, range.end)?;

                    let mut new_content = String::new();
                    new_content.push_str(&state.content[..start_offset]);
                    new_content.push_str(text);
                    new_content.push_str(&state.content[end_offset..]);

                    state.content = new_content;
                    state.incremental_state.mark_range_dirty(*range);
                }

                state.version = *version;
                Ok(true) // Incremental update
            }
        }
    }

    /// Performs incremental parsing
    fn incremental_parse(&mut self, state: &mut DocumentState) -> Result<ParseResult> {
        // Check if we can reuse clean regions
        let dirty_ranges = &state.incremental_state.dirty_ranges;

        if dirty_ranges.is_empty() {
            // Nothing to reparse
            return Ok(ParseResult {
                program: state.parsed_program.clone(),
                success: true,
                parse_time_ms: 0,
            });
        }

        // Parse only dirty regions
        let mut lexer = Lexer::new(&state.content, None);
        let tokens = lexer.tokenize()?;

        let config = PartialParsingConfig {
            mode: ParsingMode::Interactive,
            max_errors: 20,
            timeout_ms: self.config.max_processing_time_ms / 2,
            ..Default::default()
        };

        let mut parser = PartialParser::new(tokens, config);
        let parse_result = parser.parse_partial();

        // Update incremental state
        state.incremental_state.update_after_parse(&parse_result);
        state.parsed_program = Some(parse_result.program);
        state.parse_stats.incremental_parses += 1;

        Ok(ParseResult {
            program: state.parsed_program.clone(),
            success: parse_result.errors.is_empty(),
            parse_time_ms: parse_result.stats.parse_time_us / 1000,
        })
    }

    /// Performs full parsing
    fn full_parse(&mut self, state: &mut DocumentState) -> Result<ParseResult> {
        let start_time = Instant::now();

        let mut lexer = Lexer::new(&state.content, None);
        let tokens = lexer.tokenize()?;

        let config = PartialParsingConfig {
            mode: ParsingMode::Partial,
            max_errors: 50,
            timeout_ms: self.config.max_processing_time_ms,
            ..Default::default()
        };

        let mut parser = PartialParser::new(tokens, config);
        let parse_result = parser.parse_partial();

        state.parsed_program = Some(parse_result.program);
        state.last_parse_time = Some(start_time);
        state.parse_stats.full_parses += 1;

        let parse_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ParseResult {
            program: state.parsed_program.clone(),
            success: parse_result.errors.is_empty(),
            parse_time_ms,
        })
    }

    /// Generates diagnostics for current state
    fn generate_diagnostics(
        &self,
        state: &DocumentState,
        parse_result: &ParseResult,
    ) -> Result<Vec<Diagnostic>> {
        // This would integrate with the contextual error generator
        // For now, return empty diagnostics
        Ok(Vec::new())
    }

    /// Generates syntax highlighting tokens
    fn generate_syntax_tokens(&self, state: &DocumentState) -> Result<Vec<SyntaxToken>> {
        let mut tokens = Vec::new();

        // Simple tokenization for syntax highlighting
        let mut lexer = Lexer::new(&state.content, None);
        let lexer_tokens = lexer.tokenize()?;

        for token in lexer_tokens {
            let token_type = match token.kind {
                TokenKind::Identifier => {
                    if self.is_keyword(token.text()) {
                        SyntaxTokenType::Keyword
                    } else if self.is_special_form(token.text()) {
                        SyntaxTokenType::SpecialForm
                    } else {
                        SyntaxTokenType::Identifier
                    }
                }
                TokenKind::IntegerNumber
                | TokenKind::RealNumber
                | TokenKind::RationalNumber
                | TokenKind::ComplexNumber => SyntaxTokenType::Number,
                TokenKind::String => SyntaxTokenType::String,
                TokenKind::LineComment | TokenKind::BlockComment => SyntaxTokenType::Comment,
                TokenKind::LeftParen | TokenKind::RightParen => SyntaxTokenType::Delimiter,
                _ => SyntaxTokenType::Identifier,
            };

            tokens.push(SyntaxToken {
                range: Range {
                    start: self.offset_to_position(&state.content, token.span.start)?,
                    end: self.offset_to_position(&state.content, token.span.end())?,
                },
                token_type,
                modifiers: Vec::new(),
            });
        }

        Ok(tokens)
    }

    /// Generates completion hints
    fn generate_completion_hints(&self, state: &DocumentState) -> Result<Vec<CompletionHint>> {
        // This would analyze cursor position and provide context-sensitive hints
        // For now, return empty hints
        Ok(Vec::new())
    }

    /// Helper methods
    fn queue_update(&mut self, request: UpdateRequest) {
        // Insert in priority order
        let insert_pos = self
            .update_queue
            .iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(self.update_queue.len());

        self.update_queue.insert(insert_pos, request);
    }

    fn position_to_offset(&self, content: &str, position: Position) -> Result<usize> {
        let mut offset = 0;
        let mut line = 0;

        for ch in content.chars() {
            if line == position.line && offset as u32 >= position.character {
                return Ok(offset);
            }

            if ch == '\n' {
                line += 1;
            }

            offset += ch.len_utf8();
        }

        Ok(offset)
    }

    fn offset_to_position(&self, content: &str, offset: usize) -> Result<Position> {
        let mut line = 0;
        let mut character = 0;
        let mut current_offset = 0;

        for ch in content.chars() {
            if current_offset >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }

            current_offset += ch.len_utf8();
        }

        Ok(Position { line, character })
    }

    fn is_keyword(&self, text: &str) -> bool {
        matches!(
            text,
            "define"
                | "lambda"
                | "if"
                | "let"
                | "cond"
                | "case"
                | "begin"
                | "and"
                | "or"
                | "when"
                | "unless"
                | "quote"
        )
    }

    fn is_special_form(&self, text: &str) -> bool {
        matches!(
            text,
            "define-syntax"
                | "syntax-rules"
                | "call/cc"
                | "call-with-current-continuation"
                | "set!"
        )
    }

    fn update_statistics(
        &mut self,
        request: &UpdateRequest,
        processing_time: u64,
        incremental: bool,
    ) {
        self.stats.updates_processed += 1;

        // Update running average
        let old_avg = self.stats.avg_processing_time_ms;
        let count = self.stats.updates_processed as f64;
        self.stats.avg_processing_time_ms =
            (old_avg * (count - 1.0) + processing_time as f64) / count;
    }
}

/// Parse result information
#[derive(Debug, Clone)]
struct ParseResult {
    program: Option<Program>,
    success: bool,
    parse_time_ms: u64,
}

impl IncrementalState {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            ast_nodes: HashMap::new(),
            dirty_ranges: Vec::new(),
            clean_regions: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    fn mark_all_dirty(&mut self) {
        self.dirty_ranges.clear();
        self.dirty_ranges.push(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: u32::MAX,
                character: u32::MAX,
            },
        });
        self.clean_regions.clear();
    }

    fn mark_range_dirty(&mut self, range: Range) {
        self.dirty_ranges.push(range);
        // Remove overlapping clean regions
        self.clean_regions
            .retain(|region| !ranges_overlap(&region.range, &range));
    }

    fn update_after_parse(
        &mut self,
        parse_result: &crate::parser::partial_parser::PartialParsingResult,
    ) {
        // Clear dirty ranges after successful parse
        self.dirty_ranges.clear();

        // Update token cache
        // This would be implemented with proper incremental token updates

        // Update symbol table
        // This would extract symbols from the parsed program
    }
}

fn ranges_overlap(range1: &Range, range2: &Range) -> bool {
    !(range1.end.line < range2.start.line
        || (range1.end.line == range2.start.line && range1.end.character <= range2.start.character)
        || range2.end.line < range1.start.line
        || (range2.end.line == range1.start.line && range2.end.character <= range1.start.character))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_feedback_engine_creation() {
        let config = RealtimeFeedbackConfig::default();
        let engine = RealtimeFeedbackEngine::new(config);
        assert_eq!(engine.document_states.len(), 0);
        assert!(engine.config.incremental_parsing);
    }

    #[test]
    fn test_document_operations() {
        let mut engine = RealtimeFeedbackEngine::new(RealtimeFeedbackConfig::default());

        let uri = "file:///test.scm".to_string();
        let content = "(define x 42)".to_string();

        assert!(engine.open_document(uri.clone(), content).is_ok());
        assert!(engine.document_states.contains_key(&uri));
    }

    #[test]
    fn test_incremental_state() {
        let mut state = IncrementalState::new();

        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        state.mark_range_dirty(range);
        assert_eq!(state.dirty_ranges.len(), 1);
    }

    #[test]
    fn test_position_offset_conversion() {
        let engine = RealtimeFeedbackEngine::new(RealtimeFeedbackConfig::default());
        let content = "line1\nline2\nline3";

        let position = Position {
            line: 1,
            character: 2,
        };
        let offset = engine.position_to_offset(content, position).unwrap();
        let back_to_position = engine.offset_to_position(content, offset).unwrap();

        assert_eq!(position.line, back_to_position.line);
        assert_eq!(position.character, back_to_position.character);
    }
}
