//! Multiple values implementation
//!
//! This module implements R7RS multiple values system including values,
//! call-with-values, and multi-value continuation handling.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::rc::Rc;

/// Evaluate values special form
pub fn eval_values(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.is_empty() {
        return evaluator.apply_continuation(cont, Value::Values(Vec::new()));
    }

    if operands.len() == 1 {
        return evaluator.eval(operands[0].clone(), env, cont);
    }

    // Evaluate multiple values in left-to-right order
    let first = operands[0].clone();
    let remaining = operands[1..].to_vec();

    // Evaluate first expression, then accumulate remaining
    let first_cont = Continuation::ValuesAccumulate {
        remaining_exprs: remaining,
        accumulated_values: Vec::new(),
        env: env.clone(),
        parent: Box::new(cont),
    };

    evaluator.eval(first, env, first_cont)
}

/// Evaluate call-with-values special form
pub fn eval_call_with_values(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 2 {
        return Err(LambdustError::arity_error(2, operands.len()));
    }

    let producer_expr = operands[0].clone();
    let consumer_expr = operands[1].clone();

    let cwv_cont = Continuation::CallWithValuesStep1 {
        producer_expr,
        env: env.clone(),
        parent: Box::new(cont),
    };

    evaluator.eval(consumer_expr, env, cwv_cont)
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Apply call-with-values step 1 continuation
    pub(super) fn apply_call_with_values_step1_continuation(
        &mut self,
        consumer: Value,
        producer_expr: Expr,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // consumer is evaluated, now evaluate producer
        let cwv_step2_cont = Continuation::CallWithValuesStep2 {
            consumer,
            env: env.clone(),
            parent: Box::new(parent),
        };

        self.eval(producer_expr, env, cwv_step2_cont)
    }

    /// Apply call-with-values step 2 continuation
    pub(super) fn apply_call_with_values_step2_continuation(
        &mut self,
        producer: Value,
        consumer: Value,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Both consumer and producer are evaluated
        // Apply producer (should be a procedure with no arguments)
        let producer_result =
            self.apply_procedure(producer, Vec::new(), env.clone(), Continuation::Identity)?;

        // Convert result to arguments for consumer
        let consumer_args = match producer_result {
            Value::Values(values) => values,
            single_value => vec![single_value],
        };

        // Apply consumer with the values
        self.apply_procedure(consumer, consumer_args, env, parent)
    }
}
