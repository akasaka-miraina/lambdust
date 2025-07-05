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
            _ => false,
        }
    }
}

impl Value {
    /// Check if this value is a procedure
    pub fn is_procedure(&self) -> bool {
        matches!(self, Value::Procedure(_))
    }

    /// Get the procedure if this is a procedure
    pub fn as_procedure(&self) -> Option<&Procedure> {
        match self {
            Value::Procedure(p) => Some(p),
            _ => None,
        }
    }
}
