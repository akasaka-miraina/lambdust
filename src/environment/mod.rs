//! Environment management for Lambdust Scheme interpreter
//!
//! This module provides the environment system, including:
//! - Shared environment with Arc<Environment>
//! - Copy-on-Write optimizations
//! - Environment change tracking
//! - Multi-threaded environment access

pub mod core;

// Re-export main environment types
pub use core::*;