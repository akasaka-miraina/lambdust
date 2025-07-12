//! Special forms evaluation for R7RS semantics
//!
//! This module implements evaluation of special forms like lambda, if, define, etc.

use crate::ast::Expr;
use crate::builtins::utils::make_boolean;
use crate::debug::{DebugTracer, TraceLevel};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::macros::pattern_matching::{Pattern, Template};
use crate::macros::{expand_macro, Macro, SyntaxRule, SyntaxRulesTransformer};
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
            "lambda" => {
                // Check for typed syntax and dispatch accordingly
                if self.is_typed_lambda(operands) {
                    self.eval_typed_lambda(operands, env, cont)
                } else {
                    self.eval_lambda(operands, env, cont)
                }
            }
            "if" => self.eval_if(operands, env, cont),
            "set!" => self.eval_set(operands, env, cont),
            "quote" => self.eval_quote_special_form(operands, cont),
            "define" => {
                // Check for typed syntax and dispatch accordingly
                if self.is_typed_define(operands) {
                    self.eval_typed_define(operands, env, cont)
                } else {
                    self.eval_define(operands, env, cont)
                }
            }
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
            // Macro system
            "define-syntax" => self.eval_define_syntax(operands, env, cont),
            // Quasiquote system
            "quasiquote" => self.eval_quasiquote(operands, env, cont),
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
        let value =
            crate::evaluator::ast_converter::AstConverter::expr_to_value(operands[0].clone())?;
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
            env: Rc::clone(&env),
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
            env: Rc::clone(&env),
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
                ));
            }
        };

        let value_expr = operands[1].clone();

        let assign_cont = Continuation::Assignment {
            variable,
            env: Rc::clone(&env),
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

        #[cfg(debug_assertions)]
        DebugTracer::trace(
            "evaluator::special_forms",
            "eval_begin",
            line!(),
            TraceLevel::ENTRY,
            format!("Begin with {} operands", operands.len()),
        );

        if operands.is_empty() {
            #[cfg(debug_assertions)]
            DebugTracer::trace(
                "evaluator::special_forms",
                "eval_begin",
                line!(),
                TraceLevel::INFO,
                "Empty begin, returning Undefined".to_string(),
            );

            return self.apply_continuation(cont, Value::Undefined);
        }

        #[cfg(debug_assertions)]
        for (i, expr) in operands.iter().enumerate() {
            DebugTracer::trace_expr(
                "evaluator::special_forms",
                "eval_begin",
                line!(),
                TraceLevel::INFO,
                format!("Begin expr[{i}]"),
                expr,
            );
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
                    env: Rc::clone(&env),
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
                        ));
                    }
                };

                let params = Expr::List(def_list[1..].to_vec());
                let body = operands[1..].to_vec();

                // Transform to (define name (lambda params body...))
                let mut lambda_parts = vec![Expr::Variable("lambda".to_string()), params];
                lambda_parts.extend(body);
                let lambda_expr = Expr::List(lambda_parts);

                let define_cont = Continuation::Define {
                    variable: function_name,
                    env: Rc::clone(&env),
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
            return self.apply_continuation(cont, make_boolean(true));
        }

        let first = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let and_cont = Continuation::And {
            remaining,
            env: Rc::clone(&env),
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
            return self.apply_continuation(cont, make_boolean(false));
        }

        let first = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let or_cont = Continuation::Or {
            remaining,
            env: Rc::clone(&env),
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

        #[cfg(debug_assertions)]
        DebugTracer::trace(
            "evaluator::special_forms",
            "eval_sequence",
            line!(),
            TraceLevel::ENTRY,
            format!("Evaluating sequence of {} expressions", exprs.len()),
        );

        if exprs.is_empty() {
            #[cfg(debug_assertions)]
            DebugTracer::trace(
                "evaluator::special_forms",
                "eval_sequence",
                line!(),
                TraceLevel::INFO,
                "Empty sequence, returning Undefined".to_string(),
            );

            return self.apply_continuation(cont, Value::Undefined);
        }

        if exprs.len() == 1 {
            let (first_expr, _) = exprs.split_first().unwrap();

            #[cfg(debug_assertions)]
            DebugTracer::trace_expr(
                "evaluator::special_forms",
                "eval_sequence",
                line!(),
                TraceLevel::INFO,
                "Single expression in sequence".to_string(),
                first_expr,
            );

            #[cfg(debug_assertions)]
            DebugTracer::trace(
                "evaluator::special_forms",
                "eval_sequence",
                line!(),
                TraceLevel::INFO,
                format!("Calling eval with continuation: {cont:?}"),
            );

            #[cfg(debug_assertions)]
            {
                        DebugTracer::trace_expr(
                    "evaluator::special_forms",
                    "eval_sequence",
                    line!(),
                    TraceLevel::INFO,
                    "BEFORE eval() call - expression".to_string(),
                    first_expr,
                );
            }

            let eval_result = self.eval(first_expr.clone(), env, cont);

            #[cfg(debug_assertions)]
            {
                        DebugTracer::trace(
                    "evaluator::special_forms",
                    "eval_sequence",
                    line!(),
                    TraceLevel::INFO,
                    "AFTER eval() call - about to check result".to_string(),
                );
            }

            #[cfg(debug_assertions)]
            {
                        DebugTracer::trace(
                    "evaluator::special_forms",
                    "eval_sequence",
                    line!(),
                    TraceLevel::INFO,
                    format!("eval() returned: {eval_result:?}"),
                );
            }

            return eval_result;
        }

        let (first, remaining) = exprs.split_first().unwrap();

        #[cfg(debug_assertions)]
        DebugTracer::trace_expr(
            "evaluator::special_forms",
            "eval_sequence",
            line!(),
            TraceLevel::INFO,
            format!("First expr (remaining: {})", remaining.len()),
            first,
        );

        let begin_cont = Continuation::Begin {
            remaining: remaining.to_vec(),
            env: Rc::clone(&env),
            parent: Box::new(cont),
        };

        #[cfg(debug_assertions)]
        DebugTracer::trace_continuation(
            "evaluator::special_forms",
            "eval_sequence",
            line!(),
            TraceLevel::INFO,
            "Created Begin continuation".to_string(),
            "Begin",
            Some(env.depth()),
        );

        self.eval(first.clone(), env, begin_cont)
    }

    /// Apply special form continuations (delegated from main `apply_continuation`)
    /// Apply special form continuations (called from mod.rs)
    pub fn apply_special_form_continuation(
        &mut self,
        cont: Continuation,
        value: Value,
    ) -> Result<Value> {
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
            return self.handle_truthy_cond_test(test_value, consequent, env, parent);
        }

        if remaining_clauses.is_empty() {
            return self.apply_continuation(parent, Value::Undefined);
        }

        self.process_next_cond_clause(remaining_clauses, env, parent)
    }

    /// Handle the case when a cond test is truthy
    fn handle_truthy_cond_test(
        &mut self,
        test_value: Value,
        consequent: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if consequent.is_empty() {
            self.apply_continuation(parent, test_value)
        } else {
            self.eval_sequence(consequent, env, parent)
        }
    }

    /// Process the next clause in a cond expression
    fn process_next_cond_clause(
        &mut self,
        remaining_clauses: Vec<(Expr, Vec<Expr>)>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
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
            env: Rc::clone(&env),
            parent: Box::new(parent),
        };

        self.eval(next_test, env, cond_cont)
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

        #[cfg(debug_assertions)]
        DebugTracer::trace_value(
            "evaluator::special_forms",
            "apply_define_continuation",
            line!(),
            TraceLevel::ENTRY,
            format!("Defining variable: {variable}"),
            &value,
        );

        env.define(variable.clone(), value);

        #[cfg(debug_assertions)]
        DebugTracer::trace(
            "evaluator::special_forms",
            "apply_define_continuation",
            line!(),
            TraceLevel::INFO,
            format!("Variable '{variable}' defined, returning Undefined"),
        );

        let result = self.apply_continuation(parent, Value::Undefined)?;

        #[cfg(debug_assertions)]
        DebugTracer::trace_value(
            "evaluator::special_forms",
            "apply_define_continuation",
            line!(),
            TraceLevel::EXIT,
            "Define continuation result".to_string(),
            &result,
        );

        Ok(result)
    }

    /// Apply begin continuation
    fn apply_begin_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {

        #[cfg(debug_assertions)]
        DebugTracer::trace_value(
            "evaluator::special_forms",
            "apply_begin_continuation",
            line!(),
            TraceLevel::ENTRY,
            format!("Begin continuation, remaining: {}", remaining.len()),
            &value,
        );

        if remaining.is_empty() {
            #[cfg(debug_assertions)]
            DebugTracer::trace(
                "evaluator::special_forms",
                "apply_begin_continuation",
                line!(),
                TraceLevel::INFO,
                "No remaining expressions, returning value".to_string(),
            );

            let result = self.apply_continuation(parent, value)?;

            #[cfg(debug_assertions)]
            DebugTracer::trace_value(
                "evaluator::special_forms",
                "apply_begin_continuation",
                line!(),
                TraceLevel::EXIT,
                "Final result".to_string(),
                &result,
            );

            Ok(result)
        } else {
            #[cfg(debug_assertions)]
            DebugTracer::trace(
                "evaluator::special_forms",
                "apply_begin_continuation",
                line!(),
                TraceLevel::INFO,
                format!("Evaluating remaining {} expressions", remaining.len()),
            );

            let result = self.eval_sequence(remaining, env, parent)?;

            #[cfg(debug_assertions)]
            DebugTracer::trace_value(
                "evaluator::special_forms",
                "apply_begin_continuation",
                line!(),
                TraceLevel::EXIT,
                "Sequence result".to_string(),
                &result,
            );

            Ok(result)
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
            self.apply_continuation(parent, make_boolean(false))
        } else if remaining.is_empty() {
            self.apply_continuation(parent, value)
        } else {
            let first = remaining[0].clone();
            let rest = remaining[1..].to_vec();

            let and_cont = Continuation::And {
                remaining: rest,
                env: Rc::clone(&env),
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
            self.apply_continuation(parent, make_boolean(false))
        } else {
            let first = remaining[0].clone();
            let rest = remaining[1..].to_vec();

            let or_cont = Continuation::Or {
                remaining: rest,
                env: Rc::clone(&env),
                parent: Box::new(parent),
            };

            self.eval(first, env, or_cont)
        }
    }

    /// Evaluate define-syntax special form
    fn eval_define_syntax(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        // Syntax: (define-syntax <name> <transformer>)
        if operands.len() != 2 {
            return Err(LambdustError::syntax_error(
                "define-syntax: expected exactly 2 arguments".to_string(),
            ));
        }

        // Extract macro name
        let macro_name = match &operands[0] {
            Expr::Variable(name) => name.clone(),
            _ => {
                return Err(LambdustError::syntax_error(
                    "define-syntax: first argument must be a symbol".to_string(),
                ));
            }
        };

        // Parse transformer (must be syntax-rules)
        let transformer = &operands[1];
        let macro_def = self.parse_syntax_rules_transformer(transformer)?;

        // Define the macro in the environment
        env.define_macro(macro_name.clone(), macro_def);

        // Return the macro name as the result
        Ok(Value::Symbol(macro_name))
    }

    /// Parse syntax-rules transformer
    fn parse_syntax_rules_transformer(&self, expr: &Expr) -> Result<Macro> {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                // Check if it's a syntax-rules form
                if let Expr::Variable(name) = &exprs[0] {
                    if name == "syntax-rules" {
                        return self.parse_syntax_rules(&exprs[1..]);
                    }
                }
            }
            _ => {}
        }

        Err(LambdustError::syntax_error(
            "define-syntax: transformer must be a syntax-rules expression".to_string(),
        ))
    }

    /// Parse syntax-rules expression
    fn parse_syntax_rules(&self, args: &[Expr]) -> Result<Macro> {
        // Syntax: (syntax-rules <literals> <rule1> <rule2> ...)
        if args.is_empty() {
            return Err(LambdustError::syntax_error(
                "syntax-rules: missing literal list".to_string(),
            ));
        }

        // Parse literals
        let literals = self.parse_literals(&args[0])?;

        // Parse rules
        let mut rules = Vec::new();
        for rule_expr in &args[1..] {
            let rule = self.parse_syntax_rule(rule_expr)?;
            rules.push(rule);
        }

        if rules.is_empty() {
            return Err(LambdustError::syntax_error(
                "syntax-rules: at least one rule is required".to_string(),
            ));
        }

        // Create syntax-rules transformer
        let transformer = SyntaxRulesTransformer::new(literals, rules);
        Ok(Macro::SyntaxRules {
            name: "user-defined".to_string(),
            transformer,
        })
    }

    /// Parse literals list
    fn parse_literals(&self, expr: &Expr) -> Result<Vec<String>> {
        match expr {
            Expr::List(exprs) => {
                let mut literals = Vec::new();
                for expr in exprs {
                    match expr {
                        Expr::Variable(name) => literals.push(name.clone()),
                        _ => {
                            return Err(LambdustError::syntax_error(
                                "syntax-rules: literals must be symbols".to_string(),
                            ));
                        }
                    }
                }
                Ok(literals)
            }
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: literals must be a list".to_string(),
            )),
        }
    }

    /// Parse syntax rule
    fn parse_syntax_rule(&self, expr: &Expr) -> Result<SyntaxRule> {
        match expr {
            Expr::List(exprs) if exprs.len() == 2 => {
                // Parse pattern
                let pattern = self.parse_pattern(&exprs[0])?;
                // Parse template
                let template = self.parse_template(&exprs[1])?;
                Ok(SyntaxRule { pattern, template })
            }
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: each rule must be a list of two elements (pattern template)"
                    .to_string(),
            )),
        }
    }

    /// Parse pattern
    fn parse_pattern(&self, expr: &Expr) -> Result<Pattern> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis".to_string(),
                    ));
                }
                Ok(Pattern::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Pattern::List(vec![]));
                }

                let mut patterns = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if patterns.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding pattern".to_string(),
                                ));
                            }
                            let last_pattern = patterns.pop().unwrap();
                            patterns.push(Pattern::Ellipsis(Box::new(last_pattern)));
                            i += 1;
                            continue;
                        }
                    }

                    patterns.push(self.parse_pattern(&exprs[i])?);
                    i += 1;
                }

                Ok(Pattern::List(patterns))
            }
            Expr::Literal(lit) => Ok(Pattern::Literal(format!("{lit:?}"))),
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid pattern".to_string(),
            )),
        }
    }

    /// Parse template
    fn parse_template(&self, expr: &Expr) -> Result<Template> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis in template".to_string(),
                    ));
                }
                Ok(Template::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Template::List(vec![]));
                }

                let mut templates = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if templates.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding template".to_string(),
                                ));
                            }
                            let last_template = templates.pop().unwrap();
                            templates.push(Template::Ellipsis(Box::new(last_template)));
                            i += 1;
                            continue;
                        }
                    }

                    templates.push(self.parse_template(&exprs[i])?);
                    i += 1;
                }

                Ok(Template::List(templates))
            }
            Expr::Literal(lit) => Ok(Template::Literal(format!("{lit:?}"))),
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid template".to_string(),
            )),
        }
    }

    /// Evaluate quasiquote special form
    fn eval_quasiquote(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Syntax: (quasiquote <template>)
        if operands.len() != 1 {
            return Err(LambdustError::syntax_error(
                "quasiquote: expected exactly 1 argument".to_string(),
            ));
        }

        let template = &operands[0];
        let result = self.expand_quasiquote(template, env, 0)?;
        self.apply_continuation(cont, result)
    }

    /// Expand quasiquote template
    fn expand_quasiquote(
        &mut self,
        template: &Expr,
        env: Rc<Environment>,
        depth: usize,
    ) -> Result<Value> {
        match template {
            // Handle unquote
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) if name == "unquote" => {
                        if depth == 0 {
                            // Evaluate the unquoted expression
                            if exprs.len() != 2 {
                                return Err(LambdustError::syntax_error(
                                    "unquote: expected exactly 1 argument".to_string(),
                                ));
                            }
                            return self.eval(exprs[1].clone(), env, Continuation::Identity);
                        }
                        // Nested quasiquote, decrease depth
                        self.expand_quasiquote_list(exprs, env, depth - 1)
                    }
                    Expr::Variable(name) if name == "unquote-splicing" => {
                        Err(LambdustError::syntax_error(
                            "unquote-splicing: can only appear within a list".to_string(),
                        ))
                    }
                    Expr::Variable(name) if name == "quasiquote" => {
                        // Nested quasiquote, increase depth
                        self.expand_quasiquote_list(exprs, env, depth + 1)
                    }
                    _ => {
                        // Regular list processing
                        self.expand_quasiquote_list(exprs, env, depth)
                    }
                }
            }
            // Handle vectors
            Expr::Vector(exprs) => {
                let mut result = Vec::new();
                for expr in exprs {
                    let value = self.expand_quasiquote(expr, env.clone(), depth)?;
                    result.push(value);
                }
                Ok(Value::from_vector(result))
            }
            // Handle atoms (literals, variables, etc.)
            _ => {
                // Convert expression to value as-is (quoted)
                self.quote_expression(template)
            }
        }
    }

    /// Expand quasiquote list with unquote-splicing support
    fn expand_quasiquote_list(
        &mut self,
        exprs: &[Expr],
        env: Rc<Environment>,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < exprs.len() {
            match &exprs[i] {
                Expr::List(inner_exprs) if !inner_exprs.is_empty() => {
                    if let Expr::Variable(name) = &inner_exprs[0] {
                        if name == "unquote-splicing" && depth == 0 {
                            // Handle unquote-splicing
                            if inner_exprs.len() != 2 {
                                return Err(LambdustError::syntax_error(
                                    "unquote-splicing: expected exactly 1 argument".to_string(),
                                ));
                            }
                            let spliced_value = self.eval(
                                inner_exprs[1].clone(),
                                env.clone(),
                                Continuation::Identity,
                            )?;

                            // Convert to list and splice
                            match spliced_value {
                                Value::Vector(vec) => {
                                    result.extend(vec);
                                }
                                Value::Nil => {
                                    // Empty list, nothing to splice
                                }
                                _ => {
                                    // Try to convert to list
                                    let list_items = self.value_to_list(spliced_value)?;
                                    result.extend(list_items);
                                }
                            }
                            i += 1;
                            continue;
                        }
                    }
                    // Regular expression
                    let value = self.expand_quasiquote(&exprs[i], env.clone(), depth)?;
                    result.push(value);
                }
                _ => {
                    // Regular expression
                    let value = self.expand_quasiquote(&exprs[i], env.clone(), depth)?;
                    result.push(value);
                }
            }
            i += 1;
        }

        Ok(Value::from_vector(result))
    }

    /// Quote an expression (convert to value representation)
    fn quote_expression(&self, expr: &Expr) -> Result<Value> {
        use crate::evaluator::ast_converter::AstConverter;
        AstConverter::expr_to_value(expr.clone())
    }

    /// Convert a value to a list of values
    fn value_to_list(&self, value: Value) -> Result<Vec<Value>> {
        match value {
            Value::Vector(vec) => Ok(vec),
            Value::Nil => Ok(Vec::new()),
            Value::Pair(pair_ref) => {
                let mut result = Vec::new();
                let mut current = Value::Pair(pair_ref);

                loop {
                    match current {
                        Value::Pair(pair_ref) => {
                            let pair = pair_ref.borrow();
                            result.push(pair.car.clone());
                            current = pair.cdr.clone();
                        }
                        Value::Nil => break,
                        _ => {
                            return Err(LambdustError::type_error(
                                "unquote-splicing: expected proper list".to_string(),
                            ));
                        }
                    }
                }
                Ok(result)
            }
            _ => Err(LambdustError::type_error(
                "unquote-splicing: expected list or vector".to_string(),
            )),
        }
    }
}
