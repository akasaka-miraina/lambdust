//! Code quality metrics and optimization opportunity analysis.

use crate::diagnostics::Span;
use super::analysis_types::{OptimizationType, OptimizationImpact};
use std::collections::HashMap;

/// Code quality metrics.
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Cyclomatic complexity
    pub complexity: HashMap<String, usize>,
    /// Lines of code
    pub lines_of_code: usize,
    /// Number of functions
    pub function_count: usize,
    /// Average function length
    pub avg_function_length: f64,
    /// Nesting depth
    pub max_nesting_depth: usize,
    /// Code duplication
    pub duplication: Vec<DuplicationInfo>,
}

/// Code duplication information.
#[derive(Debug, Clone)]
pub struct DuplicationInfo {
    /// Duplicated code blocks
    pub blocks: Vec<Span>,
    /// Similarity score
    pub similarity: f64,
    /// Length of duplication
    pub length: usize,
}

/// Optimization opportunity.
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Description
    pub description: String,
    /// Location where optimization can be applied
    pub location: Option<Span>,
    /// Estimated impact
    pub impact: OptimizationImpact,
    /// Suggested changes
    pub suggestion: String,
}

impl QualityMetrics {
    /// Creates new empty quality metrics.
    pub fn new() -> Self {
        Self {
            complexity: HashMap::new(),
            lines_of_code: 0,
            function_count: 0,
            avg_function_length: 0.0,
            max_nesting_depth: 0,
            duplication: Vec::new(),
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}