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
pub use continuation::{Continuation, DynamicPoint, LightContinuation};
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
            return Err(LambdustError::syntax_error("Empty application".to_string()));
        }

        // Try to handle special forms first
        if let Some(special_result) =
            self.try_eval_special_form(&exprs, env.clone(), cont.clone())?
        {
            return Ok(special_result);
        }

        // Regular function application: evaluate operator first
        let (operator_expr, args) = exprs.split_first().unwrap();

        let operator_cont = Continuation::Operator {
            args: args.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(operator_expr.clone(), env, operator_cont)
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
                return Ok(Some(self.eval_known_special_form(
                    name,
                    &exprs[1..],
                    env,
                    cont,
                )?));
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
                let values: Result<Vec<Value>> =
                    exprs.into_iter().map(Self::expr_to_value).collect();
                Ok(Value::from_vector(values?))
            }
            Expr::Quote(expr) => Self::expr_to_value(*expr),
            Expr::DottedList(elements, tail) => {
                // Handle dotted list: (a b . c) -> cons(a, cons(b, c))
                let mut result = Self::expr_to_value(*tail)?;
                for expr in elements.into_iter().rev() {
                    let value = Self::expr_to_value(expr)?;
                    result = Value::cons(value, result);
                }
                Ok(result)
            }
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
        expr: Expr,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // For basic quasiquote without unquote/unquote-splicing,
        // it's equivalent to quote
        let value = Self::expr_to_value(expr)?;
        self.apply_continuation(cont, value)
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

        let (first_expr, remaining) = exprs.split_first().unwrap();

        let vector_cont = Continuation::VectorEval {
            evaluated_elements: Vec::new(),
            remaining_elements: remaining.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr.clone(), env, vector_cont)
    }

    /// Apply continuation: κ(v)
    pub fn apply_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Performance optimization: Try lightweight continuation first
        if let Some(light_cont) = LightContinuation::from_continuation(&cont) {
            return light_cont.apply(value);
        }

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
            Continuation::ValuesAccumulate {
                remaining_exprs,
                mut accumulated_values,
                env,
                parent,
            } => {
                // Add current value to accumulated values
                accumulated_values.push(value);

                if remaining_exprs.is_empty() {
                    // All expressions evaluated, create Values result
                    self.apply_continuation(*parent, Value::Values(accumulated_values))
                } else {
                    // Continue evaluating remaining expressions
                    let (next_expr, remaining) = remaining_exprs.split_first().unwrap();

                    let next_cont = Continuation::ValuesAccumulate {
                        remaining_exprs: remaining.to_vec(),
                        accumulated_values,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr.clone(), env, next_cont)
                }
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
                    let (next_expr, remaining) = remaining_elements.split_first().unwrap();

                    let vector_cont = Continuation::VectorEval {
                        evaluated_elements,
                        remaining_elements: remaining.to_vec(),
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr.clone(), env, vector_cont)
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
            let (next_arg, remaining) = remaining_args.split_first().unwrap();

            let app_cont = Continuation::Application {
                operator,
                evaluated_args,
                remaining_args: remaining.to_vec(),
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(next_arg.clone(), env, app_cont)
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
                let (first_arg, remaining) = args.split_first().unwrap();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining.to_vec(),
                    env: env.clone(),
                    parent: Box::new(parent),
                };

                self.eval(first_arg.clone(), env, app_cont)
            }
            EvalOrder::RightToLeft => {
                // Evaluate from right to left
                let (last_arg, remaining) = args.split_last().unwrap();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining.to_vec(),
                    env: env.clone(),
                    parent: Box::new(parent),
                };

                self.eval(last_arg.clone(), env, app_cont)
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

                    // Evaluate body
                    self.eval_sequence(body, Rc::new(lambda_env), cont)
                }
                Procedure::Continuation {
                    continuation: _captured_cont,
                } => {
                    // Apply captured continuation (basic escape implementation)
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }

                    // Basic escape: return the value directly to the captured continuation
                    // This implements a simplified form of non-local exit
                    // A full implementation would need to properly restore the captured continuation state
                    Ok(args[0].clone())
                }
                Procedure::CapturedContinuation {
                    continuation: captured_cont,
                } => {
                    // Apply captured continuation from evaluator
                    // R7RS allows continuations to be called with multiple values,
                    // but only the first value is used (others are ignored)
                    // If no arguments are provided, use #<void> as the default value
                    let escape_value = if args.is_empty() {
                        Value::Undefined
                    } else {
                        args[0].clone()
                    };

                    // Apply the captured continuation with complete non-local exit
                    // Use only the first argument, following R7RS semantics
                    self.apply_captured_continuation_with_non_local_exit(
                        *captured_cont.clone(),
                        escape_value,
                    )
                }
                Procedure::ReusableContinuation {
                    continuation: captured_cont,
                    capture_env,
                    is_escaping,
                    ..
                } => {
                    // Handle reusable continuation (for both escape and reuse)
                    let escape_value = if args.is_empty() {
                        Value::Undefined
                    } else {
                        args[0].clone()
                    };

                    // Simple heuristic: if current continuation has CallCc, use escape semantics
                    // This handles the common case of call/cc immediate escape
                    let is_escape_context = matches!(cont, Continuation::CallCc { .. }) || is_escaping;
                    
                    if is_escape_context {
                        // Use escape semantics (skip intermediate computations)
                        self.apply_captured_continuation_with_non_local_exit(
                            *captured_cont.clone(),
                            escape_value,
                        )
                    } else {
                        // Use reuse semantics (preserve computation context)
                        self.apply_reusable_continuation_with_context(
                            *captured_cont.clone(),
                            capture_env.clone(),
                            escape_value,
                            cont,
                        )
                    }
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
            | Continuation::Or { .. } => self.apply_special_form_continuation(cont, value),
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
            return Err(LambdustError::syntax_error(
                "No expressions to evaluate".to_string(),
            ));
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
        self.apply_procedure(
            procedure,
            args,
            self.global_env.clone(),
            Continuation::Identity,
        )
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

    /// Apply captured continuation with complete non-local exit
    /// This provides true call/cc behavior by completely abandoning the current
    /// continuation chain and jumping directly to the captured continuation
    fn apply_captured_continuation_with_non_local_exit(
        &mut self,
        captured_cont: Continuation,
        escape_value: Value,
    ) -> Result<Value> {
        // Perform complete non-local exit by recursively skipping ALL intermediate
        // computations until we reach the true capture point
        self.apply_captured_continuation_complete_exit(captured_cont, escape_value)
    }

    /// Recursively skip all intermediate computations to implement complete non-local exit
    fn apply_captured_continuation_complete_exit(
        &mut self,
        captured_cont: Continuation,
        escape_value: Value,
    ) -> Result<Value> {
        match captured_cont {
            // For CallCc continuation, skip to its parent (the capture point)
            Continuation::CallCc { parent, .. } => {
                // This is where call/cc was originally called, so we apply the parent
                // continuation with the escape value
                self.apply_continuation(*parent, escape_value)
            }
            // For Application continuations, we need to distinguish between:
            // 1. call/cc escape (should skip all intermediate computation)
            // 2. captured continuation reuse (should preserve computation context)
            // 
            // The fundamental issue is that both cases look the same at this point.
            // For now, implement proper escape behavior by skipping Application continuations.
            // This means continuation reuse might not work correctly for certain cases,
            // but call/cc escape semantics will be preserved.
            Continuation::Application { parent, .. } => {
                // Skip the Application and continue up the chain
                // This implements proper call/cc escape semantics
                self.apply_captured_continuation_complete_exit(*parent, escape_value)
            }
            // For other intermediate computation continuations, skip them entirely
            cont if cont.is_intermediate_computation() => {
                if let Some(parent) = cont.parent() {
                    // Recursively skip up the chain until we find a non-intermediate continuation
                    self.apply_captured_continuation_complete_exit(parent.clone(), escape_value)
                } else {
                    // If we reach the top with no parent, return the escape value directly
                    Ok(escape_value)
                }
            }
            // For non-intermediate continuations (like Identity, Define, etc.),
            // apply them normally as they represent valid continuation points
            _ => self.apply_continuation(captured_cont, escape_value),
        }
    }

    /// Apply reusable continuation with context preservation (for continuation reuse)
    fn apply_reusable_continuation_with_context(
        &mut self,
        captured_cont: Continuation,
        _capture_env: Rc<Environment>,
        value: Value,
        _current_cont: Continuation,
    ) -> Result<Value> {
        // For continuation reuse, we need to preserve the computation context
        // instead of performing a complete escape
        match captured_cont {
            // For CallCc continuation, apply the value in the captured environment
            Continuation::CallCc { parent, .. } => {
                // Restore the capture environment and apply the parent continuation
                self.apply_continuation(*parent, value)
            }
            // For Application continuations, we preserve the context
            Continuation::Application { 
                operator,
                evaluated_args,
                remaining_args,
                env,
                parent 
            } => {
                // Build new application with the value inserted in the captured context
                // This enables proper continuation reuse semantics
                let mut new_args = evaluated_args;
                new_args.push(value);
                
                if remaining_args.is_empty() {
                    // All arguments are ready, apply the operator
                    self.apply_procedure(operator, new_args, env, *parent)
                } else {
                    // Continue evaluating remaining arguments
                    let next_arg = &remaining_args[0];
                    let remaining = remaining_args[1..].to_vec();
                    
                    let app_cont = Continuation::Application {
                        operator,
                        evaluated_args: new_args,
                        remaining_args: remaining,
                        env: env.clone(),
                        parent,
                    };
                    
                    self.eval(next_arg.clone(), env, app_cont)
                }
            }
            // For other continuations, apply them normally
            _ => self.apply_continuation(captured_cont, value),
        }
    }

}

/// Public API for evaluation
pub fn eval_with_formal_semantics(expr: Expr, env: Rc<Environment>) -> Result<Value> {
    let mut evaluator = Evaluator::new();
    evaluator.eval(expr, env, Continuation::Identity)
}
