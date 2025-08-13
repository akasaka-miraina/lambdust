use crate::eval::{Value, monadic_architecture::MonadicEvaluationResult};
use super::MockMonadicComputation;

/// Expected outcome of a test
#[derive(Debug, Clone)]
pub enum ExpectedOutcome {
    /// Expect successful evaluation with specific value
    Success(Value),
    
    /// Expect successful evaluation with any value
    SuccessAny,
    
    /// Expect error with specific message
    Error(String),
    
    /// Expect error of any kind
    ErrorAny,
    
    /// Expect specific monadic computation
    MonadicComputation(MockMonadicComputation),
    
    /// Custom validation function
    Custom(fn(&MonadicEvaluationResult) -> bool),
}