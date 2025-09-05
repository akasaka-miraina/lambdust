//! Use case: Evaluate a monadic expression

use super::monadic_evaluation_orchestrator::MonadicEvaluationOrchestrator;
use std::sync::Arc;

/// Use case: Evaluate a monadic expression
#[derive(Debug)]
pub struct EvaluateMonadicExpressionUseCase {
    /// The orchestrator that handles this use case
    pub orchestrator: Arc<MonadicEvaluationOrchestrator>,
}
