//! Evaluation order strategies and exception handling types
//!
//! This module defines evaluation order strategies and exception handler information
//! for the R7RS evaluator.

use crate::environment::Environment;
use crate::value::Value;
use std::rc::Rc;

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