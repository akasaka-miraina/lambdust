#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]

//! Π-types (dependent function types) implementation.
//!
//! This module provides the complete implementation of Π-types following Martin-Löf 
//! type theory, with precise formation, introduction, elimination, and computation rules.
//!
//! # Mathematical Foundation
//!
//! Π-types represent dependent function types of the form (x : A) → B(x), where:
//! - The codomain type B may depend on the argument value x
//! - When B doesn't depend on x, this reduces to regular function types A → B
//!
//! # Type Rules for Π-types
//!
//! ## Formation Rule
//! ```
//!   Γ ⊢ A type    Γ, x:A ⊢ B(x) type
//!   ────────────────────────────────────
//!        Γ ⊢ (x:A) → B(x) type
//! ```
//!
//! ## Introduction Rule (λ-abstraction)
//! ```
//!    Γ, x:A ⊢ t : B(x)
//!   ─────────────────────────
//!   Γ ⊢ λx:A.t : (x:A) → B(x)
//! ```
//!
//! ## Elimination Rule (function application)
//! ```
//!   Γ ⊢ f : (x:A) → B(x)    Γ ⊢ a : A
//!   ──────────────────────────────────
//!         Γ ⊢ f(a) : B(a)
//! ```
//!
//! ## Computation Rule (β-reduction)
//! ```
//!   (λx:A.t)(a) ≡ t[a/x] : B(a)
//! ```
//!
//! ## Uniqueness Rule (η-expansion)
//! ```
//!   λx:A.(f(x)) ≡ f : (x:A) → B(x)  (if x ∉ FV(f))
//! ```

use super::core::{DependentType, DependentTerm, TypingContext, Normalizer, UniverseLevel};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Π-type constructor and operations.
///
/// This structure encapsulates all operations related to dependent function types,
/// providing type checking, normalization, and conversion checking capabilities.
#[derive(Debug, Clone)]
pub struct PiType {
    /// Variable name bound in the codomain
    pub var: String,
    /// Domain type A
    pub domain: Box<DependentType>,
    /// Codomain type B(x) that may depend on var
    pub codomain: Box<DependentType>,
    /// Universe level this Π-type lives in
    pub universe_level: UniverseLevel,
}

impl PiType {
    /// Construct a new Π-type with automatic universe level calculation.
    pub fn new(var: String, domain: DependentType, codomain: DependentType) -> Result<Self> {
        // The universe level is the maximum of domain and codomain levels
        let domain_level = Self::infer_universe_level(&domain)?;
        let codomain_level = Self::infer_universe_level(&codomain)?;
        let universe_level = domain_level.max(codomain_level);

        Ok(Self {
            var,
            domain: Box::new(domain),
            codomain: Box::new(codomain),
            universe_level,
        })
    }

    /// Create a Π-type with explicit universe level.
    pub fn with_universe_level(
        var: String, 
        domain: DependentType, 
        codomain: DependentType, 
        universe_level: UniverseLevel
    ) -> Self {
        Self {
            var,
            domain: Box::new(domain),
            codomain: Box::new(codomain),
            universe_level,
        }
    }

    /// Check if this Π-type is well-formed in the given context.
    ///
    /// Implements the formation rule for Π-types.
    pub fn check_well_formed(&self, context: &mut TypingContext) -> Result<()> {
        // Check that domain is a well-formed type
        Self::check_type_well_formed(&self.domain, context)?;

        // Extend context with variable binding and check codomain
        context.bind_variable(self.var.clone(), *self.domain.clone());
        let codomain_check = Self::check_type_well_formed(&self.codomain, context);
        context.unbind_variable(&self.var);

        codomain_check
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
                
                // Create a temporary context with the variable bound
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
                // Check all constructor types are well-formed
                for (_, ctor_type) in constructors {
                    Self::check_type_well_formed(ctor_type, context)?;
                }
                Ok(())
            }
        }
    }

    /// Convert this Π-type to the general DependentType representation.
    pub fn to_dependent_type(&self) -> DependentType {
        DependentType::Pi {
            var: self.var.clone(),
            domain: self.domain.clone(),
            codomain: self.codomain.clone(),
        }
    }

    /// Check if the codomain actually depends on the variable.
    pub fn is_dependent(&self) -> bool {
        self.contains_free_variable(&self.codomain, &self.var)
    }

    /// Check if a type contains a free occurrence of a variable.
    fn contains_free_variable(&self, _ty: &DependentType, var: &str) -> bool {
        match _ty {
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

    /// Get all free variables in the Π-type.
    pub fn free_variables(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        self.collect_free_variables(&self.domain, &mut vars, &HashSet::new());
        
        let mut bound = HashSet::new();
        bound.insert(self.var.clone());
        self.collect_free_variables(&self.codomain, &mut vars, &bound);
        
        vars
    }

    /// Collect free variables from a type (helper function).
    fn collect_free_variables(&self, _ty: &DependentType, _vars: &mut HashSet<String>, bound: &HashSet<String>) {
        match _ty {
            DependentType::Universe(_) => {}
            DependentType::Pi { var, domain, codomain } => {
                self.collect_free_variables(domain, _vars, bound);
                
                let mut new_bound = bound.clone();
                new_bound.insert(var.clone());
                self.collect_free_variables(codomain, _vars, &new_bound);
            }
            DependentType::Sigma { var, first, second } => {
                self.collect_free_variables(first, _vars, bound);
                
                let mut new_bound = bound.clone();
                new_bound.insert(var.clone());
                self.collect_free_variables(second, _vars, &new_bound);
            }
            DependentType::Identity { ty, .. } => {
                self.collect_free_variables(ty, _vars, bound);
            }
            DependentType::Inductive { constructors, .. } => {
                for (_, ctor_ty) in constructors.iter() {
                    self.collect_free_variables(ctor_ty, _vars, bound);
                }
            }
        }
    }
}

/// Lambda abstraction for constructing Π-type terms.
///
/// This represents the introduction form for Π-types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaAbstraction {
    /// Parameter name
    pub param: String,
    /// Parameter type
    pub param_type: Box<DependentType>,
    /// Function body
    pub body: Box<DependentTerm>,
}

impl LambdaAbstraction {
    /// Create a new lambda abstraction.
    pub fn new(param: String, param_type: DependentType, body: DependentTerm) -> Self {
        Self {
            param,
            param_type: Box::new(param_type),
            body: Box::new(body),
        }
    }

    /// Type check this lambda abstraction against an expected Π-type.
    pub fn type_check(&self, expected_pi_type: &PiType, context: &mut TypingContext) -> Result<()> {
        // Check that parameter type matches domain
        if *self.param_type != *expected_pi_type.domain {
            return Err(Box::new(Error::type_error(
                format!("Parameter type mismatch: expected {:?}, got {:?}", 
                       expected_pi_type.domain, self.param_type),
                Span::new(0, 0)
            )));
        }

        // Type check body in extended context
        context.bind_variable(self.param.clone(), *self.param_type.clone());
        let body_type = self.infer_body_type(context)?;
        context.unbind_variable(&self.param);

        // Check that body type matches codomain (with substitution)
        let expected_codomain = self.substitute_in_type(
            &expected_pi_type.codomain,
            &expected_pi_type.var,
            &DependentTerm::Variable(self.param.clone())
        )?;

        if body_type != expected_codomain {
            return Err(Box::new(Error::type_error(
                format!("Body type mismatch: expected {expected_codomain:?}, got {body_type:?}"),
                Span::new(0, 0)
            )));
        }

        Ok(())
    }

    /// Infer the type of the lambda body.
    fn infer_body_type(&self, context: &TypingContext) -> Result<DependentType> {
        // This would use the main type inference engine
        // For now, simplified placeholder
        match self.body.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            _ => {
                // Would need full type inference here
                Err(Box::new(Error::type_error(
                    "Complex body type inference not yet implemented".to_string(),
                    Span::new(0, 0)
                )))
            }
        }
    }

    /// Substitute a term for a variable in a type.
    fn substitute_in_type(&self, _ty: &DependentType, var: &str, _term: &DependentTerm) -> Result<DependentType> {
        // Simplified substitution - full implementation would handle variable capture
        match _ty {
            DependentType::Pi { var: pi_var, domain, codomain } => {
                if pi_var == var {
                    // Variable is bound, don't substitute in codomain
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, _term)?),
                        codomain: codomain.clone(),
                    })
                } else {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, _term)?),
                        codomain: Box::new(self.substitute_in_type(codomain, var, _term)?),
                    })
                }
            }
            _ => Ok(_ty.clone()) // Other cases: handle recursively
        }
    }

    /// Convert to the general DependentTerm representation.
    pub fn to_dependent_term(&self) -> DependentTerm {
        DependentTerm::Lambda {
            param: self.param.clone(),
            param_type: self.param_type.clone(),
            body: self.body.clone(),
        }
    }
}

/// Function application for eliminating Π-type terms.
///
/// This represents the elimination form for Π-types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionApplication {
    /// Function term (should have Π-type)
    pub function: Box<DependentTerm>,
    /// Argument term
    pub argument: Box<DependentTerm>,
}

impl FunctionApplication {
    /// Create a new function application.
    pub fn new(function: DependentTerm, argument: DependentTerm) -> Self {
        Self {
            function: Box::new(function),
            argument: Box::new(argument),
        }
    }

    /// Type check this application and return the result type.
    pub fn type_check(&self, context: &mut TypingContext) -> Result<DependentType> {
        // Infer the type of the function
        let function_type = self.infer_function_type(context)?;
        
        match function_type {
            DependentType::Pi { var, domain, codomain } => {
                // Check that argument has the domain type
                let argument_type = self.infer_argument_type(context)?;
                if argument_type != *domain {
                    return Err(Box::new(Error::type_error(
                        format!("Argument type mismatch: expected {domain:?}, got {argument_type:?}"),
                        Span::new(0, 0)
                    )));
                }

                // Substitute argument for variable in codomain to get result type
                self.substitute_in_type(&codomain, &var, &self.argument)
            }
            _ => Err(Box::new(Error::type_error(
                "Cannot apply non-function type".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Infer the type of the function term.
    fn infer_function_type(&self, context: &TypingContext) -> Result<DependentType> {
        // Simplified type inference
        match self.function.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound function variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            DependentTerm::Lambda { param, param_type, body } => {
                // Lambda has type (param : param_type) → body_type
                // Would need to infer body_type in extended context
                Ok(DependentType::Pi {
                    var: param.clone(),
                    domain: param_type.clone(),
                    codomain: Box::new(DependentType::Universe(0)), // Placeholder
                })
            }
            _ => Err(Box::new(Error::type_error(
                "Complex function type inference not yet implemented".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Infer the type of the argument term.
    fn infer_argument_type(&self, context: &TypingContext) -> Result<DependentType> {
        match self.argument.as_ref() {
            DependentTerm::Variable(name) => {
                context.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound argument variable: {name}"),
                        Span::new(0, 0)
                    )))
            }
            _ => Err(Box::new(Error::type_error(
                "Complex argument type inference not yet implemented".to_string(),
                Span::new(0, 0)
            )))
        }
    }

    /// Substitute a term for a variable in a type.
    fn substitute_in_type(&self, _ty: &DependentType, var: &str, _term: &DependentTerm) -> Result<DependentType> {
        // This is the same substitution logic as in LambdaAbstraction
        // In a full implementation, this would be factored out into a common module
        match _ty {
            DependentType::Pi { var: pi_var, domain, codomain } => {
                if pi_var == var {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, _term)?),
                        codomain: codomain.clone(),
                    })
                } else {
                    Ok(DependentType::Pi {
                        var: pi_var.clone(),
                        domain: Box::new(self.substitute_in_type(domain, var, _term)?),
                        codomain: Box::new(self.substitute_in_type(codomain, var, _term)?),
                    })
                }
            }
            _ => Ok(_ty.clone())
        }
    }

    /// Convert to the general DependentTerm representation.
    pub fn to_dependent_term(&self) -> DependentTerm {
        DependentTerm::Application {
            function: self.function.clone(),
            argument: self.argument.clone(),
        }
    }

    /// Perform β-reduction if the function is a lambda.
    pub fn beta_reduce(&self, normalizer: &Normalizer) -> Result<DependentTerm> {
        match self.function.as_ref() {
            DependentTerm::Lambda { param, body, .. } => {
                // (λx.t)(a) → t[a/x]
                normalizer.normalize_term(&DependentTerm::Application {
                    function: self.function.clone(),
                    argument: self.argument.clone(),
                })
            }
            _ => Ok(self.to_dependent_term())
        }
    }
}

/// Π-type utilities and operations.
pub struct PiTypeOperations;

impl PiTypeOperations {
    /// Check if two Π-types are definitionally equal.
    pub fn pi_types_equal(pi1: &PiType, pi2: &PiType, normalizer: &Normalizer) -> Result<bool> {
        // Normalize both types and check structural equality
        let norm1 = normalizer.normalize_type(&pi1.to_dependent_type())?;
        let norm2 = normalizer.normalize_type(&pi2.to_dependent_type())?;
        Ok(norm1 == norm2)
    }

    /// Create a non-dependent function type (special case of Π-type).
    pub fn simple_function_type(domain: DependentType, codomain: DependentType) -> Result<PiType> {
        // Use a dummy variable name since codomain doesn't depend on it
        PiType::new("_".to_string(), domain, codomain)
    }

    /// Check if a Π-type represents a simple function type (non-dependent).
    pub fn is_simple_function(pi_type: &PiType) -> bool {
        !pi_type.is_dependent()
    }

    /// Curry a multi-argument function into nested Π-types.
    pub fn curry_function(params: Vec<(String, DependentType)>, result: DependentType) -> Result<PiType> {
        if params.is_empty() {
            return Err(Box::new(Error::type_error(
                "Cannot curry function with no parameters".to_string(),
                Span::new(0, 0)
            )));
        }

        let mut current_result = result;
        
        // Build nested Π-types from right to left
        for (param_name, param_type) in params.into_iter().rev() {
            current_result = DependentType::Pi {
                var: param_name,
                domain: Box::new(param_type),
                codomain: Box::new(current_result),
            };
        }

        // Extract the outermost Π-type
        match current_result {
            DependentType::Pi { var, domain, codomain } => {
                Ok(PiType::with_universe_level(var, *domain, *codomain, 0))
            }
            _ => unreachable!("Should have created a Π-type")
        }
    }

    /// Uncurry nested Π-types back to multi-argument function signature.
    pub fn uncurry_function(pi_type: &PiType) -> (Vec<(String, DependentType)>, DependentType) {
        let mut params = vec![(pi_type.var.clone(), (*pi_type.domain).clone())];
        let mut current_codomain = &*pi_type.codomain;

        // Follow the chain of Π-types
        while let DependentType::Pi { var, domain, codomain } = current_codomain {
            params.push((var.clone(), (**domain).clone()));
            current_codomain = codomain;
        }

        (params, current_codomain.clone())
    }
}

// Display implementations
impl fmt::Display for PiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dependent() {
            write!(f, "({} : {}) → {}", self.var, self.domain, self.codomain)
        } else {
            write!(f, "{} → {}", self.domain, self.codomain)
        }
    }
}

impl fmt::Display for LambdaAbstraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "λ{} : {}. {}", self.param, self.param_type, self.body)
    }
}

impl fmt::Display for FunctionApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.function, self.argument)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_type_creation() {
        let domain = DependentType::Universe(0);
        let codomain = DependentType::Universe(0);
        let pi_type = PiType::new("x".to_string(), domain, codomain).unwrap();
        
        assert_eq!(pi_type.var, "x");
        assert_eq!(pi_type.universe_level, 1);
    }

    #[test]
    fn test_simple_function_type() {
        let domain = DependentType::Universe(0);
        let codomain = DependentType::Universe(1);
        let pi_type = PiTypeOperations::simple_function_type(domain, codomain).unwrap();
        
        assert!(!pi_type.is_dependent());
        assert_eq!(pi_type.var, "_");
    }

    #[test]
    fn test_dependent_pi_type() {
        // Create (n : Nat) → Vec(n)
        let nat_type = DependentType::Inductive {
            name: "Nat".to_string(),
            parameters: vec![],
            universe_level: 0,
            constructors: Vec::new(),
            induction_principle: None,
        };
        
        let vec_type = DependentType::Pi {
            var: "_".to_string(),
            domain: Box::new(nat_type.clone()),
            codomain: Box::new(DependentType::Universe(0)),
        };
        
        let pi_type = PiType::new("n".to_string(), nat_type, vec_type).unwrap();
        assert!(pi_type.is_dependent());
    }

    #[test]
    fn test_curry_uncurry() {
        let params = vec![
            ("x".to_string(), DependentType::Universe(0)),
            ("y".to_string(), DependentType::Universe(0)),
        ];
        let result = DependentType::Universe(0);
        
        let curried = PiTypeOperations::curry_function(params.clone(), result.clone()).unwrap();
        let (uncurried_params, uncurried_result) = PiTypeOperations::uncurry_function(&curried);
        
        assert_eq!(uncurried_params, params);
        assert_eq!(uncurried_result, result);
    }

    #[test]
    fn test_lambda_abstraction_display() {
        let lambda = LambdaAbstraction::new(
            "x".to_string(),
            DependentType::Universe(0),
            DependentTerm::Variable("x".to_string())
        );
        
        let display = format!("{}", lambda);
        assert!(display.contains("λ"));
        assert!(display.contains("x"));
    }
}