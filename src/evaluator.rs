//! Core evaluator for Scheme expressions

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::macros::MacroExpander;
use crate::value::{Procedure, Value};
use std::rc::Rc;

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 1000;

/// The main evaluator for Scheme expressions
#[derive(Debug)]
pub struct Evaluator {
    /// Global environment
    pub global_env: Rc<Environment>,
    /// Current recursion depth
    recursion_depth: usize,
    /// Macro expander
    macro_expander: MacroExpander,
}

impl Evaluator {
    /// Create a new evaluator with built-in procedures
    pub fn new() -> Self {
        Evaluator {
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            macro_expander: MacroExpander::new(),
        }
    }

    /// Create a new evaluator with a custom global environment
    pub fn with_environment(env: Environment) -> Self {
        Evaluator {
            global_env: Rc::new(env),
            recursion_depth: 0,
            macro_expander: MacroExpander::new(),
        }
    }

    /// Evaluate an expression in the global environment
    pub fn eval(&mut self, expr: Expr) -> Result<Value> {
        // First expand macros
        let expanded = self.macro_expander.expand_all(expr)?;
        self.eval_in_env(expanded, self.global_env.clone())
    }

    /// Evaluate an expression in a specific environment
    pub fn eval_in_env(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        // Check recursion depth
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            return Err(LambdustError::StackOverflow);
        }

        self.recursion_depth += 1;
        let result = self.eval_impl(expr, env);
        self.recursion_depth -= 1;
        result
    }

    /// Internal evaluation implementation
    fn eval_impl(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Variable(name) => env.get(&name),
            Expr::List(exprs) => self.eval_list(exprs, env),
            Expr::Quote(expr) => self.eval_quote(*expr),
            Expr::Quasiquote(expr) => self.eval_quasiquote(*expr, env),
            Expr::Unquote(_) => Err(LambdustError::SyntaxError(
                "unquote outside of quasiquote".to_string(),
            )),
            Expr::UnquoteSplicing(_) => Err(LambdustError::SyntaxError(
                "unquote-splicing outside of quasiquote".to_string(),
            )),
            Expr::DottedList(exprs, tail) => self.eval_dotted_list(exprs, *tail, env),
        }
    }

    /// Evaluate a literal expression
    fn eval_literal(&self, lit: Literal) -> Result<Value> {
        match lit {
            Literal::Boolean(b) => Ok(Value::Boolean(b)),
            Literal::Number(n) => Ok(Value::Number(n)),
            Literal::String(s) => Ok(Value::String(s)),
            Literal::Character(c) => Ok(Value::Character(c)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    /// Evaluate a list expression (procedure call or special form)
    fn eval_list(&mut self, exprs: Vec<Expr>, env: Rc<Environment>) -> Result<Value> {
        if exprs.is_empty() {
            return Ok(Value::Nil);
        }

        let operator = &exprs[0];
        let operands = &exprs[1..];

        // Check for special forms (note: macros are already expanded at this point)
        if let Expr::Variable(name) = operator {
            match name.as_str() {
                "quote" => return self.eval_quote_special(operands),
                "define" => return self.eval_define(operands, env),
                "set!" => return self.eval_set(operands, env),
                "lambda" => return self.eval_lambda(operands, env),
                "if" => return self.eval_if(operands, env),
                "and" => return self.eval_and(operands, env),
                "or" => return self.eval_or(operands, env),
                "begin" => return self.eval_begin(operands, env),
                "do" => return self.eval_do(operands, env),
                _ => {} // Not a special form, treat as procedure call
            }
        }

        // Evaluate operator
        let proc = self.eval_impl(operator.clone(), env.clone())?;

        // Evaluate operands
        let mut args = Vec::new();
        for operand in operands {
            args.push(self.eval_impl(operand.clone(), env.clone())?);
        }

        // Apply procedure
        self.apply_procedure(proc, args, env)
    }

    /// Apply a procedure to arguments
    fn apply_procedure(
        &mut self,
        proc: Value,
        args: Vec<Value>,
        _env: Rc<Environment>,
    ) -> Result<Value> {
        match proc {
            Value::Procedure(procedure) => match procedure {
                Procedure::Builtin { func, arity, .. } => {
                    // Check arity for built-in procedures
                    if let Some(expected) = arity {
                        if args.len() != expected {
                            return Err(LambdustError::ArityError {
                                expected,
                                actual: args.len(),
                            });
                        }
                    }
                    func(&args)
                }
                Procedure::Lambda {
                    params,
                    variadic,
                    body,
                    closure,
                } => {
                    // Create new environment with parameter bindings
                    let new_env = closure.bind_parameters(&params, &args, variadic)?;

                    // Evaluate body expressions
                    let mut result = Value::Undefined;
                    for expr in body {
                        result = self.eval_impl(expr, Rc::new(new_env.clone()))?;
                    }
                    Ok(result)
                }
                Procedure::Continuation { .. } => {
                    // TODO: Implement continuations
                    Err(LambdustError::RuntimeError(
                        "Continuations not yet implemented".to_string(),
                    ))
                }
            },
            _ => Err(LambdustError::TypeError(format!(
                "Not a procedure: {proc}"
            ))),
        }
    }

    /// Evaluate a quote expression
    fn eval_quote(&self, expr: Expr) -> Result<Value> {
        self.expr_to_value(expr)
    }

    /// Evaluate quote special form
    fn eval_quote_special(&self, operands: &[Expr]) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::ArityError {
                expected: 1,
                actual: operands.len(),
            });
        }
        self.eval_quote(operands[0].clone())
    }

    /// Evaluate a quasiquote expression
    fn eval_quasiquote(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        self.eval_quasiquote_impl(expr, env, 0)
    }

    /// Internal quasiquote evaluation with nesting level
    fn eval_quasiquote_impl(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        level: usize,
    ) -> Result<Value> {
        match expr {
            Expr::Unquote(inner) => {
                if level == 0 {
                    self.eval_impl(*inner, env)
                } else {
                    let inner_result = self.eval_quasiquote_impl(*inner, env, level - 1)?;
                    Ok(Value::from_vector(vec![
                        Value::Symbol("unquote".to_string()),
                        inner_result,
                    ]))
                }
            }
            Expr::UnquoteSplicing(inner) => {
                if level == 0 {
                    // TODO: Handle splicing properly
                    self.eval_impl(*inner, env)
                } else {
                    let inner_result = self.eval_quasiquote_impl(*inner, env, level - 1)?;
                    Ok(Value::from_vector(vec![
                        Value::Symbol("unquote-splicing".to_string()),
                        inner_result,
                    ]))
                }
            }
            Expr::Quasiquote(inner) => {
                let inner_result = self.eval_quasiquote_impl(*inner, env, level + 1)?;
                Ok(Value::from_vector(vec![
                    Value::Symbol("quasiquote".to_string()),
                    inner_result,
                ]))
            }
            Expr::List(exprs) => {
                let mut result = Vec::new();
                for expr in exprs {
                    result.push(self.eval_quasiquote_impl(expr, env.clone(), level)?);
                }
                Ok(Value::from_vector(result))
            }
            _ => self.expr_to_value(expr),
        }
    }

    /// Convert an expression to a value (for quote)
    fn expr_to_value(&self, expr: Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Variable(name) => Ok(Value::Symbol(name)),
            Expr::List(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.expr_to_value(expr)?);
                }
                Ok(Value::from_vector(values))
            }
            Expr::DottedList(exprs, tail) => {
                let mut result = self.expr_to_value(*tail)?;
                for expr in exprs.into_iter().rev() {
                    result = Value::cons(self.expr_to_value(expr)?, result);
                }
                Ok(result)
            }
            _ => Err(LambdustError::RuntimeError(
                "Cannot quote this expression".to_string(),
            )),
        }
    }

    /// Evaluate a dotted list
    fn eval_dotted_list(
        &mut self,
        exprs: Vec<Expr>,
        tail: Expr,
        env: Rc<Environment>,
    ) -> Result<Value> {
        // Evaluate all expressions and create an improper list
        let mut values = Vec::new();
        for expr in exprs {
            values.push(self.eval_impl(expr, env.clone())?);
        }
        let tail_value = self.eval_impl(tail, env)?;

        let mut result = tail_value;
        for value in values.into_iter().rev() {
            result = Value::cons(value, result);
        }
        Ok(result)
    }

    /// Evaluate define special form
    fn eval_define(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::SyntaxError(
                "define: too few arguments".to_string(),
            ));
        }

        match &operands[0] {
            // (define var value)
            Expr::Variable(name) => {
                if operands.len() != 2 {
                    return Err(LambdustError::SyntaxError(
                        "define: too many arguments".to_string(),
                    ));
                }
                let value = self.eval_impl(operands[1].clone(), env.clone())?;
                env.define(name.clone(), value);
                Ok(Value::Undefined)
            }
            // (define (name params...) body...)
            Expr::List(def_exprs) => {
                if def_exprs.is_empty() {
                    return Err(LambdustError::SyntaxError(
                        "define: empty function definition".to_string(),
                    ));
                }

                let name = match &def_exprs[0] {
                    Expr::Variable(n) => n.clone(),
                    _ => {
                        return Err(LambdustError::SyntaxError(
                            "define: invalid function name".to_string(),
                        ));
                    }
                };

                let params: Result<Vec<String>> = def_exprs[1..]
                    .iter()
                    .map(|expr| match expr {
                        Expr::Variable(param) => Ok(param.clone()),
                        _ => Err(LambdustError::SyntaxError(
                            "define: invalid parameter".to_string(),
                        )),
                    })
                    .collect();

                let params = params?;
                let body = operands[1..].to_vec();

                let lambda = Value::Procedure(Procedure::Lambda {
                    params,
                    variadic: false,
                    body,
                    closure: env.clone(),
                });

                env.define(name, lambda);
                Ok(Value::Undefined)
            }
            _ => Err(LambdustError::SyntaxError(
                "define: invalid syntax".to_string(),
            )),
        }
    }

    /// Evaluate set! special form
    fn eval_set(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::ArityError {
                expected: 2,
                actual: operands.len(),
            });
        }

        let name = match &operands[0] {
            Expr::Variable(n) => n,
            _ => {
                return Err(LambdustError::SyntaxError(
                    "set!: not a variable".to_string(),
                ));
            }
        };

        let value = self.eval_impl(operands[1].clone(), env.clone())?;
        env.set(name, value)?;
        Ok(Value::Undefined)
    }

    /// Evaluate lambda special form
    fn eval_lambda(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::SyntaxError(
                "lambda: too few arguments".to_string(),
            ));
        }

        // Parse parameters
        let (params, variadic) = match &operands[0] {
            Expr::List(param_exprs) => {
                let params: Result<Vec<String>> = param_exprs
                    .iter()
                    .map(|expr| match expr {
                        Expr::Variable(param) => Ok(param.clone()),
                        _ => Err(LambdustError::SyntaxError(
                            "lambda: invalid parameter".to_string(),
                        )),
                    })
                    .collect();
                (params?, false)
            }
            Expr::Variable(param) => {
                // Single parameter (variadic)
                (vec![param.clone()], true)
            }
            Expr::DottedList(param_exprs, rest) => {
                let mut params: Vec<String> = param_exprs
                    .iter()
                    .map(|expr| match expr {
                        Expr::Variable(param) => Ok(param.clone()),
                        _ => Err(LambdustError::SyntaxError(
                            "lambda: invalid parameter".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>>>()?;

                match rest.as_ref() {
                    Expr::Variable(rest_param) => {
                        params.push(rest_param.clone());
                        (params, true)
                    }
                    _ => {
                        return Err(LambdustError::SyntaxError(
                            "lambda: invalid rest parameter".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(LambdustError::SyntaxError(
                    "lambda: invalid parameter list".to_string(),
                ));
            }
        };

        let body = operands[1..].to_vec();

        Ok(Value::Procedure(Procedure::Lambda {
            params,
            variadic,
            body,
            closure: env,
        }))
    }

    /// Evaluate if special form
    fn eval_if(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        match operands.len() {
            2 => {
                let test = self.eval_impl(operands[0].clone(), env.clone())?;
                if test.is_truthy() {
                    self.eval_impl(operands[1].clone(), env)
                } else {
                    Ok(Value::Undefined)
                }
            }
            3 => {
                let test = self.eval_impl(operands[0].clone(), env.clone())?;
                if test.is_truthy() {
                    self.eval_impl(operands[1].clone(), env)
                } else {
                    self.eval_impl(operands[2].clone(), env)
                }
            }
            _ => Err(LambdustError::ArityError {
                expected: 2,
                actual: operands.len(),
            }),
        }
    }

    /// Evaluate begin special form
    fn eval_begin(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.is_empty() {
            return Ok(Value::Undefined);
        }

        let mut result = Value::Undefined;
        for expr in operands {
            result = self.eval_impl(expr.clone(), env.clone())?;
        }
        Ok(result)
    }

    // Note: cond, case, let, let*, letrec are now handled by macros

    fn eval_and(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        for operand in operands {
            let value = self.eval_impl(operand.clone(), env.clone())?;
            if !value.is_truthy() {
                return Ok(value);
            }
        }
        Ok(Value::Boolean(true))
    }

    fn eval_or(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        for operand in operands {
            let value = self.eval_impl(operand.clone(), env.clone())?;
            if value.is_truthy() {
                return Ok(value);
            }
        }
        Ok(Value::Boolean(false))
    }

    fn eval_do(&mut self, _operands: &[Expr], _env: Rc<Environment>) -> Result<Value> {
        Err(LambdustError::RuntimeError(
            "do not yet implemented".to_string(),
        ))
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn eval_str(input: &str) -> Result<Value> {
        let tokens = tokenize(input)?;
        let ast = parse(tokens)?;
        let mut evaluator = Evaluator::new();
        evaluator.eval(ast)
    }

    #[test]
    fn test_eval_literals() {
        assert_eq!(eval_str("42").unwrap(), Value::from(42i64));
        assert_eq!(eval_str("#t").unwrap(), Value::Boolean(true));
        assert_eq!(eval_str("\"hello\"").unwrap(), Value::from("hello"));
        assert_eq!(eval_str("()").unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_quote() {
        assert_eq!(eval_str("'x").unwrap(), Value::Symbol("x".to_string()));
        assert_eq!(
            eval_str("'(1 2 3)").unwrap(),
            Value::from_vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );
    }

    #[test]
    fn test_eval_arithmetic() {
        // These will work once we implement arithmetic builtins
        // assert_eq!(eval_str("(+ 1 2)").unwrap(), Value::from(3i64));
        // assert_eq!(eval_str("(* 3 4)").unwrap(), Value::from(12i64));
    }

    #[test]
    fn test_eval_define() {
        let mut evaluator = Evaluator::new();
        let tokens = tokenize("(define x 42)").unwrap();
        let ast = parse(tokens).unwrap();
        evaluator.eval(ast).unwrap();

        let tokens = tokenize("x").unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(evaluator.eval(ast).unwrap(), Value::from(42i64));
    }

    #[test]
    fn test_eval_lambda() {
        let result = eval_str("(lambda (x) x)").unwrap();
        assert!(result.is_procedure());
    }

    #[test]
    fn test_eval_if() {
        assert_eq!(eval_str("(if #t 1 2)").unwrap(), Value::from(1i64));
        assert_eq!(eval_str("(if #f 1 2)").unwrap(), Value::from(2i64));
    }

    #[test]
    fn test_eval_begin() {
        let result = eval_str("(begin 1 2 3)").unwrap();
        assert_eq!(result, Value::from(3i64));
    }
}
