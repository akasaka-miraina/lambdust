//! Call/cc (call-with-current-continuation) implementation
//!
//! This module implements R7RS call/cc special form for capturing and
//! invoking continuations, enabling non-local exits and advanced control flow.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::{Procedure, Value};
use std::rc::Rc;

/// Evaluate call/cc special form
pub fn eval_call_cc(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 1 {
        return Err(LambdustError::arity_error(1, operands.len()));
    }

    let proc_expr = operands[0].clone();

    // Create a captured continuation that holds the current continuation
    let captured_cont = Value::Procedure(Procedure::CapturedContinuation {
        continuation: Box::new(cont.clone()),
    });

    let call_cc_cont = Continuation::CallCc {
        captured_cont,
        env: env.clone(),
        parent: Box::new(cont),
    };

    evaluator.eval(proc_expr, env, call_cc_cont)
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Apply call/cc continuation
    pub(super) fn apply_call_cc_continuation(
        &mut self,
        procedure: Value,
        captured_cont: Value,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Apply the procedure with the captured continuation as argument
        self.apply_procedure(procedure, vec![captured_cont], env, parent)
    }

    /// Apply captured continuation
    pub(super) fn apply_captured_continuation(
        &mut self,
        value: Value,
        cont: Continuation,
    ) -> Result<Value> {
        self.apply_continuation(cont, value)
    }
}