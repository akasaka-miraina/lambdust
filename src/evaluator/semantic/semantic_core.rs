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
    /// Purely functional evaluation without side effects
    pub fn eval_pure_functional(
        &self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        recursion_depth: usize,
    ) -> Result<Value> {
        // Stack overflow protection (functional approach)
        if recursion_depth > self.max_recursion_depth {
            return Err(LambdustError::stack_overflow_old());
        }

        #[cfg(debug_assertions)]
        if self.debug_tracer {
            DebugTracer::trace_expr(
                "evaluator::semantic",
                "eval_pure_functional",
                line!(),
                TraceLevel::INFO,
                "Evaluating expression".to_string(),
                &expr,
            );
        }

        // Continue with pure evaluation using recursion_depth parameter
        // Call the actual evaluation implementation
        match expr {
            Expr::Literal(lit) => {
                // Basic literal evaluation
                match lit {
                    crate::ast::Literal::Number(n) => Ok(Value::Number(n.clone())),
                    crate::ast::Literal::String(s) => Ok(Value::String(s.clone())),
                    crate::ast::Literal::Boolean(b) => Ok(Value::Boolean(b)),
                    crate::ast::Literal::Character(c) => Ok(Value::Character(c)),
                    crate::ast::Literal::Nil => Ok(Value::Nil),
                }
            },
            Expr::Variable(name) => {
                // Variable lookup
                env.get(&name).ok_or_else(|| {
                    crate::error::LambdustError::runtime_error(format!("Unbound variable: {}", name))
                })
            },
            Expr::List(elements) => {
                if elements.is_empty() {
                    Ok(Value::Nil)
                } else {
                    // Immutable application evaluation using existing infrastructure
                    if let Some(first_element) = elements.first() {
                        match first_element {
                            Expr::Variable(symbol) => {
                                // Check if this is a special form
                                match symbol.as_str() {
                                    "quote" => self.eval_quote_immutable(&elements[1..]),
                                    "if" => self.eval_if_immutable(&elements[1..], &env),
                                    "lambda" => self.eval_lambda_immutable(&elements[1..], &env),
                                    "define" => self.eval_define_immutable(&elements[1..], &env),
                                    _ => self.eval_application_immutable(&elements, &env),
                                }
                            },
                            _ => self.eval_application_immutable(&elements, &env),
                        }
                    } else {
                        Ok(Value::Nil)
                    }
                }
            },
            Expr::HygienicVariable(symbol) => {
                // Hygienic variable lookup (simplified)
                env.get(&symbol.name).ok_or_else(|| {
                    crate::error::LambdustError::runtime_error(format!("Unbound hygienic variable: {}", symbol.name))
                })
            },
            Expr::Quote(_quoted_expr) => {
                // Quote evaluation (placeholder - should convert expr to value)
                Ok(Value::Nil)
            },
            Expr::Quasiquote(_) => {
                // Quasiquote evaluation (placeholder)
                Ok(Value::Nil)
            },
            Expr::Vector(_) => {
                // Vector evaluation (placeholder)
                Ok(Value::Vector(Vec::new()))
            },
            _ => {
                // Catch-all for any remaining patterns
                Err(crate::error::LambdustError::runtime_error("Unsupported expression type in semantic evaluation".to_string()))
            }
        }
    }

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
    
    /// Immutable quote evaluation
    fn eval_quote_immutable(&self, args: &[Expr]) -> Result<Value> {
        if args.len() != 1 {
            return Err(crate::error::LambdustError::runtime_error("quote requires exactly one argument".to_string()));
        }
        // Convert expression to value for quote
        self.expr_to_value(&args[0])
    }
    
    /// Immutable if evaluation
    fn eval_if_immutable(&self, args: &[Expr], env: &Rc<Environment>) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            return Err(crate::error::LambdustError::runtime_error("if requires 2 or 3 arguments".to_string()));
        }
        
        let condition = self.eval_pure_functional(args[0].clone(), env.clone(), crate::evaluator::Continuation::Identity, 0)?;
        if self.is_truthy(&condition) {
            self.eval_pure_functional(args[1].clone(), env.clone(), crate::evaluator::Continuation::Identity, 0)
        } else if args.len() == 3 {
            self.eval_pure_functional(args[2].clone(), env.clone(), crate::evaluator::Continuation::Identity, 0)
        } else {
            Ok(Value::Undefined)
        }
    }
    
    /// Immutable lambda evaluation
    fn eval_lambda_immutable(&self, args: &[Expr], env: &Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(crate::error::LambdustError::runtime_error("lambda requires exactly 2 arguments".to_string()));
        }
        
        // Extract parameters and body
        let params = self.extract_parameters(&args[0])?;
        let body = args[1].clone();
        
        Ok(Value::Procedure(crate::value::Procedure::Lambda {
            params,
            body: vec![body],
            closure: env.clone(),
            variadic: false,
        }))
    }
    
    /// Immutable define evaluation (returns defined value)
    fn eval_define_immutable(&self, args: &[Expr], env: &Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(crate::error::LambdustError::runtime_error("define requires exactly 2 arguments".to_string()));
        }
        
        // In immutable context, just return the value that would be defined
        self.eval_pure_functional(args[1].clone(), env.clone(), crate::evaluator::Continuation::Identity, 0)
    }
    
    /// Immutable application evaluation
    fn eval_application_immutable(&self, elements: &[Expr], env: &Rc<Environment>) -> Result<Value> {
        if elements.is_empty() {
            return Ok(Value::Nil);
        }
        
        let operator = self.eval_pure_functional(elements[0].clone(), env.clone(), crate::evaluator::Continuation::Identity, 0)?;
        let operands: Result<Vec<Value>> = elements[1..]
            .iter()
            .map(|arg| self.eval_pure_functional(arg.clone(), env.clone(), crate::evaluator::Continuation::Identity, 0))
            .collect();
        let operands = operands?;
        
        // Apply operator to operands
        self.apply_immutable(operator, operands, env)
    }
    
    /// Helper method to convert expression to value
    fn expr_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => match lit {
                crate::ast::Literal::Number(n) => Ok(Value::Number(n.clone())),
                crate::ast::Literal::String(s) => Ok(Value::String(s.clone())),
                crate::ast::Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                crate::ast::Literal::Character(c) => Ok(Value::Character(*c)),
                crate::ast::Literal::Nil => Ok(Value::Nil),
            },
            Expr::Variable(s) => Ok(Value::Symbol(s.clone())),
            Expr::List(elements) => {
                let values: Result<Vec<Value>> = elements
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::from_vector(values?))
            },
            Expr::Vector(elements) => {
                let values: Result<Vec<Value>> = elements
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::Vector(values?))
            },
            _ => Err(crate::error::LambdustError::runtime_error("Cannot convert expression to value".to_string())),
        }
    }
    
    /// Helper method to check truthiness
    fn is_truthy(&self, value: &Value) -> bool {
        !matches!(value, Value::Boolean(false) | Value::Nil)
    }
    
    /// Helper method to extract parameters from lambda
    fn extract_parameters(&self, expr: &Expr) -> Result<Vec<String>> {
        match expr {
            Expr::Variable(s) => Ok(vec![s.clone()]),
            Expr::List(elements) => {
                elements
                    .iter()
                    .map(|e| match e {
                        Expr::Variable(s) => Ok(s.clone()),
                        _ => Err(crate::error::LambdustError::runtime_error("Invalid parameter in lambda".to_string())),
                    })
                    .collect()
            },
            Expr::Literal(crate::ast::Literal::Nil) => Ok(vec![]),
            _ => Err(crate::error::LambdustError::runtime_error("Invalid parameter list in lambda".to_string())),
        }
    }
    
    /// Immutable application of operator to operands
    fn apply_immutable(&self, operator: Value, operands: Vec<Value>, env: &Rc<Environment>) -> Result<Value> {
        match operator {
            Value::Procedure(proc) => {
                match proc {
                    crate::value::Procedure::Builtin { name, .. } => {
                        // Apply builtin function
                        self.apply_builtin_immutable(&name, operands, env)
                    },
                    crate::value::Procedure::Lambda { params, body, closure, variadic: _ } => {
                        // Create new environment with parameter bindings
                        if params.len() != operands.len() {
                            return Err(crate::error::LambdustError::runtime_error(
                                format!("Arity mismatch: expected {} arguments, got {}", params.len(), operands.len())
                            ));
                        }
                        
                        let mut new_env = (*closure).clone();
                        for (param, value) in params.iter().zip(operands.iter()) {
                            new_env.define(param.clone(), value.clone());
                        }
                        
                        // Evaluate all body expressions and return the last result
                        let env_rc = Rc::new(new_env);
                        let mut result = Value::Undefined;
                        for expr in body {
                            result = self.eval_pure_functional(expr.clone(), env_rc.clone(), crate::evaluator::Continuation::Identity, 0)?;
                        }
                        Ok(result)
                    },
                    _ => Err(crate::error::LambdustError::runtime_error("Cannot apply non-procedure".to_string())),
                }
            },
            _ => Err(crate::error::LambdustError::runtime_error("Cannot apply non-procedure".to_string())),
        }
    }
    
    /// Apply builtin function in immutable context
    fn apply_builtin_immutable(&self, name: &str, operands: Vec<Value>, _env: &Rc<Environment>) -> Result<Value> {
        match name {
            "+" => {
                let mut result = crate::lexer::SchemeNumber::Integer(0);
                for operand in operands {
                    match operand {
                        Value::Number(n) => {
                            result = match (&result, n) {
                                (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                    crate::lexer::SchemeNumber::Integer(a + b)
                                },
                                (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                    crate::lexer::SchemeNumber::Real(a + (b as f64))
                                },
                                (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Real(b)) => {
                                    crate::lexer::SchemeNumber::Real(*a as f64 + b)
                                },
                                (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Real(b)) => {
                                    crate::lexer::SchemeNumber::Real(a + b)
                                },
                                _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in +".to_string())),
                            };
                        },
                        _ => return Err(crate::error::LambdustError::runtime_error("+ requires numeric arguments".to_string())),
                    }
                }
                Ok(Value::Number(result))
            },
            "-" => {
                if operands.is_empty() {
                    return Err(crate::error::LambdustError::runtime_error("- requires at least 1 argument".to_string()));
                }
                
                if operands.len() == 1 {
                    // Unary negation
                    match &operands[0] {
                        Value::Number(n) => {
                            let negated = match n {
                                crate::lexer::SchemeNumber::Integer(i) => crate::lexer::SchemeNumber::Integer(-i),
                                crate::lexer::SchemeNumber::Real(r) => crate::lexer::SchemeNumber::Real(-r),
                                _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in unary -".to_string())),
                            };
                            Ok(Value::Number(negated))
                        },
                        _ => return Err(crate::error::LambdustError::runtime_error("- requires numeric arguments".to_string())),
                    }
                } else {
                    // Binary subtraction
                    let first = match &operands[0] {
                        Value::Number(n) => n.clone(),
                        _ => return Err(crate::error::LambdustError::runtime_error("- requires numeric arguments".to_string())),
                    };
                    
                    let mut result = first;
                    for operand in &operands[1..] {
                        match operand {
                            Value::Number(n) => {
                                result = match (&result, n) {
                                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                        crate::lexer::SchemeNumber::Integer(a - b)
                                    },
                                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                        crate::lexer::SchemeNumber::Real(a - (*b as f64))
                                    },
                                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Real(b)) => {
                                        crate::lexer::SchemeNumber::Real(*a as f64 - b)
                                    },
                                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Real(b)) => {
                                        crate::lexer::SchemeNumber::Real(a - b)
                                    },
                                    _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in -".to_string())),
                                };
                            },
                            _ => return Err(crate::error::LambdustError::runtime_error("- requires numeric arguments".to_string())),
                        }
                    }
                    Ok(Value::Number(result))
                }
            },
            "*" => {
                let mut result = crate::lexer::SchemeNumber::Integer(1);
                for operand in operands {
                    match operand {
                        Value::Number(n) => {
                            result = match (&result, n) {
                                (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                    crate::lexer::SchemeNumber::Integer(a * b)
                                },
                                (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                    crate::lexer::SchemeNumber::Real(a * (b as f64))
                                },
                                (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Real(b)) => {
                                    crate::lexer::SchemeNumber::Real(*a as f64 * b)
                                },
                                (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Real(b)) => {
                                    crate::lexer::SchemeNumber::Real(a * b)
                                },
                                _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in *".to_string())),
                            };
                        },
                        _ => return Err(crate::error::LambdustError::runtime_error("* requires numeric arguments".to_string())),
                    }
                }
                Ok(Value::Number(result))
            },
            "/" => {
                if operands.is_empty() {
                    return Err(crate::error::LambdustError::runtime_error("/ requires at least 1 argument".to_string()));
                }
                
                if operands.len() == 1 {
                    // Reciprocal (1/x)
                    match &operands[0] {
                        Value::Number(n) => {
                            let reciprocal = match n {
                                crate::lexer::SchemeNumber::Integer(i) => {
                                    if *i == 0 {
                                        return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                    }
                                    crate::lexer::SchemeNumber::Real(1.0 / (*i as f64))
                                },
                                crate::lexer::SchemeNumber::Real(r) => {
                                    if *r == 0.0 {
                                        return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                    }
                                    crate::lexer::SchemeNumber::Real(1.0 / r)
                                },
                                _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in /".to_string())),
                            };
                            Ok(Value::Number(reciprocal))
                        },
                        _ => return Err(crate::error::LambdustError::runtime_error("/ requires numeric arguments".to_string())),
                    }
                } else {
                    // Division
                    let first = match &operands[0] {
                        Value::Number(n) => n.clone(),
                        _ => return Err(crate::error::LambdustError::runtime_error("/ requires numeric arguments".to_string())),
                    };
                    
                    let mut result = first;
                    for operand in &operands[1..] {
                        match operand {
                            Value::Number(n) => {
                                result = match (&result, n) {
                                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                        if *b == 0 {
                                            return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                        }
                                        // Always convert to real for division to avoid integer division truncation
                                        crate::lexer::SchemeNumber::Real(*a as f64 / (*b as f64))
                                    },
                                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Integer(b)) => {
                                        if *b == 0 {
                                            return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                        }
                                        crate::lexer::SchemeNumber::Real(a / (*b as f64))
                                    },
                                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Real(b)) => {
                                        if *b == 0.0 {
                                            return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                        }
                                        crate::lexer::SchemeNumber::Real(*a as f64 / b)
                                    },
                                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Real(b)) => {
                                        if *b == 0.0 {
                                            return Err(crate::error::LambdustError::runtime_error("Division by zero".to_string()));
                                        }
                                        crate::lexer::SchemeNumber::Real(a / b)
                                    },
                                    _ => return Err(crate::error::LambdustError::runtime_error("Unsupported number type in /".to_string())),
                                };
                            },
                            _ => return Err(crate::error::LambdustError::runtime_error("/ requires numeric arguments".to_string())),
                        }
                    }
                    Ok(Value::Number(result))
                }
            },
            "=" => {
                if operands.len() < 2 {
                    return Err(crate::error::LambdustError::runtime_error("= requires at least 2 arguments".to_string()));
                }
                let first = &operands[0];
                for operand in &operands[1..] {
                    if !self.values_equal(first, operand) {
                        return Ok(Value::Boolean(false));
                    }
                }
                Ok(Value::Boolean(true))
            },
            "cons" => {
                if operands.len() != 2 {
                    return Err(crate::error::LambdustError::runtime_error("cons requires exactly 2 arguments".to_string()));
                }
                use crate::value::PairData;
                use std::rc::Rc;
                use std::cell::RefCell;
                Ok(Value::Pair(Rc::new(RefCell::new(PairData::new(operands[0].clone(), operands[1].clone())))))
            },
            "car" => {
                if operands.len() != 1 {
                    return Err(crate::error::LambdustError::runtime_error("car requires exactly 1 argument".to_string()));
                }
                match &operands[0] {
                    Value::Pair(pair_data) => Ok(pair_data.borrow().car.clone()),
                    Value::Nil => Err(crate::error::LambdustError::runtime_error("car: cannot take car of empty list".to_string())),
                    _ => Err(crate::error::LambdustError::runtime_error("car: not a pair".to_string())),
                }
            },
            "cdr" => {
                if operands.len() != 1 {
                    return Err(crate::error::LambdustError::runtime_error("cdr requires exactly 1 argument".to_string()));
                }
                match &operands[0] {
                    Value::Pair(pair_data) => Ok(pair_data.borrow().cdr.clone()),
                    Value::Nil => Err(crate::error::LambdustError::runtime_error("cdr: cannot take cdr of empty list".to_string())),
                    _ => Err(crate::error::LambdustError::runtime_error("cdr: not a pair".to_string())),
                }
            },
            _ => Err(crate::error::LambdustError::runtime_error(format!("Unknown builtin function: {}", name))),
        }
    }
    
    /// Helper method to compare values for equality
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                match (a, b) {
                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Integer(b)) => a == b,
                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Real(b)) => (a - b).abs() < f64::EPSILON,
                    (crate::lexer::SchemeNumber::Integer(a), crate::lexer::SchemeNumber::Real(b)) => ((*a as f64) - b).abs() < f64::EPSILON,
                    (crate::lexer::SchemeNumber::Real(a), crate::lexer::SchemeNumber::Integer(b)) => (a - (*b as f64)).abs() < f64::EPSILON,
                    _ => false,
                }
            },
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Default for SemanticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}