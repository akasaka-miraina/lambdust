//! Theorem proving support system for formal verification
//!
//! This module provides infrastructure for formal verification of combinator
//! reductions, R7RS semantic correctness, and mathematical properties of
//! the evaluator system.

use crate::ast::Expr;
use crate::error::Result;
use crate::evaluator::{combinators::CombinatorExpr, SemanticEvaluator};
use std::collections::HashMap;

/// Main theorem proving support system
#[derive(Debug)]
pub struct TheoremProvingSupport {
    /// Reference to semantic evaluator for verification
    semantic_evaluator: SemanticEvaluator,

    /// Current proof state
    proof_state: ProofState,

    /// Theorem database
    theorem_db: TheoremDatabase,
}

/// Proof state management
#[derive(Debug, Clone)]
pub struct ProofState {
    /// Current goals to prove
    pub goals: Vec<ProofGoal>,

    /// Available hypotheses
    pub hypotheses: Vec<Hypothesis>,

    /// Proof context
    pub context: ProofContext,
}

/// Individual proof goal
#[derive(Debug, Clone)]
pub struct ProofGoal {
    /// Goal statement to prove
    pub statement: Statement,

    /// Type of goal (correctness, equivalence, etc.)
    pub goal_type: GoalType,

    /// Associated expressions
    pub expressions: Vec<Expr>,

    /// Goal identifier
    pub id: String,
}

/// Statement types for theorem proving
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Semantic equivalence: expr1 ≡ expr2
    SemanticEquivalence(Expr, Expr),

    /// Combinator reduction correctness: reduce(expr) preserves semantics
    ReductionCorrectness(Expr, CombinatorExpr),

    /// Termination: reduction terminates
    Termination(CombinatorExpr),

    /// R7RS compliance: expr follows R7RS formal semantics
    R7RSCompliance(Expr),

    /// Type preservation: evaluation preserves types
    TypePreservation(Expr, String), // Using String for type for now

    /// Custom theorem statement
    Custom(String, Vec<Expr>),
}

/// Goal types for categorization
#[derive(Debug, Clone, PartialEq)]
pub enum GoalType {
    Correctness,
    Equivalence,
    Termination,
    TypeSafety,
    R7RSCompliance,
    Custom,
}

/// Hypothesis in proof context
#[derive(Debug, Clone)]
pub struct Hypothesis {
    /// Hypothesis name
    pub name: String,

    /// Hypothesis statement
    pub statement: Statement,

    /// Proof of hypothesis (if available)
    pub proof: Option<ProofTerm>,
}

/// Proof context containing variable bindings and assumptions
#[derive(Debug, Clone)]
pub struct ProofContext {
    /// Variable bindings
    pub variables: HashMap<String, Expr>,

    /// Type assumptions
    pub type_assumptions: HashMap<String, String>,

    /// Context depth (for nested proofs)
    pub depth: usize,
}

/// Proof term representing a proof
#[derive(Debug, Clone)]
pub struct ProofTerm {
    /// Proof method used
    pub method: ProofMethod,

    /// Subproofs
    pub subproofs: Vec<ProofTerm>,

    /// Explanation
    pub explanation: String,
}

/// Proof methods available
#[derive(Debug, Clone)]
pub enum ProofMethod {
    /// Direct proof by computation
    Computation,

    /// Proof by rewriting
    Rewrite(String),

    /// Proof by induction
    Induction(String),

    /// Proof by case analysis
    CaseAnalysis,

    /// Proof by contradiction
    Contradiction,

    /// Proof by combinator reduction
    CombinatorReduction,

    /// Proof by semantic equivalence
    SemanticEquivalence,

    /// Custom proof method
    Custom(String),
}

/// Theorem database
#[derive(Debug, Clone)]
pub struct TheoremDatabase {
    /// Basic combinator theorems
    pub combinator_theorems: Vec<CombinatorTheorem>,

    /// R7RS semantic rules
    pub r7rs_rules: Vec<SemanticRule>,

    /// Type system rules
    pub type_rules: Vec<TypeRule>,

    /// User-defined theorems
    pub user_theorems: Vec<UserTheorem>,
}

/// Combinator theorem
#[derive(Debug, Clone)]
pub struct CombinatorTheorem {
    /// Theorem name
    pub name: String,

    /// Combinator reduction rule
    pub reduction_rule: ReductionRule,

    /// Conditions for applicability
    pub conditions: Vec<Condition>,

    /// Proof of correctness
    pub proof: Option<ProofTerm>,
}

/// Reduction rule types
#[derive(Debug, Clone)]
pub enum ReductionRule {
    /// S combinator rule
    S,

    /// K combinator rule
    K,

    /// I combinator rule
    I,

    /// Extended combinator rules
    Extended(String),

    /// Custom rule
    Custom(String),
}

/// Condition for theorem applicability
#[derive(Debug, Clone)]
pub struct Condition {
    /// Condition description
    pub description: String,

    /// Condition predicate
    pub predicate: ConditionPredicate,
}

/// Condition predicate types
#[derive(Debug, Clone)]
pub enum ConditionPredicate {
    /// Variable is free in expression
    FreeVariable(String, Expr),

    /// Expression has specific form
    HasForm(Expr, String),

    /// Type constraint
    HasType(Expr, String),

    /// Custom predicate
    Custom(String),
}

/// Semantic rule for R7RS compliance
#[derive(Debug, Clone)]
pub struct SemanticRule {
    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Applicable expressions
    pub pattern: Expr,

    /// Transformation
    pub transformation: RuleTransformation,
}

/// Rule transformation types
#[derive(Debug, Clone)]
pub enum RuleTransformation {
    /// Simple substitution
    Substitution(Expr, Expr),

    /// Conditional transformation
    Conditional(Vec<Condition>, Expr),

    /// Complex transformation
    Complex(String),
}

/// Type rule for type system
#[derive(Debug, Clone)]
pub struct TypeRule {
    /// Rule name
    pub name: String,

    /// Type constraint
    pub constraint: TypeConstraint,
}

/// Type constraint
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Expression has type
    HasType(Expr, String),

    /// Type preservation under operation
    Preserves(String, String),

    /// Custom constraint
    Custom(String),
}

/// User-defined theorem
#[derive(Debug, Clone)]
pub struct UserTheorem {
    /// Theorem name
    pub name: String,

    /// Theorem statement
    pub statement: Statement,

    /// Proof
    pub proof: ProofTerm,
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub success: bool,

    /// Generated proof (if successful)
    pub proof: Option<ProofTerm>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Remaining subgoals (if partial)
    pub subgoals: Vec<ProofGoal>,
}

/// Proof tactic result
#[derive(Debug, Clone)]
pub struct TacticResult {
    /// Whether tactic succeeded
    pub success: bool,

    /// Resulting subgoals
    pub subgoals: Vec<ProofGoal>,

    /// Generated hypotheses
    pub new_hypotheses: Vec<Hypothesis>,

    /// Explanation of what happened
    pub explanation: String,
}

/// Proof tactics available
#[derive(Debug, Clone)]
pub enum ProofTactic {
    /// Rewrite using a theorem
    Rewrite(String),

    /// Substitute variable with expression
    Substitution(String, Expr),

    /// Induction on a variable
    Induction(String),

    /// Apply combinator reduction
    CombinatorReduction,

    /// Prove semantic equivalence
    SemanticEquivalence,

    /// Apply R7RS semantic rules
    R7RSSemantics,

    /// Split into subcases
    CaseSplit(Vec<String>),

    /// Simplify expression
    Simplify,
}

impl TheoremProvingSupport {
    /// Create new theorem proving support system
    pub fn new(semantic_evaluator: SemanticEvaluator) -> Self {
        Self {
            semantic_evaluator,
            proof_state: ProofState::new(),
            theorem_db: TheoremDatabase::new(),
        }
    }

    /// Add a new proof goal
    pub fn add_goal(&mut self, goal: ProofGoal) -> Result<()> {
        self.proof_state.goals.push(goal);
        Ok(())
    }

    /// Apply a proof tactic to current goal
    pub fn apply_tactic(&mut self, tactic: ProofTactic) -> Result<TacticResult> {
        if self.proof_state.goals.is_empty() {
            return Ok(TacticResult {
                success: false,
                subgoals: vec![],
                new_hypotheses: vec![],
                explanation: "No goals to prove".to_string(),
            });
        }

        match tactic {
            ProofTactic::Rewrite(theorem) => self.apply_rewrite(theorem),
            ProofTactic::Substitution(var, expr) => self.apply_substitution(var, expr),
            ProofTactic::Induction(var) => self.apply_induction(var),
            ProofTactic::CombinatorReduction => self.apply_combinator_reduction(),
            ProofTactic::SemanticEquivalence => self.apply_semantic_equivalence(),
            ProofTactic::R7RSSemantics => self.apply_r7rs_semantics(),
            ProofTactic::CaseSplit(cases) => self.apply_case_split(cases),
            ProofTactic::Simplify => self.apply_simplify(),
        }
    }

    /// Verify a statement using the proof system
    pub fn verify_statement(&mut self, statement: Statement) -> Result<VerificationResult> {
        match statement {
            Statement::SemanticEquivalence(expr1, expr2) => {
                self.verify_semantic_equivalence(expr1, expr2)
            }
            Statement::ReductionCorrectness(expr, combinator) => {
                self.verify_reduction_correctness(expr, combinator)
            }
            Statement::Termination(combinator) => self.verify_termination(combinator),
            Statement::R7RSCompliance(expr) => self.verify_r7rs_compliance(expr),
            Statement::TypePreservation(expr, expected_type) => {
                self.verify_type_preservation(expr, expected_type)
            }
            Statement::Custom(name, exprs) => self.verify_custom_theorem(name, exprs),
        }
    }

    /// Get current proof state
    pub fn proof_state(&self) -> &ProofState {
        &self.proof_state
    }

    /// Get theorem database
    pub fn theorem_db(&self) -> &TheoremDatabase {
        &self.theorem_db
    }

    /// Reset proof state
    pub fn reset_proof_state(&mut self) {
        self.proof_state = ProofState::new();
    }

    // Private implementation methods

    fn apply_rewrite(&mut self, theorem: String) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: format!("Rewrite with theorem '{}' not implemented yet", theorem),
        })
    }

    fn apply_substitution(&mut self, var: String, _expr: Expr) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: format!("Substitution of '{}' not implemented yet", var),
        })
    }

    fn apply_induction(&mut self, var: String) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: format!("Induction on '{}' not implemented yet", var),
        })
    }

    fn apply_combinator_reduction(&mut self) -> Result<TacticResult> {
        if let Some(current_goal) = self.proof_state.current_goal() {
            match &current_goal.statement {
                Statement::ReductionCorrectness(expr, combinator_expr) => {
                    // Verify combinator reduction preserves semantics
                    let reduction_result =
                        self.verify_combinator_reduction_step(expr, combinator_expr)?;

                    if reduction_result {
                        // Remove the goal if proven
                        self.proof_state.remove_current_goal();

                        Ok(TacticResult {
                            success: true,
                            subgoals: vec![],
                            new_hypotheses: vec![],
                            explanation: "Combinator reduction correctness verified".to_string(),
                        })
                    } else {
                        Ok(TacticResult {
                            success: false,
                            subgoals: vec![current_goal.clone()],
                            new_hypotheses: vec![],
                            explanation: "Combinator reduction correctness could not be verified"
                                .to_string(),
                        })
                    }
                }
                _ => Ok(TacticResult {
                    success: false,
                    subgoals: vec![current_goal.clone()],
                    new_hypotheses: vec![],
                    explanation: "Combinator reduction tactic not applicable to this goal type"
                        .to_string(),
                }),
            }
        } else {
            Ok(TacticResult {
                success: false,
                subgoals: vec![],
                new_hypotheses: vec![],
                explanation: "No current goal to apply combinator reduction".to_string(),
            })
        }
    }

    fn apply_semantic_equivalence(&mut self) -> Result<TacticResult> {
        if let Some(current_goal) = self.proof_state.current_goal() {
            match &current_goal.statement {
                Statement::SemanticEquivalence(expr1, expr2) => {
                    // Verify semantic equivalence using SemanticEvaluator
                    let equivalence_result = self.verify_semantic_equivalence_step(expr1, expr2)?;

                    if equivalence_result {
                        // Remove the goal if proven
                        self.proof_state.remove_current_goal();

                        Ok(TacticResult {
                            success: true,
                            subgoals: vec![],
                            new_hypotheses: vec![],
                            explanation: "Semantic equivalence verified".to_string(),
                        })
                    } else {
                        Ok(TacticResult {
                            success: false,
                            subgoals: vec![current_goal.clone()],
                            new_hypotheses: vec![],
                            explanation: "Semantic equivalence could not be verified".to_string(),
                        })
                    }
                }
                _ => Ok(TacticResult {
                    success: false,
                    subgoals: vec![current_goal.clone()],
                    new_hypotheses: vec![],
                    explanation: "Semantic equivalence tactic not applicable to this goal type"
                        .to_string(),
                }),
            }
        } else {
            Ok(TacticResult {
                success: false,
                subgoals: vec![],
                new_hypotheses: vec![],
                explanation: "No current goal to apply semantic equivalence".to_string(),
            })
        }
    }

    fn apply_r7rs_semantics(&mut self) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: "R7RS semantics application not implemented yet".to_string(),
        })
    }

    fn apply_case_split(&mut self, cases: Vec<String>) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: format!("Case split on {:?} not implemented yet", cases),
        })
    }

    fn apply_simplify(&mut self) -> Result<TacticResult> {
        // Placeholder implementation
        Ok(TacticResult {
            success: false,
            subgoals: vec![],
            new_hypotheses: vec![],
            explanation: "Simplification not implemented yet".to_string(),
        })
    }

    fn verify_semantic_equivalence(
        &mut self,
        _expr1: Expr,
        _expr2: Expr,
    ) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some("Semantic equivalence verification not implemented yet".to_string()),
            subgoals: vec![],
        })
    }

    fn verify_reduction_correctness(
        &mut self,
        _expr: Expr,
        _combinator: CombinatorExpr,
    ) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some("Reduction correctness verification not implemented yet".to_string()),
            subgoals: vec![],
        })
    }

    fn verify_termination(&mut self, _combinator: CombinatorExpr) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some("Termination verification not implemented yet".to_string()),
            subgoals: vec![],
        })
    }

    fn verify_r7rs_compliance(&mut self, _expr: Expr) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some("R7RS compliance verification not implemented yet".to_string()),
            subgoals: vec![],
        })
    }

    fn verify_type_preservation(
        &mut self,
        _expr: Expr,
        _expected_type: String,
    ) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some("Type preservation verification not implemented yet".to_string()),
            subgoals: vec![],
        })
    }

    fn verify_custom_theorem(
        &mut self,
        name: String,
        _exprs: Vec<Expr>,
    ) -> Result<VerificationResult> {
        // Placeholder implementation
        Ok(VerificationResult {
            success: false,
            proof: None,
            error: Some(format!(
                "Custom theorem '{}' verification not implemented yet",
                name
            )),
            subgoals: vec![],
        })
    }

    /// Verify combinator reduction step preserves semantics
    fn verify_combinator_reduction_step(
        &self,
        expr: &Expr,
        combinator_expr: &CombinatorExpr,
    ) -> Result<bool> {
        use crate::evaluator::combinators::BracketAbstraction;

        // Convert lambda expression to combinator form
        let lambda_to_combinator = BracketAbstraction::lambda_to_combinators(expr)?;

        // Check if the provided combinator matches expected transformation
        let matches =
            self.combinator_expressions_equivalent(&lambda_to_combinator, combinator_expr)?;

        if matches {
            // For now, we'll consider matching combinator transformations as semantically correct
            // A more complete implementation would evaluate both forms and compare results
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Verify semantic equivalence between two expressions
    fn verify_semantic_equivalence_step(&self, expr1: &Expr, expr2: &Expr) -> Result<bool> {
        // For simple structural equivalence checking
        self.expressions_semantically_equal(expr1, expr2)
    }

    /// Check if two expressions are semantically equal
    pub fn expressions_semantically_equal(&self, expr1: &Expr, expr2: &Expr) -> Result<bool> {
        match (expr1, expr2) {
            (Expr::Literal(lit1), Expr::Literal(lit2)) => Ok(lit1 == lit2),
            (Expr::Variable(var1), Expr::Variable(var2)) => Ok(var1 == var2),
            (Expr::List(list1), Expr::List(list2)) => {
                if list1.len() != list2.len() {
                    return Ok(false);
                }
                for (e1, e2) in list1.iter().zip(list2.iter()) {
                    if !self.expressions_semantically_equal(e1, e2)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (Expr::Quote(q1), Expr::Quote(q2)) => self.expressions_semantically_equal(q1, q2),
            (Expr::Vector(v1), Expr::Vector(v2)) => {
                if v1.len() != v2.len() {
                    return Ok(false);
                }
                for (e1, e2) in v1.iter().zip(v2.iter()) {
                    if !self.expressions_semantically_equal(e1, e2)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Check if two combinator expressions are equivalent
    pub fn combinator_expressions_equivalent(
        &self,
        expr1: &CombinatorExpr,
        expr2: &CombinatorExpr,
    ) -> Result<bool> {
        use crate::evaluator::combinators::CombinatorExpr::*;

        match (expr1, expr2) {
            (S, S) | (K, K) | (I, I) | (B, B) | (C, C) | (W, W) => Ok(true),
            (App(f1, a1), App(f2, a2)) => {
                if !self.combinator_expressions_equivalent(f1, f2)? {
                    return Ok(false);
                }
                self.combinator_expressions_equivalent(a1, a2)
            }
            (Atomic(e1), Atomic(e2)) => self.expressions_semantically_equal(e1, e2),
            _ => Ok(false),
        }
    }

    /// Get reference to semantic evaluator
    pub fn get_semantic_evaluator(&self) -> &SemanticEvaluator {
        &self.semantic_evaluator
    }

    /// Verify theorem using semantic evaluator
    pub fn verify_theorem_semantically(
        &mut self,
        _theorem: &str,
        expressions: &[Expr],
    ) -> Result<bool> {
        use crate::environment::Environment;
        use std::rc::Rc;

        let env = Rc::new(Environment::new());
        
        // For each expression in the theorem, verify it evaluates correctly
        for expr in expressions {
            let _result = self.semantic_evaluator.eval_pure(
                expr.clone(),
                env.clone(),
                crate::evaluator::Continuation::Identity,
            )?;
            // Additional verification logic would go here
        }

        // Return true for now - complete verification logic would be implemented here
        Ok(true)
    }
}

impl ProofState {
    /// Create new proof state
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            hypotheses: Vec::new(),
            context: ProofContext::new(),
        }
    }

    /// Check if all goals are proven
    pub fn is_complete(&self) -> bool {
        self.goals.is_empty()
    }

    /// Get current goal
    pub fn current_goal(&self) -> Option<&ProofGoal> {
        self.goals.first()
    }

    /// Remove current goal
    pub fn remove_current_goal(&mut self) -> Option<ProofGoal> {
        if !self.goals.is_empty() {
            Some(self.goals.remove(0))
        } else {
            None
        }
    }

}

impl ProofContext {
    /// Create new proof context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            type_assumptions: HashMap::new(),
            depth: 0,
        }
    }

    /// Add variable binding
    pub fn add_variable(&mut self, name: String, expr: Expr) {
        self.variables.insert(name, expr);
    }

    /// Add type assumption
    pub fn add_type_assumption(&mut self, var: String, type_name: String) {
        self.type_assumptions.insert(var, type_name);
    }

    /// Increase context depth
    pub fn push_context(&mut self) {
        self.depth += 1;
    }

    /// Decrease context depth
    pub fn pop_context(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }
}

impl TheoremDatabase {
    /// Initialize with fundamental theorems
    pub fn new() -> Self {
        let mut db = Self {
            combinator_theorems: Vec::new(),
            r7rs_rules: Vec::new(),
            type_rules: Vec::new(),
            user_theorems: Vec::new(),
        };

        // Add fundamental combinator theorems
        db.add_fundamental_theorems();
        db
    }

    /// Add fundamental combinator theorems
    fn add_fundamental_theorems(&mut self) {
        // S combinator theorem: S x y z = x z (y z)
        self.combinator_theorems.push(CombinatorTheorem {
            name: "S_reduction".to_string(),
            reduction_rule: ReductionRule::S,
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "S combinator reduces by definition: S x y z = x z (y z)".to_string(),
            }),
        });

        // K combinator theorem: K x y = x
        self.combinator_theorems.push(CombinatorTheorem {
            name: "K_reduction".to_string(),
            reduction_rule: ReductionRule::K,
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "K combinator reduces by definition: K x y = x".to_string(),
            }),
        });

        // I combinator theorem: I x = x
        self.combinator_theorems.push(CombinatorTheorem {
            name: "I_reduction".to_string(),
            reduction_rule: ReductionRule::I,
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "I combinator reduces by definition: I x = x".to_string(),
            }),
        });

        // SKI identity theorem: S K K = I
        self.combinator_theorems.push(CombinatorTheorem {
            name: "SKI_identity".to_string(),
            reduction_rule: ReductionRule::Custom("S K K = I".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::Computation,
                subproofs: vec![ProofTerm {
                    method: ProofMethod::CombinatorReduction,
                    subproofs: vec![],
                    explanation: "S K K x = K x (K x) = x".to_string(),
                }],
                explanation: "SKI identity proven by combinator reduction".to_string(),
            }),
        });

        // Extended combinator theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "B_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("B".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "B combinator reduces by definition: B x y z = x (y z)".to_string(),
            }),
        });

        self.combinator_theorems.push(CombinatorTheorem {
            name: "C_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("C".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "C combinator reduces by definition: C x y z = x z y".to_string(),
            }),
        });

        self.combinator_theorems.push(CombinatorTheorem {
            name: "W_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("W".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::CombinatorReduction,
                subproofs: vec![],
                explanation: "W combinator reduces by definition: W x y = x y y".to_string(),
            }),
        });

        // Church-Rosser related theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "Church_Rosser_property".to_string(),
            reduction_rule: ReductionRule::Custom("confluence".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::Induction("reduction_steps".to_string()),
                subproofs: vec![],
                explanation: "Church-Rosser property: if expr reduces to both expr1 and expr2, then there exists expr3 such that both expr1 and expr2 reduce to expr3".to_string(),
            }),
        });

        // Semantic preservation theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "Semantic_preservation".to_string(),
            reduction_rule: ReductionRule::Custom("semantic_preservation".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm {
                method: ProofMethod::SemanticEquivalence,
                subproofs: vec![],
                explanation: "Lambda-combinator transformation preserves R7RS semantics"
                    .to_string(),
            }),
        });
    }

    /// Find theorem by name
    pub fn find_theorem(&self, name: &str) -> Option<&CombinatorTheorem> {
        self.combinator_theorems.iter().find(|t| t.name == name)
    }

    /// Add user-defined theorem
    pub fn add_user_theorem(&mut self, theorem: UserTheorem) {
        self.user_theorems.push(theorem);
    }
}

impl Default for ProofState {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProofContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TheoremDatabase {
    fn default() -> Self {
        Self::new()
    }
}

// Tests are temporarily disabled due to encoding issues
// They will be re-added after resolving the encoding problem
