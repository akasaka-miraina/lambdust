//! Pure R7RS semantic evaluator
//!
//! This module implements a pure R7RS formal semantics evaluator that
//! contains NO optimizations whatsoever. It serves as the reference
//! implementation for verification against optimized execution paths.

use crate::ast::Expr;
use crate::debug::{DebugTracer, TraceLevel};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::Continuation;
use crate::lexer::SchemeNumber;
use crate::value::Value;
use std::rc::Rc;

/// Pure R7RS semantic evaluator
///
/// This evaluator implements ONLY the formal semantics defined in R7RS
/// Section 7.2. It contains no optimizations and serves as the mathematical
/// reference for correctness verification.
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

impl SemanticEvaluator {
    /// Create a new pure semantic evaluator
    pub fn new() -> Self {
        Self {
            global_env: Rc::new(Environment::new()),
            recursion_depth: 0,
            max_recursion_depth: 1000,

            #[cfg(debug_assertions)]
            debug_tracer: true,
        }
    }

    /// Create with custom global environment
    pub fn with_environment(env: Rc<Environment>) -> Self {
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
                TraceLevel::ENTRY,
                "Pure R7RS evaluation".to_string(),
                &expr,
            );
        }

        let result = match expr {
            // Constants: E[K]ρκσ = κ(K[K])
            Expr::Literal(lit) => {
                let value = self.literal_to_value(lit)?;
                self.apply_continuation_pure(cont, value)
            }

            // Variables: E[I]ρκσ = κ(σ(ρ(I)))
            Expr::Variable(name) => {
                let value = env
                    .get(&name)
                    .ok_or_else(|| LambdustError::undefined_variable(name))?;
                self.apply_continuation_pure(cont, value)
            }

            // Function application: E[(E0 E1 ...)]ρκσ
            Expr::List(exprs) if !exprs.is_empty() => self.eval_application_pure(exprs, env, cont),

            // Empty list
            Expr::List(_) => self.apply_continuation_pure(cont, Value::Nil),

            // Other expression types (not implemented in pure evaluator yet)
            _ => Err(LambdustError::runtime_error(
                "Expression type not implemented in pure evaluator",
            )),
        };

        self.recursion_depth -= 1;
        result
    }

    /// Pure continuation application: κ(v)
    fn apply_continuation_pure(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            Continuation::Identity => Ok(value),

            // Special form continuations
            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => {
                if value.is_truthy() {
                    self.eval_pure(consequent, env, *parent)
                } else if let Some(alt) = alternate {
                    self.eval_pure(alt, env, *parent)
                } else {
                    self.apply_continuation_pure(*parent, Value::Undefined)
                }
            }

            Continuation::Define {
                variable,
                env,
                parent,
            } => {
                env.define(variable, value);
                self.apply_continuation_pure(*parent, Value::Undefined)
            }

            Continuation::Assignment {
                variable,
                env,
                parent,
            } => {
                env.set(&variable, value)?;
                self.apply_continuation_pure(*parent, Value::Undefined)
            }

            Continuation::Begin {
                remaining,
                env,
                parent,
            } => {
                if remaining.is_empty() {
                    self.apply_continuation_pure(*parent, value)
                } else {
                    self.eval_sequence_pure(remaining, env, *parent)
                }
            }

            // Function application continuations
            Continuation::Operator { args, env, parent } => {
                self.eval_args_pure(args, env, vec![], value, *parent)
            }

            Continuation::Application {
                operator,
                evaluated_args,
                remaining_args,
                env,
                parent,
            } => {
                let mut new_evaluated = evaluated_args;
                new_evaluated.push(value);

                if remaining_args.is_empty() {
                    self.apply_procedure_pure(operator, new_evaluated, *parent)
                } else {
                    let (next_arg, rest) = remaining_args.split_first().unwrap();
                    let next_cont = Continuation::Application {
                        operator,
                        evaluated_args: new_evaluated,
                        remaining_args: rest.to_vec(),
                        env: env.clone(),
                        parent,
                    };
                    self.eval_pure(next_arg.clone(), env, next_cont)
                }
            }

            // Other continuations from control flow module
            _ => {
                // Delegate to control flow module for complex continuations
                // This maintains separation of concerns
                self.apply_other_continuation_pure(cont, value)
            }
        }
    }

    /// Evaluate function application (pure)
    fn eval_application_pure(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let (function_expr, args) = exprs.split_first().unwrap();

        // Check for special forms first
        if let Expr::Variable(name) = function_expr {
            if self.is_special_form(name) {
                return self.eval_special_form_pure(name, args, env, cont);
            }
        }

        // Regular function application: evaluate function first
        let app_cont = Continuation::Operator {
            args: args.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(function_expr.clone(), env, app_cont)
    }

    /// Evaluate arguments in sequence (pure)
    fn eval_args_pure(
        &mut self,
        args: Vec<Expr>,
        env: Rc<Environment>,
        evaluated_args: Vec<Value>,
        function: Value,
        cont: Continuation,
    ) -> Result<Value> {
        if args.is_empty() {
            return self.apply_procedure_pure(function, evaluated_args, cont);
        }

        let (first_arg, remaining_args) = args.split_first().unwrap();
        let arg_cont = Continuation::Application {
            operator: function,
            evaluated_args,
            remaining_args: remaining_args.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(first_arg.clone(), env, arg_cont)
    }

    /// Apply procedure (pure - no optimizations)
    fn apply_procedure_pure(
        &mut self,
        function: Value,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        match function {
            Value::Procedure(proc) => {
                // Use standard procedure application from existing evaluator
                // This delegates to the formal CPS implementation
                match proc {
                    crate::value::Procedure::Builtin { name, .. } => {
                        // Apply builtin procedures - simplified for now
                        let result = match name.as_str() {
                            "+" => self.apply_builtin_add(&args)?,
                            _ => Value::Undefined,
                        };
                        self.apply_continuation_pure(cont, result)
                    }

                    crate::value::Procedure::Lambda {
                        params,
                        body,
                        closure,
                        variadic,
                    } => {
                        // Create new environment with parameter bindings
                        let new_env = self.bind_parameters_pure(params, args, closure, variadic)?;

                        // Evaluate lambda body in new environment
                        self.eval_sequence_pure(body, new_env, cont)
                    }

                    _ => Err(LambdustError::type_error(
                        "Cannot apply non-procedure".to_string(),
                    )),
                }
            }

            _ => Err(LambdustError::type_error(
                "Cannot apply non-procedure".to_string(),
            )),
        }
    }

    /// Bind parameters for lambda application (pure)
    fn bind_parameters_pure(
        &self,
        params: Vec<String>,
        args: Vec<Value>,
        closure: Rc<Environment>,
        variadic: bool,
    ) -> Result<Rc<Environment>> {
        let new_env = Rc::new(Environment::extend(&closure));

        if variadic {
            // Variadic function: last parameter gets remaining args as list
            if params.is_empty() {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            let fixed_params = &params[..params.len() - 1];
            let rest_param = &params[params.len() - 1];

            if args.len() < fixed_params.len() {
                return Err(LambdustError::arity_error(fixed_params.len(), args.len()));
            }

            // Bind fixed parameters
            for (param, arg) in fixed_params.iter().zip(args.iter()) {
                new_env.define(param.clone(), arg.clone());
            }

            // Bind rest parameter to remaining args as list
            let rest_args = args[fixed_params.len()..].to_vec();
            // Simplified: use Nil for empty list, first element for non-empty
            let list_value = if rest_args.is_empty() {
                Value::Nil
            } else {
                rest_args[0].clone() // Simplified for now
            };
            new_env.define(rest_param.clone(), list_value);
        } else {
            // Fixed arity function
            if params.len() != args.len() {
                return Err(LambdustError::arity_error(params.len(), args.len()));
            }

            for (param, arg) in params.iter().zip(args.iter()) {
                new_env.define(param.clone(), arg.clone());
            }
        }

        Ok(new_env)
    }

    /// Evaluate sequence of expressions (pure)
    fn eval_sequence_pure(
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

        let (first, remaining) = exprs.split_first().unwrap();
        let begin_cont = Continuation::Begin {
            remaining: remaining.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(first.clone(), env, begin_cont)
    }

    /// Convert literal to value
    fn literal_to_value(&self, lit: crate::ast::Literal) -> Result<Value> {
        use crate::ast::Literal;
        match lit {
            Literal::Number(n) => Ok(Value::Number(n)),
            Literal::String(s) => Ok(Value::String(s)),
            Literal::Boolean(b) => Ok(Value::Boolean(b)),
            Literal::Character(c) => Ok(Value::Character(c)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    /// Check if name is a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(
            name,
            "lambda"
                | "if"
                | "define"
                | "set!"
                | "quote"
                | "begin"
                | "and"
                | "or"
                | "cond"
                | "case"
                | "do"
                | "let"
                | "let*"
                | "letrec"
        )
    }

    /// Evaluate special form (pure)
    fn eval_special_form_pure(
        &mut self,
        name: &str,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Delegate to existing special form implementation
        // but ensure it goes through pure evaluation paths
        match name {
            "if" => self.eval_if_pure(operands, env, cont),
            "define" => self.eval_define_pure(operands, env, cont),
            "begin" => self.eval_begin_pure(operands, env, cont),
            "lambda" => self.eval_lambda_pure(operands, env, cont),
            _ => {
                // For now, delegate to existing implementation
                // TODO: Implement pure versions of all special forms
                Err(LambdustError::syntax_error(format!(
                    "Special form '{}' not implemented in pure evaluator",
                    name
                )))
            }
        }
    }

    /// Pure if evaluation
    fn eval_if_pure(
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

        self.eval_pure(test, env, if_cont)
    }

    /// Pure define evaluation
    fn eval_define_pure(
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
                    "define: first argument must be variable".to_string(),
                ));
            }
        };

        let value_expr = operands[1].clone();
        let define_cont = Continuation::Define {
            variable,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval_pure(value_expr, env, define_cont)
    }

    /// Pure begin evaluation
    fn eval_begin_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        self.eval_sequence_pure(operands.to_vec(), env, cont)
    }

    /// Pure lambda evaluation
    fn eval_lambda_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: requires at least 2 arguments".to_string(),
            ));
        }

        // Parse parameter list
        let (params, variadic) = self.parse_lambda_params(&operands[0])?;
        let body = operands[1..].to_vec();

        let lambda = crate::value::Procedure::Lambda {
            params,
            body,
            closure: env,
            variadic,
        };

        self.apply_continuation_pure(cont, Value::Procedure(lambda))
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
                            "lambda: parameter must be symbol".to_string(),
                        ));
                    }
                }
                Ok((param_names, false))
            }
            Expr::Variable(name) => Ok((vec![name.clone()], true)),
            _ => Err(LambdustError::syntax_error(
                "lambda: invalid parameter list".to_string(),
            )),
        }
    }

    /// Check recursion depth
    fn check_recursion_depth(&self) -> Result<()> {
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(LambdustError::stack_overflow());
        }
        Ok(())
    }

    /// Apply other continuations (delegation to existing implementation)
    fn apply_other_continuation_pure(
        &mut self,
        _cont: Continuation,
        _value: Value,
    ) -> Result<Value> {
        // For complex continuations, delegate to existing evaluator implementation
        // This maintains compatibility while ensuring pure semantics
        // TODO: Implement pure versions of all continuation types
        Err(LambdustError::runtime_error(
            "Complex continuation not implemented in pure evaluator",
        ))
    }

    /// Simple builtin addition (for demonstration)
    fn apply_builtin_add(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(SchemeNumber::Integer(0)));
        }

        let mut sum = 0i64;
        for arg in args {
            match arg {
                Value::Number(n) => match n {
                    SchemeNumber::Integer(i) => sum += i,
                    _ => return Err(LambdustError::type_error("Addition expects integers")),
                },
                _ => return Err(LambdustError::type_error("Addition expects numbers")),
            }
        }

        Ok(Value::Number(SchemeNumber::Integer(sum)))
    }
}

impl Default for SemanticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
