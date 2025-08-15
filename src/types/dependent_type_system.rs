//! Dependent Type System Implementation
//!
//! This module provides a dependent type system implementation using the generic
//! type system framework. This system supports CaTT (Cartesian Type Theory) 
//! constructs and prepares for future categorical semantics integration.

use crate::diagnostics::{UnifiedResult, TypeError, Span};
use super::generic_type_system::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Dependent type system implementation
pub struct DependentTypeSystem {
    /// Universe hierarchy
    universe_hierarchy: UniverseHierarchy,
    /// Type equality checker
    equality_checker: TypeEqualityChecker,
    /// Proof term manager
    proof_manager: ProofTermManager,
    /// Next variable ID
    next_var_id: usize,
}

/// Dependent type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DepType {
    /// Universe/Kind (Type_i)
    Universe(UniverseLevel),
    /// Dependent function type (Π x:A. B)
    Pi {
        param_name: String,
        param_type: Box<DepType>,
        body_type: Box<DepType>,
    },
    /// Dependent sum type (Σ x:A. B)
    Sigma {
        param_name: String,
        param_type: Box<DepType>,
        body_type: Box<DepType>,
    },
    /// Lambda abstraction (λ x:A. t)
    Lambda {
        param_name: String,
        param_type: Box<DepType>,
        body_term: Box<DepTerm>,
    },
    /// Application (f a)
    Application {
        function: Box<DepType>,
        argument: Box<DepTerm>,
    },
    /// Variable reference
    Variable {
        name: String,
        de_bruijn_index: usize,
    },
    /// Type annotation (t : T)
    Annotation {
        term: Box<DepTerm>,
        type_: Box<DepType>,
    },
    /// Identity type (a =_A b)
    Identity {
        type_: Box<DepType>,
        left: Box<DepTerm>,
        right: Box<DepTerm>,
    },
    /// Inductive type definition
    Inductive {
        name: String,
        parameters: Vec<(String, DepType)>,
        universe: UniverseLevel,
        constructors: Vec<InductiveConstructor>,
    },
    /// Base types for compatibility
    Base(BaseTypeKind),
    /// CaTT-specific constructs
    CaTT(CaTTConstruct),
}

/// Terms in dependent type theory
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DepTerm {
    /// Variable
    Variable {
        name: String,
        de_bruijn_index: usize,
    },
    /// Lambda abstraction
    Lambda {
        param_name: String,
        param_type: Box<DepType>,
        body: Box<DepTerm>,
    },
    /// Application
    Application {
        function: Box<DepTerm>,
        argument: Box<DepTerm>,
    },
    /// Pair construction (a, b)
    Pair {
        first: Box<DepTerm>,
        second: Box<DepTerm>,
    },
    /// First projection (π₁)
    First(Box<DepTerm>),
    /// Second projection (π₂)
    Second(Box<DepTerm>),
    /// Constructor application
    Constructor {
        name: String,
        arguments: Vec<DepTerm>,
    },
    /// Pattern matching
    Match {
        scrutinee: Box<DepTerm>,
        return_type: Option<Box<DepType>>,
        cases: Vec<MatchCase>,
    },
    /// Reflexivity proof (refl)
    Refl {
        type_: Box<DepType>,
        term: Box<DepTerm>,
    },
    /// Type casting with proof
    Cast {
        term: Box<DepTerm>,
        from_type: Box<DepType>,
        to_type: Box<DepType>,
        proof: Box<DepTerm>, // Proof that from_type = to_type
    },
    /// Literal values
    Literal(LiteralValue),
}

/// Base type kinds for dependent system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseTypeKind {
    /// Natural numbers
    Nat,
    /// Integers
    Int,
    /// Booleans
    Bool,
    /// Strings
    String,
    /// Empty type
    Empty,
    /// Unit type
    Unit,
}

/// CaTT-specific constructs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CaTTConstruct {
    /// Object in category
    Object(String),
    /// Morphism between objects
    Morphism {
        domain: Box<DepType>,
        codomain: Box<DepType>,
        name: String,
    },
    /// Composition of morphisms
    Composition {
        first: Box<DepTerm>,
        second: Box<DepTerm>,
    },
    /// Identity morphism
    Identity(Box<DepType>),
    /// Functor application
    Functor {
        functor: String,
        object: Box<DepType>,
    },
    /// Natural transformation
    NaturalTransformation {
        source_functor: String,
        target_functor: String,
        name: String,
    },
    /// Adjoint functors
    Adjunction {
        left_adjoint: String,
        right_adjoint: String,
        unit: Box<DepTerm>,
        counit: Box<DepTerm>,
    },
}

/// Inductive type constructor
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InductiveConstructor {
    pub name: String,
    pub type_: DepType,
}

/// Pattern matching case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: DepTerm,
}

/// Patterns for pattern matching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// Variable pattern
    Variable(String),
    /// Constructor pattern
    Constructor {
        name: String,
        args: Vec<Pattern>,
    },
    /// Wildcard pattern
    Wildcard,
    /// Literal pattern
    Literal(LiteralValue),
}

/// Literal values in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralValue {
    Nat(usize),
    Int(i64),
    Bool(bool),
    String(String),
    Unit,
}

/// Universe hierarchy management
#[derive(Debug, Clone)]
pub struct UniverseHierarchy {
    /// Universe constraints
    constraints: Vec<UniverseConstraint>,
    /// Maximum known universe level
    max_level: usize,
}

/// Universe constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UniverseConstraint {
    /// u₁ < u₂
    LessThan(UniverseLevel, UniverseLevel),
    /// u₁ ≤ u₂
    LessEqual(UniverseLevel, UniverseLevel),
    /// u₁ = u₂
    Equal(UniverseLevel, UniverseLevel),
}

/// Universe level in dependent types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseLevel {
    level: usize,
    name: Option<String>,
}

/// Type equality checker for dependent types
#[derive(Debug, Clone)]
pub struct TypeEqualityChecker {
    /// Reduction strategy
    reduction_strategy: ReductionStrategy,
    /// Definitional equality cache
    equality_cache: HashMap<(DepType, DepType), bool>,
}

/// Reduction strategies for type checking
#[derive(Debug, Clone, Copy)]
pub enum ReductionStrategy {
    /// Call-by-value
    CallByValue,
    /// Call-by-name  
    CallByName,
    /// Lazy evaluation
    Lazy,
    /// Full reduction
    Full,
}

/// Proof term manager
#[derive(Debug, Clone)]
pub struct ProofTermManager {
    /// Proof term cache
    proof_cache: HashMap<DepType, DepProofTerm>,
    /// Proof obligations
    obligations: Vec<ProofObligation<DepType>>,
}

/// Dependent type proof term
#[derive(Debug, Clone)]
pub struct DepProofTerm {
    /// The proposition being proven
    proposition: DepType,
    /// The proof term
    proof: DepTerm,
    /// Dependencies on other proofs
    dependencies: Vec<String>,
}

/// Dependent type context
#[derive(Debug, Clone)]
pub struct DepContext {
    /// Variable bindings with types
    variables: HashMap<String, DepType>,
    /// Term definitions
    definitions: HashMap<String, DepTerm>,
    /// Type definitions
    type_definitions: HashMap<String, DepType>,
    /// Inductive type definitions
    inductives: HashMap<String, InductiveDefinition>,
    /// Universe constraints
    universe_constraints: Vec<UniverseConstraint>,
    /// Current scope depth
    scope_depth: usize,
}

/// Complete inductive type definition
#[derive(Debug, Clone)]
pub struct InductiveDefinition {
    pub name: String,
    pub parameters: Vec<(String, DepType)>,
    pub universe: UniverseLevel,
    pub constructors: Vec<InductiveConstructor>,
    pub elimination_rule: Option<DepTerm>,
}

/// Implementation of UniverseLevel for dependent types
impl UniverseLevel {
    pub fn new(level: usize) -> Self {
        UniverseLevel { level, name: None }
    }
    
    pub fn named(level: usize, name: String) -> Self {
        UniverseLevel { level, name: Some(name) }
    }
    
    pub fn level(&self) -> usize {
        self.level
    }
}

impl super::generic_type_system::UniverseLevel for UniverseLevel {
    fn succ(&self) -> Self {
        UniverseLevel {
            level: self.level + 1,
            name: self.name.clone(),
        }
    }
    
    fn max(&self, other: &Self) -> Self {
        if self.level >= other.level {
            self.clone()
        } else {
            other.clone()
        }
    }
    
    fn zero() -> Self {
        UniverseLevel::new(0)
    }
    
    fn omega() -> Self {
        UniverseLevel::new(usize::MAX)
    }
}

/// Implementation of ProofWitness for dependent types
impl ProofWitness for DepProofTerm {
    type Term = DepTerm;
    
    fn from_term(term: Self::Term) -> Self {
        DepProofTerm {
            proposition: DepType::Base(BaseTypeKind::Unit), // Placeholder
            proof: term,
            dependencies: Vec::new(),
        }
    }
    
    fn validates(&self, proposition: &dyn TypeRepr) -> bool {
        // Check if the proof term proves the given proposition
        // This would involve type checking the proof term
        true // Simplified for now
    }
    
    fn compose(&self, other: &Self) -> UnifiedResult<Self> {
        // Compose proof terms (modus ponens, etc.)
        Ok(DepProofTerm {
            proposition: other.proposition.clone(),
            proof: DepTerm::Application {
                function: Box::new(self.proof.clone()),
                argument: Box::new(other.proof.clone()),
            },
            dependencies: {
                let mut deps = self.dependencies.clone();
                deps.extend(other.dependencies.iter().cloned());
                deps
            },
        })
    }
}

/// Implementation of TermRepr for DepTerm
impl TermRepr for DepTerm {
    type Type = DepType;
    
    fn get_type(&self) -> Self::Type {
        // Type inference for terms - simplified
        match self {
            DepTerm::Variable { .. } => DepType::Base(BaseTypeKind::Unit), // Would look up in context
            DepTerm::Lambda { param_type, body, .. } => {
                let body_type = body.get_type();
                DepType::Pi {
                    param_name: "x".to_string(), // Simplified
                    param_type: param_type.clone(),
                    body_type: Box::new(body_type),
                }
            }
            DepTerm::Application { function, .. } => {
                match function.get_type() {
                    DepType::Pi { body_type, .. } => *body_type,
                    _ => DepType::Base(BaseTypeKind::Unit), // Type error
                }
            }
            DepTerm::Literal(lit) => match lit {
                LiteralValue::Nat(_) => DepType::Base(BaseTypeKind::Nat),
                LiteralValue::Int(_) => DepType::Base(BaseTypeKind::Int),
                LiteralValue::Bool(_) => DepType::Base(BaseTypeKind::Bool),
                LiteralValue::String(_) => DepType::Base(BaseTypeKind::String),
                LiteralValue::Unit => DepType::Base(BaseTypeKind::Unit),
            },
            _ => DepType::Base(BaseTypeKind::Unit), // Simplified
        }
    }
    
    fn apply_substitution(&self, subst: &dyn TermSubstitution) -> Self {
        subst.apply(self).downcast_ref::<DepTerm>()
            .expect("Substitution should return DepTerm")
            .clone()
    }
    
    fn reduce(&self) -> Self {
        // Beta reduction and normalization
        match self {
            DepTerm::Application { function, argument } => {
                let reduced_func = function.reduce();
                match reduced_func {
                    DepTerm::Lambda { body, .. } => {
                        // Beta reduction: (λx.t) a → t[x := a]
                        // Simplified: would need proper substitution
                        body.reduce()
                    }
                    _ => DepTerm::Application {
                        function: Box::new(reduced_func),
                        argument: Box::new(argument.reduce()),
                    }
                }
            }
            DepTerm::First(pair) => {
                match pair.reduce() {
                    DepTerm::Pair { first, .. } => first.reduce(),
                    reduced => DepTerm::First(Box::new(reduced)),
                }
            }
            DepTerm::Second(pair) => {
                match pair.reduce() {
                    DepTerm::Pair { second, .. } => second.reduce(),
                    reduced => DepTerm::Second(Box::new(reduced)),
                }
            }
            _ => self.clone(), // Already in normal form
        }
    }
    
    fn compose_morphism(&self, other: &Self) -> UnifiedResult<Self> {
        // Categorical composition
        Ok(DepTerm::Application {
            function: Box::new(self.clone()),
            argument: Box::new(other.clone()),
        })
    }
}

/// Implementation of TypeRepr for DepType
impl TypeRepr for DepType {
    type Universe = UniverseLevel;
    type Witness = DepProofTerm;
    
    fn universe(&self) -> Self::Universe {
        match self {
            DepType::Universe(level) => level.succ(),
            DepType::Pi { param_type, body_type, .. } => {
                param_type.universe().max(&body_type.universe())
            }
            DepType::Sigma { param_type, body_type, .. } => {
                param_type.universe().max(&body_type.universe())
            }
            DepType::Identity { type_, .. } => type_.universe(),
            DepType::Inductive { universe, .. } => universe.clone(),
            DepType::Base(_) => UniverseLevel::zero(),
            _ => UniverseLevel::zero(), // Simplified
        }
    }
    
    fn is_well_formed(&self, context: &dyn TypeContext<Type = Self>) -> bool {
        match self {
            DepType::Universe(_) => true,
            DepType::Pi { param_type, body_type, param_name } => {
                param_type.is_well_formed(context) && {
                    let extended_context = context.extend(param_name.clone(), param_type.as_ref().clone());
                    body_type.is_well_formed(&extended_context)
                }
            }
            DepType::Sigma { param_type, body_type, param_name } => {
                param_type.is_well_formed(context) && {
                    let extended_context = context.extend(param_name.clone(), param_type.as_ref().clone());
                    body_type.is_well_formed(&extended_context)
                }
            }
            DepType::Variable { name, .. } => {
                context.lookup(name).is_some()
            }
            DepType::Identity { type_, .. } => {
                type_.is_well_formed(context)
            }
            _ => true, // Simplified
        }
    }
    
    fn apply_substitution(&self, subst: &dyn Substitution<Type = Self>) -> Self {
        subst.apply(self)
    }
    
    fn free_variables(&self) -> HashSet<TypeVariable> {
        let mut vars = HashSet::new();
        self.collect_free_vars(&mut vars);
        vars
    }
    
    fn compose_with(&self, other: &Self) -> UnifiedResult<Self> {
        // Function composition in type theory: (B → C) ∘ (A → B) = (A → C)
        match (self, other) {
            (
                DepType::Pi { body_type: b_to_c, .. },
                DepType::Pi { param_type: a, body_type: a_to_b, param_name }
            ) if **a_to_b == **b_to_c => {
                Ok(DepType::Pi {
                    param_name: param_name.clone(),
                    param_type: a.clone(),
                    body_type: b_to_c.clone(),
                })
            }
            _ => Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                "Cannot compose incompatible types".to_string()
            )),
        }
    }
    
    fn unit(&self) -> UnifiedResult<Self> {
        // Identity function: A → A
        Ok(DepType::Pi {
            param_name: "x".to_string(),
            param_type: Box::new(self.clone()),
            body_type: Box::new(self.clone()),
        })
    }
    
    fn bind(&self, f_type: &Self) -> UnifiedResult<Self> {
        // For dependent types, bind would be more complex
        // This is a simplified version
        match f_type {
            DepType::Pi { body_type, .. } => Ok(body_type.as_ref().clone()),
            _ => Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                "Bind requires function type".to_string()
            )),
        }
    }
    
    fn identity(&self) -> Self {
        DepType::Pi {
            param_name: "x".to_string(),
            param_type: Box::new(self.clone()),
            body_type: Box::new(self.clone()),
        }
    }
    
    fn has_monad_structure(&self) -> bool {
        // Dependent types can encode monad structures
        matches!(self, DepType::CaTT(_))
    }
}

impl DepType {
    /// Collect free type variables
    fn collect_free_vars(&self, vars: &mut HashSet<TypeVariable>) {
        match self {
            DepType::Variable { name, de_bruijn_index } => {
                vars.insert(TypeVariable::new(name.clone(), *de_bruijn_index));
            }
            DepType::Pi { param_type, body_type, .. } |
            DepType::Sigma { param_type, body_type, .. } => {
                param_type.collect_free_vars(vars);
                body_type.collect_free_vars(vars);
            }
            DepType::Identity { type_, .. } => {
                type_.collect_free_vars(vars);
            }
            _ => {}, // Other cases don't introduce variables
        }
    }
    
    /// Display type for debugging
    pub fn display_type(&self) -> String {
        match self {
            DepType::Universe(level) => format!("Type_{}", level.level()),
            DepType::Pi { param_name, param_type, body_type } => {
                format!("Π {}:{}. {}", param_name, param_type.display_type(), body_type.display_type())
            }
            DepType::Sigma { param_name, param_type, body_type } => {
                format!("Σ {}:{}. {}", param_name, param_type.display_type(), body_type.display_type())
            }
            DepType::Variable { name, .. } => name.clone(),
            DepType::Identity { type_, .. } => format!("Id_{}", type_.display_type()),
            DepType::Base(base) => match base {
                BaseTypeKind::Nat => "ℕ".to_string(),
                BaseTypeKind::Int => "ℤ".to_string(),
                BaseTypeKind::Bool => "𝔹".to_string(),
                BaseTypeKind::String => "String".to_string(),
                BaseTypeKind::Empty => "⊥".to_string(),
                BaseTypeKind::Unit => "⊤".to_string(),
            },
            _ => "<complex type>".to_string(),
        }
    }
}

/// Implementation of TypeContext for DepContext
impl TypeContext<DepType> for DepContext {
    fn lookup(&self, var: &str) -> Option<DepType> {
        self.variables.get(var).cloned()
    }
    
    fn extend(&self, var: String, ty: DepType) -> Self {
        let mut new_context = self.clone();
        new_context.variables.insert(var, ty);
        new_context.scope_depth += 1;
        new_context
    }
    
    fn extend_many(&self, bindings: Vec<(String, DepType)>) -> Self {
        let mut new_context = self.clone();
        let bindings_len = bindings.len();
        for (var, ty) in bindings {
            new_context.variables.insert(var, ty);
        }
        new_context.scope_depth += bindings_len;
        new_context
    }
    
    fn bindings(&self) -> Vec<(String, DepType)> {
        self.variables.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    
    fn extend_term(&self, var: String, term: Box<dyn TermRepr>, ty: DepType) -> Self {
        let mut new_context = self.extend(var.clone(), ty);
        if let Ok(dep_term) = term.as_ref().as_any().downcast_ref::<DepTerm>() {
            new_context.definitions.insert(var, dep_term.clone());
        }
        new_context
    }
    
    fn is_well_formed(&self) -> bool {
        // Check that all types in the context are well-formed
        self.variables.values().all(|ty| ty.is_well_formed(self))
    }
}

impl DepContext {
    /// Creates new empty context
    pub fn new() -> Self {
        DepContext {
            variables: HashMap::new(),
            definitions: HashMap::new(),
            type_definitions: HashMap::new(),
            inductives: HashMap::new(),
            universe_constraints: Vec::new(),
            scope_depth: 0,
        }
    }
    
    /// Define an inductive type
    pub fn define_inductive(&mut self, definition: InductiveDefinition) {
        let name = definition.name.clone();
        self.inductives.insert(name, definition);
    }
    
    /// Look up term definition
    pub fn lookup_term(&self, name: &str) -> Option<&DepTerm> {
        self.definitions.get(name)
    }
}

/// Type system implementation for dependent types
impl TypeSystem for DependentTypeSystem {
    type Type = DepType;
    type Context = DepContext;
    type Constraint = DepConstraintSystem; // Would need to implement
    type Inference = DepInferenceEngine; // Would need to implement
    
    fn new() -> UnifiedResult<Self> {
        Ok(DependentTypeSystem {
            universe_hierarchy: UniverseHierarchy::new(),
            equality_checker: TypeEqualityChecker::new(),
            proof_manager: ProofTermManager::new(),
            next_var_id: 0,
        })
    }
    
    fn system_name(&self) -> &'static str {
        "dependent-types"
    }
    
    fn capabilities(&self) -> TypeSystemCapabilities {
        TypeSystemCapabilities {
            dependent_types: true,
            higher_order: true,
            polymorphism: true,
            gradual_typing: false,
            linear_types: false,
            effect_systems: true,
            categorical: true,
            monadic: true,
            catt_support: true,
        }
    }
    
    fn can_extend_with<Other: TypeSystem>(&self) -> bool {
        true // Dependent types can generally be extended
    }
}

// Placeholder implementations for constraint system and inference engine
pub struct DepConstraintSystem;
pub struct DepInferenceEngine;

impl ConstraintSystem<DepType> for DepConstraintSystem {
    type Constraint = (); // Placeholder
    
    fn add_constraint(&mut self, _constraint: Self::Constraint) {}
    fn solve(&self) -> UnifiedResult<Box<dyn Substitution<Type = DepType>>> {
        Ok(Box::new(DepSubstitution::empty()))
    }
    fn is_consistent(&self) -> bool { true }
    fn add_proof_obligation(&mut self, _obligation: ProofObligation<DepType>) {}
}

impl InferenceEngine<DepType, DepContext> for DepInferenceEngine {
    fn infer(&self, _expr: &dyn ExpressionRepr, _context: &DepContext) -> UnifiedResult<DepType> {
        Ok(DepType::Base(BaseTypeKind::Unit)) // Placeholder
    }
    
    fn check(&self, _expr: &dyn ExpressionRepr, _ty: &DepType, _context: &DepContext) -> UnifiedResult<()> {
        Ok(()) // Placeholder
    }
    
    fn infer_with_proof(&self, _expr: &dyn ExpressionRepr, _context: &DepContext) 
        -> UnifiedResult<(DepType, Box<dyn ProofTerm>)> {
        Ok((DepType::Base(BaseTypeKind::Unit), Box::new(DepProofTerm {
            proposition: DepType::Base(BaseTypeKind::Unit),
            proof: DepTerm::Literal(LiteralValue::Unit),
            dependencies: Vec::new(),
        })))
    }
    
    fn supports_bidirectional(&self) -> bool {
        true
    }
}

pub struct DepSubstitution;

impl DepSubstitution {
    pub fn empty() -> Self {
        DepSubstitution
    }
}

impl Substitution<DepType> for DepSubstitution {
    fn apply(&self, ty: &DepType) -> DepType {
        ty.clone() // Placeholder
    }
    
    fn compose(&self, _other: &dyn Substitution<Type = DepType>) -> Box<dyn Substitution<Type = DepType>> {
        Box::new(DepSubstitution)
    }
    
    fn identity() -> Box<dyn Substitution<Type = DepType>> {
        Box::new(DepSubstitution)
    }
    
    fn domain(&self) -> HashSet<TypeVariable> {
        HashSet::new()
    }
}

impl UniverseHierarchy {
    pub fn new() -> Self {
        UniverseHierarchy {
            constraints: Vec::new(),
            max_level: 0,
        }
    }
}

impl TypeEqualityChecker {
    pub fn new() -> Self {
        TypeEqualityChecker {
            reduction_strategy: ReductionStrategy::CallByValue,
            equality_cache: HashMap::new(),
        }
    }
}

impl ProofTermManager {
    pub fn new() -> Self {
        ProofTermManager {
            proof_cache: HashMap::new(),
            obligations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependent_type_system_creation() {
        let system = DependentTypeSystem::new().unwrap();
        assert_eq!(system.system_name(), "dependent-types");
        assert!(system.capabilities().dependent_types);
        assert!(system.capabilities().catt_support);
    }

    #[test]
    fn test_pi_type_creation() {
        let nat_type = DepType::Base(BaseTypeKind::Nat);
        let pi_type = DepType::Pi {
            param_name: "n".to_string(),
            param_type: Box::new(nat_type.clone()),
            body_type: Box::new(nat_type),
        };
        
        assert!(pi_type.display_type().contains("Π"));
        assert_eq!(pi_type.universe(), UniverseLevel::zero());
    }

    #[test]
    fn test_identity_type() {
        let nat_type = DepType::Base(BaseTypeKind::Nat);
        let zero = DepTerm::Literal(LiteralValue::Nat(0));
        let one = DepTerm::Literal(LiteralValue::Nat(1));
        
        let id_type = DepType::Identity {
            type_: Box::new(nat_type),
            left: Box::new(zero),
            right: Box::new(one),
        };
        
        assert!(id_type.display_type().contains("Id_"));
    }

    #[test]
    fn test_catt_constructs() {
        let obj = DepType::CaTT(CaTTConstruct::Object("A".to_string()));
        let morphism = DepType::CaTT(CaTTConstruct::Morphism {
            domain: Box::new(obj.clone()),
            codomain: Box::new(obj),
            name: "f".to_string(),
        });
        
        assert!(morphism.has_monad_structure());
    }

    #[test]
    fn test_context_operations() {
        let mut ctx = DepContext::new();
        let nat_type = DepType::Base(BaseTypeKind::Nat);
        
        ctx = ctx.extend("x".to_string(), nat_type.clone());
        assert_eq!(ctx.lookup("x"), Some(nat_type));
        assert!(ctx.is_well_formed());
    }

    #[test]
    fn test_universe_levels() {
        let level0 = UniverseLevel::zero();
        let level1 = level0.succ();
        let level2 = level1.succ();
        
        assert!(level0 < level1);
        assert!(level1 < level2);
        assert_eq!(level1.max(&level2), level2);
    }

    #[test]
    fn test_term_reduction() {
        let lambda = DepTerm::Lambda {
            param_name: "x".to_string(),
            param_type: Box::new(DepType::Base(BaseTypeKind::Nat)),
            body: Box::new(DepTerm::Variable {
                name: "x".to_string(),
                de_bruijn_index: 0,
            }),
        };
        
        let app = DepTerm::Application {
            function: Box::new(lambda),
            argument: Box::new(DepTerm::Literal(LiteralValue::Nat(42))),
        };
        
        let reduced = app.reduce();
        // Should reduce to the variable (which would be substituted)
        assert!(matches!(reduced, DepTerm::Variable { .. }));
    }
}