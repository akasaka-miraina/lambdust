//! Result of monadic expression evaluation

use crate::effects::Effect;
use crate::eval::Value;

use super::{
    evaluation_metadata::EvaluationMetadata, evaluation_metrics::EvaluationMetrics,
    monadic_computation::MonadicComputation,
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
