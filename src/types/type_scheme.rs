use super::{Type, TypeVar, Constraint};
use std::collections::HashMap;

/// Type scheme for polymorphic types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    /// Quantified type variables
    pub vars: Vec<TypeVar>,
    /// Type constraints
    pub constraints: Vec<Constraint>,
    /// The actual type
    pub type_: Type,
}

impl TypeScheme {
    /// Creates a monomorphic type scheme.
    pub fn monomorphic(type_: Type) -> Self {
        Self {
            vars: Vec::new(),
            constraints: Vec::new(),
            type_,
        }
    }
    
    /// Creates a polymorphic type scheme.
    pub fn polymorphic(vars: Vec<TypeVar>, constraints: Vec<Constraint>, type_: Type) -> Self {
        Self {
            vars,
            constraints,
            type_,
        }
    }
    
    /// Instantiates this type scheme with fresh type variables.
    pub fn instantiate(&self) -> Type {
        if self.vars.is_empty() {
            return self.type_.clone();
        }
        
        // Create fresh variables for each quantified variable
        let fresh_vars: HashMap<TypeVar, Type> = self.vars
            .iter()
            .map(|var| (var.clone(), Type::fresh_var()))
            .collect();
        
        // Substitute in the type
        self.substitute_vars(&self.type_, &fresh_vars)
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn substitute_vars(&self, type_: &Type, subst: &HashMap<TypeVar, Type>) -> Type {
        match type_ {
            Type::Variable(var) => {
                subst.get(var).cloned().unwrap_or_else(|| type_.clone())
            }
            Type::Pair(a, b) => {
                Type::pair(
                    self.substitute_vars(a, subst),
                    self.substitute_vars(b, subst),
                )
            }
            Type::List(t) => Type::list(self.substitute_vars(t, subst)),
            Type::Vector(t) => Type::vector(self.substitute_vars(t, subst)),
            Type::Function { params, return_type } => {
                Type::function(
                    params.iter().map(|p| self.substitute_vars(p, subst)).collect(),
                    self.substitute_vars(return_type, subst),
                )
            }
            // Handle other cases as needed
            _ => type_.clone(), // For now, just clone for unhandled cases
        }
    }
}