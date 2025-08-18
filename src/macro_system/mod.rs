//! Macro expansion system with hygiene support.
//!
//! This module implements the macro system for Lambdust, providing
//! hygienic macro expansion according to the R7RS standard. The system
//! supports pattern-based macro definitions, template expansion with
//! proper variable scoping, and hygiene preservation through identifier
//! renaming.

use crate::ast::{Expr};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub mod pattern;
pub mod template;
pub mod hygiene;
pub mod environment;
pub mod expander;
pub mod builtins;
pub mod syntax_rules;

// Individual structure modules
pub mod macro_transformer;
pub mod macro_expander;
pub mod gc_integration;

// Syntax object system modules
pub mod syntax_objects;
pub mod advanced_hygiene;
pub mod syntax_case;
pub mod quasisyntax;
pub mod syntax_integration;
pub mod scope_management;
pub mod syntax_tests;
pub mod unified_expander;

// Identifier transformer system modules
pub mod identifier_transformers;
pub mod variable_transformer_builtins;
pub mod context_aware_expander;
pub mod variable_transformer_integration;
pub mod identifier_transformer_tests;
pub mod identifier_transformer_r6rs;

pub use pattern::*;
pub use template::*;
pub use hygiene::*;
pub use environment::*;
pub use expander::*;
pub use builtins::*;
// Selective imports to avoid name conflicts
pub use syntax_rules::{
    SyntaxRulesTransformer, parse_syntax_rules, expand_syntax_rules,
    validate_pattern, validate_template, syntax_rules_to_macro_transformer
};

// Re-export individual structures
pub use macro_transformer::*;
pub use macro_expander::*;
pub use gc_integration::{
    GcMacroCoordinator, GcMacroConfig, ExpansionId, ExpansionContext,
    GcMacroExpansionResult, MacroExpansionStatistics, MacroExpanderGcExt
};

// Re-export syntax object system
pub use syntax_objects::{
    SyntaxObject, LexicalContext, BindingInfo, SyntaxId, SyntaxProperty,
    HygieneEnvironment, syntax_utils, next_syntax_id
};
pub use advanced_hygiene::{
    HygieneResolver, Mark, MarkSet, BindingOccurrence, ReferenceOccurrence,
    HygieneStats, fresh_mark, hygiene_utils
};
pub use syntax_case::{
    SyntaxPattern, SyntaxTemplate, SyntaxBindings, syntax_procedures
};
pub use quasisyntax::{
    QuasisyntaxTemplate, QuasisyntaxContext, QuasisyntaxExpansionResult,
    QuasisyntaxParser, quasisyntax_interface
};
pub use syntax_integration::{
    SyntaxAwareMacroExpander, IntegrationStats, SyntaxCaseTransformer,
    legacy_bridge, integration_utils
};
pub use scope_management::{
    ScopeManager, LexicalScope, ScopeBinding, ScopeType, ScopeId, ScopeStats,
    ScopeManagerStats, next_scope_id, scope_utils
};
pub use unified_expander::{
    UnifiedMacroExpander, UnifiedMacroTransformer, MacroTransformerType,
    ExpansionMode, UnifiedExpansionStats, unified_utils
};

// Re-export identifier transformer system
pub use identifier_transformers::{
    VariableTransformer, VariableTransformerRegistry, IdentifierContext, ContextDetector,
    TransformerProcedure, TransformationLogic, TransformerClause, GuardCondition
};
pub use variable_transformer_builtins::{
    VariableTransformerBuiltins, make_variable_transformer, make_simple_variable_transformer,
    create_accessor_transformer, create_vector_accessor_transformer, create_hash_accessor_transformer,
    variable_transformer_p, apply_variable_transformer, variable_transformer_utils
};
pub use context_aware_expander::{
    ContextAwareMacroExpander, ContextAwareExpansionStats
};
pub use variable_transformer_integration::{
    VariableTransformerAwareSyntaxCase, VariableTransformerAwareTemplate,
    SyntaxTemplateTransformerSupport, syntax_procedures as variable_syntax_procedures
};
pub use identifier_transformer_r6rs::{
    R6RSIdentifierTransformerSystem, R6RSComplianceFlags, R6RSComplianceStats,
    R6RSComplianceValidator, R6RSValidationResult, R6RSValidationSummary,
    r6rs_compliance_tests
};

// Macro-time computation system modules
pub mod macro_time_computation;
pub mod advanced_quasisyntax;
pub mod macro_time_transformers;
pub mod macro_time_integration;
pub mod macro_time_demo;

// Re-export macro-time computation system
pub use macro_time_computation::{
    MacroTimeEnvironment, MacroTimeValue, Phase, MacroTimeProcedure,
    MacroDebugState, MacroExpansionFrame, MacroExpansionEvent, MacroEventType,
    MacroTimeStats, TemplateUtilities
};
pub use advanced_quasisyntax::{
    AdvancedQuasisyntaxProcessor, AdvancedQuasisyntaxTemplate, AdvancedGenerationContext,
    TemplateCombiner, QuasisyntaxStats, advanced_quasisyntax_interface
};
pub use macro_time_transformers::{
    MacroTimeTransformer, MacroTimeTransformerProc, MacroTimeClause, MacroTimeGuard,
    MacroTimeTransformerStats, transformer_factory
};
pub use macro_time_integration::{
    MacroTimeAwareExpander, MacroTimeIntegrationStats, MacroTimeIntegrationConfig,
    MacroTimeCompatibilityReport, integration_interface
};
pub use macro_time_demo::{
    run_all_demonstrations, demonstrate_repeat_macro, demonstrate_make_list,
    demonstrate_advanced_quasisyntax, demonstrate_phase_separation
};

/// Global counter for generating unique identifiers for hygiene.
static HYGIENE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique identifier for hygiene purposes.
pub fn next_hygiene_id() -> u64 {
    HYGIENE_COUNTER.fetch_add(1, Ordering::SeqCst)
}