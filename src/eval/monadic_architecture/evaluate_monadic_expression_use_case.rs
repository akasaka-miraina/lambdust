//! Use case: Evaluate a monadic expression

use std::sync::Arc;
use super::monadic_evaluation_orchestrator::MonadicEvaluationOrchestrator;

/// Use case: Evaluate a monadic expression
#[derive(Debug)]
pub struct EvaluateMonadicExpressionUseCase {
    /// The orchestrator that handles this use case
    pub orchestrator: Arc<MonadicEvaluationOrchestrator>,
}