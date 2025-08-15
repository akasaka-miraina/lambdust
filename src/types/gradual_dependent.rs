//! Gradual dependent type system for Lambdust.
//!
//! This module integrates dependent types with gradual typing, allowing
//! smooth transitions between dependent and dynamic typing modes.
//!
//! # Theoretical Foundation
//!
//! The gradual dependent type system is based on:
//! - **Gradual Dependent Types** (Tanter & Wolff)
//! - **AGT (Abstracting Gradual Types)** framework
//! - **Consistent subtyping** with dependent types
//! - **Type-level dynamic values** (⋆ at type level)

use super::{Type, TypeVar, gradual, dependent_bridge::*};
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Gradual dependent type extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradualDependentType {
    /// Type-level dynamic: ⋆ (star)
    /// Represents unknown or dynamic computation at type level
    TypeDynamic,
    
    /// Partially known Π-type: (x : A) → B(x)
    /// where A or B might contain TypeDynamic
    GradualPi {
        var: String,
        domain: Box<Type>,
        codomain: Box<Type>,
    },
    
    /// Partially known Σ-type: (x : A) × B(x)
    /// where A or B might contain TypeDynamic  
    GradualSigma {
        var: String,
        first: Box<Type>,
        second: Box<Type>,
    },
    
    /// Refinement type with gradual predicate: {x : T | P(x)}
    /// where P might be unknown (TypeDynamic)
    GradualRefinement {
        var: String,
        base_type: Box<Type>,
        predicate: GradualTerm,
    },
    
    /// Type family application with gradual indices
    GradualIndexed {
        family: String,
        indices: Vec<GradualTerm>,
    },
}

/// Terms that can appear in gradual dependent types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradualTerm {
    /// Fully known term
    Known(Term),
    
    /// Dynamic/unknown term: ⋆
    Dynamic,
    
    /// Gradual application: f(args) where f or args might be dynamic
    GradualApp {
        function: Box<GradualTerm>,
        arguments: Vec<GradualTerm>,
    },
    
    /// Gradual abstraction: λx.e where e might be dynamic
    GradualLambda {
        param: String,
        param_type: Box<Type>,
        body: Box<GradualTerm>,
    },
}

/// Consistency relation for gradual dependent types.
///
/// Extends the basic gradual consistency to handle dependent types.
pub fn gradual_dependent_consistent(t1: &Type, t2: &Type) -> bool {
    // First check basic gradual consistency
    if gradual::consistent(t1, t2) {
        return true;
    }
    
    // Then check dependent-specific consistency
    match (t1, t2) {
        // Pi types are consistent if domains and codomains are consistent
        _ if is_pi_like(t1) && is_pi_like(t2) => {
            let (dom1, cod1) = extract_pi_parts(t1);
            let (dom2, cod2) = extract_pi_parts(t2);
            gradual::consistent(&dom1, &dom2) && gradual::consistent(&cod1, &cod2)
        }
        
        // Sigma types are consistent if components are consistent
        _ if is_sigma_like(t1) && is_sigma_like(t2) => {
            let (fst1, snd1) = extract_sigma_parts(t1);
            let (fst2, snd2) = extract_sigma_parts(t2);
            gradual::consistent(&fst1, &fst2) && gradual::consistent(&snd1, &snd2)
        }
        
        // A dependent type is consistent with Dynamic
        _ if has_dependent_structure(t1) => *t2 == Type::Dynamic,
        _ if has_dependent_structure(t2) => *t1 == Type::Dynamic,
        
        _ => false,
    }
}

/// Gradual join for dependent types.
///
/// Computes the least upper bound in the gradual dependent type lattice.
pub fn gradual_dependent_join(t1: &Type, t2: &Type) -> Option<Type> {
    if !gradual_dependent_consistent(t1, t2) {
        return None;
    }
    
    match (t1, t2) {
        // If either is Dynamic, result is Dynamic
        (Type::Dynamic, _) | (_, Type::Dynamic) => Some(Type::Dynamic),
        
        // Join Pi types
        _ if is_pi_like(t1) && is_pi_like(t2) => {
            let (dom1, cod1) = extract_pi_parts(t1);
            let (dom2, cod2) = extract_pi_parts(t2);
            
            let joined_domain = gradual::join_types(&dom1, &dom2)?;
            let joined_codomain = gradual::join_types(&cod1, &cod2)?;
            
            Some(Type::function(vec![joined_domain], joined_codomain))
        }
        
        // Join Sigma types (pairs)
        _ if is_sigma_like(t1) && is_sigma_like(t2) => {
            let (fst1, snd1) = extract_sigma_parts(t1);
            let (fst2, snd2) = extract_sigma_parts(t2);
            
            let joined_first = gradual::join_types(&fst1, &fst2)?;
            let joined_second = gradual::join_types(&snd1, &snd2)?;
            
            Some(Type::pair(joined_first, joined_second))
        }
        
        // Fallback to regular gradual join
        _ => gradual::join_types(t1, t2),
    }
}

/// Cast insertion for gradual dependent types.
///
/// Handles runtime checks for dependent type casts.
#[derive(Debug, Clone)]
pub enum GradualDependentCast {
    /// Basic gradual cast
    Basic(gradual::Cast),
    
    /// Dependent upcast: static dependent → dynamic
    DependentUpcast {
        from: DependentType,
        to: Type,
    },
    
    /// Dependent downcast: dynamic → static dependent (with runtime check)
    DependentDowncast {
        from: Type,
        to: DependentType,
        /// Predicate to check at runtime
        check: GradualTerm,
    },
    
    /// Index cast: cast for type family indices
    IndexCast {
        family: String,
        index_casts: Vec<GradualDependentCast>,
    },
}

/// Insert casts for gradual dependent type transitions.
pub fn insert_gradual_dependent_cast(source: &Type, target: &Type) -> GradualDependentCast {
    // Check if this is a dependent type situation
    if has_dependent_structure(source) || has_dependent_structure(target) {
        match (source, target) {
            // Upcast from dependent to dynamic
            (dep, Type::Dynamic) if has_dependent_structure(dep) => {
                GradualDependentCast::DependentUpcast {
                    from: extract_dependent_type(dep),
                    to: target.clone(),
                }
            }
            
            // Downcast from dynamic to dependent
            (Type::Dynamic, dep) if has_dependent_structure(dep) => {
                GradualDependentCast::DependentDowncast {
                    from: source.clone(),
                    to: extract_dependent_type(dep),
                    check: generate_runtime_check(dep),
                }
            }
            
            // Other cases fall back to basic cast
            _ => GradualDependentCast::Basic(gradual::insert_cast(source, target)),
        }
    } else {
        // No dependent types involved
        GradualDependentCast::Basic(gradual::insert_cast(source, target))
    }
}

/// Normalization of gradual dependent types.
///
/// Performs type-level computation while handling dynamic parts.
pub fn normalize_gradual_dependent(type_: &Type) -> Result<Type> {
    match type_ {
        Type::Dynamic => Ok(Type::Dynamic),
        
        Type::Function { params, return_type } => {
            // Try to normalize function types
            let normalized_params: Result<Vec<Type>> = params.iter()
                .map(normalize_gradual_dependent)
                .collect();
            let normalized_return = normalize_gradual_dependent(return_type)?;
            
            Ok(Type::function(normalized_params?, normalized_return))
        }
        
        Type::Pair(a, b) => {
            let normalized_a = normalize_gradual_dependent(a)?;
            let normalized_b = normalize_gradual_dependent(b)?;
            Ok(Type::pair(normalized_a, normalized_b))
        }
        
        // For dependent types with dynamic parts, partial normalization
        _ if has_dependent_structure(type_) => {
            // Perform whatever normalization is possible
            // Dynamic parts remain dynamic
            Ok(type_.clone()) // Simplified for now
        }
        
        // Base types need no normalization
        _ => Ok(type_.clone()),
    }
}

/// Progressive precision enhancement.
///
/// Converts gradual dependent types to more precise static types
/// as more information becomes available.
pub fn refine_gradual_dependent(
    original: &Type, 
    additional_info: &HashMap<String, Type>
) -> Type {
    match original {
        Type::Dynamic => {
            // Try to infer a more specific type based on context
            Type::Dynamic // Simplified - would use sophisticated inference
        }
        
        Type::Function { params, return_type } => {
            let refined_params = params.iter()
                .map(|p| refine_gradual_dependent(p, additional_info))
                .collect();
            let refined_return = refine_gradual_dependent(return_type, additional_info);
            
            Type::function(refined_params, refined_return)
        }
        
        Type::Variable(var) => {
            // Try to resolve variable from additional info
            additional_info.get(&format!("var_{}", var.id))
                .cloned()
                .unwrap_or_else(|| original.clone())
        }
        
        _ => original.clone(),
    }
}

// ============= HELPER FUNCTIONS =============

/// Check if a type has dependent structure.
fn has_dependent_structure(type_: &Type) -> bool {
    match type_ {
        Type::Function { .. } => true, // Could be Pi type
        Type::Pair(..) => true,        // Could be Sigma type
        Type::Variable(_) => true,     // Could be dependent variable
        _ => false,
    }
}

/// Check if a type is Pi-like (function type).
fn is_pi_like(type_: &Type) -> bool {
    matches!(type_, Type::Function { .. })
}

/// Check if a type is Sigma-like (pair type).
fn is_sigma_like(type_: &Type) -> bool {
    matches!(type_, Type::Pair(..) | Type::Vector(_) | Type::List(_))
}

/// Extract Pi type components.
fn extract_pi_parts(type_: &Type) -> (Type, Type) {
    match type_ {
        Type::Function { params, return_type } => {
            let domain = if params.is_empty() {
                Type::Unit
            } else if params.len() == 1 {
                params[0].clone()
            } else {
                // Multiple parameters - create product type
                Type::Pair(
                    Box::new(params[0].clone()),
                    Box::new(params[1].clone()) // Simplified
                )
            };
            (domain, (**return_type).clone())
        }
        _ => (Type::Dynamic, Type::Dynamic),
    }
}

/// Extract Sigma type components.
fn extract_sigma_parts(type_: &Type) -> (Type, Type) {
    match type_ {
        Type::Pair(a, b) => ((**a).clone(), (**b).clone()),
        Type::List(t) => ((**t).clone(), Type::list((**t).clone())),
        Type::Vector(t) => ((**t).clone(), Type::vector((**t).clone())),
        _ => (Type::Dynamic, Type::Dynamic),
    }
}

/// Extract dependent type representation.
fn extract_dependent_type(type_: &Type) -> DependentType {
    match type_ {
        Type::Function { params, return_type } if params.len() == 1 => {
            DependentType::Pi {
                var: "_x".to_string(),
                domain: Box::new(params[0].clone().into()),
                codomain: Box::new((*return_type).clone().into()),
            }
        }
        Type::Pair(a, b) => {
            DependentType::Sigma {
                var: "_x".to_string(),
                first: Box::new((**a).clone().into()),
                second: Box::new((**b).clone().into()),
            }
        }
        _ => DependentType::Universe(0), // Fallback
    }
}

/// Generate runtime check for dependent type cast.
fn generate_runtime_check(type_: &Type) -> GradualTerm {
    match type_ {
        Type::Function { params, .. } => {
            // Check arity and parameter types
            GradualTerm::Known(Term::bool(true)) // Simplified
        }
        Type::Pair(..) => {
            // Check that value is actually a pair
            GradualTerm::Known(Term::bool(true)) // Simplified
        }
        _ => GradualTerm::Dynamic,
    }
}

/// Type-level computation with gradual elements.
pub fn compute_gradual_type(term: &GradualTerm) -> Result<Type> {
    match term {
        GradualTerm::Known(t) => {
            // Use standard dependent type checker
            let mut checker = DependentTypeChecker::new();
            checker.check_term(t, None)
        }
        
        GradualTerm::Dynamic => Ok(Type::Dynamic),
        
        GradualTerm::GradualApp { function, arguments } => {
            let func_type = compute_gradual_type(function)?;
            
            // If function is dynamic, result is dynamic
            if func_type == Type::Dynamic {
                return Ok(Type::Dynamic);
            }
            
            // Otherwise try to apply
            match func_type {
                Type::Function { params, return_type } => {
                    if params.len() == arguments.len() {
                        Ok((*return_type).clone())
                    } else {
                        Ok(Type::Dynamic)
                    }
                }
                _ => Ok(Type::Dynamic),
            }
        }
        
        GradualTerm::GradualLambda { param_type, body, .. } => {
            let body_type = compute_gradual_type(body)?;
            Ok(Type::function(vec![(**param_type).clone()], body_type))
        }
    }
}

/// Error recovery for dependent type checking.
///
/// When dependent type checking fails, fall back to gradual typing.
pub fn recover_with_gradual(error: &Error, original_type: &Type) -> Type {
    match error {
        Error::TypeError { .. } => {
            // Convert to gradual type
            gradual::approximate_type(original_type)
        }
        _ => Type::Dynamic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradual_dependent_consistency() {
        let pi_type = Type::function(vec![Type::Number], Type::String);
        let dynamic = Type::Dynamic;
        
        assert!(gradual_dependent_consistent(&pi_type, &dynamic));
        assert!(gradual_dependent_consistent(&dynamic, &pi_type));
    }
    
    #[test]
    fn test_gradual_dependent_join() {
        let pi1 = Type::function(vec![Type::Number], Type::String);
        let pi2 = Type::function(vec![Type::Dynamic], Type::String);
        
        let joined = gradual_dependent_join(&pi1, &pi2).unwrap();
        match joined {
            Type::Function { params, return_type } => {
                assert_eq!(params[0], Type::Dynamic);
                assert_eq!(*return_type, Type::String);
            }
            _ => panic!("Expected function type"),
        }
    }
    
    #[test]
    fn test_dependent_cast_insertion() {
        let dep_type = Type::function(vec![Type::Number], Type::String);
        let dynamic = Type::Dynamic;
        
        let cast = insert_gradual_dependent_cast(&dep_type, &dynamic);
        assert!(matches!(cast, GradualDependentCast::DependentUpcast { .. }));
        
        let cast = insert_gradual_dependent_cast(&dynamic, &dep_type);
        assert!(matches!(cast, GradualDependentCast::DependentDowncast { .. }));
    }
    
    #[test]
    fn test_gradual_term_computation() {
        let known_term = GradualTerm::Known(Term::Int(42));
        let computed = compute_gradual_type(&known_term).unwrap();
        assert_eq!(computed, Type::Number);
        
        let dynamic_term = GradualTerm::Dynamic;
        let computed = compute_gradual_type(&dynamic_term).unwrap();
        assert_eq!(computed, Type::Dynamic);
    }
    
    #[test]
    fn test_type_refinement() {
        let gradual_func = Type::function(vec![Type::Dynamic], Type::String);
        let mut context = HashMap::new();
        // Would add refinement information to context
        
        let refined = refine_gradual_dependent(&gradual_func, &context);
        // In a full implementation, this would show more specific types
        assert!(matches!(refined, Type::Function { .. }));
    }
}