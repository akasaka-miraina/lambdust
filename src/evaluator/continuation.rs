//! Continuation system for R7RS formal semantics
//!
//! This module implements the continuation data structures and operations
//! following the R7RS formal semantics specification.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::value::Value;
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