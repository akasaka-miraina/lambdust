//! Input for monadic expression evaluation

use crate::eval::{Environment, operational_semantics::EvaluationContext};
use crate::ast::{Expr, Spanned};
use std::rc::Rc;

use super::monad_type::MonadType;

/// Input for monadic expression evaluation
#[derive(Debug, Clone)]
pub struct MonadicEvaluationInput {
    /// The expression to evaluate
    pub expression: Spanned<Expr>,
    
    /// The environment for evaluation
    pub environment: Rc<Environment>,
    
    /// Expected result type
    pub expected_monad: Option<MonadType>,
    
    /// Additional context
    pub context: EvaluationContext,
}