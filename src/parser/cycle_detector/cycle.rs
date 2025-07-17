//! Represents a cycle in the dependency graph

use std::fmt;
use crate::parser::cycle_detector::cycle_type::CycleType;

/// Represents a cycle in the dependency graph
///
/// A cycle indicates a circular dependency between functions or variables
/// in the Scheme program, which needs to be detected and handled appropriately.
#[derive(Debug, Clone, PartialEq)]
pub struct Cycle {
    /// Names of variables/functions in the cycle
    pub nodes: Vec<String>,
    /// Type of cycle
    pub cycle_type: CycleType,
}

impl fmt::Display for Cycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cycle_type {
            CycleType::SelfReference => write!(f, "{} → {}", self.nodes[0], self.nodes[0]),
            CycleType::Direct => write!(f, "{} ➡ {}", self.nodes.join(" → "), self.nodes[0]),
            CycleType::Indirect => write!(f, "{} ↪ {}", self.nodes.join(" → "), self.nodes[0]),
        }
    }
}

impl Cycle {

    /// Get the length of the cycle
    #[must_use] pub fn length(&self) -> usize {
        self.nodes.len()
    }
}
