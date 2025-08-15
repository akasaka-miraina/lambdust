//! Dependent type system for Lambdust.
//!
//! This module implements a dependent type system based on Modified Martin-Löf
//! type theory, integrated with Lambdust's gradual typing system.
//!
//! # Theoretical Foundation
//!
//! The dependent type system is built on:
//! - Π-types (dependent function types): (x : A) → B(x)
//! - Σ-types (dependent pair types): (x : A) × B(x)  
//! - Universe hierarchy: Type₀ : Type₁ : Type₂ : ...
//! - Inductive types with pattern matching
//! - Type-level computation and normalization

use super::{Type, TypeVar, Kind, Constraint, Effect};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Universe levels for type hierarchy.
pub type UniverseLevel = u32;

/// Dependent type extensions to the main Type enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependentType {
    // ============= UNIVERSE HIERARCHY =============
    
    /// Universe type (Type₀, Type₁, etc.)
    Universe(UniverseLevel),
    
    // ============= DEPENDENT TYPES =============
    
    /// Π-type (dependent function): (x : A) → B(x)
    Pi {
        /// Binding variable name
        var: String,
        /// Domain type A
        domain: Box<Type>,
        /// Codomain type B(x), potentially depending on var
        codomain: Box<Type>,
    },
    
    /// Σ-type (dependent pair): (x : A) × B(x) 
    Sigma {
        /// Binding variable name
        var: String,
        /// First component type A
        first: Box<Type>,
        /// Second component type B(x), potentially depending on var
        second: Box<Type>,
    },
    
    // ============= TYPE-LEVEL COMPUTATION =============
    
    /// Type-level application: f(args...)
    TypeApp {
        /// Type-level function
        function: Box<Type>,
        /// Type-level arguments
        arguments: Vec<Type>,
    },
    
    /// Type-level lambda: λx.T
    TypeLambda {
        /// Parameter variable
        param: String,
        /// Parameter type (or universe)
        param_type: Box<Type>,
        /// Body type expression
        body: Box<Type>,
    },
    
    // ============= INDEXED TYPES =============
    
    /// Indexed type family: F(indices...)
    Indexed {
        /// Family name
        family: String,
        /// Type indices
        indices: Vec<Term>,
        /// Kind of the family
        kind: Kind,
    },
    
    // ============= INDUCTIVE TYPES =============
    
    /// Inductive type definition
    Inductive {
        /// Type name
        name: String,
        /// Parameters
        params: Vec<(String, Type)>,
        /// Universe level
        universe: UniverseLevel,
        /// Constructor types
        constructors: HashMap<String, Type>,
    },
    
    // ============= REFINEMENT TYPES =============
    
    /// Refinement type: {x : T | P(x)}
    Refinement {
        /// Binding variable
        var: String,
        /// Base type
        base_type: Box<Type>,
        /// Predicate (as a term)
        predicate: Term,
    },
}

/// Term-level expressions for dependent types.
///
/// These represent computations that can appear in types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// Variable reference
    Var(String),
    
    /// Lambda abstraction: λx.e
    Lambda {
        param: String,
        param_type: Box<Type>,
        body: Box<Term>,
    },
    
    /// Application: f(args...)
    App {
        function: Box<Term>,
        arguments: Vec<Term>,
    },
    
    /// Pair construction: (a, b)
    Pair(Box<Term>, Box<Term>),
    
    /// First projection: π₁(e)
    Fst(Box<Term>),
    
    /// Second projection: π₂(e)
    Snd(Box<Term>),
    
    /// Integer literal
    Int(i64),
    
    /// Boolean literal
    Bool(bool),
    
    /// String literal
    String(String),
    
    /// List construction
    List(Vec<Term>),
    
    /// Case analysis / pattern matching
    Case {
        scrutinee: Box<Term>,
        branches: Vec<(Pattern, Term)>,
    },
    
    /// Type annotation: (e : T)
    Annotated {
        term: Box<Term>,
        type_: Box<Type>,
    },
}

/// Pattern matching patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Variable pattern (always matches, binds variable)
    Var(String),
    
    /// Constructor pattern: Constructor(subpatterns...)
    Constructor {
        name: String,
        args: Vec<Pattern>,
    },
    
    /// Integer literal pattern
    Int(i64),
    
    /// Boolean literal pattern
    Bool(bool),
    
    /// Pair pattern: (p₁, p₂)
    Pair(Box<Pattern>, Box<Pattern>),
    
    /// Wildcard pattern: _ (matches anything, no binding)
    Wildcard,
}

/// Type checking context for dependent types.
#[derive(Debug, Clone)]
pub struct DependentContext {
    /// Variable bindings: var ↦ type
    vars: HashMap<String, Type>,
    /// Type definitions
    types: HashMap<String, DependentType>,
    /// Term definitions (for type-level computation)
    terms: HashMap<String, Term>,
    /// Universe constraints
    universes: HashMap<String, UniverseLevel>,
}

impl DependentContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            types: HashMap::new(),
            terms: HashMap::new(),
            universes: HashMap::new(),
        }
    }
    
    /// Add a variable binding.
    pub fn bind_var(&mut self, name: String, type_: Type) {
        self.vars.insert(name, type_);
    }
    
    /// Look up a variable type.
    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
    
    /// Add a type definition.
    pub fn define_type(&mut self, name: String, def: DependentType) {
        self.types.insert(name, def);
    }
    
    /// Look up a type definition.
    pub fn lookup_type(&self, name: &str) -> Option<&DependentType> {
        self.types.get(name)
    }
}

/// Type checker for dependent types.
pub struct DependentTypeChecker {
    /// Current typing context
    context: DependentContext,
    /// Fresh variable counter
    fresh_counter: u32,
}

impl DependentTypeChecker {
    /// Create a new type checker.
    pub fn new() -> Self {
        Self {
            context: DependentContext::new(),
            fresh_counter: 0,
        }
    }
    
    /// Generate a fresh type variable.
    pub fn fresh_var(&mut self) -> String {
        self.fresh_counter += 1;
        format!("_dep{}", self.fresh_counter)
    }
    
    /// Type check a term and return its type.
    pub fn check_term(&mut self, term: &Term, expected: Option<&Type>) -> Result<Type> {
        match term {
            Term::Var(name) => {
                self.context.lookup_var(name)
                    .cloned()
                    .ok_or_else(|| Box::new(Error::type_error(
                        format!("Unbound variable: {name}"), 
                        Span::new(0, 0)
                    )))
            }
            
            Term::Lambda { param, param_type, body } => {
                let mut body_checker = self.clone();
                body_checker.context.bind_var(param.clone(), (**param_type).clone());
                let body_type = body_checker.check_term(body, None)?;
                
                Ok(Type::Function {
                    params: vec![(**param_type).clone()],
                    return_type: Box::new(body_type),
                })
            }
            
            Term::App { function, arguments } => {
                let func_type = self.check_term(function, None)?;
                self.check_application(&func_type, arguments)
            }
            
            Term::Int(_) => Ok(Type::Number),
            Term::Bool(_) => Ok(Type::Boolean),
            Term::String(_) => Ok(Type::String),
            
            Term::Pair(a, b) => {
                let a_type = self.check_term(a, None)?;
                let b_type = self.check_term(b, None)?;
                Ok(Type::pair(a_type, b_type))
            }
            
            Term::Fst(pair) => {
                let pair_type = self.check_term(pair, None)?;
                match pair_type {
                    Type::Pair(first, _) => Ok(*first),
                    _ => Err(Box::new(Error::type_error("Expected pair type for fst".to_string(), Span::new(0, 0))))
                }
            }
            
            Term::Snd(pair) => {
                let pair_type = self.check_term(pair, None)?;
                match pair_type {
                    Type::Pair(_, second) => Ok(*second),
                    _ => Err(Box::new(Error::type_error("Expected pair type for snd".to_string(), Span::new(0, 0))))
                }
            }
            
            Term::List(elements) => {
                if elements.is_empty() {
                    Ok(Type::list(Type::Dynamic))
                } else {
                    let first_type = self.check_term(&elements[0], None)?;
                    // Check all elements have the same type
                    for elem in elements.iter().skip(1) {
                        let elem_type = self.check_term(elem, Some(&first_type))?;
                        if elem_type != first_type {
                            return Err(Box::new(Error::type_error(
                                "List elements must have the same type".to_string(),
                                Span::new(0, 0)
                            )));
                        }
                    }
                    Ok(Type::list(first_type))
                }
            }
            
            Term::Annotated { term, type_ } => {
                let inferred = self.check_term(term, Some(type_))?;
                if self.types_equal(&inferred, type_)? {
                    Ok((**type_).clone())
                } else {
                    Err(Box::new(Error::type_error(
                        format!("Type annotation mismatch: expected {type_:?}, got {inferred:?}"),
                        Span::new(0, 0)
                    )))
                }
            }
            
            Term::Case { .. } => {
                // Pattern matching type checking - complex implementation
                todo!("Pattern matching type checking")
            }
        }
    }
    
    /// Check function application.
    fn check_application(&mut self, func_type: &Type, args: &[Term]) -> Result<Type> {
        match func_type {
            Type::Function { params, return_type } => {
                if args.len() != params.len() {
                    return Err(Box::new(Error::type_error(
                        format!("Arity mismatch: expected {} args, got {}", 
                               params.len(), args.len()),
                        Span::new(0, 0)
                    )));
                }
                
                for (arg, expected_param) in args.iter().zip(params.iter()) {
                    let arg_type = self.check_term(arg, Some(expected_param))?;
                    if !self.types_equal(&arg_type, expected_param)? {
                        return Err(Box::new(Error::type_error(
                            format!("Argument type mismatch: expected {expected_param:?}, got {arg_type:?}"),
                            Span::new(0, 0)
                        )));
                    }
                }
                
                Ok((**return_type).clone())
            }
            _ => Err(Box::new(Error::type_error(
                "Cannot apply non-function type".to_string(),
                Span::new(0, 0)
            )))
        }
    }
    
    /// Check if two types are equal (with normalization).
    fn types_equal(&self, t1: &Type, t2: &Type) -> Result<bool> {
        // For now, structural equality
        // In full implementation, would normalize types first
        Ok(t1 == t2)
    }
    
    /// Normalize a type (reduce type-level computations).
    pub fn normalize_type(&self, type_: &Type) -> Result<Type> {
        // Placeholder implementation
        // Full normalization would evaluate type-level applications
        Ok(type_.clone())
    }
    
    /// Convert between regular Type and DependentType representations.
    pub fn lift_to_dependent(&self, type_: &Type) -> DependentType {
        match type_ {
            Type::Function { params, return_type } if params.len() == 1 => {
                // Convert to Π-type if dependency can be inferred
                DependentType::Pi {
                    var: "_x".to_string(),
                    domain: Box::new(params[0].clone()),
                    codomain: return_type.clone(),
                }
            }
            Type::Pair(first, second) => {
                // Convert to Σ-type if dependency can be inferred
                DependentType::Sigma {
                    var: "_x".to_string(),
                    first: first.clone(),
                    second: second.clone(),
                }
            }
            _ => {
                // Cannot lift to dependent type
                DependentType::Universe(0) // Placeholder
            }
        }
    }
}

impl Clone for DependentTypeChecker {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            fresh_counter: self.fresh_counter,
        }
    }
}

impl Default for DependentContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DependentTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ============= INTEGRATION WITH MAIN TYPE SYSTEM =============

/// Extend the main Type enum with dependent type variants.
pub fn extend_type_with_dependent(base_type: Type) -> Type {
    // This would require modifying the main Type enum
    // For now, we use the existing Type structure
    base_type
}

/// Check if a type is dependent (contains type-level computation).
pub fn is_dependent_type(type_: &Type) -> bool {
    match type_ {
        Type::Function { params, return_type } => {
            // Check if return type depends on parameters
            // This is a simplified check - full implementation would
            // track variable dependencies
            params.iter().any(is_dependent_type) || is_dependent_type(return_type)
        }
        Type::Pair(a, b) => is_dependent_type(a) || is_dependent_type(b),
        Type::List(t) | Type::Vector(t) => is_dependent_type(t),
        Type::Variable(_) => true, // Conservative assumption
        _ => false,
    }
}

/// Gradual integration: check consistency with dependent types.
pub fn dependent_consistent(dep_type: &DependentType, regular_type: &Type) -> bool {
    match (dep_type, regular_type) {
        (DependentType::Pi { domain, codomain, .. }, 
         Type::Function { params, return_type }) => {
            params.len() == 1 && 
            **domain == params[0] && 
            **codomain == **return_type
        }
        (DependentType::Sigma { first, second, .. }, 
         Type::Pair(a, b)) => {
            **first == **a && **second == **b
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_term_checking() {
        let mut checker = DependentTypeChecker::new();
        
        // Check integer literal
        let term = Term::Int(42);
        let type_ = checker.check_term(&term, None).unwrap();
        assert_eq!(type_, Type::Number);
        
        // Check boolean literal  
        let term = Term::Bool(true);
        let type_ = checker.check_term(&term, None).unwrap();
        assert_eq!(type_, Type::Boolean);
    }
    
    #[test]
    fn test_lambda_typing() {
        let mut checker = DependentTypeChecker::new();
        
        // λx:Number. x + 1 (simplified)
        let term = Term::Lambda {
            param: "x".to_string(),
            param_type: Box::new(Type::Number),
            body: Box::new(Term::Var("x".to_string())),
        };
        
        let type_ = checker.check_term(&term, None).unwrap();
        match type_ {
            Type::Function { params, return_type: _ } => {
                assert_eq!(params[0], Type::Number);
            }
            _ => panic!("Expected function type"),
        }
    }
    
    #[test]  
    fn test_dependent_type_lifting() {
        let checker = DependentTypeChecker::new();
        
        let func_type = Type::function(vec![Type::Number], Type::String);
        let dep_type = checker.lift_to_dependent(&func_type);
        
        match dep_type {
            DependentType::Pi { domain, codomain, .. } => {
                assert_eq!(**domain, Type::Number);
                assert_eq!(**codomain, Type::String);
            }
            _ => panic!("Expected Pi type"),
        }
    }
    
    #[test]
    fn test_is_dependent_type() {
        assert!(!is_dependent_type(&Type::Number));
        assert!(is_dependent_type(&Type::Variable(TypeVar::with_id(1))));
        
        let func_type = Type::function(vec![Type::Number], Type::String);
        assert!(!is_dependent_type(&func_type));
    }
}