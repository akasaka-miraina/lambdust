//! Effect system using monads for the Lambdust language.
//!
//! This module implements a comprehensive effect system that tracks and transforms
//! side effects through monadic abstractions, providing:
//!
//! - Transparent effect tracking for pure vs. impure computations
//! - Monadic effects (IO, State, Error) with automatic lifting
//! - Generational environments for handling mutations
//! - Effect handlers for custom effect management
//! - Integration with the type system for effect inference
//!
//! The system preserves Scheme semantics while enabling pure functional programming.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub mod monad;
pub mod handler;
pub mod generational;
pub mod lifting;
pub mod advanced_monads;
pub mod continuation_monad;
pub mod builtin_monads;
pub mod list_monad;
pub mod parser_monad;

// Individual structure modules
pub mod effect_context;
pub mod effect_handler_ref;
pub mod effect_system;
pub mod lifting_config;

pub use monad::*;
pub use handler::*;
pub use generational::*;
pub use lifting::*;
pub use advanced_monads::*;

// Import continuation monad components selectively to avoid conflicts
pub use continuation_monad::{
    ContinuationMonad, ContinuationFunction, ContinuationComputation, 
    EvaluationFrame, EffectfulComputation, run_continuation, escape_continuation
};

// Import builtin monads selectively to avoid conflicts
pub use builtin_monads::{
    Maybe, Either, IO, State, Reader, IOContext, FileMode, FileHandle,
    ListMonad, ParserMonad
};

// Import list and parser monads
pub use list_monad::{List, ValueList, ListFunc};
pub use parser_monad::{Parser, ParseResult, ParseError, Input, Position, ParserCache};

// Re-export with aliases to resolve conflicts
pub use continuation_monad::ContIOAction;
pub use continuation_monad::ContStateAction;
// Note: builtin_monads doesn't export IOAction/StateAction directly, they're nested in enums

// Re-export individual structures
pub use effect_context::*;
pub use effect_handler_ref::*;
pub use effect_system::*;
pub use lifting_config::*;

/// Effect types in the Lambdust language.
///
/// Effects track the computational context and side effects that
/// operations may produce. The effect system ensures that pure
/// computations remain referentially transparent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Pure computation (no effects)
    Pure,
    /// IO effects (input/output operations)
    IO,
    /// State effects (mutations that create new generations)
    State,
    /// Error effects (exceptions and error handling)
    Error,
    /// Custom effects with a name
    Custom(String),
}

// EffectContext moved to effect_context.rs

// EffectHandlerRef moved to effect_handler_ref.rs

/// Trait for effect handlers that can manage computational effects.
pub trait EffectHandler: std::fmt::Debug {
    /// Handles an effect with the given arguments.
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult>;
    
    /// Returns the name of the effect this handler manages.
    fn effect_name(&self) -> &str;
    
    /// Returns true if this handler can handle the given effect.
    fn can_handle(&self, effect: &Effect) -> bool;
}

/// Result of handling an effect.
#[derive(Debug, Clone)]
pub enum EffectResult {
    /// Effect was handled successfully with a value
    Value(Value),
    /// Effect was handled and should continue with another computation (simplified)
    Continue(Value),
    /// Effect was not handled by this handler
    Unhandled,
    /// Effect handling resulted in an error
    Error(DiagnosticError),
}

// EffectSystem moved to effect_system.rs

// LiftingConfig moved to lifting_config.rs

// LiftingRule is in lifting.rs (more complex version with transformations)

/// Condition for when to apply an effect lifting rule.
#[derive(Debug, Clone)]
pub enum LiftingCondition {
    /// Always apply this rule
    Always,
    /// Apply when the operation name matches
    OperationName(String),
    /// Apply when any of the effects are present
    HasEffect(Vec<Effect>),
    /// Custom condition function
    Custom(fn(&str, &[Effect]) -> bool),
}

impl Effect {
    /// Returns true if this effect is pure.
    pub fn is_pure(&self) -> bool {
        matches!(self, Effect::Pure)
    }
    
    /// Returns true if this effect represents IO.
    pub fn is_io(&self) -> bool {
        matches!(self, Effect::IO)
    }
    
    /// Returns true if this effect represents state.
    pub fn is_state(&self) -> bool {
        matches!(self, Effect::State)
    }
    
    /// Returns true if this effect represents errors.
    pub fn is_error(&self) -> bool {
        matches!(self, Effect::Error)
    }
    
    /// Combines two effects, returning the more "impure" one.
    pub fn combine(&self, other: &Effect) -> Effect {
        match (self, other) {
            (Effect::Pure, other) => other.clone()),
            (this, Effect::Pure) => this.clone()),
            (Effect::Error, _) | (_, Effect::Error) => Effect::Error,
            (Effect::IO, _) | (_, Effect::IO) => Effect::IO,
            (Effect::State, _) | (_, Effect::State) => Effect::State,
            (Effect::Custom(a), Effect::Custom(b)) if a == b => Effect::Custom(a.clone()),
            (Effect::Custom(a), _) => Effect::Custom(a.clone()),
        }
    }
    
    /// Returns the "strength" of this effect for ordering.
    pub fn strength(&self) -> u8 {
        match self {
            Effect::Pure => 0,
            Effect::State => 1,
            Effect::IO => 2,
            Effect::Error => 3,
            Effect::Custom(_) => 4,
        }
    }
}

impl PartialOrd for Effect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Effect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.strength().cmp(&other.strength())
    }
}

// EffectContext implementations moved to effect_context.rs

// EffectHandlerRef implementations moved to effect_handler_ref.rs

// EffectSystem implementations moved to effect_system.rs

// LiftingConfig implementations moved to lifting_config.rs

// LiftingCondition implementations kept in main mod.rs (it's an enum, not a struct)

// Default implementations moved to respective structure files

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Pure => write!(f, "Pure"),
            Effect::IO => write!(f, "IO"),
            Effect::State => write!(f, "State"),
            Effect::Error => write!(f, "Error"),
            Effect::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

// Display implementations moved to respective structure files