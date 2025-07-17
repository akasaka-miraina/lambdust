//! Type of cycle detected in the dependency graph

/// Types of cycles that can be detected in dependency analysis
/// 
/// Categorizes different patterns of cyclic dependencies
/// for appropriate handling and error reporting.
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    /// Simple self-reference (a depends on a)
    SelfReference,
    /// Direct cycle (a depends on b, b depends on a)
    Direct,
    /// Indirect cycle (a → b → c → a)
    Indirect,
}

