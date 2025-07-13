//! Special forms evaluation for R7RS semantics
//!
//! This module implements evaluation of special forms like lambda, if, define, etc.

use crate::ast::Expr;
use crate::debug::DebugTracer;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::evaluator::control_flow::{
    eval_call_cc, eval_call_with_values, eval_delay, eval_do, eval_dynamic_wind,
    eval_force, eval_guard, eval_lazy, eval_promise_predicate, eval_raise, eval_values,
    eval_with_exception_handler,
};
use crate::macros::{Macro, SyntaxRulesTransformer};
use crate::value::{Procedure, Value};
use std::rc::Rc;

impl Evaluator {
    // ========================================
    // PUBLIC INTERFACE - Special Forms
    // ========================================

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
            "do" => eval_do(self, operands, env, cont),
            "delay" => eval_delay(self, operands, env, cont),
            "lazy" => eval_lazy(self, operands, env, cont),
            "force" => eval_force(self, operands, env, cont),
            "promise?" => eval_promise_predicate(self, operands, env, cont),
            "call/cc" | "call-with-current-continuation" => eval_call_cc(self, operands, env, cont),
            "values" => eval_values(self, operands, env, cont),
            "call-with-values" => eval_call_with_values(self, operands, env, cont),
            "dynamic-wind" => eval_dynamic_wind(self, operands, env, cont),
            "raise" => eval_raise(self, operands, env, cont),
            "with-exception-handler" => eval_with_exception_handler(self, operands, env, cont),
            "guard" => eval_guard(self, operands, env, cont),
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
                if let Ok(expanded) = crate::macros::expand_macro(name, operands) {
                    self.eval_with_continuation(expanded, env, cont)
                } else {
                    Err(LambdustError::syntax_error(format!(
                        "Unknown special form: {name}"
                    )))
                }
            }
        }
    }

    /// Evaluate lambda expressions
    pub fn eval_lambda(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let _tracer = DebugTracer; // Simplified debug tracer
        // tracer.log("Starting lambda evaluation"); // Debug logging disabled

        let params_expr = &operands[0];
        let body_exprs = &operands[1..];

        let (params, variadic) = self.parse_lambda_params(params_expr)?;
        let body = body_exprs.to_vec();

        // tracer.log("Lambda parameters parsed successfully"); // Debug logging disabled

        let proc = Procedure::Lambda {
            params,
            body,
            closure: env,
            variadic,
        };

        let result = Value::Procedure(proc);
        self.apply_evaluator_continuation(cont, result)
    }

    /// Evaluate if conditionals
    pub fn eval_if(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 || operands.len() > 3 {
            return Err(LambdustError::arity_error_range(2, 3, operands.len()));
        }

        let test_expr = operands[0].clone();
        let then_expr = operands[1].clone();
        let else_expr = operands.get(2).cloned();

        let if_cont = Continuation::IfTest {
            consequent: then_expr,
            alternate: else_expr,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_with_continuation(test_expr, env, if_cont)
    }

    /// Evaluate cond multi-branch conditionals
    pub fn eval_cond(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return Err(LambdustError::syntax_error(
                "cond: missing clauses".to_string(),
            ));
        }

        let clauses = self.parse_cond_clauses(operands)?;

        if clauses.is_empty() {
            return self.apply_evaluator_continuation(cont, Value::Undefined);
        }

        let first_clause = clauses[0].clone();
        let remaining_clauses: Vec<(Expr, Vec<Expr>)> = clauses[1..].iter()
            .map(|clause| (clause.test.clone(), clause.body.clone()))
            .collect();

        let cond_cont = Continuation::CondTest {
            consequent: first_clause.body.clone(),
            remaining_clauses,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_with_continuation(first_clause.test, env, cond_cont)
    }

    /// Evaluate set! assignment
    pub fn eval_set(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let var_expr = &operands[0];
        let val_expr = operands[1].clone();

        let Expr::Variable(var_name) = var_expr else {
            return Err(LambdustError::syntax_error(
                "set!: first argument must be a variable".to_string(),
            ));
        };

        let assignment_cont = Continuation::Assignment {
            variable: var_name.clone(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_with_continuation(val_expr, env, assignment_cont)
    }

    /// Evaluate begin sequential evaluation
    pub fn eval_begin(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_evaluator_continuation(cont, Value::Undefined);
        }

        self.eval_sequence(operands.to_vec(), env, cont)
    }

    /// Evaluate define variable/function definitions
    pub fn eval_define(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let first = &operands[0];

        match first {
            Expr::Variable(name) => {
                // Variable definition: (define var expr)
                if operands.len() != 2 {
                    return Err(LambdustError::arity_error(2, operands.len()));
                }

                let val_expr = operands[1].clone();
                let define_cont = Continuation::Define {
                    variable: name.clone(),
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.eval_with_continuation(val_expr, env, define_cont)
            }
            Expr::List(params) => {
                // Function definition: (define (name params...) body...)
                if params.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "define: function name missing".to_string(),
                    ));
                }

                let Expr::Variable(func_name) = &params[0] else {
                    return Err(LambdustError::syntax_error(
                        "define: function name must be a variable".to_string(),
                    ));
                };

                let param_list = Expr::List(params[1..].to_vec());
                let body_exprs = &operands[1..];

                // Transform into: (define func_name (lambda (params...) body...))
                let mut lambda_operands = vec![param_list];
                lambda_operands.extend(body_exprs.iter().cloned());

                let lambda_result = self.eval_lambda(&lambda_operands, env.clone(), Continuation::Identity)?;

                let define_cont = Continuation::Define {
                    variable: func_name.clone(),
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.apply_evaluator_continuation(define_cont, lambda_result)
            }
            _ => Err(LambdustError::syntax_error(
                "define: invalid syntax".to_string(),
            )),
        }
    }

    /// Evaluate and logical conjunction
    pub fn eval_and(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_evaluator_continuation(cont, Value::Boolean(true));
        }

        let first_expr = operands[0].clone();
        let remaining_exprs = operands[1..].to_vec();

        let and_cont = Continuation::And {
            remaining: remaining_exprs,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_with_continuation(first_expr, env, and_cont)
    }

    /// Evaluate or logical disjunction
    pub fn eval_or(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_evaluator_continuation(cont, Value::Boolean(false));
        }

        let first_expr = operands[0].clone();
        let remaining_exprs = operands[1..].to_vec();

        let or_cont = Continuation::Or {
            remaining: remaining_exprs,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_with_continuation(first_expr, env, or_cont)
    }


    /// Apply special form continuations
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
            } => {
                if value.is_truthy() {
                    if consequent.is_empty() {
                        // No body: return test result
                        self.apply_evaluator_continuation(*parent, value)
                    } else {
                        // Evaluate clause body
                        self.eval_sequence(consequent, env, *parent)
                    }
                } else {
                    // Test failed, try remaining clauses
                    if remaining_clauses.is_empty() {
                        self.apply_evaluator_continuation(*parent, Value::Undefined)
                    } else {
                        let (next_test, next_body) = remaining_clauses[0].clone();
                        let remaining = remaining_clauses[1..].to_vec();
                        let cond_cont = Continuation::CondTest {
                            consequent: next_body,
                            remaining_clauses: remaining,
                            env: env.clone(),
                            parent,
                        };
                        self.eval_with_continuation(next_test, env, cond_cont)
                    }
                }
            }

            Continuation::Assignment { variable, env, parent } => {
                self.apply_assignment_continuation(value, variable, env, *parent)
            }

            Continuation::Define { variable, env, parent } => {
                self.apply_define_continuation(value, variable, env, *parent)
            }

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

            _ => self.apply_evaluator_continuation(cont, value),
        }
    }

    // ========================================
    // PRIVATE HELPER METHODS
    // ========================================

    /// Evaluate quote special form
    fn eval_quote_special_form(&mut self, operands: &[Expr], cont: Continuation) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let quoted = self.quote_expression(&operands[0])?;
        self.apply_evaluator_continuation(cont, quoted)
    }

    /// Evaluate case expressions (via macro expansion)
    fn eval_case(&mut self, _operands: &[Expr], _env: Rc<Environment>, _cont: Continuation) -> Result<Value> {
        // TODO: Implement case via macro expansion to cond
        // For now, return an error
        Err(LambdustError::syntax_error("case special form not yet implemented".to_string()))
    }

    /// Evaluate define-syntax macro definitions
    fn eval_define_syntax(&mut self, operands: &[Expr], env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let Expr::Variable(name) = &operands[0] else {
            return Err(LambdustError::syntax_error(
                "define-syntax: first argument must be a name".to_string(),
            ));
        };

        let transformer_expr = &operands[1];
        let transformer = self.parse_syntax_rules_transformer(transformer_expr)?;

        env.define_macro(name.clone(), Macro::SyntaxRules {
            name: name.clone(),
            transformer,
        });
        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Evaluate quasiquote expressions
    fn eval_quasiquote(&mut self, operands: &[Expr], env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let template = &operands[0];
        let expanded = self.expand_quasiquote(template, env)?;
        self.apply_evaluator_continuation(cont, expanded)
    }

    /// Parse lambda parameters
    fn parse_lambda_params(&self, params_expr: &Expr) -> Result<(Vec<String>, bool)> {
        match params_expr {
            Expr::List(params) => {
                let mut param_names = Vec::new();
                for param in params {
                    if let Expr::Variable(name) = param {
                        param_names.push(name.clone());
                    } else {
                        return Err(LambdustError::syntax_error(
                            "lambda: parameter must be a variable".to_string(),
                        ));
                    }
                }
                Ok((param_names, false))
            }
            Expr::Variable(name) => {
                // Single parameter (variadic)
                Ok((vec![name.clone()], true))
            }
            _ => Err(LambdustError::syntax_error(
                "lambda: invalid parameter list".to_string(),
            )),
        }
    }

    /// Parse cond clauses
    fn parse_cond_clauses(&self, operands: &[Expr]) -> Result<Vec<CondClause>> {
        let mut clauses = Vec::new();
        
        for operand in operands {
            if let Expr::List(clause_parts) = operand {
                if clause_parts.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "cond: empty clause".to_string(),
                    ));
                }

                let test = clause_parts[0].clone();
                let body = clause_parts[1..].to_vec();

                clauses.push(CondClause { test, body });
            } else {
                return Err(LambdustError::syntax_error(
                    "cond: clause must be a list".to_string(),
                ));
            }
        }

        Ok(clauses)
    }

    /// Parse syntax-rules transformer
    fn parse_syntax_rules_transformer(&self, _expr: &Expr) -> Result<SyntaxRulesTransformer> {
        // TODO: Implement full syntax-rules parsing
        // For now, return a simple placeholder
        Ok(SyntaxRulesTransformer {
            literals: vec![],
            rules: vec![],
        })
    }

    /// Expand quasiquote templates
    fn expand_quasiquote(&mut self, template: &Expr, env: Rc<Environment>) -> Result<Value> {
        match template {
            Expr::List(elements) => {
                let mut result = Vec::new();
                for element in elements {
                    // TODO: Handle unquote and unquote-splicing
                    let expanded = self.expand_quasiquote(element, env.clone())?;
                    result.push(expanded);
                }
                Ok(Value::from_vector(result))
            }
            _ => self.quote_expression(template),
        }
    }

    /// Quote expression to value
    fn quote_expression(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => match lit {
                crate::ast::Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                crate::ast::Literal::Number(n) => Ok(Value::Number(n.clone())),
                crate::ast::Literal::String(s) => Ok(Value::String(s.clone())),
                crate::ast::Literal::Character(c) => Ok(Value::Character(*c)),
                crate::ast::Literal::Nil => Ok(Value::Nil),
            },
            Expr::Variable(name) => Ok(Value::Symbol(name.clone())),
            Expr::List(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.quote_expression(expr)?);
                }
                Ok(Value::from_vector(values))
            }
            Expr::Quote(_) => Err(LambdustError::syntax_error(
                "quote: cannot quote quoted expression".to_string(),
            )),
            Expr::Vector(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.quote_expression(expr)?);
                }
                Ok(Value::Vector(values))
            }
            _ => Err(LambdustError::syntax_error(
                "quote: unsupported expression type".to_string(),
            )),
        }
    }

    /// Apply if test continuation
    fn apply_if_test_continuation(
        &mut self,
        test_result: Value,
        then_expr: Expr,
        else_expr: Option<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if test_result.is_truthy() {
            self.eval_with_continuation(then_expr, env, cont)
        } else if let Some(else_expr) = else_expr {
            self.eval_with_continuation(else_expr, env, cont)
        } else {
            self.apply_evaluator_continuation(cont, Value::Undefined)
        }
    }




    /// Apply assignment continuation
    fn apply_assignment_continuation(
        &mut self,
        value: Value,
        var: String,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        env.set(&var, value)?;
        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Apply define continuation
    fn apply_define_continuation(
        &mut self,
        value: Value,
        var: String,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        env.define(var, value);
        self.apply_evaluator_continuation(cont, Value::Undefined)
    }

    /// Apply begin continuation
    fn apply_begin_continuation(
        &mut self,
        _value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        self.eval_sequence(remaining, env, cont)
    }

    /// Apply and continuation
    fn apply_and_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if !value.is_truthy() {
            self.apply_evaluator_continuation(cont, value)
        } else if remaining.is_empty() {
            self.apply_evaluator_continuation(cont, value)
        } else {
            let first_expr = remaining[0].clone();
            let rest_exprs = remaining[1..].to_vec();

            let and_cont = Continuation::And {
                remaining: rest_exprs,
                env: env.clone(),
                parent: Box::new(cont),
            };

            self.eval_with_continuation(first_expr, env, and_cont)
        }
    }

    /// Apply or continuation
    fn apply_or_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            self.apply_evaluator_continuation(cont, value)
        } else if remaining.is_empty() {
            self.apply_evaluator_continuation(cont, value)
        } else {
            let first_expr = remaining[0].clone();
            let rest_exprs = remaining[1..].to_vec();

            let or_cont = Continuation::Or {
                remaining: rest_exprs,
                env: env.clone(),
                parent: Box::new(cont),
            };

            self.eval_with_continuation(first_expr, env, or_cont)
        }
    }
}

/// Cond clause structure
#[derive(Debug, Clone)]
struct CondClause {
    test: Expr,
    body: Vec<Expr>,
}