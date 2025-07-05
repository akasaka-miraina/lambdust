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

    // Create a reusable captured continuation that holds the parent continuation
    // This enables both non-local exit and continuation reuse
    let reuse_id = evaluator.next_reuse_id();
    let captured_cont = Value::Procedure(Procedure::ReusableContinuation {
        continuation: Box::new(cont.clone()),
        capture_env: env.clone(),
        reuse_id,
        is_escaping: false, // Will be set to true if used for escape
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

    /// Apply captured continuation with complete non-local exit
    pub(super) fn apply_captured_continuation(
        &mut self,
        value: Value,
        cont: Continuation,
    ) -> Result<Value> {
        // Implement complete non-local exit by skipping all intermediate computations
        self.apply_captured_continuation_with_complete_exit(cont, value)
    }

    /// Apply captured continuation with complete non-local exit semantics
    /// This is the core of call/cc's non-local exit behavior
    fn apply_captured_continuation_with_complete_exit(
        &mut self,
        captured_cont: Continuation,
        escape_value: Value,
    ) -> Result<Value> {
        // For call/cc, we need to completely bypass all intermediate continuations
        // and jump directly to the captured point
        match captured_cont {
            // For CallCc continuation, skip to its parent (the capture point)
            Continuation::CallCc { parent, .. } => {
                self.apply_continuation(*parent, escape_value)
            }
            // For intermediate computation continuations, skip them entirely
            cont if cont.is_intermediate_computation() => {
                if let Some(parent) = cont.parent() {
                    self.apply_captured_continuation_with_complete_exit(parent.clone(), escape_value)
                } else {
                    // No parent, this is the final destination
                    Ok(escape_value)
                }
            }
            // For non-intermediate continuations, apply normally
            _ => self.apply_continuation(captured_cont, escape_value),
        }
    }
}
