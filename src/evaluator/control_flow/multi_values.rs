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
        return evaluator.apply_evaluator_continuation(cont, Value::Values(Vec::new()));
    }

    if operands.len() == 1 {
        let (first_expr, _) = operands.split_first().unwrap();
        return evaluator.eval_with_continuation(first_expr.clone(), env, cont);
    }

    // Evaluate multiple values in left-to-right order
    let (first, remaining) = operands.split_first().unwrap();

    // Evaluate first expression, then accumulate remaining
    let first_cont = Continuation::ValuesAccumulate {
        remaining_exprs: remaining.to_vec(),
        accumulated_values: Vec::new(),
        env: env.clone(),
        parent: Box::new(cont),
    };

    evaluator.eval_with_continuation(first.clone(), env, first_cont)
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

    let producer_expr = &operands[0];
    let consumer_expr = &operands[1];

    let cwv_cont = Continuation::CallWithValuesStep1 {
        producer_expr: producer_expr.clone(),
        env: env.clone(),
        parent: Box::new(cont),
    };

    evaluator.eval_with_continuation(consumer_expr.clone(), env, cwv_cont)
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

        self.eval_with_continuation(producer_expr, env, cwv_step2_cont)
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
            self.apply_procedure_with_continuation(producer, Vec::new(), env.clone(), Continuation::Identity)?;

        // Convert result to arguments for consumer
        let consumer_args = match producer_result {
            Value::Values(values) => values,
            single_value => vec![single_value],
        };

        // Apply consumer with the values
        self.apply_procedure_with_continuation(consumer, consumer_args, env, parent)
    }
}
