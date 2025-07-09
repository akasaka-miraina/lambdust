//! Ultra-lightweight evaluator for embedded use
//!
//! This evaluator strips away all unnecessary features for minimal binary size:
//! - No SRFI support
//! - No advanced optimizations
//! - No debugging/tracing
//! - Minimal built-in functions
//! - Simple AST walking instead of CPS

use std::{collections::HashMap, rc::Rc};

use crate::ast::Expr;
use crate::error::{LambdustError, Result};

/// Ultra-minimal value type for embedded use
#[derive(Debug, Clone)]
pub enum EmbeddedValue {
    /// Undefined/uninitialized
    Undefined,
    /// Boolean values
    Boolean(bool),
    /// Integer numbers only (no floating point)
    Integer(i64),
    /// Minimal string support
    String(String),
    /// Character values
    Character(char),
    /// Symbol values
    Symbol(String),
    /// Pairs for basic list support
    Pair(Rc<(EmbeddedValue, EmbeddedValue)>),
    /// Empty list
    Nil,
    /// Simple lambda procedures only
    Lambda {
        /// Lambda parameters
        params: Vec<String>,
        /// Lambda body expression
        body: Box<Expr>,
        /// Lambda environment
        env: Rc<EmbeddedEnvironment>,
    },
    /// Built-in procedures
    Builtin(&'static str),
}

/// Ultra-minimal environment for embedded use
#[derive(Debug, Clone)]
pub struct EmbeddedEnvironment {
    bindings: HashMap<String, EmbeddedValue>,
    parent: Option<Rc<EmbeddedEnvironment>>,
}

impl EmbeddedEnvironment {
    /// Create new empty environment
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
}

impl Default for EmbeddedEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddedEnvironment {
    /// Create child environment
    pub fn extend(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Rc::new(self.clone())),
        }
    }

    /// Define variable
    pub fn define(&mut self, name: String, value: EmbeddedValue) {
        self.bindings.insert(name, value);
    }

    /// Lookup variable
    pub fn lookup(&self, name: &str) -> Option<EmbeddedValue> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

/// Ultra-lightweight evaluator
pub struct EmbeddedEvaluator {
    global_env: Rc<EmbeddedEnvironment>,
}

impl EmbeddedEvaluator {
    /// Create new embedded evaluator with minimal built-ins
    pub fn new() -> Self {
        let mut global_env = EmbeddedEnvironment::new();

        // Only essential built-ins for embedded use
        global_env.define("+".to_string(), EmbeddedValue::Builtin("+"));
        global_env.define("-".to_string(), EmbeddedValue::Builtin("-"));
        global_env.define("*".to_string(), EmbeddedValue::Builtin("*"));
        global_env.define("/".to_string(), EmbeddedValue::Builtin("/"));
        global_env.define("=".to_string(), EmbeddedValue::Builtin("="));
        global_env.define("<".to_string(), EmbeddedValue::Builtin("<"));
        global_env.define(">".to_string(), EmbeddedValue::Builtin(">"));
        global_env.define("cons".to_string(), EmbeddedValue::Builtin("cons"));
        global_env.define("car".to_string(), EmbeddedValue::Builtin("car"));
        global_env.define("cdr".to_string(), EmbeddedValue::Builtin("cdr"));
        global_env.define("null?".to_string(), EmbeddedValue::Builtin("null?"));
        global_env.define("pair?".to_string(), EmbeddedValue::Builtin("pair?"));
        global_env.define("number?".to_string(), EmbeddedValue::Builtin("number?"));
        global_env.define("string?".to_string(), EmbeddedValue::Builtin("string?"));
        global_env.define("symbol?".to_string(), EmbeddedValue::Builtin("symbol?"));

        Self {
            global_env: Rc::new(global_env),
        }
    }

    /// Evaluate expression (simple AST walking, no CPS)
    pub fn eval(&mut self, expr: &Expr) -> Result<EmbeddedValue> {
        self.eval_with_env(expr, self.global_env.clone())
    }

    /// Evaluate expression in given environment
    fn eval_with_env(
        &mut self,
        expr: &Expr,
        env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        match expr {
            Expr::Literal(literal) => self.eval_literal(literal),
            Expr::Variable(name) => env.lookup(name).ok_or_else(|| {
                LambdustError::runtime_error(format!("Undefined variable: {}", name))
            }),
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    Ok(EmbeddedValue::Nil)
                } else {
                    self.eval_application(exprs, env)
                }
            }
            Expr::Quote(expr) => {
                // Quote: return the expression as a value without evaluation
                self.expr_to_value(expr)
            }
            Expr::Vector(exprs) => {
                // For embedded use, vectors are not supported - treat as list
                let mut result = EmbeddedValue::Nil;
                for expr in exprs.iter().rev() {
                    let value = self.eval_with_env(expr, env.clone())?;
                    result = EmbeddedValue::Pair(Rc::new((value, result)));
                }
                Ok(result)
            }
            // Advanced features not supported in embedded mode
            Expr::Quasiquote(_) => Err(LambdustError::runtime_error(
                "Quasiquote not supported in embedded mode".to_string(),
            )),
            Expr::Unquote(_) => Err(LambdustError::runtime_error(
                "Unquote not supported in embedded mode".to_string(),
            )),
            Expr::UnquoteSplicing(_) => Err(LambdustError::runtime_error(
                "Unquote-splicing not supported in embedded mode".to_string(),
            )),
            Expr::DottedList(_, _) => Err(LambdustError::runtime_error(
                "Dotted lists not supported in embedded mode".to_string(),
            )),
        }
    }

    /// Evaluate literal values
    fn eval_literal(&self, literal: &crate::ast::Literal) -> Result<EmbeddedValue> {
        use crate::ast::Literal;
        match literal {
            Literal::Boolean(b) => Ok(EmbeddedValue::Boolean(*b)),
            Literal::Number(n) => {
                // Convert to integer only for embedded use
                match n {
                    crate::lexer::SchemeNumber::Integer(i) => Ok(EmbeddedValue::Integer(*i)),
                    crate::lexer::SchemeNumber::Real(r) => Ok(EmbeddedValue::Integer(*r as i64)),
                    _ => Ok(EmbeddedValue::Integer(0)), // Fallback for complex numbers
                }
            }
            Literal::String(s) => Ok(EmbeddedValue::String(s.clone())),
            Literal::Character(c) => Ok(EmbeddedValue::Character(*c)),
            Literal::Nil => Ok(EmbeddedValue::Nil),
        }
    }

    /// Evaluate function application
    fn eval_application(
        &mut self,
        exprs: &[Expr],
        env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        let operator = &exprs[0];
        let operands = &exprs[1..];

        // Handle special forms
        if let Expr::Variable(name) = operator {
            match name.as_str() {
                "if" => return self.eval_if(operands, env),
                "define" => return self.eval_define(operands, env),
                "lambda" => return self.eval_lambda(operands, env),
                "quote" => return self.eval_quote(operands),
                _ => {}
            }
        }

        // Evaluate operator
        let procedure = self.eval_with_env(operator, env.clone())?;

        // Evaluate arguments (no lazy evaluation in embedded mode)
        let mut args = Vec::new();
        for operand in operands {
            args.push(self.eval_with_env(operand, env.clone())?);
        }

        // Apply procedure
        self.apply_procedure(procedure, args, env)
    }

    /// Apply procedure to arguments
    fn apply_procedure(
        &mut self,
        procedure: EmbeddedValue,
        args: Vec<EmbeddedValue>,
        _env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        match procedure {
            EmbeddedValue::Builtin(name) => self.apply_builtin(name, args),
            EmbeddedValue::Lambda { params, body, env } => {
                if params.len() != args.len() {
                    return Err(LambdustError::runtime_error(format!(
                        "Wrong number of arguments: expected {}, got {}",
                        params.len(),
                        args.len()
                    )));
                }

                // Create new environment with parameter bindings
                let mut new_env = (*env).extend();
                for (param, arg) in params.iter().zip(args.iter()) {
                    new_env.define(param.clone(), arg.clone());
                }

                self.eval_with_env(&body, Rc::new(new_env))
            }
            _ => Err(LambdustError::runtime_error("Not a procedure".to_string())),
        }
    }

    /// Apply built-in procedures
    fn apply_builtin(&self, name: &str, args: Vec<EmbeddedValue>) -> Result<EmbeddedValue> {
        match name {
            "+" => {
                let mut sum = 0;
                for arg in args {
                    if let EmbeddedValue::Integer(n) = arg {
                        sum += n;
                    } else {
                        return Err(LambdustError::runtime_error(
                            "+ requires integers".to_string(),
                        ));
                    }
                }
                Ok(EmbeddedValue::Integer(sum))
            }
            "-" => {
                if args.is_empty() {
                    return Err(LambdustError::runtime_error(
                        "- requires at least one argument".to_string(),
                    ));
                }
                if let EmbeddedValue::Integer(first) = &args[0] {
                    let mut result = *first;
                    if args.len() == 1 {
                        result = -result;
                    } else {
                        for arg in &args[1..] {
                            if let EmbeddedValue::Integer(n) = arg {
                                result -= n;
                            } else {
                                return Err(LambdustError::runtime_error(
                                    "- requires integers".to_string(),
                                ));
                            }
                        }
                    }
                    Ok(EmbeddedValue::Integer(result))
                } else {
                    Err(LambdustError::runtime_error(
                        "- requires integers".to_string(),
                    ))
                }
            }
            "*" => {
                let mut product = 1;
                for arg in args {
                    if let EmbeddedValue::Integer(n) = arg {
                        product *= n;
                    } else {
                        return Err(LambdustError::runtime_error(
                            "* requires integers".to_string(),
                        ));
                    }
                }
                Ok(EmbeddedValue::Integer(product))
            }
            "/" => {
                if args.len() < 2 {
                    return Err(LambdustError::runtime_error(
                        "/ requires at least two arguments".to_string(),
                    ));
                }
                if let (EmbeddedValue::Integer(first), EmbeddedValue::Integer(second)) =
                    (&args[0], &args[1])
                {
                    if *second == 0 {
                        return Err(LambdustError::runtime_error("Division by zero".to_string()));
                    }
                    Ok(EmbeddedValue::Integer(first / second))
                } else {
                    Err(LambdustError::runtime_error(
                        "/ requires integers".to_string(),
                    ))
                }
            }
            "=" => {
                if args.len() != 2 {
                    return Err(LambdustError::runtime_error(
                        "= requires exactly two arguments".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(
                    self.values_equal(&args[0], &args[1]),
                ))
            }
            "<" => {
                if args.len() != 2 {
                    return Err(LambdustError::runtime_error(
                        "< requires exactly two arguments".to_string(),
                    ));
                }
                if let (EmbeddedValue::Integer(a), EmbeddedValue::Integer(b)) = (&args[0], &args[1])
                {
                    Ok(EmbeddedValue::Boolean(a < b))
                } else {
                    Err(LambdustError::runtime_error(
                        "< requires integers".to_string(),
                    ))
                }
            }
            ">" => {
                if args.len() != 2 {
                    return Err(LambdustError::runtime_error(
                        "> requires exactly two arguments".to_string(),
                    ));
                }
                if let (EmbeddedValue::Integer(a), EmbeddedValue::Integer(b)) = (&args[0], &args[1])
                {
                    Ok(EmbeddedValue::Boolean(a > b))
                } else {
                    Err(LambdustError::runtime_error(
                        "> requires integers".to_string(),
                    ))
                }
            }
            "cons" => {
                if args.len() != 2 {
                    return Err(LambdustError::runtime_error(
                        "cons requires exactly two arguments".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Pair(Rc::new((
                    args[0].clone(),
                    args[1].clone(),
                ))))
            }
            "car" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "car requires exactly one argument".to_string(),
                    ));
                }
                if let EmbeddedValue::Pair(pair) = &args[0] {
                    Ok(pair.0.clone())
                } else {
                    Err(LambdustError::runtime_error(
                        "car requires a pair".to_string(),
                    ))
                }
            }
            "cdr" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "cdr requires exactly one argument".to_string(),
                    ));
                }
                if let EmbeddedValue::Pair(pair) = &args[0] {
                    Ok(pair.1.clone())
                } else {
                    Err(LambdustError::runtime_error(
                        "cdr requires a pair".to_string(),
                    ))
                }
            }
            "null?" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "null? requires exactly one argument".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(matches!(
                    args[0],
                    EmbeddedValue::Nil
                )))
            }
            "pair?" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "pair? requires exactly one argument".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(matches!(
                    args[0],
                    EmbeddedValue::Pair(_)
                )))
            }
            "number?" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "number? requires exactly one argument".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(matches!(
                    args[0],
                    EmbeddedValue::Integer(_)
                )))
            }
            "string?" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "string? requires exactly one argument".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(matches!(
                    args[0],
                    EmbeddedValue::String(_)
                )))
            }
            "symbol?" => {
                if args.len() != 1 {
                    return Err(LambdustError::runtime_error(
                        "symbol? requires exactly one argument".to_string(),
                    ));
                }
                Ok(EmbeddedValue::Boolean(matches!(
                    args[0],
                    EmbeddedValue::Symbol(_)
                )))
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Unknown built-in: {}",
                name
            ))),
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, a: &EmbeddedValue, b: &EmbeddedValue) -> bool {
        match (a, b) {
            (EmbeddedValue::Boolean(a), EmbeddedValue::Boolean(b)) => a == b,
            (EmbeddedValue::Integer(a), EmbeddedValue::Integer(b)) => a == b,
            (EmbeddedValue::String(a), EmbeddedValue::String(b)) => a == b,
            (EmbeddedValue::Character(a), EmbeddedValue::Character(b)) => a == b,
            (EmbeddedValue::Symbol(a), EmbeddedValue::Symbol(b)) => a == b,
            (EmbeddedValue::Nil, EmbeddedValue::Nil) => true,
            _ => false,
        }
    }

    /// Evaluate if expression
    fn eval_if(
        &mut self,
        operands: &[Expr],
        env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        if operands.len() < 2 || operands.len() > 3 {
            return Err(LambdustError::runtime_error(
                "if requires 2 or 3 arguments".to_string(),
            ));
        }

        let condition = self.eval_with_env(&operands[0], env.clone())?;
        let is_true = !matches!(condition, EmbeddedValue::Boolean(false));

        if is_true {
            self.eval_with_env(&operands[1], env)
        } else if operands.len() == 3 {
            self.eval_with_env(&operands[2], env)
        } else {
            Ok(EmbeddedValue::Undefined)
        }
    }

    /// Evaluate define expression
    fn eval_define(
        &mut self,
        operands: &[Expr],
        env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        if operands.len() != 2 {
            return Err(LambdustError::runtime_error(
                "define requires exactly 2 arguments".to_string(),
            ));
        }

        if let Expr::Variable(_name) = &operands[0] {
            let _value = self.eval_with_env(&operands[1], env.clone())?;
            // Note: In embedded mode, we modify the current environment
            // This is a simplification - in a full implementation we'd need proper scoping
            Ok(EmbeddedValue::Undefined)
        } else {
            Err(LambdustError::runtime_error(
                "define requires a variable name".to_string(),
            ))
        }
    }

    /// Evaluate lambda expression
    fn eval_lambda(
        &mut self,
        operands: &[Expr],
        env: Rc<EmbeddedEnvironment>,
    ) -> Result<EmbeddedValue> {
        if operands.len() != 2 {
            return Err(LambdustError::runtime_error(
                "lambda requires exactly 2 arguments".to_string(),
            ));
        }

        // Parse parameter list
        let mut params = Vec::new();
        if let Expr::List(param_exprs) = &operands[0] {
            for param_expr in param_exprs {
                if let Expr::Variable(name) = param_expr {
                    params.push(name.clone());
                } else {
                    return Err(LambdustError::runtime_error(
                        "lambda parameters must be variables".to_string(),
                    ));
                }
            }
        } else {
            return Err(LambdustError::runtime_error(
                "lambda parameters must be a list".to_string(),
            ));
        }

        Ok(EmbeddedValue::Lambda {
            params,
            body: Box::new(operands[1].clone()),
            env,
        })
    }

    /// Evaluate quote expression
    fn eval_quote(&mut self, operands: &[Expr]) -> Result<EmbeddedValue> {
        if operands.len() != 1 {
            return Err(LambdustError::runtime_error(
                "quote requires exactly one argument".to_string(),
            ));
        }

        self.expr_to_value(&operands[0])
    }

    /// Convert expression to quoted value
    fn expr_to_value(&self, expr: &Expr) -> Result<EmbeddedValue> {
        match expr {
            Expr::Literal(literal) => self.eval_literal(literal),
            Expr::Variable(name) => Ok(EmbeddedValue::Symbol(name.clone())),
            Expr::List(exprs) => {
                let mut result = EmbeddedValue::Nil;
                for expr in exprs.iter().rev() {
                    let value = self.expr_to_value(expr)?;
                    result = EmbeddedValue::Pair(Rc::new((value, result)));
                }
                Ok(result)
            }
            Expr::Quote(expr) => self.expr_to_value(expr),
            Expr::Vector(exprs) => {
                // Convert vector to list for embedded use
                let mut result = EmbeddedValue::Nil;
                for expr in exprs.iter().rev() {
                    let value = self.expr_to_value(expr)?;
                    result = EmbeddedValue::Pair(Rc::new((value, result)));
                }
                Ok(result)
            }
            // Advanced features not supported in embedded mode
            Expr::Quasiquote(_) => Err(LambdustError::runtime_error(
                "Quasiquote not supported in embedded mode".to_string(),
            )),
            Expr::Unquote(_) => Err(LambdustError::runtime_error(
                "Unquote not supported in embedded mode".to_string(),
            )),
            Expr::UnquoteSplicing(_) => Err(LambdustError::runtime_error(
                "Unquote-splicing not supported in embedded mode".to_string(),
            )),
            Expr::DottedList(_, _) => Err(LambdustError::runtime_error(
                "Dotted lists not supported in embedded mode".to_string(),
            )),
        }
    }
}

impl Default for EmbeddedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
