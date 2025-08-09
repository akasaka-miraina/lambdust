//! Advanced Type Class System for Lambdust R7RS-large support.
//!
//! This module extends the basic type class system with advanced features:
//! - Multi-parameter type classes
//! - Functional dependencies
//! - Associated types and families
//! - Higher-kinded types support
//! - Type class aliases and superclass constraints
//! - Coherence checking and orphan instance detection

#![allow(missing_docs)]

use super::{Type, TypeVar, Kind, Constraint};
use super::type_classes::{TypeClass, TypeClassInstance, TypeClassEnv};
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Advanced type class with multi-parameter support.
#[derive(Debug, Clone)]
pub struct AdvancedTypeClass {
    /// Base type class information
    pub base: TypeClass,
    /// Additional type parameters
    pub type_params: Vec<TypeVar>,
    /// Functional dependencies
    pub fundeps: Vec<FunctionalDependency>,
    /// Associated types
    pub associated_types: HashMap<String, AssociatedType>,
    /// Type families
    pub type_families: HashMap<String, TypeFamily>,
    /// Coherence rules
    pub coherence: CoherenceInfo,
}

/// Functional dependency specification.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionalDependency {
    /// Determining parameters (left side)
    pub determiners: Vec<usize>,
    /// Determined parameters (right side)
    pub determined: Vec<usize>,
}

/// Associated type definition.
#[derive(Debug, Clone)]
pub struct AssociatedType {
    /// Name of the associated type
    pub name: String,
    /// Kind of the associated type
    pub kind: Kind,
    /// Default type (if any)
    pub default: Option<Type>,
    /// Constraints on the associated type
    pub constraints: Vec<Constraint>,
}

/// Type family definition.
#[derive(Debug, Clone)]
pub enum TypeFamily {
    /// Open type family (can be extended)
    Open {
        name: String,
        kind: Kind,
        equations: Vec<TypeFamilyEquation>,
    },
    /// Closed type family (fixed set of equations)
    Closed {
        name: String,
        kind: Kind,
        equations: Vec<TypeFamilyEquation>,
    },
    /// Associated type family
    Associated {
        class: String,
        name: String,
        kind: Kind,
    },
}

/// Type family equation.
#[derive(Debug, Clone)]
pub struct TypeFamilyEquation {
    /// Left-hand side (pattern)
    pub lhs: Vec<Type>,
    /// Right-hand side (result)
    pub rhs: Type,
    /// Where clause constraints
    pub constraints: Vec<Constraint>,
}

/// Coherence information for type classes.
#[derive(Debug, Clone)]
pub struct CoherenceInfo {
    /// Whether this type class allows overlapping instances
    pub allow_overlapping: bool,
    /// Whether this type class allows orphan instances
    pub allow_orphans: bool,
    /// Whether this type class is closed (no new instances)
    pub closed: bool,
}

/// Advanced type class instance with multi-parameter support.
#[derive(Debug, Clone)]
pub struct AdvancedTypeClassInstance {
    /// Base instance information
    pub base: TypeClassInstance,
    /// Additional type arguments
    pub type_args: Vec<Type>,
    /// Associated type implementations
    pub associated_impls: HashMap<String, Type>,
    /// Type family instance implementations
    pub family_impls: HashMap<String, Vec<TypeFamilyEquation>>,
}

/// Higher-kinded type support.
#[derive(Debug, Clone)]
pub struct HigherKindedType {
    /// Base type constructor
    pub constructor: String,
    /// Kind signature
    pub kind: Kind,
    /// Type parameters with their kinds
    pub params: Vec<(TypeVar, Kind)>,
}

/// Type class alias for common patterns.
#[derive(Debug, Clone)]
pub struct TypeClassAlias {
    /// Name of the alias
    pub name: String,
    /// Type parameters
    pub params: Vec<TypeVar>,
    /// Constraints that this alias represents
    pub constraints: Vec<Constraint>,
}

/// Advanced type class environment.
#[derive(Debug, Clone)]
pub struct AdvancedTypeClassEnv {
    /// Base type class environment
    pub base: TypeClassEnv,
    /// Advanced type classes
    pub advanced_classes: HashMap<String, AdvancedTypeClass>,
    /// Advanced instances
    pub advanced_instances: HashMap<String, Vec<AdvancedTypeClassInstance>>,
    /// Type families
    pub type_families: HashMap<String, TypeFamily>,
    /// Type class aliases
    pub aliases: HashMap<String, TypeClassAlias>,
    /// Higher-kinded types
    pub hkt_types: HashMap<String, HigherKindedType>,
}

impl AdvancedTypeClass {
    /// Creates a new advanced type class.
    pub fn new(name: String, type_params: Vec<TypeVar>) -> Self {
        Self {
            base: TypeClass::new(name, Kind::Type),
            type_params,
            fundeps: Vec::new(),
            associated_types: HashMap::new(),
            type_families: HashMap::new(),
            coherence: CoherenceInfo::default(),
        }
    }

    /// Adds a functional dependency.
    pub fn with_fundep(mut self, fundep: FunctionalDependency) -> Self {
        self.fundeps.push(fundep);
        self
    }

    /// Adds an associated type.
    pub fn with_associated_type(mut self, name: String, assoc_type: AssociatedType) -> Self {
        self.associated_types.insert(name, assoc_type);
        self
    }

    /// Adds a type family.
    pub fn with_type_family(mut self, name: String, family: TypeFamily) -> Self {
        self.type_families.insert(name, family);
        self
    }

    /// Checks if two instances would overlap.
    pub fn instances_overlap(
        &self,
        inst1: &AdvancedTypeClassInstance,
        inst2: &AdvancedTypeClassInstance,
    ) -> bool {
        // Two instances overlap if there exists a substitution that makes
        // their heads unifiable
        
        if inst1.type_args.len() != inst2.type_args.len() {
            return false;
        }

        // Simplified overlap checking
        // In a full implementation, this would use unification
        for (t1, t2) in inst1.type_args.iter().zip(inst2.type_args.iter()) {
            if !self.types_potentially_unifiable(t1, t2) {
                return false;
            }
        }

        true
    }

    #[allow(clippy::only_used_in_recursion)]
    fn types_potentially_unifiable(&self, _t1: &Type, _t2: &Type) -> bool {
        match (_t1, _t2) {
            (Type::Variable(_), _) | (_, Type::Variable(_)) => true,
            (Type::Constructor { name: n1, .. }, Type::Constructor { name: n2, .. }) => n1 == n2,
            (Type::Application { constructor: c1, argument: a1 }, 
             Type::Application { constructor: c2, argument: a2 }) => {
                self.types_potentially_unifiable(c1, c2) && 
                self.types_potentially_unifiable(a1, a2)
            }
            _ => _t1 == _t2,
        }
    }

    /// Validates functional dependencies for an instance.
    pub fn validate_fundeps(&self, _instance: &AdvancedTypeClassInstance) -> Result<()> {
        for fundep in &self.fundeps {
            // Check that the functional dependency is satisfied
            // This is a complex check that requires examining all instances
            
            // Simplified validation for now
            if fundep.determiners.is_empty() || fundep.determined.is_empty() {
                return Err(Box::new(Error::type_error(
                    "Invalid functional dependency".to_string(),
                    Span::default(),
                )));
            }
        }
        Ok(())
    }
}

impl AdvancedTypeClassInstance {
    /// Creates a new advanced instance.
    pub fn new(
        class: String,
        type_args: Vec<Type>,
        span: Option<Span>,
    ) -> Self {
        Self {
            base: TypeClassInstance::new(class, type_args[0].clone(), span),
            type_args,
            associated_impls: HashMap::new(),
            family_impls: HashMap::new(),
        }
    }

    /// Adds an associated type implementation.
    pub fn with_associated_impl(mut self, name: String, ty: Type) -> Self {
        self.associated_impls.insert(name, ty);
        self
    }

    /// Adds a type family implementation.
    pub fn with_family_impl(mut self, name: String, equations: Vec<TypeFamilyEquation>) -> Self {
        self.family_impls.insert(name, equations);
        self
    }
}

impl TypeFamily {
    /// Creates a new open type family.
    pub fn open(name: String, kind: Kind) -> Self {
        TypeFamily::Open {
            name,
            kind,
            equations: Vec::new(),
        }
    }

    /// Creates a new closed type family.
    pub fn closed(name: String, kind: Kind, equations: Vec<TypeFamilyEquation>) -> Self {
        TypeFamily::Closed {
            name,
            kind,
            equations,
        }
    }

    /// Gets the name of this type family.
    pub fn name(&self) -> &str {
        match self {
            TypeFamily::Open { name, .. } |
            TypeFamily::Closed { name, .. } |
            TypeFamily::Associated { name, .. } => name,
        }
    }

    /// Gets the kind of this type family.
    pub fn kind(&self) -> &Kind {
        match self {
            TypeFamily::Open { kind, .. } |
            TypeFamily::Closed { kind, .. } |
            TypeFamily::Associated { kind, .. } => kind,
        }
    }

    /// Reduces a type family application.
    pub fn reduce(&self, args: &[Type]) -> Option<Type> {
        let equations = match self {
            TypeFamily::Open { equations, .. } |
            TypeFamily::Closed { equations, .. } => equations,
            TypeFamily::Associated { .. } => return None, // Need instance context
        };

        // Try to match against each equation
        for equation in equations {
            if let Some(result) = self.try_match_equation(equation, args) {
                return Some(result);
            }
        }

        None
    }

    fn try_match_equation(&self, equation: &TypeFamilyEquation, args: &[Type]) -> Option<Type> {
        if equation.lhs.len() != args.len() {
            return None;
        }

        // Simplified pattern matching
        // In a full implementation, this would use proper unification
        for (pattern, arg) in equation.lhs.iter().zip(args.iter()) {
            if !self.pattern_matches(pattern, arg) {
                return None;
            }
        }

        Some(equation.rhs.clone())
    }

    fn pattern_matches(&self, pattern: &Type, arg: &Type) -> bool {
        match (pattern, arg) {
            (Type::Variable(_), _) => true, // Variables match anything
            _ => pattern == arg,
        }
    }
}

impl AdvancedTypeClassEnv {
    /// Creates a new advanced type class environment.
    pub fn new() -> Self {
        Self {
            base: TypeClassEnv::new(),
            advanced_classes: HashMap::new(),
            advanced_instances: HashMap::new(),
            type_families: HashMap::new(),
            aliases: HashMap::new(),
            hkt_types: HashMap::new(),
        }
    }

    /// Adds an advanced type class.
    pub fn add_advanced_class(&mut self, class: AdvancedTypeClass) {
        let name = class.base.name.clone();
        self.advanced_classes.insert(name, class);
    }

    /// Adds an advanced instance.
    pub fn add_advanced_instance(&mut self, instance: AdvancedTypeClassInstance) {
        let class_name = instance.base.class.clone();
        self.advanced_instances
            .entry(class_name)
            .or_default()
            .push(instance);
    }

    /// Adds a type family.
    pub fn add_type_family(&mut self, family: TypeFamily) {
        let name = family.name().to_string();
        self.type_families.insert(name, family);
    }

    /// Adds a type class alias.
    pub fn add_alias(&mut self, alias: TypeClassAlias) {
        self.aliases.insert(alias.name.clone(), alias);
    }

    /// Registers a higher-kinded type.
    pub fn add_hkt_type(&mut self, hkt: HigherKindedType) {
        self.hkt_types.insert(hkt.constructor.clone(), hkt);
    }

    /// Checks for coherence violations.
    pub fn check_coherence(&self) -> Result<()> {
        for (class_name, instances) in &self.advanced_instances {
            if let Some(class) = self.advanced_classes.get(class_name) {
                if !class.coherence.allow_overlapping {
                    self.check_no_overlapping_instances(instances)?;
                }
                
                if !class.coherence.allow_orphans {
                    self.check_no_orphan_instances(instances)?;
                }
            }
        }
        Ok(())
    }

    fn check_no_overlapping_instances(
        &self,
        instances: &[AdvancedTypeClassInstance],
    ) -> Result<()> {
        for (i, inst1) in instances.iter().enumerate() {
            for inst2 in instances.iter().skip(i + 1) {
                // This would need access to the class to check overlap
                // Simplified for now
                if inst1.type_args == inst2.type_args {
                    return Err(Box::new(Error::type_error(
                        "Overlapping instances detected".to_string(),
                        Span::default(),
                    )));
                }
            }
        }
        Ok(())
    }

    fn check_no_orphan_instances(
        &self,
        _instances: &[AdvancedTypeClassInstance],
    ) -> Result<()> {
        // Orphan instance checking requires module system information
        // which we don't have in this simplified implementation
        Ok(())
    }

    /// Creates built-in advanced type classes for R7RS-large.
    pub fn create_builtin_advanced_classes(&mut self) {
        // Foldable type class
        let foldable = AdvancedTypeClass::new(
            "Foldable".to_string(),
            vec![TypeVar::with_name("t")],
        );
        self.add_advanced_class(foldable);

        // Traversable type class
        let traversable = AdvancedTypeClass::new(
            "Traversable".to_string(),
            vec![TypeVar::with_name("t")],
        );
        self.add_advanced_class(traversable);

        // MonadFail type class
        let monad_fail = AdvancedTypeClass::new(
            "MonadFail".to_string(),
            vec![TypeVar::with_name("m")],
        );
        self.add_advanced_class(monad_fail);

        // Alternative type class
        let alternative = AdvancedTypeClass::new(
            "Alternative".to_string(),
            vec![TypeVar::with_name("f")],
        );
        self.add_advanced_class(alternative);

        // Category type class (for arrow types)
        let category = AdvancedTypeClass::new(
            "Category".to_string(),
            vec![TypeVar::with_name("cat")],
        );
        self.add_advanced_class(category);

        // Arrow type class
        let arrow = AdvancedTypeClass::new(
            "Arrow".to_string(),
            vec![TypeVar::with_name("arr")],
        );
        self.add_advanced_class(arrow);
    }

    /// Creates built-in type families for R7RS-large.
    pub fn create_builtin_type_families(&mut self) {
        // Element type family
        let elem_family = TypeFamily::open(
            "Elem".to_string(), 
            Kind::arrow(Kind::Type, Kind::Type)
        );
        self.add_type_family(elem_family);

        // Index type family
        let index_family = TypeFamily::open(
            "Index".to_string(),
            Kind::arrow(Kind::Type, Kind::Type)
        );
        self.add_type_family(index_family);

        // Container type family
        let container_family = TypeFamily::open(
            "Container".to_string(),
            Kind::arrow(Kind::Type, Kind::arrow(Kind::Type, Kind::Type))
        );
        self.add_type_family(container_family);
    }
}

impl FunctionalDependency {
    /// Creates a new functional dependency.
    pub fn new(determiners: Vec<usize>, determined: Vec<usize>) -> Self {
        Self {
            determiners,
            determined,
        }
    }

    /// Checks if this functional dependency is valid.
    pub fn is_valid(&self, arity: usize) -> bool {
        self.determiners.iter().all(|&i| i < arity) &&
        self.determined.iter().all(|&i| i < arity)
    }
}

impl CoherenceInfo {
    /// Creates default coherence rules (strict).
    pub fn strict() -> Self {
        Self {
            allow_overlapping: false,
            allow_orphans: false,
            closed: false,
        }
    }

    /// Creates permissive coherence rules.
    pub fn permissive() -> Self {
        Self {
            allow_overlapping: true,
            allow_orphans: true,
            closed: false,
        }
    }
}

impl Default for CoherenceInfo {
    fn default() -> Self {
        Self::strict()
    }
}

impl Default for AdvancedTypeClassEnv {
    fn default() -> Self {
        let mut env = Self::new();
        env.create_builtin_advanced_classes();
        env.create_builtin_type_families();
        env
    }
}

impl fmt::Display for AdvancedTypeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class")?;
        
        if !self.type_params.is_empty() {
            write!(f, " (")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 { write!(f, " ")?; }
                write!(f, "{param}")?;
            }
            write!(f, ")")?;
        }
        
        write!(f, " {}", self.base.name)?;
        
        if !self.fundeps.is_empty() {
            write!(f, " |")?;
            for (i, fundep) in self.fundeps.iter().enumerate() {
                if i > 0 { write!(f, ",")?; }
                write!(f, " {fundep}")?;
            }
        }
        
        write!(f, " where")?;
        
        for (method_name, method_type) in &self.base.methods {
            write!(f, "\n  {} : {}", method_name, method_type.type_)?;
        }
        
        for (assoc_name, assoc_type) in &self.associated_types {
            write!(f, "\n  type {} : {}", assoc_name, assoc_type.kind)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for FunctionalDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, &det) in self.determiners.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{det}")?;
        }
        write!(f, " ->")?;
        for &det in &self.determined {
            write!(f, " {det}")?;
        }
        Ok(())
    }
}

impl fmt::Display for TypeFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeFamily::Open { name, kind, .. } => {
                write!(f, "type family {name} : {kind}")
            }
            TypeFamily::Closed { name, kind, equations } => {
                write!(f, "type family {name} : {kind} where")?;
                for equation in equations {
                    write!(f, "\n  {equation}")?;
                }
                Ok(())
            }
            TypeFamily::Associated { class, name, kind } => {
                write!(f, "type {name} : {kind} (associated with {class})")
            }
        }
    }
}

impl fmt::Display for TypeFamilyEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, lhs_type) in self.lhs.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{lhs_type}")?;
        }
        write!(f, " = {}", self.rhs)?;
        
        if !self.constraints.is_empty() {
            write!(f, " where")?;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if i > 0 { write!(f, ",")?; }
                write!(f, " {} {}", constraint.class, constraint.type_)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_type_class_creation() {
        let mut monad_plus = AdvancedTypeClass::new(
            "MonadPlus".to_string(),
            vec![TypeVar::with_name("m")]
        );

        let fundep = FunctionalDependency::new(vec![0], vec![]);
        monad_plus = monad_plus.with_fundep(fundep);

        assert_eq!(monad_plus.base.name, "MonadPlus");
        assert_eq!(monad_plus.type_params.len(), 1);
        assert_eq!(monad_plus.fundeps.len(), 1);
    }

    #[test]
    fn test_type_family_creation() {
        let mut equations = Vec::new();
        equations.push(TypeFamilyEquation {
            lhs: vec![Type::Constructor { name: "Int".to_string(), kind: Kind::Type }],
            rhs: Type::Constructor { name: "Int".to_string(), kind: Kind::Type },
            constraints: vec![],
        });

        let family = TypeFamily::closed(
            "Identity".to_string(),
            Kind::arrow(Kind::Type, Kind::Type),
            equations
        );

        assert_eq!(family.name(), "Identity");
        assert_eq!(*family.kind(), Kind::arrow(Kind::Type, Kind::Type));
    }

    #[test]
    fn test_functional_dependency_validation() {
        let fundep = FunctionalDependency::new(vec![0, 1], vec![2]);
        assert!(fundep.is_valid(3));
        assert!(!fundep.is_valid(2));
    }

    #[test]
    fn test_coherence_info() {
        let strict = CoherenceInfo::strict();
        assert!(!strict.allow_overlapping);
        assert!(!strict.allow_orphans);

        let permissive = CoherenceInfo::permissive();
        assert!(permissive.allow_overlapping);
        assert!(permissive.allow_orphans);
    }

    #[test]
    fn test_advanced_env_creation() {
        let env = AdvancedTypeClassEnv::new();
        assert!(env.advanced_classes.is_empty());
        assert!(env.type_families.is_empty());

        let default_env = AdvancedTypeClassEnv::default();
        assert!(!default_env.advanced_classes.is_empty());
        assert!(!default_env.type_families.is_empty());
    }
}