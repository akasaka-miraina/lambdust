//! Procedure types and operations

use super::{Continuation, Value};
use crate::ast::Expr;
use crate::environment::Environment;
use std::rc::Rc;

/// Procedure representation
#[derive(Clone)]
pub enum Procedure {
    /// User-defined procedure (lambda)
    Lambda {
        /// Parameter names
        params: Vec<String>,
        /// Whether this is a variadic procedure
        variadic: bool,
        /// Body expressions
        body: Vec<Expr>,
        /// Closure environment
        closure: Rc<Environment>,
    },
    /// Built-in procedure
    Builtin {
        /// Procedure name
        name: String,
        /// Arity (number of arguments, None for variadic)
        arity: Option<usize>,
        /// Function pointer
        func: fn(&[Value]) -> crate::Result<Value>,
    },
    /// Host function (dynamic closure)
    HostFunction {
        /// Function name
        name: String,
        /// Arity (number of arguments, None for variadic)
        arity: Option<usize>,
        /// Function closure
        func: crate::host::HostFunc,
    },
    /// Continuation (for call/cc)
    Continuation {
        /// Captured continuation
        continuation: Box<Continuation>,
    },
    /// Captured continuation from call/cc (evaluator internal)
    CapturedContinuation {
        /// Captured evaluator continuation
        continuation: Box<crate::evaluator::Continuation>,
    },
    /// Reusable captured continuation with context preservation
    ReusableContinuation {
        /// Captured evaluator continuation
        continuation: Box<crate::evaluator::Continuation>,
        /// Capture environment for context restoration
        capture_env: Rc<Environment>,
        /// Unique reuse identifier
        reuse_id: usize,
        /// Whether this continuation was from call/cc escape or explicit storage
        is_escaping: bool,
    },
}

impl std::fmt::Debug for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lambda {
                params,
                variadic,
                body,
                ..
            } => f
                .debug_struct("Lambda")
                .field("params", params)
                .field("variadic", variadic)
                .field("body", body)
                .finish(),
            Self::Builtin { name, arity, .. } => f
                .debug_struct("Builtin")
                .field("name", name)
                .field("arity", arity)
                .finish(),
            Self::HostFunction { name, arity, .. } => f
                .debug_struct("HostFunction")
                .field("name", name)
                .field("arity", arity)
                .finish(),
            Self::Continuation { continuation } => f
                .debug_struct("Continuation")
                .field("continuation", continuation)
                .finish(),
            Self::CapturedContinuation { .. } => f.debug_struct("CapturedContinuation").finish(),
            Self::ReusableContinuation {
                reuse_id,
                is_escaping,
                ..
            } => f
                .debug_struct("ReusableContinuation")
                .field("reuse_id", reuse_id)
                .field("is_escaping", is_escaping)
                .finish(),
        }
    }
}

impl Procedure {
    /// Call the procedure with the given arguments
    pub fn call(&self, args: &[crate::value::Value]) -> crate::error::Result<crate::value::Value> {
        match self {
            Self::Builtin { func, .. } => (func)(args),
            Self::Lambda { .. } => {
                // Lambda calls require evaluator context, which is not available here
                // This is a simplified implementation that will need proper evaluator integration
                Err(crate::error::LambdustError::runtime_error(
                    "Lambda procedure calls require evaluator context".to_string()
                ))
            }
            Self::HostFunction { .. } => {
                // Host function calls also require evaluator context
                Err(crate::error::LambdustError::runtime_error(
                    "Host function calls require evaluator context".to_string()
                ))
            }
            Self::Continuation { .. } => {
                Err(crate::error::LambdustError::runtime_error(
                    "Continuation calls require evaluator context".to_string()
                ))
            }
            Self::CapturedContinuation { .. } => {
                Err(crate::error::LambdustError::runtime_error(
                    "Captured continuation calls require evaluator context".to_string()
                ))
            }
            Self::ReusableContinuation { .. } => {
                Err(crate::error::LambdustError::runtime_error(
                    "Reusable continuation calls require evaluator context".to_string()
                ))
            }
        }
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Lambda {
                    params: l_params,
                    variadic: l_variadic,
                    body: l_body,
                    closure: l_closure,
                },
                Self::Lambda {
                    params: r_params,
                    variadic: r_variadic,
                    body: r_body,
                    closure: r_closure,
                },
            ) => {
                l_params == r_params
                    && l_variadic == r_variadic
                    && l_body == r_body
                    && std::ptr::eq(l_closure.as_ref(), r_closure.as_ref())
            }
            (
                Self::Builtin {
                    name: l_name,
                    arity: l_arity,
                    ..
                },
                Self::Builtin {
                    name: r_name,
                    arity: r_arity,
                    ..
                },
            ) => l_name == r_name && l_arity == r_arity,
            (
                Self::HostFunction {
                    name: l_name,
                    arity: l_arity,
                    ..
                },
                Self::HostFunction {
                    name: r_name,
                    arity: r_arity,
                    ..
                },
            ) => l_name == r_name && l_arity == r_arity,
            (Self::Continuation { continuation: _ }, Self::Continuation { continuation: _ }) => {
                false // Continuations are never equal
            }
            (Self::CapturedContinuation { .. }, Self::CapturedContinuation { .. }) => {
                false // Captured continuations are never equal
            }
            (
                Self::ReusableContinuation { reuse_id: l_id, .. },
                Self::ReusableContinuation { reuse_id: r_id, .. },
            ) => {
                l_id == r_id // Reusable continuations are equal if they have the same reuse ID
            }
            _ => false,
        }
    }
}

impl Value {
    /// Check if this value is a procedure
    #[must_use] pub fn is_procedure(&self) -> bool {
        matches!(self, Value::Procedure(_))
    }

    /// Get the procedure if this is a procedure
    #[must_use] pub fn as_procedure(&self) -> Option<&Procedure> {
        match self {
            Value::Procedure(p) => Some(p),
            _ => None,
        }
    }
}
