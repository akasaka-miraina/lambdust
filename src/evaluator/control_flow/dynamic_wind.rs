//! Dynamic-wind implementation
//!
//! This module implements R7RS dynamic-wind special form for managing
//! dynamic extent and cleanup operations.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::rc::Rc;

/// Evaluate dynamic-wind special form
pub fn eval_dynamic_wind(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 3 {
        return Err(LambdustError::arity_error(3, operands.len()));
    }

    let before_expr = &operands[0];
    let thunk_expr = &operands[1];
    let after_expr = &operands[2];

    // Evaluate the before and after thunks
    let before_thunk = evaluator.eval(before_expr.clone(), env.clone(), Continuation::Identity)?;
    let after_thunk = evaluator.eval(after_expr.clone(), env.clone(), Continuation::Identity)?;

    // Validate that they are procedures
    if !matches!(before_thunk, Value::Procedure(_)) {
        return Err(LambdustError::type_error(
            "dynamic-wind: before thunk must be a procedure".to_string(),
        ));
    }

    if !matches!(after_thunk, Value::Procedure(_)) {
        return Err(LambdustError::type_error(
            "dynamic-wind: after thunk must be a procedure".to_string(),
        ));
    }

    // Push new dynamic point
    let dynamic_point_id =
        evaluator.push_dynamic_point(Some(before_thunk.clone()), Some(after_thunk.clone()));

    // Execute before thunk
    evaluator.apply_procedure_with_evaluator(
        before_thunk,
        vec![],
        env.clone(),
        Continuation::Identity,
    )?;

    // Evaluate the main thunk expression to get the procedure
    let main_thunk = evaluator.eval(thunk_expr.clone(), env.clone(), Continuation::Identity)?;

    // Validate that it's a procedure
    if !matches!(main_thunk, Value::Procedure(_)) {
        return Err(LambdustError::type_error(
            "dynamic-wind: main thunk must be a procedure".to_string(),
        ));
    }

    // Create continuation that will execute after thunk
    let wind_cont = Continuation::DynamicWind {
        after_thunk: after_thunk.clone(),
        dynamic_point_id,
        parent: Box::new(cont),
    };

    // Execute main thunk
    evaluator.apply_procedure_with_evaluator(main_thunk, vec![], env, wind_cont)
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Apply dynamic-wind continuation
    pub(super) fn apply_dynamic_wind_continuation(
        &mut self,
        value: Value,
        after_thunk: Value,
        dynamic_point_id: usize,
        parent: Continuation,
    ) -> Result<Value> {
        // Execute after thunk
        self.apply_procedure_with_evaluator(
            after_thunk,
            vec![],
            self.global_env.clone(),
            Continuation::Identity,
        )?;

        // Remove the dynamic point from the stack
        if let Some(point) = self.find_dynamic_point_mut(dynamic_point_id) {
            point.deactivate();
        }
        self.pop_dynamic_point();

        // Continue with the parent continuation
        self.apply_continuation(parent, value)
    }
}
