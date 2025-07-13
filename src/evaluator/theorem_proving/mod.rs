//! Theorem Proving Support System
//!
//! このモジュールは形式的検証、コンビネータ削減検証、
//! R7RS意味論正当性検証、評価器システムの数学的性質証明の
//! インフラストラクチャを提供します。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（Statement, ProofTerm, 戦術等）
//! - `proof_system`: 証明状態管理とゴール処理
//! - `theorem_database`: 定理データベースと基本定理
//! - `verification_engine`: メイン検証エンジンと戦術実装

pub mod core_types;
pub mod proof_system;
pub mod theorem_database;
pub mod verification_engine;

// Re-export main types for backward compatibility
pub use core_types::{
    Statement, ProofTerm, ProofMethod, ProofTermType, ProofStep,
    ProofGoal, GoalType, Hypothesis, ProofContext,
    VerificationResult, TacticResult, ProofTactic,
    ReductionRule, Condition, ConditionPredicate,
    RuleTransformation, TypeConstraint,
};

pub use proof_system::{
    ProofState, ProofGoalBuilder, HypothesisBuilder, ProofStateValidator,
};

pub use theorem_database::{
    TheoremDatabase, CombinatorTheorem, SemanticRule, TypeRule, UserTheorem,
    DatabaseStatistics, DatabaseSummary,
};

pub use verification_engine::TheoremProvingSupport;

use crate::ast::Expr;
use crate::error::Result;
use crate::evaluator::{combinators::CombinatorExpr, SemanticEvaluator};

/// Create a new theorem proving support system with default configuration
pub fn create_theorem_proving_support(
    semantic_evaluator: SemanticEvaluator,
) -> TheoremProvingSupport {
    TheoremProvingSupport::new(semantic_evaluator)
}

/// Create a theorem proving support system with custom database
pub fn create_theorem_proving_support_with_database(
    semantic_evaluator: SemanticEvaluator,
    _database: TheoremDatabase,
) -> TheoremProvingSupport {
    let support = TheoremProvingSupport::new(semantic_evaluator);
    // In a complete implementation, we would replace the database
    // For now, we just return the support with default database
    support
}

/// Verify semantic equivalence between two expressions
pub fn verify_semantic_equivalence(
    expr1: &Expr,
    expr2: &Expr,
    semantic_evaluator: &SemanticEvaluator,
) -> Result<bool> {
    let support = TheoremProvingSupport::new(semantic_evaluator.clone());
    support.expressions_semantically_equal(expr1, expr2)
}

/// Verify combinator reduction correctness
pub fn verify_combinator_reduction(
    expr: &Expr,
    combinator_expr: &CombinatorExpr,
    semantic_evaluator: &SemanticEvaluator,
) -> Result<bool> {
    let support = TheoremProvingSupport::new(semantic_evaluator.clone());
    support.combinator_expressions_equivalent(
        &convert_expr_to_combinator(expr)?,
        combinator_expr,
    )
}

/// Verify R7RS compliance for an expression
pub fn verify_r7rs_compliance(
    expr: &Expr,
    semantic_evaluator: &SemanticEvaluator,
) -> Result<VerificationResult> {
    let mut support = TheoremProvingSupport::new(semantic_evaluator.clone());
    support.verify_statement(Statement::R7RSCompliance(expr.clone()))
}

/// Create a basic proof goal for semantic equivalence
pub fn create_semantic_equivalence_goal(
    expr1: Expr,
    expr2: Expr,
    id: Option<String>,
) -> Result<ProofGoal> {
    ProofGoalBuilder::new()
        .statement(Statement::SemanticEquivalence(expr1, expr2))
        .goal_type(GoalType::Equivalence)
        .id(id.unwrap_or_else(|| "semantic_equiv".to_string()))
        .build()
}

/// Create a basic proof goal for reduction correctness
pub fn create_reduction_correctness_goal(
    expr: Expr,
    combinator_expr: CombinatorExpr,
    id: Option<String>,
) -> Result<ProofGoal> {
    ProofGoalBuilder::new()
        .statement(Statement::ReductionCorrectness(expr.clone(), combinator_expr))
        .goal_type(GoalType::Correctness)
        .expression(expr)
        .id(id.unwrap_or_else(|| "reduction_correctness".to_string()))
        .build()
}

/// Create a basic proof goal for R7RS compliance
pub fn create_r7rs_compliance_goal(
    expr: Expr,
    id: Option<String>,
) -> Result<ProofGoal> {
    ProofGoalBuilder::new()
        .statement(Statement::R7RSCompliance(expr.clone()))
        .goal_type(GoalType::R7RSCompliance)
        .expression(expr)
        .id(id.unwrap_or_else(|| "r7rs_compliance".to_string()))
        .build()
}

/// Utility function to convert expression to combinator (placeholder)
fn convert_expr_to_combinator(expr: &Expr) -> Result<CombinatorExpr> {
    use crate::evaluator::combinators::BracketAbstraction;
    BracketAbstraction::lambda_to_combinators(expr)
}

/// Validate a proof state for consistency
pub fn validate_proof_state(state: &ProofState) -> Result<()> {
    ProofStateValidator::validate(state)
}

/// Check dependencies in a proof state
pub fn check_proof_dependencies(state: &ProofState) -> Result<Vec<String>> {
    ProofStateValidator::check_dependencies(state)
}

/// Create a hypothesis from a proven statement
pub fn create_hypothesis_from_statement(
    name: String,
    statement: Statement,
    proof: Option<ProofTerm>,
) -> Result<Hypothesis> {
    HypothesisBuilder::new()
        .name(name)
        .statement(statement)
        .proof(proof.unwrap_or_else(|| ProofTerm::new_simple(
            ProofMethod::Custom("assumed".to_string()),
            "Assumed as hypothesis".to_string(),
            Statement::Axiom("hypothesis".to_string()),
        )))
        .build()
}

/// Get database statistics
pub fn get_database_statistics(database: &TheoremDatabase) -> DatabaseStatistics {
    database.get_statistics()
}

/// Export database summary
pub fn export_database_summary(database: &TheoremDatabase) -> DatabaseSummary {
    database.export_summary()
}

/// Validate theorem database for consistency
pub fn validate_theorem_database(database: &TheoremDatabase) -> Vec<String> {
    database.validate_all_theorems()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_create_theorem_proving_support() {
        let semantic_evaluator = SemanticEvaluator::new();
        let support = create_theorem_proving_support(semantic_evaluator);
        
        // Test that the support system is created with empty proof state
        assert!(support.proof_state().is_complete());
        assert_eq!(support.proof_state().goal_count(), 0);
    }

    #[test]
    fn test_create_semantic_equivalence_goal() {
        let expr1 = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
        let expr2 = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
        
        let goal = create_semantic_equivalence_goal(expr1, expr2, Some("test_goal".to_string())).unwrap();
        
        assert_eq!(goal.id, "test_goal");
        assert_eq!(goal.goal_type, GoalType::Equivalence);
        match goal.statement {
            Statement::SemanticEquivalence(_, _) => (),
            _ => panic!("Expected SemanticEquivalence statement"),
        }
    }

    #[test]
    fn test_create_r7rs_compliance_goal() {
        let expr = Expr::Variable("x".to_string());
        
        let goal = create_r7rs_compliance_goal(expr, None).unwrap();
        
        assert_eq!(goal.id, "r7rs_compliance");
        assert_eq!(goal.goal_type, GoalType::R7RSCompliance);
        match goal.statement {
            Statement::R7RSCompliance(_) => (),
            _ => panic!("Expected R7RSCompliance statement"),
        }
    }

    #[test]
    fn test_database_statistics() {
        let database = TheoremDatabase::new();
        let stats = get_database_statistics(&database);
        
        // Check that fundamental theorems are loaded
        assert!(stats.combinator_theorem_count > 0);
        assert!(stats.r7rs_rule_count > 0);
        assert!(stats.proven_theorem_count > 0);
    }

    #[test]
    fn test_validate_empty_proof_state() {
        let state = ProofState::new();
        assert!(validate_proof_state(&state).is_ok());
    }

    #[test]
    fn test_check_empty_proof_dependencies() {
        let state = ProofState::new();
        let deps = check_proof_dependencies(&state).unwrap();
        assert!(deps.is_empty());
    }
}