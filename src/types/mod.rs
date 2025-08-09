//! Type system implementation for Lambdust.
//!
//! This module provides the four-level gradual type system:
//! 1. Dynamic (default, R7RS compatible)
//! 2. Contracts (runtime checking)
//! 3. Static (compile-time checking with Hindley-Milner inference) 
//! 4. Dependent (experimental, basic framework)

#![allow(missing_docs)]

use crate::diagnostics::Span;
use std::collections::HashSet;
use std::fmt;

pub mod substitution;
pub mod unification;
pub mod constraints;
pub mod inference;
pub mod type_classes;
pub mod gradual;
pub mod algebraic;
pub mod advanced_type_classes;
pub mod r7rs_integration;
pub mod integration_bridge;

// Individual structure modules
pub mod type_var;
pub mod constraint;
pub mod row;
pub mod type_scheme;
pub mod type_env;
pub mod type_constructor;
pub mod type_checker;

// Re-export main types - order matters for dependencies
pub use substitution::*;
pub use constraints::*;
pub use unification::*;
pub use inference::*;
pub use type_classes::*;
pub use gradual::*;
pub use algebraic::*;
pub use advanced_type_classes::*;
pub use r7rs_integration::*;
pub use integration_bridge::*;

// Re-export individual structures
pub use type_var::*;
pub use constraint::*;
pub use row::*;
pub use type_scheme::*;
pub use type_env::*;
pub use type_constructor::*;
pub use type_checker::*;

/// Type system levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeLevel {
    /// Dynamic typing (default)
    Dynamic,
    /// Contract-based typing
    Contracts,
    /// Static typing
    Static,
    /// Dependent typing
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
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    // ============= TYPE VARIABLES =============
    
    /// Type variable for inference
    Variable(TypeVar),
    
    // ============= TYPE CONSTRUCTORS =============
    
    /// Type constructor (like Maybe, IO, etc.)
    Constructor {
        name: String,
        kind: Kind,
    },
    /// Type application (F A)
    Application {
        constructor: Box<Type>,
        argument: Box<Type>,
    },
    
    // ============= POLYMORPHISM =============
    
    /// Universal quantification (∀ a b. Type)
    Forall {
        vars: Vec<TypeVar>,
        body: Box<Type>,
    },
    /// Existential quantification (∃ a. Type)
    Exists {
        vars: Vec<TypeVar>,
        body: Box<Type>,
    },
    
    // ============= TYPE CLASSES =============
    
    /// Type with constraints (Show a => a -> String)
    Constrained {
        constraints: Vec<Constraint>,
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
        input: Box<Type>,
        effects: Vec<Effect>,
        output: Box<Type>,
    },
    
    // ============= ADVANCED TYPES =============
    
    /// Record type (row polymorphism)
    Record(Row),
    /// Variant type (sum types)
    Variant(Row),
    /// Recursive type (μ t. Type)
    Recursive {
        var: TypeVar,
        body: Box<Type>,
    },
}

// TypeVar moved to type_var.rs

/// Kind of a type (type of types).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Type kind (*)
    Type,
    /// Type constructor kind (* -> *)
    Arrow(Box<Kind>, Box<Kind>),
    /// Row kind (for record types)
    Row,
    /// Effect kind
    Effect,
}

// Constraint moved to constraint.rs

/// Effect in the effect system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// IO effect
    IO,
    /// State effect with state type
    State(Type),
    /// Exception effect with exception type
    Exception(Type),
    /// Custom effect
    Custom(String),
    /// Pure computation (no effects)
    Pure,
    /// Error effect (simplified)
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
            Type::Function { params, return_type } => {
                params.iter().any(|p| p.contains_var(var)) || return_type.contains_var(var)
            }
            Type::Application { constructor, argument } => {
                constructor.contains_var(var) || argument.contains_var(var)
            }
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
                row.fields.values().any(|t| t.contains_var(var)) ||
                (row.rest.as_ref() == Some(var))
            }
            Type::Recursive { var: rv, body } => {
                rv != var && body.contains_var(var)
            }
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
            Type::Function { params, return_type } => {
                for param in params {
                    param.collect_free_vars(vars, bound);
                }
                return_type.collect_free_vars(vars, bound);
            }
            Type::Application { constructor, argument } => {
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
            Type::Function { params, return_type } => {
                write!(f, "(->")?;
                for param in params {
                    write!(f, " {param}")?;
                }
                write!(f, " {return_type})")
            }
            Type::Constructor { name, .. } => write!(f, "{name}"),
            Type::Application { constructor, argument } => {
                write!(f, "({constructor} {argument})")
            }
            Type::Forall { vars, body } => {
                write!(f, "(∀ (")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
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
                    if i > 0 { write!(f, ", ")?; }
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