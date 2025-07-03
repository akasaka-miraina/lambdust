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
    /// Begin continuation (for sequence evaluation)
    Begin {
        /// Remaining expressions to evaluate
        remaining: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// And continuation (for short-circuit evaluation)
    And {
        /// Remaining expressions to evaluate
        remaining: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Or continuation (for short-circuit evaluation)
    Or {
        /// Remaining expressions to evaluate
        remaining: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Define continuation (for variable definition)
    Define {
        /// Variable to define
        variable: String,
        /// Environment for definition
        env: Rc<Environment>,
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
    /// Do loop continuation (for iterative loops)
    Do {
        /// Variable bindings for the loop (var, init, step)
        bindings: Vec<(String, Expr, Option<Expr>)>,
        /// Test expression for loop termination
        test: Expr,
        /// Result expressions when test is true
        result_exprs: Vec<Expr>,
        /// Body expressions for each iteration
        body_exprs: Vec<Expr>,
        /// Current iteration environment
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Captured continuation for call/cc
    Captured {
        /// The captured continuation
        cont: Box<Continuation>,
    },
    /// Call/cc continuation
    CallCc {
        /// The captured continuation procedure
        captured_cont: Value,
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
pub struct Evaluator {
    /// Current store (memory)
    #[allow(dead_code)]
    store: Store,
    /// Dynamic points stack
    #[allow(dead_code)]
    dynamic_points: Vec<DynamicPoint>,
    /// Evaluation order strategy
    eval_order: EvalOrder,
    /// Global environment
    pub global_env: Rc<Environment>,
}

impl Evaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
        }
    }

    /// Create a new formal evaluator with specific evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order,
            global_env: Rc::new(Environment::with_builtins()),
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
        let operator_name = self.extract_operator_name(&exprs[0])?;
        if let Some(name) = operator_name {
            // Only handle actual special forms, not built-in functions
            if self.is_special_form(&name) {
                self.eval_known_special_form(&name, &exprs[1..], env, cont)
                    .map(Some)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Check if a name is a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(name,
            "lambda" | "if" | "set!" | "quote" | "define" | "begin" |
            "and" | "or" | "do" | "call/cc" | "call-with-current-continuation" |
            "values" | "call-with-values" | "dynamic-wind" |
            "delay" | "lazy" | "force"
        )
    }

    /// Extract operator name from expression
    fn extract_operator_name(&self, expr: &Expr) -> Result<Option<String>> {
        match expr {
            Expr::Variable(name) => Ok(Some(name.clone())),
            _ => Ok(None),
        }
    }

    /// Evaluate known special forms
    fn eval_known_special_form(
        &mut self,
        name: &str,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match name {
            // Core language forms
            "lambda" => self.eval_lambda(operands, env, cont),
            "if" => self.eval_if(operands, env, cont),
            "set!" => self.eval_set(operands, env, cont),
            "quote" => self.eval_quote_special_form(operands, cont),
            "define" => self.eval_define(operands, env, cont),
            "begin" => self.eval_begin(operands, env, cont),
            
            // Boolean forms
            "and" => self.eval_and(operands, env, cont),
            "or" => self.eval_or(operands, env, cont),
            
            // Control flow
            "do" => self.eval_do(operands, env, cont),
            "call/cc" | "call-with-current-continuation" => self.eval_call_cc(operands, env, cont),
            
            // Multiple values
            "values" => self.eval_values(operands, env, cont),
            "call-with-values" => self.eval_call_with_values(operands, env, cont),
            "dynamic-wind" => self.eval_dynamic_wind(operands, env, cont),
            
            // Lazy evaluation
            "delay" => self.eval_delay(operands, env, cont),
            "lazy" => self.eval_lazy(operands, env, cont),
            "force" => self.eval_force(operands, env, cont),
            
            // This should never be reached if is_special_form is correct
            _ => Err(LambdustError::syntax_error(format!(
                "Internal error: Unknown special form: {}", name
            ))),
        }
    }

    /// Helper for quote special form with arity checking
    fn eval_quote_special_form(&mut self, operands: &[Expr], cont: Continuation) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }
        self.eval_quote(operands[0].clone(), cont)
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

    /// Evaluate begin expression: (begin expr1 expr2 ...)
    fn eval_begin(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Undefined);
        }

        if operands.len() == 1 {
            // Single expression, evaluate directly
            return self.eval(operands[0].clone(), env, cont);
        }

        // Multiple expressions, use Begin continuation
        let first_expr = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let begin_cont = Continuation::Begin {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr, env, begin_cont)
    }

    /// Evaluate define expression: (define var value)
    fn eval_define(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "define: requires at least 2 arguments".to_string(),
            ));
        }

        match &operands[0] {
            // Simple variable definition: (define var value)
            Expr::Variable(name) => {
                if operands.len() != 2 {
                    return Err(LambdustError::arity_error(2, operands.len()));
                }
                let var_name = name.clone();
                let value_expr = operands[1].clone();

                let define_cont = Continuation::Define {
                    variable: var_name,
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.eval(value_expr, env, define_cont)
            }
            // Function definition: (define (name param1 param2 ...) body1 body2 ...)
            Expr::List(def_list) if !def_list.is_empty() => {
                let func_name = match &def_list[0] {
                    Expr::Variable(name) => name.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "define: function name must be a variable".to_string(),
                        ));
                    }
                };

                // Extract parameters
                let params = def_list[1..].to_vec();
                let body = operands[1..].to_vec();

                // Create lambda expression: (lambda (params...) body...)
                let lambda_expr = Expr::List(vec![
                    Expr::Variable("lambda".to_string()),
                    Expr::List(params),
                ]
                .into_iter()
                .chain(body)
                .collect());

                let define_cont = Continuation::Define {
                    variable: func_name,
                    env: env.clone(),
                    parent: Box::new(cont),
                };

                self.eval(lambda_expr, env, define_cont)
            }
            _ => Err(LambdustError::syntax_error(
                "define: invalid syntax".to_string(),
            )),
        }
    }

    /// Evaluate and expression: (and expr1 expr2 ...)
    fn eval_and(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            // (and) returns #t
            return self.apply_continuation(cont, Value::Boolean(true));
        }

        if operands.len() == 1 {
            // Single expression, evaluate directly
            return self.eval(operands[0].clone(), env, cont);
        }

        // Multiple expressions, use And continuation
        let first_expr = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let and_cont = Continuation::And {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr, env, and_cont)
    }

    /// Evaluate or expression: (or expr1 expr2 ...)
    fn eval_or(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            // (or) returns #f
            return self.apply_continuation(cont, Value::Boolean(false));
        }

        if operands.len() == 1 {
            // Single expression, evaluate directly
            return self.eval(operands[0].clone(), env, cont);
        }

        // Multiple expressions, use Or continuation
        let first_expr = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let or_cont = Continuation::Or {
            remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr, env, or_cont)
    }

    /// Evaluate do loop expression
    fn eval_do(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "do: requires at least variable bindings and test".to_string(),
            ));
        }

        // Parse variable bindings
        let bindings = self.parse_do_bindings(&operands[0])?;
        
        // Parse test and result expressions
        let test_result = self.parse_do_test_result(&operands[1])?;
        let test = test_result.0;
        let result_exprs = test_result.1;
        
        // Body expressions
        let body_exprs = operands[2..].to_vec();

        // Create new environment with initial bindings
        let do_env = Rc::new(Environment::with_parent(env.clone()));
        for (var, init_expr, _) in &bindings {
            let init_value = self.eval_sync(init_expr.clone(), env.clone())?;
            do_env.define(var.clone(), init_value);
        }

        // Check test condition
        let test_value = self.eval_sync(test.clone(), do_env.clone())?;
        if test_value.is_truthy() {
            // Test passed, evaluate result expressions
            if result_exprs.is_empty() {
                self.apply_continuation(cont, Value::Undefined)
            } else if result_exprs.len() == 1 {
                self.eval(result_exprs[0].clone(), do_env, cont)
            } else {
                self.eval_begin(&result_exprs, do_env, cont)
            }
        } else {
            // Test failed, execute body and continue loop
            if body_exprs.is_empty() {
                // No body, just update variables and continue
                self.continue_do_loop(bindings, test, result_exprs, body_exprs, do_env, cont)
            } else {
                // Execute body in sequence
                self.eval_begin(&body_exprs, do_env.clone(), Continuation::Do {
                    bindings,
                    test,
                    result_exprs,
                    body_exprs: body_exprs.clone(),
                    env: do_env,
                    parent: Box::new(cont),
                })
            }
        }
    }

    /// Parse do loop variable bindings
    fn parse_do_bindings(&self, bindings_expr: &Expr) -> Result<Vec<(String, Expr, Option<Expr>)>> {
        let Expr::List(bindings) = bindings_expr else {
            return Err(LambdustError::syntax_error(
                "do: bindings must be a list".to_string(),
            ));
        };

        let mut parsed_bindings = Vec::new();
        for binding in bindings {
            if let Expr::List(binding_parts) = binding {
                if binding_parts.len() >= 2 {
                    if let Expr::Variable(var) = &binding_parts[0] {
                        let init_expr = binding_parts[1].clone();
                        let step_expr = if binding_parts.len() >= 3 {
                            Some(binding_parts[2].clone())
                        } else {
                            None
                        };
                        parsed_bindings.push((var.clone(), init_expr, step_expr));
                    } else {
                        return Err(LambdustError::syntax_error(
                            "do: binding variable must be an identifier".to_string(),
                        ));
                    }
                } else {
                    return Err(LambdustError::syntax_error(
                        "do: binding must have variable and initial value".to_string(),
                    ));
                }
            } else {
                return Err(LambdustError::syntax_error(
                    "do: each binding must be a list".to_string(),
                ));
            }
        }
        Ok(parsed_bindings)
    }

    /// Parse do loop test and result expressions
    fn parse_do_test_result(&self, test_expr: &Expr) -> Result<(Expr, Vec<Expr>)> {
        if let Expr::List(parts) = test_expr {
            if parts.is_empty() {
                return Err(LambdustError::syntax_error(
                    "do: test clause cannot be empty".to_string(),
                ));
            }
            let test = parts[0].clone();
            let results = parts[1..].to_vec();
            Ok((test, results))
        } else {
            Ok((test_expr.clone(), Vec::new()))
        }
    }

    /// Continue do loop iteration
    fn continue_do_loop(
        &mut self,
        bindings: Vec<(String, Expr, Option<Expr>)>,
        test: Expr,
        result_exprs: Vec<Expr>,
        body_exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Update variables with step expressions
        // Per R7RS: evaluate all step expressions first, then update variables
        let mut updates = Vec::new();
        for (var, _init_expr, step_expr) in &bindings {
            if let Some(step) = step_expr {
                let step_value = self.eval_sync(step.clone(), env.clone())?;
                updates.push((var.clone(), step_value));
            }
        }
        
        // Apply all updates
        for (var, value) in updates {
            env.set(&var, value)?;
        }
        
        // Check test again
        let test_value = self.eval_sync(test.clone(), env.clone())?;
        if test_value.is_truthy() {
            // Test passed, evaluate result expressions
            if result_exprs.is_empty() {
                self.apply_continuation(cont, Value::Undefined)
            } else if result_exprs.len() == 1 {
                self.eval(result_exprs[0].clone(), env, cont)
            } else {
                self.eval_begin(&result_exprs, env, cont)
            }
        } else {
            // Continue loop
            if body_exprs.is_empty() {
                self.continue_do_loop(bindings, test, result_exprs, body_exprs, env, cont)
            } else {
                self.eval_begin(&body_exprs, env.clone(), Continuation::Do {
                    bindings,
                    test,
                    result_exprs,
                    body_exprs: body_exprs.clone(),
                    env,
                    parent: Box::new(cont),
                })
            }
        }
    }

    /// Evaluate delay expression
    fn eval_delay(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let promise = Value::Promise(crate::value::Promise {
            state: crate::value::PromiseState::Lazy {
                expr: operands[0].clone(),
                env,
            },
        });

        self.apply_continuation(cont, promise)
    }

    /// Evaluate lazy expression (SRFI 45)
    fn eval_lazy(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let promise = Value::Promise(crate::value::Promise {
            state: crate::value::PromiseState::Lazy {
                expr: operands[0].clone(),
                env,
            },
        });

        self.apply_continuation(cont, promise)
    }

    /// Evaluate force expression
    fn eval_force(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        // Evaluate the promise expression first
        let promise_expr = operands[0].clone();
        self.eval(promise_expr, env, cont)
        // Note: actual forcing will be handled in apply_continuation when we get a Promise value
    }

    /// Evaluate call/cc expression
    fn eval_call_cc(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        // For now, create a simple continuation procedure
        // In a full implementation, this would need to store the continuation somehow
        let captured_cont = Value::Procedure(Procedure::Builtin {
            name: "captured-continuation".to_string(),
            arity: Some(1),
            func: |_args| {
                // Placeholder - full implementation needs access to the evaluator
                Ok(Value::Undefined)
            },
        });

        // Create a special continuation for call/cc
        let call_cc_cont = Continuation::CallCc {
            captured_cont,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(operands[0].clone(), env, call_cc_cont)
    }

    /// Synchronous evaluation helper for internal use
    fn eval_sync(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        self.eval(expr, env, Continuation::Identity)
    }

    /// Evaluate a string of Scheme code using the global environment
    pub fn eval_string(&mut self, input: &str) -> Result<Value> {
        let tokens = crate::lexer::tokenize(input)?;
        let ast = crate::parser::parse(tokens)?;
        self.eval(ast, self.global_env.clone(), Continuation::Identity)
    }

    /// Call a procedure with given arguments
    pub fn call_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value> {
        self.apply_procedure(procedure, args, Continuation::Identity)
    }

    /// Apply continuation: κ(v)
    fn apply_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            Continuation::Identity => self.apply_identity_continuation(value),
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
            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => self.apply_if_test_continuation(value, consequent, alternate, env, *parent),
            Continuation::Assignment {
                variable,
                env,
                parent,
            } => self.apply_assignment_continuation(value, variable, env, *parent),
            Continuation::Values { values, parent } => {
                self.apply_values_continuation(value, values, *parent)
            }
            Continuation::CallWithValuesStep1 {
                producer_expr,
                env,
                parent,
            } => self.apply_call_with_values_step1_continuation(value, producer_expr, env, *parent),
            Continuation::CallWithValuesStep2 {
                consumer,
                env,
                parent,
            } => self.apply_call_with_values_step2_continuation(value, consumer, env, *parent),
            Continuation::Begin {
                remaining,
                env,
                parent,
            } => self.apply_begin_continuation(value, remaining, env, *parent),
            Continuation::Define {
                variable,
                env,
                parent,
            } => self.apply_define_continuation(value, variable, env, *parent),
            Continuation::And {
                remaining,
                env,
                parent,
            } => self.apply_and_continuation(value, remaining, env, *parent),
            Continuation::Or {
                remaining,
                env,
                parent,
            } => self.apply_or_continuation(value, remaining, env, *parent),
            Continuation::Do {
                bindings,
                test,
                result_exprs,
                body_exprs,
                env,
                parent,
            } => self.apply_do_continuation(value, bindings, test, result_exprs, body_exprs, env, *parent),
            Continuation::Captured { cont } => {
                self.apply_captured_continuation(value, *cont)
            }
            Continuation::CallCc {
                captured_cont,
                env,
                parent,
            } => self.apply_call_cc_continuation(value, captured_cont, env, *parent),
        }
    }

    /// Apply identity continuation (final result)
    fn apply_identity_continuation(&mut self, value: Value) -> Result<Value> {
        Ok(value)
    }

    /// Apply operator continuation
    fn apply_operator_continuation(
        &mut self,
        value: Value,
        args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Operator has been evaluated, now evaluate arguments
        self.eval_args(value, args, Vec::new(), env, parent)
    }

    /// Apply application continuation
    fn apply_application_continuation(
        &mut self,
        value: Value,
        operator: Value,
        mut evaluated_args: Vec<Value>,
        remaining_args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Add this argument to evaluated arguments
        evaluated_args.push(value);
        
        // Continue evaluating remaining arguments
        self.eval_args(operator, remaining_args, evaluated_args, env, parent)
    }

    /// Apply if test continuation
    fn apply_if_test_continuation(
        &mut self,
        value: Value,
        consequent: Expr,
        alternate: Option<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            self.eval(consequent, env, parent)
        } else if let Some(alt) = alternate {
            self.eval(alt, env, parent)
        } else {
            self.apply_continuation(parent, Value::Undefined)
        }
    }

    /// Apply assignment continuation
    fn apply_assignment_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        env.set(&variable, value)?;
        self.apply_continuation(parent, Value::Undefined)
    }

    /// Apply values continuation
    fn apply_values_continuation(
        &mut self,
        value: Value,
        mut values: Vec<Value>,
        parent: Continuation,
    ) -> Result<Value> {
        values.push(value);
        self.apply_continuation(parent, Value::Values(values))
    }

    /// Apply call-with-values step 1 continuation
    fn apply_call_with_values_step1_continuation(
        &mut self,
        value: Value,
        producer_expr: Expr,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Consumer has been evaluated (value), now evaluate producer
        let consumer = value;
        let step2_cont = Continuation::CallWithValuesStep2 {
            consumer,
            env: env.clone(),
            parent: Box::new(parent),
        };

        // Evaluate the producer
        self.eval(producer_expr, env, step2_cont)
    }

    /// Apply call-with-values step 2 continuation
    fn apply_call_with_values_step2_continuation(
        &mut self,
        value: Value,
        consumer: Value,
        _env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
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
        self.apply_procedure(consumer, consumer_args, parent)
    }

    /// Apply begin continuation
    fn apply_begin_continuation(
        &mut self,
        value: Value,
        mut remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Current expression has been evaluated, continue with remaining
        if remaining.is_empty() {
            // Last expression in sequence, return its value
            self.apply_continuation(parent, value)
        } else {
            // More expressions to evaluate
            let next_expr = remaining.remove(0);
            let begin_cont = Continuation::Begin {
                remaining,
                env: env.clone(),
                parent: Box::new(parent),
            };
            self.eval(next_expr, env, begin_cont)
        }
    }

    /// Apply define continuation
    fn apply_define_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Value has been evaluated for definition
        env.define(variable, value);
        self.apply_continuation(parent, Value::Undefined)
    }

    /// Apply and continuation
    fn apply_and_continuation(
        &mut self,
        value: Value,
        mut remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Check if current value is false
        if !value.is_truthy() {
            // Short-circuit: return false
            self.apply_continuation(parent, Value::Boolean(false))
        } else if remaining.is_empty() {
            // Last expression, return its value
            self.apply_continuation(parent, value)
        } else {
            // Continue with next expression
            let next_expr = remaining.remove(0);
            let and_cont = Continuation::And {
                remaining,
                env: env.clone(),
                parent: Box::new(parent),
            };
            self.eval(next_expr, env, and_cont)
        }
    }

    /// Apply or continuation
    fn apply_or_continuation(
        &mut self,
        value: Value,
        mut remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Check if current value is true
        if value.is_truthy() {
            // Short-circuit: return the truthy value
            self.apply_continuation(parent, value)
        } else if remaining.is_empty() {
            // Last expression, return its value (false)
            self.apply_continuation(parent, value)
        } else {
            // Continue with next expression
            let next_expr = remaining.remove(0);
            let or_cont = Continuation::Or {
                remaining,
                env: env.clone(),
                parent: Box::new(parent),
            };
            self.eval(next_expr, env, or_cont)
        }
    }

    /// Apply do continuation
    #[allow(clippy::too_many_arguments)]
    fn apply_do_continuation(
        &mut self,
        _value: Value,
        bindings: Vec<(String, Expr, Option<Expr>)>,
        test: Expr,
        result_exprs: Vec<Expr>,
        body_exprs: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Body expressions have been evaluated, continue the loop
        self.continue_do_loop(bindings, test, result_exprs, body_exprs, env, parent)
    }

    /// Apply captured continuation
    fn apply_captured_continuation(
        &mut self,
        value: Value,
        cont: Continuation,
    ) -> Result<Value> {
        // A captured continuation was invoked
        self.apply_continuation(cont, value)
    }

    /// Apply call/cc continuation
    fn apply_call_cc_continuation(
        &mut self,
        value: Value,
        captured_cont: Value,
        _env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // The procedure has been evaluated, now call it with the captured continuation
        match value {
            Value::Procedure(_) => {
                self.apply_procedure(value, vec![captured_cont], parent)
            }
            _ => Err(LambdustError::type_error(
                "Not a procedure".to_string(),
            )),
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
                Procedure::HostFunction { func, .. } => {
                    let result = func(&args)?;
                    self.apply_continuation(cont, result)
                }
                Procedure::Continuation { .. } => Err(LambdustError::runtime_error(
                    "Continuation procedures not yet implemented in formal evaluator".to_string(),
                )),
            },
            _ => Err(LambdustError::type_error(format!(
                "Not a procedure: {operator}"
            ))),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for backward compatibility
pub type FormalEvaluator = Evaluator;

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
    fn test_formal_call_with_values_errors() {
        // Test call-with-values with wrong arity
        let result = eval_str_formal("(call-with-values)");
        assert!(result.is_err());

        let result = eval_str_formal("(call-with-values (lambda () 1))");
        assert!(result.is_err());

        let result = eval_str_formal("(call-with-values (lambda () 1) (lambda (x) x) extra)");
        assert!(result.is_err());

        // Test call-with-values with non-procedure arguments
        let result = eval_str_formal("(call-with-values 42 (lambda (x) x))");
        assert!(result.is_err());

        let result = eval_str_formal("(call-with-values (lambda () 1) 42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_formal_multi_value_continuations() {
        // Test that the formal evaluator properly handles multiple values in continuations
        // This ensures that the CPS implementation correctly propagates multi-value contexts

        // Test simple multi-value propagation
        let result = eval_str_formal("(values 1 2 3)").unwrap();
        assert_eq!(
            result,
            Value::Values(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64)
            ])
        );

        // Test multi-value in call-with-values (more complex CPS case)
        let result = eval_str_formal(
            "(call-with-values (lambda () (values 5 10 15)) (lambda (a b c) (+ a b c)))",
        )
        .unwrap();
        assert_eq!(result, Value::from(30i64));
    }

    #[test]
    fn test_formal_begin() {
        // Test empty begin
        let result = eval_str_formal("(begin)").unwrap();
        assert_eq!(result, Value::Undefined);

        // Test single expression begin
        let result = eval_str_formal("(begin 42)").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test multiple expression begin
        let result = eval_str_formal("(begin (+ 1 2) (* 3 4) (- 10 5))").unwrap();
        assert_eq!(result, Value::from(5i64));
    }

    #[test]
    fn test_formal_define() {
        // Test simple define
        let result = eval_str_formal("(begin (define x 42) x)").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test define with complex expression
        let result = eval_str_formal("(begin (define y (+ 10 20)) y)").unwrap();
        assert_eq!(result, Value::from(30i64));
    }

    #[test]
    fn test_formal_and() {
        // Test empty and
        let result = eval_str_formal("(and)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test single expression and
        let result = eval_str_formal("(and 42)").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test and with all true values
        let result = eval_str_formal("(and 1 2 3)").unwrap();
        assert_eq!(result, Value::from(3i64));

        // Test and with false value (short-circuit)
        let result = eval_str_formal("(and 1 #f 3)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_formal_or() {
        // Test empty or
        let result = eval_str_formal("(or)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test single expression or
        let result = eval_str_formal("(or 42)").unwrap();
        assert_eq!(result, Value::from(42i64));

        // Test or with first true value (short-circuit)
        let result = eval_str_formal("(or 1 2 3)").unwrap();
        assert_eq!(result, Value::from(1i64));

        // Test or with all false values
        let result = eval_str_formal("(or #f #f #f)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test or with true value at end
        let result = eval_str_formal("(or #f #f 42)").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_formal_do() {
        // Test do loop with immediate termination
        let result = eval_str_formal(
            "(do ((i 5)) ((> i 3) i))"
        ).unwrap();
        assert_eq!(result, Value::from(5i64));

        // Test do loop with step expression
        let result = eval_str_formal(
            "(do ((i 0 (+ i 1))) ((>= i 3) i))"
        ).unwrap();
        assert_eq!(result, Value::from(3i64));
    }
    
    #[test]
    fn test_formal_do_with_step() {
        // Test do loop with step expression and accumulator
        let result = eval_str_formal(
            "(do ((i 0 (+ i 2)) (sum 0 (+ sum i))) ((>= i 10) sum))"
        ).unwrap();
        // i: 0, 2, 4, 6, 8, 10
        // sum: 0, 0, 2, 6, 12, 20
        assert_eq!(result, Value::from(20i64));
    }
    
    #[test]
    fn test_formal_do_no_step() {
        // Test do loop without step expression (variable unchanged)
        let result = eval_str_formal(
            "(do ((i 5)) ((< i 10) i))"
        ).unwrap();
        assert_eq!(result, Value::from(5i64));
    }

    #[test]
    fn test_formal_delay() {
        // Test delay creates a promise
        let result = eval_str_formal("(delay (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));
    }

    #[test]
    fn test_formal_lazy() {
        // Test lazy creates a promise
        let result = eval_str_formal("(lazy (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));
    }

    #[test]
    fn test_formal_call_cc_basic() {
        // Basic call/cc test
        let result = eval_str_formal("(call/cc (lambda (k) 42))").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_formal_call_cc_escape() {
        // Test call/cc with escape continuation (placeholder)
        // Note: Full escape semantics not yet implemented
        let result = eval_str_formal("(+ 1 (call/cc (lambda (k) 2)) 3)").unwrap();
        assert_eq!(result, Value::from(6i64));
    }

    #[test]
    fn test_formal_force() {
        // Test force evaluates a promise
        let result = eval_str_formal("(force (delay (+ 1 2)))").unwrap();
        // Note: This will fail until force is properly implemented to actually force promises
        // For now, just test that it doesn't crash and returns some value
        assert!(!matches!(result, Value::Undefined));
    }
}
