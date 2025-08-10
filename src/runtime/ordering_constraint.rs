//! Ordering constraints for effect execution.

use crate::effects::Effect;

/// Ordering constraint for effects.
#[derive(Debug, Clone)]
pub struct OrderingConstraint {
    /// Effect type this constraint applies to
    pub effect_type: Effect,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Priority for this constraint
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub priority: u8,
}

/// Type of ordering constraint.
#[derive(Debug, Clone)]
pub enum ConstraintType {
    /// Effects must be serialized (no parallelism)
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Serialized,
    /// Effects can run in parallel but must maintain ordering
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    OrderedParallel,
    /// Effects are isolated and can run freely
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Isolated,
}