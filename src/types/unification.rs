//! Type unification for Hindley-Milner type inference.
//!
//! This module implements the unification algorithm that finds the most general
//! unifier (MGU) for two types, which is essential for type inference.

#![allow(missing_docs)]

use super::{Type, TypeVar, Substitution, Row, Effect};
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashSet;

/// Unification error types.
#[derive(Debug, Clone)]
pub enum UnificationError {
    /// Types cannot be unified (type mismatch)
    TypeMismatch {
        expected: Type,
        actual: Type,
        span: Option<Span>,
    },
    /// Infinite type (occurs check failure)
    InfiniteType {
        var: TypeVar,
        type_: Type,
        span: Option<Span>,
    },
    /// Kind mismatch in type constructors
    KindMismatch {
        expected_kind: String,
        actual_kind: String,
        span: Option<Span>,
    },
    /// Arity mismatch in function types
    ArityMismatch {
        expected: usize,
        actual: usize,
        span: Option<Span>,
    },
    /// Row unification failure
    RowMismatch {
        field: String,
        expected: Type,
        actual: Type,
        span: Option<Span>,
    },
}

/// The main unification algorithm.
///
/// This implements Algorithm W's unification component, finding the most
/// general unifier (MGU) for two types.
pub struct Unifier {
    /// Substitution accumulated during unification
    substitution: Substitution,
}

impl Unifier {
    /// Creates a new unifier.
    pub fn new() -> Self {
        Self {
            substitution: Substitution::empty(),
        }
    }
    
    /// Creates a unifier with an initial substitution.
    pub fn with_substitution(substitution: Substitution) -> Self {
        Self { substitution }
    }
    
    /// Gets the current substitution.
    pub fn substitution(&self) -> &Substitution {
        &self.substitution
    }
    
    /// Unifies two types, returning the most general unifier.
    pub fn unify(&mut self, type1: &Type, type2: &Type, span: Option<Span>) -> Result<()> {
        // Apply current substitution to both types
        let t1 = self.substitution.apply_to_type(type1);
        let t2 = self.substitution.apply_to_type(type2);
        
        self.unify_types(&t1, &t2, span)
    }
    
    /// Internal unification implementation.
    fn unify_types(&mut self, type1: &Type, type2: &Type, span: Option<Span>) -> Result<()> {
        match (type1, type2) {
            // Same types unify trivially
            (t1, t2) if t1 == t2 => Ok(()),
            
            // Variable unification
            (Type::Variable(var), type_) | (type_, Type::Variable(var)) => {
                self.unify_variable(var, type_, span)
            }
            
            // Base type unification
            (Type::Number, Type::Number) |
            (Type::String, Type::String) |
            (Type::Symbol, Type::Symbol) |
            (Type::Boolean, Type::Boolean) |
            (Type::Char, Type::Char) |
            (Type::Unit, Type::Unit) => Ok(()),
            
            // Compound type unification
            (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
                self.unify_types(a1, a2, span)?;
                self.unify_types(b1, b2, span)
            }
            
            (Type::List(t1), Type::List(t2)) |
            (Type::Vector(t1), Type::Vector(t2)) => {
                self.unify_types(t1, t2, span)
            }
            
            (Type::Function { params: p1, return_type: r1 },
             Type::Function { params: p2, return_type: r2 }) => {
                self.unify_function_types(p1, r1, p2, r2, span)
            }
            
            // Type constructor unification
            (Type::Constructor { name: n1, kind: k1 },
             Type::Constructor { name: n2, kind: k2 }) => {
                if n1 == n2 && k1 == k2 {
                    Ok(())
                } else {
                    Err(self.type_mismatch_error(type1.clone()), type2.clone()), span))
                }
            }
            
            (Type::Application { constructor: c1, argument: a1 },
             Type::Application { constructor: c2, argument: a2 }) => {
                self.unify_types(c1, c2, span)?;
                self.unify_types(a1, a2, span)
            }
            
            // Polymorphic type unification
            (Type::Forall { vars: v1, body: b1 },
             Type::Forall { vars: v2, body: b2 }) => {
                self.unify_forall_types(v1, b1, v2, b2, span)
            }
            
            (Type::Exists { vars: v1, body: b1 },
             Type::Exists { vars: v2, body: b2 }) => {
                self.unify_exists_types(v1, b1, v2, b2, span)
            }
            
            // Constrained type unification
            (Type::Constrained { constraints: _c1, type_: t1 },
             Type::Constrained { constraints: _c2, type_: t2 }) => {
                // TODO: Implement constraint unification
                self.unify_types(t1, t2, span)
            }
            
            // Gradual typing
            (Type::Dynamic, _) | (_, Type::Dynamic) => {
                // Dynamic types unify with anything
                Ok(())
            }
            
            (Type::Unknown, type_) | (type_, Type::Unknown) => {
                // Unknown types become the other type
                if let Type::Unknown = type_ {
                    Ok(()) // Both unknown
                } else {
                    // Replace unknown with concrete type
                    // This would require tracking unknowns as variables
                    Ok(())
                }
            }
            
            // Effect type unification
            (Type::Effectful { input: i1, effects: e1, output: o1 },
             Type::Effectful { input: i2, effects: e2, output: o2 }) => {
                self.unify_types(i1, i2, span)?;
                self.unify_types(o1, o2, span)?;
                self.unify_effects(e1, e2, span)
            }
            
            // Row type unification
            (Type::Record(row1), Type::Record(row2)) |
            (Type::Variant(row1), Type::Variant(row2)) => {
                self.unify_rows(row1, row2, span)
            }
            
            // Recursive type unification
            (Type::Recursive { var: v1, body: b1 },
             Type::Recursive { var: v2, body: b2 }) => {
                // Unify by alpha-renaming v2 to v1 in b2
                let renamed_b2 = self.alpha_rename(b2, v2, v1);
                self.unify_types(b1, &renamed_b2, span)
            }
            
            // Type mismatch
            _ => Err(self.type_mismatch_error(type1.clone()), type2.clone()), span)),
        }
    }
    
    /// Unifies a type variable with a type.
    fn unify_variable(&mut self, var: &TypeVar, type_: &Type, span: Option<Span>) -> Result<()> {
        // Check if variable is already bound
        if let Some(bound_type) = self.substitution.get(var).clone())() {
            return self.unify_types(&bound_type, type_, span);
        }
        
        // Occurs check: prevent infinite types
        if type_.contains_var(var) {
            return Err(Box::new(Error::type_error(
                format!("Infinite type: {var} occurs in {type_}"),
                span.unwrap_or(Span::new(0, 0)),
            ));
        }
        
        // Create new substitution and compose with existing one
        let new_subst = Substitution::single(var.clone()), type_.clone());
        self.substitution = self.substitution.compose(&new_subst);
        
        Ok(())
    }
    
    /// Unifies function types.
    fn unify_function_types(
        &mut self,
        params1: &[Type],
        return1: &Type,
        params2: &[Type],
        return2: &Type,
        span: Option<Span>,
    ) -> Result<()> {
        // Check arity
        if params1.len() != params2.len() {
            return Err(Box::new(Error::type_error(
                format!(
                    "Function arity mismatch: expected {} parameters, got {}",
                    params1.len(),
                    params2.len()
                ),
                span.unwrap_or(Span::new(0, 0)),
            ));
        }
        
        // Unify parameter types
        for (p1, p2) in params1.iter().zip(params2.iter()) {
            self.unify_types(p1, p2, span)?;
        }
        
        // Unify return types
        self.unify_types(return1, return2, span)
    }
    
    /// Unifies forall types by alpha-renaming.
    fn unify_forall_types(
        &mut self,
        vars1: &[TypeVar],
        body1: &Type,
        vars2: &[TypeVar],
        body2: &Type,
        span: Option<Span>,
    ) -> Result<()> {
        if vars1.len() != vars2.len() {
            return Err(self.type_mismatch_error(
                Type::Forall { vars: vars1.to_vec(), body: Box::new(body1.clone()) },
                Type::Forall { vars: vars2.to_vec(), body: Box::new(body2.clone()) },
                span,
            ));
        }
        
        // Alpha-rename vars2 to vars1 in body2
        let mut renamed_body2 = body2.clone());
        for (v1, v2) in vars1.iter().zip(vars2.iter()) {
            renamed_body2 = self.alpha_rename(&renamed_body2, v2, v1);
        }
        
        self.unify_types(body1, &renamed_body2, span)
    }
    
    /// Unifies existential types.
    fn unify_exists_types(
        &mut self,
        vars1: &[TypeVar],
        body1: &Type,
        vars2: &[TypeVar],
        body2: &Type,
        span: Option<Span>,
    ) -> Result<()> {
        // Similar to forall unification
        self.unify_forall_types(vars1, body1, vars2, body2, span)
    }
    
    /// Unifies effect lists.
    fn unify_effects(&mut self, effects1: &[Effect], effects2: &[Effect], span: Option<Span>) -> Result<()> {
        // For now, require exact match of effects
        // TODO: Implement proper effect unification with subtyping
        if effects1.len() != effects2.len() {
            return Err(Box::new(Error::type_error(
                format!(
                    "Effect arity mismatch: {} vs {}",
                    effects1.len(),
                    effects2.len()
                ),
                span.unwrap_or(Span::new(0, 0)),
            ));
        }
        
        for (e1, e2) in effects1.iter().zip(effects2.iter()) {
            self.unify_single_effect(e1, e2, span)?;
        }
        
        Ok(())
    }
    
    /// Unifies a single effect.
    fn unify_single_effect(&mut self, effect1: &Effect, effect2: &Effect, span: Option<Span>) -> Result<()> {
        match (effect1, effect2) {
            (Effect::IO, Effect::IO) => Ok(()),
            (Effect::State(t1), Effect::State(t2)) => self.unify_types(t1, t2, span),
            (Effect::Exception(t1), Effect::Exception(t2)) => self.unify_types(t1, t2, span),
            (Effect::Custom(n1), Effect::Custom(n2)) if n1 == n2 => Ok(()),
            _ => Err(Box::new(Error::type_error(
                format!("Effect mismatch: {effect1:?} vs {effect2:?}"),
                span.unwrap_or(Span::new(0, 0)),
            )),
        }
    }
    
    /// Unifies row types.
    fn unify_rows(&mut self, row1: &Row, row2: &Row, span: Option<Span>) -> Result<()> {
        // Check that common fields have the same types
        for (field, type1) in &row1.fields {
            if let Some(type2) = row2.fields.get(field) {
                self.unify_types(type1, type2, span)?;
            }
        }
        
        // Handle row variables and missing fields
        let fields1_only: HashSet<_> = row1.fields.keys()
            .filter(|k| !row2.fields.contains_key(*k))
            .collect();
        let fields2_only: HashSet<_> = row2.fields.keys()
            .filter(|k| !row1.fields.contains_key(*k))
            .collect();
        
        match (&row1.rest, &row2.rest) {
            (None, None) => {
                // Both rows are closed - they must have exactly the same fields
                if !fields1_only.is_empty() || !fields2_only.is_empty() {
                    return Err(Box::new(Error::type_error(
                        "Row field mismatch in closed rows".to_string(),
                        span.unwrap_or(Span::new(0, 0)),
                    ));
                }
            }
            (Some(var), None) => {
                // row1 is open, row2 is closed
                // var must unify with a row containing only fields2_only
                let remaining_fields: std::collections::HashMap<_, _> = fields2_only
                    .into_iter()
                    .map(|k| (k.clone()), row2.fields[k].clone()))
                    .collect();
                let remaining_row = Row::closed(remaining_fields);
                self.unify_variable(var, &Type::Record(remaining_row), span)?;
            }
            (None, Some(var)) => {
                // row1 is closed, row2 is open
                let remaining_fields: std::collections::HashMap<_, _> = fields1_only
                    .into_iter()
                    .map(|k| (k.clone()), row1.fields[k].clone()))
                    .collect();
                let remaining_row = Row::closed(remaining_fields);
                self.unify_variable(var, &Type::Record(remaining_row), span)?;
            }
            (Some(var1), Some(var2)) => {
                // Both rows are open
                if var1 == var2 {
                    // Same variable - just check that extra fields match
                    if fields1_only != fields2_only {
                        return Err(Box::new(Error::type_error(
                            "Row variable field mismatch".to_string(),
                            span.unwrap_or(Span::new(0, 0)),
                        ));
                    }
                } else {
                    // Different variables - need to create new row with shared extension
                    // This is complex and requires careful handling
                    // For now, just unify the variables directly
                    self.unify_variable(var1, &Type::Variable(var2.clone()), span)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Alpha-renames a type variable in a type.
    fn alpha_rename(&self, type_: &Type, old_var: &TypeVar, new_var: &TypeVar) -> Type {
        if old_var == new_var {
            return type_.clone());
        }
        
        let subst = Substitution::single(old_var.clone()), Type::Variable(new_var.clone()));
        subst.apply_to_type(type_)
    }
    
    /// Creates a type mismatch error.
    fn type_mismatch_error(&self, expected: Type, actual: Type, span: Option<Span>) -> Error {
        Error::type_error(
            format!("Type mismatch: expected {expected}, got {actual}"),
            span.unwrap_or(Span::new(0, 0)),
        )
    }
    
    /// Extracts the final substitution.
    pub fn into_substitution(self) -> Substitution {
        self.substitution.normalize()
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to unify two types.
pub fn unify(type1: &Type, type2: &Type, span: Option<Span>) -> Result<Substitution> {
    let mut unifier = Unifier::new();
    unifier.unify(type1, type2, span)?;
    Ok(unifier.into_substitution())
}

/// Convenience function to unify multiple type pairs.
pub fn unify_many(pairs: &[(Type, Type)], span: Option<Span>) -> Result<Substitution> {
    let mut unifier = Unifier::new();
    
    for (type1, type2) in pairs {
        unifier.unify(type1, type2, span)?;
    }
    
    Ok(unifier.into_substitution())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TypeVar, Type};

    #[test]
    fn test_unify_identical_types() {
        let result = unify(&Type::Number, &Type::Number, None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_unify_variable_with_type() {
        let var = TypeVar::with_id(1);
        let result = unify(&Type::Variable(var.clone()), &Type::Number, None);
        
        assert!(result.is_ok());
        let subst = result.unwrap();
        assert_eq!(subst.apply_to_type(&Type::Variable(var)), Type::Number);
    }

    #[test]
    fn test_unify_function_types() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(2);
        
        let func1 = Type::function(vec![Type::Variable(var1.clone())], Type::Variable(var2.clone()));
        let func2 = Type::function(vec![Type::Number], Type::String);
        
        let result = unify(&func1, &func2, None);
        assert!(result.is_ok());
        
        let subst = result.unwrap();
        assert_eq!(subst.apply_to_type(&Type::Variable(var1)), Type::Number);
        assert_eq!(subst.apply_to_type(&Type::Variable(var2)), Type::String);
    }

    #[test]
    fn test_occurs_check() {
        let var = TypeVar::with_id(1);
        let recursive_type = Type::list(Type::Variable(var.clone()));
        
        let result = unify(&Type::Variable(var), &recursive_type, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_type_mismatch() {
        let result = unify(&Type::Number, &Type::String, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_function_arity_mismatch() {
        let func1 = Type::function(vec![Type::Number], Type::Number);
        let func2 = Type::function(vec![Type::Number, Type::String], Type::Number);
        
        let result = unify(&func1, &func2, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_pair_types() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(2);
        
        let pair1 = Type::pair(Type::Variable(var1.clone()), Type::Variable(var2.clone()));
        let pair2 = Type::pair(Type::Number, Type::String);
        
        let result = unify(&pair1, &pair2, None);
        assert!(result.is_ok());
        
        let subst = result.unwrap();
        assert_eq!(subst.apply_to_type(&Type::Variable(var1)), Type::Number);
        assert_eq!(subst.apply_to_type(&Type::Variable(var2)), Type::String);
    }

    #[test]
    fn test_unify_with_dynamic() {
        let result = unify(&Type::Dynamic, &Type::Number, None);
        assert!(result.is_ok());
        
        let result2 = unify(&Type::String, &Type::Dynamic, None);
        assert!(result2.is_ok());
    }
}