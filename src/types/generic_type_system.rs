#![allow(missing_docs)]//! Generic Type System with CaTT and Monad Integration Support
//!
//! This module provides a highly flexible, extensible type system foundation
//! designed for future integration with Cartesian Type Theory (CaTT) and
//! natural monad structure incorporation while maintaining backward compatibility.

use crate::diagnostics::{Span, TypeError, UnifiedResult};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

/// Core generic type system trait providing extensibility for various type theories.
pub trait TypeSystem: Send + Sync + 'static {
    /// The type representation used by this type system
    type Type: TypeRepr;
    /// The context/environment type for this system
    type Context: TypeContext<Self::Type>;
    /// The constraint system used
    type Constraint: ConstraintSystem<Self::Type>;
    /// The inference engine for this system
    type Inference: InferenceEngine<Self::Type, Self::Context>;

    /// Creates a new instance of this type system
    fn new() -> UnifiedResult<Self>
    where
        Self: Sized;

    /// Returns the name/identifier for this type system
    fn system_name(&self) -> &'static str;

    /// Returns capabilities supported by this type system
    fn capabilities(&self) -> TypeSystemCapabilities;

    /// Checks if this system can be extended with another
    fn can_extend_with<Other: TypeSystem>(&self) -> bool;
}

/// Generic type representation supporting various type theories.
pub trait TypeRepr: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync {
    /// Universe/kind level for dependent types
    type Universe: UniverseLevel;
    /// Witness/proof type for dependent constructs
    type Witness: ProofWitness;

    /// Returns the universe level of this type
    fn universe(&self) -> Self::Universe;

    /// Checks if this type is well-formed in the given context
    fn is_well_formed(&self, context: &impl TypeContext<Self>) -> bool;

    /// Applies a substitution to this type
    fn apply_substitution(&self, subst: &impl Substitution<Self>) -> Self;

    /// Returns the free type variables in this type
    fn free_variables(&self) -> HashSet<TypeVariable>;

    /// For CaTT: composition operation
    fn compose_with(&self, other: &Self) -> UnifiedResult<Self>;

    /// For monads: unit/return operation
    fn unit(&self) -> UnifiedResult<Self>;

    /// For monads: bind operation type
    fn bind(&self, f_type: &Self) -> UnifiedResult<Self>;

    /// Identity morphism (for categorical structure)
    fn identity(&self) -> Self;

    /// Checks if this type supports monad structure
    fn has_monad_structure(&self) -> bool;
}

/// Type context/environment abstraction.
pub trait TypeContext<Type: TypeRepr>: Clone + Send + Sync {
    /// Looks up a variable in the context
    fn lookup(&self, var: &str) -> Option<Type>;

    /// Extends the context with a new binding
    fn extend(&self, var: String, ty: Type) -> Self;

    /// Extends with multiple bindings
    fn extend_many(&self, bindings: Vec<(String, Type)>) -> Self;

    /// Returns all bindings in scope
    fn bindings(&self) -> Vec<(String, Type)>;

    /// For dependent types: extends with term binding
    fn extend_term(&self, var: String, term: impl TermRepr, ty: Type) -> Self;

    /// Checks context well-formedness
    fn is_well_formed(&self) -> bool;
}

/// Constraint system abstraction for various solving approaches.
pub trait ConstraintSystem<Type: TypeRepr>: Clone + Send + Sync {
    type Constraint: ConstraintRepr<Type>;

    /// Adds a constraint to the system
    fn add_constraint(&mut self, constraint: Self::Constraint);

    /// Solves all constraints, returning substitutions
    fn solve(&self) -> UnifiedResult<Box<dyn Substitution<Type>>>
    where
        Self: Sized;

    /// Checks if the constraint system is consistent
    fn is_consistent(&self) -> bool;

    /// For dependent types: adds proof obligation
    fn add_proof_obligation(&mut self, obligation: ProofObligation<Type>);
}

/// Individual constraint representation.
pub trait ConstraintRepr<Type: TypeRepr>: Clone + Send + Sync {
    /// Applies substitution to constraint
    fn apply_substitution(&self, subst: &impl Substitution<Type>) -> Self;

    /// Returns variables involved in this constraint
    fn variables(&self) -> HashSet<TypeVariable>;

    /// Simplifies the constraint if possible
    fn simplify(&self) -> Option<Self>;
}

/// Inference engine abstraction supporting multiple algorithms.
pub trait InferenceEngine<Type: TypeRepr, Context: TypeContext<Type>>: Send + Sync {
    /// Infers the type of an expression
    fn infer(&self, expr: &impl ExpressionRepr, context: &Context) -> UnifiedResult<Type>;

    /// Checks if an expression has the given type
    fn check(&self, expr: &impl ExpressionRepr, ty: &Type, context: &Context) -> UnifiedResult<()>;

    /// For dependent types: infers with proof terms
    fn infer_with_proof(
        &self,
        expr: &impl ExpressionRepr,
        context: &Context,
    ) -> UnifiedResult<(Type, UnifiedProofTerm)>
    where
        Self: Sized;

    /// Supports bidirectional type checking
    fn supports_bidirectional(&self) -> bool;
}

/// Generic substitution system.
pub trait Substitution<Type: TypeRepr>: Send + Sync + std::fmt::Debug {
    /// Applies this substitution to a type
    fn apply(&self, ty: &Type) -> Type;

    /// Composes this substitution with another
    fn compose(&self, other: &dyn Substitution<Type>) -> Box<dyn Substitution<Type>>;

    /// Identity substitution
    fn identity() -> Box<dyn Substitution<Type>>
    where
        Self: Sized;

    /// Returns the domain of this substitution
    fn domain(&self) -> HashSet<TypeVariable>;

    /// Clone this substitution as a boxed trait object
    fn clone_boxed(&self) -> Box<dyn Substitution<Type>>;
}

/// Implement Clone for boxed Substitution trait objects
impl<Type: TypeRepr> Clone for Box<dyn Substitution<Type>> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Universe levels for dependent type systems.
pub trait UniverseLevel: Clone + PartialEq + Eq + PartialOrd + Ord + Send + Sync {
    /// Next universe level
    fn succ(&self) -> Self;

    /// Maximum of two universe levels
    fn max(&self, other: &Self) -> Self;

    /// Universe level 0 (types)
    fn zero() -> Self;

    /// Universe level ω (impredicative)
    fn omega() -> Self;
}

/// Proof witnesses for dependent type systems.
pub trait ProofWitness: Clone + Send + Sync {
    type Term: TermRepr;

    /// Constructs a proof witness from a term
    fn from_term(term: Self::Term) -> Self;

    /// Checks if the witness is valid for the given proposition
    fn validates(&self, proposition: &impl TypeRepr) -> bool;

    /// Composition of proof witnesses
    fn compose(&self, other: &Self) -> UnifiedResult<Self>;
}

/// Term representation for dependent systems.
pub trait TermRepr: Clone + Send + Sync {
    type Type: TypeRepr;

    /// Returns the type of this term
    fn get_type(&self) -> Self::Type;

    /// Applies term substitution
    fn apply_substitution(&self, subst: &impl TermSubstitution) -> Self;

    /// Reduces/evaluates the term
    fn reduce(&self) -> Self;

    /// For categorical structure: morphism composition
    fn compose_morphism(&self, other: &Self) -> UnifiedResult<Self>;
}

/// Expression representation abstraction.
pub trait ExpressionRepr: Send + Sync {
    /// Returns the span/location of this expression
    fn span(&self) -> Option<Span>;

    /// For pattern matching on expression types
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

/// Visitor pattern for expressions.
pub trait ExpressionVisitor {
    type Result;

    fn visit_variable(&mut self, name: &str) -> Self::Result;
    fn visit_application(
        &mut self,
        func: &impl ExpressionRepr,
        args: &[impl ExpressionRepr],
    ) -> Self::Result;
    fn visit_lambda(&mut self, param: &str, body: &impl ExpressionRepr) -> Self::Result;
    fn visit_let(
        &mut self,
        bindings: &[(String, impl ExpressionRepr)],
        body: &impl ExpressionRepr,
    ) -> Self::Result;
}

/// Capabilities supported by different type systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeSystemCapabilities {
    /// Supports dependent types
    pub dependent_types: bool,
    /// Supports higher-order types
    pub higher_order: bool,
    /// Supports polymorphism
    pub polymorphism: bool,
    /// Supports gradual typing
    pub gradual_typing: bool,
    /// Supports linear types
    pub linear_types: bool,
    /// Supports effect systems
    pub effect_systems: bool,
    /// Supports categorical structure
    pub categorical: bool,
    /// Supports monad integration
    pub monadic: bool,
    /// Supports CaTT constructs
    pub catt_support: bool,
}

/// Type variable representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeVariable {
    /// Variable name
    pub name: String,
    /// Unique identifier
    pub id: usize,
    /// Universe level for dependent types
    pub universe: usize,
}

impl TypeVariable {
    /// Creates a new type variable
    pub fn new(name: String, id: usize) -> Self {
        Self {
            name,
            id,
            universe: 0,
        }
    }

    /// Creates a type variable at a specific universe level
    pub fn at_universe(name: String, id: usize, universe: usize) -> Self {
        Self { name, id, universe }
    }
}

/// Proof obligation for dependent type systems.
#[derive(Debug, Clone)]
pub struct ProofObligation<Type: TypeRepr> {
    /// The proposition to prove
    pub proposition: Type,
    /// Context under which this must be proven
    pub context: String, // Simplified for now
    /// Priority/importance of this obligation
    pub priority: ProofPriority,
}

/// Priority levels for proof obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProofPriority {
    /// Must be proven for correctness
    Critical,
    /// Important for optimization
    Important,
    /// Nice to have for completeness
    Optional,
}

/// Unified proof term enum that supports all type systems
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifiedProofTerm {
    /// Simple proof for basic type systems
    Simple { proposition: String, validity: bool },
    /// Hindley-Milner proof term
    HindleyMilner {
        type_scheme: String,
        witness: Vec<u8>, // Serialized proof witness
    },
    /// Dependent type proof with universe levels
    Dependent {
        term: String,
        universe_level: u32,
        dependencies: Vec<String>,
    },
    /// Monad-aware proof with effect tracking
    MonadAware {
        computation_type: String,
        effect_signature: Vec<String>,
        monad_laws_proof: bool,
    },
    /// Category theory proof for CaTT integration
    Category {
        morphism: String,
        source: String,
        target: String,
        composition_proof: Vec<UnifiedProofTerm>,
    },
}

impl UnifiedProofTerm {
    /// The proposition this proof term proves
    pub fn proves_proposition(&self) -> String {
        match self {
            UnifiedProofTerm::Simple { proposition, .. } => proposition.clone(),
            UnifiedProofTerm::HindleyMilner { type_scheme, .. } => type_scheme.clone(),
            UnifiedProofTerm::Dependent { term, .. } => term.clone(),
            UnifiedProofTerm::MonadAware {
                computation_type, ..
            } => computation_type.clone(),
            UnifiedProofTerm::Category { morphism, .. } => morphism.clone(),
        }
    }

    /// Checks if this proof is valid
    pub fn is_valid(&self) -> bool {
        match self {
            UnifiedProofTerm::Simple { validity, .. } => *validity,
            UnifiedProofTerm::HindleyMilner { witness, .. } => !witness.is_empty(),
            UnifiedProofTerm::Dependent { dependencies, .. } => {
                dependencies.iter().all(|d| !d.is_empty())
            }
            UnifiedProofTerm::MonadAware {
                monad_laws_proof, ..
            } => *monad_laws_proof,
            UnifiedProofTerm::Category {
                composition_proof, ..
            } => composition_proof.iter().all(|p| p.is_valid()),
        }
    }

    /// Attempts to combine two proofs (generalized modus ponens)
    pub fn combine_with(&self, other: &Self) -> UnifiedResult<Self> {
        match (self, other) {
            (
                UnifiedProofTerm::Simple {
                    proposition: p1,
                    validity: v1,
                },
                UnifiedProofTerm::Simple {
                    proposition: p2,
                    validity: v2,
                },
            ) => Ok(UnifiedProofTerm::Simple {
                proposition: format!("({}) ∧ ({})", p1, p2),
                validity: *v1 && *v2,
            }),
            (
                UnifiedProofTerm::Category {
                    composition_proof: cp,
                    ..
                },
                other,
            ) => {
                let mut new_cp = cp.clone();
                new_cp.push(other.clone());
                let mut result = self.clone();
                if let UnifiedProofTerm::Category {
                    composition_proof, ..
                } = &mut result
                {
                    *composition_proof = new_cp;
                }
                Ok(result)
            }
            _ => Ok(self.clone()), // Default: return first proof
        }
    }
}

/// Term substitution for dependent systems.
pub trait TermSubstitution: Send + Sync {
    /// Applies substitution to a term
    fn apply(&self, term: &impl TermRepr) -> impl TermRepr;

    /// Composition with another substitution
    fn compose(&self, other: &impl TermSubstitution) -> impl TermSubstitution;
}

/// Monad structure integration for CaTT compatibility.
pub trait MonadStructure<Type: TypeRepr>: Send + Sync {
    /// The monad constructor
    type Constructor: TypeConstructor<Type>;

    /// Unit/return operation
    fn unit(&self) -> Type;

    /// Bind operation type
    fn bind(&self, m_type: &Type, f_type: &Type) -> UnifiedResult<Type>;

    /// Join operation (for monads that support it)
    fn join(&self, mm_type: &Type) -> UnifiedResult<Type>;

    /// Functor mapping
    fn fmap(&self, f_type: &Type, m_type: &Type) -> UnifiedResult<Type>;

    /// Checks monad laws
    fn check_monad_laws(&self) -> bool;

    /// Natural transformation between monads
    fn natural_transform<Other: MonadStructure<Type>>(
        &self,
        other: &Other,
        m_type: &Type,
    ) -> UnifiedResult<Type>;
}

/// Type constructor abstraction.
pub trait TypeConstructor<Type: TypeRepr>: Clone + Send + Sync {
    /// Applies the constructor to argument types
    fn apply(&self, args: &[Type]) -> UnifiedResult<Type>;

    /// Returns the arity of this constructor
    fn arity(&self) -> usize;

    /// For higher-kinded types: the kind of this constructor
    fn kind(&self) -> TypeKind;

    /// For categorical structure: composition with another constructor
    fn compose(&self, other: &Self) -> UnifiedResult<Self>;
}

/// Kind system for higher-order types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TypeKind {
    /// Base kind (*)
    Type,
    /// Function kind (* -> *)
    Arrow(Box<TypeKind>, Box<TypeKind>),
    /// Higher-order kind
    Higher(Vec<TypeKind>),
}

impl TypeKind {
    /// Creates a function kind
    pub fn arrow(from: TypeKind, to: TypeKind) -> Self {
        Self::Arrow(Box::new(from), Box::new(to))
    }

    /// Creates a curried function kind
    pub fn arrows(args: Vec<TypeKind>, result: TypeKind) -> Self {
        args.into_iter()
            .rev()
            .fold(result, |acc, arg| Self::arrow(arg, acc))
    }
}

/// Generic type system registry for managing multiple type systems.
pub struct TypeSystemRegistry {
    /// Registered type systems
    systems: HashMap<String, Box<dyn AnyTypeSystem>>,
    /// Default type system
    default_system: Option<String>,
    /// System capabilities cache
    capabilities_cache: HashMap<String, TypeSystemCapabilities>,
}

/// Type-erased type system for registry storage.
pub trait AnyTypeSystem: Send + Sync {
    fn system_name(&self) -> &'static str;
    fn capabilities(&self) -> TypeSystemCapabilities;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: TypeSystem> AnyTypeSystem for T {
    fn system_name(&self) -> &'static str {
        self.system_name()
    }

    fn capabilities(&self) -> TypeSystemCapabilities {
        self.capabilities()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypeSystemRegistry {
    /// Creates a new type system registry
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            default_system: None,
            capabilities_cache: HashMap::new(),
        }
    }

    /// Registers a type system
    pub fn register<T: TypeSystem>(&mut self, system: T) -> UnifiedResult<()> {
        let name = system.system_name().to_string();
        let capabilities = system.capabilities();

        self.systems.insert(name.clone(), Box::new(system));
        self.capabilities_cache.insert(name.clone(), capabilities);

        // Set as default if it's the first system
        if self.default_system.is_none() {
            self.default_system = Some(name);
        }

        Ok(())
    }

    /// Gets a type system by name
    pub fn get<T: TypeSystem>(&self, name: &str) -> Option<&T> {
        self.systems.get(name)?.as_any().downcast_ref::<T>()
    }

    /// Returns capabilities for a system
    pub fn capabilities(&self, name: &str) -> Option<TypeSystemCapabilities> {
        self.capabilities_cache.get(name).copied()
    }

    /// Finds systems supporting specific capabilities
    pub fn find_systems_with_capabilities(&self, required: TypeSystemCapabilities) -> Vec<String> {
        self.capabilities_cache
            .iter()
            .filter(|(_, caps)| self.supports_capabilities(**caps, required))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Checks if capabilities are supported
    fn supports_capabilities(
        &self,
        available: TypeSystemCapabilities,
        required: TypeSystemCapabilities,
    ) -> bool {
        (!required.dependent_types || available.dependent_types)
            && (!required.higher_order || available.higher_order)
            && (!required.polymorphism || available.polymorphism)
            && (!required.gradual_typing || available.gradual_typing)
            && (!required.linear_types || available.linear_types)
            && (!required.effect_systems || available.effect_systems)
            && (!required.categorical || available.categorical)
            && (!required.monadic || available.monadic)
            && (!required.catt_support || available.catt_support)
    }
}

impl Default for TypeSystemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypeSystemCapabilities {
    fn default() -> Self {
        Self {
            dependent_types: false,
            higher_order: false,
            polymorphism: true,
            gradual_typing: false,
            linear_types: false,
            effect_systems: false,
            categorical: false,
            monadic: false,
            catt_support: false,
        }
    }
}

/// CaTT-ready capabilities for future integration.
impl TypeSystemCapabilities {
    /// Full CaTT compatibility
    pub fn catt_compatible() -> Self {
        Self {
            dependent_types: true,
            higher_order: true,
            polymorphism: true,
            gradual_typing: true,
            linear_types: false,
            effect_systems: true,
            categorical: true,
            monadic: true,
            catt_support: true,
        }
    }

    /// Monad-focused capabilities
    pub fn monad_focused() -> Self {
        Self {
            dependent_types: false,
            higher_order: true,
            polymorphism: true,
            gradual_typing: false,
            linear_types: false,
            effect_systems: true,
            categorical: true,
            monadic: true,
            catt_support: false,
        }
    }

    /// Traditional Hindley-Milner
    pub fn hindley_milner() -> Self {
        Self {
            dependent_types: false,
            higher_order: false,
            polymorphism: true,
            gradual_typing: false,
            linear_types: false,
            effect_systems: false,
            categorical: false,
            monadic: false,
            catt_support: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example type system implementation for testing
    struct SimpleTypeSystem;

    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SimpleType;

    #[derive(Clone)]
    struct SimpleContext;

    #[derive(Clone)]
    struct SimpleConstraintSystem;

    struct SimpleInference;

    impl UniverseLevel for usize {
        fn succ(&self) -> Self {
            self + 1
        }
        fn max(&self, other: &Self) -> Self {
            (*self).max(*other)
        }
        fn zero() -> Self {
            0
        }
        fn omega() -> Self {
            usize::MAX
        }
    }

    impl TermRepr for () {
        type Type = SimpleType;
        fn get_type(&self) -> Self::Type {
            SimpleType
        }
        fn apply_substitution(&self, _: &impl TermSubstitution) -> Self {}
        fn reduce(&self) -> Self {}
        fn compose_morphism(&self, _: &Self) -> UnifiedResult<Self> {
            Ok(())
        }
    }

    impl ProofWitness for () {
        type Term = ();
        fn from_term(_: Self::Term) -> Self {}
        fn validates(&self, _: &impl TypeRepr) -> bool {
            true
        }
        fn compose(&self, _: &Self) -> UnifiedResult<Self> {
            Ok(())
        }
    }

    impl TypeRepr for SimpleType {
        type Universe = usize;
        type Witness = ();

        fn universe(&self) -> Self::Universe {
            0
        }
        fn is_well_formed(&self, _: &impl TypeContext<Self>) -> bool {
            true
        }
        fn apply_substitution(&self, _: &impl Substitution<Self>) -> Self {
            self.clone()
        }
        fn free_variables(&self) -> HashSet<TypeVariable> {
            HashSet::new()
        }
        fn compose_with(&self, _: &Self) -> UnifiedResult<Self> {
            Ok(self.clone())
        }
        fn unit(&self) -> UnifiedResult<Self> {
            Ok(self.clone())
        }
        fn bind(&self, _: &Self) -> UnifiedResult<Self> {
            Ok(self.clone())
        }
        fn identity(&self) -> Self {
            self.clone()
        }
        fn has_monad_structure(&self) -> bool {
            false
        }
    }

    impl TypeContext<SimpleType> for SimpleContext {
        fn lookup(&self, _: &str) -> Option<SimpleType> {
            None
        }
        fn extend(&self, _: String, _: SimpleType) -> Self {
            self.clone()
        }
        fn extend_many(&self, _: Vec<(String, SimpleType)>) -> Self {
            self.clone()
        }
        fn bindings(&self) -> Vec<(String, SimpleType)> {
            Vec::new()
        }
        fn extend_term(&self, _: String, _: impl TermRepr, _: SimpleType) -> Self {
            self.clone()
        }
        fn is_well_formed(&self) -> bool {
            true
        }
    }

    impl ConstraintRepr<SimpleType> for () {
        fn apply_substitution(&self, _: &impl Substitution<SimpleType>) -> Self {}
        fn variables(&self) -> HashSet<TypeVariable> {
            HashSet::new()
        }
        fn simplify(&self) -> Option<Self> {
            Some(())
        }
    }

    impl Substitution<SimpleType> for () {
        fn apply(&self, ty: &SimpleType) -> SimpleType {
            ty.clone()
        }
        fn compose(
            &self,
            _other: &dyn Substitution<SimpleType>,
        ) -> Box<dyn Substitution<SimpleType>> {
            Box::new(())
        }
        fn domain(&self) -> HashSet<TypeVariable> {
            HashSet::new()
        }
        fn identity() -> Box<dyn Substitution<SimpleType>>
        where
            Self: Sized,
        {
            Box::new(())
        }
        fn clone_boxed(&self) -> Box<dyn Substitution<SimpleType>> {
            Box::new(())
        }
    }

    impl ConstraintSystem<SimpleType> for SimpleConstraintSystem {
        type Constraint = ();

        fn add_constraint(&mut self, _: Self::Constraint) {}
        fn solve(&self) -> UnifiedResult<Box<dyn Substitution<SimpleType>>> {
            Ok(Box::new(()))
        }
        fn is_consistent(&self) -> bool {
            true
        }
        fn add_proof_obligation(&mut self, _: ProofObligation<SimpleType>) {}
    }

    impl InferenceEngine<SimpleType, SimpleContext> for SimpleInference {
        fn infer(&self, _: &impl ExpressionRepr, _: &SimpleContext) -> UnifiedResult<SimpleType> {
            Ok(SimpleType)
        }
        fn check(
            &self,
            _: &impl ExpressionRepr,
            _: &SimpleType,
            _: &SimpleContext,
        ) -> UnifiedResult<()> {
            Ok(())
        }
        fn infer_with_proof(
            &self,
            _: &impl ExpressionRepr,
            _: &SimpleContext,
        ) -> UnifiedResult<(SimpleType, UnifiedProofTerm)> {
            Err(crate::diagnostics::UnifiedError::new(
                TypeError,
                "Proof terms not supported",
            ))
        }
        fn supports_bidirectional(&self) -> bool {
            false
        }
    }

    impl TypeSystem for SimpleTypeSystem {
        type Type = SimpleType;
        type Context = SimpleContext;
        type Constraint = SimpleConstraintSystem;
        type Inference = SimpleInference;

        fn new() -> UnifiedResult<Self> {
            Ok(SimpleTypeSystem)
        }
        fn system_name(&self) -> &'static str {
            "simple"
        }
        fn capabilities(&self) -> TypeSystemCapabilities {
            TypeSystemCapabilities::hindley_milner()
        }
        fn can_extend_with<Other: TypeSystem>(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_type_system_registry() {
        let mut registry = TypeSystemRegistry::new();
        registry.register(SimpleTypeSystem).unwrap();

        assert!(registry.get::<SimpleTypeSystem>("simple").is_some());
        let caps = registry.capabilities("simple").unwrap();
        assert!(caps.polymorphism);
        assert!(!caps.dependent_types);
    }

    #[test]
    fn test_capability_filtering() {
        let mut registry = TypeSystemRegistry::new();
        registry.register(SimpleTypeSystem).unwrap();

        let monad_systems =
            registry.find_systems_with_capabilities(TypeSystemCapabilities::monad_focused());
        assert!(monad_systems.is_empty());

        let hm_systems =
            registry.find_systems_with_capabilities(TypeSystemCapabilities::hindley_milner());
        assert_eq!(hm_systems.len(), 1);
    }

    #[test]
    fn test_type_kind_construction() {
        let base = TypeKind::Type;
        let arrow = TypeKind::arrow(TypeKind::Type, TypeKind::Type);
        let curried = TypeKind::arrows(vec![TypeKind::Type, TypeKind::Type], TypeKind::Type);

        assert!(matches!(arrow, TypeKind::Arrow(_, _)));
        assert!(matches!(curried, TypeKind::Arrow(_, _)));
    }

    #[test]
    fn test_catt_capabilities() {
        let caps = TypeSystemCapabilities::catt_compatible();
        assert!(caps.dependent_types);
        assert!(caps.categorical);
        assert!(caps.monadic);
        assert!(caps.catt_support);
    }
}
