//! Variable usage and scope analysis components.

use crate::diagnostics::Span;
use super::analysis_types::ScopeType;
use std::collections::HashMap;

/// Variable usage analysis.
#[derive(Debug, Clone)]
pub struct VariableUsage {
    /// Variables and their usage patterns
    pub variables: HashMap<String, VariableInfo>,
    /// Unused variables
    pub unused: Vec<String>,
    /// Potentially uninitialized variables
    pub uninitialized: Vec<String>,
}

/// Information about a variable.
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Definition location
    pub definition: Option<Span>,
    /// Usage locations
    pub uses: Vec<Span>,
    /// Whether it's read
    pub read: bool,
    /// Whether it's written
    pub written: bool,
    /// Whether it's captured in a closure
    pub captured: bool,
    /// Scope information
    pub scope: ScopeInfo,
}

/// Scope information for variables.
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// Scope type
    pub scope_type: ScopeType,
    /// Nesting level
    pub level: usize,
    /// Scope identifier
    pub scope_id: String,
}

impl VariableUsage {
    /// Creates a new empty variable usage analysis.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            unused: Vec::new(),
            uninitialized: Vec::new(),
        }
    }
}

impl Default for VariableUsage {
    fn default() -> Self {
        Self::new()
    }
}