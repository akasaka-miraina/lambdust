//! Continuation system for R7RS formal semantics
//!
//! This module implements the continuation data structures and operations
//! following the R7RS formal semantics specification.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::value::Value;
use std::rc::Rc;

/// Lightweight continuation operations for common cases
/// This enum represents simple continuations that can be inlined for performance
#[derive(Debug, Clone)]
pub enum LightContinuation {
    /// Identity continuation (direct return)
    Identity,
    /// Simple value accumulation
    Values(Vec<Value>),
    /// Variable assignment operation
    Assignment {
        /// Variable name to assign to
        var_name: String,
        /// Environment for the assignment
        env: Rc<Environment>,
    },
    /// Begin sequence evaluation
    Begin {
        /// Remaining expressions to evaluate
        remaining: Vec<Expr>,
        /// Environment for evaluation
        env: Rc<Environment>,
    },
    /// Define operation with variable name
    Define {
        /// Variable to define
        variable: String,
        /// Environment for definition
        env: Rc<Environment>,
    },
}

impl LightContinuation {
    /// Check if a continuation can be converted to a lightweight variant
    pub fn from_continuation(cont: &Continuation) -> Option<Self> {
        match cont {
            Continuation::Identity => Some(LightContinuation::Identity),

            // Simple cases with Identity parent
            Continuation::Values { values, parent }
                if matches!(**parent, Continuation::Identity) =>
            {
                Some(LightContinuation::Values(values.clone()))
            }
            Continuation::Assignment {
                variable,
                env,
                parent,
            } if matches!(**parent, Continuation::Identity) => {
                Some(LightContinuation::Assignment {
                    var_name: variable.clone(),
                    env: env.clone(),
                })
            }
            Continuation::Begin {
                remaining,
                env,
                parent,
            } if matches!(**parent, Continuation::Identity) && remaining.is_empty() => {
                Some(LightContinuation::Begin {
                    remaining: remaining.clone(),
                    env: env.clone(),
                })
            }
            Continuation::Define {
                variable,
                env,
                parent,
            } if matches!(**parent, Continuation::Identity) => Some(LightContinuation::Define {
                variable: variable.clone(),
                env: env.clone(),
            }),

            // Skip complex cases that require evaluator context
            // SimpleApplication and IfTest are disabled to avoid context issues
            _ => None,
        }
    }

    /// Apply a lightweight continuation (inlined for performance)
    #[inline]
    pub fn apply(self, value: Value) -> Result<Value, crate::error::LambdustError> {
        match self {
            LightContinuation::Identity => Ok(value),
            LightContinuation::Values(mut values) => {
                values.push(value);
                Ok(Value::Values(values))
            }
            LightContinuation::Assignment { var_name, env } => {
                env.set(&var_name, value)?;
                Ok(Value::Undefined)
            }
            LightContinuation::Begin { remaining, env: _ } => {
                if remaining.is_empty() {
                    Ok(value)
                } else {
                    // For complex Begin operations, fall back to full continuation
                    Err(crate::error::LambdustError::runtime_error(
                        "Complex Begin operation requires full continuation".to_string(),
                    ))
                }
            }
            LightContinuation::Define { variable, env } => {
                // Define operation can be inlined
                env.define(variable, value);
                Ok(Value::Undefined)
            }
        }
    }
}

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

impl Continuation {
    /// Calculate the depth of continuation chain
    pub fn depth(&self) -> usize {
        match self {
            Continuation::Identity => 0,
            Continuation::Application { parent, .. } => parent.depth() + 1,
            Continuation::Operator { parent, .. } => parent.depth() + 1,
            Continuation::IfTest { parent, .. } => parent.depth() + 1,
            Continuation::Assignment { parent, .. } => parent.depth() + 1,
            Continuation::Begin { parent, .. } => parent.depth() + 1,
            Continuation::Values { parent, .. } => parent.depth() + 1,
            Continuation::ValuesAccumulate { parent, .. } => parent.depth() + 1,
            Continuation::VectorEval { parent, .. } => parent.depth() + 1,
            Continuation::Define { parent, .. } => parent.depth() + 1,
            Continuation::CallCc { parent, .. } => parent.depth() + 1,
            Continuation::Do { parent, .. } => parent.depth() + 1,
            Continuation::DynamicWind { parent, .. } => parent.depth() + 1,
            Continuation::ExceptionHandler { parent, .. } => parent.depth() + 1,
            Continuation::GuardClause { parent, .. } => parent.depth() + 1,
            Continuation::CondTest { parent, .. } => parent.depth() + 1,
            Continuation::And { parent, .. } => parent.depth() + 1,
            Continuation::Or { parent, .. } => parent.depth() + 1,
            Continuation::CallWithValuesStep1 { parent, .. } => parent.depth() + 1,
            Continuation::CallWithValuesStep2 { parent, .. } => parent.depth() + 1,
            Continuation::Captured { .. } => 0, // Captured continuations don't have parents
        }
    }

    /// Find the root (deepest) continuation in the chain
    /// This is used for complete non-local exit in call/cc
    pub fn find_root_continuation(&self) -> &Continuation {
        match self {
            Continuation::Identity => self,
            Continuation::Application { parent, .. } => parent.find_root_continuation(),
            Continuation::Operator { parent, .. } => parent.find_root_continuation(),
            Continuation::IfTest { parent, .. } => parent.find_root_continuation(),
            Continuation::Assignment { parent, .. } => parent.find_root_continuation(),
            Continuation::Begin { parent, .. } => parent.find_root_continuation(),
            Continuation::Values { parent, .. } => parent.find_root_continuation(),
            Continuation::ValuesAccumulate { parent, .. } => parent.find_root_continuation(),
            Continuation::VectorEval { parent, .. } => parent.find_root_continuation(),
            Continuation::Define { parent, .. } => parent.find_root_continuation(),
            Continuation::CallCc { parent, .. } => parent.find_root_continuation(),
            Continuation::Do { parent, .. } => parent.find_root_continuation(),
            Continuation::DynamicWind { parent, .. } => parent.find_root_continuation(),
            Continuation::ExceptionHandler { parent, .. } => parent.find_root_continuation(),
            Continuation::GuardClause { parent, .. } => parent.find_root_continuation(),
            Continuation::CondTest { parent, .. } => parent.find_root_continuation(),
            Continuation::And { parent, .. } => parent.find_root_continuation(),
            Continuation::Or { parent, .. } => parent.find_root_continuation(),
            Continuation::CallWithValuesStep1 { parent, .. } => parent.find_root_continuation(),
            Continuation::CallWithValuesStep2 { parent, .. } => parent.find_root_continuation(),
            Continuation::Captured { cont } => cont.find_root_continuation(),
        }
    }

    /// Check if this continuation represents an intermediate computation
    /// These continuations should be skipped during non-local exit
    pub fn is_intermediate_computation(&self) -> bool {
        matches!(
            self,
            Continuation::Application { .. }
                | Continuation::Operator { .. }
                | Continuation::Values { .. }
                | Continuation::ValuesAccumulate { .. }
                | Continuation::VectorEval { .. }
        )
    }

    /// Get the parent continuation, if any
    pub fn parent(&self) -> Option<&Continuation> {
        match self {
            Continuation::Identity => None,
            Continuation::Application { parent, .. } => Some(parent),
            Continuation::Operator { parent, .. } => Some(parent),
            Continuation::IfTest { parent, .. } => Some(parent),
            Continuation::Assignment { parent, .. } => Some(parent),
            Continuation::Begin { parent, .. } => Some(parent),
            Continuation::Values { parent, .. } => Some(parent),
            Continuation::ValuesAccumulate { parent, .. } => Some(parent),
            Continuation::VectorEval { parent, .. } => Some(parent),
            Continuation::Define { parent, .. } => Some(parent),
            Continuation::CallCc { parent, .. } => Some(parent),
            Continuation::Do { parent, .. } => Some(parent),
            Continuation::DynamicWind { parent, .. } => Some(parent),
            Continuation::ExceptionHandler { parent, .. } => Some(parent),
            Continuation::GuardClause { parent, .. } => Some(parent),
            Continuation::CondTest { parent, .. } => Some(parent),
            Continuation::And { parent, .. } => Some(parent),
            Continuation::Or { parent, .. } => Some(parent),
            Continuation::CallWithValuesStep1 { parent, .. } => Some(parent),
            Continuation::CallWithValuesStep2 { parent, .. } => Some(parent),
            Continuation::Captured { .. } => None, // Captured continuations don't have logical parents
        }
    }
}
