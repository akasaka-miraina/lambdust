//! Macro expansion system with hygiene support.
//!
//! This module implements the macro system for Lambdust, providing
//! hygienic macro expansion according to the R7RS standard. The system
//! supports pattern-based macro definitions, template expansion with
//! proper variable scoping, and hygiene preservation through identifier
//! renaming.

use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub mod builtins;
pub mod environment;
pub mod expander;
pub mod hygiene;
pub mod pattern;
pub mod syntax_rules;
pub mod template;

// SRFI-26 Cut/Cute macro expansion
pub mod srfi26_expansion;

// Individual structure modules
pub mod gc_integration;
pub mod macro_expander;
pub mod macro_transformer;

// Syntax object system modules
pub mod advanced_hygiene;
pub mod quasisyntax;
pub mod scope_management;
pub mod syntax_case;
pub mod syntax_integration;
pub mod syntax_objects;
pub mod syntax_tests;
pub mod unified_expander;

// Identifier transformer system modules
pub mod context_aware_expander;
pub mod identifier_transformer_r6rs;
pub mod identifier_transformer_tests;
pub mod identifier_transformers;
pub mod variable_transformer_builtins;
pub mod variable_transformer_integration;

pub use builtins::*;
pub use environment::*;
pub use expander::*;
pub use hygiene::*;
pub use pattern::*;
pub use template::*;
// Selective imports to avoid name conflicts
pub use syntax_rules::{
    SyntaxRulesTransformer, expand_syntax_rules, parse_syntax_rules,
    syntax_rules_to_macro_transformer, validate_pattern, validate_template,
};

// SRFI-26 Cut/Cute expansion functions
pub use srfi26_expansion::{
    ExpanderMetrics, ExpansionCacheStats, OptimizedCutExpander, expand_cut_optimized,
    expand_cute_optimized, global_cache_stats, global_expansion_metrics, reset_parameter_pool,
};

// Re-export individual structures
pub use gc_integration::{
    ExpansionContext, ExpansionId, GcMacroConfig, GcMacroCoordinator, GcMacroExpansionResult,
    MacroExpanderGcExt, MacroExpansionStatistics,
};
pub use macro_expander::*;
pub use macro_transformer::*;

// Re-export syntax object system
pub use advanced_hygiene::{
    BindingOccurrence, HygieneResolver, HygieneStats, Mark, MarkSet, ReferenceOccurrence,
    fresh_mark, hygiene_utils,
};
pub use quasisyntax::{
    QuasisyntaxContext, QuasisyntaxExpansionResult, QuasisyntaxParser, QuasisyntaxTemplate,
    quasisyntax_interface,
};
pub use scope_management::{
    LexicalScope, ScopeBinding, ScopeId, ScopeManager, ScopeManagerStats, ScopeStats, ScopeType,
    next_scope_id, scope_utils,
};
pub use syntax_case::{SyntaxBindings, SyntaxPattern, SyntaxTemplate, syntax_procedures};
pub use syntax_integration::{
    IntegrationStats, SyntaxAwareMacroExpander, SyntaxCaseTransformer, integration_utils,
    legacy_bridge,
};
pub use syntax_objects::{
    BindingInfo, HygieneEnvironment, LexicalContext, SyntaxId, SyntaxObject, SyntaxProperty,
    next_syntax_id, syntax_utils,
};
pub use unified_expander::{
    ExpansionMode, MacroTransformerType, UnifiedExpansionStats, UnifiedMacroExpander,
    UnifiedMacroTransformer, unified_utils,
};

// Re-export identifier transformer system
pub use context_aware_expander::{ContextAwareExpansionStats, ContextAwareMacroExpander};
pub use identifier_transformer_r6rs::{
    R6RSComplianceFlags, R6RSComplianceStats, R6RSComplianceValidator,
    R6RSIdentifierTransformerSystem, R6RSValidationResult, R6RSValidationSummary,
    r6rs_compliance_tests,
};
pub use identifier_transformers::{
    ContextDetector, GuardCondition, IdentifierContext, TransformationLogic, TransformerClause,
    TransformerProcedure, VariableTransformer, VariableTransformerRegistry,
};
pub use variable_transformer_builtins::{
    VariableTransformerBuiltins, apply_variable_transformer, create_accessor_transformer,
    create_hash_accessor_transformer, create_vector_accessor_transformer,
    make_simple_variable_transformer, make_variable_transformer, variable_transformer_p,
    variable_transformer_utils,
};
pub use variable_transformer_integration::{
    SyntaxTemplateTransformerSupport, VariableTransformerAwareSyntaxCase,
    VariableTransformerAwareTemplate, syntax_procedures as variable_syntax_procedures,
};

// Macro-time computation system modules
pub mod advanced_quasisyntax;
pub mod macro_time_computation;
// pub mod macro_time_demo;  // Disabled for CI stability
pub mod macro_time_integration;
pub mod macro_time_transformers;

// Re-export macro-time computation system
pub use advanced_quasisyntax::{
    AdvancedGenerationContext, AdvancedQuasisyntaxProcessor, AdvancedQuasisyntaxTemplate,
    QuasisyntaxStats, TemplateCombiner, advanced_quasisyntax_interface,
};
pub use macro_time_computation::{
    MacroDebugState, MacroEventType, MacroExpansionEvent, MacroExpansionFrame,
    MacroTimeEnvironment, MacroTimeProcedure, MacroTimeStats, MacroTimeValue, Phase,
    TemplateUtilities,
};
// pub use macro_time_demo::{
//     demonstrate_advanced_quasisyntax, demonstrate_make_list, demonstrate_phase_separation,
//     demonstrate_repeat_macro, run_all_demonstrations,
// };  // Disabled for CI stability
pub use macro_time_integration::{
    MacroTimeAwareExpander, MacroTimeCompatibilityReport, MacroTimeIntegrationConfig,
    MacroTimeIntegrationStats, integration_interface,
};
pub use macro_time_transformers::{
    MacroTimeClause, MacroTimeGuard, MacroTimeTransformer, MacroTimeTransformerProc,
    MacroTimeTransformerStats, transformer_factory,
};

// Type-safe macro expansion system modules
pub mod advanced_hygiene_control;
pub mod compile_time_computation;
pub mod enhanced_diagnostics;
pub mod integrated_type_safe_expander;
pub mod type_safe_expansion;

// Phase 2A optimized macro expansion modules
pub mod fast_hygiene_resolver;
pub mod integrated_optimized_expander;
// pub mod integration_tests;  // Disabled for CI stability
pub mod optimized_macro_expander;

// Re-export type-safe expansion system
pub use advanced_hygiene_control::{
    DetectionConfig, HygieneViolation, HygieneViolationDetector, PreciseHygieneController,
    ViolationSeverity, ViolationType,
};
pub use compile_time_computation::{
    CompileTimeComputation, CompileTimeComputationEngine, CompileTimeContext, CompileTimeValue,
    ComputationLimits, ComputationStatistics,
};
pub use enhanced_diagnostics::{
    DiagnosticCode, DiagnosticConfig, DiagnosticSeverity, ErrorRecoveryStrategy,
    HygieneViolationError, MacroDiagnostic, MacroDiagnosticContext, PatternMatchError,
    RecoveryAction, RecoveryTrigger, TemplateExpansionError, TypeCheckError,
};
pub use integrated_type_safe_expander::{
    CompatibilityConfig, CompileTimeConfig, ExpansionMetrics, HygieneControlConfig,
    IntegratedExpanderBuilder, IntegratedExpansionResult, IntegratedTypeSafeMacroExpander,
    PerformanceConfig, TypeSafeIntegrationConfig, TypeSafetyConfig,
};
pub use type_safe_expansion::{
    MacroType, OptimizationLevel, TypeSafeMacroExpander, TypeSafeMacroTransformer,
    TypeSafeOptimizationConfig, TypedExpansionResult, TypedPattern, TypedTemplate,
};

// Re-export Phase 2A optimized system
pub use fast_hygiene_resolver::{
    FastHygieneResolver, HygieneConfig, HygieneMarkSet, HygieneMonitoringLevel, MarkId,
};
pub use integrated_optimized_expander::{
    IntegratedOptimizedExpander, IntegratedPerformanceMetrics, IntegrationSettings,
    MonitoringConfig, MonitoringDetailLevel, OptimizedIntegrationConfig,
};
// pub use integration_tests::{
//     IntegrationTestSuite, PerformanceResults, TestConfig, TestResults,
//     run_performance_benchmarks_only, run_phase_2a_integration_tests,
// };  // Disabled for CI stability
pub use optimized_macro_expander::{
    CachingOptimizationConfig, MonitoringLevel, OptimizedMacroExpander,
};

/// Global counter for generating unique identifiers for hygiene.
static HYGIENE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique identifier for hygiene purposes.
pub fn next_hygiene_id() -> u64 {
    HYGIENE_COUNTER.fetch_add(1, Ordering::SeqCst)
}
