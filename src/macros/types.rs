//! Core type definitions for the macro system

use crate::ast::Expr;
use crate::error::Result;
use std::collections::HashMap;

/// Macro transformer type
pub type MacroTransformer = fn(&[Expr]) -> Result<Expr>;

/// Macro definition
#[derive(Debug, Clone)]
pub enum Macro {
    /// Built-in macro with transformer function
    Builtin {
        /// Name of the builtin macro
        name: String,
        /// Transformer function for the macro
        transformer: MacroTransformer,
    },
    /// Syntax-rules macro with pattern/template rules
    SyntaxRules {
        /// Name of the syntax-rules macro
        name: String,
        /// Transformer implementing syntax-rules pattern matching
        transformer: super::syntax_rules::SyntaxRulesTransformer,
    },
    /// Hygienic syntax-rules macro with symbol collision prevention
    HygienicSyntaxRules {
        /// Name of the hygienic macro
        name: String,
        /// Hygienic transformer with symbol renaming
        transformer: super::hygiene::HygienicSyntaxRulesTransformer,
    },
    /// Syntax-case macro with advanced pattern matching
    SyntaxCase {
        /// Name of the syntax-case macro
        name: String,
        /// Syntax-case transformer with guard conditions
        transformer: super::syntax_case::SyntaxCaseTransformer,
    },
}

/// Variable bindings from pattern matching
pub type VariableBindings = HashMap<String, BindingValue>;

/// Value bound to a pattern variable
#[derive(Debug, Clone, PartialEq)]
pub enum BindingValue {
    /// Single expression binding
    Single(Expr),
    /// Multiple expressions (from ellipsis)
    Multiple(Vec<Expr>),
    /// Nested bindings (from nested ellipsis)
    Nested(Vec<BindingValue>),
}
