//! Special forms evaluation for R7RS semantics
//!
//! This module implements evaluation of special forms like lambda, if, define, etc.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::macros::expand_macro;
use crate::value::{Procedure, Value};
use std::rc::Rc;

impl Evaluator {
    /// Dispatch to the appropriate special form evaluation
    pub fn eval_known_special_form(
        &mut self,
        name: &str,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match name {
            "lambda" => self.eval_lambda(operands, env, cont),
            "if" => self.eval_if(operands, env, cont),
            "set!" => self.eval_set(operands, env, cont),
            "quote" => self.eval_quote_special_form(operands, cont),
            "define" => self.eval_define(operands, env, cont),
            "begin" => self.eval_begin(operands, env, cont),
            "and" => self.eval_and(operands, env, cont),
            "or" => self.eval_or(operands, env, cont),
            "cond" => self.eval_cond(operands, env, cont),
            "case" => self.eval_case(operands, env, cont),
            "do" => self.eval_do(operands, env, cont),
            "delay" => self.eval_delay(operands, env, cont),
            "lazy" => self.eval_lazy(operands, env, cont),
            "force" => self.eval_force(operands, env, cont),
            "promise?" => self.eval_promise_predicate(operands, env, cont),
            "call/cc" | "call-with-current-continuation" => self.eval_call_cc(operands, env, cont),
            "values" => self.eval_values(operands, env, cont),
            "call-with-values" => self.eval_call_with_values(operands, env, cont),
            "dynamic-wind" => self.eval_dynamic_wind(operands, env, cont),
            "raise" => self.eval_raise(operands, env, cont),
            "with-exception-handler" => self.eval_with_exception_handler(operands, env, cont),
            "guard" => self.eval_guard(operands, env, cont),
            // Higher-order functions as special forms
            "map" => self.eval_map_special_form(operands, env, cont),
            "apply" => self.eval_apply_special_form(operands, env, cont),
            "fold" => self.eval_fold_special_form(operands, env, cont),
            "fold-right" => self.eval_fold_right_special_form(operands, env, cont),
            "filter" => self.eval_filter_special_form(operands, env, cont),
            // Hash table higher-order functions
            "hash-table-walk" => self.eval_hash_table_walk_special_form(operands, env, cont),
            "hash-table-fold" => self.eval_hash_table_fold_special_form(operands, env, cont),
            // Store system memory management
            "memory-usage" => self.eval_memory_usage_special_form(operands, env, cont),
            "memory-statistics" => self.eval_memory_statistics_special_form(operands, env, cont),
            "collect-garbage" => self.eval_collect_garbage_special_form(operands, env, cont),
            "set-memory-limit!" => self.eval_set_memory_limit_special_form(operands, env, cont),
            "allocate-location" => self.eval_allocate_location_special_form(operands, env, cont),
            "location-ref" => self.eval_location_ref_special_form(operands, env, cont),
            "location-set!" => self.eval_location_set_special_form(operands, env, cont),
            // Import functionality
            "import" => self.eval_import(operands, env, cont),
            _ => {
                // Try macro expansion first
                if let Some(expanded) = self.try_expand_macro(name, operands)? {
                    self.eval(expanded, env, cont)
                } else {
                    Err(LambdustError::syntax_error(format!(
                        "Unknown special form: {name}"
                    )))
                }
            }
        }
    }

    /// Evaluate quote special form
    fn eval_quote_special_form(&mut self, operands: &[Expr], cont: Continuation) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }
        let value = crate::evaluator::Evaluator::expr_to_value(operands[0].clone())?;
        self.apply_continuation(cont, value)
    }

    /// Evaluate lambda special form
    pub fn eval_lambda(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: requires at least 2 arguments (params and body)".to_string(),
            ));
        }

        // Parse parameter list
        let (params, is_variadic) = self.parse_lambda_params(&operands[0])?;

        // Body is the rest of the operands
        let body = operands[1..].to_vec();

        let lambda = Procedure::Lambda {
            params,
            body,
            closure: env,
            variadic: is_variadic,
        };

        self.apply_continuation(cont, Value::Procedure(lambda))
    }

    /// Parse lambda parameter list
    fn parse_lambda_params(&self, params_expr: &Expr) -> Result<(Vec<String>, bool)> {
        match params_expr {
            // (param1 param2 ...)
            Expr::List(params) => {
                let mut param_names = Vec::new();
                for param in params {
                    if let Expr::Variable(name) = param {
                        param_names.push(name.clone());
                    } else {
                        return Err(LambdustError::syntax_error(
                            "lambda: parameter must be a symbol".to_string(),
                        ));
                    }
                }
                Ok((param_names, false))
            }
            // Single parameter (variadic)
            Expr::Variable(name) => Ok((vec![name.clone()], true)),
            _ => Err(LambdustError::syntax_error(
                "lambda: invalid parameter list".to_string(),
            )),
        }
    }

    /// Evaluate if special form
    pub fn eval_if(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 || operands.len() > 3 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let test = operands[0].clone();
        let consequent = operands[1].clone();
        let alternate = operands.get(2).cloned();

        let if_cont = Continuation::IfTest {
            consequent,
            alternate,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(test, env, if_cont)
    }

    /// Evaluate cond special form
    pub fn eval_cond(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return Err(LambdustError::syntax_error(
                "cond: requires at least one clause".to_string(),
            ));
        }

        let clauses = self.parse_cond_clauses(operands)?;
        if clauses.is_empty() {
            return self.apply_continuation(cont, Value::Undefined);
        }

        let (test, consequent) = clauses[0].clone();
        let remaining_clauses = clauses[1..].to_vec();

        // Special handling for else clause
        if let Expr::Variable(name) = &test {
            if name == "else" {
                if !remaining_clauses.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "cond: else clause must be last".to_string(),
                    ));
                }
                return self.eval_sequence(consequent, env, cont);
            }
        }

        let cond_cont = Continuation::CondTest {
            consequent,
            remaining_clauses,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(test, env, cond_cont)
    }

    /// Parse cond clauses
    fn parse_cond_clauses(&self, operands: &[Expr]) -> Result<Vec<(Expr, Vec<Expr>)>> {
        let mut clauses = Vec::new();
        for operand in operands {
            if let Expr::List(clause_exprs) = operand {
                if clause_exprs.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "cond: empty clause".to_string(),
                    ));
                }
                let test = clause_exprs[0].clone();
                let consequent = clause_exprs[1..].to_vec();
                clauses.push((test, consequent));
            } else {
                return Err(LambdustError::syntax_error(
                    "cond: clause must be a list".to_string(),
                ));
            }
        }
        Ok(clauses)
    }

    /// Evaluate case special form (via macro expansion)
    fn eval_case(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let expanded = expand_macro("case", operands)?;
        self.eval(expanded, env, cont)
    }

    /// Evaluate set! special form
    pub fn eval_set(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let variable = match &operands[0] {
            Expr::Variable(name) => name.clone(),
            _ => {
                return Err(LambdustError::syntax_error(
                    "set!: first argument must be a variable".to_string(),
                ))
            }
        };

        let value_expr = operands[1].clone();

        let assign_cont = Continuation::Assignment {
            variable,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(value_expr, env, assign_cont)
    }

    /// Evaluate begin special form
    pub fn eval_begin(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Undefined);
        }
        self.eval_sequence(operands.to_vec(), env, cont)
    }

    /// Evaluate define special form
    pub fn eval_define(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        match &operands[0] {
            // Variable definition: (define var value)
            Expr::Variable(name) => {
                if operands.len() != 2 {
                    return Err(LambdustError::arity_error(2, operands.len()));
                }

                let variable = name.clone();
                let value_expr = operands[1].clone();

                let define_cont = Continuation::Define {
                    variable,
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.eval(value_expr, env, define_cont)
            }
            // Function definition: (define (name params...) body...)
            Expr::List(def_list) => {
                if def_list.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "define: empty function definition".to_string(),
                    ));
                }

                let function_name = match &def_list[0] {
                    Expr::Variable(name) => name.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "define: function name must be a symbol".to_string(),
                        ))
                    }
                };

                let params = Expr::List(def_list[1..].to_vec());
                let body = operands[1..].to_vec();

                // Transform to (define name (lambda params body...))
                let mut lambda_parts = vec![
                    Expr::Variable("lambda".to_string()),
                    params,
                ];
                lambda_parts.extend(body);
                let lambda_expr = Expr::List(lambda_parts);

                let define_cont = Continuation::Define {
                    variable: function_name,
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.eval(lambda_expr, env, define_cont)
            }
            _ => Err(LambdustError::syntax_error(
                "define: invalid definition form".to_string(),
            )),
        }
    }

    /// Evaluate and special form
    pub fn eval_and(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Boolean(true));
        }

        let first = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let and_cont = Continuation::And {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first, env, and_cont)
    }

    /// Evaluate or special form
    pub fn eval_or(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Boolean(false));
        }

        let first = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let or_cont = Continuation::Or {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first, env, or_cont)
    }

    /// Evaluate sequence of expressions
    pub fn eval_sequence(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_continuation(cont, Value::Undefined);
        }

        if exprs.len() == 1 {
            return self.eval(exprs[0].clone(), env, cont);
        }

        let first = exprs[0].clone();
        let remaining = exprs[1..].to_vec();

        let begin_cont = Continuation::Begin {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first, env, begin_cont)
    }

    /// Apply special form continuations (delegated from main apply_continuation)
    /// Apply special form continuations (called from mod.rs)
    pub fn apply_special_form_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => self.apply_if_test_continuation(value, consequent, alternate, env, *parent),
            Continuation::CondTest {
                consequent,
                remaining_clauses,
                env,
                parent,
            } => self.apply_cond_test_continuation(
                value,
                consequent,
                remaining_clauses,
                env,
                *parent,
            ),
            Continuation::Assignment {
                variable,
                env,
                parent,
            } => self.apply_assignment_continuation(value, variable, env, *parent),
            Continuation::Define {
                variable,
                env,
                parent,
            } => self.apply_define_continuation(value, variable, env, *parent),
            Continuation::Begin {
                remaining,
                env,
                parent,
            } => self.apply_begin_continuation(value, remaining, env, *parent),
            Continuation::And {
                remaining,
                env,
                parent,
            } => self.apply_and_continuation(value, remaining, env, *parent),
            Continuation::Or {
                remaining,
                env,
                parent,
            } => self.apply_or_continuation(value, remaining, env, *parent),
            // Delegate other continuations back to control flow
            _ => self.apply_control_flow_continuation(cont, value),
        }
    }

    /// Apply if test continuation
    fn apply_if_test_continuation(
        &mut self,
        test_value: Value,
        consequent: Expr,
        alternate: Option<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if test_value.is_truthy() {
            self.eval(consequent, env, parent)
        } else if let Some(alt) = alternate {
            self.eval(alt, env, parent)
        } else {
            self.apply_continuation(parent, Value::Undefined)
        }
    }

    /// Apply cond test continuation
    fn apply_cond_test_continuation(
        &mut self,
        test_value: Value,
        consequent: Vec<Expr>,
        remaining_clauses: Vec<(Expr, Vec<Expr>)>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if test_value.is_truthy() {
            if consequent.is_empty() {
                self.apply_continuation(parent, test_value)
            } else {
                self.eval_sequence(consequent, env, parent)
            }
        } else if remaining_clauses.is_empty() {
            self.apply_continuation(parent, Value::Undefined)
        } else {
            // Continue with next clause
            let (next_test, next_consequent) = remaining_clauses[0].clone();
            let remaining = remaining_clauses[1..].to_vec();

            // Special handling for else clause
            if let Expr::Variable(name) = &next_test {
                if name == "else" {
                    if !remaining.is_empty() {
                        return Err(LambdustError::syntax_error(
                            "cond: else clause must be last".to_string(),
                        ));
                    }
                    return self.eval_sequence(next_consequent, env, parent);
                }
            }

            let cond_cont = Continuation::CondTest {
                consequent: next_consequent,
                remaining_clauses: remaining,
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(next_test, env, cond_cont)
        }
    }

    /// Apply assignment continuation
    fn apply_assignment_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        env.set(&variable, value)?;
        self.apply_continuation(parent, Value::Undefined)
    }

    /// Apply define continuation
    fn apply_define_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        env.define(variable, value);
        self.apply_continuation(parent, Value::Undefined)
    }

    /// Apply begin continuation
    fn apply_begin_continuation(
        &mut self,
        _value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if remaining.is_empty() {
            self.apply_continuation(parent, Value::Undefined)
        } else {
            self.eval_sequence(remaining, env, parent)
        }
    }

    /// Apply and continuation
    fn apply_and_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if !value.is_truthy() {
            self.apply_continuation(parent, Value::Boolean(false))
        } else if remaining.is_empty() {
            self.apply_continuation(parent, value)
        } else {
            let first = remaining[0].clone();
            let rest = remaining[1..].to_vec();

            let and_cont = Continuation::And {
                remaining: rest,
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(first, env, and_cont)
        }
    }

    /// Apply or continuation
    fn apply_or_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            self.apply_continuation(parent, value)
        } else if remaining.is_empty() {
            self.apply_continuation(parent, Value::Boolean(false))
        } else {
            let first = remaining[0].clone();
            let rest = remaining[1..].to_vec();

            let or_cont = Continuation::Or {
                remaining: rest,
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(first, env, or_cont)
        }
    }
}