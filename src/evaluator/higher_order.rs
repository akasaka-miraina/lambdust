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

        let (proc_expr, list_exprs) = operands.split_first().unwrap();

        // Evaluate procedure first
        let proc_value = self.eval(proc_expr.clone(), env.clone(), Continuation::Identity)?;

        // Evaluate list arguments
        let mut lists = Vec::new();
        for list_expr in list_exprs {
            let list_value = self.eval(list_expr.clone(), env.clone(), Continuation::Identity)?;
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
                LambdustError::type_error(
                    "apply: second argument must be a proper list".to_string(),
                )
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
            let list_value = self.eval(list_expr.clone(), env.clone(), Continuation::Identity)?;
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
            let list_value = self.eval(list_expr.clone(), env.clone(), Continuation::Identity)?;
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
                            return Err(LambdustError::arity_error(params.len() - 1, args.len()));
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
                Procedure::Continuation {
                    continuation: _captured_cont,
                } => {
                    // Apply captured continuation (simplified implementation)
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    // For now, just return the value directly
                    self.apply_continuation(cont, args[0].clone())
                }
                Procedure::CapturedContinuation {
                    continuation: captured_cont,
                } => {
                    // Apply captured continuation from evaluator
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    // Apply the captured continuation with the argument
                    self.apply_continuation(*captured_cont.clone(), args[0].clone())
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

    /// Evaluate hash-table-walk as special form
    pub fn eval_hash_table_walk_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let table_expr = operands[0].clone();
        let proc_expr = operands[1].clone();

        // Evaluate hash table
        let table_value = self.eval(table_expr, env.clone(), Continuation::Identity)?;
        let hash_table = match &table_value {
            Value::HashTable(ht) => ht,
            _ => {
                return Err(LambdustError::type_error(
                    "hash-table-walk: first argument must be a hash table".to_string(),
                ));
            }
        };

        // Evaluate procedure
        let proc_value = self.eval(proc_expr, env.clone(), Continuation::Identity)?;

        // Apply procedure to each key-value pair
        let ht = hash_table.borrow();
        for (key, value) in ht.iter() {
            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone()];

            // Apply procedure to key-value pair
            self.apply_procedure_with_evaluator(
                proc_value.clone(),
                call_args,
                env.clone(),
                Continuation::Identity,
            )?;
        }

        self.apply_continuation(cont, Value::Undefined)
    }

    /// Evaluate hash-table-fold as special form
    pub fn eval_hash_table_fold_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        let table_expr = operands[0].clone();
        let proc_expr = operands[1].clone();
        let init_expr = operands[2].clone();

        // Evaluate hash table
        let table_value = self.eval(table_expr, env.clone(), Continuation::Identity)?;
        let hash_table = match &table_value {
            Value::HashTable(ht) => ht,
            _ => {
                return Err(LambdustError::type_error(
                    "hash-table-fold: first argument must be a hash table".to_string(),
                ));
            }
        };

        // Evaluate procedure
        let proc_value = self.eval(proc_expr, env.clone(), Continuation::Identity)?;

        // Evaluate initial value
        let mut accumulator = self.eval(init_expr, env.clone(), Continuation::Identity)?;

        // Fold over each key-value pair
        let ht = hash_table.borrow();
        for (key, value) in ht.iter() {
            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone(), accumulator];

            // Apply procedure to key, value, and accumulator
            accumulator = self.apply_procedure_with_evaluator(
                proc_value.clone(),
                call_args,
                env.clone(),
                Continuation::Identity,
            )?;
        }

        self.apply_continuation(cont, accumulator)
    }

    /// Evaluate memory-usage as special form
    pub fn eval_memory_usage_special_form(
        &mut self,
        operands: &[Expr],
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !operands.is_empty() {
            return Err(LambdustError::arity_error(0, operands.len()));
        }

        let usage = self.memory_usage();
        let result = Value::Number(crate::lexer::SchemeNumber::Integer(usage as i64));
        self.apply_continuation(cont, result)
    }

    /// Evaluate memory-statistics as special form
    pub fn eval_memory_statistics_special_form(
        &mut self,
        operands: &[Expr],
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !operands.is_empty() {
            return Err(LambdustError::arity_error(0, operands.len()));
        }

        let stats = self.store_statistics();

        // Create association list with statistics (universal across store types)
        let mut stats_pairs = vec![
            Value::cons(
                Value::Symbol("total-allocations".to_string()),
                Value::Number(crate::lexer::SchemeNumber::Integer(
                    stats.total_allocations() as i64,
                )),
            ),
            Value::cons(
                Value::Symbol("total-deallocations".to_string()),
                Value::Number(crate::lexer::SchemeNumber::Integer(
                    stats.total_deallocations() as i64,
                )),
            ),
            Value::cons(
                Value::Symbol("current-memory-usage".to_string()),
                Value::Number(crate::lexer::SchemeNumber::Integer(
                    self.memory_usage() as i64
                )),
            ),
            Value::cons(
                Value::Symbol("peak-memory-usage".to_string()),
                Value::Number(crate::lexer::SchemeNumber::Integer(
                    stats.memory_usage() as i64
                )),
            ),
        ];

        // Add store-type-specific statistics
        match &stats {
            crate::evaluator::types::StoreStatisticsWrapper::Traditional(traditional_stats) => {
                stats_pairs.push(Value::cons(
                    Value::Symbol("gc-cycles".to_string()),
                    Value::Number(crate::lexer::SchemeNumber::Integer(
                        traditional_stats.gc_cycles as i64,
                    )),
                ));
                if let Ok(store) = self.store() {
                    stats_pairs.push(Value::cons(
                        Value::Symbol("location-count".to_string()),
                        Value::Number(crate::lexer::SchemeNumber::Integer(
                            store.location_count() as i64
                        )),
                    ));
                }
                stats_pairs.push(Value::cons(
                    Value::Symbol("store-type".to_string()),
                    Value::Symbol("traditional-gc".to_string()),
                ));
            }
            #[cfg(feature = "raii-store")]
            crate::evaluator::types::StoreStatisticsWrapper::Raii(raii_stats) => {
                stats_pairs.push(Value::cons(
                    Value::Symbol("active-locations".to_string()),
                    Value::Number(crate::lexer::SchemeNumber::Integer(
                        raii_stats.active_locations as i64,
                    )),
                ));
                stats_pairs.push(Value::cons(
                    Value::Symbol("peak-active-locations".to_string()),
                    Value::Number(crate::lexer::SchemeNumber::Integer(
                        raii_stats.peak_active_locations as i64,
                    )),
                ));
                stats_pairs.push(Value::cons(
                    Value::Symbol("auto-cleanup-events".to_string()),
                    Value::Number(crate::lexer::SchemeNumber::Integer(
                        raii_stats.auto_cleanup_events as i64,
                    )),
                ));
                stats_pairs.push(Value::cons(
                    Value::Symbol("store-type".to_string()),
                    Value::Symbol("raii".to_string()),
                ));
            }
        }

        let result = Value::from_vector(stats_pairs);
        self.apply_continuation(cont, result)
    }

    /// Evaluate collect-garbage as special form
    pub fn eval_collect_garbage_special_form(
        &mut self,
        operands: &[Expr],
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !operands.is_empty() {
            return Err(LambdustError::arity_error(0, operands.len()));
        }

        self.collect_garbage();
        self.apply_continuation(cont, Value::Undefined)
    }

    /// Evaluate set-memory-limit! as special form
    pub fn eval_set_memory_limit_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let limit_expr = operands[0].clone();
        let limit_value = self.eval(limit_expr, env.clone(), Continuation::Identity)?;

        let limit = match &limit_value {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Memory limit must be an integer".to_string(),
                ));
            }
        };

        self.set_memory_limit(limit);
        self.apply_continuation(cont, Value::Undefined)
    }

    /// Evaluate allocate-location as special form
    pub fn eval_allocate_location_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let value_expr = operands[0].clone();
        let value = self.eval(value_expr, env.clone(), Continuation::Identity)?;

        let _location_handle = self.allocate(value)?;
        // For now, return the location handle's ID as a number
        // In a full implementation, we'd need a Location value type
        let location_id = _location_handle.id();
        let result = Value::Number(crate::lexer::SchemeNumber::Integer(location_id as i64));
        self.apply_continuation(cont, result)
    }

    /// Evaluate location-ref as special form
    pub fn eval_location_ref_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let location_expr = operands[0].clone();
        let location_value = self.eval(location_expr, env.clone(), Continuation::Identity)?;

        let location_id = match &location_value {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Location reference must be an integer".to_string(),
                ));
            }
        };

        let location = crate::evaluator::types::Location::new(location_id);

        if let Some(value) = self.store_get(location) {
            self.apply_continuation(cont, value.clone())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )))
        }
    }

    /// Evaluate location-set! as special form
    pub fn eval_location_set_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let location_expr = operands[0].clone();
        let value_expr = operands[1].clone();

        let location_value = self.eval(location_expr, env.clone(), Continuation::Identity)?;
        let new_value = self.eval(value_expr, env.clone(), Continuation::Identity)?;

        let location_id = match &location_value {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Location reference must be an integer".to_string(),
                ));
            }
        };

        let location = crate::evaluator::types::Location::new(location_id);
        self.store_set(location, new_value)?;

        self.apply_continuation(cont, Value::Undefined)
    }
}
