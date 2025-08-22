//! Enhanced diagnostic system for macro expansion errors.
//!
//! This module provides sophisticated error reporting, recovery mechanisms,
//! and diagnostic tools for macro expansion, significantly improving the
//! development experience with clear, actionable error messages.

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::macro_system::advanced_hygiene_control::{
    HygieneViolation, ViolationSeverity, ViolationType,
};
use crate::macro_system::type_safe_expansion::MacroType;
use crate::macro_system::{Pattern, PatternBindings, Template};
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Enhanced diagnostic context for macro expansion errors.
#[derive(Debug)]
pub struct MacroDiagnosticContext {
    /// Stack of macro expansion contexts
    expansion_stack: Vec<ExpansionContext>,
    /// Error recovery strategies
    recovery_strategies: Vec<ErrorRecoveryStrategy>,
    /// Diagnostic configuration
    config: DiagnosticConfig,
    /// Accumulated diagnostics
    diagnostics: Vec<MacroDiagnostic>,
    /// Context for error correlation
    error_correlation: ErrorCorrelationContext,
}

/// Context information for macro expansion operations
///
/// Tracks the state and progress of macro expansion to provide detailed
/// diagnostic information when errors occur during the expansion process.
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    /// Macro name being expanded
    pub macro_name: String,
    /// Input expression that triggered expansion
    pub input_expression: Spanned<Expr>,
    /// Pattern being matched
    pub pattern: Option<Pattern>,
    /// Template being expanded
    pub template: Option<Template>,
    /// Current bindings
    pub bindings: Option<PatternBindings>,
    /// Expansion depth
    pub depth: usize,
    /// Source location
    pub source_span: Span,
    /// Timestamp for performance tracking
    pub timestamp: std::time::Instant,
}

/// Comprehensive diagnostic for macro expansion issues.
#[derive(Debug, Clone)]
pub struct MacroDiagnostic {
    /// Diagnostic severity
    pub severity: DiagnosticSeverity,
    /// Primary diagnostic code
    pub code: DiagnosticCode,
    /// Human-readable message
    pub message: String,
    /// Primary source location
    pub primary_span: Span,
    /// Secondary locations with explanations
    pub secondary_spans: Vec<SecondarySpan>,
    /// Suggested fixes
    pub suggestions: Vec<DiagnosticSuggestion>,
    /// Related diagnostics
    pub related_diagnostics: Vec<RelatedDiagnostic>,
    /// Macro expansion context
    pub expansion_context: Option<ExpansionContext>,
    /// Help text
    pub help_text: Option<String>,
    /// Documentation links
    pub documentation_links: Vec<String>,
}

/// Severity levels for diagnostic messages
///
/// Categorizes the importance and impact of diagnostic messages,
/// from informational to critical system failures.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DiagnosticSeverity {
    /// Informational message
    Info,
    /// Warning that doesn't prevent compilation
    Warning,
    /// Error that prevents successful expansion
    Error,
    /// Critical error indicating system failure
    Critical,
    /// Internal compiler error
    InternalError,
}

/// Specific diagnostic codes for different types of macro expansion errors.
///
/// Provides fine-grained categorization of errors to enable precise error
/// reporting and targeted recovery strategies.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DiagnosticCode {
    /// Pattern matching errors
    PatternMatch(PatternMatchError),
    /// Template expansion errors
    TemplateExpansion(TemplateExpansionError),
    /// Type checking errors
    TypeCheck(TypeCheckError),
    /// Hygiene violation errors
    HygieneViolation(HygieneViolationError),
    /// Macro definition errors
    MacroDefinition(MacroDefinitionError),
    /// Syntax errors
    Syntax(SyntaxError),
    /// Internal system errors
    Internal(InternalError),
}

/// Specific errors that can occur during pattern matching.
///
/// Categorizes different failure modes in pattern matching to provide
/// clear diagnostic messages and appropriate recovery suggestions.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum PatternMatchError {
    /// Pattern doesn't match input
    NoMatch,
    /// Ambiguous pattern match
    AmbiguousMatch,
    /// Invalid ellipsis usage
    InvalidEllipsis,
    /// Missing required elements
    MissingElements,
    /// Too many elements provided
    TooManyElements,
    /// Type mismatch in pattern
    TypeMismatch,
    /// Invalid literal pattern
    InvalidLiteral,
}

/// Errors that occur during template expansion phase of macro processing.
///
/// Identifies specific problems with template expansion to enable targeted
/// error reporting and recovery mechanisms.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TemplateExpansionError {
    /// Unbound template variable
    UnboundVariable,
    /// Invalid ellipsis expansion
    InvalidEllipsisExpansion,
    /// Circular template reference
    CircularReference,
    /// Template depth exceeded
    DepthExceeded,
    /// Invalid template syntax
    InvalidSyntax,
    /// Type error in template
    TypeError,
}

/// Type checking errors that occur during macro expansion with type safety.
///
/// Identifies type-related problems in typed macro expansions to provide
/// clear error messages and maintain type system integrity.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TypeCheckError {
    /// Type mismatch
    TypeMismatch,
    /// Unknown type
    UnknownType,
    /// Type inference failed
    InferenceFailed,
    /// Invalid type annotation
    InvalidAnnotation,
    /// Circular type reference
    CircularType,
    /// Type constraint violation
    ConstraintViolation,
}

/// Hygiene violation errors detected during macro expansion.
///
/// Represents specific violations of macro hygiene rules that could
/// lead to incorrect variable bindings or scope violations.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum HygieneViolationError {
    /// Unintended variable capture
    UnintendedCapture,
    /// Reference to unbound identifier
    UnboundReference,
    /// Hygiene inconsistency
    HygieneInconsistency,
    /// Circular macro dependency
    CircularDependency,
}

/// Errors in macro definition syntax or semantics.
///
/// Identifies problems with macro definitions that prevent successful
/// macro registration or expansion capability.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum MacroDefinitionError {
    /// Invalid macro syntax
    InvalidSyntax,
    /// Duplicate macro definition
    DuplicateDefinition,
    /// Missing macro body
    MissingBody,
    /// Invalid pattern structure
    InvalidPattern,
    /// Invalid template structure
    InvalidTemplate,
}

/// Syntax errors encountered during macro parsing and expansion.
///
/// Represents low-level syntax problems that prevent successful parsing
/// of macro definitions or expansions.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SyntaxError {
    /// Unexpected token
    UnexpectedToken,
    /// Missing closing delimiter
    MissingClosingDelimiter,
    /// Invalid identifier
    InvalidIdentifier,
    /// Invalid literal
    InvalidLiteral,
    /// Malformed expression
    MalformedExpression,
}

/// Internal system errors that indicate bugs or resource problems.
///
/// Represents unexpected failures in the macro system that require
/// developer attention and system-level debugging.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum InternalError {
    /// Assertion failure
    AssertionFailure,
    /// Unimplemented feature
    Unimplemented,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Inconsistent state
    InconsistentState,
}

/// Secondary span information for multi-location error reporting.
///
/// Provides additional context for errors that involve multiple source
/// locations, enhancing diagnostic clarity.
#[derive(Debug, Clone)]
pub struct SecondarySpan {
    /// Location of secondary diagnostic
    pub span: Span,
    /// Explanation for this location
    pub message: String,
    /// Style of the secondary diagnostic
    pub style: SecondarySpanStyle,
}

/// Visual styles for secondary diagnostic spans in error reporting.
///
/// Controls how secondary spans are rendered to convey their relationship
/// to the primary error and their informational content.
#[derive(Debug, Clone)]
pub enum SecondarySpanStyle {
    /// Note (informational)
    Note,
    /// Help (suggestion)
    Help,
    /// Related (connected to primary)
    Related,
    /// Context (provides background)
    Context,
}

/// Suggested fix for a diagnostic error with replacement text.
///
/// Provides actionable suggestions for resolving errors, including
/// the specific text changes and applicability assessment.
#[derive(Debug, Clone)]
pub struct DiagnosticSuggestion {
    /// Description of the suggestion
    pub description: String,
    /// Source span to replace
    pub span: Span,
    /// Replacement text
    pub replacement: String,
    /// Applicability of the suggestion
    pub applicability: SuggestionApplicability,
    /// Additional context
    pub context: Option<String>,
}

/// Confidence level for automatic application of diagnostic suggestions.
///
/// Indicates how safe and reliable a diagnostic suggestion is for
/// automatic application or user guidance.
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionApplicability {
    /// Always safe to apply automatically
    Automatic,
    /// Usually safe, but might need user confirmation
    MaybeIncorrect,
    /// Might be incorrect, requires manual review
    HasPlaceholders,
    /// Cannot be applied automatically
    Unspecified,
}

/// A diagnostic related to the primary error for additional context.
///
/// Links related issues to provide comprehensive error reporting
/// and help users understand error relationships.
#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    /// Related diagnostic message
    pub message: String,
    /// Location of related issue
    pub span: Span,
    /// How this relates to the primary diagnostic
    pub relation: DiagnosticRelation,
}

/// Types of relationships between primary and related diagnostics.
///
/// Categorizes how related diagnostics connect to the primary error
/// to provide appropriate context and resolution guidance.
#[derive(Debug, Clone)]
pub enum DiagnosticRelation {
    /// Caused by this issue
    CausedBy,
    /// Causes this issue
    Causes,
    /// Related to this issue
    RelatedTo,
    /// Similar to this issue
    SimilarTo,
    /// Conflicts with this issue
    ConflictsWith,
}

/// Configuration for diagnostic reporting behavior and formatting.
///
/// Controls how diagnostic messages are formatted, filtered, and presented
/// to provide optimal developer experience.
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// Maximum number of diagnostics to report
    pub max_diagnostics: usize,
    /// Include suggestions in diagnostics
    pub include_suggestions: bool,
    /// Include help text
    pub include_help_text: bool,
    /// Include documentation links
    pub include_documentation_links: bool,
    /// Color output (for terminal)
    pub colored_output: bool,
    /// Verbosity level
    pub verbosity: DiagnosticVerbosity,
    /// Error recovery level
    pub recovery_level: ErrorRecoveryLevel,
}

/// Verbosity levels for diagnostic output to control information detail.
///
/// Controls the amount of information included in diagnostic messages
/// to match user preferences and development contexts.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticVerbosity {
    /// Minimal information
    Quiet,
    /// Normal information
    Normal,
    /// Detailed information
    Verbose,
    /// Debug-level information
    Debug,
}

/// Levels of error recovery aggressiveness during macro expansion.
///
/// Controls how much effort the system puts into recovering from errors
/// and continuing macro expansion despite failures.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorRecoveryLevel {
    /// No error recovery
    None,
    /// Basic error recovery
    Basic,
    /// Advanced error recovery
    Advanced,
    /// Aggressive error recovery (may produce incorrect results)
    Aggressive,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            max_diagnostics: 100,
            include_suggestions: true,
            include_help_text: true,
            include_documentation_links: true,
            colored_output: true,
            verbosity: DiagnosticVerbosity::Normal,
            recovery_level: ErrorRecoveryLevel::Basic,
        }
    }
}

/// Error recovery strategy for handling macro expansion failures.
#[derive(Debug, Clone)]
pub struct ErrorRecoveryStrategy {
    /// Strategy name
    pub name: String,
    /// When to apply this strategy
    pub trigger: RecoveryTrigger,
    /// How to recover from the error
    pub action: RecoveryAction,
    /// Priority of this strategy (higher = more preferred)
    pub priority: i32,
    /// Whether this strategy is enabled
    pub enabled: bool,
}

/// Conditions that trigger the application of error recovery strategies.
///
/// Defines when specific recovery strategies should be applied during
/// macro expansion error handling.
#[derive(Debug, Clone)]
pub enum RecoveryTrigger {
    /// Apply for any error
    AnyError,
    /// Apply for specific diagnostic codes
    DiagnosticCode(DiagnosticCode),
    /// Apply when pattern matching fails
    PatternMatchFailure,
    /// Apply when template expansion fails
    TemplateExpansionFailure,
    /// Apply for type errors
    TypeError,
    /// Apply for hygiene violations
    HygieneViolation,
    /// Custom trigger condition
    Custom(String),
}

/// Actions that can be taken to recover from macro expansion errors.
///
/// Defines specific strategies for handling errors and continuing
/// macro expansion when problems are encountered.
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Skip the problematic expression
    Skip,
    /// Use original expression unchanged
    UseOriginal,
    /// Try alternative expansion
    TryAlternative,
    /// Apply partial expansion
    PartialExpansion,
    /// Insert placeholder
    InsertPlaceholder(String),
    /// Custom recovery function
    Custom(String),
}

/// Context for correlating related errors.
#[derive(Debug, Default)]
pub struct ErrorCorrelationContext {
    /// Error clusters (related errors)
    error_clusters: Vec<ErrorCluster>,
    /// Correlation statistics
    statistics: CorrelationStatistics,
}

/// A cluster of related errors for correlation analysis.
///
/// Groups related diagnostic errors to identify common root causes
/// and provide better error reporting.
#[derive(Debug, Clone)]
pub struct ErrorCluster {
    /// Cluster identifier
    pub id: String,
    /// Errors in this cluster
    pub errors: Vec<usize>, // indices into diagnostics
    /// Root cause (if identified)
    pub root_cause: Option<usize>,
    /// Cluster confidence score
    pub confidence: f64,
}

/// Statistics about error correlation analysis effectiveness.
///
/// Tracks the performance and accuracy of error correlation to improve
/// diagnostic clustering algorithms.
#[derive(Debug, Default)]
pub struct CorrelationStatistics {
    /// Number of error clusters found
    pub clusters_found: usize,
    /// Number of uncorrelated errors
    pub uncorrelated_errors: usize,
    /// Average cluster size
    pub average_cluster_size: f64,
    /// Correlation accuracy
    pub correlation_accuracy: f64,
}

impl MacroDiagnosticContext {
    /// Creates a new diagnostic context.
    pub fn new() -> Self {
        Self::with_config(DiagnosticConfig::default())
    }

    /// Creates a new diagnostic context with custom configuration.
    pub fn with_config(config: DiagnosticConfig) -> Self {
        Self {
            expansion_stack: Vec::new(),
            recovery_strategies: Self::create_default_recovery_strategies(),
            config,
            diagnostics: Vec::new(),
            error_correlation: ErrorCorrelationContext::default(),
        }
    }

    /// Enters a new macro expansion context.
    pub fn enter_expansion(&mut self, macro_name: String, input: Spanned<Expr>, span: Span) {
        let context = ExpansionContext {
            macro_name,
            input_expression: input,
            pattern: None,
            template: None,
            bindings: None,
            depth: self.expansion_stack.len(),
            source_span: span,
            timestamp: std::time::Instant::now(),
        };

        self.expansion_stack.push(context);
    }

    /// Exits the current macro expansion context.
    pub fn exit_expansion(&mut self) {
        self.expansion_stack.pop();
    }

    /// Reports a diagnostic with full context.
    pub fn report_diagnostic(&mut self, diagnostic: MacroDiagnostic) {
        if self.diagnostics.len() >= self.config.max_diagnostics {
            return;
        }

        // Add expansion context if not already present
        let mut diagnostic = diagnostic;
        if diagnostic.expansion_context.is_none() {
            diagnostic.expansion_context = self.expansion_stack.last().cloned();
        }

        self.diagnostics.push(diagnostic);
    }

    /// Creates a diagnostic for pattern matching failure.
    pub fn create_pattern_match_diagnostic(
        &self,
        error: PatternMatchError,
        pattern: &Pattern,
        input: &Spanned<Expr>,
        message: String,
    ) -> MacroDiagnostic {
        let mut diagnostic = MacroDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::PatternMatch(error.clone()),
            message,
            primary_span: input.span,
            secondary_spans: Vec::new(),
            suggestions: Vec::new(),
            related_diagnostics: Vec::new(),
            expansion_context: self.expansion_stack.last().cloned(),
            help_text: None,
            documentation_links: Vec::new(),
        };

        // Add context-specific help
        match error {
            PatternMatchError::NoMatch => {
                diagnostic.help_text = Some(
                    "The pattern doesn't match the input expression. \
                     Check the structure and types of both pattern and input."
                        .to_string(),
                );
                diagnostic.suggestions.push(DiagnosticSuggestion {
                    description: "Consider using a more general pattern".to_string(),
                    span: input.span,
                    replacement: "_".to_string(),
                    applicability: SuggestionApplicability::HasPlaceholders,
                    context: Some("Use wildcard pattern to match anything".to_string()),
                });
            }
            PatternMatchError::TooManyElements => {
                diagnostic.help_text = Some(
                    "The input has more elements than the pattern expects. \
                     Consider using an ellipsis pattern (...) to match variable-length lists."
                        .to_string(),
                );
            }
            PatternMatchError::MissingElements => {
                diagnostic.help_text = Some(
                    "The input has fewer elements than the pattern requires. \
                     Check if all required elements are present."
                        .to_string(),
                );
            }
            _ => {}
        }

        if self.config.include_documentation_links {
            diagnostic
                .documentation_links
                .push("https://lambdust-docs.org/macros/pattern-matching".to_string());
        }

        diagnostic
    }

    /// Creates a diagnostic for template expansion failure.
    pub fn create_template_expansion_diagnostic(
        &self,
        error: TemplateExpansionError,
        template: &Template,
        bindings: &PatternBindings,
        span: Span,
        message: String,
    ) -> MacroDiagnostic {
        let mut diagnostic = MacroDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::TemplateExpansion(error.clone()),
            message,
            primary_span: span,
            secondary_spans: Vec::new(),
            suggestions: Vec::new(),
            related_diagnostics: Vec::new(),
            expansion_context: self.expansion_stack.last().cloned(),
            help_text: None,
            documentation_links: Vec::new(),
        };

        // Add context-specific help
        match error {
            TemplateExpansionError::UnboundVariable => {
                diagnostic.help_text = Some(
                    "A template variable is not bound by the pattern. \
                     Ensure all template variables are captured in the pattern."
                        .to_string(),
                );
            }
            TemplateExpansionError::InvalidEllipsisExpansion => {
                diagnostic.help_text = Some(
                    "Invalid ellipsis usage in template expansion. \
                     Ellipsis variables must correspond to ellipsis patterns."
                        .to_string(),
                );
            }
            TemplateExpansionError::CircularReference => {
                diagnostic.help_text = Some(
                    "Circular reference detected in template expansion. \
                     This would cause infinite recursion."
                        .to_string(),
                );
            }
            _ => {}
        }

        if self.config.include_documentation_links {
            diagnostic
                .documentation_links
                .push("https://lambdust-docs.org/macros/template-expansion".to_string());
        }

        diagnostic
    }

    /// Creates a diagnostic from a hygiene violation.
    pub fn create_hygiene_diagnostic(&self, violation: &HygieneViolation) -> MacroDiagnostic {
        let severity = match violation.severity {
            ViolationSeverity::Info => DiagnosticSeverity::Info,
            ViolationSeverity::Warning => DiagnosticSeverity::Warning,
            ViolationSeverity::Error => DiagnosticSeverity::Error,
            ViolationSeverity::Critical => DiagnosticSeverity::Critical,
        };

        let code = match &violation.violation_type {
            ViolationType::UnintendedCapture { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::UnintendedCapture)
            }
            ViolationType::UnboundReference { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::UnboundReference)
            }
            ViolationType::MacroConflict { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::HygieneInconsistency)
            }
            ViolationType::HygieneInconsistency { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::HygieneInconsistency)
            }
            ViolationType::CircularDependency { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::CircularDependency)
            }
            ViolationType::TemplateVariableScope { .. } => {
                DiagnosticCode::HygieneViolation(HygieneViolationError::HygieneInconsistency)
            }
        };

        MacroDiagnostic {
            severity,
            code,
            message: violation.description.clone(),
            primary_span: violation.primary_span,
            secondary_spans: violation
                .related_spans
                .iter()
                .map(|(span, msg)| SecondarySpan {
                    span: *span,
                    message: msg.clone(),
                    style: SecondarySpanStyle::Related,
                })
                .collect(),
            suggestions: if let Some(fix) = &violation.suggested_fix {
                vec![DiagnosticSuggestion {
                    description: "Apply suggested fix".to_string(),
                    span: violation.primary_span,
                    replacement: fix.clone(),
                    applicability: SuggestionApplicability::MaybeIncorrect,
                    context: None,
                }]
            } else {
                Vec::new()
            },
            related_diagnostics: Vec::new(),
            expansion_context: self.expansion_stack.last().cloned(),
            help_text: Some(self.get_hygiene_help_text(&violation.violation_type)),
            documentation_links: if self.config.include_documentation_links {
                vec!["https://lambdust-docs.org/macros/hygiene".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    fn get_hygiene_help_text(&self, violation_type: &ViolationType) -> String {
        match violation_type {
            ViolationType::UnintendedCapture { .. } => {
                "Unintended capture occurs when a macro accidentally binds to a variable \
                 in the usage context. Use hygienic macro expansion to avoid this."
                    .to_string()
            }
            ViolationType::UnboundReference { .. } => {
                "This identifier is referenced but not bound in any accessible scope. \
                 Check the spelling and scope of the identifier."
                    .to_string()
            }
            ViolationType::HygieneInconsistency { .. } => {
                "Hygiene renaming is inconsistent. This may indicate an issue with \
                 the macro expansion system."
                    .to_string()
            }
            ViolationType::CircularDependency { .. } => {
                "Circular dependency detected in macro definitions. \
                 This creates infinite expansion loops."
                    .to_string()
            }
            _ => "Hygiene violation detected. Check macro expansion for naming conflicts."
                .to_string(),
        }
    }

    /// Attempts error recovery for failed macro expansion.
    pub fn attempt_recovery(
        &mut self,
        diagnostic: &MacroDiagnostic,
        original_expr: &Spanned<Expr>,
    ) -> Option<Spanned<Expr>> {
        if self.config.recovery_level == ErrorRecoveryLevel::None {
            return None;
        }

        // Find applicable recovery strategies
        let applicable_strategies: Vec<_> = self
            .recovery_strategies
            .iter()
            .filter(|strategy| strategy.enabled && self.strategy_applies(strategy, diagnostic))
            .collect();

        if applicable_strategies.is_empty() {
            return None;
        }

        // Sort by priority and try each strategy
        let mut sorted_strategies = applicable_strategies;
        sorted_strategies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for strategy in sorted_strategies {
            if let Some(recovered) =
                self.apply_recovery_strategy(strategy, original_expr, diagnostic)
            {
                return Some(recovered);
            }
        }

        None
    }

    fn strategy_applies(
        &self,
        strategy: &ErrorRecoveryStrategy,
        diagnostic: &MacroDiagnostic,
    ) -> bool {
        match &strategy.trigger {
            RecoveryTrigger::AnyError => true,
            RecoveryTrigger::DiagnosticCode(code) => &diagnostic.code == code,
            RecoveryTrigger::PatternMatchFailure => {
                matches!(diagnostic.code, DiagnosticCode::PatternMatch(_))
            }
            RecoveryTrigger::TemplateExpansionFailure => {
                matches!(diagnostic.code, DiagnosticCode::TemplateExpansion(_))
            }
            RecoveryTrigger::TypeError => {
                matches!(diagnostic.code, DiagnosticCode::TypeCheck(_))
            }
            RecoveryTrigger::HygieneViolation => {
                matches!(diagnostic.code, DiagnosticCode::HygieneViolation(_))
            }
            RecoveryTrigger::Custom(_) => {
                // TODO: Implement custom trigger evaluation
                false
            }
        }
    }

    fn apply_recovery_strategy(
        &self,
        strategy: &ErrorRecoveryStrategy,
        original_expr: &Spanned<Expr>,
        diagnostic: &MacroDiagnostic,
    ) -> Option<Spanned<Expr>> {
        match &strategy.action {
            RecoveryAction::UseOriginal => Some(original_expr.clone()),
            RecoveryAction::Skip => {
                // Return an empty begin form
                Some(Spanned::new(Expr::Begin(vec![]), original_expr.span))
            }
            RecoveryAction::InsertPlaceholder(placeholder) => Some(Spanned::new(
                Expr::Identifier(placeholder.clone()),
                original_expr.span,
            )),
            _ => {
                // TODO: Implement other recovery actions
                None
            }
        }
    }

    fn create_default_recovery_strategies() -> Vec<ErrorRecoveryStrategy> {
        vec![
            ErrorRecoveryStrategy {
                name: "use-original".to_string(),
                trigger: RecoveryTrigger::PatternMatchFailure,
                action: RecoveryAction::UseOriginal,
                priority: 10,
                enabled: true,
            },
            ErrorRecoveryStrategy {
                name: "skip-expression".to_string(),
                trigger: RecoveryTrigger::AnyError,
                action: RecoveryAction::Skip,
                priority: 5,
                enabled: false, // Only enable in aggressive mode
            },
        ]
    }

    /// Gets all accumulated diagnostics.
    pub fn get_diagnostics(&self) -> &[MacroDiagnostic] {
        &self.diagnostics
    }

    /// Gets diagnostics of a specific severity or higher.
    pub fn get_diagnostics_by_severity(
        &self,
        min_severity: DiagnosticSeverity,
    ) -> Vec<&MacroDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity >= min_severity)
            .collect()
    }

    /// Clears all accumulated diagnostics.
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Gets the current expansion stack depth.
    pub fn expansion_depth(&self) -> usize {
        self.expansion_stack.len()
    }
}

impl Default for MacroDiagnosticContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticCode::PatternMatch(e) => write!(f, "E001: Pattern match error: {e:?}"),
            DiagnosticCode::TemplateExpansion(e) => {
                write!(f, "E002: Template expansion error: {e:?}")
            }
            DiagnosticCode::TypeCheck(e) => write!(f, "E003: Type check error: {e:?}"),
            DiagnosticCode::HygieneViolation(e) => write!(f, "E004: Hygiene violation: {e:?}"),
            DiagnosticCode::MacroDefinition(e) => {
                write!(f, "E005: Macro definition error: {e:?}")
            }
            DiagnosticCode::Syntax(e) => write!(f, "E006: Syntax error: {e:?}"),
            DiagnosticCode::Internal(e) => write!(f, "E999: Internal error: {e:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_diagnostic_context_creation() {
        let context = MacroDiagnosticContext::new();
        assert_eq!(context.expansion_depth(), 0);
        assert!(context.get_diagnostics().is_empty());
    }

    #[test]
    fn test_expansion_stack() {
        let mut context = MacroDiagnosticContext::new();

        context.enter_expansion(
            "test-macro".to_string(),
            Spanned::new(Expr::Literal(Literal::Number(42.0)), Span::new(0, 1)),
            Span::new(0, 1),
        );

        assert_eq!(context.expansion_depth(), 1);

        context.exit_expansion();
        assert_eq!(context.expansion_depth(), 0);
    }

    #[test]
    fn test_diagnostic_severity_ordering() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Critical);
    }

    #[test]
    fn test_diagnostic_code_display() {
        let code = DiagnosticCode::PatternMatch(PatternMatchError::NoMatch);
        let display = format!("{}", code);
        assert!(display.starts_with("E001"));
    }

    #[test]
    fn test_recovery_strategy_creation() {
        let strategies = MacroDiagnosticContext::create_default_recovery_strategies();
        assert!(!strategies.is_empty());
        assert!(strategies.iter().any(|s| s.name == "use-original"));
    }
}
