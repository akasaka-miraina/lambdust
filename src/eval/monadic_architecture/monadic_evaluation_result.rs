//! Result of monadic expression evaluation

use crate::eval::Value;
use crate::effects::Effect;

use super::{
    monadic_computation::MonadicComputation,
    evaluation_metadata::EvaluationMetadata,
    evaluation_metrics::EvaluationMetrics,
};

/// Result of monadic expression evaluation
#[derive(Debug, Clone)]
pub struct MonadicEvaluationResult {
    /// The resulting computation
    pub computation: MonadicComputation<Value>,
    
    /// Metadata about the evaluation
    pub metadata: EvaluationMetadata,
    
    /// Any side effects that occurred
    pub effects: Vec<Effect>,
    
    /// Performance metrics
    pub metrics: EvaluationMetrics,
}