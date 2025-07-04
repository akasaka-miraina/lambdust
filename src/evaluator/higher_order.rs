//! Higher-order functions as special forms
//!
//! This module implements higher-order functions (map, apply, fold, etc.)
//! as special forms that integrate with the evaluator for lambda support.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::{Procedure, Value};
use std::rc::Rc;

impl Evaluator {
    /// Evaluate map as special form
    pub fn eval_map_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc_expr = operands[0].clone();
        let list_exprs = operands[1..].to_vec();

        // Evaluate procedure first
        let proc_value = self.eval(proc_expr, env.clone(), Continuation::Identity)?;

        // Evaluate list arguments
        let mut lists = Vec::new();
        for list_expr in list_exprs {
            let list_value = self.eval(list_expr, env.clone(), Continuation::Identity)?;
            if !list_value.is_list() {
                return Err(LambdustError::type_error(
                    "map: all arguments except the first must be lists".to_string(),
                ));
            }
            let list_vec = list_value.to_vector().ok_or_else(|| {
                LambdustError::type_error("map: argument is not a proper list".to_string())
            })?;
            lists.push(list_vec);
        }

        // Apply map logic
        self.apply_map_logic(proc_value, lists, env, cont)
    }

    /// Apply map logic with lambda support
    fn apply_map_logic(
        &mut self,
        procedure: Value,
        lists: Vec<Vec<Value>>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Find minimum length
        let min_length = lists.iter().map(|v| v.len()).min().unwrap_or(0);

        let mut results = Vec::new();

        for i in 0..min_length {
            // Collect arguments for this iteration
            let args: Vec<Value> = lists.iter().map(|list| list[i].clone()).collect();

            // Apply procedure to arguments
            let result = self.apply_procedure_with_evaluator(
                procedure.clone(),
                args,
                env.clone(),
                Continuation::Identity,
            )?;
            results.push(result);
        }

        let result_list = Value::from_vector(results);
        self.apply_continuation(cont, result_list)
    }

    /// Evaluate apply as special form
    pub fn eval_apply_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc_expr = operands[0].clone();
        let arg_exprs = operands[1..].to_vec();

        // Evaluate procedure
        let proc_value = self.eval(proc_expr, env.clone(), Continuation::Identity)?;

        // Evaluate arguments
        let mut call_args = Vec::new();

        if arg_exprs.len() == 1 {
            // Simple form: (apply proc args)
            let arg_list = self.eval(arg_exprs[0].clone(), env.clone(), Continuation::Identity)?;
            if !arg_list.is_list() {
                return Err(LambdustError::type_error(
                    "apply: second argument must be a list".to_string(),
                ));
            }
            call_args = arg_list.to_vector().ok_or_else(|| {
                LambdustError::type_error("apply: second argument must be a proper list".to_string())
            })?;
        } else {
            // Extended form: (apply proc arg1 arg2 ... args)
            for arg_expr in &arg_exprs[..arg_exprs.len() - 1] {
                let arg_value = self.eval(arg_expr.clone(), env.clone(), Continuation::Identity)?;
                call_args.push(arg_value);
            }

            let last_arg = self.eval(
                arg_exprs[arg_exprs.len() - 1].clone(),
                env.clone(),
                Continuation::Identity,
            )?;
            if !last_arg.is_list() {
                return Err(LambdustError::type_error(
                    "apply: last argument must be a list".to_string(),
                ));
            }
            let last_list = last_arg.to_vector().ok_or_else(|| {
                LambdustError::type_error("apply: last argument must be a proper list".to_string())
            })?;
            call_args.extend(last_list);
        }

        // Apply procedure with evaluator support
        self.apply_procedure_with_evaluator(proc_value, call_args, env, cont)
    }

    /// Evaluate fold as special form
    pub fn eval_fold_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        let kons_expr = operands[0].clone();
        let knil_expr = operands[1].clone();
        let list_exprs = operands[2..].to_vec();

        // Evaluate kons procedure
        let kons = self.eval(kons_expr, env.clone(), Continuation::Identity)?;

        // Evaluate initial value
        let mut accumulator = self.eval(knil_expr, env.clone(), Continuation::Identity)?;

        // Evaluate list arguments
        let mut lists = Vec::new();
        for list_expr in list_exprs {
            let list_value = self.eval(list_expr, env.clone(), Continuation::Identity)?;
            if !list_value.is_list() {
                return Err(LambdustError::type_error(
                    "fold: all list arguments must be lists".to_string(),
                ));
            }
            let list_vec = list_value.to_vector().ok_or_else(|| {
                LambdustError::type_error("fold: argument is not a proper list".to_string())
            })?;
            lists.push(list_vec);
        }

        // Apply fold logic
        let min_length = lists.iter().map(|v| v.len()).min().unwrap_or(0);

        for i in 0..min_length {
            // Prepare arguments: accumulator + elements from each list
            let mut call_args = vec![accumulator];
            for list in &lists {
                call_args.push(list[i].clone());
            }

            // Apply kons function
            accumulator = self.apply_procedure_with_evaluator(
                kons.clone(),
                call_args,
                env.clone(),
                Continuation::Identity,
            )?;
        }

        self.apply_continuation(cont, accumulator)
    }

    /// Evaluate fold-right as special form
    pub fn eval_fold_right_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        let kons_expr = operands[0].clone();
        let knil_expr = operands[1].clone();
        let list_exprs = operands[2..].to_vec();

        // Evaluate kons procedure
        let kons = self.eval(kons_expr, env.clone(), Continuation::Identity)?;

        // Evaluate initial value
        let mut accumulator = self.eval(knil_expr, env.clone(), Continuation::Identity)?;

        // Evaluate list arguments
        let mut lists = Vec::new();
        for list_expr in list_exprs {
            let list_value = self.eval(list_expr, env.clone(), Continuation::Identity)?;
            if !list_value.is_list() {
                return Err(LambdustError::type_error(
                    "fold-right: all list arguments must be lists".to_string(),
                ));
            }
            let list_vec = list_value.to_vector().ok_or_else(|| {
                LambdustError::type_error("fold-right: argument is not a proper list".to_string())
            })?;
            lists.push(list_vec);
        }

        // Apply fold-right logic (process from right to left)
        let min_length = lists.iter().map(|v| v.len()).min().unwrap_or(0);

        for i in (0..min_length).rev() {
            // Prepare arguments: elements from each list + accumulator
            let mut call_args = Vec::new();
            for list in &lists {
                call_args.push(list[i].clone());
            }
            call_args.push(accumulator);

            // Apply kons function
            accumulator = self.apply_procedure_with_evaluator(
                kons.clone(),
                call_args,
                env.clone(),
                Continuation::Identity,
            )?;
        }

        self.apply_continuation(cont, accumulator)
    }

    /// Evaluate filter as special form
    pub fn eval_filter_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let predicate_expr = operands[0].clone();
        let list_expr = operands[1].clone();

        // Evaluate predicate
        let predicate = self.eval(predicate_expr, env.clone(), Continuation::Identity)?;

        // Evaluate list
        let list_value = self.eval(list_expr, env.clone(), Continuation::Identity)?;
        if !list_value.is_list() {
            return Err(LambdustError::type_error(
                "filter: second argument must be a list".to_string(),
            ));
        }

        let list_vec = list_value.to_vector().ok_or_else(|| {
            LambdustError::type_error("filter: second argument must be a proper list".to_string())
        })?;

        let mut results = Vec::new();

        for item in list_vec {
            // Apply predicate to item
            let keep = self.apply_procedure_with_evaluator(
                predicate.clone(),
                vec![item.clone()],
                env.clone(),
                Continuation::Identity,
            )?;

            if keep.is_truthy() {
                results.push(item);
            }
        }

        let result_list = Value::from_vector(results);
        self.apply_continuation(cont, result_list)
    }

    /// Apply procedure with full evaluator integration
    pub fn apply_procedure_with_evaluator(
        &mut self,
        procedure: Value,
        args: Vec<Value>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match procedure {
            Value::Procedure(proc) => match proc {
                Procedure::Builtin { func, arity, .. } => {
                    // Check arity if specified
                    if let Some(expected) = arity {
                        if args.len() != expected {
                            return Err(LambdustError::arity_error(expected, args.len()));
                        }
                    }

                    // Apply builtin function
                    let result = func(&args)?;
                    self.apply_continuation(cont, result)
                }
                Procedure::Lambda {
                    params,
                    body,
                    closure,
                    variadic,
                } => {
                    // Check arity for lambda
                    if variadic {
                        if args.len() < params.len() - 1 {
                            return Err(LambdustError::arity_error(
                                params.len() - 1,
                                args.len(),
                            ));
                        }
                    } else if args.len() != params.len() {
                        return Err(LambdustError::arity_error(params.len(), args.len()));
                    }

                    // Create new environment for lambda body
                    let lambda_env = Environment::with_parent(closure);

                    // Bind parameters
                    if variadic {
                        // Bind fixed parameters
                        for (i, param) in params.iter().enumerate().take(params.len() - 1) {
                            lambda_env.define(param.clone(), args[i].clone());
                        }

                        // Bind rest parameter
                        let rest_param = &params[params.len() - 1];
                        let rest_args = args[(params.len() - 1)..].to_vec();
                        lambda_env.define(rest_param.clone(), Value::from_vector(rest_args));
                    } else {
                        for (param, arg) in params.iter().zip(args.iter()) {
                            lambda_env.define(param.clone(), arg.clone());
                        }
                    }

                    // Evaluate body with full evaluator support
                    self.eval_sequence(body, Rc::new(lambda_env), cont)
                }
                Procedure::Continuation { continuation: _captured_cont } => {
                    // Apply captured continuation (simplified implementation)
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    // For now, just return the value directly
                    self.apply_continuation(cont, args[0].clone())
                }
                Procedure::HostFunction { func, arity, .. } => {
                    // Check arity if specified
                    if let Some(expected) = arity {
                        if args.len() != expected {
                            return Err(LambdustError::arity_error(expected, args.len()));
                        }
                    }

                    // Apply host function
                    let result = func(&args)?;
                    self.apply_continuation(cont, result)
                }
            },
            _ => Err(LambdustError::type_error(
                "Cannot apply non-procedure".to_string(),
            )),
        }
    }
}