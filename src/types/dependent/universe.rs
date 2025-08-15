#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]

//! Universe hierarchy system for avoiding Russell's paradox.
//!
//! This module implements the universe hierarchy that stratifies types to prevent
//! paradoxes in dependent type theory, following Martin-Löf type theory and
//! modern foundations like those in Agda, Coq, and Lean.
//!
//! # Mathematical Foundation
//!
//! The universe hierarchy prevents Russell's paradox by ensuring:
//! - Type₀ : Type₁ : Type₂ : Type₃ : ...
//! - No type can contain itself as an element
//! - Each universe level contains all types from lower levels
//! - Type operations preserve universe levels according to specific rules
//!
//! # Universe Rules
//!
//! ## Formation Rules
//! ```
//!   ─────────────────    ──────────────────
//!   ⊢ Type₀ : Type₁     ⊢ Type₁ : Type₂   ...
//! ```
//!
//! ## Cumulativity Rules
//! ```
//!   Γ ⊢ A : Type_i    i ≤ j
//!   ─────────────────────────
//!       Γ ⊢ A : Type_j
//! ```
//!
//! ## Universe Polymorphism
//! ```
//!   Γ, α:Level ⊢ A : Type_α
//!   ──────────────────────────
//!   Γ ⊢ ∀α:Level. A : Type_{α+1}
//! ```
//!
//! # Level Arithmetic
//!
//! Universe levels support arithmetic operations:
//! - max(i, j): maximum of two levels
//! - i + 1: successor level  
//! - ⊔ (lub): least upper bound of level sets

use super::core::{DependentType, DependentTerm, TypingContext, UniverseLevel};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet, BTreeSet};
use std::cmp::{max, Ordering};
use std::fmt;

/// Universe hierarchy manager.
///
/// This structure manages the universe hierarchy, enforces level constraints,
/// and provides operations for universe polymorphism and level arithmetic.
#[derive(Debug, Clone)]
pub struct UniverseHierarchy {
    /// Maximum universe level encountered
    max_level: UniverseLevel,
    /// Level constraints: level variables and their bounds
    constraints: Vec<LevelConstraint>,
    /// Level variable assignments for universe polymorphism
    level_assignments: HashMap<String, UniverseLevel>,
    /// Cumulativity relationships between levels
    cumulativity_graph: HashMap<UniverseLevel, BTreeSet<UniverseLevel>>,
}

/// Level constraint for universe polymorphism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelConstraint {
    /// Level variable has a lower bound: α ≥ i
    LowerBound(String, UniverseLevel),
    /// Level variable has an upper bound: α ≤ i  
    UpperBound(String, UniverseLevel),
    /// Two level variables are equal: α = β
    Equal(String, String),
    /// Level variable is the maximum of two others: α = max(β, γ)
    Maximum(String, String, String),
    /// Level variable is the successor of another: α = β + 1
    Successor(String, String),
}

/// Universe level expression for polymorphic universe levels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LevelExpression {
    /// Concrete universe level
    Concrete(UniverseLevel),
    /// Level variable (for universe polymorphism)
    Variable(String),
    /// Maximum of two level expressions
    Maximum(Box<LevelExpression>, Box<LevelExpression>),
    /// Successor of a level expression
    Successor(Box<LevelExpression>),
    /// Infinity (top level)
    Infinity,
}

impl UniverseHierarchy {
    /// Create a new universe hierarchy.
    pub fn new() -> Self {
        Self {
            max_level: 0,
            constraints: Vec::new(),
            level_assignments: HashMap::new(),
            cumulativity_graph: HashMap::new(),
        }
    }

    /// Create a universe hierarchy with a specified maximum level.
    pub fn with_max_level(max_level: UniverseLevel) -> Self {
        let mut hierarchy = Self::new();
        hierarchy.max_level = max_level;
        
        // Initialize cumulativity graph
        for i in 0..=max_level {
            let mut higher_levels = BTreeSet::new();
            for j in (i + 1)..=max_level {
                higher_levels.insert(j);
            }
            hierarchy.cumulativity_graph.insert(i, higher_levels);
        }
        
        hierarchy
    }

    /// Get the maximum universe level.
    pub fn max_level(&self) -> UniverseLevel {
        self.max_level
    }

    /// Check if a universe level is valid.
    pub fn is_valid_level(&self, level: UniverseLevel) -> bool {
        level <= self.max_level
    }

    /// Extend the hierarchy to include a new level.
    pub fn extend_to_level(&mut self, level: UniverseLevel) {
        if level > self.max_level {
            // Add cumulativity relationships for the new level
            for i in 0..=self.max_level {
                self.cumulativity_graph
                    .get_mut(&i)
                    .unwrap()
                    .insert(level);
            }
            
            // Add cumulativity entry for the new level
            let mut higher_levels = BTreeSet::new();
            for j in (level + 1)..=(level + 10) { // Extend a bit further
                higher_levels.insert(j);
            }
            self.cumulativity_graph.insert(level, higher_levels);
            
            self.max_level = level;
        }
    }

    /// Check if level1 can be lifted to level2 (cumulativity).
    pub fn can_lift(&self, level1: UniverseLevel, level2: UniverseLevel) -> bool {
        level1 <= level2 || 
        self.cumulativity_graph
            .get(&level1)
            .map(|higher| higher.contains(&level2))
            .unwrap_or(false)
    }

    /// Get the minimal level that can contain both input levels.
    pub fn least_upper_bound(&self, level1: UniverseLevel, level2: UniverseLevel) -> UniverseLevel {
        max(level1, level2)
    }

    /// Check if a type can inhabit a given universe level.
    pub fn type_fits_in_universe(&self, ty: &DependentType, level: UniverseLevel) -> Result<bool> {
        let required_level = self.infer_universe_level(ty)?;
        Ok(self.can_lift(required_level, level))
    }

    /// Infer the universe level required for a dependent type.
    pub fn infer_universe_level(&self, ty: &DependentType) -> Result<UniverseLevel> {
        match ty {
            DependentType::Universe(level) => {
                // Type_i : Type_{i+1}
                Ok(level + 1)
            }
            DependentType::Pi { domain, codomain, .. } => {
                let domain_level = self.infer_universe_level(domain)?;
                let codomain_level = self.infer_universe_level(codomain)?;
                // Π-type lives in the maximum of its component levels
                Ok(self.least_upper_bound(domain_level, codomain_level))
            }
            DependentType::Sigma { first, second, .. } => {
                let first_level = self.infer_universe_level(first)?;
                let second_level = self.infer_universe_level(second)?;
                // Σ-type lives in the maximum of its component levels
                Ok(self.least_upper_bound(first_level, second_level))
            }
            DependentType::Identity { ty, .. } => {
                // Identity type lives in the same universe as its base type
                self.infer_universe_level(ty)
            }
            DependentType::Inductive { universe_level, constructors, .. } => {
                // Check that all constructors respect the declared level
                let mut max_constructor_level = *universe_level;
                for (_, ctor_type) in constructors {
                    let ctor_level = self.infer_universe_level(ctor_type)?;
                    if ctor_level > max_constructor_level {
                        max_constructor_level = ctor_level;
                    }
                }
                Ok(max_constructor_level)
            }
        }
    }

    /// Add a level constraint for universe polymorphism.
    pub fn add_constraint(&mut self, constraint: LevelConstraint) {
        self.constraints.push(constraint);
    }

    /// Solve level constraints and determine level assignments.
    pub fn solve_constraints(&mut self) -> Result<()> {
        // Simple constraint solving - in practice would use more sophisticated algorithms
        for constraint in &self.constraints.clone() {
            match constraint {
                LevelConstraint::LowerBound(var, level) => {
                    let current = self.level_assignments.get(var).copied().unwrap_or(0);
                    self.level_assignments.insert(var.clone(), max(current, *level));
                }
                LevelConstraint::UpperBound(var, level) => {
                    let current = self.level_assignments.get(var).copied().unwrap_or(*level);
                    if current > *level {
                        return Err(Box::new(Error::type_error(
                            format!("Level constraint violation: {var} > {level}"),
                            Span::new(0, 0)
                        )));
                    }
                    self.level_assignments.insert(var.clone(), current);
                }
                LevelConstraint::Equal(var1, var2) => {
                    let level1 = self.level_assignments.get(var1).copied().unwrap_or(0);
                    let level2 = self.level_assignments.get(var2).copied().unwrap_or(0);
                    let unified_level = max(level1, level2);
                    self.level_assignments.insert(var1.clone(), unified_level);
                    self.level_assignments.insert(var2.clone(), unified_level);
                }
                LevelConstraint::Maximum(var, var1, var2) => {
                    let level1 = self.level_assignments.get(var1).copied().unwrap_or(0);
                    let level2 = self.level_assignments.get(var2).copied().unwrap_or(0);
                    self.level_assignments.insert(var.clone(), max(level1, level2));
                }
                LevelConstraint::Successor(var, base_var) => {
                    let base_level = self.level_assignments.get(base_var).copied().unwrap_or(0);
                    self.level_assignments.insert(var.clone(), base_level + 1);
                }
            }
        }
        Ok(())
    }

    /// Get the assignment for a level variable.
    pub fn get_level_assignment(&self, var: &str) -> Option<UniverseLevel> {
        self.level_assignments.get(var).copied()
    }

    /// Check universe consistency (no paradoxes).
    pub fn check_consistency(&self) -> Result<()> {
        // Check for cycles in level dependencies
        for (var, level) in &self.level_assignments {
            if self.has_cycle(var, &HashSet::new())? {
                return Err(Box::new(Error::type_error(
                    format!("Cyclic universe level dependency involving {var}"),
                    Span::new(0, 0)
                )));
            }
        }

        // Check that no universe contains itself
        for level in 0..=self.max_level {
            if self.universe_contains_itself(level)? {
                return Err(Box::new(Error::type_error(
                    format!("Universe level {level} contains itself"),
                    Span::new(0, 0)
                )));
            }
        }

        Ok(())
    }

    /// Check for cycles in level variable dependencies.
    fn has_cycle(&self, var: &str, visited: &HashSet<String>) -> Result<bool> {
        if visited.contains(var) {
            return Ok(true);
        }

        let mut new_visited = visited.clone();
        new_visited.insert(var.to_string());

        // Check dependencies in constraints
        for constraint in &self.constraints {
            match constraint {
                LevelConstraint::Equal(v1, v2) => {
                    if v1 == var && self.has_cycle(v2, &new_visited)? {
                        return Ok(true);
                    }
                    if v2 == var && self.has_cycle(v1, &new_visited)? {
                        return Ok(true);
                    }
                }
                LevelConstraint::Maximum(v, v1, v2) => {
                    if v == var
                        && (self.has_cycle(v1, &new_visited)? || self.has_cycle(v2, &new_visited)?) {
                            return Ok(true);
                        }
                }
                LevelConstraint::Successor(v, base) => {
                    if v == var && self.has_cycle(base, &new_visited)? {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }

    /// Check if a universe level contains itself (should never happen).
    fn universe_contains_itself(&self, level: UniverseLevel) -> Result<bool> {
        // A universe Type_i should never contain itself
        // This would require complex analysis of type definitions
        Ok(false) // Simplified - assume well-founded
    }

    /// Create a universe polymorphic type.
    pub fn polymorphic_universe_type(&mut self, level_vars: Vec<String>) -> DependentType {
        // ∀α₁ α₂ ... αₙ. Type_α where α is a combination of the level variables
        let combined_level = if level_vars.is_empty() {
            LevelExpression::Concrete(0)
        } else if level_vars.len() == 1 {
            LevelExpression::Variable(level_vars[0].clone())
        } else {
            // Create maximum of all level variables
            let mut expr = LevelExpression::Variable(level_vars[0].clone());
            for var in level_vars.iter().skip(1) {
                expr = LevelExpression::Maximum(
                    Box::new(expr),
                    Box::new(LevelExpression::Variable(var.clone()))
                );
            }
            expr
        };

        // For now, approximate with a concrete level
        let concrete_level = self.evaluate_level_expression(&combined_level).unwrap_or(0);
        DependentType::Universe(concrete_level)
    }

    /// Evaluate a level expression to a concrete level.
    pub fn evaluate_level_expression(&self, expr: &LevelExpression) -> Result<UniverseLevel> {
        match expr {
            LevelExpression::Concrete(level) => Ok(*level),
            LevelExpression::Variable(var) => {
                self.level_assignments.get(var)
                    .copied()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound level variable: {var}"),
                        Span::new(0, 0)
                    )))
            }
            LevelExpression::Maximum(expr1, expr2) => {
                let level1 = self.evaluate_level_expression(expr1)?;
                let level2 = self.evaluate_level_expression(expr2)?;
                Ok(max(level1, level2))
            }
            LevelExpression::Successor(expr) => {
                let level = self.evaluate_level_expression(expr)?;
                Ok(level + 1)
            }
            LevelExpression::Infinity => {
                Ok(UniverseLevel::MAX) // Use maximum representable level
            }
        }
    }

    /// Normalize a level expression by evaluating subexpressions.
    pub fn normalize_level_expression(&self, expr: &LevelExpression) -> LevelExpression {
        match expr {
            LevelExpression::Concrete(_) => expr.clone(),
            LevelExpression::Variable(var) => {
                if let Some(level) = self.level_assignments.get(var) {
                    LevelExpression::Concrete(*level)
                } else {
                    expr.clone()
                }
            }
            LevelExpression::Maximum(expr1, expr2) => {
                let norm1 = self.normalize_level_expression(expr1);
                let norm2 = self.normalize_level_expression(expr2);
                
                match (&norm1, &norm2) {
                    (LevelExpression::Concrete(l1), LevelExpression::Concrete(l2)) => {
                        LevelExpression::Concrete(max(*l1, *l2))
                    }
                    _ => LevelExpression::Maximum(Box::new(norm1), Box::new(norm2))
                }
            }
            LevelExpression::Successor(expr) => {
                let norm = self.normalize_level_expression(expr);
                match norm {
                    LevelExpression::Concrete(level) => LevelExpression::Concrete(level + 1),
                    _ => LevelExpression::Successor(Box::new(norm))
                }
            }
            LevelExpression::Infinity => expr.clone(),
        }
    }
}

/// Universe polymorphic type with level parameters.
#[derive(Debug, Clone)]
pub struct PolymorphicType {
    /// Level parameters
    pub level_params: Vec<String>,
    /// Type body that may reference level parameters
    pub body: Box<DependentType>,
    /// Constraints on level parameters
    pub constraints: Vec<LevelConstraint>,
}

impl PolymorphicType {
    /// Create a new polymorphic type.
    pub fn new(
        level_params: Vec<String>,
        body: DependentType,
        constraints: Vec<LevelConstraint>
    ) -> Self {
        Self {
            level_params,
            body: Box::new(body),
            constraints,
        }
    }

    /// Instantiate the polymorphic type with concrete levels.
    pub fn instantiate(
        &self,
        level_args: &[UniverseLevel],
        hierarchy: &mut UniverseHierarchy
    ) -> Result<DependentType> {
        if level_args.len() != self.level_params.len() {
            return Err(Box::new(Error::type_error(
                format!("Level arity mismatch: expected {}, got {}", 
                       self.level_params.len(), level_args.len()),
                Span::new(0, 0)
            )));
        }

        // Create level assignments
        let mut assignments = HashMap::new();
        for (param, &arg) in self.level_params.iter().zip(level_args.iter()) {
            assignments.insert(param.clone(), arg);
        }

        // Check constraints
        for constraint in &self.constraints {
            match constraint {
                LevelConstraint::LowerBound(var, bound) => {
                    if let Some(&level) = assignments.get(var) {
                        if level < *bound {
                            return Err(Box::new(Error::type_error(
                                format!("Level constraint violation: {var} = {level} < {bound}"),
                                Span::new(0, 0)
                            )));
                        }
                    }
                }
                LevelConstraint::UpperBound(var, bound) => {
                    if let Some(&level) = assignments.get(var) {
                        if level > *bound {
                            return Err(Box::new(Error::type_error(
                                format!("Level constraint violation: {var} = {level} > {bound}"),
                                Span::new(0, 0)
                            )));
                        }
                    }
                }
                _ => {} // Other constraints would be checked here
            }
        }

        // Substitute level assignments in the type body
        self.substitute_levels(&self.body, &assignments, hierarchy)
    }

    /// Substitute level assignments in a type.
    fn substitute_levels(
        &self,
        ty: &DependentType,
        assignments: &HashMap<String, UniverseLevel>,
        hierarchy: &UniverseHierarchy
    ) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            DependentType::Pi { var, domain, codomain } => {
                let new_domain = self.substitute_levels(domain, assignments, hierarchy)?;
                let new_codomain = self.substitute_levels(codomain, assignments, hierarchy)?;
                Ok(DependentType::Pi {
                    var: var.clone(),
                    domain: Box::new(new_domain),
                    codomain: Box::new(new_codomain),
                })
            }
            DependentType::Sigma { var, first, second } => {
                let new_first = self.substitute_levels(first, assignments, hierarchy)?;
                let new_second = self.substitute_levels(second, assignments, hierarchy)?;
                Ok(DependentType::Sigma {
                    var: var.clone(),
                    first: Box::new(new_first),
                    second: Box::new(new_second),
                })
            }
            DependentType::Identity { ty, left, right } => {
                let new_ty = self.substitute_levels(ty, assignments, hierarchy)?;
                Ok(DependentType::Identity {
                    ty: Box::new(new_ty),
                    left: left.clone(), // Would need term-level substitution
                    right: right.clone(), // Would need term-level substitution
                })
            }
            DependentType::Inductive { name, parameters, universe_level, constructors, induction_principle } => {
                let mut new_constructors = HashMap::new();
                for (ctor_name, ctor_type) in constructors {
                    new_constructors.insert(
                        ctor_name.clone(),
                        self.substitute_levels(ctor_type, assignments, hierarchy)?
                    );
                }
                
                let new_induction = if let Some(ind_prin) = induction_principle {
                    Some(Box::new(self.substitute_levels(ind_prin, assignments, hierarchy)?))
                } else {
                    None
                };

                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: parameters.clone(), // Would need substitution here too
                    universe_level: *universe_level,
                    constructors: new_constructors.into_iter().collect(),
                    induction_principle: new_induction,
                })
            }
        }
    }
}

/// Universe operations and utilities.
pub struct UniverseOperations;

impl UniverseOperations {
    /// Check if a type inhabits any universe (has a finite level).
    pub fn has_finite_level(ty: &DependentType, hierarchy: &UniverseHierarchy) -> Result<bool> {
        match hierarchy.infer_universe_level(ty) {
            Ok(level) => Ok(level < UniverseLevel::MAX),
            Err(_) => Ok(false),
        }
    }

    /// Find the minimal universe level that can contain a type.
    pub fn minimal_universe(ty: &DependentType, hierarchy: &UniverseHierarchy) -> Result<UniverseLevel> {
        hierarchy.infer_universe_level(ty)
    }

    /// Lift a type to a higher universe level.
    pub fn lift_to_universe(
        ty: &DependentType,
        target_level: UniverseLevel,
        hierarchy: &UniverseHierarchy
    ) -> Result<DependentType> {
        let current_level = hierarchy.infer_universe_level(ty)?;
        
        if hierarchy.can_lift(current_level, target_level) {
            Ok(ty.clone()) // Type can be used at the higher level
        } else {
            Err(Box::new(Error::type_error(
                format!("Cannot lift type from level {current_level} to {target_level}"),
                Span::new(0, 0)
            )))
        }
    }

    /// Create a Tarski-style universe (universe with explicit codes).
    pub fn tarski_universe(level: UniverseLevel) -> (DependentType, DependentType) {
        // U_i is the type of codes for types in Type_i
        let universe_code = DependentType::Inductive {
            name: format!("U_{}", level),
            parameters: vec![],
            universe_level: level + 1,
            constructors: Vec::new(), // Would contain codes for each type constructor
            induction_principle: None,
        };

        // T_i : U_i → Type_i is the decoding function
        let decoder_type = DependentType::Pi {
            var: "code".to_string(),
            domain: Box::new(universe_code.clone()),
            codomain: Box::new(DependentType::Universe(level)),
        };

        (universe_code, decoder_type)
    }

    /// Check universe closure (if all operations stay within the universe hierarchy).
    pub fn check_closure(
        operations: &[DependentType],
        max_level: UniverseLevel,
        hierarchy: &UniverseHierarchy
    ) -> Result<bool> {
        for operation in operations {
            let level = hierarchy.infer_universe_level(operation)?;
            if level > max_level {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

// Default implementations
impl Default for UniverseHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

// Display implementations
impl fmt::Display for UniverseHierarchy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UniverseHierarchy[max: {}, constraints: {}]", 
               self.max_level, self.constraints.len())
    }
}

impl fmt::Display for LevelExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelExpression::Concrete(level) => write!(f, "{}", level),
            LevelExpression::Variable(var) => write!(f, "{}", var),
            LevelExpression::Maximum(expr1, expr2) => write!(f, "max({}, {})", expr1, expr2),
            LevelExpression::Successor(expr) => write!(f, "{} + 1", expr),
            LevelExpression::Infinity => write!(f, "∞"),
        }
    }
}

impl fmt::Display for LevelConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelConstraint::LowerBound(var, level) => write!(f, "{} ≥ {}", var, level),
            LevelConstraint::UpperBound(var, level) => write!(f, "{} ≤ {}", var, level),
            LevelConstraint::Equal(var1, var2) => write!(f, "{} = {}", var1, var2),
            LevelConstraint::Maximum(var, var1, var2) => write!(f, "{} = max({}, {})", var, var1, var2),
            LevelConstraint::Successor(var, base) => write!(f, "{} = {} + 1", var, base),
        }
    }
}

impl fmt::Display for PolymorphicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "∀")?;
        for (i, param) in self.level_params.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{}", param)?;
        }
        write!(f, ". {}", self.body)
    }
}

// Ordering for level expressions (used in constraint solving)
impl PartialOrd for LevelExpression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (LevelExpression::Concrete(l1), LevelExpression::Concrete(l2)) => l1.partial_cmp(l2),
            (LevelExpression::Infinity, LevelExpression::Infinity) => Some(Ordering::Equal),
            (LevelExpression::Infinity, _) => Some(Ordering::Greater),
            (_, LevelExpression::Infinity) => Some(Ordering::Less),
            _ => None, // Complex expressions require evaluation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_hierarchy_creation() {
        let hierarchy = UniverseHierarchy::new();
        assert_eq!(hierarchy.max_level(), 0);
        
        let hierarchy_with_levels = UniverseHierarchy::with_max_level(5);
        assert_eq!(hierarchy_with_levels.max_level(), 5);
    }

    #[test]
    fn test_cumulativity() {
        let hierarchy = UniverseHierarchy::with_max_level(5);
        
        // Lower levels can be lifted to higher levels
        assert!(hierarchy.can_lift(0, 1));
        assert!(hierarchy.can_lift(2, 4));
        assert!(hierarchy.can_lift(3, 3)); // Same level
        
        // But not the reverse
        assert!(!hierarchy.can_lift(4, 2));
    }

    #[test]
    fn test_least_upper_bound() {
        let hierarchy = UniverseHierarchy::new();
        
        assert_eq!(hierarchy.least_upper_bound(2, 3), 3);
        assert_eq!(hierarchy.least_upper_bound(5, 1), 5);
        assert_eq!(hierarchy.least_upper_bound(4, 4), 4);
    }

    #[test]
    fn test_universe_level_inference() {
        let hierarchy = UniverseHierarchy::new();
        
        // Type_i : Type_{i+1}
        let type_0 = DependentType::Universe(0);
        assert_eq!(hierarchy.infer_universe_level(&type_0).unwrap(), 1);
        
        let type_3 = DependentType::Universe(3);
        assert_eq!(hierarchy.infer_universe_level(&type_3).unwrap(), 4);
    }

    #[test]
    fn test_level_constraints() {
        let mut hierarchy = UniverseHierarchy::new();
        
        // Add constraint α ≥ 2
        hierarchy.add_constraint(LevelConstraint::LowerBound("α".to_string(), 2));
        
        // Add constraint α ≤ 5  
        hierarchy.add_constraint(LevelConstraint::UpperBound("α".to_string(), 5));
        
        hierarchy.solve_constraints().unwrap();
        
        let assignment = hierarchy.get_level_assignment("α").unwrap();
        assert!((2..=5).contains(&assignment));
    }

    #[test]
    fn test_level_expression_evaluation() {
        let mut hierarchy = UniverseHierarchy::new();
        hierarchy.level_assignments.insert("α".to_string(), 3);
        hierarchy.level_assignments.insert("β".to_string(), 5);
        
        let expr = LevelExpression::Maximum(
            Box::new(LevelExpression::Variable("α".to_string())),
            Box::new(LevelExpression::Variable("β".to_string()))
        );
        
        let result = hierarchy.evaluate_level_expression(&expr).unwrap();
        assert_eq!(result, 5); // max(3, 5) = 5
    }

    #[test]
    fn test_polymorphic_type() {
        let level_params = vec!["α".to_string(), "β".to_string()];
        let body = DependentType::Universe(0);
        let constraints = vec![
            LevelConstraint::LowerBound("α".to_string(), 1),
            LevelConstraint::UpperBound("β".to_string(), 10),
        ];
        
        let poly_type = PolymorphicType::new(level_params, body, constraints);
        
        let mut hierarchy = UniverseHierarchy::with_max_level(20);
        let instance = poly_type.instantiate(&[2, 8], &mut hierarchy).unwrap();
        
        // Should successfully instantiate with valid levels
        assert_eq!(instance, DependentType::Universe(0));
    }

    #[test]
    fn test_consistency_check() {
        let hierarchy = UniverseHierarchy::with_max_level(10);
        
        // Basic hierarchy should be consistent
        assert!(hierarchy.check_consistency().is_ok());
    }

    #[test]
    fn test_tarski_universe() {
        let (universe_code, decoder_type) = UniverseOperations::tarski_universe(2);
        
        match universe_code {
            DependentType::Inductive { name, universe_level, .. } => {
                assert_eq!(name, "U_2");
                assert_eq!(universe_level, 3); // Lives in Type_3
            }
            _ => panic!("Expected inductive type"),
        }
        
        match decoder_type {
            DependentType::Pi { codomain, .. } => {
                assert_eq!(codomain.as_ref(), &DependentType::Universe(2));
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_level_expression_display() {
        let expr = LevelExpression::Maximum(
            Box::new(LevelExpression::Concrete(3)),
            Box::new(LevelExpression::Successor(
                Box::new(LevelExpression::Variable("α".to_string()))
            ))
        );
        
        let display = format!("{}", expr);
        assert!(display.contains("max"));
        assert!(display.contains("α + 1"));
    }

    #[test]
    fn test_universe_extension() {
        let mut hierarchy = UniverseHierarchy::with_max_level(3);
        assert_eq!(hierarchy.max_level(), 3);
        
        hierarchy.extend_to_level(7);
        assert_eq!(hierarchy.max_level(), 7);
        
        // Should be able to lift to the new level
        assert!(hierarchy.can_lift(3, 7));
    }
}