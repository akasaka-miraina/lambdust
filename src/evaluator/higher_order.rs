//! Higher-order functions as special forms
//!
//! This module implements higher-order functions (map, apply, fold, etc.)
//! as special forms that integrate with the evaluator for lambda support.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::{Procedure, Value};
// use crate::value::conversions::ToValue;
use std::rc::Rc;
use std::collections::HashMap;

/// Statistics for location registry usage
#[derive(Debug, Clone)]
pub struct LocationRegistryStats {
    /// Total number of active locations
    pub total_locations: usize,
    /// Next location ID to be assigned
    pub next_id: usize,
    /// Estimated memory usage in bytes
    pub memory_usage: usize,
}

/// Location registry for managing stable location references
#[derive(Debug, Clone)]
pub struct LocationRegistry {
    locations: HashMap<usize, Value>,
    next_id: usize,
}

impl Default for LocationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationRegistry {
    /// Create a new location registry
    #[must_use] pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Create a new location with the given value
    pub fn create_location(&mut self, value: Value) -> usize {
        let location_id = self.next_id;
        self.next_id += 1;
        self.locations.insert(location_id, value);
        location_id
    }
    
    /// Get the value at a location
    #[must_use] pub fn get_location(&self, location_id: usize) -> Option<&Value> {
        self.locations.get(&location_id)
    }
    
    /// Set the value at a location
    pub fn set_location(&mut self, location_id: usize, value: Value) -> Result<()> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.locations.entry(location_id) {
            e.insert(value);
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!("Invalid location ID: {location_id}")))
        }
    }
    
    /// Get statistics about the location registry
    #[must_use] pub fn get_statistics(&self) -> LocationRegistryStats {
        LocationRegistryStats {
            total_locations: self.locations.len(),
            next_id: self.next_id,
            memory_usage: self.estimate_memory_usage(),
        }
    }
    
    /// Estimate memory usage of the registry
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: HashMap overhead + entries
        std::mem::size_of::<HashMap<usize, Value>>() +
        self.locations.len() * (std::mem::size_of::<usize>() + std::mem::size_of::<Value>())
    }
    
    /// Clear all locations (useful for cleanup)
    pub fn clear(&mut self) {
        self.locations.clear();
        // Note: we don't reset next_id to maintain unique IDs
    }
}

impl Evaluator {
    // ========================================
    // PUBLIC INTERFACE - Higher-order functions
    // ========================================

    /// Evaluate map as special form
    pub fn eval_map_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let (proc_expr, list_exprs) = operands.split_first().unwrap();

        // Evaluate procedure first
        let proc_value = self.eval_with_continuation(proc_expr.clone(), env.clone(), Continuation::Identity)?;

        // Evaluate list arguments  
        let lists = self.eval_list_arguments(list_exprs, env, "map")?;

        // Apply map logic
        self.apply_map_logic(proc_value, lists, cont)
    }

    /// Evaluate apply as special form
    pub fn eval_apply_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc_expr = operands[0].clone();
        let arg_exprs = operands[1..].to_vec();

        // Evaluate procedure
        let proc_value = self.eval_with_continuation(proc_expr, env.clone(), Continuation::Identity)?;

        // Evaluate arguments
        let call_args = if arg_exprs.len() == 1 {
            // Simple form: (apply proc args)
            let arg_list = self.eval_with_continuation(arg_exprs[0].clone(), env.clone(), Continuation::Identity)?;
            Self::expect_proper_list(&arg_list, "apply")?
        } else {
            // Extended form: (apply proc arg1 arg2 ... args)
            // Evaluate fixed arguments
            let mut args = self.eval_expressions(&arg_exprs[..arg_exprs.len() - 1], env)?;

            // Evaluate and append last argument (must be a list)
            // Environment-First: share same environment reference
            let last_arg = self.eval_with_continuation(arg_exprs[arg_exprs.len() - 1].clone(), env.clone(), Continuation::Identity)?;
            let last_list = Self::expect_proper_list(&last_arg, "apply")?;
            args.extend(last_list);
            args
        };

        // Apply procedure with evaluator support
        self.apply_procedure_with_evaluator(proc_value, call_args, cont)
    }

    /// Evaluate fold as special form
    pub fn eval_fold_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Environment-First: borrow environment reference, clone only when needed
        let (kons, accumulator, lists) = self.eval_fold_arguments(operands, env, "fold")?;
        
        // Apply fold logic (left-to-right)
        let result = self.apply_fold_logic(kons, accumulator, lists, false)?;
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate fold-right as special form
    pub fn eval_fold_right_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Environment-First: borrow environment reference, clone only when needed
        let (kons, accumulator, lists) = self.eval_fold_arguments(operands, env, "fold-right")?;
        
        // Apply fold logic (right-to-left)
        let result = self.apply_fold_logic(kons, accumulator, lists, true)?;
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate filter as special form
    pub fn eval_filter_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Environment-First: borrow environment reference, clone only when needed
        let (predicate, mut args) = self.eval_procedure_and_args(operands, env, 2, "filter")?;
        if args.len() != 1 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let list_vec = Self::expect_proper_list(&args.remove(0), "filter")?;

        let mut results = Vec::new();

        for item in list_vec {
            // Apply predicate to item
            let keep = self.apply_procedure_with_evaluator(
                predicate.clone(),
                vec![item.clone()],
                Continuation::Identity,
            )?;

            if keep.is_truthy() {
                results.push(item);
            }
        }

        self.apply_evaluator_continuation(cont, Value::from_vector(results))
    }

    /// Apply procedure with full evaluator integration
    pub fn apply_procedure_with_evaluator(
        &mut self,
        procedure: Value,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        // Early validation: ensure we have a procedure
        let Value::Procedure(proc) = procedure else {
            return Err(LambdustError::type_error(
                "Cannot apply non-procedure".to_string(),
            ));
        };

        // Now handle the specific procedure type
        match proc {
            Procedure::Builtin { func, arity, .. } => {
                self.check_arity_and_apply(arity, &args, cont, || func(&args))
            }
            Procedure::Lambda {
                params,
                body,
                closure,
                variadic,
            } => self.apply_lambda(params, body, closure, variadic, args, cont),
            Procedure::Continuation {
                continuation: _captured_cont,
            } => {
                self.check_arity_and_apply(Some(1), &args, cont, || {
                    // For now, just return the value directly
                    Ok(args[0].clone())
                })
            }
            Procedure::CapturedContinuation {
                continuation: captured_cont,
            } => self.apply_captured_continuation_for_procedure(captured_cont, args),
            Procedure::ReusableContinuation {
                continuation: captured_cont,
                is_escaping,
                ..
            } => self.apply_reusable_continuation_for_procedure(captured_cont, is_escaping, args),
            Procedure::HostFunction { func, arity, .. } => {
                self.check_arity_and_apply(arity, &args, cont, || func(&args))
            }
        }
    }

    /// Evaluate hash-table-walk as special form
    pub fn eval_hash_table_walk_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let table_expr = operands[0].clone();
        let proc_expr = operands[1].clone();

        // Evaluate hash table
        // Environment-First: share same environment reference
        let table_value = self.eval_with_continuation(table_expr, env.clone(), Continuation::Identity)?;
        let Value::HashTable(hash_table) = &table_value else {
                return Err(LambdustError::type_error(
                    "hash-table-walk: first argument must be a hash table".to_string(),
                ));
            };

        // Evaluate procedure
        // Environment-First: share same environment reference
        let proc_value = self.eval_with_continuation(proc_expr, env.clone(), Continuation::Identity)?;

        // Apply procedure to each key-value pair
        let ht = hash_table.borrow();
        for (key, value) in &ht.table {
            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone()];

            // Apply procedure to key-value pair
            self.apply_procedure_with_evaluator(
                proc_value.clone(),
                call_args,
                Continuation::Identity,
            )?;
        }

        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Evaluate hash-table-fold as special form
    pub fn eval_hash_table_fold_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        let table_expr = operands[0].clone();
        let proc_expr = operands[1].clone();
        let init_expr = operands[2].clone();

        // Evaluate hash table
        // Environment-First: share same environment reference
        let table_value = self.eval_with_continuation(table_expr, env.clone(), Continuation::Identity)?;
        let Value::HashTable(hash_table) = &table_value else {
                return Err(LambdustError::type_error(
                    "hash-table-fold: first argument must be a hash table".to_string(),
                ));
            };

        // Evaluate procedure
        // Environment-First: share same environment reference
        let proc_value = self.eval_with_continuation(proc_expr, env.clone(), Continuation::Identity)?;

        // Evaluate initial value
        // Environment-First: share same environment reference
        let mut accumulator = self.eval_with_continuation(init_expr, env.clone(), Continuation::Identity)?;

        // Fold over each key-value pair
        let ht = hash_table.borrow();

        #[cfg(debug_assertions)]
        eprintln!(
            "DEBUG: hash-table-fold starting with {} entries",
            ht.table.len()
        );

        for (i, (key, value)) in ht.table.iter().enumerate() {
            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: fold iteration {i}: key={key:?}, value={value:?}, acc={accumulator:?}"
            );

            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone(), accumulator];

            #[cfg(debug_assertions)]
            eprintln!("DEBUG: about to call lambda with args: {call_args:?}");

            // Apply procedure to key, value, and accumulator
            let lambda_result = self.apply_procedure_with_evaluator(
                proc_value.clone(),
                call_args,
                Continuation::Identity,
            )?;

            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: apply_procedure_with_evaluator returned: {lambda_result:?}"
            );

            accumulator = lambda_result;

            #[cfg(debug_assertions)]
            eprintln!("DEBUG: fold result {i}: new_acc={accumulator:?}");
        }

        self.apply_evaluator_continuation(cont, accumulator)
    }

    /// Evaluate memory-usage as special form
    pub fn eval_memory_usage_special_form(
        &mut self,
        operands: &[Expr],
        _env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !operands.is_empty() {
            return Err(LambdustError::arity_error(0, operands.len()));
        }

        let usage = self.memory_usage();
        let result = Value::Number(crate::lexer::SchemeNumber::Integer(usage as i64));
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate memory-statistics as special form
    pub fn eval_memory_statistics_special_form(
        &mut self,
        operands: &[Expr],
        _env: &Rc<Environment>,
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

        // Add RAII store-specific statistics (Phase 5-Step2: RAII-only)
        let raii_stats = stats.raii_statistics();
        {
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

        let result = Value::from_vector(stats_pairs);
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate collect-garbage as special form
    pub fn eval_collect_garbage_special_form(
        &mut self,
        operands: &[Expr],
        _env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !operands.is_empty() {
            return Err(LambdustError::arity_error(0, operands.len()));
        }

        let _ = self.collect_garbage();
        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Evaluate set-memory-limit! as special form
    pub fn eval_set_memory_limit_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let limit_expr = operands[0].clone();
        // Environment-First: share same environment reference
        let limit_value = self.eval_with_continuation(limit_expr, env.clone(), Continuation::Identity)?;

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
        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Evaluate allocate-location as special form
    pub fn eval_allocate_location_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let value_expr = operands[0].clone();
        // Environment-First: share same environment reference
        let value = self.eval_with_continuation(value_expr, env.clone(), Continuation::Identity)?;

        let location_handle = self.allocate(value)?;
        // For now, return the location handle's ID as a number
        // In a full implementation, we'd need a Location value type
        let location_id = location_handle.id();
        let result = Value::Number(crate::lexer::SchemeNumber::Integer(location_id as i64));
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate location-ref as special form
    pub fn eval_location_ref_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let location_expr = operands[0].clone();
        // Environment-First: share same environment reference
        let location_value = self.eval_with_continuation(location_expr, env.clone(), Continuation::Identity)?;

        let location_id = match &location_value {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Location reference must be an integer".to_string(),
                ));
            }
        };

        // Phase 5-Step2: location-ref implementation using location registry
        if let Some(location_registry) = self.get_location_registry() {
            if let Some(value) = location_registry.get_location(location_id) {
                Ok(value.clone())
            } else {
                Err(LambdustError::runtime_error(format!("Invalid location ID: {location_id}")))
            }
        } else {
            Err(LambdustError::runtime_error("Location registry not available".to_string()))
        }
    }

    /// Evaluate location-set! as special form
    pub fn eval_location_set_special_form(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let location_expr = operands[0].clone();
        let value_expr = operands[1].clone();

        // Environment-First: share same environment reference
        let location_value = self.eval_with_continuation(location_expr, env.clone(), Continuation::Identity)?;
        // Environment-First: share same environment reference
        let new_value = self.eval_with_continuation(value_expr, env.clone(), Continuation::Identity)?;

        let location_id = match &location_value {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Location reference must be an integer".to_string(),
                ));
            }
        };

        // Phase 5-Step2: location-set! implementation using location registry
        if let Some(location_registry) = self.get_location_registry_mut() {
            location_registry.set_location(location_id, new_value)?;
            Ok(Value::Undefined)
        } else {
            Err(LambdustError::runtime_error("Location registry not available".to_string()))
        }
    }

    // ========================================
    // PRIVATE HELPER METHODS
    // ========================================

    /// Evaluate a list of expressions to values
    fn eval_expressions(
        &mut self,
        exprs: &[Expr],
        env: &Rc<Environment>,
    ) -> Result<Vec<Value>> {
        let mut values = Vec::new();
        for expr in exprs {
            // Environment-First: share same environment reference
            let value = self.eval_with_continuation(expr.clone(), env.clone(), Continuation::Identity)?;
            values.push(value);
        }
        Ok(values)
    }

    /// Validate and convert a value to a proper list vector
    fn expect_proper_list(value: &Value, func_name: &str) -> Result<Vec<Value>> {
        if !value.is_list() {
            return Err(LambdustError::type_error(format!(
                "{func_name}: expected list, got {value}"
            )));
        }
        value.to_vector().ok_or_else(|| {
            LambdustError::type_error(format!(
                "{func_name}: expected proper list, got improper list"
            ))
        })
    }

    /// Evaluate multiple list expressions and convert to proper list vectors
    fn eval_list_arguments(
        &mut self,
        list_exprs: &[Expr],
        env: &Rc<Environment>,
        func_name: &str,
    ) -> Result<Vec<Vec<Value>>> {
        let mut lists = Vec::new();
        for list_expr in list_exprs {
            // Environment-First: share same environment reference
            let list_value = self.eval_with_continuation(list_expr.clone(), env.clone(), Continuation::Identity)?;
            let list_vec = Self::expect_proper_list(&list_value, func_name)?;
            lists.push(list_vec);
        }
        Ok(lists)
    }

    /// Common pattern: evaluate procedure as first argument, then other arguments
    fn eval_procedure_and_args(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        min_args: usize,
        _func_name: &str,
    ) -> Result<(Value, Vec<Value>)> {
        if operands.len() < min_args {
            return Err(LambdustError::arity_error(min_args, operands.len()));
        }

        // Environment-First: share same environment reference
        let proc_value = self.eval_with_continuation(operands[0].clone(), env.clone(), Continuation::Identity)?;
        let other_values = self.eval_expressions(&operands[1..], env)?;
        
        Ok((proc_value, other_values))
    }

    /// Common fold logic implementation
    fn apply_fold_logic(
        &mut self,
        kons: Value,
        mut accumulator: Value,
        lists: Vec<Vec<Value>>,
        is_right_fold: bool,
    ) -> Result<Value> {
        let min_length = lists.iter().map(std::vec::Vec::len).min().unwrap_or(0);

        let indices: Box<dyn Iterator<Item = usize>> = if is_right_fold {
            Box::new((0..min_length).rev())
        } else {
            Box::new(0..min_length)
        };

        for i in indices {
            let call_args = if is_right_fold {
                // fold-right: elements from each list + accumulator
                let mut args = Vec::new();
                for list in &lists {
                    args.push(list[i].clone());
                }
                args.push(accumulator);
                args
            } else {
                // fold: accumulator + elements from each list
                let mut args = vec![accumulator];
                for list in &lists {
                    args.push(list[i].clone());
                }
                args
            };

            // Apply kons function
            // Environment-First: procedure shares evaluator environment
            accumulator = self.apply_procedure_with_evaluator(
                kons.clone(),
                call_args,
                Continuation::Identity,
            )?;
        }

        Ok(accumulator)
    }

    /// Common pattern for evaluating fold-style functions (kons, knil, lists...)
    fn eval_fold_arguments(
        &mut self,
        operands: &[Expr],
        env: &Rc<Environment>,
        func_name: &str,
    ) -> Result<(Value, Value, Vec<Vec<Value>>)> {
        if operands.len() < 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        let kons_expr = operands[0].clone();
        let knil_expr = operands[1].clone();
        let list_exprs = operands[2..].to_vec();

        // Evaluate kons procedure
        // Environment-First: share same environment reference
        let kons = self.eval_with_continuation(kons_expr, env.clone(), Continuation::Identity)?;

        // Evaluate initial value
        // Environment-First: share same environment reference
        let accumulator = self.eval_with_continuation(knil_expr, env.clone(), Continuation::Identity)?;

        // Evaluate list arguments
        let lists = self.eval_list_arguments(&list_exprs, env, func_name)?;

        Ok((kons, accumulator, lists))
    }

    /// Apply map logic with lambda support
    fn apply_map_logic(
        &mut self,
        procedure: Value,
        lists: Vec<Vec<Value>>,
        cont: Continuation,
    ) -> Result<Value> {
        // Find minimum length
        let min_length = lists.iter().map(std::vec::Vec::len).min().unwrap_or(0);

        let mut results = Vec::new();

        for i in 0..min_length {
            // Collect arguments for this iteration
            let args: Vec<Value> = lists.iter().map(|list| list[i].clone()).collect();

            // Apply procedure to arguments
            // Environment-First: procedure shares evaluator environment
            let result = self.apply_procedure_with_evaluator(
                procedure.clone(),
                args,
                Continuation::Identity,
            )?;
            results.push(result);
        }

        self.apply_evaluator_continuation(cont, Value::from_vector(results))
    }

    /// Check arity and return result through continuation
    fn check_arity_and_apply<F>(
        &mut self,
        arity: Option<usize>,
        args: &[Value],
        cont: Continuation,
        apply_fn: F,
    ) -> Result<Value>
    where
        F: FnOnce() -> Result<Value>,
    {
        // Check arity if specified
        if let Some(expected) = arity {
            if args.len() != expected {
                return Err(LambdustError::arity_error(expected, args.len()));
            }
        }

        // Apply function and return through continuation
        let result = apply_fn()?;
        self.apply_evaluator_continuation(cont, result)
    }

    /// Apply lambda procedure
    fn apply_lambda(
        &mut self,
        params: Vec<String>,
        body: Vec<Expr>,
        closure: Rc<Environment>,
        variadic: bool,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        // Check arity for lambda
        if variadic {
            if args.len() < params.len() - 1 {
                return Err(LambdustError::arity_error(params.len() - 1, args.len()));
            }
        } else if args.len() != params.len() {
            return Err(LambdustError::arity_error(params.len(), args.len()));
        }

        // Create new environment for lambda body
        // Use closure environment for lambda evaluation, not the calling environment
        // Environment-First: share the same environment reference
        let lambda_env = Rc::new(Environment::with_parent(closure));

        // Bind parameters
        if variadic {
            self.bind_variadic_parameters(&lambda_env, &params, &args);
        } else {
            self.bind_fixed_parameters(&lambda_env, &params, &args);
        }

        // Evaluate body in lambda environment
        // Environment-First: use shared reference directly
        let result = self.eval_sequence(body, lambda_env, Continuation::Identity)?;

        #[cfg(debug_assertions)]
        eprintln!("DEBUG: lambda eval_sequence result: {result:?}");

        self.apply_evaluator_continuation(cont, result)
    }

    /// Bind fixed parameters (non-variadic lambda)
    fn bind_fixed_parameters(
        &self,
        lambda_env: &Rc<Environment>,
        params: &[String],
        args: &[Value],
    ) {
        for (param, arg) in params.iter().zip(args.iter()) {
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: binding param '{param}' to value {arg:?}");
            lambda_env.define(param.clone(), arg.clone());
        }
    }

    /// Bind variadic parameters (variadic lambda)
    fn bind_variadic_parameters(
        &self,
        lambda_env: &Rc<Environment>,
        params: &[String],
        args: &[Value],
    ) {
        // Bind fixed parameters
        for (i, param) in params.iter().enumerate().take(params.len() - 1) {
            lambda_env.define(param.clone(), args[i].clone());
        }

        // Bind rest parameter
        let rest_param = &params[params.len() - 1];
        let rest_args = args[(params.len() - 1)..].to_vec();
        lambda_env.define(rest_param.clone(), Value::from_vector(rest_args));
    }

    /// Apply captured continuation procedure for higher-order functions
    fn apply_captured_continuation_for_procedure(
        &mut self,
        captured_cont: Box<Continuation>,
        args: Vec<Value>,
    ) -> Result<Value> {
        // Captured continuations bypass the normal continuation flow
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        // Apply the captured continuation directly with the argument
        self.apply_evaluator_continuation(*captured_cont, args[0].clone())
    }

    /// Apply reusable continuation procedure for higher-order functions
    fn apply_reusable_continuation_for_procedure(
        &mut self,
        captured_cont: Box<Continuation>,
        is_escaping: bool,
        args: Vec<Value>,
    ) -> Result<Value> {
        // Reusable continuations also bypass normal continuation flow
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        if is_escaping {
            // Use escape semantics
            self.apply_captured_continuation_with_non_local_exit(
                *captured_cont,
                args[0].clone(),
            )
        } else {
            // Use normal continuation semantics
            self.apply_evaluator_continuation(*captured_cont, args[0].clone())
        }
    }
}