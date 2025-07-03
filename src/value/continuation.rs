//! Continuation types for call/cc

use crate::ast::Expr;
use crate::environment::Environment;
use std::rc::Rc;

/// Continuation representation for call/cc
#[derive(Clone, Debug)]
pub struct Continuation {
    /// The call stack at the time of capture
    pub stack: Vec<StackFrame>,
    /// The environment at the time of capture
    pub env: Rc<Environment>,
}

/// Stack frame for continuation representation
#[derive(Clone, Debug)]
pub struct StackFrame {
    /// The expression being evaluated
    pub expr: Expr,
    /// The environment for this frame
    pub env: Rc<Environment>,
}