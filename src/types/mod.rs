//! Type system implementation for Lambdust.
//!
//! This module provides the four-level gradual type system implementing a sophisticated
//! type-theoretic foundation for the Lambdust Scheme dialect. The system supports
//! gradual migration from dynamic to static typing while maintaining R7RS compatibility.
//!
//! ## Architectural Overview
//!
//! The type system is built on a modern type-theoretic foundation with four distinct levels:
//!
//! ### 1. Dynamic Type Level (Default, R7RS Compatible)
//! - **Purpose**: Maintains full compatibility with R7RS Scheme
//! - **Computational Model**: Dynamic dispatch with runtime type checking
//! - **Performance**: Optimized for flexibility over speed
//! - **Memory Layout**: Tagged union representation for all values
//! - **R7RS Compliance**: 100% compatible with existing Scheme code
//!
//! ### 2. Contract Type Level (Runtime Checking)
//! - **Purpose**: Gradual introduction of type safety through contracts
//! - **Computational Model**: Runtime contract verification with blame tracking
//! - **Performance**: Moderate overhead for contract boundary checks
//! - **Integration**: Seamless interoperability with dynamic code
//! - **Error Handling**: Precise blame assignment for contract violations
//!
//! ### 3. Static Type Level (Hindley-Milner Inference)
//! - **Purpose**: Compile-time type safety with automatic inference
//! - **Algorithmic Complexity**: O(n log n) for most expressions via Algorithm W
//! - **Computational Model**: Principal type inference with unification
//! - **Memory Layout**: Optimized representations based on inferred types
//! - **Performance**: Zero runtime type checking overhead
//!
//! ### 4. Dependent Type Level (Experimental)
//! - **Purpose**: Advanced type-level computation and verification
//! - **Computational Model**: Type-level evaluation during compilation
//! - **Theory**: Based on Martin-Löf type theory with computational content
//! - **Integration**: Designed for gradual migration from static types
//! - **Applications**: Refinement types, indexed types, proof carrying code
//!
//! ## Core Type Representation
//!
//! The `Type` enum provides a unified representation across all type levels:
//!
//! - **Basic Types**: Number, String, Symbol, Boolean, Char, Bytevector
//! - **Compound Types**: Pair, List, Vector with parametric polymorphism
//! - **Function Types**: Full support for higher-order functions with effects
//! - **Type Variables**: Unification variables for inference algorithms
//! - **Polymorphic Types**: Universal and existential quantification
//! - **Effect Types**: Monadic effects with computational contexts
//!
//! ## Key Algorithms
//!
//! ### Hindley-Milner Type Inference
//! - **Algorithm**: Damas-Milner Algorithm W with optimizations
//! - **Time Complexity**: O(n log n) average case, O(n²) worst case
//! - **Space Complexity**: O(n) for constraint generation
//! - **Optimizations**: Occurs check optimization, level-based generalization
//!
//! ### Unification Algorithm
//! - **Algorithm**: Robinson's unification with path compression
//! - **Time Complexity**: Nearly linear with union-find optimizations
//! - **Space Complexity**: O(n) for substitution representation
//! - **Features**: Occurs check, infinite type detection
//!
//! ### Constraint Solving
//! - **Algorithm**: Constraint-based type inference with delayed solving
//! - **Approach**: Generate constraints then solve via fixed-point iteration
//! - **Complexity**: Polynomial in constraint size with good heuristics
//! - **Benefits**: Better error messages, supports advanced features
//!
//! ## Dependencies and Integration Points
//!
//! - **Core Dependencies**: `diagnostics::Span` for source location tracking
//! - **AST Integration**: Type annotations and inference for `ast::Expr`
//! - **Evaluation Integration**: Type-directed optimizations in `eval` module
//! - **Effect System**: Coordination with `effects` module for monadic types
//! - **Contract System**: Runtime integration with `contracts` module
//!
//! ## Performance Characteristics
//!
//! - **Type Checking**: O(n log n) for most programs via Algorithm W
//! - **Memory Usage**: Compact representation with sharing for common types
//! - **Inference Speed**: Optimized for interactive development (sub-second)
//! - **Optimization Opportunities**: Type-directed code specialization
//!
//! ## Module Organization
//!
//! The type system is organized into focused modules:
//! - Core type definitions and algorithms (this module)
//! - Specialized inference engines (`inference.rs`, `unification.rs`)
//! - Gradual typing integration (`gradual.rs`, `gradual_system.rs`)
//! - Advanced features (`dependent/`, `type_classes.rs`)
//! - R7RS integration and compatibility (`r7rs_integration.rs`)

use crate::diagnostics::Span;
use std::collections::HashSet;
use std::fmt;

/// Advanced type class system with higher-kinded types and functional dependencies
pub mod advanced_type_classes;
/// Algebraic data types and sum/product type construction
pub mod algebraic;
/// Type constraint generation, solving, and satisfaction checking
pub mod constraints;
/// Dependent type system with type-level computation and proof construction
pub mod dependent;
/// Gradual typing system for smooth static/dynamic integration
pub mod gradual;
/// Consistency checking for gradual type transitions
pub mod gradual_consistency;
/// Integration between gradual types and runtime contract system
pub mod gradual_contract_integration;
/// Bridge between gradual types and expression evaluator
pub mod gradual_evaluator_integration;
/// Type inference algorithms for gradual typing
pub mod gradual_inference;
/// Complete gradual typing system implementation
pub mod gradual_system;
/// Hindley-Milner type inference with extensions
pub mod inference;
/// Integration bridge for connecting different type system components
pub mod integration_bridge;
/// R7RS Scheme compatibility layer for type system
pub mod r7rs_integration;
/// Type variable substitution algorithms and utilities
pub mod substitution;
/// Type class system with instance resolution
pub mod type_classes;
/// Bridge between type system and AST expression types
pub mod type_expr_bridge;
/// Unification algorithms for type inference and checking
pub mod unification;
/// Unified error reporting system for type-related errors
pub mod unified_type_errors;

// New Generic Type System Framework
/// Experimental dependent type system with full type-level computation
#[cfg(feature = "experimental-type-system")]
pub mod dependent_type_system;
/// Generic type inference framework supporting multiple inference algorithms
#[cfg(feature = "experimental-type-system")]
pub mod generic_type_inference;
/// Generic type system architecture with pluggable components
#[cfg(feature = "experimental-type-system")]
pub mod generic_type_system;
/// Classical Hindley-Milner type system with modern extensions
#[cfg(feature = "experimental-type-system")]
pub mod hindley_milner_system;
/// Monadic type system with effect tracking and computation contexts
#[cfg(feature = "experimental-type-system")]
pub mod monad_aware_system;

// Note: Temporarily disabled modules that depend on old dependent type system
// pub mod dependent_bridge;
// pub mod gradual_dependent;
// pub mod type_level_computation;
// pub mod scheme_dependent_integration;
// pub mod proof_assistant;

// Individual structure modules
/// Type constraint system for expressing and solving type relationships
pub mod constraint;
/// Row type system for extensible records and structural typing
pub mod row;
/// Type checker implementation with inference and unification
pub mod type_checker;
/// Type constructor definitions and operations
pub mod type_constructor;
/// Type environment management for variable binding and scope
pub mod type_env;
/// Type scheme representation with polymorphic quantification
pub mod type_scheme;
/// Type variable management for unification and substitution
pub mod type_var;

// Re-export main types - order matters for dependencies
pub use advanced_type_classes::*;
pub use algebraic::*;
pub use constraints::*;
pub use dependent::*;
pub use gradual::*;
pub use gradual_consistency::*;
pub use gradual_contract_integration::*;
pub use gradual_evaluator_integration::*;
pub use gradual_inference::*;
pub use gradual_system::*;
pub use inference::*;
pub use integration_bridge::*;
pub use r7rs_integration::*;
pub use substitution::*;
pub use type_classes::*;
pub use type_expr_bridge::*;
pub use unification::*;

// Re-export generic type system framework - temporarily disable problematic modules
#[cfg(feature = "experimental-type-system")]
pub use dependent_type_system::*;
#[cfg(feature = "experimental-type-system")]
pub use generic_type_inference::{GenericInferenceEngine, InferenceResult};
#[cfg(feature = "experimental-type-system")]
pub use generic_type_system::{
    ConstraintSystem, ExpressionRepr, ExpressionVisitor, InferenceEngine, TypeContext, TypeRepr,
    TypeSystem, TypeSystemCapabilities,
};
#[cfg(feature = "experimental-type-system")]
pub use hindley_milner_system::*;
#[cfg(feature = "experimental-type-system")]
pub use monad_aware_system::*;

// pub use gradual_dependent::*;
// pub use type_level_computation::*;
// pub use scheme_dependent_integration::*;
// pub use proof_assistant::*;

// Re-export individual structures
pub use constraint::*;
pub use row::*;
pub use type_checker::*;
pub use type_constructor::*;
pub use type_env::*;
pub use type_scheme::*;
pub use type_var::*;

/// Type system levels representing the gradual typing hierarchy.
///
/// This enum defines the four-level gradual type system, where programs can
/// migrate progressively from dynamic to dependent typing while maintaining
/// interoperability between levels.
///
/// ## Design Principles
///
/// - **Gradual Migration**: Higher levels provide more guarantees but require more annotations
/// - **Backward Compatibility**: Lower levels remain fully functional
/// - **Interoperability**: Seamless interaction between different levels
/// - **Performance Scaling**: Higher levels enable better optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeLevel {
    /// Dynamic typing (default, R7RS compatible).
    ///
    /// - All values carry runtime type information
    /// - Type errors detected at runtime
    /// - Maximum flexibility, minimal annotations
    /// - Full R7RS Scheme compatibility
    Dynamic,

    /// Contract-based typing with runtime verification.
    ///
    /// - Contracts specify expected behavior
    /// - Violations detected at runtime with blame assignment
    /// - Gradual introduction of type safety
    /// - Interoperable with dynamic code
    Contracts,

    /// Static typing with Hindley-Milner inference.
    ///
    /// - Types inferred automatically where possible
    /// - Compile-time type checking and optimization
    /// - Principal types guarantee most general solutions
    /// - Zero runtime type checking overhead
    Static,

    /// Dependent typing (experimental).
    ///
    /// - Types can depend on values
    /// - Compile-time verification of program properties
    /// - Advanced features like refinement types
    /// - Research-level implementation
    Dependent,
}

// Type variable counter moved to type_var.rs

/// A type in the Lambdust type system.
///
/// This represents all type constructs from the specification:
/// - Basic types (Number, String, Symbol, Boolean, Char)
/// - Compound types (Pair, List, Vector)
/// - Function types (->)
/// - Type variables for inference
/// - Type constructors and applications
/// - Gradual types (Dynamic)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ============= BASIC TYPES =============
    /// Number type
    Number,
    /// String type
    String,
    /// Symbol type
    Symbol,
    /// Boolean type
    Boolean,
    /// Character type
    Char,
    /// Bytevector type
    Bytevector,
    /// Unit type (empty tuple)
    Unit,

    // ============= COMPOUND TYPES =============
    /// Pair type (Pair A B)
    Pair(Box<Type>, Box<Type>),
    /// List type (List A)
    List(Box<Type>),
    /// Vector type (Vector A)
    Vector(Box<Type>),
    /// Function type (-> A B C ... Z)
    Function {
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },

    // ============= TYPE VARIABLES =============
    /// Type variable for inference
    Variable(TypeVar),

    // ============= TYPE CONSTRUCTORS =============
    /// Type constructor (like Maybe, IO, etc.)
    Constructor {
        /// Name of the type constructor
        name: String,
        /// Kind of the type constructor
        kind: Kind,
    },
    /// Type application (F A)
    Application {
        /// Type constructor being applied
        constructor: Box<Type>,
        /// Argument type being applied to constructor
        argument: Box<Type>,
    },

    // ============= POLYMORPHISM =============
    /// Universal quantification (∀ a b. Type)
    Forall {
        /// Quantified type variables
        vars: Vec<TypeVar>,
        /// Body type with quantified variables
        body: Box<Type>,
    },
    /// Existential quantification (∃ a. Type)
    Exists {
        /// Quantified type variables
        vars: Vec<TypeVar>,
        /// Body type with quantified variables
        body: Box<Type>,
    },

    // ============= TYPE CLASSES =============
    /// Type with constraints (Show a => a -> String)
    Constrained {
        /// Type class constraints that must be satisfied
        constraints: Vec<Constraint>,
        /// The constrained type
        type_: Box<Type>,
    },

    // ============= GRADUAL TYPING =============
    /// Dynamic type (gradual typing)
    Dynamic,
    /// Unknown type (inference placeholder)
    Unknown,

    // ============= EFFECTS =============
    /// Type with effects (a ~> IO b)
    Effectful {
        /// Input type
        input: Box<Type>,
        /// Effects that may occur
        effects: Vec<Effect>,
        /// Output type after applying effects
        output: Box<Type>,
    },

    // ============= ADVANCED TYPES =============
    /// Record type (row polymorphism)
    Record(Row),
    /// Variant type (sum types)
    Variant(Row),
    /// Recursive type (μ t. Type)
    Recursive {
        /// The type variable being recursively bound
        var: TypeVar,
        /// The body of the recursive type
        body: Box<Type>,
    },
}

// TypeVar moved to type_var.rs

/// Kind of a type (type of types).
///
/// In type theory, kinds classify types just as types classify values.
/// This system uses a simple kind language sufficient for Lambdust's type features.
///
/// ## Kind Theory
///
/// - **Type Kind**: Proper types that classify values (like `Number`, `String`)
/// - **Arrow Kind**: Type constructors that take types to produce types
/// - **Row Kind**: Structural types for records and variants (row polymorphism)
/// - **Effect Kind**: Computational effects and monadic contexts
///
/// ## Examples
///
/// - `Number : Type` (Number has kind Type)
/// - `List : Type -> Type` (List constructor takes a type to produce a type)
/// - `{x : Number | r} : Row` (Row types with field information)
/// - `IO : Type -> Type` (IO effect constructor)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Type kind (*) - classifies proper types that have values.
    ///
    /// Examples: `Number`, `String`, `Boolean`, `(List Number)`
    Type,

    /// Type constructor kind (* -> *) - classifies type-level functions.
    ///
    /// Takes a kind and produces a kind. Used for parameterized types.
    /// Examples: `List : Type -> Type`, `-> : Type -> Type -> Type`
    Arrow(Box<Kind>, Box<Kind>),

    /// Row kind - classifies record and variant type structures.
    ///
    /// Used in row polymorphism for extensible records and variants.
    /// Examples: `{x : Number | r}`, `[Left Number | Right String]`
    Row,

    /// Effect kind - classifies computational effects.
    ///
    /// Used in the effect system to track side effects and monadic computations.
    /// Examples: `IO`, `State Number`, `Exception String`
    Effect,
}

// Constraint moved to constraint.rs

/// Effect in the effect system for tracking computational side effects.
///
/// The effect system provides a way to track and control side effects in
/// computations while maintaining the functional programming paradigm through
/// monadic abstractions.
///
/// ## Design Principles
///
/// - **Effect Tracking**: All side effects are explicitly tracked at the type level
/// - **Monadic Interface**: Effects are handled through monad transformers
/// - **Composability**: Effects can be combined and nested
/// - **Purity Preservation**: Pure computations are distinguished from effectful ones
/// - **R7RS Compatibility**: Traditional Scheme side effects are supported
///
/// ## Effect Categories
///
/// - **I/O Effects**: File system, network, and console operations
/// - **State Effects**: Mutable state with typed state variables
/// - **Exception Effects**: Typed exception handling and control flow
/// - **Custom Effects**: User-defined effects for domain-specific needs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// I/O effect for input/output operations.
    ///
    /// Represents computations that interact with the external world:
    /// - File system operations
    /// - Network communication  
    /// - Console I/O
    /// - System calls
    IO,

    /// State effect with typed state variable.
    ///
    /// Represents stateful computations with a specific state type.
    /// The state type parameter ensures type safety for state operations.
    State(Type),

    /// Mutation effect for in-place updates.
    ///
    /// Represents computations that perform destructive updates:
    /// - Vector/string mutations
    /// - Hash table modifications
    /// - Mutable reference updates
    Mutation,

    /// Exception effect with typed exception values.
    ///
    /// Represents computations that may raise exceptions of a specific type.
    /// Enables typed exception handling and error recovery.
    Exception(Type),

    /// Custom user-defined effect.
    ///
    /// Allows users to define domain-specific effects with custom names.
    /// The string identifier distinguishes different custom effects.
    Custom(String),

    /// Pure computation (no effects).
    ///
    /// Represents computations with no side effects - functions that
    /// depend only on their inputs and produce deterministic outputs.
    Pure,

    /// Error effect for simplified error handling.
    ///
    /// A simplified form of exception handling without specific error types.
    /// Used for basic error propagation and failure modes.
    Error,
}

// Row moved to row.rs

// TypeScheme moved to type_scheme.rs

// TypeEnv moved to type_env.rs

// TypeClassInstance moved to type_class_instance.rs

// TypeConstructor moved to type_constructor.rs

// TypeConstraint moved to constraint.rs

// TypeChecker moved to type_checker.rs

// TypeVar implementations moved to type_var.rs

impl Type {
    /// Creates a fresh type variable.
    pub fn fresh_var() -> Self {
        Type::Variable(TypeVar::new())
    }

    /// Creates a named type variable.
    pub fn named_var(name: impl Into<String>) -> Self {
        Type::Variable(TypeVar::with_name(name))
    }

    /// Creates a function type from parameter types and return type.
    pub fn function(params: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    /// Creates a pair type.
    pub fn pair(first: Type, second: Type) -> Self {
        Type::Pair(Box::new(first), Box::new(second))
    }

    /// Creates a list type.
    pub fn list(element_type: Type) -> Self {
        Type::List(Box::new(element_type))
    }

    /// Creates a vector type.
    pub fn vector(element_type: Type) -> Self {
        Type::Vector(Box::new(element_type))
    }

    /// Creates a universal quantification.
    pub fn forall(vars: Vec<TypeVar>, body: Type) -> Self {
        if vars.is_empty() {
            body
        } else {
            Type::Forall {
                vars,
                body: Box::new(body),
            }
        }
    }

    /// Creates a constrained type.
    pub fn constrained(constraints: Vec<Constraint>, type_: Type) -> Self {
        if constraints.is_empty() {
            type_
        } else {
            Type::Constrained {
                constraints,
                type_: Box::new(type_),
            }
        }
    }

    /// Returns true if this type is a type variable.
    pub fn is_variable(&self) -> bool {
        matches!(self, Type::Variable(_))
    }

    /// Returns true if this type is a function type.
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    /// Returns true if this type is polymorphic.
    pub fn is_polymorphic(&self) -> bool {
        matches!(self, Type::Forall { .. } | Type::Exists { .. })
    }

    /// Returns true if this type contains the given type variable.
    pub fn contains_var(&self, var: &TypeVar) -> bool {
        match self {
            Type::Variable(v) => v == var,
            Type::Pair(a, b) => a.contains_var(var) || b.contains_var(var),
            Type::List(t) | Type::Vector(t) => t.contains_var(var),
            Type::Function {
                params,
                return_type,
            } => params.iter().any(|p| p.contains_var(var)) || return_type.contains_var(var),
            Type::Application {
                constructor,
                argument,
            } => constructor.contains_var(var) || argument.contains_var(var),
            Type::Forall { vars, body } | Type::Exists { vars, body } => {
                !vars.contains(var) && body.contains_var(var)
            }
            Type::Constrained { constraints, type_ } => {
                constraints.iter().any(|c| c.type_.contains_var(var)) || type_.contains_var(var)
            }
            Type::Effectful { input, output, .. } => {
                input.contains_var(var) || output.contains_var(var)
            }
            Type::Record(row) | Type::Variant(row) => {
                row.fields.values().any(|t| t.contains_var(var)) || (row.rest.as_ref() == Some(var))
            }
            Type::Recursive { var: rv, body } => rv != var && body.contains_var(var),
            _ => false,
        }
    }

    /// Gets all free type variables in this type.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut vars = HashSet::new();
        self.collect_free_vars(&mut vars, &HashSet::new());
        vars
    }

    fn collect_free_vars(&self, vars: &mut HashSet<TypeVar>, bound: &HashSet<TypeVar>) {
        match self {
            Type::Variable(v) => {
                if !bound.contains(v) {
                    vars.insert(v.clone());
                }
            }
            Type::Pair(a, b) => {
                a.collect_free_vars(vars, bound);
                b.collect_free_vars(vars, bound);
            }
            Type::List(t) | Type::Vector(t) => {
                t.collect_free_vars(vars, bound);
            }
            Type::Function {
                params,
                return_type,
            } => {
                for param in params {
                    param.collect_free_vars(vars, bound);
                }
                return_type.collect_free_vars(vars, bound);
            }
            Type::Application {
                constructor,
                argument,
            } => {
                constructor.collect_free_vars(vars, bound);
                argument.collect_free_vars(vars, bound);
            }
            Type::Forall { vars: qvars, body } | Type::Exists { vars: qvars, body } => {
                let mut new_bound = bound.clone();
                new_bound.extend(qvars.iter().cloned());
                body.collect_free_vars(vars, &new_bound);
            }
            Type::Constrained { constraints, type_ } => {
                for constraint in constraints {
                    constraint.type_.collect_free_vars(vars, bound);
                }
                type_.collect_free_vars(vars, bound);
            }
            Type::Effectful { input, output, .. } => {
                input.collect_free_vars(vars, bound);
                output.collect_free_vars(vars, bound);
            }
            Type::Record(row) | Type::Variant(row) => {
                for field_type in row.fields.values() {
                    field_type.collect_free_vars(vars, bound);
                }
                if let Some(rest_var) = &row.rest {
                    if !bound.contains(rest_var) {
                        vars.insert(rest_var.clone());
                    }
                }
            }
            Type::Recursive { var, body } => {
                let mut new_bound = bound.clone();
                new_bound.insert(var.clone());
                body.collect_free_vars(vars, &new_bound);
            }
            _ => {} // Base types have no variables
        }
    }
}

impl Kind {
    /// Creates an arrow kind.
    pub fn arrow(from: Kind, to: Kind) -> Self {
        Kind::Arrow(Box::new(from), Box::new(to))
    }

    /// Returns the arity of this kind (number of arguments it takes).
    pub fn arity(&self) -> usize {
        match self {
            Kind::Type | Kind::Row | Kind::Effect => 0,
            Kind::Arrow(_, to) => 1 + to.arity(),
        }
    }
}

// Row implementations moved to row.rs

// Row implementations moved to row.rs

// TypeScheme implementations moved to type_scheme.rs

// TypeEnv implementations moved to type_env.rs

// TypeChecker implementations moved to type_checker.rs

// Default implementations moved to respective files

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::String => write!(f, "String"),
            Type::Symbol => write!(f, "Symbol"),
            Type::Boolean => write!(f, "Boolean"),
            Type::Char => write!(f, "Char"),
            Type::Unit => write!(f, "()"),
            Type::Dynamic => write!(f, "Dynamic"),
            Type::Unknown => write!(f, "?"),
            Type::Variable(var) => {
                if let Some(name) = &var.name {
                    write!(f, "{name}")
                } else {
                    write!(f, "t{}", var.id)
                }
            }
            Type::Pair(a, b) => write!(f, "(Pair {a} {b})"),
            Type::List(t) => write!(f, "(List {t})"),
            Type::Vector(t) => write!(f, "(Vector {t})"),
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "(->")?;
                for param in params {
                    write!(f, " {param}")?;
                }
                write!(f, " {return_type})")
            }
            Type::Constructor { name, .. } => write!(f, "{name}"),
            Type::Application {
                constructor,
                argument,
            } => {
                write!(f, "({constructor} {argument})")
            }
            Type::Forall { vars, body } => {
                write!(f, "(∀ (")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    if let Some(name) = &var.name {
                        write!(f, "{name}")?;
                    } else {
                        write!(f, "t{}", var.id)?;
                    }
                }
                write!(f, ") {body})")
            }
            Type::Constrained { constraints, type_ } => {
                write!(f, "(")?;
                for (i, constraint) in constraints.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", constraint.class, constraint.type_)?;
                }
                write!(f, " => {type_})")
            }
            // Add more display implementations as needed
            _ => write!(f, "<{self:?}"),
        }
    }
}

// Display implementations moved to respective files

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Type => write!(f, "*"),
            Kind::Row => write!(f, "Row"),
            Kind::Effect => write!(f, "Effect"),
            Kind::Arrow(from, to) => write!(f, "({from} -> {to})"),
        }
    }
}
