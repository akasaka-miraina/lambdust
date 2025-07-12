//! Type Inference Engine
//! Hindley-Milner style type inference with polynomial universe support

use super::polynomial_types::{PolynomialType, BaseType, UniverseLevel};
use crate::value::Value;
use crate::lexer::SchemeNumber;
use std::collections::HashMap;

/// Type inference context
#[derive(Debug, Clone)]
pub struct InferenceContext {
    /// Variable type assignments
    type_env: HashMap<String, PolynomialType>,
    /// Type variable constraints
    constraints: Vec<TypeConstraint>,
    /// Next type variable ID
    next_var_id: usize,
    /// Current universe level
    universe_level: UniverseLevel,
}

/// Type constraint for unification
#[derive(Debug, Clone, PartialEq)]
pub struct TypeConstraint {
    /// Left side of constraint
    pub left: PolynomialType,
    /// Right side of constraint
    pub right: PolynomialType,
    /// Constraint kind
    pub kind: ConstraintKind,
}

/// Kind of type constraint
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintKind {
    /// Equality constraint: T1 = T2
    Equality,
    /// Subtype constraint: T1 <: T2
    Subtype,
    /// Universe constraint: T : Universe_i
    Universe,
}

/// Type substitution
#[derive(Debug, Clone)]
pub struct TypeSubstitution {
    /// Variable -> Type mappings
    substitutions: HashMap<String, PolynomialType>,
}

impl TypeSubstitution {
    /// Create empty substitution
    pub fn empty() -> Self {
        Self {
            substitutions: HashMap::new(),
        }
    }

    /// Add substitution
    pub fn add(&mut self, var: String, typ: PolynomialType) {
        self.substitutions.insert(var, typ);
    }

    /// Apply substitution to type
    pub fn apply(&self, typ: &PolynomialType) -> PolynomialType {
        match typ {
            PolynomialType::Variable { name, .. } => {
                self.substitutions.get(name)
                    .cloned()
                    .unwrap_or_else(|| typ.clone())
            }
            PolynomialType::Function { input, output } => {
                PolynomialType::Function {
                    input: Box::new(self.apply(input)),
                    output: Box::new(self.apply(output)),
                }
            }
            PolynomialType::Product { left, right } => {
                PolynomialType::Product {
                    left: Box::new(self.apply(left)),
                    right: Box::new(self.apply(right)),
                }
            }
            PolynomialType::Sum { left, right } => {
                PolynomialType::Sum {
                    left: Box::new(self.apply(left)),
                    right: Box::new(self.apply(right)),
                }
            }
            PolynomialType::List { element_type } => {
                PolynomialType::List {
                    element_type: Box::new(self.apply(element_type)),
                }
            }
            PolynomialType::Pi { param_name, param_type, body_type } => {
                PolynomialType::Pi {
                    param_name: param_name.clone(),
                    param_type: Box::new(self.apply(param_type)),
                    body_type: Box::new(self.apply(body_type)),
                }
            }
            PolynomialType::Sigma { param_name, param_type, body_type } => {
                PolynomialType::Sigma {
                    param_name: param_name.clone(),
                    param_type: Box::new(self.apply(param_type)),
                    body_type: Box::new(self.apply(body_type)),
                }
            }
            PolynomialType::Identity { base_type, left, right } => {
                PolynomialType::Identity {
                    base_type: Box::new(self.apply(base_type)),
                    left: Box::new(self.apply(left)),
                    right: Box::new(self.apply(right)),
                }
            }
            _ => typ.clone(),
        }
    }

    /// Compose with another substitution
    pub fn compose(&self, other: &TypeSubstitution) -> TypeSubstitution {
        let mut result = other.clone();
        for (var, typ) in &self.substitutions {
            result.substitutions.insert(var.clone(), other.apply(typ));
        }
        result
    }
}

impl InferenceContext {
    /// Create new inference context
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            constraints: Vec::new(),
            next_var_id: 0,
            universe_level: UniverseLevel::new(0),
        }
    }

    /// Generate fresh type variable
    pub fn fresh_var(&mut self) -> PolynomialType {
        let var_name = format!("τ{}", self.next_var_id);
        self.next_var_id += 1;
        PolynomialType::Variable {
            name: var_name,
            level: self.universe_level,
        }
    }

    /// Add type binding
    pub fn bind(&mut self, var: String, typ: PolynomialType) {
        self.type_env.insert(var, typ);
    }

    /// Lookup type binding
    pub fn lookup(&self, var: &str) -> Option<&PolynomialType> {
        self.type_env.get(var)
    }

    /// Add constraint
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    /// Get constraints
    pub fn constraints(&self) -> &[TypeConstraint] {
        &self.constraints
    }

    /// Clear constraints
    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
    }
}

/// Type inference engine
#[derive(Debug)]
pub struct TypeInference {
    /// Inference context
    context: InferenceContext,
}

impl TypeInference {
    /// Create new type inference engine
    pub fn new() -> Self {
        Self {
            context: InferenceContext::new(),
        }
    }

    /// Infer type of a value
    pub fn infer(&mut self, value: &Value) -> Result<PolynomialType, crate::error::LambdustError> {
        self.infer_with_context(value, &mut self.context.clone())
    }

    /// Infer type with specific context
    pub fn infer_with_context(&mut self, value: &Value, context: &mut InferenceContext) -> Result<PolynomialType, crate::error::LambdustError> {
        match value {
            Value::Number(SchemeNumber::Integer(_)) => {
                Ok(PolynomialType::Base(BaseType::Integer))
            }
            Value::Number(SchemeNumber::Real(_)) => {
                Ok(PolynomialType::Base(BaseType::Real))
            }
            Value::Number(SchemeNumber::Rational(_, _)) => {
                Ok(PolynomialType::Base(BaseType::Real))
            }
            Value::Boolean(_) => {
                Ok(PolynomialType::Base(BaseType::Boolean))
            }
            Value::String(_) | Value::ShortString(_) => {
                Ok(PolynomialType::Base(BaseType::String))
            }
            Value::Character(_) => {
                Ok(PolynomialType::Base(BaseType::Character))
            }
            Value::Symbol(_) | Value::ShortSymbol(_) => {
                Ok(PolynomialType::Base(BaseType::Symbol))
            }
            Value::Nil => {
                // Nil: List α for fresh α
                let elem_type = context.fresh_var();
                Ok(PolynomialType::List {
                    element_type: Box::new(elem_type),
                })
            }
            Value::Pair(_) => {
                // For now, infer as generic list
                let elem_type = context.fresh_var();
                Ok(PolynomialType::List {
                    element_type: Box::new(elem_type),
                })
            }
            Value::Procedure(_) => {
                // Generic function: α → β
                let input_type = context.fresh_var();
                let output_type = context.fresh_var();
                Ok(PolynomialType::Function {
                    input: Box::new(input_type),
                    output: Box::new(output_type),
                })
            }
            _ => {
                // Unknown type: generate fresh variable
                Ok(context.fresh_var())
            }
        }
    }

    /// Unify two types
    pub fn unify(&mut self, type1: &PolynomialType, type2: &PolynomialType) -> Result<TypeSubstitution, crate::error::LambdustError> {
        self.unify_types(type1, type2)
    }

    /// Unify types implementation
    fn unify_types(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<TypeSubstitution, crate::error::LambdustError> {
        match (type1, type2) {
            // Same types
            (t1, t2) if t1 == t2 => Ok(TypeSubstitution::empty()),
            
            // Variable unification
            (PolynomialType::Variable { name, .. }, t2) => {
                if self.occurs_check(name, t2) {
                    return Err(crate::error::LambdustError::type_error(
                        format!("Occurs check failed: {} occurs in {:?}", name, t2)
                    ));
                }
                let mut subst = TypeSubstitution::empty();
                subst.add(name.clone(), t2.clone());
                Ok(subst)
            }
            (t1, PolynomialType::Variable { name, .. }) => {
                if self.occurs_check(name, t1) {
                    return Err(crate::error::LambdustError::type_error(
                        format!("Occurs check failed: {} occurs in {:?}", name, t1)
                    ));
                }
                let mut subst = TypeSubstitution::empty();
                subst.add(name.clone(), t1.clone());
                Ok(subst)
            }
            
            // Function types
            (PolynomialType::Function { input: i1, output: o1 },
             PolynomialType::Function { input: i2, output: o2 }) => {
                let input_subst = self.unify_types(i1, i2)?;
                let o1_applied = input_subst.apply(o1);
                let o2_applied = input_subst.apply(o2);
                let output_subst = self.unify_types(&o1_applied, &o2_applied)?;
                Ok(input_subst.compose(&output_subst))
            }
            
            // Product types
            (PolynomialType::Product { left: l1, right: r1 },
             PolynomialType::Product { left: l2, right: r2 }) => {
                let left_subst = self.unify_types(l1, l2)?;
                let r1_applied = left_subst.apply(r1);
                let r2_applied = left_subst.apply(r2);
                let right_subst = self.unify_types(&r1_applied, &r2_applied)?;
                Ok(left_subst.compose(&right_subst))
            }
            
            // List types
            (PolynomialType::List { element_type: e1 },
             PolynomialType::List { element_type: e2 }) => {
                self.unify_types(e1, e2)
            }
            
            // Base types
            (PolynomialType::Base(b1), PolynomialType::Base(b2)) if b1 == b2 => {
                Ok(TypeSubstitution::empty())
            }
            
            // Universe types
            (PolynomialType::Universe(l1), PolynomialType::Universe(l2)) if l1 == l2 => {
                Ok(TypeSubstitution::empty())
            }
            
            // Incompatible types
            _ => Err(crate::error::LambdustError::type_error(
                format!("Cannot unify {:?} with {:?}", type1, type2)
            ))
        }
    }

    /// Occurs check to prevent infinite types
    fn occurs_check(&self, var: &str, typ: &PolynomialType) -> bool {
        match typ {
            PolynomialType::Variable { name, .. } => name == var,
            PolynomialType::Function { input, output } => {
                self.occurs_check(var, input) || self.occurs_check(var, output)
            }
            PolynomialType::Product { left, right } => {
                self.occurs_check(var, left) || self.occurs_check(var, right)
            }
            PolynomialType::Sum { left, right } => {
                self.occurs_check(var, left) || self.occurs_check(var, right)
            }
            PolynomialType::List { element_type } => {
                self.occurs_check(var, element_type)
            }
            PolynomialType::Pi { param_type, body_type, .. } => {
                self.occurs_check(var, param_type) || self.occurs_check(var, body_type)
            }
            PolynomialType::Sigma { param_type, body_type, .. } => {
                self.occurs_check(var, param_type) || self.occurs_check(var, body_type)
            }
            PolynomialType::Identity { base_type, left, right } => {
                self.occurs_check(var, base_type) || self.occurs_check(var, left) || self.occurs_check(var, right)
            }
            _ => false,
        }
    }

    /// Solve constraints and return unified type
    pub fn solve_constraints(&mut self) -> Result<TypeSubstitution, crate::error::LambdustError> {
        let mut final_subst = TypeSubstitution::empty();
        
        for constraint in self.context.constraints().to_vec() {
            match constraint.kind {
                ConstraintKind::Equality => {
                    let subst = self.unify_types(&constraint.left, &constraint.right)?;
                    final_subst = final_subst.compose(&subst);
                }
                ConstraintKind::Subtype => {
                    // For now, treat subtype constraints as equality
                    let subst = self.unify_types(&constraint.left, &constraint.right)?;
                    final_subst = final_subst.compose(&subst);
                }
                ConstraintKind::Universe => {
                    // Universe constraints are handled separately
                    // No action needed for universe constraints in this step
                }
            }
        }
        
        self.context.clear_constraints();
        Ok(final_subst)
    }

    /// Get current context
    pub fn context(&self) -> &InferenceContext {
        &self.context
    }

    /// Get mutable context
    pub fn context_mut(&mut self) -> &mut InferenceContext {
        &mut self.context
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_basic_type_inference() {
        let mut inference = TypeInference::new();
        
        let int_value = Value::Number(SchemeNumber::Integer(42));
        let inferred = inference.infer(&int_value).unwrap();
        
        assert_eq!(inferred, PolynomialType::Base(BaseType::Integer));
    }

    #[test]
    fn test_fresh_variables() {
        let mut context = InferenceContext::new();
        
        let var1 = context.fresh_var();
        let var2 = context.fresh_var();
        
        // Should be different variables
        assert_ne!(var1, var2);
    }

    #[test]
    fn test_unification() {
        let mut inference = TypeInference::new();
        
        let int_type = PolynomialType::Base(BaseType::Integer);
        let var_type = PolynomialType::Variable {
            name: "α".to_string(),
            level: UniverseLevel::new(0),
        };
        
        let subst = inference.unify(&var_type, &int_type).unwrap();
        let result = subst.apply(&var_type);
        
        assert_eq!(result, int_type);
    }

    #[test]
    fn test_function_unification() {
        let mut inference = TypeInference::new();
        
        let int_type = PolynomialType::Base(BaseType::Integer);
        let var_type = PolynomialType::Variable {
            name: "α".to_string(),
            level: UniverseLevel::new(0),
        };
        
        let func1 = PolynomialType::Function {
            input: Box::new(var_type.clone()),
            output: Box::new(int_type.clone()),
        };
        
        let func2 = PolynomialType::Function {
            input: Box::new(int_type.clone()),
            output: Box::new(int_type.clone()),
        };
        
        let subst = inference.unify(&func1, &func2).unwrap();
        let result = subst.apply(&func1);
        
        assert_eq!(result, func2);
    }
}