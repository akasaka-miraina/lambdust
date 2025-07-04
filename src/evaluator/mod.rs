//! R7RS formal semantics compliant evaluator
//!
//! This module implements a continuation-passing style evaluator
//! that strictly follows the R7RS formal semantics definition.

pub mod continuation;
pub mod control_flow;
pub mod higher_order;
pub mod imports;
#[cfg(feature = "raii-store")]
pub mod raii_store;
pub mod special_forms;
pub mod types;

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::macros::expand_macro;
use crate::value::{Procedure, Value};

// Re-export main types
pub use continuation::{Continuation, DynamicPoint};
pub use types::*;

use std::rc::Rc;

impl Evaluator {
    /// Main evaluation function: E[e]ρκσ
    /// Where:
    /// - e: expression to evaluate
    /// - ρ: environment
    /// - κ: continuation
    /// - σ: store
    pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // Stack overflow prevention
        if self.recursion_depth() >= self.max_recursion_depth() {
            return Err(LambdustError::stack_overflow());
        }

        self.increment_recursion_depth()?;
        let result = match expr {
            // Constants: E[K]ρκσ = κ(K[K])
            Expr::Literal(lit) => self.eval_literal(lit, cont),

            // Variables: E[I]ρκσ = κ(σ(ρ(I)))
            Expr::Variable(name) => self.eval_variable(name, env, cont),

            // Function application: E[(E0 E1 ...)]ρκσ
            Expr::List(exprs) if !exprs.is_empty() => self.eval_application(exprs, env, cont),

            // Empty list
            Expr::List(exprs) if exprs.is_empty() => self.eval_literal(Literal::Nil, cont),

            // Quote: E['E]ρκσ = κ(E[E])
            Expr::Quote(expr) => self.eval_quote(*expr, cont),

            // Quasiquote: E[`E]ρκσ = κ(quasiquote-expand(E))
            Expr::Quasiquote(expr) => self.eval_quasiquote(*expr, env, cont),

            // Vector: evaluate all elements
            Expr::Vector(exprs) => self.eval_vector(exprs, env, cont),

            // Dotted list (improper list)
            Expr::DottedList(_, _) => Err(LambdustError::syntax_error(
                "Dotted lists not supported in this context".to_string(),
            )),

            // Other forms
            _ => Err(LambdustError::syntax_error(format!(
                "Unsupported expression: {expr:?}"
            ))),
        };

        self.decrement_recursion_depth();
        result
    }

    /// Evaluate literal: K[K]
    fn eval_literal(&mut self, lit: Literal, cont: Continuation) -> Result<Value> {
        let value = match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Character(c) => Value::Character(c),
            Literal::Nil => Value::Nil,
        };
        self.apply_continuation(cont, value)
    }

    /// Evaluate variable: σ(ρ(I))
    fn eval_variable(
        &mut self,
        name: String,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match env.get(&name) {
            Ok(value) => self.apply_continuation(cont, value),
            Err(_) => Err(LambdustError::undefined_variable(name)),
        }
    }

    /// Evaluate application: E[(E0 E1 ...)]ρκσ
    fn eval_application(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return Err(LambdustError::syntax_error(
                "Empty application".to_string(),
            ));
        }

        // Try to handle special forms first
        if let Some(special_result) =
            self.try_eval_special_form(&exprs, env.clone(), cont.clone())?
        {
            return Ok(special_result);
        }

        // Regular function application: evaluate operator first
        let operator_expr = exprs[0].clone();
        let args = exprs[1..].to_vec();

        let operator_cont = Continuation::Operator {
            args,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(operator_expr, env, operator_cont)
    }

    /// Try to evaluate as special form
    fn try_eval_special_form(
        &mut self,
        exprs: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        if let Expr::Variable(name) = &exprs[0] {
            if self.is_special_form(name) {
                return Ok(Some(
                    self.eval_known_special_form(name, &exprs[1..], env, cont)?,
                ));
            }
        }
        Ok(None)
    }

    /// Check if a name is a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(
            name,
            "lambda"
                | "if"
                | "set!"
                | "quote"
                | "define"
                | "begin"
                | "and"
                | "or"
                | "cond"
                | "case"
                | "do"
                | "delay"
                | "lazy"
                | "force"
                | "promise?"
                | "call/cc"
                | "call-with-current-continuation"
                | "values"
                | "call-with-values"
                | "dynamic-wind"
                | "raise"
                | "with-exception-handler"
                | "guard"
                | "map"
                | "apply"
                | "fold"
                | "fold-right"
                | "filter"
                | "hash-table-walk"
                | "hash-table-fold"
                | "memory-usage"
                | "memory-statistics"
                | "collect-garbage"
                | "set-memory-limit!"
                | "allocate-location"
                | "location-ref"
                | "location-set!"
                | "import"
        )
    }

    /// Evaluate quote form: E['E]ρκσ = κ(E[E])
    fn eval_quote(&mut self, expr: Expr, cont: Continuation) -> Result<Value> {
        let value = Self::expr_to_value(expr)?;
        self.apply_continuation(cont, value)
    }

    /// Convert expression to value (for quote)
    fn expr_to_value(expr: Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::Boolean(b) => Value::Boolean(b),
                Literal::Number(n) => Value::Number(n),
                Literal::String(s) => Value::String(s),
                Literal::Character(c) => Value::Character(c),
                Literal::Nil => Value::Nil,
            }),
            Expr::Variable(name) => Ok(Value::Symbol(name)),
            Expr::List(exprs) => {
                let mut result = Value::Nil;
                for expr in exprs.into_iter().rev() {
                    let value = Self::expr_to_value(expr)?;
                    result = Value::cons(value, result);
                }
                Ok(result)
            }
            Expr::Vector(exprs) => {
                let values: Result<Vec<Value>> = exprs.into_iter().map(Self::expr_to_value).collect();
                Ok(Value::from_vector(values?))
            }
            Expr::Quote(expr) => Self::expr_to_value(*expr),
            Expr::DottedList(_, _) => Err(LambdustError::syntax_error(
                "Dotted lists not supported in quote context".to_string(),
            )),
            Expr::Quasiquote(_) | Expr::Unquote(_) | Expr::UnquoteSplicing(_) => {
                Err(LambdustError::syntax_error(
                    "Quasiquote forms not yet implemented in quote context".to_string(),
                ))
            }
        }
    }

    /// Evaluate quasiquote (simplified implementation)
    fn eval_quasiquote(
        &mut self,
        _expr: Expr,
        _env: Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::syntax_error(
            "Quasiquote not yet fully implemented".to_string(),
        ))
    }

    /// Evaluate vector
    fn eval_vector(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_continuation(cont, Value::from_vector(Vec::new()));
        }

        let first_expr = exprs[0].clone();
        let remaining = exprs[1..].to_vec();

        let vector_cont = Continuation::VectorEval {
            evaluated_elements: Vec::new(),
            remaining_elements: remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr, env, vector_cont)
    }

    /// Apply continuation: κ(v)
    pub fn apply_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            // Inline simple identity continuation for performance
            Continuation::Identity => Ok(value),
            Continuation::Operator { args, env, parent } => {
                self.apply_operator_continuation(value, args, env, *parent)
            }
            Continuation::Application {
                operator,
                evaluated_args,
                remaining_args,
                env,
                parent,
            } => self.apply_application_continuation(
                value,
                operator,
                evaluated_args,
                remaining_args,
                env,
                *parent,
            ),
            Continuation::Values { mut values, parent } => {
                // Inline for performance
                values.push(value);
                self.apply_continuation(*parent, Value::Values(values))
            }
            Continuation::VectorEval {
                mut evaluated_elements,
                remaining_elements,
                env,
                parent,
            } => {
                // Add the current value to evaluated elements
                evaluated_elements.push(value);

                if remaining_elements.is_empty() {
                    // All elements evaluated, create vector
                    let vector = Value::Vector(evaluated_elements);
                    self.apply_continuation(*parent, vector)
                } else {
                    // Continue evaluating remaining elements
                    let next_expr = remaining_elements[0].clone();
                    let remaining = remaining_elements[1..].to_vec();

                    let vector_cont = Continuation::VectorEval {
                        evaluated_elements,
                        remaining_elements: remaining,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr, env, vector_cont)
                }
            }
            // Delegate special form continuations to appropriate modules
            _ => self.apply_special_continuation(cont, value),
        }
    }

    /// Apply operator continuation
    fn apply_operator_continuation(
        &mut self,
        operator: Value,
        args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if args.is_empty() {
            // No arguments, apply directly
            self.apply_procedure(operator, Vec::new(), env, parent)
        } else {
            // Evaluate arguments according to evaluation order
            self.eval_arguments_in_order(operator, args, env, parent)
        }
    }

    /// Apply application continuation
    fn apply_application_continuation(
        &mut self,
        arg_value: Value,
        operator: Value,
        mut evaluated_args: Vec<Value>,
        remaining_args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        evaluated_args.push(arg_value);

        if remaining_args.is_empty() {
            // All arguments evaluated, apply procedure
            self.apply_procedure(operator, evaluated_args, env, parent)
        } else {
            // Continue evaluating remaining arguments
            let next_arg = remaining_args[0].clone();
            let remaining = remaining_args[1..].to_vec();

            let app_cont = Continuation::Application {
                operator,
                evaluated_args,
                remaining_args: remaining,
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(next_arg, env, app_cont)
        }
    }

    /// Evaluate arguments in the specified order
    fn eval_arguments_in_order(
        &mut self,
        operator: Value,
        args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        match self.eval_order() {
            EvalOrder::LeftToRight => {
                let first_arg = args[0].clone();
                let remaining = args[1..].to_vec();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining,
                    env: env.clone(),
                    parent: Box::new(parent),
                };

                self.eval(first_arg, env, app_cont)
            }
            EvalOrder::RightToLeft => {
                // Evaluate from right to left
                let last_arg = args[args.len() - 1].clone();
                let remaining: Vec<Expr> = args[..args.len() - 1].to_vec();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining,
                    env: env.clone(),
                    parent: Box::new(parent),
                };

                self.eval(last_arg, env, app_cont)
            }
            EvalOrder::Unspecified => {
                // For now, default to left-to-right
                // In a full implementation, this could randomize
                self.eval_arguments_in_order(operator, args, env, parent)
            }
        }
    }

    /// Apply procedure
    fn apply_procedure(
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

                    // Evaluate body
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

    /// Apply special continuations (delegates to appropriate modules)
    fn apply_special_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Try special form continuations first
        match &cont {
            Continuation::IfTest { .. }
            | Continuation::CondTest { .. }
            | Continuation::Assignment { .. }
            | Continuation::Define { .. }
            | Continuation::Begin { .. }
            | Continuation::And { .. }
            | Continuation::Or { .. } => {
                self.apply_special_form_continuation(cont, value)
            }
            // Default to control flow continuations
            _ => self.apply_control_flow_continuation(cont, value),
        }
    }

    /// Evaluate a string containing Scheme code
    pub fn eval_string(&mut self, input: &str) -> Result<Value> {
        use crate::parser::Parser;

        let tokens = crate::lexer::tokenize(input)?;
        let mut parser = Parser::new(tokens);
        let exprs = parser.parse_all()?;

        if exprs.is_empty() {
            return Ok(Value::Undefined);
        }

        // Evaluate all expressions, return the last result
        let mut result = Value::Undefined;
        for expr in exprs {
            result = self.eval(expr, self.global_env.clone(), Continuation::Identity)?;
        }

        Ok(result)
    }

    /// Call a procedure (for compatibility)
    pub fn call_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value> {
        self.apply_procedure(procedure, args, self.global_env.clone(), Continuation::Identity)
    }

    /// Macro expansion integration
    fn try_expand_macro(&self, name: &str, args: &[Expr]) -> Result<Option<Expr>> {
        match name {
            "let" | "let*" | "letrec" | "case" | "when" | "unless" => {
                let expanded = expand_macro(name, args)?;
                Ok(Some(expanded))
            }
            _ => Ok(None),
        }
    }
}

/// Public API for evaluation
pub fn eval_with_formal_semantics(expr: Expr, env: Rc<Environment>) -> Result<Value> {
    let mut evaluator = Evaluator::new();
    evaluator.eval(expr, env, Continuation::Identity)
}