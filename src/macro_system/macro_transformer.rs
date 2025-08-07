//! Macro transformer for pattern-based macro expansion.

use crate::diagnostics::Span;
use crate::eval::Environment;
use super::{Pattern, Template};
use std::rc::Rc;

/// A macro transformer that defines how to expand a macro.
#[derive(Debug, Clone)]
pub struct MacroTransformer {
    /// The pattern that the macro matches against
    pub pattern: Pattern,
    /// The template that defines the expansion
    pub template: Template,
    /// The lexical environment where the macro was defined
    pub definition_env: Rc<Environment>,
    /// The name of the macro (for debugging)
    pub name: Option<String>,
    /// Source location for error reporting
    pub source: Option<Span>,
}