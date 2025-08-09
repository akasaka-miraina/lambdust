//! Type constraint system for Hindley-Milner inference.
//!
//! This module implements type constraints that are generated during type inference
//! and solved by unification to determine the types of expressions.

#![allow(missing_docs)]

use super::{Type, TypeVar, Substitution, TypeConstraint};
use crate::diagnostics::{Error, Span};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// A constraint solver that resolves type constraints.
#[derive(Debug)]
pub struct ConstraintSolver {
    /// Constraints to solve
    constraints: Vec<TypeConstraint>,
    /// Current substitution
    substitution: Substitution,
    /// Type class instances
    instances: HashMap<String, Vec<Type>>,
    /// Errors encountered during solving
    errors: Vec<Error>,
}

/// Result of constraint solving.
#[derive(Debug, Clone)]
pub struct SolverResult {
    /// Final substitution
    pub substitution: Substitution,
    /// Unresolved constraints
    pub unresolved: Vec<TypeConstraint>,
    /// Errors encountered
    pub errors: Vec<Error>,
}

impl TypeConstraint {
    /// Applies a substitution to this constraint.
    pub fn apply_substitution(&self, subst: &Substitution) -> Self {
        match self {
            TypeConstraint::Equal { left, right, span, reason } => {
                TypeConstraint::Equal {
                    left: subst.apply_to_type(left),
                    right: subst.apply_to_type(right),
                    span: *span,
                    reason: reason.clone(),
                }
            }
            TypeConstraint::Instance { class, type_, span } => {
                TypeConstraint::Instance {
                    class: class.clone(),
                    type_: subst.apply_to_type(type_),
                    span: *span,
                }
            }
            TypeConstraint::Subtype { left, right, span } => {
                TypeConstraint::Subtype {
                    left: subst.apply_to_type(left),
                    right: subst.apply_to_type(right),
                    span: *span,
                }
            }
            TypeConstraint::Default { var, default_type, span } => {
                // Only apply substitution if variable is not bound
                if subst.contains_var(var) {
                    // Variable is bound, constraint is resolved
                    TypeConstraint::Default {
                        var: var.clone(),
                        default_type: default_type.clone(),
                        span: *span,
                    }
                } else {
                    TypeConstraint::Default {
                        var: var.clone(),
                        default_type: subst.apply_to_type(default_type),
                        span: *span,
                    }
                }
            }
            TypeConstraint::Ambiguous { vars, span } => {
                // Filter out variables that are now bound
                let unbound_vars: Vec<_> = vars
                    .iter()
                    .filter(|var| !subst.contains_var(var))
                    .cloned()
                    .collect();
                
                TypeConstraint::Ambiguous {
                    vars: unbound_vars,
                    span: *span,
                }
            }
        }
    }
    
    /// Gets all type variables mentioned in this constraint.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        match self {
            TypeConstraint::Equal { left, right, .. } => {
                let mut vars = left.free_vars();
                vars.extend(right.free_vars());
                vars
            }
            TypeConstraint::Instance { type_, .. } => type_.free_vars(),
            TypeConstraint::Subtype { left, right, .. } => {
                let mut vars = left.free_vars();
                vars.extend(right.free_vars());
                vars
            }
            TypeConstraint::Default { var, default_type, .. } => {
                let mut vars = default_type.free_vars();
                vars.insert(var.clone());
                vars
            }
            TypeConstraint::Ambiguous { vars, .. } => {
                vars.iter().cloned().collect()
            }
        }
    }
    
    /// Returns true if this constraint is resolved.
    pub fn is_resolved(&self) -> bool {
        match self {
            TypeConstraint::Equal { left, right, .. } => left == right,
            TypeConstraint::Instance { .. } => false, // Requires instance resolution
            TypeConstraint::Subtype { .. } => false, // Requires subtype checking
            TypeConstraint::Default { .. } => false, // Requires default resolution
            TypeConstraint::Ambiguous { vars, .. } => vars.is_empty(),
        }
    }
}

impl ConstraintSolver {
    /// Creates a new constraint solver.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitution: Substitution::empty(),
            instances: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    /// Creates a solver with initial constraints.
    pub fn with_constraints(constraints: Vec<TypeConstraint>) -> Self {
        Self {
            constraints,
            substitution: Substitution::empty(),
            instances: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    /// Adds a constraint to solve.
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Adds multiple constraints.
    pub fn add_constraints(&mut self, constraints: Vec<TypeConstraint>) {
        self.constraints.extend(constraints);
    }
    
    /// Adds a type class instance.
    pub fn add_instance(&mut self, class: String, type_: Type) {
        self.instances.entry(class).or_default().push(type_);
    }
    
    /// Solves all constraints, returning the result.
    pub fn solve(mut self) -> SolverResult {
        // Iteratively solve constraints until fixed point
        let mut changed = true;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 100;
        
        while changed && iteration < MAX_ITERATIONS {
            changed = false;
            iteration += 1;
            
            // Apply current substitution to all constraints
            self.constraints = self.constraints
                .into_iter()
                .map(|c| c.apply_substitution(&self.substitution))
                .collect();
            
            // Try to solve each constraint
            let mut new_constraints = Vec::new();
            let constraints_to_solve: Vec<_> = self.constraints.drain(..).collect();
            
            for constraint in constraints_to_solve {
                match self.solve_constraint(constraint) {
                    ConstraintResult::Solved(subst) => {
                        // Compose with existing substitution
                        self.substitution = self.substitution.compose(&subst);
                        changed = true;
                    }
                    ConstraintResult::Unresolved(c) => {
                        new_constraints.push(c);
                    }
                    ConstraintResult::Error(error) => {
                        self.errors.push(error);
                    }
                }
            }
            
            self.constraints = new_constraints;
        }
        
        if iteration >= MAX_ITERATIONS {
            self.errors.push(Error::internal_error(
                "Constraint solver exceeded maximum iterations"
            ));
        }
        
        // Handle default constraints for unresolved variables
        self.apply_defaults();
        
        // Check for ambiguous types
        self.check_ambiguity();
        
        SolverResult {
            substitution: self.substitution.normalize(),
            unresolved: self.constraints,
            errors: self.errors,
        }
    }
    
    /// Solves a single constraint.
    fn solve_constraint(&mut self, constraint: TypeConstraint) -> ConstraintResult {
        match constraint {
            TypeConstraint::Equal { left, right, span, reason } => {
                self.solve_equality_constraint(&left, &right, span, &reason)
            }
            TypeConstraint::Instance { class, type_, span } => {
                self.solve_instance_constraint(&class, &type_, span)
            }
            TypeConstraint::Subtype { left, right, span } => {
                self.solve_subtype_constraint(&left, &right, span)
            }
            TypeConstraint::Default { var, default_type, span } => {
                // Keep default constraints for later processing
                ConstraintResult::Unresolved(TypeConstraint::Default { var, default_type, span })
            }
            TypeConstraint::Ambiguous { vars, span } => {
                // Keep ambiguity constraints for final check
                ConstraintResult::Unresolved(TypeConstraint::Ambiguous { vars, span })
            }
        }
    }
    
    /// Solves an equality constraint using unification.
    fn solve_equality_constraint(
        &self,
        left: &Type,
        right: &Type,
        span: Option<Span>,
        _reason: &str,
    ) -> ConstraintResult {
        use crate::types::unification::unify;
        
        match unify(left, right, span) {
            Ok(subst) => ConstraintResult::Solved(subst),
            Err(error) => ConstraintResult::Error(*error),
        }
    }
    
    /// Solves an instance constraint.
    fn solve_instance_constraint(
        &self,
        class: &str,
        type_: &Type,
        span: Option<Span>,
    ) -> ConstraintResult {
        // Check if we have an instance for this type
        if let Some(instances) = self.instances.get(class) {
            for instance_type in instances {
                // Try to unify the type with the instance type
                if let Ok(subst) = crate::types::unification::unify(type_, instance_type, span) {
                    return ConstraintResult::Solved(subst);
                }
            }
        }
        
        // Check if this is a built-in type class that we can resolve
        match class {
            "Eq" => self.solve_eq_instance(type_, span),
            "Show" => self.solve_show_instance(type_, span),
            "Num" => self.solve_num_instance(type_, span),
            "Default" => self.solve_default_instance(type_, span),
            _ => {
                // Cannot resolve this instance
                ConstraintResult::Error(Error::type_error(
                    format!("No instance of {class} for type {type_}"),
                    span.unwrap_or(Span::new(0, 0)),
                ))
            }
        }
    }
    
    /// Solves a subtype constraint.
    fn solve_subtype_constraint(
        &self,
        left: &Type,
        right: &Type,
        span: Option<Span>,
    ) -> ConstraintResult {
        // For now, just use equality
        // TODO: Implement proper subtyping
        self.solve_equality_constraint(left, right, span, "subtype")
    }
    
    /// Solves Eq instance constraint.
    fn solve_eq_instance(&self, type_: &Type, span: Option<Span>) -> ConstraintResult {
        match type_ {
            Type::Number | Type::String | Type::Boolean | Type::Char | Type::Symbol => {
                ConstraintResult::Solved(Substitution::empty())
            }
            Type::Pair(a, _b) => {
                // Pair is Eq if both components are Eq
                ConstraintResult::Unresolved(TypeConstraint::Instance {
                    class: "Eq".to_string(),
                    type_: (**a).clone(),
                    span,
                })
                // TODO: Also add constraint for b
            }
            Type::List(t) | Type::Vector(t) => {
                ConstraintResult::Unresolved(TypeConstraint::Instance {
                    class: "Eq".to_string(),
                    type_: (**t).clone(),
                    span,
                })
            }
            _ => ConstraintResult::Error(Error::type_error(
                format!("Type {type_} is not an instance of Eq"),
                span.unwrap_or(Span::new(0, 0)),
            )),
        }
    }
    
    /// Solves Show instance constraint.
    fn solve_show_instance(&self, type_: &Type, span: Option<Span>) -> ConstraintResult {
        match type_ {
            Type::Number | Type::String | Type::Boolean | Type::Char | Type::Symbol => {
                ConstraintResult::Solved(Substitution::empty())
            }
            _ => ConstraintResult::Error(Error::type_error(
                format!("Type {type_} is not an instance of Show"),
                span.unwrap_or(Span::new(0, 0)),
            )),
        }
    }
    
    /// Solves Num instance constraint.
    fn solve_num_instance(&self, type_: &Type, span: Option<Span>) -> ConstraintResult {
        match type_ {
            Type::Number => ConstraintResult::Solved(Substitution::empty()),
            _ => ConstraintResult::Error(Error::type_error(
                format!("Type {type_} is not an instance of Num"),
                span.unwrap_or(Span::new(0, 0)),
            )),
        }
    }
    
    /// Solves Default instance constraint.
    fn solve_default_instance(&self, type_: &Type, span: Option<Span>) -> ConstraintResult {
        match type_ {
            Type::Number | Type::String | Type::Boolean | Type::Char => {
                ConstraintResult::Solved(Substitution::empty())
            }
            _ => ConstraintResult::Error(Error::type_error(
                format!("Type {type_} is not an instance of Default"),
                span.unwrap_or(Span::new(0, 0)),
            )),
        }
    }
    
    /// Applies default types to unresolved variables.
    fn apply_defaults(&mut self) {
        let mut defaults_to_apply = Vec::new();
        
        for constraint in &self.constraints {
            if let TypeConstraint::Default { var, default_type, .. } = constraint {
                if !self.substitution.contains_var(var) {
                    defaults_to_apply.push((var.clone(), default_type.clone()));
                }
            }
        }
        
        for (var, default_type) in defaults_to_apply {
            let default_subst = Substitution::single(var, default_type);
            self.substitution = self.substitution.compose(&default_subst);
        }
        
        // Remove resolved default constraints
        self.constraints.retain(|c| {
            !matches!(c, TypeConstraint::Default { var, .. } if self.substitution.contains_var(var))
        });
    }
    
    /// Checks for ambiguous types and reports warnings.
    fn check_ambiguity(&mut self) {
        for constraint in &self.constraints {
            if let TypeConstraint::Ambiguous { vars, span: _ } = constraint {
                if !vars.is_empty() {
                    let var_names: Vec<_> = vars.iter()
                        .map(|v| v.name.as_deref().unwrap_or("?"))
                        .collect();
                    
                    // For now, just log a warning
                    // TODO: Implement proper ambiguity resolution
                    eprintln!(
                        "Warning: Ambiguous types for variables: {}",
                        var_names.join(", ")
                    );
                }
            }
        }
    }
}

/// Result of solving a single constraint.
#[derive(Debug)]
enum ConstraintResult {
    /// Constraint was solved with the given substitution
    Solved(Substitution),
    /// Constraint could not be solved yet
    Unresolved(TypeConstraint),
    /// Constraint led to an error
    Error(Error),
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TypeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeConstraint::Equal { left, right, reason, .. } => {
                write!(f, "{left} = {right} ({reason})")
            }
            TypeConstraint::Instance { class, type_, .. } => {
                write!(f, "{class} {type_}")
            }
            TypeConstraint::Subtype { left, right, .. } => {
                write!(f, "{left} <: {right}")
            }
            TypeConstraint::Default { var, default_type, .. } => {
                write!(f, "default {var} = {default_type}")
            }
            TypeConstraint::Ambiguous { vars, .. } => {
                write!(f, "ambiguous: ")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{var}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TypeVar, Type};

    #[test]
    fn test_equality_constraint() {
        let var = TypeVar::with_id(1);
        let constraint = TypeConstraint::equal(
            Type::Variable(var.clone()),
            Type::Number,
            None,
            "test constraint"
        );
        
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(constraint);
        
        let result = solver.solve();
        assert!(result.errors.is_empty());
        assert_eq!(result.substitution.apply_to_type(&Type::Variable(var)), Type::Number);
    }

    #[test]
    fn test_instance_constraint() {
        let constraint = TypeConstraint::instance("Eq", Type::Number, None);
        
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(constraint);
        
        let result = solver.solve();
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_unsolvable_instance() {
        let constraint = TypeConstraint::instance("Eq", Type::function(vec![Type::Number], Type::Number), None);
        
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(constraint);
        
        let result = solver.solve();
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_default_constraint() {
        let constraint = TypeConstraint::instance("Default", Type::Number, None);
        
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(constraint);
        
        let result = solver.solve();
        
        if !result.errors.is_empty() {
            println!("Constraint solver errors:");
            for error in &result.errors {
                println!("  {}", error);
            }
        }
        
        assert!(result.errors.is_empty());
    }
}