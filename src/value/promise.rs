//! Promise types for lazy evaluation (SRFI 45)

use super::Value;
use crate::ast::Expr;
use crate::environment::Environment;
use std::rc::Rc;

/// Promise for lazy evaluation (SRFI 45)
#[derive(Clone, Debug)]
pub struct Promise {
    /// The current state of the promise
    pub state: PromiseState,
}

/// State of a promise
#[derive(Clone, Debug)]
pub enum PromiseState {
    /// Unevaluated promise with expression and environment
    Lazy {
        /// Expression to evaluate
        expr: Expr,
        /// Environment for evaluation
        env: Rc<Environment>,
    },
    /// Evaluated promise with cached value
    Eager {
        /// Cached result value
        value: Box<Value>,
    },
}