//! Core functionality for pure R7RS semantic evaluator
//!
//! This module contains the main evaluation logic and core data structures
//! for the pure semantic evaluator.

use crate::ast::{Expr, Literal};
use crate::debug::{DebugTracer, TraceLevel};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::Continuation;
// Removed unused import: use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use std::rc::Rc;

/// Pure R7RS semantic evaluator
///
/// This evaluator implements ONLY the formal semantics defined in R7RS
/// Section 7.2. It contains no optimizations and serves as the mathematical
/// reference for correctness verification.
#[derive(Debug, Clone)]
pub struct SemanticEvaluator {
    /// Global environment containing R7RS standard library
    #[allow(dead_code)]
    global_env: Rc<Environment>,

    /// Recursion depth monitoring (stack overflow prevention)
    recursion_depth: usize,
    max_recursion_depth: usize,

    /// Debug tracer (disabled in release builds)
    #[cfg(debug_assertions)]
    debug_tracer: bool,
}

/// Statistics for S-expression reductions in pure semantic evaluation
#[derive(Debug, Default, Clone)]
pub struct ReductionStats {
    /// Number of beta reductions (lambda applications)
    pub beta_reductions: usize,
    /// Number of constant folding operations
    pub constant_folds: usize,
    /// Number of conditional reductions (if with constant test)
    pub conditional_reductions: usize,
    /// Number of identity reductions (+0, *1, and #t, or #f)
    pub identity_reductions: usize,
    /// Total expressions analyzed for reduction
    pub expressions_analyzed: usize,
    /// Total expressions successfully reduced
    pub expressions_reduced: usize,
}

impl SemanticEvaluator {
    /// Create a new pure semantic evaluator
    #[must_use] pub fn new() -> Self {
        Self {
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,

            #[cfg(debug_assertions)]
            debug_tracer: true,
        }
    }


    /// Create with custom global environment
    #[must_use] pub fn with_environment(env: Rc<Environment>) -> Self {
        Self {
            global_env: env,
            recursion_depth: 0,
            max_recursion_depth: 1000,

            #[cfg(debug_assertions)]
            debug_tracer: true,
        }
    }

    /// R7RS formal semantics evaluation: E[e]ρκσ
    ///
    /// This function implements the exact formal semantics from R7RS
    /// without any optimizations or shortcuts.
    ///
    /// - e: expression to evaluate
    /// - ρ: environment (variable bindings)  
    /// - κ: continuation
    /// - σ: store (implicit in Rust's memory model)
    pub fn eval_pure(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Stack overflow protection
        self.check_recursion_depth()?;
        self.recursion_depth += 1;

        #[cfg(debug_assertions)]
        if self.debug_tracer {
            DebugTracer::trace_expr(
                "evaluator::semantic",
                "eval_pure",
                line!(),
                TraceLevel::INFO,
                "Evaluating expression".to_string(),
                &expr,
            );
        }

        let result = match expr {
            // Literals evaluate to themselves
            Expr::Literal(lit) => {
                let value = self.literal_to_value(lit)?;
                self.apply_continuation_pure(cont, value)
            }

            // Variables lookup in environment
            Expr::Variable(name) => {
                if self.is_special_form(&name) {
                    return Err(LambdustError::syntax_error(format!(
                        "Special form '{name}' used as variable"
                    )));
                }

                match env.get(&name) {
                    Some(value) => self.apply_continuation_pure(cont, value),
                    None => Err(LambdustError::runtime_error(format!(
                        "Undefined variable: {name}"
                    ))),
                }
            }

            // Hygienic variables (from macro expansion)
            Expr::HygienicVariable(symbol) => {
                match env.get(symbol.original_name()) {
                    Some(value) => self.apply_continuation_pure(cont, value),
                    None => Err(LambdustError::runtime_error(format!(
                        "Undefined hygienic variable: {}",
                        symbol.unique_name()
                    ))),
                }
            }

            // Function application and special forms
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "Cannot evaluate empty list".to_string(),
                    ));
                }

                match &exprs[0] {
                    Expr::Variable(name) if self.is_special_form(name) => {
                        self.eval_special_form_pure(name, &exprs[1..], env, cont)
                    }
                    _ => self.eval_application_pure(exprs, env, cont),
                }
            }

            // Quote expressions (not supported in pure semantic evaluation)
            Expr::Quote(_) => {
                Err(LambdustError::type_error(
                    "Quote expressions not supported in pure semantic evaluation"
                ))
            }
            Expr::Quasiquote(_) => {
                Err(LambdustError::type_error(
                    "Quasiquote expressions not supported in pure semantic evaluation"
                ))
            }
            Expr::Unquote(_) => {
                Err(LambdustError::type_error(
                    "Unquote expressions not supported in pure semantic evaluation"
                ))
            }
            Expr::UnquoteSplicing(_) => {
                Err(LambdustError::type_error(
                    "Unquote-splicing expressions not supported in pure semantic evaluation"
                ))
            }
            Expr::Vector(_) => {
                Err(LambdustError::type_error(
                    "Vector expressions not supported in pure semantic evaluation"
                ))
            }
            Expr::DottedList(_, _) => {
                Err(LambdustError::type_error(
                    "Dotted list expressions not supported in pure semantic evaluation"
                ))
            }
        };

        self.recursion_depth -= 1;
        result
    }

    /// Check if name is a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(
            name,
            "if" | "define"
                | "set!"
                | "lambda"
                | "quote"
                | "quasiquote"
                | "unquote"
                | "unquote-splicing"
                | "let"
                | "let*"
                | "letrec"
                | "begin"
                | "cond"
                | "case"
                | "and"
                | "or"
                | "when"
                | "unless"
                | "do"
        )
    }

    /// Evaluate function application (pure)
    fn eval_application_pure(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let operator = exprs[0].clone();
        let args = exprs[1..].to_vec();

        let operator_cont = Continuation::Operator {
            args,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(operator, env, operator_cont)
    }

    /// Evaluate arguments for function application (pure)
    pub(super) fn eval_args_pure(
        &mut self,
        args: Vec<Expr>,
        env: Rc<Environment>,
        evaluated_args: Vec<Value>,
        operator: Value,
        cont: Continuation,
    ) -> Result<Value> {
        if args.is_empty() {
            return self.apply_procedure_pure(operator, evaluated_args, cont);
        }

        let (first_arg, remaining_args) = args.split_first().unwrap();
        let arg_cont = Continuation::Application {
            operator,
            evaluated_args,
            remaining_args: remaining_args.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(first_arg.clone(), env, arg_cont)
    }

    /// Apply procedure in pure evaluation
    pub(super) fn apply_procedure_pure(
        &mut self,
        procedure: Value,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        match procedure {
            Value::Procedure(proc) => match proc {
                Procedure::Lambda {
                    params,
                    body,
                    closure,
                    variadic,
                } => {
                    let new_env = self.bind_parameters_pure(params, args, closure, variadic)?;
                    self.eval_sequence_pure(body, new_env, cont)
                }
                Procedure::Builtin { name: _, arity: _, func } => {
                    let result = func(&args)?;
                    self.apply_continuation_pure(cont, result)
                }
                Procedure::HostFunction { name, arity: _, func: _ } => {
                    // Host functions not supported in pure semantic evaluation
                    Err(LambdustError::type_error(format!(
                        "Host function '{name}' not supported in pure evaluation"
                    )))
                }
                Procedure::Continuation { continuation: _ } => {
                    // Continuations require special handling beyond pure evaluation
                    Err(LambdustError::type_error(
                        "Continuation procedures not supported in pure evaluation"
                    ))
                }
                Procedure::CapturedContinuation { continuation: _ } => {
                    // Captured continuations require special handling
                    Err(LambdustError::type_error(
                        "Captured continuation procedures not supported in pure evaluation"
                    ))
                }
                Procedure::ReusableContinuation { continuation: _, capture_env: _, reuse_id: _, is_escaping: _ } => {
                    // Reusable continuations require special handling
                    Err(LambdustError::type_error(
                        "Reusable continuation procedures not supported in pure evaluation"
                    ))
                }
            },
            _ => Err(LambdustError::type_error(
                "Cannot apply non-procedure value",
            )),
        }
    }

    /// Bind parameters for lambda application (pure)
    fn bind_parameters_pure(
        &self,
        params: Vec<String>,
        args: Vec<Value>,
        env: Rc<Environment>,
        is_variadic: bool,
    ) -> Result<Rc<Environment>> {
        let new_env = Environment::with_parent(env);

        if is_variadic {
            if args.len() < params.len().saturating_sub(1) {
                return Err(LambdustError::arity_error_min(
                    params.len().saturating_sub(1),
                    args.len(),
                ));
            }

            // Bind fixed parameters
            for (i, param) in params.iter().take(params.len().saturating_sub(1)).enumerate() {
                new_env.define(param.clone(), args[i].clone());
            }

            // Bind variadic parameter to list of remaining arguments
            if let Some(last_param) = params.last() {
                let remaining_args = args[params.len().saturating_sub(1)..]
                    .iter()
                    .fold(Value::Nil, |acc, arg| {
                        Value::cons(arg.clone(), acc)
                    });
                new_env.define(last_param.clone(), remaining_args);
            }
        } else {
            if args.len() != params.len() {
                return Err(LambdustError::arity_error(params.len(), args.len()));
            }

            for (param, arg) in params.into_iter().zip(args) {
                new_env.define(param, arg);
            }
        }

        Ok(Rc::new(new_env))
    }

    /// Evaluate sequence of expressions (pure)
    pub(super) fn eval_sequence_pure(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_continuation_pure(cont, Value::Undefined);
        }

        if exprs.len() == 1 {
            return self.eval_pure(exprs[0].clone(), env, cont);
        }

        let (first, rest) = exprs.split_first().unwrap();
        let sequence_cont = Continuation::Begin {
            remaining: rest.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(first.clone(), env, sequence_cont)
    }

    /// Convert literal to value
    fn literal_to_value(&self, lit: Literal) -> Result<Value> {
        match lit {
            Literal::Number(n) => Ok(Value::Number(n)),
            Literal::String(s) => Ok(Value::String(s)),
            Literal::Boolean(b) => Ok(Value::Boolean(b)),
            Literal::Character(c) => Ok(Value::Character(c)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    /// Check recursion depth to prevent stack overflow
    fn check_recursion_depth(&self) -> Result<()> {
        if self.recursion_depth >= self.max_recursion_depth {
            Err(LambdustError::runtime_error(
                "Stack overflow: maximum recursion depth exceeded".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Apply other continuation types (delegated to control flow)
    pub fn apply_other_continuation_pure(
        &mut self,
        _cont: Continuation,
        value: Value,
    ) -> Result<Value> {
        // Placeholder implementation for complex continuations
        // These would be handled by the control flow module
        Ok(value)
    }
}

impl Default for SemanticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}