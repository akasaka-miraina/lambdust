//! Macro system implementation for Scheme

// Module declarations
pub mod builtin;
pub mod expander;
pub mod pattern_matching;
pub mod syntax_rules;
pub mod types;

// Re-export public types for convenience
pub use expander::MacroExpander;
pub use pattern_matching::{Pattern, Template, SyntaxRule};
pub use syntax_rules::SyntaxRulesTransformer;
pub use types::{BindingValue, Macro, MacroTransformer, VariableBindings};

use crate::ast::Expr;
use crate::error::{LambdustError, Result};

// Import builtin macro functions
use builtin::*;

/// Public helper function for expanding macros by name
pub fn expand_macro(name: &str, args: &[Expr]) -> Result<Expr> {
    let _expander = MacroExpander::new();
    match name {
        "let" => expand_let(args),
        "let*" => expand_let_star(args),
        "letrec" => expand_letrec(args),
        "case" => expand_case(args),
        "when" => expand_when(args),
        "unless" => expand_unless(args),
        _ => Err(LambdustError::syntax_error(format!(
            "Unknown macro: {name}"
        ))),
    }
}


