//! Effect dependency graph for tracking effect relationships.

use std::collections::HashMap;

/// Dependency graph for effects.
#[derive(Debug, Default)]
pub struct EffectDependencyGraph {
    /// Dependencies between effects
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    dependencies: HashMap<u64, Vec<u64>>,
    /// Reverse dependencies for efficient lookup
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    reverse_dependencies: HashMap<u64, Vec<u64>>,
}