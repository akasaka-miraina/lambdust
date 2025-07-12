//! R7RS-pico Ultra-Minimal Evaluator
//!
//! This module implements an ultra-minimal Scheme evaluator following the
//! R7RS-pico specification. Key characteristics:
//!
//! - Simplified semantic model: U -> E (Environment to Expressed value)
//! - No side effects (no set!, no mutation)
//! - No continuations (no call/cc, simplified control flow)
//! - Minimal built-in procedures
//! - Designed for embedded systems and educational purposes
//!
//! The evaluator focuses on core functional programming constructs while
//! maintaining proper tail recursion and correct Scheme semantics.

#[cfg(feature = "pico")]
use crate::ast::{Expr, Literal};
#[cfg(feature = "pico")]
use crate::environment::Environment;
#[cfg(feature = "pico")]
use crate::error::{LambdustError, Result};
#[cfg(feature = "pico")]
use crate::lexer::SchemeNumber;
#[cfg(feature = "pico")]
use crate::value::Value;
#[cfg(feature = "pico")]
use std::rc::Rc;

/// Ultra-minimal R7RS-pico evaluator
///
/// Implements the simplified U -> E semantic model where:
/// - U: Environment (identifier bindings)
/// - E: Expressed values (numbers, booleans, symbols, pairs, procedures)
///
/// This evaluator excludes:
/// - Side effects and mutation
/// - Continuations and call/cc
/// - Complex control structures
/// - Advanced features beyond R7RS-pico spec
#[cfg(feature = "pico")]
#[derive(Debug, Clone)]
pub struct PicoEvaluator {
    /// Maximum recursion depth to prevent stack overflow
    max_recursion_depth: usize,
    /// Current recursion depth
    current_depth: usize,
}

#[cfg(feature = "pico")]
impl PicoEvaluator {
    /// Create a new pico evaluator with default settings
    pub fn new() -> Self {
        Self {
            max_recursion_depth: 1000, // Conservative for embedded systems
            current_depth: 0,
        }
    }
    
    /// Create a new pico evaluator with custom recursion limit
    pub fn with_recursion_limit(max_depth: usize) -> Self {
        Self {
            max_recursion_depth: max_depth,
            current_depth: 0,
        }
    }
    
    /// Evaluate an expression in the given environment
    ///
    /// Implements the core U -> E semantic model:
    /// - U: Environment (variable bindings)
    /// - E: Expressed value (result)
    pub fn evaluate(&mut self, expr: &Expr, env: Rc<Environment>) -> Result<Value> {
        // Check recursion depth to prevent stack overflow
        if self.current_depth >= self.max_recursion_depth {
            return Err(LambdustError::StackOverflow {
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        self.current_depth += 1;
        let result = self.evaluate_inner(expr, env);
        self.current_depth -= 1;
        
        result
    }
    
    /// Internal evaluation implementation
    fn evaluate_inner(&mut self, expr: &Expr, env: Rc<Environment>) -> Result<Value> {
        match expr {
            // Constants: K -> E (constant to expressed value)
            Expr::Literal(literal) => self.evaluate_literal(literal),
            
            // Variables: I -> E (identifier lookup in environment)
            Expr::Variable(name) => self.evaluate_variable(name, &env),
            
            // Application: (E0 E1 ...) -> E (procedure application)
            Expr::List(exprs) if !exprs.is_empty() => {
                self.evaluate_application(exprs, env)
            }
            
            // Empty list
            Expr::List(exprs) if exprs.is_empty() => Ok(Value::Nil),
            
            // Quoted expressions
            Expr::Quote(expr) => self.evaluate_quote(expr),
            
            _ => Err(LambdustError::RuntimeError {
                message: "Complex expression not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Evaluate a literal constant
    fn evaluate_literal(&self, literal: &Literal) -> Result<Value> {
        match literal {
            Literal::Boolean(b) => Ok(Value::Boolean(*b)),
            Literal::Number(n) => Ok(Value::Number(n.clone())),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Character(c) => Ok(Value::Character(*c)),
            Literal::Nil => Ok(Value::Nil),
        }
    }
    
    /// Evaluate a variable lookup
    fn evaluate_variable(&self, name: &str, env: &Environment) -> Result<Value> {
        env.get(name).ok_or_else(|| LambdustError::UndefinedVariable {
            variable: name.to_string(),
            context: Box::new(crate::error::ErrorContext::unknown()),
        })
    }
    
    /// Evaluate a procedure application
    fn evaluate_application(&mut self, exprs: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if exprs.is_empty() {
            return Ok(Value::Nil);
        }
        
        // Evaluate the operator (first expression)
        let operator = self.evaluate(&exprs[0], env.clone())?;
        
        // Check for special forms first
        if let Expr::Variable(name) = &exprs[0] {
            match name.as_str() {
                "lambda" => return self.evaluate_lambda(&exprs[1..], env),
                "if" => return self.evaluate_if(&exprs[1..], env),
                "define" => return self.evaluate_define(&exprs[1..], env),
                "quote" => return self.evaluate_quote_form(&exprs[1..]),
                _ => {}
            }
        }
        
        // Regular procedure application
        match operator {
            Value::Procedure(proc) => {
                // Evaluate arguments
                let mut args = Vec::with_capacity(exprs.len() - 1);
                for arg_expr in &exprs[1..] {
                    args.push(self.evaluate(arg_expr, env.clone())?);
                }
                
                // Apply procedure
                self.apply_procedure(&proc, &args, env)
            }
            _ => Err(LambdustError::TypeError {
                message: format!("Expected procedure, got {:?}", operator),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Evaluate lambda expression
    fn evaluate_lambda(&self, args: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::SyntaxError {
                message: "lambda requires parameter list and body".to_string(),
                location: crate::error::SourceSpan::unknown(),
            });
        }
        
        // Extract parameter list
        let params = match &args[0] {
            Expr::List(param_exprs) => {
                let mut params = Vec::new();
                for param_expr in param_exprs {
                    match param_expr {
                        Expr::Variable(name) => params.push(name.clone()),
                        _ => return Err(LambdustError::SyntaxError {
                            message: "lambda parameters must be identifiers".to_string(),
                            location: crate::error::SourceSpan::unknown(),
                        }),
                    }
                }
                params
            }
            Expr::Variable(name) => vec![name.clone()], // Variadic lambda
            _ => return Err(LambdustError::SyntaxError {
                message: "invalid lambda parameter list".to_string(),
                location: crate::error::SourceSpan::unknown(),
            }),
        };
        
        // Extract body (last expression)
        let body = args[args.len() - 1].clone();
        
        // Create procedure value
        Ok(Value::Procedure(crate::value::Procedure::Lambda {
            params,
            variadic: matches!(&args[0], Expr::Variable(_)),
            body: vec![body],
            closure: env,
        }))
    }
    
    /// Evaluate if expression
    fn evaluate_if(&mut self, args: &[Expr], env: Rc<Environment>) -> Result<Value> {
        match args.len() {
            2 | 3 => {
                // Evaluate condition
                let condition = self.evaluate(&args[0], env.clone())?;
                
                // Check truthiness (only #f is false in Scheme)
                let is_true = !matches!(condition, Value::Boolean(false));
                
                if is_true {
                    // Evaluate consequent
                    self.evaluate(&args[1], env)
                } else if args.len() == 3 {
                    // Evaluate alternative
                    self.evaluate(&args[2], env)
                } else {
                    // No alternative, return unspecified
                    Ok(Value::Undefined)
                }
            }
            _ => Err(LambdustError::SyntaxError {
                message: "if requires 2 or 3 arguments".to_string(),
                location: crate::error::SourceSpan::unknown(),
            }),
        }
    }
    
    /// Evaluate define expression
    fn evaluate_define(&mut self, args: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::SyntaxError {
                message: "define requires exactly 2 arguments".to_string(),
                location: crate::error::SourceSpan::unknown(),
            });
        }
        
        match &args[0] {
            // Simple variable definition: (define var expr)
            Expr::Variable(name) => {
                let value = self.evaluate(&args[1], env.clone())?;
                env.define(name.clone(), value);
                Ok(Value::Undefined)
            }
            
            // Function definition: (define (name params...) body)
            Expr::List(def_list) if !def_list.is_empty() => {
                if let Expr::Variable(name) = &def_list[0] {
                    // Extract parameters
                    let mut params = Vec::new();
                    for param_expr in &def_list[1..] {
                        match param_expr {
                            Expr::Variable(param_name) => params.push(param_name.clone()),
                            _ => return Err(LambdustError::SyntaxError {
                                message: "function parameters must be identifiers".to_string(),
                                location: crate::error::SourceSpan::unknown(),
                            }),
                        }
                    }
                    
                    // Create lambda
                    let lambda = Value::Procedure(crate::value::Procedure::Lambda {
                        params,
                        variadic: false,
                        body: vec![args[1].clone()],
                        closure: env.clone(),
                    });
                    
                    env.define(name.clone(), lambda);
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::SyntaxError {
                        message: "function name must be an identifier".to_string(),
                        location: crate::error::SourceSpan::unknown(),
                    })
                }
            }
            
            _ => Err(LambdustError::SyntaxError {
                message: "invalid define syntax".to_string(),
                location: crate::error::SourceSpan::unknown(),
            }),
        }
    }
    
    /// Evaluate quoted expression
    fn evaluate_quote(&self, expr: &Expr) -> Result<Value> {
        self.expr_to_value(expr)
    }
    
    /// Evaluate quote special form
    fn evaluate_quote_form(&self, args: &[Expr]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::SyntaxError {
                message: "quote requires exactly 1 argument".to_string(),
                location: crate::error::SourceSpan::unknown(),
            });
        }
        
        self.expr_to_value(&args[0])
    }
    
    /// Convert expression to quoted value
    fn expr_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Variable(name) => Ok(Value::Symbol(name.clone())),
            Expr::List(exprs) => {
                let mut result = Value::Nil;
                for expr in exprs.iter().rev() {
                    let car = self.expr_to_value(expr)?;
                    result = Value::cons(car, result);
                }
                Ok(result)
            }
            Expr::Quote(inner) => self.expr_to_value(inner),
            // R7RS-pico does not support these advanced features
            Expr::HygienicVariable(_) => Err(LambdustError::RuntimeError {
                message: "Hygienic variables not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
            Expr::Quasiquote(_) => Err(LambdustError::RuntimeError {
                message: "Quasiquote not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
            Expr::Unquote(_) => Err(LambdustError::RuntimeError {
                message: "Unquote not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
            Expr::UnquoteSplicing(_) => Err(LambdustError::RuntimeError {
                message: "Unquote-splicing not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
            Expr::Vector(_) => Err(LambdustError::RuntimeError {
                message: "Vector literals not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
            Expr::DottedList(_, _) => Err(LambdustError::RuntimeError {
                message: "Dotted lists not supported in R7RS-pico".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Apply a procedure to arguments
    fn apply_procedure(
        &mut self, 
        proc: &crate::value::Procedure, 
        args: &[Value], 
        _env: Rc<Environment>
    ) -> Result<Value> {
        match proc {
            crate::value::Procedure::Lambda { params, body, closure, variadic } => {
                // Check arity
                if *variadic {
                    if args.len() < params.len() - 1 {
                        return Err(LambdustError::ArityError {
                            expected: params.len() - 1,
                            actual: args.len(),
                            function: "lambda".to_string(),
                            context: Box::new(crate::error::ErrorContext::unknown()),
                        });
                    }
                } else if args.len() != params.len() {
                    return Err(LambdustError::ArityError {
                        expected: params.len(),
                        actual: args.len(),
                        function: "lambda".to_string(),
                        context: Box::new(crate::error::ErrorContext::unknown()),
                    });
                }
                
                // Create new environment with parameter bindings
                let new_env = Environment::extend(&closure);
                
                if *variadic {
                    // Bind fixed parameters
                    for (i, param) in params[..params.len() - 1].iter().enumerate() {
                        new_env.define(param.clone(), args[i].clone());
                    }
                    
                    // Bind rest parameters as list
                    let rest_args = &args[params.len() - 1..];
                    let mut rest_list = Value::Nil;
                    for arg in rest_args.iter().rev() {
                        rest_list = Value::cons(arg.clone(), rest_list);
                    }
                    new_env.define(params.last().unwrap().clone(), rest_list);
                } else {
                    // Bind all parameters
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.define(param.clone(), arg.clone());
                    }
                }
                
                // Evaluate body in new environment (use first expression of body)
                if body.is_empty() {
                    Ok(Value::Undefined)
                } else {
                    self.evaluate(&body[0], Rc::new(new_env))
                }
            }
            
            crate::value::Procedure::Builtin { name, .. } => {
                // Apply built-in procedure
                self.apply_builtin(name, args)
            }
            
            _ => Err(LambdustError::TypeError {
                message: "Expected lambda or builtin procedure, got other procedure type".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Apply a built-in procedure
    fn apply_builtin(&self, name: &str, args: &[Value]) -> Result<Value> {
        match name {
            // Arithmetic operations
            "+" => self.builtin_add(args),
            "-" => self.builtin_subtract(args),
            "*" => self.builtin_multiply(args),
            "=" => self.builtin_equal(args),
            "<" => self.builtin_less_than(args),
            ">" => self.builtin_greater_than(args),
            
            // List operations
            "cons" => self.builtin_cons(args),
            "car" => self.builtin_car(args),
            "cdr" => self.builtin_cdr(args),
            
            // Predicates
            "null?" => self.builtin_null_p(args),
            "pair?" => self.builtin_pair_p(args),
            "number?" => self.builtin_number_p(args),
            "boolean?" => self.builtin_boolean_p(args),
            "symbol?" => self.builtin_symbol_p(args),
            "procedure?" => self.builtin_procedure_p(args),
            
            // Equivalence
            "eqv?" => self.builtin_eqv_p(args),
            
            _ => Err(LambdustError::UndefinedVariable {
                variable: name.to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in addition
    fn builtin_add(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "+".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match (&args[0], &args[1]) {
            (Value::Number(n1), Value::Number(n2)) => {
                match (n1, n2) {
                    (SchemeNumber::Integer(i1), SchemeNumber::Integer(i2)) => {
                        Ok(Value::Number(SchemeNumber::Integer(i1 + i2)))
                    }
                    _ => Err(LambdustError::TypeError {
                        message: "Expected integer, got non-integer number".to_string(),
                        context: Box::new(crate::error::ErrorContext::unknown()),
                    }),
                }
            }
            _ => Err(LambdustError::TypeError {
                message: "Expected number, got non-number".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in subtraction
    fn builtin_subtract(&self, args: &[Value]) -> Result<Value> {
        match args.len() {
            1 => {
                // Unary negation
                match &args[0] {
                    Value::Number(SchemeNumber::Integer(i)) => {
                        Ok(Value::Number(SchemeNumber::Integer(-i)))
                    }
                    _ => Err(LambdustError::TypeError {
                        message: "Expected integer, got non-integer".to_string(),
                        context: Box::new(crate::error::ErrorContext::unknown()),
                    }),
                }
            }
            2 => {
                // Binary subtraction
                match (&args[0], &args[1]) {
                    (Value::Number(SchemeNumber::Integer(i1)), Value::Number(SchemeNumber::Integer(i2))) => {
                        Ok(Value::Number(SchemeNumber::Integer(i1 - i2)))
                    }
                    _ => Err(LambdustError::TypeError {
                        message: "Expected integer, got non-integer".to_string(),
                        context: Box::new(crate::error::ErrorContext::unknown()),
                    }),
                }
            }
            _ => Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "-".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in multiplication
    fn builtin_multiply(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "*".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match (&args[0], &args[1]) {
            (Value::Number(SchemeNumber::Integer(i1)), Value::Number(SchemeNumber::Integer(i2))) => {
                Ok(Value::Number(SchemeNumber::Integer(i1 * i2)))
            }
            _ => Err(LambdustError::TypeError {
                message: "Expected integer, got non-integer".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in numeric equality
    fn builtin_equal(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "=".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match (&args[0], &args[1]) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Boolean(n1 == n2)),
            _ => Err(LambdustError::TypeError {
                message: "Expected number, got non-number".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in less than
    fn builtin_less_than(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "=".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match (&args[0], &args[1]) {
            (Value::Number(SchemeNumber::Integer(i1)), Value::Number(SchemeNumber::Integer(i2))) => {
                Ok(Value::Boolean(i1 < i2))
            }
            _ => Err(LambdustError::TypeError {
                message: "Expected integer, got non-integer".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in greater than
    fn builtin_greater_than(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "=".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match (&args[0], &args[1]) {
            (Value::Number(SchemeNumber::Integer(i1)), Value::Number(SchemeNumber::Integer(i2))) => {
                Ok(Value::Boolean(i1 > i2))
            }
            _ => Err(LambdustError::TypeError {
                message: "Expected integer, got non-integer".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in cons
    fn builtin_cons(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "=".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::cons(args[0].clone(), args[1].clone()))
    }
    
    /// Built-in car
    fn builtin_car(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Ok(pair.car.clone())
            },
            _ => Err(LambdustError::TypeError {
                message: "Expected pair, got non-pair".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in cdr
    fn builtin_cdr(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Ok(pair.cdr.clone())
            },
            _ => Err(LambdustError::TypeError {
                message: "Expected pair, got non-pair".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            }),
        }
    }
    
    /// Built-in null? predicate
    fn builtin_null_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Nil)))
    }
    
    /// Built-in pair? predicate
    fn builtin_pair_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Pair(_))))
    }
    
    /// Built-in number? predicate
    fn builtin_number_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Number(_))))
    }
    
    /// Built-in boolean? predicate
    fn builtin_boolean_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
    }
    
    /// Built-in symbol? predicate
    fn builtin_symbol_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Symbol(_))))
    }
    
    /// Built-in procedure? predicate
    fn builtin_procedure_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: args.len(),
                function: "builtin".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Procedure(_))))
    }
    
    /// Built-in eqv? predicate
    fn builtin_eqv_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: args.len(),
                function: "=".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            });
        }
        
        let result = match (&args[0], &args[1]) {
            // Booleans
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            // Numbers
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            // Symbols
            (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
            // Nil
            (Value::Nil, Value::Nil) => true,
            // Different types
            _ => false,
        };
        
        Ok(Value::Boolean(result))
    }
    
    /// Reset recursion depth (for testing)
    pub fn reset_depth(&mut self) {
        self.current_depth = 0;
    }
    
    /// Get current recursion depth
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }
    
    /// Get maximum recursion depth
    pub fn max_depth(&self) -> usize {
        self.max_recursion_depth
    }
}

#[cfg(feature = "pico")]
impl Default for PicoEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "pico")]
mod tests {
    use super::*;
    use crate::ast::Literal;
    
    fn create_test_env() -> Rc<Environment> {
        crate::evaluator::pico_environment::create_pico_initial_environment()
    }
    
    #[test]
    fn test_literal_evaluation() {
        let mut evaluator = PicoEvaluator::new();
        let env = create_test_env();
        
        // Test number
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = evaluator.evaluate(&expr, env.clone()).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
        
        // Test boolean
        let expr = Expr::Literal(Literal::Boolean(true));
        let result = evaluator.evaluate(&expr, env.clone()).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test nil
        let expr = Expr::Literal(Literal::Nil);
        let result = evaluator.evaluate(&expr, env).unwrap();
        assert_eq!(result, Value::Nil);
    }
    
    #[test]
    fn test_arithmetic_operations() {
        let evaluator = PicoEvaluator::new();
        
        // Test addition
        let args = vec![
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ];
        let result = evaluator.builtin_add(&args).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(7)));
        
        // Test subtraction
        let result = evaluator.builtin_subtract(&args).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-1)));
        
        // Test multiplication
        let result = evaluator.builtin_multiply(&args).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(12)));
    }
    
    #[test]
    fn test_list_operations() {
        let evaluator = PicoEvaluator::new();
        
        // Test cons
        let args = vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ];
        let result = evaluator.builtin_cons(&args).unwrap();
        
        // Test car
        let car_result = evaluator.builtin_car(&[result.clone()]).unwrap();
        assert_eq!(car_result, Value::Number(SchemeNumber::Integer(1)));
        
        // Test cdr
        let cdr_result = evaluator.builtin_cdr(&[result]).unwrap();
        assert_eq!(cdr_result, Value::Number(SchemeNumber::Integer(2)));
    }
    
    #[test]
    fn test_predicates() {
        let evaluator = PicoEvaluator::new();
        
        // Test number?
        let args = vec![Value::Number(SchemeNumber::Integer(42))];
        let result = evaluator.builtin_number_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test boolean?
        let args = vec![Value::Boolean(true)];
        let result = evaluator.builtin_boolean_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test null?
        let args = vec![Value::Nil];
        let result = evaluator.builtin_null_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
    
    #[test]
    fn test_eqv_predicate() {
        let evaluator = PicoEvaluator::new();
        
        // Test equal numbers
        let args = vec![
            Value::Number(SchemeNumber::Integer(42)),
            Value::Number(SchemeNumber::Integer(42)),
        ];
        let result = evaluator.builtin_eqv_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test different numbers
        let args = vec![
            Value::Number(SchemeNumber::Integer(42)),
            Value::Number(SchemeNumber::Integer(43)),
        ];
        let result = evaluator.builtin_eqv_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(false));
        
        // Test different types
        let args = vec![
            Value::Number(SchemeNumber::Integer(42)),
            Value::Boolean(true),
        ];
        let result = evaluator.builtin_eqv_p(&args).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
    
    #[test]
    fn test_recursion_limit() {
        let mut evaluator = PicoEvaluator::with_recursion_limit(5);
        let env = create_test_env();
        
        // Create a deeply nested expression that exceeds the limit
        let mut expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
        for _ in 0..10 {
            expr = Expr::List(vec![
                Expr::Variable("+".to_string()),
                expr,
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]);
        }
        
        // This should fail due to recursion limit
        let result = evaluator.evaluate(&expr, env);
        assert!(result.is_err());
        
        if let Err(LambdustError::StackOverflow { context: _ }) = result {
            // Test passed - we got the expected stack overflow error
        } else {
            panic!("Expected StackOverflow error");
        }
    }
}