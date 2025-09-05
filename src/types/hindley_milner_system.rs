#![allow(missing_docs)]
//! Hindley-Milner Type System Implementation
//!
//! This module provides a concrete implementation of the Hindley-Milner type system
//! using the generic type system framework. This serves as the foundational type
//! system for Lambdust with polymorphic type inference.

#[cfg(feature = "experimental-type-system")]
use super::generic_type_system::*;
use crate::diagnostics::{Span, TypeError, UnifiedResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Dummy term type for HM system (which doesn't use dependent types)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HMTerm;

impl TermRepr for HMTerm {
    type Type = HMType;

    fn get_type(&self) -> Self::Type {
        // Dummy type for unit term
        HMType::Unit
    }

    fn apply_substitution(&self, _subst: &impl TermSubstitution) -> Self {
        // No-op for unit term
        HMTerm
    }

    fn reduce(&self) -> Self {
        // Already in normal form
        HMTerm
    }

    fn compose_morphism(&self, _other: &Self) -> UnifiedResult<Self> {
        // Composition of unit terms is unit
        Ok(HMTerm)
    }
}

/// Hindley-Milner type system implementation
pub struct HindleyMilnerSystem {
    /// Next type variable ID to allocate
    next_var_id: usize,
    /// Current universe level (always 0 for HM)
    universe_level: usize,
}

/// Hindley-Milner type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HMType {
    /// Base types (Number, String, Boolean, etc.)
    Base(BaseType),
    /// Function type (a -> b)
    Function(Box<HMType>, Box<HMType>),
    /// Type variable
    Variable(HMTypeVariable),
    /// Polymorphic type (∀a. T)
    Forall(Vec<HMTypeVariable>, Box<HMType>),
    /// List type [a]
    List(Box<HMType>),
    /// Pair type (a, b)
    Pair(Box<HMType>, Box<HMType>),
    /// Unit type ()
    Unit,
}

/// Base types in Hindley-Milner system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    Number,
    String,
    Boolean,
    Symbol,
    Unit,
}

/// Type variable in Hindley-Milner system
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HMTypeVariable {
    pub id: usize,
    pub name: Option<String>,
}

/// Hindley-Milner type context
#[derive(Debug, Clone)]
pub struct HMContext {
    /// Variable bindings
    bindings: HashMap<String, HMType>,
    /// Parent context for nested scopes
    parent: Option<Box<HMContext>>,
}

/// Hindley-Milner constraint system
#[derive(Debug, Clone)]
pub struct HMConstraintSystem {
    /// List of unification constraints
    constraints: Vec<HMConstraint>,
    /// Type variable substitution
    substitution: HMSubstitution,
}

/// Individual constraint in Hindley-Milner system
#[derive(Debug, Clone)]
pub enum HMConstraint {
    /// Unification constraint (T1 = T2)
    Unify(HMType, HMType),
    /// Instance constraint (T1 ⪯ T2)
    Instance(HMType, HMType),
}

/// Substitution in Hindley-Milner system
#[derive(Debug, Clone)]
pub struct HMSubstitution {
    /// Mapping from type variables to types
    mapping: HashMap<HMTypeVariable, HMType>,
}

/// Hindley-Milner inference engine
pub struct HMInferenceEngine {
    /// Current type variable counter
    var_counter: usize,
    /// Generalization level
    level: usize,
}

/// Simple universe level for HM (always 0)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HMUniverse(pub usize);

/// Simple proof witness for HM (not used)
#[derive(Debug, Clone)]
pub struct HMProofWitness;

/// Implementation of UniverseLevel for HMUniverse
impl UniverseLevel for HMUniverse {
    fn succ(&self) -> Self {
        HMUniverse(self.0 + 1)
    }

    fn max(&self, other: &Self) -> Self {
        HMUniverse(self.0.max(other.0))
    }

    fn zero() -> Self {
        HMUniverse(0)
    }

    fn omega() -> Self {
        HMUniverse(usize::MAX)
    }
}

/// Implementation of ProofWitness for HMProofWitness
impl ProofWitness for HMProofWitness {
    type Term = HMTerm; // HM doesn't use proof terms

    fn from_term(_: Self::Term) -> Self {
        HMProofWitness
    }

    fn validates(&self, _: &impl TypeRepr) -> bool {
        true // Always valid in HM
    }

    fn compose(&self, _: &Self) -> UnifiedResult<Self> {
        Ok(HMProofWitness)
    }
}

/// Implementation of TypeRepr for HMType
impl TypeRepr for HMType {
    type Universe = HMUniverse;
    type Witness = HMProofWitness;

    fn universe(&self) -> Self::Universe {
        HMUniverse(0) // HM is always at universe 0
    }

    fn is_well_formed(&self, _context: &impl TypeContext<Self>) -> bool {
        match self {
            HMType::Base(_) => true,
            HMType::Function(param, ret) => {
                param.is_well_formed(_context) && ret.is_well_formed(_context)
            }
            HMType::Variable(_) => true,
            HMType::Forall(vars, body) => {
                // Check no duplicate variables and body is well-formed
                let mut seen = HashSet::new();
                for var in vars {
                    if !seen.insert(var) {
                        return false; // Duplicate variable
                    }
                }
                body.is_well_formed(_context)
            }
            HMType::List(elem) => elem.is_well_formed(_context),
            HMType::Pair(first, second) => {
                first.is_well_formed(_context) && second.is_well_formed(_context)
            }
            HMType::Unit => true,
        }
    }

    fn apply_substitution(&self, subst: &impl Substitution<Self>) -> Self {
        subst.apply(self)
    }

    fn free_variables(&self) -> HashSet<TypeVariable> {
        let mut vars = HashSet::new();
        self.collect_free_vars(&mut vars, &HashSet::new());
        vars
    }

    fn compose_with(&self, other: &Self) -> UnifiedResult<Self> {
        // For HM, composition means function composition
        Ok(HMType::Function(
            Box::new(other.clone()),
            Box::new(self.clone()),
        ))
    }

    fn unit(&self) -> UnifiedResult<Self> {
        // Identity function for this type
        Ok(HMType::Function(
            Box::new(self.clone()),
            Box::new(self.clone()),
        ))
    }

    fn bind(&self, f_type: &Self) -> UnifiedResult<Self> {
        // For HM, bind is just function application type
        match f_type {
            HMType::Function(input, output) => {
                if self == input.as_ref() {
                    Ok(output.as_ref().clone())
                } else {
                    Err(crate::diagnostics::UnifiedError::new(
                        TypeError,
                        format!(
                            "Cannot bind type {} with function type {}",
                            self.display_type(),
                            f_type.display_type()
                        ),
                    ))
                }
            }
            _ => Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                "Bind requires function type".to_string(),
            )),
        }
    }

    fn identity(&self) -> Self {
        // Identity function type
        HMType::Function(Box::new(self.clone()), Box::new(self.clone()))
    }

    fn has_monad_structure(&self) -> bool {
        false // Basic HM doesn't have monads
    }
}

impl HMType {
    /// Collects free type variables
    fn collect_free_vars(&self, vars: &mut HashSet<TypeVariable>, bound: &HashSet<HMTypeVariable>) {
        match self {
            HMType::Variable(var) => {
                if !bound.contains(var) {
                    vars.insert(TypeVariable::new(
                        var.name.clone().unwrap_or_else(|| format!("t{}", var.id)),
                        var.id,
                    ));
                }
            }
            HMType::Function(param, ret) => {
                param.collect_free_vars(vars, bound);
                ret.collect_free_vars(vars, bound);
            }
            HMType::List(elem) => {
                elem.collect_free_vars(vars, bound);
            }
            HMType::Pair(first, second) => {
                first.collect_free_vars(vars, bound);
                second.collect_free_vars(vars, bound);
            }
            HMType::Forall(type_vars, body) => {
                let mut new_bound = bound.clone();
                new_bound.extend(type_vars.iter().cloned());
                body.collect_free_vars(vars, &new_bound);
            }
            HMType::Base(_) => {} // No variables in base types
            HMType::Unit => {}    // No variables in unit type
        }
    }

    /// Display type for error messages
    pub fn display_type(&self) -> String {
        match self {
            HMType::Base(base) => match base {
                BaseType::Number => "Number".to_string(),
                BaseType::String => "String".to_string(),
                BaseType::Boolean => "Boolean".to_string(),
                BaseType::Symbol => "Symbol".to_string(),
                BaseType::Unit => "()".to_string(),
            },
            HMType::Function(param, ret) => {
                format!("({} -> {})", param.display_type(), ret.display_type())
            }
            HMType::Variable(var) => var.name.clone().unwrap_or_else(|| format!("t{}", var.id)),
            HMType::Forall(vars, body) => {
                let var_names: Vec<String> = vars
                    .iter()
                    .map(|v| v.name.clone().unwrap_or_else(|| format!("t{}", v.id)))
                    .collect();
                format!("∀{}. {}", var_names.join(" "), body.display_type())
            }
            HMType::List(elem) => {
                format!("[{}]", elem.display_type())
            }
            HMType::Pair(first, second) => {
                format!("({}, {})", first.display_type(), second.display_type())
            }
            HMType::Unit => "Unit".to_string(),
        }
    }

    /// Creates a fresh type variable
    pub fn fresh_var(counter: &mut usize) -> Self {
        let id = *counter;
        *counter += 1;
        HMType::Variable(HMTypeVariable { id, name: None })
    }

    /// Creates a named type variable
    pub fn named_var(name: String, counter: &mut usize) -> Self {
        let id = *counter;
        *counter += 1;
        HMType::Variable(HMTypeVariable {
            id,
            name: Some(name),
        })
    }
}

/// Implementation of TypeContext for HMContext
impl TypeContext<HMType> for HMContext {
    fn lookup(&self, var: &str) -> Option<HMType> {
        if let Some(ty) = self.bindings.get(var) {
            Some(ty.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(var)
        } else {
            None
        }
    }

    fn extend(&self, var: String, ty: HMType) -> Self {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(var, ty);
        HMContext {
            bindings: new_bindings,
            parent: None, // Flattened for simplicity
        }
    }

    fn extend_many(&self, bindings: Vec<(String, HMType)>) -> Self {
        let mut new_bindings = self.bindings.clone();
        for (var, ty) in bindings {
            new_bindings.insert(var, ty);
        }
        HMContext {
            bindings: new_bindings,
            parent: None,
        }
    }

    fn bindings(&self) -> Vec<(String, HMType)> {
        self.bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn extend_term(&self, var: String, _term: impl TermRepr, ty: HMType) -> Self {
        // HM doesn't use term-level information
        self.extend(var, ty)
    }

    fn is_well_formed(&self) -> bool {
        // Check that all types in bindings are well-formed
        self.bindings.values().all(|ty| ty.is_well_formed(self))
    }
}

impl HMContext {
    /// Creates a new empty context
    pub fn new() -> Self {
        HMContext {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Creates a context with parent
    pub fn with_parent(parent: HMContext) -> Self {
        HMContext {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
}

/// Implementation of Substitution for HMSubstitution
impl Substitution<HMType> for HMSubstitution {
    fn apply(&self, ty: &HMType) -> HMType {
        match ty {
            HMType::Variable(var) => {
                if let Some(replacement) = self.mapping.get(var) {
                    // Apply substitution recursively to avoid infinite loops
                    self.apply(replacement)
                } else {
                    ty.clone()
                }
            }
            HMType::Function(param, ret) => {
                HMType::Function(Box::new(self.apply(param)), Box::new(self.apply(ret)))
            }
            HMType::List(elem) => HMType::List(Box::new(self.apply(elem))),
            HMType::Pair(first, second) => {
                HMType::Pair(Box::new(self.apply(first)), Box::new(self.apply(second)))
            }
            HMType::Forall(vars, body) => {
                // Don't substitute bound variables
                let filtered_mapping: HashMap<HMTypeVariable, HMType> = self
                    .mapping
                    .iter()
                    .filter(|(var, _)| !vars.contains(var))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let filtered_subst = HMSubstitution {
                    mapping: filtered_mapping,
                };
                HMType::Forall(vars.clone(), Box::new(filtered_subst.apply(body)))
            }
            _ => ty.clone(), // Base types are unchanged
        }
    }

    fn compose(&self, _other: &dyn Substitution<HMType>) -> Box<dyn Substitution<HMType>> {
        // For now, return self as a simplified implementation
        // TODO: Implement proper substitution composition
        Box::new(self.clone())
    }

    fn identity() -> Box<dyn Substitution<HMType>> {
        Box::new(HMSubstitution {
            mapping: HashMap::new(),
        })
    }

    fn domain(&self) -> HashSet<TypeVariable> {
        self.mapping
            .keys()
            .map(|var| {
                TypeVariable::new(
                    var.name.clone().unwrap_or_else(|| format!("t{}", var.id)),
                    var.id,
                )
            })
            .collect()
    }

    fn clone_boxed(&self) -> Box<dyn Substitution<HMType>> {
        Box::new(self.clone())
    }
}

impl HMSubstitution {
    /// Creates empty substitution
    pub fn empty() -> Self {
        HMSubstitution {
            mapping: HashMap::new(),
        }
    }

    /// Creates substitution from single mapping
    pub fn single(var: HMTypeVariable, ty: HMType) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(var, ty);
        HMSubstitution { mapping }
    }

    /// For downcasting in compose
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Implementation of ConstraintRepr for HMConstraint
impl ConstraintRepr<HMType> for HMConstraint {
    fn apply_substitution(&self, subst: &impl Substitution<HMType>) -> Self {
        match self {
            HMConstraint::Unify(t1, t2) => HMConstraint::Unify(subst.apply(t1), subst.apply(t2)),
            HMConstraint::Instance(t1, t2) => {
                HMConstraint::Instance(subst.apply(t1), subst.apply(t2))
            }
        }
    }

    fn variables(&self) -> HashSet<TypeVariable> {
        match self {
            HMConstraint::Unify(t1, t2) => {
                let mut vars = t1.free_variables();
                vars.extend(t2.free_variables());
                vars
            }
            HMConstraint::Instance(t1, t2) => {
                let mut vars = t1.free_variables();
                vars.extend(t2.free_variables());
                vars
            }
        }
    }

    fn simplify(&self) -> Option<Self> {
        match self {
            HMConstraint::Unify(t1, t2) if t1 == t2 => None, // Trivially true
            _ => Some(self.clone()),
        }
    }
}

/// Implementation of ConstraintSystem for HMConstraintSystem
impl ConstraintSystem<HMType> for HMConstraintSystem {
    type Constraint = HMConstraint;

    fn add_constraint(&mut self, constraint: Self::Constraint) {
        // Directly use the constraint's simplify method
        if let Some(simplified) = ConstraintRepr::<HMType>::simplify(&constraint) {
            self.constraints.push(simplified);
        }
    }

    fn solve(&self) -> UnifiedResult<Box<dyn Substitution<HMType>>> {
        let mut subst = HMSubstitution::empty();

        for constraint in &self.constraints {
            match constraint {
                HMConstraint::Unify(t1, t2) => {
                    let new_subst = self.unify(&subst.apply(t1), &subst.apply(t2))?;
                    subst = self.compose_substitutions(&subst, &new_subst);
                }
                HMConstraint::Instance(t1, t2) => {
                    // Instance constraints are handled during generalization
                    // For now, just treat as unification
                    let new_subst = self.unify(&subst.apply(t1), &subst.apply(t2))?;
                    subst = self.compose_substitutions(&subst, &new_subst);
                }
            }
        }

        Ok(Box::new(subst))
    }

    fn is_consistent(&self) -> bool {
        // Check if constraints can be solved
        let result: UnifiedResult<Box<dyn Substitution<HMType>>> = self.solve();
        result.is_ok()
    }

    fn add_proof_obligation(&mut self, _obligation: ProofObligation<HMType>) {
        // HM doesn't use proof obligations
    }
}

impl HMConstraintSystem {
    /// Creates new empty constraint system
    pub fn new() -> Self {
        HMConstraintSystem {
            constraints: Vec::new(),
            substitution: HMSubstitution::empty(),
        }
    }

    /// Unification algorithm
    fn unify(&self, t1: &HMType, t2: &HMType) -> UnifiedResult<HMSubstitution> {
        match (t1, t2) {
            // Same types
            (HMType::Base(b1), HMType::Base(b2)) if b1 == b2 => Ok(HMSubstitution::empty()),

            // Variable cases
            (HMType::Variable(v), t) | (t, HMType::Variable(v)) => {
                if t.free_variables().contains(&TypeVariable::new(
                    v.name.clone().unwrap_or_else(|| format!("t{}", v.id)),
                    v.id,
                )) {
                    Err(crate::diagnostics::UnifiedError::new(
                        TypeError,
                        format!(
                            "Occurs check failed: {} occurs in {}",
                            v.name.clone().unwrap_or_else(|| format!("t{}", v.id)),
                            t.display_type()
                        ),
                    ))
                } else {
                    Ok(HMSubstitution::single(v.clone(), t.clone()))
                }
            }

            // Function types
            (HMType::Function(p1, r1), HMType::Function(p2, r2)) => {
                let s1 = self.unify(p1, p2)?;
                let s2 = self.unify(&s1.apply(r1), &s1.apply(r2))?;
                Ok(self.compose_substitutions(&s1, &s2))
            }

            // List types
            (HMType::List(e1), HMType::List(e2)) => self.unify(e1, e2),

            // Pair types
            (HMType::Pair(f1, s1), HMType::Pair(f2, s2)) => {
                let sub1 = self.unify(f1, f2)?;
                let sub2 = self.unify(&sub1.apply(s1), &sub1.apply(s2))?;
                Ok(self.compose_substitutions(&sub1, &sub2))
            }

            // Mismatch
            _ => Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                format!(
                    "Cannot unify {} with {}",
                    t1.display_type(),
                    t2.display_type()
                ),
            )),
        }
    }

    /// Compose two substitutions
    fn compose_substitutions(&self, s1: &HMSubstitution, _s2: &HMSubstitution) -> HMSubstitution {
        // Simplified implementation - return first substitution
        // TODO: Implement proper composition
        s1.clone()
    }
}

/// Implementation of TypeSystem for HindleyMilnerSystem
impl TypeSystem for HindleyMilnerSystem {
    type Type = HMType;
    type Context = HMContext;
    type Constraint = HMConstraintSystem;
    type Inference = HMInferenceEngine;

    fn new() -> UnifiedResult<Self> {
        Ok(HindleyMilnerSystem {
            next_var_id: 0,
            universe_level: 0,
        })
    }

    fn system_name(&self) -> &'static str {
        "hindley-milner"
    }

    fn capabilities(&self) -> TypeSystemCapabilities {
        TypeSystemCapabilities::hindley_milner()
    }

    fn can_extend_with<Other: TypeSystem>(&self) -> bool {
        // Can extend with systems that add features without breaking HM
        true // For now, allow extension
    }
}

/// Implementation of InferenceEngine for HMInferenceEngine
impl InferenceEngine<HMType, HMContext> for HMInferenceEngine {
    fn infer(&self, expr: &impl ExpressionRepr, context: &HMContext) -> UnifiedResult<HMType> {
        // Placeholder implementation - would need actual expression AST
        // For now, return a fresh type variable
        let mut counter = self.var_counter;
        Ok(HMType::fresh_var(&mut counter))
    }

    fn check(
        &self,
        expr: &impl ExpressionRepr,
        ty: &HMType,
        context: &HMContext,
    ) -> UnifiedResult<()> {
        // Check if inferred type matches expected type
        let inferred_ty = self.infer(expr, context)?;
        let mut constraints = HMConstraintSystem::new();
        ConstraintSystem::<HMType>::add_constraint(
            &mut constraints,
            HMConstraint::Unify(inferred_ty, ty.clone()),
        );

        if ConstraintSystem::<HMType>::is_consistent(&constraints) {
            Ok(())
        } else {
            Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                "Type check failed".to_string(),
            ))
        }
    }

    fn infer_with_proof(
        &self,
        _expr: &impl ExpressionRepr,
        _context: &HMContext,
    ) -> UnifiedResult<(HMType, UnifiedProofTerm)> {
        Err(crate::diagnostics::UnifiedError::new(
            TypeError,
            "Proof terms not supported in Hindley-Milner".to_string(),
        ))
    }

    fn supports_bidirectional(&self) -> bool {
        true // HM supports both inference and checking
    }
}

impl HMInferenceEngine {
    /// Creates new inference engine
    pub fn new() -> Self {
        HMInferenceEngine {
            var_counter: 0,
            level: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hindley_milner_creation() {
        let hm = HindleyMilnerSystem::new().unwrap();
        assert_eq!(TypeSystem::system_name(&hm), "hindley-milner");
        assert!(TypeSystem::capabilities(&hm).polymorphism);
        assert!(!TypeSystem::capabilities(&hm).dependent_types);
    }

    #[test]
    fn test_type_unification() {
        let cs = HMConstraintSystem::new();

        // Test basic type unification
        let num1 = HMType::Base(BaseType::Number);
        let num2 = HMType::Base(BaseType::Number);
        let result = cs.unify(&num1, &num2);
        assert!(result.is_ok());

        // Test variable unification
        let var = HMType::Variable(HMTypeVariable { id: 0, name: None });
        let num = HMType::Base(BaseType::Number);
        let result = cs.unify(&var, &num);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_type_unification() {
        let cs = HMConstraintSystem::new();

        let f1 = HMType::Function(
            Box::new(HMType::Base(BaseType::Number)),
            Box::new(HMType::Base(BaseType::String)),
        );
        let f2 = HMType::Function(
            Box::new(HMType::Base(BaseType::Number)),
            Box::new(HMType::Base(BaseType::String)),
        );

        let result = cs.unify(&f1, &f2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_occurs_check() {
        let cs = HMConstraintSystem::new();

        let var = HMTypeVariable { id: 0, name: None };
        let recursive_type = HMType::Function(
            Box::new(HMType::Variable(var.clone())),
            Box::new(HMType::Base(BaseType::Number)),
        );

        let result = cs.unify(&HMType::Variable(var), &recursive_type);
        assert!(result.is_err()); // Should fail occurs check
    }

    #[test]
    fn test_context_operations() {
        let mut ctx = HMContext::new();
        let num_type = HMType::Base(BaseType::Number);

        ctx = ctx.extend("x".to_string(), num_type.clone());
        assert_eq!(ctx.lookup("x"), Some(num_type));
        assert_eq!(ctx.lookup("y"), None);
    }

    #[test]
    fn test_substitution() {
        let var = HMTypeVariable { id: 0, name: None };
        let subst = HMSubstitution::single(var.clone(), HMType::Base(BaseType::Number));

        let func_type = HMType::Function(
            Box::new(HMType::Variable(var)),
            Box::new(HMType::Base(BaseType::String)),
        );

        let result = subst.apply(&func_type);
        match result {
            HMType::Function(param, _) => {
                assert_eq!(*param, HMType::Base(BaseType::Number));
            }
            _ => panic!("Expected function type"),
        }
    }
}

/// Implementation of ConstraintRepr for MonadAwareType
impl
    crate::types::generic_type_system::ConstraintRepr<
        crate::types::monad_aware_system::MonadAwareType,
    > for HMConstraint
{
    fn apply_substitution(
        &self,
        _subst: &impl crate::types::generic_type_system::Substitution<
            crate::types::monad_aware_system::MonadAwareType,
        >,
    ) -> Self {
        self.clone() // Simplified
    }

    fn variables(&self) -> HashSet<crate::types::generic_type_system::TypeVariable> {
        HashSet::new() // Simplified
    }

    fn simplify(&self) -> Option<Self> {
        Some(self.clone())
    }
}

/// Implementation of ConstraintSystem for MonadAwareType
impl
    crate::types::generic_type_system::ConstraintSystem<
        crate::types::monad_aware_system::MonadAwareType,
    > for HMConstraintSystem
{
    type Constraint = HMConstraint;

    fn add_constraint(&mut self, constraint: Self::Constraint) {
        self.constraints.push(constraint);
    }

    fn solve(
        &self,
    ) -> crate::diagnostics::UnifiedResult<
        Box<
            dyn crate::types::generic_type_system::Substitution<
                    crate::types::monad_aware_system::MonadAwareType,
                >,
        >,
    > {
        // For now, return an empty substitution - would need to implement proper constraint solving
        Ok(Box::new(
            crate::types::monad_aware_system::MonadAwareSubstitution::empty(),
        ))
    }

    fn is_consistent(&self) -> bool {
        // Simplified check for consistency
        true
    }

    fn add_proof_obligation(
        &mut self,
        _obligation: crate::types::generic_type_system::ProofObligation<
            crate::types::monad_aware_system::MonadAwareType,
        >,
    ) {
        // Not implemented yet
    }
}

/// Implementation of InferenceEngine for MonadAwareType
impl
    crate::types::generic_type_system::InferenceEngine<
        crate::types::monad_aware_system::MonadAwareType,
        crate::types::monad_aware_system::MonadAwareContext,
    > for HMInferenceEngine
{
    fn infer(
        &self,
        _expr: &impl crate::types::generic_type_system::ExpressionRepr,
        _context: &crate::types::monad_aware_system::MonadAwareContext,
    ) -> crate::diagnostics::UnifiedResult<crate::types::monad_aware_system::MonadAwareType> {
        // Simplified inference - would need full implementation
        Ok(crate::types::monad_aware_system::MonadAwareType::HM(
            HMType::Base(BaseType::Number),
        ))
    }

    fn check(
        &self,
        _expr: &impl crate::types::generic_type_system::ExpressionRepr,
        expected_type: &crate::types::monad_aware_system::MonadAwareType,
        _context: &crate::types::monad_aware_system::MonadAwareContext,
    ) -> crate::diagnostics::UnifiedResult<()> {
        // Simplified check - would verify that expression has the expected type
        Ok(())
    }

    fn infer_with_proof(
        &self,
        _expr: &impl crate::types::generic_type_system::ExpressionRepr,
        _context: &crate::types::monad_aware_system::MonadAwareContext,
    ) -> crate::diagnostics::UnifiedResult<(
        crate::types::monad_aware_system::MonadAwareType,
        UnifiedProofTerm,
    )> {
        // Not supported for HM engine
        Err(crate::diagnostics::UnifiedError::new(
            crate::diagnostics::TypeError,
            "Proof terms not supported in Hindley-Milner".to_string(),
        ))
    }

    fn supports_bidirectional(&self) -> bool {
        true
    }
}
