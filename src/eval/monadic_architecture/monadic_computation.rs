//! Pure monadic computation - the core domain entity.
//!
//! This represents a computation that may have effects but expresses
//! them in a pure, mathematical way through monadic structures.

use crate::eval::{
    Value, Environment, 
    operational_semantics::{EvaluationContext, ComputationState},
    continuation_domain::{CapturedContinuation, ContinuationId},
};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Result, Error, Span};
use crate::effects::{
    Effect, EffectContext, Maybe, Either, IO, State, Reader,
    ContinuationMonad, EffectfulComputation,
};
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;

use super::monadic_transformation::MonadicTransformation;

/// Pure monadic computation - the core domain entity.
///
/// This represents a computation that may have effects but expresses
/// them in a pure, mathematical way through monadic structures.
#[derive(Debug, Clone)]
pub enum MonadicComputation<T: Clone> {
    /// Pure value computation
    Pure(T),
    
    /// Continuation monad computation
    Continuation(ContinuationMonad<T>),
    
    /// Maybe monad computation (optional values)
    Maybe(Maybe<T>),
    
    /// Either monad computation (error handling)
    Either(Either<Error, T>),
    
    /// IO monad computation
    IO(IO<T>),
    
    /// State monad computation
    State(State<Rc<Environment>, T>),
    
    /// Reader monad computation
    Reader(Reader<Rc<Environment>, T>),
    
    /// Composed monadic computation (monad transformers)
    Composed {
        /// Inner computation
        inner: Box<MonadicComputation<Value>>,
        
        /// Transformation function
        transform: MonadicTransformation<T>,
    },
}