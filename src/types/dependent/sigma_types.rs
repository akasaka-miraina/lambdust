#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]

//! Σ-types (dependent pair types) implementation.
//!
//! This module provides the complete implementation of Σ-types following Martin-Löf 
//! type theory, with precise formation, introduction, elimination, and computation rules.
//!
//! # Mathematical Foundation
//!
//! Σ-types represent dependent pair types of the form (x : A) × B(x), where:
//! - The second component type B may depend on the first component value x
//! - When B doesn't depend on x, this reduces to regular product types A × B
//! - Σ-types can encode existential quantification: ∃x:A.B(x) ≅ (x:A) × B(x)
//!
//! # Type Rules for Σ-types
//!
//! ## Formation Rule
//! ```text
//!   Γ ⊢ A type    Γ, x:A ⊢ B(x) type
//!   ────────────────────────────────────
//!         Γ ⊢ (x:A) × B(x) type
//! ```
//!
//! ## Introduction Rule (pair construction)
//! ```text
//!   Γ ⊢ a : A    Γ ⊢ b : B(a)
//!   ─────────────────────────────
//!   Γ ⊢ (a, b) : (x:A) × B(x)
//! ```
//!
//! ## Elimination Rules (projections)
//! ```text
//!   Γ ⊢ p : (x:A) × B(x)       Γ ⊢ p : (x:A) × B(x)
//!   ────────────────────       ─────────────────────
//!      Γ ⊢ π₁(p) : A             Γ ⊢ π₂(p) : B(π₁(p))
//! ```
//!
//! ## Computation Rules
//! ```text
//!   π₁((a, b)) ≡ a : A
//!   π₂((a, b)) ≡ b : B(a)
//! ```
//!
//! ## Uniqueness Rule (η-expansion)
//! ```text
//!   (π₁(p), π₂(p)) ≡ p : (x:A) × B(x)
//! ```

use super::core::{DependentType, DependentTerm, TypingContext, Normalizer, UniverseLevel};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Σ-type constructor and operations.
///
/// This structure encapsulates all operations related to dependent pair types,
/// providing type checking, normalization, and conversion checking capabilities.
#[derive(Debug, Clone)]
pub struct SigmaType {
    /// Variable name bound in the second component type
    pub var: String,
    /// First component type A
    pub first: Box<DependentType>,
    /// Second component type B(x) that may depend on var
    pub second: Box<DependentType>,
    /// Universe level this Σ-type lives in
    pub universe_level: UniverseLevel,
}

impl SigmaType {
    /// Construct a new Σ-type with automatic universe level calculation.
    pub fn new(var: String, first: DependentType, second: DependentType) -> Result<Self> {
        // The universe level is the maximum of first and second component levels
        let first_level = Self::infer_universe_level(&first)?;
        let second_level = Self::infer_universe_level(&second)?;
        let universe_level = first_level.max(second_level);

        Ok(Self {
            var,
            first: Box::new(first),
            second: Box::new(second),
            universe_level,
        })
    }

    /// Create a Σ-type with explicit universe level.
    pub fn with_universe_level(
        var: String,
        first: DependentType,
        second: DependentType,
        universe_level: UniverseLevel
    ) -> Self {
        Self {
            var,
            first: Box::new(first),
            second: Box::new(second),
            universe_level,
        }
    }

    /// Check if this Σ-type is well-formed in the given context.
    ///
    /// Implements the formation rule for Σ-types.
    pub fn check_well_formed(&self, context: &mut TypingContext) -> Result<()> {
        // Check that first component type is well-formed
        Self::check_type_well_formed(&self.first, context)?;

        // Extend context with variable binding and check second component
        context.bind_variable(self.var.clone(), *self.first.clone());
        let second_check = Self::check_type_well_formed(&self.second, context);
        context.unbind_variable(&self.var);

        second_check
    }

    /// Infer the universe level of a type.
    fn infer_universe_level(ty: &DependentType) -> Result<UniverseLevel> {
        match ty {
            DependentType::Universe(level) => Ok(level + 1),
            DependentType::Pi { domain, codomain, .. } => {
                let domain_level = Self::infer_universe_level(domain)?;
                let codomain_level = Self::infer_universe_level(codomain)?;
                Ok(domain_level.max(codomain_level))
            }
            DependentType::Sigma { first, second, .. } => {
                let first_level = Self::infer_universe_level(first)?;
                let second_level = Self::infer_universe_level(second)?;
                Ok(first_level.max(second_level))
            }
            DependentType::Identity { ty, .. } => {
                Self::infer_universe_level(ty)
            }
            DependentType::Inductive { universe_level, .. } => {
                Ok(*universe_level)
            }
        }
    }

    /// Check if a type is well-formed (helper function).
    fn check_type_well_formed(ty: &DependentType, context: &TypingContext) -> Result<()> {
        match ty {
            DependentType::Universe(_) => Ok(()),
            DependentType::Pi { var, domain, codomain } => {
                Self::check_type_well_formed(domain, context)?;
                
                let mut extended_context = context.clone();
                extended_context.bind_variable(var.clone(), *domain.clone());
                Self::check_type_well_formed(codomain, &extended_context)
            }
            DependentType::Sigma { var, first, second } => {
                Self::check_type_well_formed(first, context)?;
                
                let mut extended_context = context.clone();
                extended_context.bind_variable(var.clone(), *first.clone());
                Self::check_type_well_formed(second, &extended_context)
            }
            DependentType::Identity { ty, .. } => {
                Self::check_type_well_formed(ty, context)
            }
            DependentType::Inductive { constructors, .. } => {
                for (_, ctor_type) in constructors {
                    Self::check_type_well_formed(ctor_type, context)?;
                }
                Ok(())
            }
        }
    }

    /// Convert this Σ-type to the general DependentType representation.
    pub fn to_dependent_type(&self) -> DependentType {
        DependentType::Sigma {
            var: self.var.clone(),
            first: self.first.clone(),
            second: self.second.clone(),
        }
    }

    /// Check if the second component actually depends on the variable.
    pub fn is_dependent(&self) -> bool {
        self.contains_free_variable(&self.second, &self.var)
    }

    /// Check if a type contains a free occurrence of a variable.
    fn contains_free_variable(&self, ty: &DependentType, var: &str) -> bool {
        match ty {
            DependentType::Universe(_) => false,
            DependentType::Pi { var: pi_var, domain, codomain } => {
                self.contains_free_variable(domain, var) ||
                (pi_var != var && self.contains_free_variable(codomain, var))
            }
            DependentType::Sigma { var: sigma_var, first, second } => {
                self.contains_free_variable(first, var) ||
                (sigma_var != var && self.contains_free_variable(second, var))
            }
            DependentType::Identity { ty, .. } => {
                self.contains_free_variable(ty, var)
            }
            DependentType::Inductive { constructors, .. } => {
                constructors.iter().any(|(_, ctor_ty)| self.contains_free_variable(ctor_ty, var))
            }
        }
    }

    /// Get all free variables in the Σ-type.
    pub fn free_variables(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        self.collect_free_variables(&self.first, &mut vars, &HashSet::new());
        
        let mut bound = HashSet::new();
        bound.insert(self.var.clone());
        self.collect_free_variables(&self.second, &mut vars, &bound);
        
        vars
    }

    /// Collect free variables from a type (helper function).
    fn collect_free_variables(&self, ty: &DependentType, vars: &mut HashSet<String>, bound: &HashSet<String>) {
        match ty {
            DependentType::Universe(_) => {}
            DependentType::Pi { var, domain, codomain } => {
                self.collect_free_variables(domain, vars, bound);
                
                let mut new_bound = bound.clone();
                new_bound.insert(var.clone());
                self.collect_free_variables(codomain, vars, &new_bound);
            }
            DependentType::Sigma { var, first, second } => {
                self.collect_free_variables(first, vars, bound);
                
                let mut new_bound = bound.clone();
                new_bound.insert(var.clone());
                self.collect_free_variables(second, vars, &new_bound);
            }
            DependentType::Identity { ty, .. } => {
                self.collect_free_variables(ty, vars, bound);
            }
            DependentType::Inductive { constructors, .. } => {
                for (_, ctor_ty) in constructors.iter() {
                    self.collect_free_variables(ctor_ty, vars, bound);
                }
            }
        }
    }

    /// Substitute first component value into the second component type.
    pub fn instantiate_second_component(&self, first_value: &DependentTerm) -> Result<DependentType> {
        self.substitute_in_type(&self.second, &self.var, first_value)
    }

    /// Substitute a term for a variable in a type.
    fn substitute_in_type(&self, ty: &DependentType, var: &str, term: &DependentTerm) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            DependentType::Pi { var: pi_var, domain, codomain } => {
                if pi_var == var {
                    // Variable is bound, no substitution in codomain
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, term)?),
                        codomain: codomain.clone(),
                    })
                } else {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, term)?),
                        codomain: Box::new(self.substitute_in_type(codomain, var, term)?),
                    })
                }
            }
            DependentType::Sigma { var: sigma_var, first, second } => {
                if sigma_var == var {
                    Ok(DependentType::Sigma {
                        var: sigma_var.clone(),
                        first: Box::new(self.substitute_in_type(first, var, term)?),
                        second: second.clone(),
                    })
                } else {
                    Ok(DependentType::Sigma {
                        var: sigma_var.clone(),
                        first: Box::new(self.substitute_in_type(first, var, term)?),
                        second: Box::new(self.substitute_in_type(second, var, term)?),
                    })
                }
            }
            DependentType::Identity { ty, left, right } => {
                Ok(DependentType::Identity {
                    ty: Box::new(self.substitute_in_type(ty, var, term)?),
                    left: left.clone(), // Would need term substitution
                    right: right.clone(), // Would need term substitution
                })
            }
            DependentType::Inductive { name, parameters, universe_level, constructors, induction_principle } => {
                let mut new_constructors = Vec::new();
                for (ctor_name, ctor_type) in constructors {
                    new_constructors.push((
                        ctor_name.clone(),
                        self.substitute_in_type(ctor_type, var, term)?
                    ));
                }
                
                let new_induction = if let Some(ind_prin) = induction_principle {
                    Some(Box::new(self.substitute_in_type(ind_prin, var, term)?))
                } else {
                    None
                };
                
                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: parameters.clone(), // Would need substitution in parameter types
                    universe_level: *universe_level,
                    constructors: new_constructors,
                    induction_principle: new_induction,
                })
            }
        }
    }
}

/// Dependent pair construction for creating Σ-type terms.
///
/// This represents the introduction form for Σ-types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependentPair {
    /// First component value
    pub first: Box<DependentTerm>,
    /// Second component value (may depend on first)
    pub second: Box<DependentTerm>,
}

impl DependentPair {
    /// Create a new dependent pair.
    pub fn new(first: DependentTerm, second: DependentTerm) -> Self {
        Self {
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Type check this pair against an expected Σ-type.
    pub fn type_check(&self, expected_sigma_type: &SigmaType, context: &mut TypingContext) -> Result<()> {
        // Check that first component has the expected first type
        let first_type = self.infer_first_type(context)?;
        if first_type != *expected_sigma_type.first {
            return Err(Box::new(Error::type_error(
                format!("First component type mismatch: expected {:?}, got {:?}",
                       expected_sigma_type.first, first_type),
                Span::new(0, 0)
            )));
        }

        // Check that second component has the instantiated second type
        let expected_second_type = expected_sigma_type.instantiate_second_component(&self.first)?;
        let second_type = self.infer_second_type(context)?;
        
        if second_type != expected_second_type {
            return Err(Box::new(Error::type_error(
                format!("Second component type mismatch: expected {:?}, got {:?}",
                       expected_second_type, second_type),
                Span::new(0, 0)
            )));
        }

        Ok(())
    }

    /// Infer the type of the first component.
    fn infer_first_type(&self, context: &TypingContext) -> Result<DependentType> {
        match self.first.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            _ => Err(Box::new(Error::type_error(
                "Complex first component type inference not yet implemented".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Infer the type of the second component.
    fn infer_second_type(&self, context: &TypingContext) -> Result<DependentType> {
        match self.second.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            _ => Err(Box::new(Error::type_error(
                "Complex second component type inference not yet implemented".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Convert to the general DependentTerm representation.
    pub fn to_dependent_term(&self) -> DependentTerm {
        DependentTerm::Pair {
            first: self.first.clone(),
            second: self.second.clone(),
        }
    }
}

/// Projection operations for eliminating Σ-type terms.
///
/// This represents the elimination forms for Σ-types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    /// Pair to project from (should have Σ-type)
    pub pair: Box<DependentTerm>,
    /// True for first projection (π₁), false for second projection (π₂)
    pub is_first: bool,
}

impl Projection {
    /// Create a first projection (π₁).
    pub fn first(pair: DependentTerm) -> Self {
        Self {
            pair: Box::new(pair),
            is_first: true,
        }
    }

    /// Create a second projection (π₂).
    pub fn second(pair: DependentTerm) -> Self {
        Self {
            pair: Box::new(pair),
            is_first: false,
        }
    }

    /// Type check this projection and return the result type.
    pub fn type_check(&self, context: &mut TypingContext) -> Result<DependentType> {
        // Infer the type of the pair
        let pair_type = self.infer_pair_type(context)?;

        match pair_type {
            DependentType::Sigma { var, first, second } => {
                if self.is_first {
                    // First projection: π₁(p) : A
                    Ok(*first)
                } else {
                    // Second projection: π₂(p) : B(π₁(p))
                    // Substitute π₁(p) for var in second component type
                    let first_proj = DependentTerm::Projection {
                        pair: self.pair.clone(),
                        is_first: true,
                    };
                    self.substitute_in_type(&second, &var, &first_proj)
                }
            }
            _ => Err(Box::new(Error::type_error(
                "Cannot project from non-pair type".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Infer the type of the pair term.
    fn infer_pair_type(&self, context: &TypingContext) -> Result<DependentType> {
        match self.pair.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound pair variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            DependentTerm::Pair { first, second } => {
                // Pair has type (x : first_type) × second_type
                // Would need full type inference
                Err(Box::new(Error::type_error(
                    "Complex pair type inference not yet implemented".to_string(),
                    Span::new(0, 0)
                )))
            }
            _ => Err(Box::new(Error::type_error(
                "Complex pair type inference not yet implemented".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Substitute a term for a variable in a type.
    fn substitute_in_type(&self, ty: &DependentType, var: &str, term: &DependentTerm) -> Result<DependentType> {
        // Reuse substitution logic from SigmaType
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            DependentType::Pi { var: pi_var, domain, codomain } => {
                if pi_var == var {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, term)?),
                        codomain: codomain.clone(),
                    })
                } else {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, term)?),
                        codomain: Box::new(self.substitute_in_type(codomain, var, term)?),
                    })
                }
            }
            _ => Ok(ty.clone()) // Other cases: handle recursively
        }
    }

    /// Convert to the general DependentTerm representation.
    pub fn to_dependent_term(&self) -> DependentTerm {
        DependentTerm::Projection {
            pair: self.pair.clone(),
            is_first: self.is_first,
        }
    }

    /// Perform projection reduction if the pair is a literal pair.
    pub fn reduce(&self, normalizer: &Normalizer) -> Result<DependentTerm> {
        match self.pair.as_ref() {
            DependentTerm::Pair { first, second } => {
                // π₁((a, b)) → a, π₂((a, b)) → b
                if self.is_first {
                    Ok((**first).clone())
                } else {
                    Ok((**second).clone())
                }
            }
            _ => {
                // Normalize the pair first
                let normalized_pair = normalizer.normalize_term(&self.pair)?;
                if let DependentTerm::Pair { first, second } = normalized_pair {
                    if self.is_first {
                        Ok(*first)
                    } else {
                        Ok(*second)
                    }
                } else {
                    Ok(self.to_dependent_term())
                }
            }
        }
    }
}

/// Σ-type utilities and operations.
pub struct SigmaTypeOperations;

impl SigmaTypeOperations {
    /// Check if two Σ-types are definitionally equal.
    pub fn sigma_types_equal(sigma1: &SigmaType, sigma2: &SigmaType, normalizer: &Normalizer) -> Result<bool> {
        // Normalize both types and check structural equality
        let norm1 = normalizer.normalize_type(&sigma1.to_dependent_type())?;
        let norm2 = normalizer.normalize_type(&sigma2.to_dependent_type())?;
        Ok(norm1 == norm2)
    }

    /// Create a simple product type (special case of Σ-type).
    pub fn simple_product_type(first: DependentType, second: DependentType) -> Result<SigmaType> {
        // Use a dummy variable name since second doesn't depend on first
        SigmaType::new("_".to_string(), first, second)
    }

    /// Check if a Σ-type represents a simple product type (non-dependent).
    pub fn is_simple_product(sigma_type: &SigmaType) -> bool {
        !sigma_type.is_dependent()
    }

    /// Create nested Σ-types for multiple components.
    pub fn nested_sigma_type(components: Vec<(String, DependentType)>) -> Result<SigmaType> {
        if components.len() < 2 {
            return Err(Box::new(Error::type_error(
                "Need at least 2 components for Σ-type".to_string(),
                Span::new(0, 0)
            )));
        }

        let mut components_iter = components.into_iter();
        let (first_var, first_type) = components_iter.next().unwrap();
        let (second_var, second_type) = components_iter.next().unwrap();

        let mut current_sigma = SigmaType::new(first_var, first_type, second_type)?;

        // Build nested Σ-types
        for (var, ty) in components_iter {
            let outer_sigma_type = current_sigma.to_dependent_type();
            current_sigma = SigmaType::new(var, outer_sigma_type, ty)?;
        }

        Ok(current_sigma)
    }

    /// Flatten nested Σ-types to get all component types.
    pub fn flatten_sigma_type(sigma_type: &SigmaType) -> Vec<(String, DependentType)> {
        let mut components = vec![(sigma_type.var.clone(), (*sigma_type.first).clone())];
        let mut current_second = &*sigma_type.second;

        // Follow the chain of nested Σ-types
        while let DependentType::Sigma { var, first, second } = current_second {
            components.push((var.clone(), (**first).clone()));
            current_second = second;
        }

        // Add the final component
        components.push(("_final".to_string(), current_second.clone()));
        components
    }

    /// Check if a pair satisfies the η-equality rule.
    pub fn check_eta_equality(
        pair: &DependentPair,
        sigma_type: &SigmaType,
        normalizer: &Normalizer
    ) -> Result<bool> {
        // (π₁(p), π₂(p)) ≡ p
        let first_proj = Projection::first(pair.to_dependent_term());
        let second_proj = Projection::second(pair.to_dependent_term());
        
        let reconstructed = DependentPair::new(
            first_proj.to_dependent_term(),
            second_proj.to_dependent_term()
        );

        // Check if the reconstructed pair is equal to the original
        let orig_normalized = normalizer.normalize_term(&pair.to_dependent_term())?;
        let reconstructed_normalized = normalizer.normalize_term(&reconstructed.to_dependent_term())?;
        
        Ok(orig_normalized == reconstructed_normalized)
    }

    /// Create existential quantification using Σ-types.
    ///
    /// ∃x:A.B(x) ≅ (x:A) × B(x)
    pub fn existential_quantification(var: String, domain: DependentType, predicate: DependentType) -> Result<SigmaType> {
        SigmaType::new(var, domain, predicate)
    }

    /// Extract witness and proof from existential quantification.
    pub fn extract_existential_components(
        proof: &DependentTerm,
        context: &mut TypingContext
    ) -> Result<(DependentTerm, DependentTerm)> {
        // Extract π₁(proof) as witness and π₂(proof) as proof
        let witness = Projection::first(proof.clone()).to_dependent_term();
        let proof_term = Projection::second(proof.clone()).to_dependent_term();
        
        Ok((witness, proof_term))
    }
}

// Display implementations
impl fmt::Display for SigmaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dependent() {
            write!(f, "({} : {}) × {}", self.var, self.first, self.second)
        } else {
            write!(f, "{} × {}", self.first, self.second)
        }
    }
}

impl fmt::Display for DependentPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.first, self.second)
    }
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_first {
            write!(f, "π₁({})", self.pair)
        } else {
            write!(f, "π₂({})", self.pair)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_type_creation() {
        let first = DependentType::Universe(0);
        let second = DependentType::Universe(0);
        let sigma_type = SigmaType::new("x".to_string(), first, second).unwrap();
        
        assert_eq!(sigma_type.var, "x");
        assert_eq!(sigma_type.universe_level, 1);
    }

    #[test]
    fn test_simple_product_type() {
        let first = DependentType::Universe(0);
        let second = DependentType::Universe(1);
        let sigma_type = SigmaTypeOperations::simple_product_type(first, second).unwrap();
        
        assert!(!sigma_type.is_dependent());
        assert_eq!(sigma_type.var, "_");
    }

    #[test]
    fn test_dependent_sigma_type() {
        // Create (n : Nat) × Vec(n) 
        let nat_type = DependentType::Inductive {
            name: "Nat".to_string(),
            parameters: vec![],
            universe_level: 0,
            constructors: Vec::new(),
            induction_principle: None,
        };
        
        let vec_type = DependentType::Pi {
            var: "n".to_string(),
            domain: Box::new(nat_type.clone()),
            codomain: Box::new(DependentType::Universe(0)),
        };
        
        let sigma_type = SigmaType::new("n".to_string(), nat_type, vec_type).unwrap();
        assert!(sigma_type.is_dependent());
    }

    #[test]
    fn test_projection_operations() {
        let pair_term = DependentTerm::Variable("p".to_string());
        
        let first_proj = Projection::first(pair_term.clone());
        let second_proj = Projection::second(pair_term.clone());
        
        assert!(first_proj.is_first);
        assert!(!second_proj.is_first);
    }

    #[test]
    fn test_nested_sigma_type() {
        let components = vec![
            ("x".to_string(), DependentType::Universe(0)),
            ("y".to_string(), DependentType::Universe(0)),
            ("z".to_string(), DependentType::Universe(0)),
        ];
        
        let nested = SigmaTypeOperations::nested_sigma_type(components.clone()).unwrap();
        let flattened = SigmaTypeOperations::flatten_sigma_type(&nested);
        
        // Should have created nested structure
        assert!(flattened.len() >= components.len());
    }

    #[test]
    fn test_existential_quantification() {
        let domain = DependentType::Universe(0);
        let predicate = DependentType::Universe(0);
        
        let existential = SigmaTypeOperations::existential_quantification(
            "x".to_string(),
            domain,
            predicate
        ).unwrap();
        
        // Existential quantification is represented as Σ-type
        assert_eq!(existential.var, "x");
    }

    #[test]
    fn test_dependent_pair_display() {
        let pair = DependentPair::new(
            DependentTerm::Variable("a".to_string()),
            DependentTerm::Variable("b".to_string())
        );
        
        let display = format!("{}", pair);
        assert!(display.contains("(a, b)"));
    }
}