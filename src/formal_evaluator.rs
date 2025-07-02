//! R7RS formal semantics compliant evaluator
//!
//! This module implements a continuation-passing style evaluator
//! that strictly follows the R7RS formal semantics definition.

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Dynamic point for dynamic-wind semantics
#[derive(Debug, Clone)]
pub struct DynamicPoint {
    /// Before thunk
    pub before: Option<Value>,
    /// After thunk
    pub after: Option<Value>,
    /// Parent dynamic point
    pub parent: Option<Box<DynamicPoint>>,
}

/// Continuation representation following R7RS semantics
#[derive(Debug, Clone)]
pub enum Continuation {
    /// Identity continuation (final result)
    Identity,
    /// Function application continuation
    Application {
        /// Operator to apply
        operator: Value,
        /// Evaluated arguments so far
        evaluated_args: Vec<Value>,
        /// Remaining arguments to evaluate
        remaining_args: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Operator evaluation continuation
    Operator {
        /// Arguments to evaluate after operator
        args: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// If test continuation
    IfTest {
        /// Consequent expression
        consequent: Expr,
        /// Alternate expression (if any)
        alternate: Option<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Assignment continuation
    Assignment {
        /// Variable to assign
        variable: String,
        /// Environment for assignment
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Values continuation (for multiple values)
    Values {
        /// Values accumulated so far
        values: Vec<Value>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Call-with-values step 1: evaluate consumer, then producer
    CallWithValuesStep1 {
        /// Producer expression to evaluate later
        producer_expr: Expr,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Call-with-values step 2: call producer, then consumer
    CallWithValuesStep2 {
        /// Consumer procedure (already evaluated)
        consumer: Value,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
}

/// Store (memory) for locations
#[derive(Debug, Clone)]
pub struct Store {
    /// Mapping from locations to values
    locations: HashMap<usize, Value>,
    /// Next available location
    next_location: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    /// Create a new store
    pub fn new() -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
        }
    }

    /// Allocate a new location
    pub fn allocate(&mut self, value: Value) -> usize {
        let loc = self.next_location;
        self.locations.insert(loc, value);
        self.next_location += 1;
        loc
    }

    /// Get value at location
    pub fn get(&self, location: usize) -> Option<&Value> {
        self.locations.get(&location)
    }

    /// Set value at location
    pub fn set(&mut self, location: usize, value: Value) -> Result<()> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.locations.entry(location) {
            e.insert(value);
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )))
        }
    }
}

/// Evaluation order strategy for modeling unspecified order
#[derive(Debug, Clone)]
pub enum EvalOrder {
    /// Left-to-right evaluation
    LeftToRight,
    /// Right-to-left evaluation
    RightToLeft,
    /// Random/unspecified order (for testing compliance)
    Unspecified,
}

/// Formal evaluator implementing R7RS semantics
#[derive(Debug)]
pub struct FormalEvaluator {
    /// Current store (memory)
    #[allow(dead_code)]
    store: Store,
    /// Dynamic points stack
    #[allow(dead_code)]
    dynamic_points: Vec<DynamicPoint>,
    /// Evaluation order strategy
    eval_order: EvalOrder,
}

impl FormalEvaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        FormalEvaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order: EvalOrder::LeftToRight,
        }
    }

    /// Create a new formal evaluator with specific evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        FormalEvaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order,
        }
    }

    /// Main evaluation function: E[e]ρκσ
    /// Where:
    /// - e: expression to evaluate
    /// - ρ: environment
    /// - κ: continuation
    /// - σ: store
    pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        match expr {
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

            // Other forms
            _ => Err(LambdustError::syntax_error(format!(
                "Unsupported expression: {expr:?}"
            ))),
        }
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

    /// Evaluate variable: I[I]ρ
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

    /// Evaluate function application
    fn eval_application(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return Err(LambdustError::syntax_error("Empty application".to_string()));
        }

        // Check for special forms first
        if let Some(special_result) =
            self.try_eval_special_form(&exprs, env.clone(), cont.clone())?
        {
            return Ok(special_result);
        }

        // Regular function application
        let operator = exprs[0].clone();
        let args = exprs[1..].to_vec();

        // Create continuation for operator evaluation
        let operator_cont = Continuation::Operator {
            args,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(operator, env, operator_cont)
    }

    /// Try to evaluate as special form
    fn try_eval_special_form(
        &mut self,
        exprs: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        if let Expr::Variable(name) = &exprs[0] {
            match name.as_str() {
                "lambda" => return Ok(Some(self.eval_lambda(&exprs[1..], env, cont)?)),
                "if" => return Ok(Some(self.eval_if(&exprs[1..], env, cont)?)),
                "set!" => return Ok(Some(self.eval_set(&exprs[1..], env, cont)?)),
                "quote" => {
                    if exprs.len() != 2 {
                        return Err(LambdustError::arity_error(1, exprs.len() - 1));
                    }
                    return Ok(Some(self.eval_quote(exprs[1].clone(), cont)?));
                }
                "values" => return Ok(Some(self.eval_values(&exprs[1..], env, cont)?)),
                "call-with-values" => {
                    return Ok(Some(self.eval_call_with_values(&exprs[1..], env, cont)?));
                }
                "dynamic-wind" => {
                    return Ok(Some(self.eval_dynamic_wind(&exprs[1..], env, cont)?));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Evaluate lambda expression
    fn eval_lambda(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: too few arguments".to_string(),
            ));
        }

        // Parse parameters
        let (params, variadic) = self.parse_lambda_params(&operands[0])?;
        let body = operands[1..].to_vec();

        let lambda = Value::Procedure(Procedure::Lambda {
            params,
            variadic,
            body,
            closure: env,
        });

        self.apply_continuation(cont, lambda)
    }

    /// Parse lambda parameters
    fn parse_lambda_params(&self, params_expr: &Expr) -> Result<(Vec<String>, bool)> {
        match params_expr {
            // (param1 param2 ...)
            Expr::List(params) => {
                let mut param_names = Vec::new();
                for param in params {
                    if let Expr::Variable(name) = param {
                        param_names.push(name.clone());
                    } else {
                        return Err(LambdustError::syntax_error(
                            "lambda: invalid parameter".to_string(),
                        ));
                    }
                }
                Ok((param_names, false))
            }
            // param (variadic)
            Expr::Variable(param) => Ok((vec![param.clone()], true)),
            // (param1 param2 . rest)
            Expr::DottedList(params, rest) => {
                let mut param_names = Vec::new();
                for param in params {
                    if let Expr::Variable(name) = param {
                        param_names.push(name.clone());
                    } else {
                        return Err(LambdustError::syntax_error(
                            "lambda: invalid parameter".to_string(),
                        ));
                    }
                }
                if let Expr::Variable(rest_name) = rest.as_ref() {
                    param_names.push(rest_name.clone());
                    Ok((param_names, true))
                } else {
                    Err(LambdustError::syntax_error(
                        "lambda: invalid rest parameter".to_string(),
                    ))
                }
            }
            _ => Err(LambdustError::syntax_error(
                "lambda: invalid parameter list".to_string(),
            )),
        }
    }

    /// Evaluate if expression
    fn eval_if(
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

        self.eval(test, env, if_cont)
    }

    /// Evaluate set! expression
    fn eval_set(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let var_name = match &operands[0] {
            Expr::Variable(name) => name.clone(),
            _ => {
                return Err(LambdustError::syntax_error(
                    "set!: not a variable".to_string(),
                ));
            }
        };

        let value_expr = operands[1].clone();

        let set_cont = Continuation::Assignment {
            variable: var_name,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(value_expr, env, set_cont)
    }

    /// Evaluate quote expression
    fn eval_quote(&mut self, expr: Expr, cont: Continuation) -> Result<Value> {
        let quoted_value = self.expr_to_value(expr)?;
        self.apply_continuation(cont, quoted_value)
    }

    /// Convert expression to value (for quote)
    #[allow(clippy::only_used_in_recursion)]
    fn expr_to_value(&self, expr: Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Boolean(b) => Ok(Value::Boolean(b)),
                Literal::Number(n) => Ok(Value::Number(n)),
                Literal::String(s) => Ok(Value::String(s)),
                Literal::Character(c) => Ok(Value::Character(c)),
                Literal::Nil => Ok(Value::Nil),
            },
            Expr::Variable(name) => Ok(Value::Symbol(name)),
            Expr::List(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.expr_to_value(expr)?);
                }
                Ok(Value::from_vector(values))
            }
            _ => Err(LambdustError::runtime_error(
                "Cannot quote this expression".to_string(),
            )),
        }
    }

    /// Evaluate values expression
    fn eval_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Values(vec![]));
        }

        // Evaluate all operands and collect them into a Values object
        let mut values = Vec::new();
        for operand in operands {
            let value = self.eval(operand.clone(), env.clone(), Continuation::Identity)?;
            values.push(value);
        }

        self.apply_continuation(cont, Value::Values(values))
    }

    /// Evaluate a sequence of expressions for values
    fn eval_sequence(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_continuation(cont, Value::Values(vec![]));
        }

        if exprs.len() == 1 {
            return self.eval(exprs[0].clone(), env, cont);
        }

        // Evaluate first expression
        let first = exprs[0].clone();
        let rest = exprs[1..].to_vec();

        // Create continuation for the rest
        let seq_cont = self.create_sequence_continuation(rest, env.clone(), cont);
        self.eval(first, env, seq_cont)
    }

    /// Create a continuation for sequence evaluation
    fn create_sequence_continuation(
        &self,
        _remaining: Vec<Expr>,
        _env: Rc<Environment>,
        final_cont: Continuation,
    ) -> Continuation {
        // For now, simplified implementation
        // A full implementation would create proper sequence continuations
        final_cont
    }

    /// Evaluate call-with-values
    fn eval_call_with_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let producer_expr = operands[0].clone();
        let consumer_expr = operands[1].clone();

        // Create a continuation for step 1: after consumer is evaluated, evaluate producer
        let step1_cont = Continuation::CallWithValuesStep1 {
            producer_expr,
            env: env.clone(),
            parent: Box::new(cont),
        };

        // First evaluate the consumer
        self.eval(consumer_expr, env, step1_cont)
    }

    /// Evaluate dynamic-wind
    fn eval_dynamic_wind(
        &mut self,
        operands: &[Expr],
        _env: Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        // This is a simplified implementation
        // A full implementation would manage dynamic points properly
        Err(LambdustError::runtime_error(
            "dynamic-wind: not yet fully implemented in formal evaluator".to_string(),
        ))
    }

    /// Apply continuation: κ(v)
    fn apply_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            Continuation::Identity => Ok(value),

            Continuation::Operator { args, env, parent } => {
                // Operator has been evaluated, now evaluate arguments
                self.eval_args(value, args, Vec::new(), env, *parent)
            }

            Continuation::Application {
                operator,
                mut evaluated_args,
                remaining_args,
                env,
                parent,
            } => {
                // Another argument has been evaluated
                evaluated_args.push(value);

                if remaining_args.is_empty() {
                    // All arguments evaluated, apply the procedure
                    self.apply_procedure(operator, evaluated_args, *parent)
                } else {
                    // More arguments to evaluate
                    let next_arg = remaining_args[0].clone();
                    let rest_args = remaining_args[1..].to_vec();

                    let app_cont = Continuation::Application {
                        operator,
                        evaluated_args,
                        remaining_args: rest_args,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_arg, env, app_cont)
                }
            }

            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => {
                // Test has been evaluated
                if value.is_truthy() {
                    self.eval(consequent, env, *parent)
                } else if let Some(alt) = alternate {
                    self.eval(alt, env, *parent)
                } else {
                    self.apply_continuation(*parent, Value::Undefined)
                }
            }

            Continuation::Assignment {
                variable,
                env,
                parent,
            } => {
                // Value has been evaluated, perform assignment
                env.set(&variable, value)?;
                self.apply_continuation(*parent, Value::Undefined)
            }

            Continuation::Values { mut values, parent } => {
                // Accumulate value for multiple values
                values.push(value);
                self.apply_continuation(*parent, Value::Values(values))
            }

            Continuation::CallWithValuesStep1 {
                producer_expr,
                env,
                parent,
            } => {
                // Consumer has been evaluated (value), now evaluate producer
                let consumer = value;
                let step2_cont = Continuation::CallWithValuesStep2 {
                    consumer,
                    env: env.clone(),
                    parent,
                };

                // Evaluate the producer
                self.eval(producer_expr, env, step2_cont)
            }

            Continuation::CallWithValuesStep2 {
                consumer,
                env: _,
                parent,
            } => {
                // Producer has been evaluated (value), now call it and then apply consumer
                let producer = value;

                // Call the producer with no arguments to get the values
                let producer_result =
                    self.apply_procedure(producer, Vec::new(), Continuation::Identity)?;

                // Convert the result to arguments for the consumer
                let consumer_args = match producer_result {
                    Value::Values(values) => values,
                    single_value => vec![single_value],
                };

                // Apply the consumer with the producer's values
                self.apply_procedure(consumer, consumer_args, *parent)
            }
        }
    }

    /// Evaluate arguments for function application
    /// Implements unspecified evaluation order per R7RS semantics
    fn eval_args(
        &mut self,
        operator: Value,
        args: Vec<Expr>,
        evaluated_args: Vec<Value>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if args.is_empty() {
            // No arguments to evaluate, apply immediately
            return self.apply_procedure(operator, evaluated_args, cont);
        }

        // Apply evaluation order strategy
        let ordered_args = self.apply_evaluation_order(args);

        // Evaluate first argument in the ordered sequence
        let first_arg = ordered_args[0].clone();
        let rest_args = ordered_args[1..].to_vec();

        let app_cont = Continuation::Application {
            operator,
            evaluated_args,
            remaining_args: rest_args,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_arg, env, app_cont)
    }

    /// Apply evaluation order strategy to arguments
    /// This models the "unspecified order" semantics in R7RS
    fn apply_evaluation_order(&self, mut args: Vec<Expr>) -> Vec<Expr> {
        match self.eval_order {
            EvalOrder::LeftToRight => args, // Natural order
            EvalOrder::RightToLeft => {
                args.reverse();
                args
            }
            EvalOrder::Unspecified => {
                // For demonstration, we alternate between left-to-right and right-to-left
                // In a real implementation, this could be truly non-deterministic
                if args.len() % 2 == 0 {
                    args.reverse();
                }
                args
            }
        }
    }

    /// Apply procedure to arguments
    fn apply_procedure(
        &mut self,
        operator: Value,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        match operator {
            Value::Procedure(proc) => match proc {
                Procedure::Builtin { func, arity, .. } => {
                    // Check arity
                    if let Some(expected) = arity {
                        if args.len() != expected {
                            return Err(LambdustError::arity_error(expected, args.len()));
                        }
                    }
                    let result = func(&args)?;
                    self.apply_continuation(cont, result)
                }
                Procedure::Lambda {
                    params,
                    variadic,
                    body,
                    closure,
                } => {
                    // Create new environment with parameter bindings
                    let new_env = closure.bind_parameters(&params, &args, variadic)?;

                    // Evaluate body
                    if body.is_empty() {
                        self.apply_continuation(cont, Value::Undefined)
                    } else if body.len() == 1 {
                        self.eval(body[0].clone(), Rc::new(new_env), cont)
                    } else {
                        // Evaluate sequence
                        self.eval_sequence(body, Rc::new(new_env), cont)
                    }
                }
                _ => Err(LambdustError::runtime_error(
                    "Unsupported procedure type".to_string(),
                )),
            },
            _ => Err(LambdustError::type_error(format!(
                "Not a procedure: {operator}"
            ))),
        }
    }
}

impl Default for FormalEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Public interface for formal evaluation
pub fn eval_with_formal_semantics(expr: Expr, env: Rc<Environment>) -> Result<Value> {
    let mut evaluator = FormalEvaluator::new();
    evaluator.eval(expr, env, Continuation::Identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn eval_str_formal(input: &str) -> Result<Value> {
        let tokens = tokenize(input)?;
        let ast = parse(tokens)?;
        let env = Rc::new(Environment::with_builtins());
        eval_with_formal_semantics(ast, env)
    }

    #[test]
    fn test_formal_literals() {
        assert_eq!(eval_str_formal("42").unwrap(), Value::from(42i64));
        assert_eq!(eval_str_formal("#t").unwrap(), Value::Boolean(true));
        assert_eq!(eval_str_formal("\"hello\"").unwrap(), Value::from("hello"));
    }

    #[test]
    fn test_formal_quote() {
        assert_eq!(
            eval_str_formal("'x").unwrap(),
            Value::Symbol("x".to_string())
        );
        assert_eq!(
            eval_str_formal("'(1 2 3)").unwrap(),
            Value::from_vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );
    }

    #[test]
    fn test_formal_lambda() {
        let result = eval_str_formal("(lambda (x) x)").unwrap();
        assert!(result.is_procedure());
    }

    #[test]
    fn test_formal_if() {
        assert_eq!(eval_str_formal("(if #t 1 2)").unwrap(), Value::from(1i64));
        assert_eq!(eval_str_formal("(if #f 1 2)").unwrap(), Value::from(2i64));
    }

    #[test]
    fn test_formal_values() {
        let result = eval_str_formal("(values 1 2 3)").unwrap();
        assert_eq!(
            result,
            Value::Values(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );
    }

    #[test]
    fn test_evaluation_order_strategies() {
        // Test different evaluation order strategies
        let mut eval_ltr = FormalEvaluator::with_eval_order(EvalOrder::LeftToRight);
        let mut eval_rtl = FormalEvaluator::with_eval_order(EvalOrder::RightToLeft);
        let mut eval_unspec = FormalEvaluator::with_eval_order(EvalOrder::Unspecified);

        let env = Rc::new(Environment::with_builtins());

        // Test literal evaluation (should be same regardless of order)
        let lit_expr = Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(42)));
        let cont = Continuation::Identity;

        let result_ltr = eval_ltr
            .eval(lit_expr.clone(), env.clone(), cont.clone())
            .unwrap();
        let result_rtl = eval_rtl
            .eval(lit_expr.clone(), env.clone(), cont.clone())
            .unwrap();
        let result_unspec = eval_unspec.eval(lit_expr, env, cont).unwrap();

        assert_eq!(result_ltr, Value::from(42i64));
        assert_eq!(result_rtl, Value::from(42i64));
        assert_eq!(result_unspec, Value::from(42i64));
    }

    #[test]
    fn test_argument_order_independence() {
        // Test that expressions that should be order-independent work correctly
        // (This is a simplified test - real order independence testing would be more complex)

        use crate::lexer::SchemeNumber;

        let args = vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ];

        let eval = FormalEvaluator::new();

        // Test left-to-right order
        let ordered_ltr = eval.apply_evaluation_order(args.clone());
        assert_eq!(ordered_ltr.len(), 3);

        // Test that reordering doesn't affect literal values
        let eval_rtl = FormalEvaluator::with_eval_order(EvalOrder::RightToLeft);
        let ordered_rtl = eval_rtl.apply_evaluation_order(args);
        assert_eq!(ordered_rtl.len(), 3);
    }

    #[test]
    fn test_formal_call_with_values() {
        // Test call-with-values with single value
        let result = eval_str_formal("(call-with-values (lambda () 42) (lambda (x) x))").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test call-with-values with multiple values
        let result = eval_str_formal(
            "(call-with-values (lambda () (values 1 2 3)) (lambda (x y z) (+ x y z)))",
        )
        .unwrap();
        assert_eq!(result, Value::from(6i64));

        // Test call-with-values with values producer
        let result =
            eval_str_formal("(call-with-values (lambda () (values 10 20)) (lambda (a b) (* a b)))")
                .unwrap();
        assert_eq!(result, Value::from(200i64));
    }

    #[test]
    fn test_formal_multi_value_continuations() {
        // Test that the formal evaluator properly handles multiple values in continuations
        // This ensures that the CPS implementation correctly propagates multi-value contexts
        
        // Test simple multi-value propagation
        let result = eval_str_formal("(values 1 2 3)").unwrap();
        assert_eq!(result, Value::Values(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64)
        ]));

        // Test multi-value in call-with-values (more complex CPS case)
        let result = eval_str_formal(
            "(call-with-values (lambda () (values 5 10 15)) (lambda (a b c) (+ a b c)))"
        ).unwrap();
        assert_eq!(result, Value::from(30i64));
    }
}
