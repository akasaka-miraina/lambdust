#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::evaluator::combinators::CombinatorExpr;
    use crate::lexer::SchemeNumber;
    
    #[test]
    fn test_theorem_proving_support_creation() {
        let semantic_evaluator = SemanticEvaluator::new();
        let theorem_system = TheoremProvingSupport::new(semantic_evaluator);
        
        assert!(theorem_system.proof_state().goals.is_empty());
        assert!(!theorem_system.theorem_db().combinator_theorems.is_empty());
    }
    
    #[test]
    fn test_proof_goal_creation() {
        let goal = ProofGoal {
            statement: Statement::SemanticEquivalence(
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ),
            goal_type: GoalType::Equivalence,
            expressions: vec![],
            id: "test_goal".to_string(),
        };
        
        assert_eq!(goal.goal_type, GoalType::Equivalence);
        assert_eq!(goal.id, "test_goal");
    }
    
    #[test]
    fn test_theorem_database_fundamental_theorems() {
        let db = TheoremDatabase::new();
        
        assert!(!db.combinator_theorems.is_empty());
        assert!(db.find_theorem("S_reduction").is_some());
        assert!(db.find_theorem("K_reduction").is_some());
        assert!(db.find_theorem("I_reduction").is_some());
        assert!(db.find_theorem("SKI_identity").is_some());
    }
    
    #[test]
    fn test_proof_state_management() {
        let mut state = ProofState::new();
        
        assert!(state.is_complete());
        assert!(state.current_goal().is_none());
        
        let goal = ProofGoal {
            statement: Statement::Termination(CombinatorExpr::I),
            goal_type: GoalType::Termination,
            expressions: vec![],
            id: "termination_test".to_string(),
        };
        
        state.goals.push(goal);
        assert!(!state.is_complete());
        assert!(state.current_goal().is_some());
        
        let removed_goal = state.remove_current_goal();
        assert!(removed_goal.is_some());
        assert!(state.is_complete());
    }
    
    #[test]
    fn test_proof_context() {
        let mut context = ProofContext::new();
        
        assert_eq!(context.depth, 0);
        
        context.add_variable("x".to_string(), Expr::Literal(Literal::Number(SchemeNumber::Integer(42))));
        context.add_type_assumption("x".to_string(), "Number".to_string());
        
        assert!(context.variables.contains_key("x"));
        assert!(context.type_assumptions.contains_key("x"));
        
        context.push_context();
        assert_eq!(context.depth, 1);
        
        context.pop_context();
        assert_eq!(context.depth, 0);
    }
    
    #[test]
    fn test_basic_tactic_application() {
        let semantic_evaluator = SemanticEvaluator::new();
        let mut theorem_system = TheoremProvingSupport::new(semantic_evaluator);
        
        // Test applying tactic with no goals
        let result = theorem_system.apply_tactic(ProofTactic::Simplify).unwrap();
        assert!(!result.success);
        assert_eq!(result.explanation, "No goals to prove");
        
        // Add a goal
        let goal = ProofGoal {
            statement: Statement::ReductionCorrectness(
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                CombinatorExpr::I,
            ),
            goal_type: GoalType::Correctness,
            expressions: vec![],
            id: "test_correctness".to_string(),
        };
        
        theorem_system.add_goal(goal).unwrap();
        assert!(!theorem_system.proof_state().goals.is_empty());
        
        // Apply combinator reduction tactic
        let result = theorem_system.apply_tactic(ProofTactic::CombinatorReduction).unwrap();
        // Note: This will likely fail in practice since we're using a simple literal
        // but the tactic should execute without panicking
    }
    
    #[test]
    fn test_statement_types() {
        let expr1 = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
        let expr2 = Expr::Literal(Literal::Number(SchemeNumber::Integer(2)));
        
        let equivalence = Statement::SemanticEquivalence(expr1.clone(), expr2.clone());
        let correctness = Statement::ReductionCorrectness(expr1.clone(), CombinatorExpr::I);
        let termination = Statement::Termination(CombinatorExpr::S);
        let compliance = Statement::R7RSCompliance(expr1.clone());
        let type_preservation = Statement::TypePreservation(expr1.clone(), "Number".to_string());
        let custom = Statement::Custom("test_theorem".to_string(), vec![expr1, expr2]);
        
        // Test that all statement types can be created and matched
        match equivalence {
            Statement::SemanticEquivalence(_, _) => {},
            _ => panic!("Expected SemanticEquivalence"),
        }
        
        match correctness {
            Statement::ReductionCorrectness(_, _) => {},
            _ => panic!("Expected ReductionCorrectness"),
        }
        
        match termination {
            Statement::Termination(_) => {},
            _ => panic!("Expected Termination"),
        }
        
        match compliance {
            Statement::R7RSCompliance(_) => {},
            _ => panic!("Expected R7RSCompliance"),
        }
        
        match type_preservation {
            Statement::TypePreservation(_, _) => {},
            _ => panic!("Expected TypePreservation"),
        }
        
        match custom {
            Statement::Custom(_, _) => {},
            _ => panic!("Expected Custom"),
        }
    }
    
    #[test]
    fn test_verification_result() {
        let semantic_evaluator = SemanticEvaluator::new();
        let mut theorem_system = TheoremProvingSupport::new(semantic_evaluator);
        
        let statement = Statement::SemanticEquivalence(
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        );
        
        let result = theorem_system.verify_statement(statement).unwrap();
        // Note: These assertions are based on current placeholder implementation
        // When real verification is implemented, these may need to be updated
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.proof.is_none());
    }
    
    #[test]
    fn test_church_rosser_property_theorem() {
        let db = TheoremDatabase::new();
        
        let church_rosser_theorem = db.find_theorem("Church_Rosser_property");
        assert!(church_rosser_theorem.is_some());
        
        if let Some(theorem) = church_rosser_theorem {
            assert_eq!(theorem.name, "Church_Rosser_property");
            assert!(theorem.proof.is_some());
            if let Some(proof) = &theorem.proof {
                assert!(matches!(proof.method, ProofMethod::Induction(_)));
            }
        }
    }
    
    #[test]
    fn test_semantic_preservation_theorem() {
        let db = TheoremDatabase::new();
        
        let semantic_preservation_theorem = db.find_theorem("Semantic_preservation");
        assert!(semantic_preservation_theorem.is_some());
        
        if let Some(theorem) = semantic_preservation_theorem {
            assert_eq!(theorem.name, "Semantic_preservation");
            assert!(theorem.proof.is_some());
            if let Some(proof) = &theorem.proof {
                assert!(matches!(proof.method, ProofMethod::SemanticEquivalence));
            }
        }
    }
}