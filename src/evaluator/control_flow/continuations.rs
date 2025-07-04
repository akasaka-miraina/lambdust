//! Continuation application handlers
//!
//! This module implements continuation handling logic for control flow constructs,
//! providing centralized continuation application for all control flow operations.

use crate::evaluator::{Continuation, Evaluator};
use crate::error::{LambdustError, Result};
use crate::value::Value;

/// Apply control flow continuations
pub fn apply_control_flow_continuation(
    evaluator: &mut Evaluator,
    cont: Continuation,
    value: Value,
) -> Result<Value> {
    match cont {
        Continuation::Do {
            bindings,
            test,
            result_exprs,
            body_exprs,
            env,
            parent,
        } => evaluator.apply_do_continuation(
            value,
            bindings,
            test,
            result_exprs,
            body_exprs,
            env,
            *parent,
        ),
        Continuation::CallWithValuesStep1 {
            producer_expr,
            env,
            parent,
        } => evaluator.apply_call_with_values_step1_continuation(value, producer_expr, env, *parent),
        Continuation::CallWithValuesStep2 {
            consumer,
            env,
            parent,
        } => evaluator.apply_call_with_values_step2_continuation(value, consumer, env, *parent),
        Continuation::Captured { cont } => evaluator.apply_captured_continuation(value, *cont),
        Continuation::CallCc {
            captured_cont,
            env,
            parent,
        } => evaluator.apply_call_cc_continuation(value, captured_cont, env, *parent),
        Continuation::ExceptionHandler {
            handler,
            env,
            parent,
        } => evaluator.apply_exception_handler_continuation(value, handler, env, *parent),
        Continuation::GuardClause {
            condition_var,
            clauses,
            else_exprs,
            env,
            parent,
        } => evaluator.apply_guard_clause_continuation(
            value,
            condition_var,
            clauses,
            else_exprs,
            env,
            *parent,
        ),
        Continuation::DynamicWind {
            after_thunk,
            dynamic_point_id,
            parent,
        } => {
            evaluator.apply_dynamic_wind_continuation(value, after_thunk, dynamic_point_id, *parent)
        }
        _ => Err(LambdustError::runtime_error(
            "Unhandled continuation type in control flow".to_string(),
        )),
    }
}