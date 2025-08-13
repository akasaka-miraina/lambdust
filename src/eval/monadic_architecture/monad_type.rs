//! Types of monads available in the system

/// Types of monads available in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonadType {
    /// Identity monad - no computational context
    Identity,
    /// Maybe monad - computations that may fail
    Maybe,
    /// Either monad - computations with error handling
    Either,
    /// IO monad - computations with side effects
    IO,
    /// State monad - stateful computations
    State,
    /// Reader monad - environment-based computations
    Reader,
    /// Continuation monad - control flow computations
    Continuation,
    /// List monad - non-deterministic computations
    List,
    /// Custom monad type
    Custom(String),
}