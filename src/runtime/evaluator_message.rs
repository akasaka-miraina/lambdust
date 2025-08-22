//! Messages sent to evaluator threads.

use crate::ast::Expr;
use crate::diagnostics::{Result, Span};
use crate::eval::Value;
use crate::module_system::ImportSpec;
use std::collections::HashMap;

/// Messages sent to evaluator threads.
#[derive(Debug)]
pub enum EvaluatorMessage {
    /// Evaluate an expression and send the result back
    Evaluate {
        /// The expression to evaluate
        expr: Box<Expr>,
        /// Source location information
        span: Option<Span>,
        /// Channel to send the result back
        sender: crossbeam::channel::Sender<Result<Value>>,
    },
    /// Define a global variable
    DefineGlobal {
        /// Variable name
        name: String,
        /// Variable value
        value: Value,
    },
    /// Import a module
    ImportModule {
        /// Import specification (boxed for memory efficiency)
        import_spec: Box<ImportSpec>,
        /// Channel to send the result back (boxed for memory efficiency)
        sender: Box<crossbeam::channel::Sender<Result<HashMap<String, Value>>>>,
    },
    /// Shutdown the evaluator thread
    Shutdown,
}
