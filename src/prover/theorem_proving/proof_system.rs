//! Proof System Implementation
//!
//! このモジュールは証明状態管理、証明ゴール処理、
//! 証明コンテキスト操作を実装します。

use super::core_types::{
    ProofGoal, Hypothesis, ProofContext, Statement, ProofTerm
};
use crate::ast::Expr;
use crate::error::Result;

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

impl ProofState {
    /// Create new proof state
    #[must_use] pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            hypotheses: Vec::new(),
            context: ProofContext::new(),
        }
    }

    /// Check if all goals are proven
    #[must_use] pub fn is_complete(&self) -> bool {
        self.goals.is_empty()
    }

    /// Get current goal
    #[must_use] pub fn current_goal(&self) -> Option<&ProofGoal> {
        self.goals.first()
    }

    /// Remove current goal
    pub fn remove_current_goal(&mut self) -> Option<ProofGoal> {
        if self.goals.is_empty() {
            None
        } else {
            Some(self.goals.remove(0))
        }
    }

    /// Add a new goal to the proof state
    pub fn add_goal(&mut self, goal: ProofGoal) {
        self.goals.push(goal);
    }

    /// Add hypothesis to the proof state
    pub fn add_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.hypotheses.push(hypothesis);
    }

    /// Get all hypotheses
    #[must_use] pub fn get_hypotheses(&self) -> &Vec<Hypothesis> {
        &self.hypotheses
    }

    /// Find hypothesis by name
    #[must_use] pub fn find_hypothesis(&self, name: &str) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.name == name)
    }

    /// Get mutable reference to context
    pub fn context_mut(&mut self) -> &mut ProofContext {
        &mut self.context
    }

    /// Get reference to context
    #[must_use] pub fn context(&self) -> &ProofContext {
        &self.context
    }

    /// Clear all goals and hypotheses
    pub fn clear(&mut self) {
        self.goals.clear();
        self.hypotheses.clear();
        self.context = ProofContext::new();
    }

    /// Get number of remaining goals
    #[must_use] pub fn goal_count(&self) -> usize {
        self.goals.len()
    }

    /// Get goal by index
    #[must_use] pub fn get_goal(&self, index: usize) -> Option<&ProofGoal> {
        self.goals.get(index)
    }

    /// Remove goal by index
    pub fn remove_goal(&mut self, index: usize) -> Option<ProofGoal> {
        if index < self.goals.len() {
            Some(self.goals.remove(index))
        } else {
            None
        }
    }

    /// Replace current goal with multiple subgoals
    pub fn replace_current_goal_with_subgoals(&mut self, subgoals: Vec<ProofGoal>) {
        if !self.goals.is_empty() {
            self.goals.remove(0);
            for (i, goal) in subgoals.into_iter().enumerate() {
                self.goals.insert(i, goal);
            }
        }
    }

    /// Check if a statement is already proven (appears in hypotheses)
    #[must_use] pub fn is_proven(&self, statement: &Statement) -> bool {
        self.hypotheses.iter().any(|h| h.statement == *statement)
    }

    /// Get all proven statements
    #[must_use] pub fn proven_statements(&self) -> Vec<&Statement> {
        self.hypotheses.iter().map(|h| &h.statement).collect()
    }
}

impl Default for ProofState {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof goal builder for convenient goal construction
#[derive(Debug)]
pub struct ProofGoalBuilder {
    statement: Option<Statement>,
    goal_type: Option<super::core_types::GoalType>,
    expressions: Vec<Expr>,
    id: Option<String>,
}

impl ProofGoalBuilder {
    /// Create new goal builder
    #[must_use] pub fn new() -> Self {
        Self {
            statement: None,
            goal_type: None,
            expressions: Vec::new(),
            id: None,
        }
    }

    /// Set the statement to prove
    pub fn statement(mut self, statement: Statement) -> Self {
        self.statement = Some(statement);
        self
    }

    /// Set the goal type
    pub fn goal_type(mut self, goal_type: super::core_types::GoalType) -> Self {
        self.goal_type = Some(goal_type);
        self
    }

    /// Add an expression
    pub fn expression(mut self, expr: Expr) -> Self {
        self.expressions.push(expr);
        self
    }

    /// Add multiple expressions
    pub fn expressions(mut self, exprs: Vec<Expr>) -> Self {
        self.expressions.extend(exprs);
        self
    }

    /// Set the goal ID
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Build the proof goal
    pub fn build(self) -> Result<ProofGoal> {
        let statement = self.statement.ok_or_else(|| {
            crate::error::LambdustError::runtime_error("Statement is required for proof goal".to_string())
        })?;

        let goal_type = self.goal_type.unwrap_or(super::core_types::GoalType::Custom);
        let id = self.id.unwrap_or_else(|| format!("goal_{}", uuid::Uuid::new_v4()));

        Ok(ProofGoal {
            statement,
            goal_type,
            expressions: self.expressions,
            id,
        })
    }
}

impl Default for ProofGoalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Hypothesis builder for convenient hypothesis construction
#[derive(Debug)]
pub struct HypothesisBuilder {
    name: Option<String>,
    statement: Option<Statement>,
    proof: Option<ProofTerm>,
}

impl HypothesisBuilder {
    /// Create new hypothesis builder
    #[must_use] pub fn new() -> Self {
        Self {
            name: None,
            statement: None,
            proof: None,
        }
    }

    /// Set the hypothesis name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the hypothesis statement
    pub fn statement(mut self, statement: Statement) -> Self {
        self.statement = Some(statement);
        self
    }

    /// Set the hypothesis proof
    pub fn proof(mut self, proof: ProofTerm) -> Self {
        self.proof = Some(proof);
        self
    }

    /// Build the hypothesis
    pub fn build(self) -> Result<Hypothesis> {
        let name = self.name.ok_or_else(|| {
            crate::error::LambdustError::runtime_error("Name is required for hypothesis".to_string())
        })?;

        let statement = self.statement.ok_or_else(|| {
            crate::error::LambdustError::runtime_error("Statement is required for hypothesis".to_string())
        })?;

        Ok(Hypothesis {
            name,
            statement,
            proof: self.proof,
        })
    }
}

impl Default for HypothesisBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof state validator
pub struct ProofStateValidator;

impl ProofStateValidator {
    /// Validate that a proof state is consistent
    pub fn validate(state: &ProofState) -> Result<()> {
        // Check that all goal IDs are unique
        let mut seen_ids = std::collections::HashSet::new();
        for goal in &state.goals {
            if !seen_ids.insert(&goal.id) {
                return Err(crate::error::LambdustError::runtime_error(
                    format!("Duplicate goal ID: {}", goal.id)
                ));
            }
        }

        // Check that all hypothesis names are unique
        let mut seen_names = std::collections::HashSet::new();
        for hypothesis in &state.hypotheses {
            if !seen_names.insert(&hypothesis.name) {
                return Err(crate::error::LambdustError::runtime_error(
                    format!("Duplicate hypothesis name: {}", hypothesis.name)
                ));
            }
        }

        // Check context depth is valid
        if state.context.depth > 100 {
            return Err(crate::error::LambdustError::runtime_error(
                "Context depth too deep (>100)".to_string()
            ));
        }

        Ok(())
    }

    /// Check if all dependencies for goals are satisfied
    pub fn check_dependencies(state: &ProofState) -> Result<Vec<String>> {
        let mut missing_deps = Vec::new();

        for goal in &state.goals {
            // Check if goal references variables that exist in context
            match &goal.statement {
                Statement::SemanticEquivalence(expr1, expr2) => {
                    Self::check_expression_variables(expr1, &state.context, &mut missing_deps);
                    Self::check_expression_variables(expr2, &state.context, &mut missing_deps);
                }
                Statement::R7RSCompliance(expr) => {
                    Self::check_expression_variables(expr, &state.context, &mut missing_deps);
                }
                Statement::TypePreservation(expr, _) => {
                    Self::check_expression_variables(expr, &state.context, &mut missing_deps);
                }
                Statement::Custom(_, exprs) => {
                    for expr in exprs {
                        Self::check_expression_variables(expr, &state.context, &mut missing_deps);
                    }
                }
                _ => {} // Other statement types don't reference expressions directly
            }
        }

        Ok(missing_deps)
    }

    fn check_expression_variables(
        expr: &Expr,
        context: &ProofContext,
        missing_deps: &mut Vec<String>
    ) {
        match expr {
            Expr::Variable(name) => {
                if !context.variables.contains_key(name) {
                    missing_deps.push(format!("Variable '{}' not in context", name));
                }
            }
            Expr::List(exprs) => {
                for e in exprs {
                    Self::check_expression_variables(e, context, missing_deps);
                }
            }
            Expr::Vector(exprs) => {
                for e in exprs {
                    Self::check_expression_variables(e, context, missing_deps);
                }
            }
            Expr::Quote(e) => {
                Self::check_expression_variables(e, context, missing_deps);
            }
            _ => {} // Literals don't reference variables
        }
    }
}

// UUID placeholder - in real code this would use a proper UUID crate
mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    #[derive(Debug)]
    pub struct Uuid(String);
    
    impl Uuid {
        pub fn new_v4() -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            
            Self(format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", 
                        timestamp & 0xFFFFFFFF,
                        (timestamp >> 32) as u16,
                        (timestamp >> 48) as u16,
                        0x4000_u16, // Version 4 UUID marker
                        timestamp & 0xFFFF_FFFF_FFFF))
        }
    }
    
    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}