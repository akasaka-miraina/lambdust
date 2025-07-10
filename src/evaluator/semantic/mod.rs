//! Pure R7RS semantic evaluator
//!
//! This module implements a pure R7RS formal semantics evaluator that
//! contains NO optimizations whatsoever. It serves as the reference
//! implementation for verification against optimized execution paths.

// Core functionality and data structures
mod semantic_core;
mod semantic_continuation;
mod semantic_special_forms;
mod semantic_builtins;
mod semantic_reduction;

// Re-export main types
pub use semantic_core::{SemanticEvaluator, ReductionStats};

// Implementation modules are included via separate files
// to maintain clean separation of concerns and file size limits

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::evaluator::Continuation;
    use crate::lexer::SchemeNumber;
    use crate::environment::Environment;
    use std::rc::Rc;

    #[test]
    fn test_modular_structure() {
        let evaluator = SemanticEvaluator::new();
        assert!(matches!(evaluator.get_reduction_stats(), ReductionStats { .. }));
    }

    #[test]
    fn test_basic_evaluation() {
        let mut evaluator = SemanticEvaluator::new();
        let env = Rc::new(Environment::new());
        
        // Test simple literal evaluation
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = evaluator.eval_pure(expr, env, Continuation::Identity);
        
        assert!(result.is_ok());
    }
}