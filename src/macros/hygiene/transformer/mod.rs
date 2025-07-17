//! Hygienic Transformer Module
//!
//! This module provides a comprehensive implementation of the hygienic macro transformer.
//! It includes pattern matching, template expansion, optimization, and SRFI 46 support.
//!
//! ## Module Structure
//!
//! - `core_types`: Basic type definitions (`OptimizationLevel`, `TransformerMetrics`, `PatternBindings`)
//! - `transformer`: Main `HygienicSyntaxRulesTransformer` implementation
//! - `pattern_matching`: Pattern matching related logic
//! - `template_expansion`: Template expansion related logic

pub mod core_types;
pub mod transformer;
pub mod pattern_matching;
pub mod template_expansion;

// Re-export main types for backward compatibility
pub use core_types::{
    OptimizationLevel, PatternBindings, TransformerMetrics,
};

pub use transformer::HygienicSyntaxRulesTransformer;

pub use pattern_matching::PatternMatcher;

pub use template_expansion::TemplateExpander;

/// Create a new hygienic transformer with default configuration
pub fn create_hygienic_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::new(literals, rules, definition_environment, macro_name)
}

/// Create a production-optimized hygienic transformer
pub fn create_optimized_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::optimized(literals, rules, definition_environment, macro_name)
}

/// Create a scope-aware hygienic transformer
pub fn create_scope_aware_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::scope_aware(literals, rules, definition_environment, macro_name)
}

/// Create a SRFI 46 enabled transformer
pub fn create_srfi46_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::with_srfi46(literals, rules, definition_environment, macro_name)
}

/// Create a legacy transformer (SRFI 46 disabled)
pub fn create_legacy_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::legacy(literals, rules, definition_environment, macro_name)
}
