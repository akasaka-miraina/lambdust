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
    /// Before thunk (procedure to call on entry)
    pub before: Option<Value>,
    /// After thunk (procedure to call on exit)
    pub after: Option<Value>,
    /// Parent dynamic point
    pub parent: Option<Box<DynamicPoint>>,
    /// Unique identifier for this dynamic point
    pub id: usize,
    /// Whether this dynamic point is active
    pub active: bool,
}

impl DynamicPoint {
    /// Create a new dynamic point
    pub fn new(
        before: Option<Value>,
        after: Option<Value>,
        parent: Option<Box<DynamicPoint>>,
        id: usize,
    ) -> Self {
        DynamicPoint {
            before,
            after,
            parent,
            id,
            active: true,
        }
    }

    /// Mark this dynamic point as inactive
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if this dynamic point is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the depth of this dynamic point
    pub fn depth(&self) -> usize {
        match &self.parent {
            Some(parent) => parent.depth() + 1,
            None => 0,
        }
    }

    /// Find the common ancestor with another dynamic point
    pub fn common_ancestor(&self, other: &DynamicPoint) -> Option<usize> {
        let mut self_path = Vec::new();
        let mut current = Some(self);
        
        // Collect path from self to root
        while let Some(point) = current {
            self_path.push(point.id);
            current = point.parent.as_ref().map(|p| p.as_ref());
        }
        
        // Check if other's path intersects with self's path
        let mut other_current = Some(other);
        while let Some(point) = other_current {
            if self_path.contains(&point.id) {
                return Some(point.id);
            }
            other_current = point.parent.as_ref().map(|p| p.as_ref());
        }
        
        None
    }

    /// Get all dynamic points from this to root
    pub fn path_to_root(&self) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = Some(self);
        
        while let Some(point) = current {
            path.push(point.id);
            current = point.parent.as_ref().map(|p| p.as_ref());
        }
        
        path
    }
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
    /// Values accumulate continuation (for proper left-to-right evaluation)
    ValuesAccumulate {
        /// Expressions remaining to evaluate
        remaining_exprs: Vec<Expr>,
        /// Values accumulated so far
        accumulated_values: Vec<Value>,
        /// Environment for evaluation
        env: Rc<Environment>,
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
    /// Dynamic-wind continuation (for after thunk execution)
    DynamicWind {
        /// After thunk to execute when unwinding
        after_thunk: Value,
        /// ID of the dynamic point
        dynamic_point_id: usize,
        /// Parent continuation
        parent: Box<Continuation>,
    },
}