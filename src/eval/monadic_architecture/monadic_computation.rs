//! Pure monadic computation - the core domain entity.
//!
//! This represents a computation that may have effects but expresses
//! them in a pure, mathematical way through monadic structures.

use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::effects::{
    ContinuationMonad, Effect, EffectContext, EffectfulComputation, Either, IO, Maybe, Reader,
    State,
};
use crate::eval::{
    Environment, Value,
    continuation_domain::{CapturedContinuation, ContinuationId},
    operational_semantics::{ComputationState, EvaluationContext},
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

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
