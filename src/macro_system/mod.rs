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

/// Global counter for generating unique identifiers for hygiene.
static HYGIENE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique identifier for hygiene purposes.
pub fn next_hygiene_id() -> u64 {
    HYGIENE_COUNTER.fetch_add(1, Ordering::SeqCst)
}