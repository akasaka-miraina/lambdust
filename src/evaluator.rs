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
    /// Cond clause test continuation
    CondTest {
        /// Current clause consequent (expressions to evaluate if test is true)
        consequent: Vec<Expr>,
        /// Remaining clauses to check if test is false
        remaining_clauses: Vec<(Expr, Vec<Expr>)>,
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
    /// Exception handler continuation
    ExceptionHandler {
        /// Handler procedure to call when exception is raised
        handler: Value,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Guard clause continuation
    GuardClause {
        /// Variable name for the exception object
        condition_var: String,
        /// Clause expressions to test (condition-expr . result-exprs)
        clauses: Vec<(Expr, Vec<Expr>)>,
        /// Else clause expressions (if any)
        else_exprs: Option<Vec<Expr>>,
        /// Environment for evaluation
        env: Rc<Environment>,
        /// Parent continuation
        parent: Box<Continuation>,
    },
    /// Vector evaluation continuation
    VectorEval {
        /// Elements evaluated so far
        evaluated_elements: Vec<Value>,
        /// Remaining elements to evaluate
        remaining_elements: Vec<Expr>,
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

/// Exception handler information for exception handling
#[derive(Debug, Clone)]
pub struct ExceptionHandlerInfo {
    /// Handler procedure
    pub handler: Value,
    /// Handler environment
    pub env: Rc<Environment>,
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
    /// Recursion depth counter for stack overflow prevention
    recursion_depth: usize,
    /// Maximum recursion depth
    max_recursion_depth: usize,
    /// Exception handlers stack for exception handling
    exception_handlers: Vec<ExceptionHandlerInfo>,
}

impl Evaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000, // Configurable recursion limit
            exception_handlers: Vec::new(),
        }
    }

    /// Create a new formal evaluator with specific evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
        }
    }

    /// Main evaluation function: E[e]ρκσ
    /// Where:
    /// - e: expression to evaluate
    /// - ρ: environment
    /// - κ: continuation
    /// - σ: store
    pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // Stack overflow prevention
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(LambdustError::stack_overflow());
        }

        self.recursion_depth += 1;
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

            // Other forms
            _ => Err(LambdustError::syntax_error(format!(
                "Unsupported expression: {expr:?}"
            ))),
        };

        self.recursion_depth -= 1;
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
                | "do"
                | "call/cc"
                | "call-with-current-continuation"
                | "values"
                | "call-with-values"
                | "dynamic-wind"
                | "delay"
                | "lazy"
                | "force"
                | "raise"
                | "with-exception-handler"
                | "guard"
                | "map"
                | "apply"
                | "fold"
                | "fold-right"
                | "filter"
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

            // Conditional forms
            "cond" => self.eval_cond(operands, env, cont),

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

            // Exception handling
            "raise" => self.eval_raise(operands, env, cont),
            "with-exception-handler" => self.eval_with_exception_handler(operands, env, cont),
            "guard" => self.eval_guard(operands, env, cont),

            // Higher-order functions (evaluator-integrated)
            "map" => self.eval_map_special_form(operands, env, cont),
            "apply" => self.eval_apply_special_form(operands, env, cont),
            "fold" => self.eval_fold_special_form(operands, env, cont),
            "fold-right" => self.eval_fold_right_special_form(operands, env, cont),
            "filter" => self.eval_filter_special_form(operands, env, cont),

            // This should never be reached if is_special_form is correct
            _ => Err(LambdustError::syntax_error(format!(
                "Internal error: Unknown special form: {}",
                name
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

    /// Evaluate cond expression
    fn eval_cond(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return Err(LambdustError::syntax_error("cond: no clauses".to_string()));
        }

        // Parse clauses: (test expr...) or (else expr...)
        let mut clauses = Vec::new();
        let mut else_clause = None;

        for operand in operands {
            match operand {
                Expr::List(clause_parts) if !clause_parts.is_empty() => {
                    let test = clause_parts[0].clone();
                    let exprs = clause_parts[1..].to_vec();

                    // Check for else clause
                    if let Expr::Variable(name) = &test {
                        if name == "else" {
                            if else_clause.is_some() {
                                return Err(LambdustError::syntax_error(
                                    "cond: multiple else clauses".to_string(),
                                ));
                            }
                            else_clause = Some(exprs);
                            continue;
                        }
                    }

                    clauses.push((test, exprs));
                }
                _ => {
                    return Err(LambdustError::syntax_error(
                        "cond: clause must be a list".to_string(),
                    ));
                }
            }
        }

        // If we have an else clause, add it as the final clause with a #t test
        if let Some(else_exprs) = else_clause {
            clauses.push((Expr::Literal(Literal::Boolean(true)), else_exprs));
        }

        if clauses.is_empty() {
            return Err(LambdustError::syntax_error(
                "cond: no valid clauses".to_string(),
            ));
        }

        // Start evaluating the first clause
        let mut clauses_iter = clauses.into_iter();
        let (first_test, first_consequent) = clauses_iter.next().unwrap();
        let remaining_clauses: Vec<_> = clauses_iter.collect();

        let cond_cont = Continuation::CondTest {
            consequent: first_consequent,
            remaining_clauses,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_test, env, cond_cont)
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

    /// Evaluate quasiquote expression
    fn eval_quasiquote(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let expanded = self.expand_quasiquote(expr, 1, env.clone())?;
        self.eval(expanded, env, cont)
    }

    /// Evaluate vector literal
    fn eval_vector(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            // Empty vector
            let empty_vector = Value::Vector(vec![]);
            return self.apply_continuation(cont, empty_vector);
        }

        // Evaluate first expression
        let first_expr = exprs[0].clone();
        let remaining_exprs = exprs[1..].to_vec();

        let vector_cont = Continuation::VectorEval {
            evaluated_elements: vec![],
            remaining_elements: remaining_exprs,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr, env, vector_cont)
    }

    /// Expand quasiquote expression
    ///
    /// This implements the R7RS quasiquote expansion algorithm:
    /// - Level 0: We are inside unquote, evaluate expression
    /// - Level 1+: We are in quasiquote, handle nested structure
    #[allow(clippy::only_used_in_recursion)]
    fn expand_quasiquote(&self, expr: Expr, level: i32, _env: Rc<Environment>) -> Result<Expr> {
        match expr {
            // Unquote: ,expr
            Expr::Unquote(inner) => {
                if level == 1 {
                    // At level 1, unquote should be evaluated
                    Ok(*inner)
                } else {
                    // Nested quasiquote, decrease level
                    Ok(Expr::Unquote(Box::new(self.expand_quasiquote(
                        *inner,
                        level - 1,
                        _env,
                    )?)))
                }
            }

            // Nested quasiquote: `expr
            Expr::Quasiquote(inner) => {
                // Increase level for nested quasiquote
                Ok(Expr::Quasiquote(Box::new(self.expand_quasiquote(
                    *inner,
                    level + 1,
                    _env,
                )?)))
            }

            // List: handle each element
            Expr::List(exprs) => {
                // Check if any element is an unquote or unquote-splicing
                let has_unquotes = exprs
                    .iter()
                    .any(|e| matches!(e, Expr::Unquote(_) | Expr::UnquoteSplicing(_)));

                if !has_unquotes && level == 1 {
                    // No unquotes at level 1, treat as quote
                    Ok(Expr::Quote(Box::new(Expr::List(exprs))))
                } else {
                    // Has unquotes or nested, need expansion
                    let mut result = Vec::new();
                    for expr in exprs {
                        // Check for unquote-splicing first
                        match &expr {
                            Expr::UnquoteSplicing(inner) if level == 1 => {
                                // This should splice the result, but for now we'll treat it as regular unquote
                                // Full splicing implementation would require more complex list handling
                                result.push(*inner.clone());
                            }
                            _ => {
                                result.push(self.expand_quasiquote(expr, level, _env.clone())?);
                            }
                        }
                    }
                    Ok(Expr::List(result))
                }
            }

            // Vector: handle each element
            Expr::Vector(exprs) => {
                // Check if any element is an unquote or unquote-splicing
                let has_unquotes = exprs
                    .iter()
                    .any(|e| matches!(e, Expr::Unquote(_) | Expr::UnquoteSplicing(_)));

                if !has_unquotes && level == 1 {
                    // No unquotes at level 1, treat as quote
                    Ok(Expr::Quote(Box::new(Expr::Vector(exprs))))
                } else {
                    // Has unquotes or nested, need expansion
                    let mut result = Vec::new();
                    for expr in exprs {
                        result.push(self.expand_quasiquote(expr, level, _env.clone())?);
                    }
                    Ok(Expr::Vector(result))
                }
            }

            // DottedList: handle each element
            Expr::DottedList(exprs, tail) => {
                let mut result = Vec::new();
                for expr in exprs {
                    result.push(self.expand_quasiquote(expr, level, _env.clone())?);
                }
                let expanded_tail = self.expand_quasiquote(*tail, level, _env)?;
                Ok(Expr::DottedList(result, Box::new(expanded_tail)))
            }

            // All other expressions at level 1 should be quoted
            _ if level == 1 => Ok(Expr::Quote(Box::new(expr))),

            // All other expressions at other levels are left as-is
            _ => Ok(expr),
        }
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
            Expr::Vector(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.expr_to_value(expr)?);
                }
                Ok(Value::Vector(values))
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
                let lambda_expr = Expr::List(
                    vec![Expr::Variable("lambda".to_string()), Expr::List(params)]
                        .into_iter()
                        .chain(body)
                        .collect(),
                );

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
                self.eval_begin(
                    &body_exprs,
                    do_env.clone(),
                    Continuation::Do {
                        bindings,
                        test,
                        result_exprs,
                        body_exprs: body_exprs.clone(),
                        env: do_env,
                        parent: Box::new(cont),
                    },
                )
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
                self.eval_begin(
                    &body_exprs,
                    env.clone(),
                    Continuation::Do {
                        bindings,
                        test,
                        result_exprs,
                        body_exprs: body_exprs.clone(),
                        env,
                        parent: Box::new(cont),
                    },
                )
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

    /// Convert evaluator continuation to value continuation for call/cc
    fn convert_continuation_to_value(
        &self,
        cont: &Continuation,
        env: Rc<Environment>,
    ) -> crate::value::Continuation {
        // Convert continuation chain to stack frames for value representation
        let mut stack = vec![];
        self.collect_continuation_stack(cont, &mut stack);

        crate::value::Continuation { stack, env }
    }

    /// Recursively collect continuation stack frames
    #[allow(clippy::only_used_in_recursion)]
    fn collect_continuation_stack(
        &self,
        cont: &Continuation,
        stack: &mut Vec<crate::value::StackFrame>,
    ) {
        match cont {
            Continuation::Identity => {
                // Base case: identity continuation
            }
            Continuation::Application {
                remaining_args,
                env,
                parent,
                ..
            } => {
                // Add current application frame
                if let Some(expr) = remaining_args.first() {
                    stack.push(crate::value::StackFrame {
                        expr: expr.clone(),
                        env: env.clone(),
                    });
                }
                self.collect_continuation_stack(parent, stack);
            }
            Continuation::Operator { args, env, parent } => {
                // Add operator evaluation frame
                if let Some(expr) = args.first() {
                    stack.push(crate::value::StackFrame {
                        expr: expr.clone(),
                        env: env.clone(),
                    });
                }
                self.collect_continuation_stack(parent, stack);
            }
            _ => {
                // For other continuation types, just recurse to parent if available
                // This is a simplified version - full implementation would handle all cases
            }
        }
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

        // Capture the current continuation by converting evaluator continuation to value continuation
        let value_cont = self.convert_continuation_to_value(&cont, env.clone());
        let captured_cont = Value::Procedure(Procedure::Continuation {
            continuation: Box::new(value_cont),
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
            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => self.apply_if_test_continuation(value, consequent, alternate, env, *parent),
            Continuation::CondTest {
                consequent,
                remaining_clauses,
                env,
                parent,
            } => self.apply_cond_test_continuation(
                value,
                consequent,
                remaining_clauses,
                env,
                *parent,
            ),
            Continuation::Assignment {
                variable,
                env,
                parent,
            } => self.apply_assignment_continuation(value, variable, env, *parent),
            Continuation::Values { mut values, parent } => {
                // Inline for performance
                values.push(value);
                self.apply_continuation(*parent, Value::Values(values))
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
            } => self.apply_do_continuation(
                value,
                bindings,
                test,
                result_exprs,
                body_exprs,
                env,
                *parent,
            ),
            Continuation::Captured { cont } => self.apply_captured_continuation(value, *cont),
            Continuation::CallCc {
                captured_cont,
                env,
                parent,
            } => self.apply_call_cc_continuation(value, captured_cont, env, *parent),
            Continuation::ExceptionHandler {
                handler,
                env,
                parent,
            } => self.apply_exception_handler_continuation(value, handler, env, *parent),
            Continuation::GuardClause {
                condition_var,
                clauses,
                else_exprs,
                env,
                parent,
            } => self.apply_guard_clause_continuation(
                value,
                condition_var,
                clauses,
                else_exprs,
                env,
                *parent,
            ),
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
                    let next_expr = remaining_elements[0].clone();
                    let remaining = remaining_elements[1..].to_vec();

                    let vector_cont = Continuation::VectorEval {
                        evaluated_elements,
                        remaining_elements: remaining,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr, env, vector_cont)
                }
            }
        }
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

    /// Apply cond test continuation
    fn apply_cond_test_continuation(
        &mut self,
        value: Value,
        consequent: Vec<Expr>,
        remaining_clauses: Vec<(Expr, Vec<Expr>)>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            // Test passed, evaluate consequent expressions
            if consequent.is_empty() {
                // If no consequent expressions, return the test result
                self.apply_continuation(parent, value)
            } else if consequent.len() == 1 {
                // Single expression, evaluate it
                self.eval(consequent[0].clone(), env, parent)
            } else {
                // Multiple expressions, evaluate as begin
                self.eval_begin(&consequent, env, parent)
            }
        } else {
            // Test failed, try next clause
            if remaining_clauses.is_empty() {
                // No more clauses, return undefined (R7RS specifies this)
                self.apply_continuation(parent, Value::Undefined)
            } else {
                // Try next clause
                let mut remaining_iter = remaining_clauses.into_iter();
                let (next_test, next_consequent) = remaining_iter.next().unwrap();
                let remaining: Vec<_> = remaining_iter.collect();

                let cond_cont = Continuation::CondTest {
                    consequent: next_consequent,
                    remaining_clauses: remaining,
                    env: env.clone(),
                    parent: Box::new(parent),
                };

                self.eval(next_test, env, cond_cont)
            }
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
        let producer_result = self.apply_procedure(producer, Vec::new(), Continuation::Identity)?;

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
    fn apply_captured_continuation(&mut self, value: Value, cont: Continuation) -> Result<Value> {
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
            Value::Procedure(_) => self.apply_procedure(value, vec![captured_cont], parent),
            _ => Err(LambdustError::type_error("Not a procedure".to_string())),
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
    pub fn apply_evaluation_order(&self, mut args: Vec<Expr>) -> Vec<Expr> {
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
        // Tail call optimization: check if this is the final continuation
        let is_tail_call = matches!(cont, Continuation::Identity);
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

                    // Tail call optimization: avoid continuation for identity
                    if is_tail_call {
                        Ok(result)
                    } else {
                        self.apply_continuation(cont, result)
                    }
                }
                Procedure::Lambda {
                    params,
                    variadic,
                    body,
                    closure,
                } => {
                    // Create new environment with parameter bindings
                    let new_env = closure.bind_parameters(&params, &args, variadic)?;

                    // Evaluate body with tail call optimization
                    if body.is_empty() {
                        if is_tail_call {
                            Ok(Value::Undefined)
                        } else {
                            self.apply_continuation(cont, Value::Undefined)
                        }
                    } else if body.len() == 1 {
                        self.eval(body[0].clone(), Rc::new(new_env), cont)
                    } else {
                        // Evaluate sequence
                        self.eval_sequence(body, Rc::new(new_env), cont)
                    }
                }
                Procedure::HostFunction { func, .. } => {
                    let result = func(&args)?;

                    // Tail call optimization: avoid continuation for identity
                    if is_tail_call {
                        Ok(result)
                    } else {
                        self.apply_continuation(cont, result)
                    }
                }
                Procedure::Continuation {
                    continuation: _continuation,
                } => {
                    // Call/cc continuation: return the value directly (simulating escape)
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    // For now, return the value directly
                    // In a full implementation, this would restore the captured continuation
                    // TODO: Implement proper continuation restoration and non-local exit
                    Ok(args[0].clone())
                }
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

impl Evaluator {
    /// Push an exception handler onto the stack
    fn push_exception_handler(&mut self, handler: Value, env: Rc<Environment>) {
        self.exception_handlers
            .push(ExceptionHandlerInfo { handler, env });
    }

    /// Pop an exception handler from the stack
    fn pop_exception_handler(&mut self) -> Option<ExceptionHandlerInfo> {
        self.exception_handlers.pop()
    }

    /// Evaluate raise expression  
    fn eval_raise(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        // Evaluate the exception object
        let exception_expr = operands[0].clone();
        let exception_value = self.eval(exception_expr, env, Continuation::Identity)?;

        // Search for exception handlers by looking for guard clauses in the continuation chain
        self.find_and_invoke_exception_handler(exception_value, cont)
    }

    /// Find and invoke the appropriate exception handler
    fn find_and_invoke_exception_handler(
        &mut self,
        exception_value: Value,
        cont: Continuation,
    ) -> Result<Value> {
        // Search the continuation chain for guard clauses
        match cont {
            Continuation::GuardClause {
                condition_var,
                clauses,
                else_exprs,
                env,
                parent,
            } => {
                // Found a guard clause - process the exception through it
                self.process_guard_exception(
                    exception_value,
                    condition_var,
                    clauses,
                    else_exprs,
                    env,
                    *parent,
                )
            }
            _ => {
                // No guard clause found, check exception handler stack
                if let Some(handler_info) = self.exception_handlers.last() {
                    let handler = handler_info.handler.clone();
                    self.exception_handlers.pop();
                    self.apply_procedure(handler, vec![exception_value], cont)
                } else {
                    // No exception handler found - propagate as runtime error
                    Err(LambdustError::runtime_error(format!(
                        "Uncaught exception: {}",
                        exception_value
                    )))
                }
            }
        }
    }

    /// Process exception through guard clauses
    fn process_guard_exception(
        &mut self,
        exception_value: Value,
        condition_var: String,
        clauses: Vec<(Expr, Vec<Expr>)>,
        else_exprs: Option<Vec<Expr>>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Pop the guard handler from the exception handler stack
        self.pop_exception_handler();

        // Create a new environment with the exception bound to the condition variable
        let guard_env = Environment::with_parent(env);
        guard_env.define(condition_var.clone(), exception_value.clone());
        let guard_env_rc = Rc::new(guard_env);

        // Try each guard clause in order
        for (condition_expr, result_exprs) in clauses {
            let condition_result =
                self.eval(condition_expr, guard_env_rc.clone(), Continuation::Identity)?;

            // If condition is true (non-#f), execute this clause
            if !matches!(condition_result, Value::Boolean(false)) {
                // Evaluate result expressions in the guard environment
                if result_exprs.len() == 1 {
                    return self.eval(result_exprs[0].clone(), guard_env_rc, parent);
                } else {
                    return self.eval_begin(&result_exprs, guard_env_rc, parent);
                }
            }
        }

        // No clause matched, try else clause
        if let Some(else_result_exprs) = else_exprs {
            if else_result_exprs.len() == 1 {
                self.eval(else_result_exprs[0].clone(), guard_env_rc, parent)
            } else {
                self.eval_begin(&else_result_exprs, guard_env_rc, parent)
            }
        } else {
            // No else clause, re-raise the exception
            self.find_and_invoke_exception_handler(exception_value, parent)
        }
    }

    /// Evaluate with-exception-handler expression
    fn eval_with_exception_handler(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let handler_expr = operands[0].clone();
        let thunk_expr = operands[1].clone();

        // First evaluate the handler expression
        let handler_value = self.eval(handler_expr, env.clone(), Continuation::Identity)?;

        // Verify that the handler is a procedure
        if !matches!(handler_value, Value::Procedure(_)) {
            return Err(LambdustError::type_error(
                "Exception handler must be a procedure".to_string(),
            ));
        }

        // Push the handler onto the exception handler stack
        self.push_exception_handler(handler_value.clone(), env.clone());

        // Create a continuation that will pop the handler when done
        let exception_handler_cont = Continuation::ExceptionHandler {
            handler: handler_value,
            env: env.clone(),
            parent: Box::new(cont),
        };

        // Evaluate the thunk (call it with no arguments)
        let thunk_call = Expr::List(vec![thunk_expr]);
        self.eval(thunk_call, env, exception_handler_cont)
    }

    /// Evaluate guard expression
    fn eval_guard(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "guard: too few arguments".to_string(),
            ));
        }

        // Parse guard syntax: (guard (var clause1 clause2 ... [else clause]) body...)
        let guard_spec = &operands[0];
        let body_exprs = &operands[1..];

        let (condition_var, clauses, else_exprs) = self.parse_guard_spec(guard_spec)?;

        // Create a guard-specific exception handler
        let guard_handler = Value::Procedure(Procedure::Builtin {
            name: "guard-handler".to_string(),
            arity: Some(1),
            func: |_args| {
                // This is a placeholder - the actual handling is done in apply_guard_clause_continuation
                Ok(Value::Undefined)
            },
        });

        // Push the guard handler onto the exception handler stack
        self.push_exception_handler(guard_handler, env.clone());

        // Create a guard clause continuation
        let guard_cont = Continuation::GuardClause {
            condition_var,
            clauses,
            else_exprs,
            env: env.clone(),
            parent: Box::new(cont),
        };

        // Evaluate the body with guard protection
        if body_exprs.len() == 1 {
            self.eval(body_exprs[0].clone(), env, guard_cont)
        } else {
            self.eval_begin(body_exprs, env, guard_cont)
        }
    }

    /// Parse guard specification: (var clause1 clause2 ... [else clause])
    #[allow(clippy::type_complexity)]
    fn parse_guard_spec(
        &self,
        spec: &Expr,
    ) -> Result<(String, Vec<(Expr, Vec<Expr>)>, Option<Vec<Expr>>)> {
        if let Expr::List(spec_items) = spec {
            if spec_items.is_empty() {
                return Err(LambdustError::syntax_error(
                    "guard: empty guard specification".to_string(),
                ));
            }

            // First item should be the condition variable
            let condition_var = if let Expr::Variable(var) = &spec_items[0] {
                var.clone()
            } else {
                return Err(LambdustError::syntax_error(
                    "guard: condition variable must be an identifier".to_string(),
                ));
            };

            let mut clauses = Vec::new();
            let mut else_exprs = None;

            // Parse clauses
            for clause_expr in &spec_items[1..] {
                if let Expr::List(clause_items) = clause_expr {
                    if clause_items.is_empty() {
                        return Err(LambdustError::syntax_error(
                            "guard: empty clause".to_string(),
                        ));
                    }

                    // Check for else clause
                    if let Expr::Variable(keyword) = &clause_items[0] {
                        if keyword == "else" {
                            else_exprs = Some(clause_items[1..].to_vec());
                            continue;
                        }
                    }

                    // Regular clause: (condition result-expr1 result-expr2 ...)
                    let condition = clause_items[0].clone();
                    let results = clause_items[1..].to_vec();
                    clauses.push((condition, results));
                } else {
                    return Err(LambdustError::syntax_error(
                        "guard: clause must be a list".to_string(),
                    ));
                }
            }

            Ok((condition_var, clauses, else_exprs))
        } else {
            Err(LambdustError::syntax_error(
                "guard: guard specification must be a list".to_string(),
            ))
        }
    }

    /// Apply exception handler continuation
    fn apply_exception_handler_continuation(
        &mut self,
        value: Value,
        _handler: Value,
        _env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Pop the exception handler from the stack since evaluation is complete
        self.pop_exception_handler();

        // Continue with the value
        self.apply_continuation(parent, value)
    }

    /// Apply guard clause continuation
    fn apply_guard_clause_continuation(
        &mut self,
        value: Value,
        _condition_var: String,
        _clauses: Vec<(Expr, Vec<Expr>)>,
        _else_exprs: Option<Vec<Expr>>,
        _env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Pop the guard handler from the exception handler stack
        self.pop_exception_handler();

        // Normal completion: just pass the value through
        self.apply_continuation(parent, value)
    }

    /// Apply a procedure to arguments with evaluator integration for lambda functions
    pub fn apply_procedure_with_evaluator(
        &mut self,
        proc: Value,
        args: Vec<Value>,
    ) -> Result<Value> {
        match proc {
            Value::Procedure(Procedure::Builtin { func, arity, .. }) => {
                // Check arity for builtin functions
                if let Some(expected) = arity {
                    if args.len() != expected {
                        return Err(LambdustError::arity_error(expected, args.len()));
                    }
                }
                func(&args)
            }
            Value::Procedure(Procedure::Lambda {
                params,
                variadic,
                body,
                closure,
            }) => {
                // Create new environment with parameter bindings
                let new_env = closure.bind_parameters(&params, &args, variadic)?;

                // Evaluate body in the new environment
                if body.is_empty() {
                    Ok(Value::Undefined)
                } else if body.len() == 1 {
                    self.eval(body[0].clone(), Rc::new(new_env), Continuation::Identity)
                } else {
                    self.eval_sequence(body, Rc::new(new_env), Continuation::Identity)
                }
            }
            Value::Procedure(Procedure::HostFunction { func, .. }) => func(&args),
            Value::Procedure(Procedure::Continuation {
                continuation: _continuation,
            }) => {
                // Handle continuation procedures
                if args.len() != 1 {
                    return Err(LambdustError::arity_error(1, args.len()));
                }
                Ok(args[0].clone())
            }
            _ => Err(LambdustError::type_error(format!(
                "Not a procedure: {proc}"
            ))),
        }
    }

    /// Evaluator-aware map implementation
    pub fn eval_map(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }

        let proc = args[0].clone();
        let lists = &args[1..];

        // Convert all arguments to vectors for easier iteration
        let mut vectors = Vec::new();
        for list in lists {
            vectors.push(list.to_vector().ok_or_else(|| {
                LambdustError::type_error("map: argument is not a proper list".to_string())
            })?);
        }

        // Check that all lists have the same length
        if vectors.is_empty() {
            return Ok(Value::from_vector(vec![]));
        }

        let length = vectors[0].len();
        for (i, vec) in vectors.iter().enumerate() {
            if vec.len() != length {
                return Err(LambdustError::runtime_error(format!(
                    "map: all lists must have the same length, list {} has length {} but expected {}",
                    i + 1,
                    vec.len(),
                    length
                )));
            }
        }

        // Apply the procedure to each set of arguments
        let mut results = Vec::new();
        for i in 0..length {
            let call_args: Vec<Value> = vectors.iter().map(|vec| vec[i].clone()).collect();
            let result = self.apply_procedure_with_evaluator(proc.clone(), call_args)?;
            results.push(result);
        }

        Ok(Value::from_vector(results))
    }

    /// Evaluator-aware apply implementation
    pub fn eval_apply(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }

        let proc = args[0].clone();

        // Build the argument list from all provided arguments
        let mut apply_args = Vec::new();

        // Add intermediate arguments (if any)
        for arg in &args[1..args.len() - 1] {
            apply_args.push(arg.clone());
        }

        // The last argument should be a list
        let last_arg = &args[args.len() - 1];
        let last_list = last_arg.to_vector().ok_or_else(|| {
            LambdustError::type_error("apply: last argument must be a proper list".to_string())
        })?;
        apply_args.extend(last_list);

        self.apply_procedure_with_evaluator(proc, apply_args)
    }

    /// Evaluator-aware fold implementation (left fold)
    pub fn eval_fold(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 3 {
            return Err(LambdustError::arity_error(3, args.len()));
        }

        let proc = args[0].clone();
        let mut accumulator = args[1].clone();
        let list = args[2].to_vector().ok_or_else(|| {
            LambdustError::type_error("fold: third argument must be a proper list".to_string())
        })?;

        for item in list {
            accumulator =
                self.apply_procedure_with_evaluator(proc.clone(), vec![accumulator, item])?;
        }

        Ok(accumulator)
    }

    /// Evaluator-aware fold-right implementation (right fold)
    pub fn eval_fold_right(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 3 {
            return Err(LambdustError::arity_error(3, args.len()));
        }

        let proc = args[0].clone();
        let mut accumulator = args[1].clone();
        let list = args[2].to_vector().ok_or_else(|| {
            LambdustError::type_error(
                "fold-right: third argument must be a proper list".to_string(),
            )
        })?;

        // Process from right to left
        for item in list.into_iter().rev() {
            accumulator =
                self.apply_procedure_with_evaluator(proc.clone(), vec![item, accumulator])?;
        }

        Ok(accumulator)
    }

    /// Evaluator-aware filter implementation
    pub fn eval_filter(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }

        let predicate = args[0].clone();
        let list = args[1].to_vector().ok_or_else(|| {
            LambdustError::type_error("filter: second argument must be a proper list".to_string())
        })?;

        let mut results = Vec::new();

        for item in list {
            let keep =
                self.apply_procedure_with_evaluator(predicate.clone(), vec![item.clone()])?;
            if keep.is_truthy() {
                results.push(item);
            }
        }

        Ok(Value::from_vector(results))
    }

    /// Special form wrapper for map
    fn eval_map_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        // Evaluate all operands first
        let mut values = Vec::new();
        for operand in operands {
            values.push(self.eval(operand.clone(), env.clone(), Continuation::Identity)?);
        }

        // Use the evaluator-integrated map implementation
        let result = self.eval_map(values)?;
        self.apply_continuation(cont, result)
    }

    /// Special form wrapper for apply
    fn eval_apply_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        // Evaluate all operands first
        let mut values = Vec::new();
        for operand in operands {
            values.push(self.eval(operand.clone(), env.clone(), Continuation::Identity)?);
        }

        // Use the evaluator-integrated apply implementation
        let result = self.eval_apply(values)?;
        self.apply_continuation(cont, result)
    }

    /// Special form wrapper for fold
    fn eval_fold_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        // Evaluate all operands first
        let mut values = Vec::new();
        for operand in operands {
            values.push(self.eval(operand.clone(), env.clone(), Continuation::Identity)?);
        }

        // Use the evaluator-integrated fold implementation
        let result = self.eval_fold(values)?;
        self.apply_continuation(cont, result)
    }

    /// Special form wrapper for fold-right
    fn eval_fold_right_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        // Evaluate all operands first
        let mut values = Vec::new();
        for operand in operands {
            values.push(self.eval(operand.clone(), env.clone(), Continuation::Identity)?);
        }

        // Use the evaluator-integrated fold-right implementation
        let result = self.eval_fold_right(values)?;
        self.apply_continuation(cont, result)
    }

    /// Special form wrapper for filter
    fn eval_filter_special_form(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        // Evaluate all operands first
        let mut values = Vec::new();
        for operand in operands {
            values.push(self.eval(operand.clone(), env.clone(), Continuation::Identity)?);
        }

        // Use the evaluator-integrated filter implementation
        let result = self.eval_filter(values)?;
        self.apply_continuation(cont, result)
    }
}
