//! Metadata about an evaluation

use super::monad_type::MonadType;

/// Metadata about an evaluation
#[derive(Debug, Clone)]
pub struct EvaluationMetadata {
    /// Number of evaluation steps taken
    pub steps_taken: usize,
    
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    
    /// Monads that were used
    pub monads_used: Vec<MonadType>,
    
    /// Whether tail call optimization was applied
    pub tail_call_optimized: bool,
}