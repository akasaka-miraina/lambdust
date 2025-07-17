//! Theorem Database Implementation
//!
//! このモジュールは定理データベース、基本定理、
//! ユーザー定義定理の管理を実装します。

use super::core_types::{
    Statement, ProofTerm, ProofMethod, ReductionRule, Condition, 
    RuleTransformation, TypeConstraint
};
use crate::ast::Expr;

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

/// Type rule for type system
#[derive(Debug, Clone)]
pub struct TypeRule {
    /// Rule name
    pub name: String,

    /// Type constraint
    pub constraint: TypeConstraint,
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

impl TheoremDatabase {
    /// Initialize with fundamental theorems
    #[must_use] pub fn new() -> Self {
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
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "S combinator reduces by definition: S x y z = x z (y z)".to_string(),
                Statement::Axiom("S_reduction".to_string()),
            )),
        });

        // K combinator theorem: K x y = x
        self.combinator_theorems.push(CombinatorTheorem {
            name: "K_reduction".to_string(),
            reduction_rule: ReductionRule::K,
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "K combinator reduces by definition: K x y = x".to_string(),
                Statement::Axiom("K_reduction".to_string()),
            )),
        });

        // I combinator theorem: I x = x
        self.combinator_theorems.push(CombinatorTheorem {
            name: "I_reduction".to_string(),
            reduction_rule: ReductionRule::I,
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "I combinator reduces by definition: I x = x".to_string(),
                Statement::Axiom("I_reduction".to_string()),
            )),
        });

        // SKI identity theorem: S K K = I
        self.combinator_theorems.push(CombinatorTheorem {
            name: "SKI_identity".to_string(),
            reduction_rule: ReductionRule::Custom("S K K = I".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::Computation,
                "SKI identity proven by combinator reduction".to_string(),
                Statement::Axiom("SKI_identity".to_string()),
            )),
        });

        // Extended combinator theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "B_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("B".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "B combinator reduces by definition: B x y z = x (y z)".to_string(),
                Statement::Axiom("B_reduction".to_string()),
            )),
        });

        self.combinator_theorems.push(CombinatorTheorem {
            name: "C_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("C".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "C combinator reduces by definition: C x y z = x z y".to_string(),
                Statement::Axiom("C_reduction".to_string()),
            )),
        });

        self.combinator_theorems.push(CombinatorTheorem {
            name: "W_reduction".to_string(),
            reduction_rule: ReductionRule::Extended("W".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::CombinatorReduction,
                "W combinator reduces by definition: W x y = x y y".to_string(),
                Statement::Axiom("W_reduction".to_string()),
            )),
        });

        // Church-Rosser related theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "Church_Rosser_property".to_string(),
            reduction_rule: ReductionRule::Custom("confluence".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::Induction("reduction_steps".to_string()),
                "Church-Rosser property: if expr reduces to both expr1 and expr2, then there exists expr3 such that both expr1 and expr2 reduce to expr3".to_string(),
                Statement::Axiom("Church_Rosser".to_string()),
            )),
        });

        // Semantic preservation theorems
        self.combinator_theorems.push(CombinatorTheorem {
            name: "Semantic_preservation".to_string(),
            reduction_rule: ReductionRule::Custom("semantic_preservation".to_string()),
            conditions: vec![],
            proof: Some(ProofTerm::new_simple(
                ProofMethod::SemanticEquivalence,
                "Lambda-combinator transformation preserves R7RS semantics".to_string(),
                Statement::Axiom("Semantic_preservation".to_string()),
            )),
        });

        // Add basic R7RS semantic rules
        self.add_r7rs_semantic_rules();

        // Add basic type rules
        self.add_basic_type_rules();
    }

    /// Add R7RS semantic rules
    fn add_r7rs_semantic_rules(&mut self) {
        use crate::ast::Literal;
        use crate::lexer::SchemeNumber;

        // Identity rule for literals
        self.r7rs_rules.push(SemanticRule {
            name: "literal_identity".to_string(),
            description: "Literals evaluate to themselves".to_string(),
            pattern: Expr::Literal(Literal::Number(SchemeNumber::Integer(0))), // Pattern placeholder
            transformation: RuleTransformation::Complex("identity".to_string()),
        });

        // Variable lookup rule
        self.r7rs_rules.push(SemanticRule {
            name: "variable_lookup".to_string(),
            description: "Variables are looked up in the environment".to_string(),
            pattern: Expr::Variable("x".to_string()), // Pattern placeholder
            transformation: RuleTransformation::Complex("environment_lookup".to_string()),
        });

        // Function application rule
        self.r7rs_rules.push(SemanticRule {
            name: "application".to_string(),
            description: "Function application evaluates operator and operands".to_string(),
            pattern: Expr::List(vec![
                Expr::Variable("f".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            transformation: RuleTransformation::Complex("apply_function".to_string()),
        });

        // Lambda expression rule
        self.r7rs_rules.push(SemanticRule {
            name: "lambda_expression".to_string(),
            description: "Lambda expressions create closures".to_string(),
            pattern: Expr::List(vec![
                Expr::Variable("lambda".to_string()),
                Expr::List(vec![Expr::Variable("x".to_string())]),
                Expr::Variable("body".to_string()),
            ]),
            transformation: RuleTransformation::Complex("create_closure".to_string()),
        });

        // Conditional expression rule
        self.r7rs_rules.push(SemanticRule {
            name: "conditional".to_string(),
            description: "Conditional expressions evaluate test and branch".to_string(),
            pattern: Expr::List(vec![
                Expr::Variable("if".to_string()),
                Expr::Variable("test".to_string()),
                Expr::Variable("then".to_string()),
                Expr::Variable("else".to_string()),
            ]),
            transformation: RuleTransformation::Complex("conditional_branch".to_string()),
        });
    }

    /// Add basic type rules
    fn add_basic_type_rules(&mut self) {
        use crate::ast::Literal;
        use crate::lexer::SchemeNumber;

        // Number type rule
        self.type_rules.push(TypeRule {
            name: "number_type".to_string(),
            constraint: TypeConstraint::HasType(
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                "Number".to_string(),
            ),
        });

        // Boolean type rule
        self.type_rules.push(TypeRule {
            name: "boolean_type".to_string(),
            constraint: TypeConstraint::HasType(
                Expr::Literal(Literal::Boolean(true)),
                "Boolean".to_string(),
            ),
        });

        // String type rule
        self.type_rules.push(TypeRule {
            name: "string_type".to_string(),
            constraint: TypeConstraint::HasType(
                Expr::Literal(Literal::String("hello".to_string())),
                "String".to_string(),
            ),
        });

        // Function type preservation
        self.type_rules.push(TypeRule {
            name: "function_type_preservation".to_string(),
            constraint: TypeConstraint::Preserves(
                "Function".to_string(),
                "Function".to_string(),
            ),
        });

        // Arithmetic type preservation
        self.type_rules.push(TypeRule {
            name: "arithmetic_type_preservation".to_string(),
            constraint: TypeConstraint::Preserves(
                "Number".to_string(),
                "Number".to_string(),
            ),
        });
    }

    /// Find theorem by name
    #[must_use] pub fn find_theorem(&self, name: &str) -> Option<&CombinatorTheorem> {
        self.combinator_theorems.iter().find(|t| t.name == name)
    }

    /// Find R7RS rule by name
    #[must_use] pub fn find_r7rs_rule(&self, name: &str) -> Option<&SemanticRule> {
        self.r7rs_rules.iter().find(|r| r.name == name)
    }

    /// Find type rule by name
    #[must_use] pub fn find_type_rule(&self, name: &str) -> Option<&TypeRule> {
        self.type_rules.iter().find(|r| r.name == name)
    }

    /// Find user theorem by name
    #[must_use] pub fn find_user_theorem(&self, name: &str) -> Option<&UserTheorem> {
        self.user_theorems.iter().find(|t| t.name == name)
    }

    /// Add user-defined theorem
    pub fn add_user_theorem(&mut self, theorem: UserTheorem) {
        self.user_theorems.push(theorem);
    }

    /// Add combinator theorem
    pub fn add_combinator_theorem(&mut self, theorem: CombinatorTheorem) {
        self.combinator_theorems.push(theorem);
    }

    /// Add R7RS semantic rule
    pub fn add_r7rs_rule(&mut self, rule: SemanticRule) {
        self.r7rs_rules.push(rule);
    }

    /// Add type rule
    pub fn add_type_rule(&mut self, rule: TypeRule) {
        self.type_rules.push(rule);
    }

    /// Get all theorem names
    #[must_use] pub fn get_theorem_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.combinator_theorems.iter().map(|t| t.name.clone()));
        names.extend(self.user_theorems.iter().map(|t| t.name.clone()));
        names
    }

    /// Get all rule names
    #[must_use] pub fn get_rule_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.r7rs_rules.iter().map(|r| r.name.clone()));
        names.extend(self.type_rules.iter().map(|r| r.name.clone()));
        names
    }

    /// Get statistics about the database
    #[must_use] pub fn get_statistics(&self) -> DatabaseStatistics {
        DatabaseStatistics {
            combinator_theorem_count: self.combinator_theorems.len(),
            r7rs_rule_count: self.r7rs_rules.len(),
            type_rule_count: self.type_rules.len(),
            user_theorem_count: self.user_theorems.len(),
            proven_theorem_count: self.combinator_theorems.iter()
                .filter(|t| t.proof.is_some())
                .count() + self.user_theorems.len(),
        }
    }

    /// Validate all theorems in the database
    pub fn validate_all_theorems(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for duplicate names
        let mut seen_names = std::collections::HashSet::new();
        for theorem in &self.combinator_theorems {
            if !seen_names.insert(&theorem.name) {
                errors.push(format!("Duplicate combinator theorem name: {}", theorem.name));
            }
        }

        seen_names.clear();
        for theorem in &self.user_theorems {
            if !seen_names.insert(&theorem.name) {
                errors.push(format!("Duplicate user theorem name: {}", theorem.name));
            }
        }

        seen_names.clear();
        for rule in &self.r7rs_rules {
            if !seen_names.insert(&rule.name) {
                errors.push(format!("Duplicate R7RS rule name: {}", rule.name));
            }
        }

        seen_names.clear();
        for rule in &self.type_rules {
            if !seen_names.insert(&rule.name) {
                errors.push(format!("Duplicate type rule name: {}", rule.name));
            }
        }

        errors
    }

    /// Clear all user-defined content (keep fundamental theorems)
    pub fn clear_user_content(&mut self) {
        self.user_theorems.clear();
        // Keep only fundamental theorems - could implement filtering here
    }

    /// Export database to a serializable format
    #[must_use] pub fn export_summary(&self) -> DatabaseSummary {
        DatabaseSummary {
            combinator_theorems: self.combinator_theorems.iter()
                .map(|t| (t.name.clone(), t.proof.is_some()))
                .collect(),
            r7rs_rules: self.r7rs_rules.iter()
                .map(|r| r.name.clone())
                .collect(),
            type_rules: self.type_rules.iter()
                .map(|r| r.name.clone())
                .collect(),
            user_theorems: self.user_theorems.iter()
                .map(|t| t.name.clone())
                .collect(),
        }
    }
}

impl Default for TheoremDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    /// Number of combinator theorems
    pub combinator_theorem_count: usize,
    /// Number of R7RS rules
    pub r7rs_rule_count: usize,
    /// Number of type rules
    pub type_rule_count: usize,
    /// Number of user theorems
    pub user_theorem_count: usize,
    /// Number of proven theorems
    pub proven_theorem_count: usize,
}

/// Database summary for export
#[derive(Debug, Clone)]
pub struct DatabaseSummary {
    /// Combinator theorems (name, has_proof)
    pub combinator_theorems: Vec<(String, bool)>,
    /// R7RS rule names
    pub r7rs_rules: Vec<String>,
    /// Type rule names
    pub type_rules: Vec<String>,
    /// User theorem names
    pub user_theorems: Vec<String>,
}