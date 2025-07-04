//! Control flow module
//!
//! This module implements R7RS control flow constructs organized into focused submodules:
//! - promises: Promise and lazy evaluation (delay, force, lazy, promise?)
//! - do_loops: Do loop iteration with step expressions
//! - call_cc: Call-with-current-continuation and continuation capture
//! - multi_values: Multiple values system (values, call-with-values)
//! - dynamic_wind: Dynamic extent management
//! - exceptions: Exception handling (raise, with-exception-handler, guard)
//! - continuations: Centralized continuation application handlers

// Module declarations
mod call_cc;
mod continuations;
mod do_loops;
mod dynamic_wind;
mod exceptions;
mod multi_values;
mod promises;

// Re-export all functions
pub use call_cc::eval_call_cc;
pub use continuations::apply_control_flow_continuation;
pub use do_loops::eval_do;
pub use dynamic_wind::eval_dynamic_wind;
pub use exceptions::{eval_guard, eval_raise, eval_with_exception_handler};
pub use multi_values::{eval_call_with_values, eval_values};
pub use promises::{eval_delay, eval_force, eval_lazy, eval_promise_predicate};
