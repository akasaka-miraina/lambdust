//! Type class system for Lambdust.
//!
//! This module implements Haskell-style type classes with:
//! - Type class definitions with kinds and methods
//! - Type class instances
//! - Constraint solving and instance resolution
//! - Built-in type classes (Eq, Ord, Show, Num, etc.)

#![allow(missing_docs)]

use super::{Type, TypeVar, TypeScheme, Kind, Constraint};
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// A type class definition.
#[derive(Debug, Clone)]
pub struct TypeClass {
    /// Name of the type class
    pub name: String,
    /// Kind of the type parameter
    pub kind: Kind,
    /// Superclasses (type classes that this extends)
    pub superclasses: Vec<String>,
    /// Method signatures
    pub methods: HashMap<String, TypeScheme>,
    /// Default method implementations
    pub defaults: HashMap<String, DefaultImpl>,
    /// Associated types
    pub associated_types: HashMap<String, Kind>,
}

/// A type class instance.
#[derive(Debug, Clone)]
pub struct TypeClassInstance {
    /// Name of the type class
    pub class: String,
    /// The type being made an instance
    pub instance_type: Type,
    /// Instance constraints (context)
    pub constraints: Vec<Constraint>,
    /// Method implementations
    pub methods: HashMap<String, MethodImpl>,
    /// Source span for error reporting
    pub span: Option<Span>,
}

/// Default method implementation.
#[derive(Debug, Clone)]
pub enum DefaultImpl {
    /// Implementation provided as an expression
    Expression(String), // For now, just store as string
    /// No default implementation
    None,
}

/// Method implementation in an instance.
#[derive(Debug, Clone)]
pub enum MethodImpl {
    /// Implementation provided as an expression
    Expression(String), // For now, just store as string
    /// Use default implementation
    Default,
}

/// Type class environment managing all type classes and instances.
#[derive(Debug, Clone)]
pub struct TypeClassEnv {
    /// Defined type classes
    pub classes: HashMap<String, TypeClass>,
    /// Type class instances
    pub instances: HashMap<String, Vec<TypeClassInstance>>,
}

/// Context for type class constraint solving.
#[derive(Debug)]
pub struct ConstraintContext {
    /// Available instances
    #[allow(dead_code)]
    instances: HashMap<String, Vec<TypeClassInstance>>,
    /// Currently being resolved (to prevent infinite recursion)
    resolving: Vec<Constraint>,
}

impl TypeClass {
    /// Creates a new type class.
    pub fn new(name: impl Into<String>, kind: Kind) -> Self {
        Self {
            name: name.into(),
            kind,
            superclasses: Vec::new(),
            methods: HashMap::new(),
            defaults: HashMap::new(),
            associated_types: HashMap::new(),
        }
    }
    
    /// Adds a superclass.
    pub fn with_superclass(mut self, superclass: impl Into<String>) -> Self {
        self.superclasses.push(superclass.into())
        self
    }
    
    /// Adds a method.
    pub fn with_method(mut self, name: impl Into<String>, type_scheme: TypeScheme) -> Self {
        self.methods.insert(name.into(), type_scheme);
        self
    }
    
    /// Adds a default implementation.
    pub fn with_default(mut self, name: impl Into<String>, impl_: DefaultImpl) -> Self {
        self.defaults.insert(name.into(), impl_);
        self
    }
    
    /// Checks if this type class is a superclass of another.
    pub fn is_superclass_of(&self, other: &TypeClass, env: &TypeClassEnv) -> bool {
        if other.superclasses.contains(&self.name) {
            return true;
        }
        
        // Check transitively
        for superclass_name in &other.superclasses {
            if let Some(superclass) = env.classes.get(superclass_name) {
                if self.is_superclass_of(superclass, env) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Gets all superclass constraints for a given type.
    pub fn superclass_constraints(&self, type_: &Type, env: &TypeClassEnv) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        
        for superclass_name in &self.superclasses {
            constraints.push(Constraint {
                class: superclass_name.clone()),
                type_: type_.clone()),
            });
            
            // Add transitive superclass constraints
            if let Some(superclass) = env.classes.get(superclass_name) {
                constraints.extend(superclass.superclass_constraints(type_, env));
            }
        }
        
        constraints
    }
}

impl TypeClassInstance {
    /// Creates a new type class instance.
    pub fn new(
        class: impl Into<String>,
        instance_type: Type,
        span: Option<Span>,
    ) -> Self {
        Self {
            class: class.into(),
            instance_type,
            constraints: Vec::new(),
            methods: HashMap::new(),
            span,
        }
    }
    
    /// Adds a constraint to the instance.
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Adds a method implementation.
    pub fn with_method(mut self, name: impl Into<String>, impl_: MethodImpl) -> Self {
        self.methods.insert(name.into(), impl_);
        self
    }
    
    /// Checks if this instance matches a given constraint.
    pub fn matches(&self, constraint: &Constraint) -> Option<super::Substitution> {
        if self.class != constraint.class {
            return None;
        }
        
        // Try to unify the instance type with the constraint type
        use crate::types::unification::unify;
        unify(&self.instance_type, &constraint.type_, self.span).ok()
    }
}

impl TypeClassEnv {
    /// Creates a new type class environment.
    pub fn new() -> Self {
        let mut env = Self {
            classes: HashMap::new(),
            instances: HashMap::new(),
        };
        
        // Add built-in type classes
        env.add_builtin_classes();
        env.add_builtin_instances();
        
        env
    }
    
    /// Adds a type class definition.
    pub fn add_class(&mut self, class: TypeClass) {
        let name = class.name.clone());
        self.classes.insert(name, class);
    }
    
    /// Adds a type class instance.
    pub fn add_instance(&mut self, instance: TypeClassInstance) {
        let class_name = instance.class.clone());
        self.instances
            .entry(class_name)
            .or_default()
            .push(instance);
    }
    
    /// Looks up a type class by name.
    pub fn get_class(&self, name: &str) -> Option<&TypeClass> {
        self.classes.get(name)
    }
    
    /// Gets all instances for a type class.
    pub fn get_instances(&self, class: &str) -> Option<&[TypeClassInstance]> {
        self.instances.get(class).map(|v| v.as_slice())
    }
    
    /// Resolves a type class constraint.
    pub fn resolve_constraint(
        &self,
        constraint: &Constraint,
        context: &mut ConstraintContext,
    ) -> Result<TypeClassInstance> {
        // Check for recursion
        if context.resolving.contains(constraint) {
            return Err(Box::new(Error::type_error(
                format!("Recursive constraint resolution for {}", constraint.class),
                Span::new(0, 0),
            ));
        }
        
        context.resolving.push(constraint.clone());
        
        // Look for matching instances
        if let Some(instances) = self.get_instances(&constraint.class) {
            for instance in instances {
                if let Some(subst) = instance.matches(constraint) {
                    // Check that all instance constraints are satisfied
                    let mut satisfied = true;
                    for instance_constraint in &instance.constraints {
                        let resolved_constraint = subst.apply_to_constraint(instance_constraint);
                        if self.resolve_constraint(&resolved_constraint, context).is_err() {
                            satisfied = false;
                            break;
                        }
                    }
                    
                    if satisfied {
                        context.resolving.pop();
                        return Ok(instance.clone());
                    }
                }
            }
        }
        
        context.resolving.pop();
        Err(Box::new(Error::type_error(
            format!("No instance of {} for type {}", constraint.class, constraint.type_),
            Span::new(0, 0),
        ))
    }
    
    /// Adds built-in type classes.
    fn add_builtin_classes(&mut self) {
        // Eq type class
        let eq_class = TypeClass::new("Eq", Kind::Type)
            .with_method("==", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::Boolean,
                    ),
                )
            ))
            .with_method("/=", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::Boolean,
                    ),
                )
            ));
        self.add_class(eq_class);
        
        // Ord type class
        let ord_class = TypeClass::new("Ord", Kind::Type)
            .with_superclass("Eq")
            .with_method("compare", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::Symbol, // Ordering type would be better
                    ),
                )
            ))
            .with_method("<", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::Boolean,
                    ),
                )
            ));
        self.add_class(ord_class);
        
        // Show type class
        let show_class = TypeClass::new("Show", Kind::Type)
            .with_method("show", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(vec![Type::named_var("a")], Type::String),
                )
            ));
        self.add_class(show_class);
        
        // Num type class
        let num_class = TypeClass::new("Num", Kind::Type)
            .with_superclass("Eq")
            .with_superclass("Show")
            .with_method("+", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::named_var("a"),
                    ),
                )
            ))
            .with_method("-", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::named_var("a"),
                    ),
                )
            ))
            .with_method("*", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a"), Type::named_var("a")],
                        Type::named_var("a"),
                    ),
                )
            ));
        self.add_class(num_class);
        
        // Functor type class
        let functor_class = TypeClass::new("Functor", Kind::arrow(Kind::Type, Kind::Type))
            .with_method("map", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("f"), TypeVar::with_name("a"), TypeVar::with_name("b")],
                    Type::function(
                        vec![
                            Type::function(vec![Type::named_var("a")], Type::named_var("b")),
                            Type::Application {
                                constructor: Box::new(Type::named_var("f")),
                                argument: Box::new(Type::named_var("a")),
                            },
                        ],
                        Type::Application {
                            constructor: Box::new(Type::named_var("f")),
                            argument: Box::new(Type::named_var("b")),
                        },
                    ),
                )
            ));
        self.add_class(functor_class);
        
        // Monad type class
        let monad_class = TypeClass::new("Monad", Kind::arrow(Kind::Type, Kind::Type))
            .with_superclass("Functor")
            .with_method("return", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("m"), TypeVar::with_name("a")],
                    Type::function(
                        vec![Type::named_var("a")],
                        Type::Application {
                            constructor: Box::new(Type::named_var("m")),
                            argument: Box::new(Type::named_var("a")),
                        },
                    ),
                )
            ))
            .with_method(">>=", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("m"), TypeVar::with_name("a"), TypeVar::with_name("b")],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::named_var("m")),
                                argument: Box::new(Type::named_var("a")),
                            },
                            Type::function(
                                vec![Type::named_var("a")],
                                Type::Application {
                                    constructor: Box::new(Type::named_var("m")),
                                    argument: Box::new(Type::named_var("b")),
                                },
                            ),
                        ],
                        Type::Application {
                            constructor: Box::new(Type::named_var("m")),
                            argument: Box::new(Type::named_var("b")),
                        },
                    ),
                )
            ));
        self.add_class(monad_class);
        
        // Default type class
        let default_class = TypeClass::new("Default", Kind::Type)
            .with_method("default", TypeScheme::monomorphic(
                Type::forall(
                    vec![TypeVar::with_name("a")],
                    Type::named_var("a"),
                )
            ));
        self.add_class(default_class);
    }
    
    /// Adds built-in type class instances.
    fn add_builtin_instances(&mut self) {
        // Eq instances for basic types
        self.add_instance(TypeClassInstance::new("Eq", Type::Number, None));
        self.add_instance(TypeClassInstance::new("Eq", Type::String, None));
        self.add_instance(TypeClassInstance::new("Eq", Type::Boolean, None));
        self.add_instance(TypeClassInstance::new("Eq", Type::Char, None));
        self.add_instance(TypeClassInstance::new("Eq", Type::Symbol, None));
        
        // Ord instances for basic types
        self.add_instance(TypeClassInstance::new("Ord", Type::Number, None));
        self.add_instance(TypeClassInstance::new("Ord", Type::String, None));
        self.add_instance(TypeClassInstance::new("Ord", Type::Boolean, None));
        self.add_instance(TypeClassInstance::new("Ord", Type::Char, None));
        
        // Show instances for basic types
        self.add_instance(TypeClassInstance::new("Show", Type::Number, None));
        self.add_instance(TypeClassInstance::new("Show", Type::String, None));
        self.add_instance(TypeClassInstance::new("Show", Type::Boolean, None));
        self.add_instance(TypeClassInstance::new("Show", Type::Char, None));
        self.add_instance(TypeClassInstance::new("Show", Type::Symbol, None));
        
        // Num instances
        self.add_instance(TypeClassInstance::new("Num", Type::Number, None));
        
        // Default instances for basic types
        self.add_instance(TypeClassInstance::new("Default", Type::Number, None));
        self.add_instance(TypeClassInstance::new("Default", Type::String, None));
        self.add_instance(TypeClassInstance::new("Default", Type::Boolean, None));
        self.add_instance(TypeClassInstance::new("Default", Type::Char, None));
        
        // Functor instances
        self.add_instance(TypeClassInstance::new(
            "Functor",
            Type::Constructor {
                name: "List".to_string(),
                kind: Kind::arrow(Kind::Type, Kind::Type),
            },
            None,
        ));
        
        // Monad instances
        self.add_instance(TypeClassInstance::new(
            "Monad",
            Type::Constructor {
                name: "List".to_string(),
                kind: Kind::arrow(Kind::Type, Kind::Type),
            },
            None,
        ));
    }
}

impl ConstraintContext {
    /// Creates a new constraint context.
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            resolving: Vec::new(),
        }
    }
    
    /// Creates a context with the given instances.
    pub fn with_instances(instances: HashMap<String, Vec<TypeClassInstance>>) -> Self {
        Self {
            instances,
            resolving: Vec::new(),
        }
    }
}

impl Default for TypeClassEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConstraintContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TypeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class")?;
        
        if !self.superclasses.is_empty() {
            write!(f, " (")?;
            for (i, superclass) in self.superclasses.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{superclass}")?;
            }
            write!(f, ") =>")?;
        }
        
        write!(f, " {} where", self.name)?;
        
        for (method_name, method_type) in &self.methods {
            write!(f, "\n  {} : {}", method_name, method_type.type_)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for TypeClassInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instance")?;
        
        if !self.constraints.is_empty() {
            write!(f, " (")?;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{} {}", constraint.class, constraint.type_)?;
            }
            write!(f, ") =>")?;
        }
        
        write!(f, " {} {}", self.class, self.instance_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_type_classes() {
        let env = TypeClassEnv::new();
        
        // Check that built-in classes exist
        assert!(env.get_class("Eq").is_some());
        assert!(env.get_class("Ord").is_some());
        assert!(env.get_class("Show").is_some());
        assert!(env.get_class("Num").is_some());
        assert!(env.get_class("Functor").is_some());
        assert!(env.get_class("Monad").is_some());
        
        // Check superclass relationships
        let ord_class = env.get_class("Ord").unwrap();
        assert!(ord_class.superclasses.contains(&"Eq".to_string()));
        
        let num_class = env.get_class("Num").unwrap();
        assert!(num_class.superclasses.contains(&"Eq".to_string()));
        assert!(num_class.superclasses.contains(&"Show".to_string()));
    }

    #[test]
    fn test_builtin_instances() {
        let env = TypeClassEnv::new();
        
        // Check that built-in instances exist
        let eq_instances = env.get_instances("Eq").unwrap();
        assert!(!eq_instances.is_empty());
        
        // Check specific instances
        let number_eq_instance = eq_instances.iter()
            .find(|inst| inst.instance_type == Type::Number);
        assert!(number_eq_instance.is_some());
    }

    #[test]
    fn test_constraint_resolution() {
        let env = TypeClassEnv::new();
        let mut context = ConstraintContext::new();
        
        let constraint = Constraint {
            class: "Eq".to_string(),
            type_: Type::Number,
        };
        
        let result = env.resolve_constraint(&constraint, &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsatisfiable_constraint() {
        let env = TypeClassEnv::new();
        let mut context = ConstraintContext::new();
        
        let constraint = Constraint {
            class: "Eq".to_string(),
            type_: Type::function(vec![Type::Number], Type::Number),
        };
        
        let result = env.resolve_constraint(&constraint, &mut context);
        assert!(result.is_err());
    }
}