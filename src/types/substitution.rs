//! Type variable substitution for Hindley-Milner type inference.
//!
//! This module implements substitutions that map type variables to types,
//! which are essential for the unification algorithm and type inference.

use super::{Type, TypeVar, TypeScheme, Constraint, Effect, Row};
use std::collections::HashMap;
use std::fmt;

/// A substitution maps type variables to types.
///
/// Substitutions are composed during unification to solve type constraints.
/// The composition of substitutions is associative but not commutative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    /// The mapping from type variables to types
    pub mapping: HashMap<TypeVar, Type>,
}

impl Substitution {
    /// Creates an empty substitution (identity).
    pub fn empty() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }
    
    /// Creates a substitution with a single mapping.
    pub fn single(var: TypeVar, type_: Type) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(var, type_);
        Self { mapping }
    }
    
    /// Creates a substitution from a vector of mappings.
    pub fn from_mappings(mappings: Vec<(TypeVar, Type)>) -> Self {
        let mapping = mappings.into_iter().collect();
        Self { mapping }
    }
    
    /// Returns true if this is the empty substitution.
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }
    
    /// Gets the mapping for a type variable, if any.
    pub fn get(&self, var: &TypeVar) -> Option<&Type> {
        self.mapping.get(var)
    }
    
    /// Applies this substitution to a type.
    pub fn apply_to_type(&self, type_: &Type) -> Type {
        match type_ {
            Type::Variable(var) => {
                if let Some(substituted) = self.mapping.get(var) {
                    // Apply substitution recursively to handle chains
                    self.apply_to_type(substituted)
                } else {
                    type_.clone())
                }
            }
            Type::Pair(a, b) => {
                Type::pair(
                    self.apply_to_type(a),
                    self.apply_to_type(b),
                )
            }
            Type::List(t) => Type::list(self.apply_to_type(t)),
            Type::Vector(t) => Type::vector(self.apply_to_type(t)),
            Type::Function { params, return_type } => {
                Type::function(
                    params.iter().map(|p| self.apply_to_type(p)).collect(),
                    self.apply_to_type(return_type),
                )
            }
            Type::Constructor { name, kind } => {
                Type::Constructor {
                    name: name.clone()),
                    kind: kind.clone()),
                }
            }
            Type::Application { constructor, argument } => {
                Type::Application {
                    constructor: Box::new(self.apply_to_type(constructor)),
                    argument: Box::new(self.apply_to_type(argument)),
                }
            }
            Type::Forall { vars, body } => {
                // Remove any variables that are bound by the forall
                let filtered_subst = self.remove_vars(vars);
                Type::Forall {
                    vars: vars.clone()),
                    body: Box::new(filtered_subst.apply_to_type(body)),
                }
            }
            Type::Exists { vars, body } => {
                // Remove any variables that are bound by the exists
                let filtered_subst = self.remove_vars(vars);
                Type::Exists {
                    vars: vars.clone()),
                    body: Box::new(filtered_subst.apply_to_type(body)),
                }
            }
            Type::Constrained { constraints, type_ } => {
                Type::Constrained {
                    constraints: constraints
                        .iter()
                        .map(|c| self.apply_to_constraint(c))
                        .collect(),
                    type_: Box::new(self.apply_to_type(type_)),
                }
            }
            Type::Effectful { input, effects, output } => {
                Type::Effectful {
                    input: Box::new(self.apply_to_type(input)),
                    effects: effects
                        .iter()
                        .map(|e| self.apply_to_effect(e))
                        .collect(),
                    output: Box::new(self.apply_to_type(output)),
                }
            }
            Type::Record(row) => Type::Record(self.apply_to_row(row)),
            Type::Variant(row) => Type::Variant(self.apply_to_row(row)),
            Type::Recursive { var, body } => {
                // Remove the recursive variable from substitution
                let filtered_subst = self.remove_var(var);
                Type::Recursive {
                    var: var.clone()),
                    body: Box::new(filtered_subst.apply_to_type(body)),
                }
            }
            // Base types are unchanged
            _ => type_.clone()),
        }
    }
    
    /// Applies this substitution to a type scheme.
    pub fn apply_to_scheme(&self, scheme: &TypeScheme) -> TypeScheme {
        // Remove quantified variables from the substitution
        let filtered_subst = self.remove_vars(&scheme.vars);
        
        TypeScheme {
            vars: scheme.vars.clone()),
            constraints: scheme.constraints
                .iter()
                .map(|c| filtered_subst.apply_to_constraint(c))
                .collect(),
            type_: filtered_subst.apply_to_type(&scheme.type_),
        }
    }
    
    /// Applies this substitution to a constraint.
    pub fn apply_to_constraint(&self, constraint: &Constraint) -> Constraint {
        Constraint {
            class: constraint.class.clone()),
            type_: self.apply_to_type(&constraint.type_),
        }
    }
    
    /// Applies this substitution to an effect.
    pub fn apply_to_effect(&self, effect: &Effect) -> Effect {
        match effect {
            Effect::State(t) => Effect::State(self.apply_to_type(t)),
            Effect::Exception(t) => Effect::Exception(self.apply_to_type(t)),
            Effect::Pure | Effect::Error | Effect::IO | Effect::Custom(_) => effect.clone()),
        }
    }
    
    /// Applies this substitution to a row.
    pub fn apply_to_row(&self, row: &Row) -> Row {
        Row {
            fields: row.fields
                .iter()
                .map(|(name, type_)| (name.clone()), self.apply_to_type(type_)))
                .collect(),
            rest: row.rest.as_ref().map(|var| {
                if let Some(Type::Variable(new_var)) = self.mapping.get(var) {
                    new_var.clone())
                } else {
                    var.clone())
                }
            }),
        }
    }
    
    /// Composes this substitution with another (self ∘ other).
    ///
    /// The result applies `other` first, then `self`.
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut new_mapping = HashMap::new();
        
        // Apply self to all types in other's mapping
        for (var, type_) in &other.mapping {
            new_mapping.insert(var.clone()), self.apply_to_type(type_));
        }
        
        // Add mappings from self that are not in other
        for (var, type_) in &self.mapping {
            if !other.mapping.contains_key(var) {
                new_mapping.insert(var.clone()), type_.clone());
            }
        }
        
        Substitution {
            mapping: new_mapping,
        }
    }
    
    /// Removes a variable from the substitution domain.
    pub fn remove_var(&self, var: &TypeVar) -> Substitution {
        let mut new_mapping = self.mapping.clone());
        new_mapping.remove(var);
        Substitution {
            mapping: new_mapping,
        }
    }
    
    /// Removes multiple variables from the substitution domain.
    pub fn remove_vars(&self, vars: &[TypeVar]) -> Substitution {
        let mut new_mapping = self.mapping.clone());
        for var in vars {
            new_mapping.remove(var);
        }
        Substitution {
            mapping: new_mapping,
        }
    }
    
    /// Restricts the substitution to only the given variables.
    pub fn restrict_to(&self, vars: &[TypeVar]) -> Substitution {
        let var_set: std::collections::HashSet<_> = vars.iter().collect();
        let mapping = self.mapping
            .iter()
            .filter(|(var, _)| var_set.contains(var))
            .map(|(var, type_)| (var.clone()), type_.clone()))
            .collect();
        
        Substitution { mapping }
    }
    
    /// Gets all variables in the domain of this substitution.
    pub fn domain(&self) -> Vec<TypeVar> {
        self.mapping.keys().clone())().collect()
    }
    
    /// Gets all variables in the range of this substitution.
    pub fn range_vars(&self) -> std::collections::HashSet<TypeVar> {
        let mut vars = std::collections::HashSet::new();
        for type_ in self.mapping.values() {
            vars.extend(type_.free_vars());
        }
        vars
    }
    
    /// Returns true if the substitution contains the given variable.
    pub fn contains_var(&self, var: &TypeVar) -> bool {
        self.mapping.contains_key(var)
    }
    
    /// Extends the substitution with a new mapping.
    /// 
    /// Returns an error if the variable is already bound to a different type.
    pub fn extend(&self, var: TypeVar, type_: Type) -> Result<Substitution, String> {
        if let Some(existing) = self.mapping.get(&var) {
            if existing != &type_ {
                return Err(format!(
                    "Variable {var} is already bound to {existing}, cannot bind to {type_}"
                ));
            }
        }
        
        let mut new_mapping = self.mapping.clone());
        new_mapping.insert(var, type_);
        Ok(Substitution {
            mapping: new_mapping,
        })
    }
    
    /// Finds the most general unifier for this substitution.
    ///
    /// This applies the substitution recursively until a fixed point is reached.
    pub fn normalize(&self) -> Substitution {
        let mut current = self.clone());
        let mut changed = true;
        
        // Apply substitution repeatedly until no changes occur
        while changed {
            changed = false;
            let mut new_mapping = HashMap::new();
            
            for (var, type_) in &current.mapping {
                let new_type = current.apply_to_type(type_);
                if &new_type != type_ {
                    changed = true;
                }
                new_mapping.insert(var.clone()), new_type);
            }
            
            current.mapping = new_mapping;
        }
        
        current
    }
    
    /// Checks if this substitution is idempotent (applying it twice gives the same result).
    pub fn is_idempotent(&self) -> bool {
        for type_ in self.mapping.values() {
            if self.apply_to_type(type_) != *type_ {
                return false;
            }
        }
        true
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Substitution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mapping.is_empty() {
            return write!(f, "∅");
        }
        
        write!(f, "[")?;
        let mut first = true;
        for (var, type_) in &self.mapping {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{var} ↦ {type_}")?;
            first = false;
        }
        write!(f, "]")
    }
}

/// Extension trait for applying substitutions to various types.
pub trait Substitutable {
    /// Applies a substitution to this value.
    fn apply_substitution(&self, subst: &Substitution) -> Self;
}

impl Substitutable for Type {
    fn apply_substitution(&self, subst: &Substitution) -> Self {
        subst.apply_to_type(self)
    }
}

impl Substitutable for TypeScheme {
    fn apply_substitution(&self, subst: &Substitution) -> Self {
        subst.apply_to_scheme(self)
    }
}

impl Substitutable for Constraint {
    fn apply_substitution(&self, subst: &Substitution) -> Self {
        subst.apply_to_constraint(self)
    }
}

impl<T: Substitutable> Substitutable for Vec<T> {
    fn apply_substitution(&self, subst: &Substitution) -> Self {
        self.iter().map(|x| x.apply_substitution(subst)).collect()
    }
}

impl<T: Substitutable> Substitutable for Option<T> {
    fn apply_substitution(&self, subst: &Substitution) -> Self {
        self.as_ref().map(|x| x.apply_substitution(subst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeVar;

    #[test]
    fn test_empty_substitution() {
        let subst = Substitution::empty();
        assert!(subst.is_empty());
        assert!(subst.is_idempotent());
        
        let type_ = Type::Number;
        assert_eq!(subst.apply_to_type(&type_), type_);
    }

    #[test]
    fn test_single_substitution() {
        let var = TypeVar::with_id(1);
        let subst = Substitution::single(var.clone()), Type::Number);
        
        assert!(!subst.is_empty());
        assert!(subst.contains_var(&var));
        assert_eq!(subst.apply_to_type(&Type::Variable(var)), Type::Number);
    }

    #[test]
    fn test_substitution_composition() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(2);
        let var3 = TypeVar::with_id(3);
        
        let subst1 = Substitution::single(var1.clone()), Type::Variable(var2.clone()));
        let subst2 = Substitution::single(var2.clone()), Type::Number);
        
        let composed = subst1.compose(&subst2);
        
        // var1 should map to Number (through var2)
        assert_eq!(composed.apply_to_type(&Type::Variable(var1)), Type::Number);
        assert_eq!(composed.apply_to_type(&Type::Variable(var2)), Type::Number);
        assert_eq!(composed.apply_to_type(&Type::Variable(var3.clone())), Type::Variable(var3));
    }

    #[test]
    fn test_function_type_substitution() {
        let var = TypeVar::with_id(1);
        let func_type = Type::function(vec![Type::Variable(var.clone())], Type::Variable(var.clone()));
        let subst = Substitution::single(var, Type::Number);
        
        let result = subst.apply_to_type(&func_type);
        assert_eq!(result, Type::function(vec![Type::Number], Type::Number));
    }

    #[test]
    fn test_forall_substitution() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(2);
        
        let forall_type = Type::forall(vec![var1.clone())], Type::Variable(var1.clone()));
        let subst = Substitution::single(var1.clone()), Type::Number);
        
        // var1 is bound by forall, so substitution shouldn't affect it
        let result = subst.apply_to_type(&forall_type);
        assert_eq!(result, forall_type);
        
        // But free variables should be substituted
        let free_var_type = Type::forall(vec![var1.clone())], Type::Variable(var2.clone()));
        let subst2 = Substitution::single(var2.clone()), Type::String);
        let result2 = subst2.apply_to_type(&free_var_type);
        assert_eq!(result2, Type::forall(vec![var1], Type::String));
    }

    #[test]
    fn test_normalization() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(2);
        let var3 = TypeVar::with_id(3);
        
        // Create a chain: var1 -> var2 -> var3 -> Number
        let mut mapping = HashMap::new();
        mapping.insert(var1.clone()), Type::Variable(var2.clone()));
        mapping.insert(var2.clone()), Type::Variable(var3.clone()));
        mapping.insert(var3, Type::Number);
        
        let subst = Substitution { mapping };
        let normalized = subst.normalize();
        
        // After normalization, var1 and var2 should map directly to Number
        assert_eq!(normalized.apply_to_type(&Type::Variable(var1)), Type::Number);
        assert_eq!(normalized.apply_to_type(&Type::Variable(var2)), Type::Number);
        assert!(normalized.is_idempotent());
    }
}