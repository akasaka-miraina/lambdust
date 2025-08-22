//! Advanced error recovery strategies for robust parsing
//!
//! This module implements multiple error recovery strategies beyond simple panic mode,
//! providing sophisticated error synchronization and AST repair capabilities.

use crate::ast::{Expr, Formals, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use std::collections::{HashMap, HashSet, VecDeque};

/// Error recovery strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Simple panic mode - skip to synchronization points
    PanicMode,
    /// Local repair - attempt minimal fixes
    LocalRepair,
    /// Phrase-level recovery - reconstruct malformed phrases
    PhraseLevel,
    /// Error productions - use grammar extensions for common errors
    ErrorProductions,
    /// Contextual recovery - use surrounding context for guidance
    Contextual,
}

/// Recovery point types for parser synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SynchronizationPoint {
    /// Statement boundaries (top-level expressions)
    StatementBoundary,
    /// Expression boundaries
    ExpressionBoundary,
    /// Block boundaries (begin, let, etc.)
    BlockBoundary,
    /// List boundaries (parentheses)
    ListBoundary,
    /// Definition boundaries (define, define-syntax)
    DefinitionBoundary,
    /// Special form boundaries
    SpecialFormBoundary,
}

/// Error recovery context maintains state for sophisticated recovery
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// Current recovery strategy
    strategy: RecoveryStrategy,
    /// Synchronization points hierarchy
    sync_points: VecDeque<SynchronizationPoint>,
    /// Expected tokens at current position
    expected_tokens: HashSet<TokenKind>,
    /// Recently consumed tokens for context
    recent_tokens: VecDeque<Token>,
    /// Nesting context for balanced constructs
    nesting_context: Vec<(TokenKind, Span)>, // (open_token, span)
    /// Error repair attempts made
    repair_attempts: usize,
    /// Maximum repair attempts before fallback
    max_repair_attempts: usize,
}

impl RecoveryContext {
    /// Creates a new recovery context
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            sync_points: VecDeque::new(),
            expected_tokens: HashSet::new(),
            recent_tokens: VecDeque::with_capacity(5),
            nesting_context: Vec::new(),
            repair_attempts: 0,
            max_repair_attempts: 3,
        }
    }

    /// Pushes a synchronization point
    pub fn push_sync_point(&mut self, point: SynchronizationPoint) {
        self.sync_points.push_back(point);
    }

    /// Pops the most recent synchronization point
    pub fn pop_sync_point(&mut self) -> Option<SynchronizationPoint> {
        self.sync_points.pop_back()
    }

    /// Adds expected tokens for current position
    pub fn add_expected_tokens(&mut self, tokens: &[TokenKind]) {
        for token in tokens {
            self.expected_tokens.insert(*token);
        }
    }

    /// Records a consumed token for context
    pub fn record_token(&mut self, token: Token) {
        if self.recent_tokens.len() >= 5 {
            self.recent_tokens.pop_front();
        }
        self.recent_tokens.push_back(token);
    }

    /// Enters a nesting context (e.g., parentheses, brackets)
    pub fn enter_nesting(&mut self, open_token: TokenKind, span: Span) {
        self.nesting_context.push((open_token, span));
    }

    /// Exits a nesting context
    pub fn exit_nesting(&mut self) -> Option<(TokenKind, Span)> {
        self.nesting_context.pop()
    }

    /// Checks if we can attempt repair
    pub fn can_attempt_repair(&self) -> bool {
        self.repair_attempts < self.max_repair_attempts
    }

    /// Records a repair attempt
    pub fn record_repair_attempt(&mut self) {
        self.repair_attempts += 1;
    }

    /// Resets repair attempts counter
    pub fn reset_repair_attempts(&mut self) {
        self.repair_attempts = 0;
    }

    /// Gets the current nesting depth
    pub fn nesting_depth(&self) -> usize {
        self.nesting_context.len()
    }

    /// Checks if we're inside a specific construct
    pub fn is_inside(&self, construct: TokenKind) -> bool {
        self.nesting_context
            .iter()
            .any(|(token, _)| *token == construct)
    }
}

/// Advanced error recovery engine
pub struct ErrorRecoveryEngine {
    /// Recovery context
    context: RecoveryContext,
    /// Error production rules for common mistakes
    error_productions: HashMap<String, ErrorProduction>,
    /// Repair rules for local fixes
    repair_rules: Vec<RepairRule>,
}

/// Error production for handling common syntax errors
#[derive(Debug, Clone)]
pub struct ErrorProduction {
    /// Pattern that triggers this production
    pattern: Vec<TokenKind>,
    /// Suggested fix
    fix: ErrorFix,
    /// Human-readable explanation
    explanation: String,
}

/// Types of error fixes
#[derive(Debug, Clone)]
pub enum ErrorFix {
    /// Insert missing tokens
    Insert(Vec<TokenKind>),
    /// Delete erroneous tokens
    Delete(usize), // number of tokens to delete
    /// Replace tokens
    Replace {
        /// Number of tokens to delete from the current position
        delete_count: usize,
        /// Tokens to insert in place of the deleted tokens
        insert: Vec<TokenKind>,
    },
    /// Rearrange tokens
    Rearrange(Vec<usize>), // new order of token indices
}

/// Local repair rule for minor syntax errors
#[derive(Debug, Clone)]
pub struct RepairRule {
    /// Condition for applying this rule
    condition: RepairCondition,
    /// Action to take
    action: RepairAction,
    /// Priority (higher = more preferred)
    priority: i32,
}

/// Conditions for repair rules
#[derive(Debug, Clone)]
pub enum RepairCondition {
    /// Missing closing delimiter
    MissingClosing(TokenKind),
    /// Missing opening delimiter
    MissingOpening(TokenKind),
    /// Wrong delimiter type
    WrongDelimiter {
        /// The type of delimiter that was expected at this position
        expected: TokenKind,
        /// The type of delimiter that was actually found
        found: TokenKind,
    },
    /// Missing separator
    MissingSeparator(TokenKind),
    /// Typo in identifier/keyword
    IdentifierTypo {
        /// The identifier or keyword that was found (potentially misspelled)
        found: String,
        /// List of candidate corrections based on similarity matching
        candidates: Vec<String>,
    },
}

/// Repair actions
#[derive(Debug, Clone)]
pub enum RepairAction {
    /// Insert a token
    InsertToken(TokenKind),
    /// Replace current token
    ReplaceToken(TokenKind),
    /// Skip current token
    SkipToken,
    /// Suggest identifier correction
    SuggestCorrection(String),
}

impl ErrorRecoveryEngine {
    /// Creates a new error recovery engine
    pub fn new(strategy: RecoveryStrategy) -> Self {
        let mut engine = Self {
            context: RecoveryContext::new(strategy),
            error_productions: HashMap::new(),
            repair_rules: Vec::new(),
        };

        engine.init_error_productions();
        engine.init_repair_rules();
        engine
    }

    /// Initializes common error productions
    fn init_error_productions(&mut self) {
        // Missing closing parenthesis
        self.error_productions.insert(
            "missing_close_paren".to_string(),
            ErrorProduction {
                pattern: vec![TokenKind::LeftParen],
                fix: ErrorFix::Insert(vec![TokenKind::RightParen]),
                explanation: "Missing closing parenthesis".to_string(),
            },
        );

        // Missing opening parenthesis in define
        self.error_productions.insert(
            "define_missing_open_paren".to_string(),
            ErrorProduction {
                pattern: vec![TokenKind::Identifier, TokenKind::Identifier],
                fix: ErrorFix::Insert(vec![TokenKind::LeftParen]),
                explanation: "Missing opening parenthesis in define".to_string(),
            },
        );

        // Wrong quote usage
        self.error_productions.insert(
            "wrong_quote".to_string(),
            ErrorProduction {
                pattern: vec![TokenKind::String],
                fix: ErrorFix::Replace {
                    delete_count: 0,
                    insert: vec![TokenKind::Quote],
                },
                explanation: "Use quote (') for symbols, not double quotes".to_string(),
            },
        );
    }

    /// Initializes repair rules
    fn init_repair_rules(&mut self) {
        // High priority: Fix missing closing delimiters
        self.repair_rules.push(RepairRule {
            condition: RepairCondition::MissingClosing(TokenKind::RightParen),
            action: RepairAction::InsertToken(TokenKind::RightParen),
            priority: 100,
        });

        // Medium priority: Fix wrong delimiters
        self.repair_rules.push(RepairRule {
            condition: RepairCondition::WrongDelimiter {
                expected: TokenKind::RightParen,
                found: TokenKind::RightParen, // This will be dynamically set
            },
            action: RepairAction::ReplaceToken(TokenKind::RightParen),
            priority: 50,
        });

        // Low priority: Skip unexpected tokens
        self.repair_rules.push(RepairRule {
            condition: RepairCondition::MissingSeparator(TokenKind::LeftParen),
            action: RepairAction::SkipToken,
            priority: 10,
        });
    }

    /// Attempts to recover from a parse error using multiple strategies
    pub fn recover_from_error(
        &mut self,
        tokens: &[Token],
        position: usize,
        error: &Error,
    ) -> RecoveryResult {
        match self.context.strategy {
            RecoveryStrategy::PanicMode => self.panic_mode_recovery(tokens, position, error),
            RecoveryStrategy::LocalRepair => self.local_repair_recovery(tokens, position, error),
            RecoveryStrategy::PhraseLevel => self.phrase_level_recovery(tokens, position, error),
            RecoveryStrategy::ErrorProductions => {
                self.error_production_recovery(tokens, position, error)
            }
            RecoveryStrategy::Contextual => self.contextual_recovery(tokens, position, error),
        }
    }

    /// Panic mode recovery - skip to synchronization points
    fn panic_mode_recovery(
        &mut self,
        tokens: &[Token],
        position: usize,
        _error: &Error,
    ) -> RecoveryResult {
        let mut pos = position;
        let mut paren_count = 0;

        // Skip tokens until we find a synchronization point
        while pos < tokens.len() {
            match tokens[pos].kind {
                TokenKind::LeftParen => paren_count += 1,
                TokenKind::RightParen => {
                    if paren_count > 0 {
                        paren_count -= 1;
                    } else {
                        // Found balanced closing paren
                        return RecoveryResult::Synchronized {
                            new_position: pos + 1,
                            recovery_type: RecoveryType::PanicMode,
                            repair: None,
                        };
                    }
                }
                TokenKind::Eof => break,
                _ => {}
            }
            pos += 1;
        }

        RecoveryResult::Failed {
            reason: "No synchronization point found".to_string(),
        }
    }

    /// Local repair recovery - attempt minimal fixes
    fn local_repair_recovery(
        &mut self,
        tokens: &[Token],
        position: usize,
        error: &Error,
    ) -> RecoveryResult {
        if !self.context.can_attempt_repair() {
            return self.panic_mode_recovery(tokens, position, error);
        }

        self.context.record_repair_attempt();

        // Try to identify the error type and apply appropriate repair
        if position < tokens.len() {
            let current_token = &tokens[position];

            // Check for missing closing delimiters
            if self.context.nesting_depth() > 0 {
                if let Some((open_token, _)) = self.context.nesting_context.last() {
                    let expected_close = match open_token {
                        TokenKind::LeftParen => TokenKind::RightParen,
                        _ => return self.panic_mode_recovery(tokens, position, error),
                    };

                    return RecoveryResult::Repaired {
                        new_position: position,
                        recovery_type: RecoveryType::LocalRepair,
                        repair: Some(TokenRepair::Insert {
                            position,
                            token: Token::new(
                                expected_close,
                                Span::new(position, 1),
                                ")".to_string(),
                            ),
                        }),
                    };
                }
            }

            // Check for typos in identifiers
            if let TokenKind::Identifier = current_token.kind {
                if let Some(correction) = self.suggest_identifier_correction(current_token.text()) {
                    return RecoveryResult::Repaired {
                        new_position: position + 1,
                        recovery_type: RecoveryType::LocalRepair,
                        repair: Some(TokenRepair::Replace {
                            position,
                            old_token: current_token.clone(),
                            new_token: Token::new(
                                TokenKind::Identifier,
                                current_token.span,
                                correction.clone(),
                            ),
                        }),
                    };
                }
            }
        }

        // Fallback to panic mode
        self.panic_mode_recovery(tokens, position, error)
    }

    /// Phrase-level recovery - reconstruct malformed phrases
    fn phrase_level_recovery(
        &mut self,
        tokens: &[Token],
        position: usize,
        _error: &Error,
    ) -> RecoveryResult {
        // Analyze the phrase structure around the error
        let phrase_start = self.find_phrase_start(tokens, position);
        let phrase_end = self.find_phrase_end(tokens, position);

        if let (Some(start), Some(end)) = (phrase_start, phrase_end) {
            // Try to reconstruct the phrase
            if let Some(repair) = self.reconstruct_phrase(tokens, start, end) {
                return RecoveryResult::Repaired {
                    new_position: end,
                    recovery_type: RecoveryType::PhraseLevel,
                    repair: Some(repair),
                };
            }
        }

        // Fallback to local repair
        self.local_repair_recovery(tokens, position, _error)
    }

    /// Error production recovery - use grammar extensions
    fn error_production_recovery(
        &mut self,
        tokens: &[Token],
        position: usize,
        _error: &Error,
    ) -> RecoveryResult {
        // Look for patterns that match error productions
        for (name, production) in &self.error_productions {
            if self.matches_error_pattern(tokens, position, &production.pattern) {
                let repair = self.apply_error_fix(tokens, position, &production.fix);
                return RecoveryResult::Repaired {
                    new_position: position + production.pattern.len(),
                    recovery_type: RecoveryType::ErrorProduction,
                    repair: Some(repair),
                };
            }
        }

        // Fallback to phrase-level recovery
        self.phrase_level_recovery(tokens, position, _error)
    }

    /// Contextual recovery - use surrounding context
    fn contextual_recovery(
        &mut self,
        tokens: &[Token],
        position: usize,
        _error: &Error,
    ) -> RecoveryResult {
        // Analyze the context to make intelligent recovery decisions
        let context_info = self.analyze_context(tokens, position);

        match context_info {
            ContextInfo::InsideDefinition => {
                // Special handling for definition context
                self.recover_in_definition(tokens, position)
            }
            ContextInfo::InsideLambda => {
                // Special handling for lambda context
                self.recover_in_lambda(tokens, position)
            }
            ContextInfo::InsideApplication => {
                // Special handling for application context
                self.recover_in_application(tokens, position)
            }
            ContextInfo::TopLevel => {
                // Top-level recovery
                self.recover_at_top_level(tokens, position)
            }
            ContextInfo::Unknown => {
                // Fallback to error production recovery
                self.error_production_recovery(tokens, position, _error)
            }
        }
    }

    /// Suggests identifier corrections for common typos
    fn suggest_identifier_correction(&self, identifier: &str) -> Option<String> {
        let common_keywords = vec![
            "define",
            "lambda",
            "if",
            "cond",
            "case",
            "let",
            "let*",
            "letrec",
            "begin",
            "quote",
            "quasiquote",
            "unquote",
            "unquote-splicing",
            "and",
            "or",
            "when",
            "unless",
            "guard",
            "call-with-current-continuation",
            "call/cc",
            "define-syntax",
            "syntax-rules",
        ];

        // Simple edit distance-based suggestion
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for keyword in &common_keywords {
            let distance = edit_distance(identifier, keyword);
            if distance <= 2 && distance < best_distance {
                best_distance = distance;
                best_match = Some(keyword.to_string());
            }
        }

        best_match
    }

    /// Helper methods for recovery implementation
    fn find_phrase_start(&self, tokens: &[Token], position: usize) -> Option<usize> {
        // Find the start of the current phrase (usually an opening paren)
        let mut pos = position;
        while pos > 0 {
            if matches!(tokens[pos].kind, TokenKind::LeftParen) {
                return Some(pos);
            }
            pos -= 1;
        }
        None
    }

    fn find_phrase_end(&self, tokens: &[Token], position: usize) -> Option<usize> {
        // Find the end of the current phrase (matching closing paren)
        let mut pos = position;
        let mut paren_count = 0;

        while pos < tokens.len() {
            match tokens[pos].kind {
                TokenKind::LeftParen => paren_count += 1,
                TokenKind::RightParen => {
                    if paren_count > 0 {
                        paren_count -= 1;
                    }
                    if paren_count == 0 {
                        return Some(pos);
                    }
                }
                TokenKind::Eof => break,
                _ => {}
            }
            pos += 1;
        }
        None
    }

    fn reconstruct_phrase(
        &self,
        tokens: &[Token],
        start: usize,
        end: usize,
    ) -> Option<TokenRepair> {
        // Analyze the phrase and suggest reconstruction
        // This is a simplified implementation
        if end > start && tokens[start].kind == TokenKind::LeftParen {
            // Check if we need to insert a closing paren
            let mut paren_count = 0;
            for (i, token) in tokens
                .iter()
                .enumerate()
                .take(end.min(tokens.len() - 1) + 1)
                .skip(start)
            {
                let actual_index = start + i;
                match token.kind {
                    TokenKind::LeftParen => paren_count += 1,
                    TokenKind::RightParen => paren_count -= 1,
                    _ => {}
                }
            }

            if paren_count > 0 {
                return Some(TokenRepair::Insert {
                    position: end,
                    token: Token::new(TokenKind::RightParen, Span::new(end, 1), ")".to_string()),
                });
            }
        }
        None
    }

    fn matches_error_pattern(
        &self,
        tokens: &[Token],
        position: usize,
        pattern: &[TokenKind],
    ) -> bool {
        if position + pattern.len() > tokens.len() {
            return false;
        }

        for (i, expected_kind) in pattern.iter().enumerate() {
            if tokens[position + i].kind != *expected_kind {
                return false;
            }
        }

        true
    }

    fn apply_error_fix(&self, tokens: &[Token], position: usize, fix: &ErrorFix) -> TokenRepair {
        match fix {
            ErrorFix::Insert(insert_tokens) => TokenRepair::Insert {
                position,
                token: Token::new(insert_tokens[0], Span::new(position, 0), "".to_string()),
            },
            ErrorFix::Delete(count) => TokenRepair::Delete {
                start: position,
                count: *count,
            },
            ErrorFix::Replace {
                delete_count,
                insert,
            } => TokenRepair::Replace {
                position,
                old_token: tokens[position].clone(),
                new_token: Token::new(insert[0], tokens[position].span, "".to_string()),
            },
            ErrorFix::Rearrange(_) => {
                // Simplified - just skip for now
                TokenRepair::Delete {
                    start: position,
                    count: 1,
                }
            }
        }
    }

    fn analyze_context(&self, tokens: &[Token], position: usize) -> ContextInfo {
        // Analyze recent tokens to determine context
        if let Some(recent) = self.context.recent_tokens.back() {
            match recent.kind {
                TokenKind::Identifier if recent.text() == "define" => ContextInfo::InsideDefinition,
                TokenKind::Identifier if recent.text() == "lambda" => ContextInfo::InsideLambda,
                TokenKind::LeftParen => ContextInfo::InsideApplication,
                _ => ContextInfo::Unknown,
            }
        } else {
            ContextInfo::TopLevel
        }
    }

    fn recover_in_definition(&self, tokens: &[Token], position: usize) -> RecoveryResult {
        // Specialized recovery for define forms
        RecoveryResult::Failed {
            reason: "Definition recovery not implemented".to_string(),
        }
    }

    fn recover_in_lambda(&self, tokens: &[Token], position: usize) -> RecoveryResult {
        // Specialized recovery for lambda forms
        RecoveryResult::Failed {
            reason: "Lambda recovery not implemented".to_string(),
        }
    }

    fn recover_in_application(&self, tokens: &[Token], position: usize) -> RecoveryResult {
        // Specialized recovery for application forms
        RecoveryResult::Failed {
            reason: "Application recovery not implemented".to_string(),
        }
    }

    fn recover_at_top_level(&self, tokens: &[Token], position: usize) -> RecoveryResult {
        // Top-level recovery
        RecoveryResult::Failed {
            reason: "Top-level recovery not implemented".to_string(),
        }
    }
}

/// Result of error recovery attempt
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Successfully synchronized to a recovery point
    Synchronized {
        /// New position in the token stream after synchronization
        new_position: usize,
        /// The type of recovery strategy that was used
        recovery_type: RecoveryType,
        /// Optional repair operation that was applied during synchronization
        repair: Option<TokenRepair>,
    },
    /// Successfully repaired the error
    Repaired {
        /// New position in the token stream after repair
        new_position: usize,
        /// The type of recovery strategy that was used
        recovery_type: RecoveryType,
        /// Optional repair operation that was applied
        repair: Option<TokenRepair>,
    },
    /// Recovery failed
    Failed {
        /// Human-readable description of why recovery failed
        reason: String,
    },
}

/// Types of recovery performed
#[derive(Debug, Clone, Copy)]
pub enum RecoveryType {
    /// Simple panic mode recovery - skip to synchronization points
    PanicMode,
    /// Local repair recovery - attempt minimal fixes to syntax errors
    LocalRepair,
    /// Phrase-level recovery - reconstruct malformed grammatical phrases
    PhraseLevel,
    /// Error production recovery - use specialized grammar rules for common errors
    ErrorProduction,
    /// Contextual recovery - use surrounding syntactic context for error repair
    Contextual,
}

/// Token repair operations
#[derive(Debug, Clone)]
pub enum TokenRepair {
    /// Insert a token at position
    Insert {
        /// Position in the token stream where the token should be inserted
        position: usize,
        /// The token to insert at the specified position
        token: Token,
    },
    /// Replace a token
    Replace {
        /// Position of the token to replace in the token stream
        position: usize,
        /// The original token that is being replaced
        old_token: Token,
        /// The new token to put in place of the old token
        new_token: Token,
    },
    /// Delete tokens
    Delete {
        /// Starting position in the token stream for deletion
        start: usize,
        /// Number of consecutive tokens to delete from the start position
        count: usize,
    },
}

/// Context information for recovery
#[derive(Debug, Clone, Copy)]
enum ContextInfo {
    InsideDefinition,
    InsideLambda,
    InsideApplication,
    TopLevel,
    Unknown,
}

/// Simple edit distance calculation for identifier correction
fn edit_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::lexer::Token;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("define", "defin"), 1);
        assert_eq!(edit_distance("lambda", "lamda"), 1);
        assert_eq!(edit_distance("if", "iff"), 1);
        assert_eq!(edit_distance("hello", "world"), 4);
    }

    #[test]
    fn test_recovery_context() {
        let mut context = RecoveryContext::new(RecoveryStrategy::LocalRepair);

        context.push_sync_point(SynchronizationPoint::ExpressionBoundary);
        context.add_expected_tokens(&[TokenKind::LeftParen, TokenKind::Identifier]);

        assert_eq!(context.sync_points.len(), 1);
        assert_eq!(context.expected_tokens.len(), 2);
        assert!(context.can_attempt_repair());
    }

    #[test]
    fn test_error_recovery_engine_creation() {
        let engine = ErrorRecoveryEngine::new(RecoveryStrategy::LocalRepair);
        assert!(!engine.error_productions.is_empty());
        assert!(!engine.repair_rules.is_empty());
    }
}
