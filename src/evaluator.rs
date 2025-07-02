//! Core evaluator for Scheme expressions

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::macros::MacroExpander;
use crate::value::{Continuation, Procedure, Value};
use std::rc::Rc;

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 1000;

/// Continuation information for tail call optimization
#[derive(Debug, Clone)]
pub enum TailCallInfo {
    /// Normal evaluation (not a tail call)
    None,
    /// Tail call (procedure and arguments)
    Call {
        /// Procedure to be called
        proc: Value,
        /// List of arguments
        args: Vec<Value>,
    },
}

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
            return Err(LambdustError::stack_overflow());
        }

        self.recursion_depth += 1;
        let result = self.eval_with_tail_optimization(expr, env);
        self.recursion_depth -= 1;
        result
    }

    /// Evaluation with tail call optimization
    fn eval_with_tail_optimization(
        &mut self,
        mut expr: Expr,
        mut env: Rc<Environment>,
    ) -> Result<Value> {
        loop {
            let tail_info = self.eval_impl_tail(expr.clone(), env.clone())?;

            match tail_info {
                TailCallInfo::None => return self.eval_impl(expr, env),
                TailCallInfo::Call { proc, args } => {
                    if let Some((new_expr, new_env)) =
                        self.try_optimize_lambda_call(&proc, &args)?
                    {
                        expr = new_expr;
                        env = new_env;
                        continue;
                    } else {
                        return self.apply_procedure(proc, args, env);
                    }
                }
            }
        }
    }

    /// Try to optimize lambda call for tail recursion
    fn try_optimize_lambda_call(
        &mut self,
        proc: &Value,
        args: &[Value],
    ) -> Result<Option<(Expr, Rc<Environment>)>> {
        let Value::Procedure(Procedure::Lambda {
            params,
            variadic,
            body,
            closure,
        }) = proc
        else {
            return Ok(None);
        };

        // Bind parameters in new environment
        let new_env = closure.bind_parameters(params, args, *variadic)?;

        // Evaluate all expressions except the last one
        self.eval_body_except_last(body, &new_env)?;

        // Return the last expression for tail call optimization
        if let Some(last_expr) = body.last() {
            Ok(Some((last_expr.clone(), Rc::new(new_env))))
        } else {
            // Empty body returns undefined, no optimization needed
            Ok(None)
        }
    }

    /// Evaluate all body expressions except the last one
    fn eval_body_except_last(&mut self, body: &[Expr], env: &Environment) -> Result<()> {
        for expr in &body[..body.len().saturating_sub(1)] {
            self.eval_impl(expr.clone(), Rc::new(env.clone()))?;
        }
        Ok(())
    }

    /// Evaluation implementation including tail call determination
    fn eval_impl_tail(&mut self, expr: Expr, env: Rc<Environment>) -> Result<TailCallInfo> {
        let Expr::List(exprs) = expr else {
            return Ok(TailCallInfo::None);
        };

        if exprs.is_empty() {
            return Ok(TailCallInfo::None);
        }

        let operator = &exprs[0];
        let operands = &exprs[1..];

        // Check for special forms
        if let Some(tail_info) = self.try_eval_special_form_tail(operator, operands, env.clone())? {
            return Ok(tail_info);
        }

        // For procedure calls, eligible for tail call optimization
        self.eval_procedure_call_tail(operator, operands, env)
    }

    /// Try to evaluate special forms for tail call optimization
    fn try_eval_special_form_tail(
        &mut self,
        operator: &Expr,
        operands: &[Expr],
        env: Rc<Environment>,
    ) -> Result<Option<TailCallInfo>> {
        let Expr::Variable(name) = operator else {
            return Ok(None);
        };

        match name.as_str() {
            "if" => Ok(Some(self.eval_if_tail(operands, env)?)),
            "begin" => Ok(Some(self.eval_begin_tail(operands, env)?)),
            // Special forms that are not tail-call optimizable
            "define"
            | "set!"
            | "lambda"
            | "quote"
            | "and"
            | "or"
            | "do"
            | "apply"
            | "map"
            | "for-each"
            | "call-with-values"
            | "delay"
            | "lazy"
            | "force"
            | "syntax-rules"
            | "call-with-current-continuation"
            | "call/cc" => Ok(Some(TailCallInfo::None)),
            _ => Ok(None), // Not a special form
        }
    }

    /// Evaluate procedure call for tail call optimization
    fn eval_procedure_call_tail(
        &mut self,
        operator: &Expr,
        operands: &[Expr],
        env: Rc<Environment>,
    ) -> Result<TailCallInfo> {
        let proc = self.eval_impl(operator.clone(), env.clone())?;
        let args = self.eval_operands(operands, env)?;
        Ok(TailCallInfo::Call { proc, args })
    }

    /// Evaluate a list of operands
    fn eval_operands(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Vec<Value>> {
        operands
            .iter()
            .map(|operand| self.eval_impl(operand.clone(), env.clone()))
            .collect()
    }

    /// Internal evaluation implementation
    fn eval_impl(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Variable(name) => env.get(&name),
            Expr::List(exprs) => self.eval_list(exprs, env),
            Expr::Quote(expr) => self.eval_quote(*expr),
            Expr::Quasiquote(expr) => self.eval_quasiquote(*expr, env),
            Expr::Unquote(_) => Err(LambdustError::syntax_error(
                "unquote outside of quasiquote".to_string(),
            )),
            Expr::UnquoteSplicing(_) => Err(LambdustError::syntax_error(
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

        // Try to evaluate as special form first
        if let Some(result) = self.try_eval_special_form(operator, operands, env.clone())? {
            return Ok(result);
        }

        // Evaluate as procedure call
        self.eval_procedure_call(operator, operands, env)
    }

    /// Try to evaluate as a special form
    fn try_eval_special_form(
        &mut self,
        operator: &Expr,
        operands: &[Expr],
        env: Rc<Environment>,
    ) -> Result<Option<Value>> {
        let Expr::Variable(name) = operator else {
            return Ok(None);
        };

        let result = match name.as_str() {
            "quote" => self.eval_quote_special(operands)?,
            "define" => self.eval_define(operands, env)?,
            "set!" => self.eval_set(operands, env)?,
            "lambda" => self.eval_lambda(operands, env)?,
            "if" => self.eval_if(operands, env)?,
            "and" => self.eval_and(operands, env)?,
            "or" => self.eval_or(operands, env)?,
            "begin" => self.eval_begin(operands, env)?,
            "do" => self.eval_do(operands, env)?,
            "apply" => self.eval_apply(operands, env)?,
            "map" => self.eval_map(operands, env)?,
            "for-each" => self.eval_for_each(operands, env)?,
            "call-with-values" => self.eval_call_with_values(operands, env)?,
            "delay" => self.eval_delay(operands, env)?,
            "lazy" => self.eval_lazy(operands, env)?,
            "force" => self.eval_force(operands, env)?,
            "syntax-rules" => self.eval_syntax_rules(operands, env)?,
            "call-with-current-continuation" | "call/cc" => self.eval_call_cc(operands, env)?,
            _ => return Ok(None), // Not a special form
        };

        Ok(Some(result))
    }

    /// Evaluate a regular procedure call
    fn eval_procedure_call(
        &mut self,
        operator: &Expr,
        operands: &[Expr],
        env: Rc<Environment>,
    ) -> Result<Value> {
        let proc = self.eval_impl(operator.clone(), env.clone())?;
        let args = self.eval_operands(operands, env.clone())?;
        self.apply_procedure(proc, args, env)
    }

    /// Public interface for calling procedures from host code
    pub fn call_procedure(&mut self, proc: Value, args: Vec<Value>) -> Result<Value> {
        self.apply_procedure(proc, args, self.global_env.clone())
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
                            return Err(LambdustError::arity_error(expected, args.len()));
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

                    // Evaluate body expressions with tail call optimization
                    if body.is_empty() {
                        return Ok(Value::Undefined);
                    }

                    // Evaluate all expressions except the last one
                    for expr in &body[..body.len() - 1] {
                        self.eval_impl(expr.clone(), Rc::new(new_env.clone()))?;
                    }

                    // The last expression is in tail position
                    if let Some(last_expr) = body.last() {
                        self.eval_with_tail_optimization(last_expr.clone(), Rc::new(new_env))
                    } else {
                        Ok(Value::Undefined)
                    }
                }
                Procedure::HostFunction { func, .. } => {
                    // Arity is already validated by HostFunctionRegistry wrapper
                    func(&args)
                }
                Procedure::Continuation { continuation } => {
                    // Apply continuation with the given arguments
                    // Continuations can accept multiple values
                    let result_value = if args.len() == 1 {
                        // Single value case
                        args[0].clone()
                    } else {
                        // Multiple values case - wrap in Values
                        Value::Values(args)
                    };

                    // Apply the continuation (this would jump to the captured context)
                    // For now, we'll use the simplified implementation
                    self.apply_continuation(*continuation.clone(), result_value)
                }
            },
            _ => Err(LambdustError::type_error(format!(
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
            return Err(LambdustError::arity_error(1, operands.len()));
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
            _ => Err(LambdustError::runtime_error(
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
            return Err(LambdustError::syntax_error(
                "define: too few arguments".to_string(),
            ));
        }

        match &operands[0] {
            // (define var value)
            Expr::Variable(name) => {
                if operands.len() != 2 {
                    return Err(LambdustError::syntax_error(
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
                    return Err(LambdustError::syntax_error(
                        "define: empty function definition".to_string(),
                    ));
                }

                let name = match &def_exprs[0] {
                    Expr::Variable(n) => n.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "define: invalid function name".to_string(),
                        ));
                    }
                };

                let params: Result<Vec<String>> = def_exprs[1..]
                    .iter()
                    .map(|expr| match expr {
                        Expr::Variable(param) => Ok(param.clone()),
                        _ => Err(LambdustError::syntax_error(
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
            _ => Err(LambdustError::syntax_error(
                "define: invalid syntax".to_string(),
            )),
        }
    }

    /// Evaluate set! special form
    fn eval_set(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let name = match &operands[0] {
            Expr::Variable(n) => n,
            _ => {
                return Err(LambdustError::syntax_error(
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
            return Err(LambdustError::syntax_error(
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
                        _ => Err(LambdustError::syntax_error(
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
                        _ => Err(LambdustError::syntax_error(
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
                        return Err(LambdustError::syntax_error(
                            "lambda: invalid rest parameter".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(LambdustError::syntax_error(
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
            _ => Err(LambdustError::arity_error(2, operands.len())),
        }
    }

    /// Tail call version of if special form
    fn eval_if_tail(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<TailCallInfo> {
        match operands.len() {
            2 => {
                let test = self.eval_impl(operands[0].clone(), env.clone())?;
                if test.is_truthy() {
                    self.eval_impl_tail(operands[1].clone(), env)
                } else {
                    Ok(TailCallInfo::None)
                }
            }
            3 => {
                let test = self.eval_impl(operands[0].clone(), env.clone())?;
                if test.is_truthy() {
                    self.eval_impl_tail(operands[1].clone(), env)
                } else {
                    self.eval_impl_tail(operands[2].clone(), env)
                }
            }
            _ => Ok(TailCallInfo::None),
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

    /// Tail call version of begin special form
    fn eval_begin_tail(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<TailCallInfo> {
        if operands.is_empty() {
            return Ok(TailCallInfo::None);
        }

        // Normal evaluation for all expressions except the last one
        for expr in &operands[..operands.len() - 1] {
            self.eval_impl(expr.clone(), env.clone())?;
        }

        // The last expression is eligible for tail call optimization
        if let Some(last_expr) = operands.last() {
            self.eval_impl_tail(last_expr.clone(), env)
        } else {
            Ok(TailCallInfo::None)
        }
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
        Err(LambdustError::runtime_error(
            "do not yet implemented".to_string(),
        ))
    }

    /// Evaluate apply special form: (apply proc args)
    fn eval_apply(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc = self.eval_impl(operands[0].clone(), env.clone())?;
        let args_value = self.eval_impl(operands[1].clone(), env.clone())?;

        // Convert arguments to vector
        let arguments = match args_value.to_vector() {
            Some(vec) => vec,
            None => {
                return Err(LambdustError::type_error(format!(
                    "apply: expected list of arguments, got {}",
                    args_value
                )));
            }
        };

        // Apply the procedure to the arguments
        self.apply_procedure(proc, arguments, env)
    }

    /// Evaluate map special form: (map proc list)
    fn eval_map(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc = self.eval_impl(operands[0].clone(), env.clone())?;
        let list_value = self.eval_impl(operands[1].clone(), env.clone())?;

        // Convert list to vector
        let elements = match list_value.to_vector() {
            Some(vec) => vec,
            None => {
                return Err(LambdustError::type_error(format!(
                    "map: expected list, got {}",
                    list_value
                )));
            }
        };

        let mut result = Vec::new();

        // Apply procedure to each element
        for element in elements {
            let mapped_value = self.apply_procedure(proc.clone(), vec![element], env.clone())?;
            result.push(mapped_value);
        }

        Ok(Value::from_vector(result))
    }

    /// Evaluate for-each special form: (for-each proc list)
    fn eval_for_each(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let proc = self.eval_impl(operands[0].clone(), env.clone())?;
        let list_value = self.eval_impl(operands[1].clone(), env.clone())?;

        // Convert list to vector
        let elements = match list_value.to_vector() {
            Some(vec) => vec,
            None => {
                return Err(LambdustError::type_error(format!(
                    "for-each: expected list, got {}",
                    list_value
                )));
            }
        };

        // Apply procedure to each element (for side effects)
        for element in elements {
            self.apply_procedure(proc.clone(), vec![element], env.clone())?;
        }

        Ok(Value::Undefined)
    }

    /// Evaluate call-with-values special form: (call-with-values producer consumer)
    fn eval_call_with_values(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let producer = self.eval_impl(operands[0].clone(), env.clone())?;
        let consumer = self.eval_impl(operands[1].clone(), env.clone())?;

        // Call the producer procedure with no arguments
        let producer_result = self.apply_procedure(producer, vec![], env.clone())?;

        // Convert the result to arguments for the consumer
        let consumer_args = match producer_result {
            Value::Values(values) => values,
            single_value => vec![single_value],
        };

        // Call the consumer with the values from the producer
        self.apply_procedure(consumer, consumer_args, env)
    }

    /// Evaluate delay special form: (delay expr)
    fn eval_delay(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        // Create a lazy promise without evaluating the expression
        Ok(crate::builtins::lazy::make_lazy_promise(
            operands[0].clone(),
            env,
        ))
    }

    /// Evaluate lazy special form: (lazy expr)
    fn eval_lazy(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        // lazy is similar to delay but for SRFI 45 semantics
        // For now, treat it the same as delay
        Ok(crate::builtins::lazy::make_lazy_promise(
            operands[0].clone(),
            env,
        ))
    }

    /// Apply a continuation with a value (supporting multiple values)
    fn apply_continuation(&mut self, _continuation: Continuation, value: Value) -> Result<Value> {
        // For now, this is a simplified implementation
        // A full implementation would restore the call stack and environment
        // and jump to the continuation point

        // In a complete implementation, this would:
        // 1. Restore the captured environment
        // 2. Restore the call stack
        // 3. Return the value to the continuation point

        // For this basic implementation, we'll just return the value
        // This allows call/cc to work in simple cases
        Ok(value)
    }

    /// Evaluate call-with-current-continuation special form: (call/cc proc)
    fn eval_call_cc(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let proc = self.eval_impl(operands[0].clone(), env.clone())?;

        // Verify that the argument is a procedure
        match &proc {
            Value::Procedure(_) => {}
            _ => {
                return Err(LambdustError::type_error(format!(
                    "call/cc: expected procedure, got {}",
                    proc
                )));
            }
        }

        // Create a continuation that captures the current evaluation context
        let continuation = Continuation {
            stack: Vec::new(), // Simplified - full implementation would capture actual call stack
            env: env.clone(),
        };

        let continuation_proc = Value::Procedure(Procedure::Continuation {
            continuation: Box::new(continuation),
        });

        // Call the procedure with the continuation as its argument
        self.apply_procedure(proc, vec![continuation_proc], continuation_proc)
    }

    /// Evaluate force special form: (force promise)
    fn eval_force(&mut self, operands: &[Expr], env: Rc<Environment>) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let promise_value = self.eval_impl(operands[0].clone(), env)?;

        match promise_value {
            Value::Promise(promise) => crate::builtins::lazy::force_promise(&promise, self),
            // If it's not a promise, just return the value (per SRFI 45)
            value => Ok(value),
        }
    }

    /// Evaluate syntax-rules special form: (syntax-rules (literals) (pattern template) ...)
    fn eval_syntax_rules(&mut self, operands: &[Expr], _env: Rc<Environment>) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "syntax-rules: requires at least literals and one rule".to_string(),
            ));
        }

        // Parse literals list
        let _literals = match &operands[0] {
            Expr::List(_) => {
                // For now, we'll ignore literals processing
                // A full implementation would parse and store these
                &operands[0]
            }
            _ => {
                return Err(LambdustError::syntax_error(
                    "syntax-rules: literals must be a list".to_string(),
                ));
            }
        };

        // Parse rules (pattern template pairs)
        let mut _rules: Vec<(crate::macros::Pattern, crate::macros::Template)> = Vec::new();
        for rule in &operands[1..] {
            match rule {
                Expr::List(rule_parts) if rule_parts.len() == 2 => {
                    // Parse pattern and template using SRFI 46 extensions
                    let _pattern = self.macro_expander.parse_pattern_srfi46(&rule_parts[0])?;
                    let _template = self.macro_expander.parse_template_srfi46(&rule_parts[1])?;
                    // Store rules for later use
                }
                _ => {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: each rule must be (pattern template)".to_string(),
                    ));
                }
            }
        }

        // For now, return a placeholder macro procedure
        // A complete implementation would create a proper macro transformer
        Ok(Value::Procedure(crate::value::Procedure::Builtin {
            name: "syntax-rules-macro".to_string(),
            arity: None,
            func: |_args| {
                Err(LambdustError::runtime_error(
                    "syntax-rules macro not yet fully implemented".to_string(),
                ))
            },
        }))
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

    #[test]
    fn test_eval_apply() {
        // Test apply with built-in function
        let result = eval_str("(apply + '(1 2 3))").unwrap();
        assert_eq!(result, Value::from(6i64));

        let result = eval_str("(apply * '(2 3 4))").unwrap();
        assert_eq!(result, Value::from(24i64));
    }

    #[test]
    fn test_eval_map() {
        // Test map with built-in function (abs)
        let result = eval_str("(map abs '(-1 -2 3 -4))").unwrap();
        assert_eq!(
            result,
            Value::from_vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64),
                Value::from(4i64)
            ])
        );
    }

    #[test]
    fn test_eval_for_each() {
        // Test for-each - it should return undefined
        let result = eval_str("(for-each abs '(1 2 3))").unwrap();
        assert_eq!(result, Value::Undefined);
    }

    #[test]
    fn test_higher_order_with_lambda() {
        // Test apply with user-defined lambda
        let result = eval_str("(apply (lambda (x y) (+ x y)) '(3 4))").unwrap();
        assert_eq!(result, Value::from(7i64));

        // Test map with user-defined lambda
        let result = eval_str("(map (lambda (x) (* x x)) '(1 2 3 4))").unwrap();
        assert_eq!(
            result,
            Value::from_vector(vec![
                Value::from(1i64),
                Value::from(4i64),
                Value::from(9i64),
                Value::from(16i64)
            ])
        );
    }

    #[test]
    fn test_values_and_call_with_values() {
        // Test values function
        let result = eval_str("(values 1 2 3)").unwrap();
        assert_eq!(
            result,
            Value::Values(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );

        // Test call-with-values with single value
        let result = eval_str("(call-with-values (lambda () 42) (lambda (x) x))").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test call-with-values with multiple values
        let result =
            eval_str("(call-with-values (lambda () (values 1 2 3)) (lambda (x y z) (+ x y z)))")
                .unwrap();
        assert_eq!(result, Value::from(6i64));

        // Test call-with-values with values producer
        let result =
            eval_str("(call-with-values (lambda () (values 10 20)) (lambda (a b) (* a b)))")
                .unwrap();
        assert_eq!(result, Value::from(200i64));
    }

    #[test]
    fn test_call_with_values_errors() {
        // Test call-with-values with wrong arity
        let result = eval_str("(call-with-values)");
        assert!(result.is_err());

        let result = eval_str("(call-with-values (lambda () 1))");
        assert!(result.is_err());

        let result = eval_str("(call-with-values (lambda () 1) (lambda (x) x) extra)");
        assert!(result.is_err());

        // Test call-with-values with non-procedure arguments
        let result = eval_str("(call-with-values 42 (lambda (x) x))");
        assert!(result.is_err());

        let result = eval_str("(call-with-values (lambda () 1) 42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_call_cc_errors() {
        // Test call/cc with wrong arity
        let result = eval_str("(call/cc)");
        assert!(result.is_err());

        let result = eval_str("(call/cc (lambda (k) k) extra)");
        assert!(result.is_err());

        // Test call/cc with non-procedure argument
        let result = eval_str("(call/cc 42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_call_cc_basic() {
        // Test basic call/cc that doesn't use the continuation
        let result = eval_str("(call/cc (lambda (k) 42))").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test call/cc with identity function
        let result = eval_str("(call/cc (lambda (cont) (cont 100)))").unwrap();
        assert_eq!(result, Value::from(100i64));
    }

    #[test]
    fn test_continuation_multi_values() {
        // Test continuation with multiple values
        let result = eval_str("(call/cc (lambda (cont) (cont 1 2 3)))").unwrap();
        assert_eq!(
            result,
            Value::Values(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );

        // Test continuation in call-with-values context
        let result = eval_str(
            "(call-with-values 
               (lambda () (call/cc (lambda (cont) (cont 5 10))))
               (lambda (a b) (+ a b)))",
        )
        .unwrap();
        assert_eq!(result, Value::from(15i64));
    }

    #[test]
    fn test_srfi_45_lazy_evaluation() {
        // Test delay creates a promise
        let result = eval_str("(promise? (delay (+ 1 2)))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test force evaluates a delayed expression
        let result = eval_str("(force (delay (+ 1 2)))").unwrap();
        assert_eq!(result, Value::from(3i64));

        // Test force on non-promise returns the value
        let result = eval_str("(force 42)").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test lazy creates a promise
        let result = eval_str("(promise? (lazy (+ 3 4)))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test nested delay/force
        let result = eval_str("(force (force (delay (delay (+ 5 6)))))").unwrap();
        assert_eq!(result, Value::from(11i64));
    }

    #[test]
    fn test_srfi_46_syntax_rules() {
        // Test basic syntax-rules parsing
        let result = eval_str("(syntax-rules () ((test x) x))");
        assert!(result.is_ok());

        // The result should be a procedure (macro transformer)
        assert!(result.unwrap().is_procedure());

        // Test syntax-rules with literals
        let result = eval_str("(syntax-rules (else) ((test x) x))");
        assert!(result.is_ok());

        // Test invalid syntax-rules (missing rules)
        let result = eval_str("(syntax-rules ())");
        assert!(result.is_err());
    }
}
