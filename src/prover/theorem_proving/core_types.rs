//! Core Types for Theorem Proving System
//!
//! このモジュールは定理証明システムの基本的な型定義と
//! データ構造を定義します。

use crate::ast::Expr;
use crate::evaluator::combinators::CombinatorExpr;
use std::collections::HashMap;

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

    // Church-Rosser related statements
    /// Church-Rosser property holds
    ChurchRosserProperty(CombinatorExpr),
    
    /// Diamond property holds
    DiamondProperty(CombinatorExpr),
    
    /// Local confluence holds
    LocalConfluence(CombinatorExpr),
    
    /// Global confluence holds
    GlobalConfluence(CombinatorExpr),
    
    /// Well-typed expression
    WellTyped(CombinatorExpr),
    
    /// Valid combinator expression
    ValidCombinatorExpression(CombinatorExpr),
    
    /// Well-founded ordering exists
    WellFoundedOrdering(CombinatorExpr),
    
    /// Measure decreases in reduction
    MeasureDecrease(CombinatorExpr),
    
    /// No infinite reductions exist
    NoInfiniteReductions(CombinatorExpr),
    
    /// Normalization exists
    NormalizationExists(CombinatorExpr),
    
    /// Unique normal form exists
    UniqueNormalForm(CombinatorExpr),
    
    /// Normalization algorithm exists
    NormalizationAlgorithm(CombinatorExpr),
    
    /// Church-Rosser theorem components
    ChurchRosserComponents,
    
    /// Complete Church-Rosser theorem
    ChurchRosserTheorem,
    
    /// Axiom statement
    Axiom(String),
}

/// Goal types for categorization
#[derive(Debug, Clone, PartialEq)]
pub enum GoalType {
    /// Correctness of evaluation results
    Correctness,
    /// Equivalence between different implementations
    Equivalence,
    /// Termination properties of evaluation
    Termination,
    /// Type safety guarantees
    TypeSafety,
    /// Compliance with R7RS standard
    R7RSCompliance,
    /// Custom user-defined goal type
    Custom,
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

    // Additional fields for Church-Rosser proofs
    /// Term type for categorization
    pub term_type: ProofTermType,

    /// Proof steps
    pub proof_steps: Vec<ProofStep>,

    /// Lemmas used in this proof
    pub lemmas_used: Vec<Statement>,

    /// Tactics used in this proof
    pub tactics_used: Vec<String>,

    /// Conclusion of this proof
    pub conclusion: Statement,
}

impl ProofTerm {
    /// Create a new simple proof term with default values
    #[must_use] pub fn new_simple(
        method: ProofMethod,
        explanation: String,
        conclusion: Statement,
    ) -> Self {
        Self {
            method,
            subproofs: vec![],
            explanation,
            term_type: ProofTermType::TheoremProof,
            proof_steps: vec![],
            lemmas_used: vec![],
            tactics_used: vec![],
            conclusion,
        }
    }
}

/// Types of proof terms
#[derive(Debug, Clone, PartialEq)]
pub enum ProofTermType {
    /// Confluence proof
    ConfluenceProof,
    
    /// Termination proof
    TerminationProof,
    
    /// Normalization proof
    NormalizationProof,
    
    /// Complete Church-Rosser proof
    ChurchRosserProof,
    
    /// General theorem proof
    TheoremProof,
}

/// A single step in a proof
#[derive(Debug, Clone)]
pub struct ProofStep {
    /// Step number in the proof
    pub step_number: usize,
    
    /// Tactic applied in this step
    pub tactic_applied: String,
    
    /// Goal before applying the tactic
    pub goal_before: Statement,
    
    /// Goal after applying the tactic
    pub goal_after: Statement,
    
    /// Justification for this step
    pub justification: String,
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

impl ProofContext {
    /// Create new proof context
    #[must_use] pub fn new() -> Self {
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

impl Default for ProofContext {
    fn default() -> Self {
        Self::new()
    }
}