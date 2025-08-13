//! Runtime environment manipulation and module management.
//!
//! This module provides facilities for dynamic environment manipulation,
//! module loading/unloading, garbage collection control, and memory monitoring.
//!
//! Note: The original structures have been migrated to focused modules:
//! - Environment management: environment_management.rs
//! - Environment hierarchy: environment_hierarchy.rs
//! - Change tracking: environment_tracking.rs
//! - Module management: module_management.rs
//! - Memory management: memory_management.rs
//! - GC policies: gc_policy.rs
//! - Memory pressure: memory_pressure.rs

// Re-export all migrated structures for backward compatibility
pub use super::environment_management::*;
pub use super::environment_hierarchy::*;
pub use super::environment_tracking::*;
pub use super::module_management::*;
pub use super::memory_management::*;
pub use super::gc_policy::*;
pub use super::memory_pressure::*;