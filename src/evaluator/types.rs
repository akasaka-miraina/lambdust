//! Core data types for the R7RS evaluator
//!
//! This module defines the basic data structures used by the evaluator,
//! including Store, evaluation order, and exception handling.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::continuation::DynamicPoint;
use crate::srfi::SrfiRegistry;
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Store (memory) for locations
#[derive(Debug, Clone)]
pub struct Store {
    /// Mapping from locations to values
    locations: HashMap<usize, Value>,
    /// Next available location
    next_location: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    /// Create a new store
    pub fn new() -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
        }
    }

    /// Allocate a new location
    pub fn allocate(&mut self, value: Value) -> usize {
        let loc = self.next_location;
        self.locations.insert(loc, value);
        self.next_location += 1;
        loc
    }

    /// Get value at location
    pub fn get(&self, location: usize) -> Option<&Value> {
        self.locations.get(&location)
    }

    /// Set value at location
    pub fn set(&mut self, location: usize, value: Value) -> Result<()> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.locations.entry(location) {
            e.insert(value);
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )))
        }
    }
}

/// Evaluation order strategy for modeling unspecified order
#[derive(Debug, Clone)]
pub enum EvalOrder {
    /// Left-to-right evaluation
    LeftToRight,
    /// Right-to-left evaluation
    RightToLeft,
    /// Random/unspecified order (for testing compliance)
    Unspecified,
}

/// Exception handler information for exception handling
#[derive(Debug, Clone)]
pub struct ExceptionHandlerInfo {
    /// Handler procedure
    pub handler: Value,
    /// Handler environment
    pub env: Rc<Environment>,
}

/// Formal evaluator implementing R7RS semantics
#[derive(Debug)]
pub struct Evaluator {
    /// Current store (memory)
    #[allow(dead_code)]
    store: Store,
    /// Dynamic points stack
    #[allow(dead_code)]
    dynamic_points: Vec<DynamicPoint>,
    /// Evaluation order strategy
    eval_order: EvalOrder,
    /// Global environment
    pub global_env: Rc<Environment>,
    /// Recursion depth counter for stack overflow prevention
    recursion_depth: usize,
    /// Maximum recursion depth
    max_recursion_depth: usize,
    /// Exception handlers stack for exception handling
    exception_handlers: Vec<ExceptionHandlerInfo>,
    /// SRFI registry for module imports
    srfi_registry: SrfiRegistry,
}

impl Evaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000, // Configurable recursion limit
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::new(),
        }
    }

    /// Create evaluator with custom evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        Evaluator {
            store: Store::new(),
            dynamic_points: Vec::new(),
            eval_order,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::new(),
        }
    }

    /// Get the current evaluation order
    pub fn eval_order(&self) -> &EvalOrder {
        &self.eval_order
    }

    /// Get current recursion depth
    pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Get maximum recursion depth
    pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// Get mutable reference to exception handlers
    pub fn exception_handlers_mut(&mut self) -> &mut Vec<ExceptionHandlerInfo> {
        &mut self.exception_handlers
    }

    /// Get reference to exception handlers
    pub fn exception_handlers(&self) -> &[ExceptionHandlerInfo] {
        &self.exception_handlers
    }

    /// Get mutable reference to SRFI registry
    pub fn srfi_registry_mut(&mut self) -> &mut SrfiRegistry {
        &mut self.srfi_registry
    }

    /// Get reference to SRFI registry
    pub fn srfi_registry(&self) -> &SrfiRegistry {
        &self.srfi_registry
    }

    /// Increment recursion depth
    pub fn increment_recursion_depth(&mut self) -> Result<()> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            Err(LambdustError::runtime_error(
                "Maximum recursion depth exceeded".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Decrement recursion depth
    pub fn decrement_recursion_depth(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}