//! Monadic transformation - pure domain logic for transforming computations

use crate::eval::Value;
use crate::diagnostics::Error;
use std::sync::Arc;

use super::{monadic_computation::MonadicComputation, monad_type::MonadType};

/// Monadic transformation - pure domain logic for transforming computations
#[derive(Clone)]
pub enum MonadicTransformation<T: Clone> {
    /// Map transformation (functor)
    Map {
        /// The mapping function to apply.
        function: Arc<dyn Fn(Value) -> T + Send + Sync>,
        /// Function name for debugging purposes.
        function_name: String, // for debugging
    },
    
    /// Bind transformation (monadic composition)
    Bind {
        /// The monadic bind function to apply.
        function: Arc<dyn Fn(Value) -> MonadicComputation<T> + Send + Sync>,
        /// Function name for debugging purposes.
        function_name: String,
    },
    
    /// Lift transformation (lift into another monad)
    Lift {
        /// The target monad type to lift into.
        target_monad: MonadType,
    },
    
    /// Filter transformation (Maybe monad)
    Filter {
        /// The predicate function for filtering.
        predicate: Arc<dyn Fn(&Value) -> bool + Send + Sync>,
        /// Predicate name for debugging purposes.
        predicate_name: String,
    },
}

impl<T: Clone> std::fmt::Debug for MonadicTransformation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonadicTransformation::Map { function_name, .. } => {
                write!(f, "Map {{ function: <{function_name}> }}")
            }
            MonadicTransformation::Bind { function_name, .. } => {
                write!(f, "Bind {{ function: <{function_name}> }}")
            }
            MonadicTransformation::Lift { target_monad } => {
                write!(f, "Lift {{ target_monad: {target_monad:?} }}")
            }
            MonadicTransformation::Filter { predicate_name, .. } => {
                write!(f, "Filter {{ predicate: <{predicate_name}> }}")
            }
        }
    }
}