//! Advanced type system tests for Lambdust
//! 
//! This module tests the advanced type system features including:
//! - Algebraic data types and type inference
//! - Gradual typing integration with R7RS values
//! - Effect type annotations and constraint solving
//! - Type classes and advanced type features

use lambdust::{
    types::*,
    eval::Value,
    ast::{Literal, Expr},
    diagnostics::{Span, Spanned},
};
use std::collections::HashMap;

// Test algebraic data types
#[test]
fn test_algebraic_data_types() {
    let mut type_checker = TypeChecker::new(TypeLevel::Static);
    
    // Test Maybe type construction
    let maybe_int = AlgebraicDataType::new(
        "Maybe".to_string(),
        vec![TypeVar::with_name("a")],
        vec![
            TypeConstructor::new("Nothing".to_string(), Kind::Type, 0),
            TypeConstructor::new("Just".to_string(), Kind::arrow(Kind::Type, Kind::Type), 1),
        ],
    );
    
    assert_eq!(maybe_int.name, "Maybe");
    assert_eq!(maybe_int.type_params.len(), 1);
    assert_eq!(maybe_int.constructors.len(), 2);
}

#[test]
fn test_type_inference_basic() {
    let mut inferrer = TypeInferencer::new();
    
    // Test inference of basic literal types
    let num_literal = Literal::number(42.0);
    let inferred_type = inferrer.infer_literal(&num_literal);
    
    match inferred_type {
        Ok(Type::Number) => (),
        other => panic!("Expected Number type, got {:?}", other),
    }
    
    // Test string literal
    let str_literal = Literal::string("hello".to_string());
    let inferred_type = inferrer.infer_literal(&str_literal);
    
    match inferred_type {
        Ok(Type::String) => (),
        other => panic!("Expected String type, got {:?}", other),
    }
}

#[test] 
fn test_function_type_inference() {
    let mut inferrer = TypeInferencer::new();
    
    // Create a function type: (Number -> Number)
    let func_type = Type::function(
        vec![Type::Number],
        Type::Number,
    );
    
    assert!(func_type.is_function());
    
    // Test parameter extraction
    if let Type::Function { params, return_type } = func_type {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], Type::Number);
        assert_eq!(*return_type, Type::Number);
    }
}

#[test]
fn test_polymorphic_types() {
    let mut type_checker = TypeChecker::new(TypeLevel::Static);
    
    // Test identity function: ∀a. a -> a
    let type_var_a = TypeVar::with_name("a");
    let identity_type = Type::forall(
        vec![type_var_a.clone()],
        Type::function(
            vec![Type::Variable(type_var_a.clone())],
            Type::Variable(type_var_a),
        ),
    );
    
    assert!(identity_type.is_polymorphic());
    
    // Test type scheme instantiation
    let scheme = TypeScheme::polymorphic(
        vec![TypeVar::with_name("a")],
        vec![],
        Type::function(
            vec![Type::Variable(TypeVar::with_name("a"))],
            Type::Variable(TypeVar::with_name("a")),
        ),
    );
    
    let instantiated = scheme.instantiate();
    // The instantiated type should have fresh type variables
    assert!(instantiated.is_function());
}

#[test]
fn test_type_constraints() {
    // Test equality constraints
    let constraint = TypeConstraint::equal(
        Type::Number,
        Type::Variable(TypeVar::with_name("a")),
        None,
        "test constraint",
    );
    
    match constraint {
        TypeConstraint::Equal { left, right, reason, .. } => {
            assert_eq!(left, Type::Number);
            assert!(matches!(right, Type::Variable(_)));
            assert_eq!(reason, "test constraint");
        }
        _ => panic!("Expected Equal constraint"),
    }
    
    // Test instance constraints
    let instance_constraint = TypeConstraint::instance(
        "Show",
        Type::Number,
        None,
    );
    
    match instance_constraint {
        TypeConstraint::Instance { class, type_, .. } => {
            assert_eq!(class, "Show");
            assert_eq!(type_, Type::Number);
        }
        _ => panic!("Expected Instance constraint"),
    }
}

#[test]
fn test_gradual_typing_integration() {
    let mut type_checker = TypeChecker::new(TypeLevel::Dynamic);
    
    // Test dynamic type
    let dynamic_type = Type::Dynamic;
    assert!(!dynamic_type.is_variable());
    assert!(!dynamic_type.is_function());
    
    // Test gradual type bridge
    let bridge = GradualTypeBridge::new();
    
    // Test conversion from R7RS value to gradual type
    let value = Value::Literal(Literal::number(42.0));
    let inferred_type = bridge.infer_from_value(&value);
    
    match inferred_type {
        Ok(Type::Number) => (),
        other => panic!("Expected Number type, got {:?}", other),
    }
}

#[test]
fn test_effect_type_annotations() {
    // Test effectful types
    let io_effect = Effect::IO;
    let state_effect = Effect::State(Type::Number);
    
    let effectful_type = Type::Effectful {
        input: Box::new(Type::String),
        effects: vec![io_effect, state_effect],
        output: Box::new(Type::Unit),
    };
    
    match effectful_type {
        Type::Effectful { input, effects, output } => {
            assert_eq!(*input, Type::String);
            assert_eq!(effects.len(), 2);
            assert_eq!(*output, Type::Unit);
        }
        _ => panic!("Expected Effectful type"),
    }
}

#[test]
fn test_row_polymorphism() {
    // Test record types with row polymorphism
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    fields.insert("age".to_string(), Type::Number);
    
    let closed_row = Row::closed(fields.clone());
    assert!(closed_row.is_closed());
    
    let open_row = Row::open(fields, TypeVar::with_name("r"));
    assert!(!open_row.is_closed());
    
    let record_type = Type::Record(closed_row);
    
    match record_type {
        Type::Record(row) => {
            assert_eq!(row.fields.len(), 2);
            assert!(row.fields.contains_key("name"));
            assert!(row.fields.contains_key("age"));
        }
        _ => panic!("Expected Record type"),
    }
}

#[test]
fn test_type_unification() {
    let mut unifier = Unifier::new();
    
    // Test unifying a type variable with a concrete type
    let var = Type::Variable(TypeVar::with_name("a"));
    let concrete = Type::Number;
    
    let result = unifier.unify(var.clone(), concrete.clone());
    assert!(result.is_ok());
    
    // Test unifying incompatible types
    let result = unifier.unify(Type::Number, Type::String);
    assert!(result.is_err());
    
    // Test occurs check
    let var_a = TypeVar::with_name("a");
    let recursive = Type::list(Type::Variable(var_a.clone()));
    let result = unifier.unify(Type::Variable(var_a), recursive);
    assert!(result.is_err()); // Should fail occurs check
}

#[test]
fn test_type_classes() {
    let mut type_env = TypeEnv::new();
    
    // Create a Show type class
    let show_class = TypeClass::new(
        "Show".to_string(),
        vec![TypeVar::with_name("a")],
        vec!["show".to_string()],
        vec![],
    );
    
    assert_eq!(show_class.name, "Show");
    assert_eq!(show_class.methods.len(), 1);
    
    // Create an instance of Show for Numbers
    let show_number_instance = TypeClassInstance {
        class: "Show".to_string(),
        type_: Type::Number,
        methods: {
            let mut methods = HashMap::new();
            methods.insert(
                "show".to_string(),
                Type::function(vec![Type::Number], Type::String),
            );
            methods
        },
    };
    
    type_env.instances.insert(
        "Show".to_string(),
        vec![show_number_instance],
    );
    
    // Test instance resolution
    assert!(type_env.instances.contains_key("Show"));
}

#[test]
fn test_kind_system() {
    // Test basic kinds
    let type_kind = Kind::Type;
    let row_kind = Kind::Row;
    let effect_kind = Kind::Effect;
    
    assert_eq!(type_kind.arity(), 0);
    assert_eq!(row_kind.arity(), 0);
    assert_eq!(effect_kind.arity(), 0);
    
    // Test arrow kinds
    let maybe_kind = Kind::arrow(Kind::Type, Kind::Type);
    assert_eq!(maybe_kind.arity(), 1);
    
    let functor_kind = Kind::arrow(
        Kind::arrow(Kind::Type, Kind::Type),
        Kind::arrow(Kind::Type, Kind::Type),
    );
    assert_eq!(functor_kind.arity(), 2);
}

#[test]
fn test_dependent_types_basic() {
    // Test basic dependent type constructs
    let length_type = DependentType::new(
        "Vec".to_string(),
        vec![
            DependentParameter::Type(TypeVar::with_name("a")),
            DependentParameter::Term(DependentTerm::Variable("n".to_string())),
        ],
    );
    
    assert_eq!(length_type.constructor, "Vec");
    assert_eq!(length_type.parameters.len(), 2);
}

#[test]
fn test_type_error_reporting() {
    let mut type_checker = TypeChecker::new(TypeLevel::Static);
    
    // Create a type error
    let error = crate::diagnostics::Error::type_error(
        "Cannot unify Number with String".to_string(),
        Some(Span::new(0, 10, Some("test".to_string()))),
    );
    
    type_checker.add_error(error);
    assert_eq!(type_checker.errors().len(), 1);
    
    type_checker.clear_errors();
    assert_eq!(type_checker.errors().len(), 0);
}

#[test]
fn test_type_variable_generation() {
    // Test type variable generation
    let var1 = TypeVar::new();
    let var2 = TypeVar::new();
    
    // Should have different IDs
    assert_ne!(var1.id, var2.id);
    
    // Test named variables
    let named_var = TypeVar::with_name("test");
    assert_eq!(named_var.name, Some("test".to_string()));
}

#[test]
fn test_substitution() {
    let mut subst = Substitution::empty();
    let var = TypeVar::with_name("a");
    
    // Add a substitution
    subst.add(var.clone(), Type::Number);
    
    // Apply substitution
    let original = Type::Variable(var.clone());
    let result = subst.apply(&original);
    
    assert_eq!(result, Type::Number);
    
    // Test composition
    let mut subst2 = Substitution::empty();
    let var2 = TypeVar::with_name("b");
    subst2.add(var2.clone(), Type::Variable(var));
    
    let composed = subst.compose(&subst2);
    let applied = composed.apply(&Type::Variable(var2));
    assert_eq!(applied, Type::Number);
}

// Integration test that combines multiple type system features
#[test]
fn test_advanced_type_system_integration() {
    let mut type_checker = TypeChecker::new(TypeLevel::Static);
    let mut type_env = TypeEnv::new();
    
    // Set up a polymorphic function with constraints
    let var_a = TypeVar::with_name("a");
    let show_constraint = Constraint {
        class: "Show".to_string(),
        type_: Type::Variable(var_a.clone()),
    };
    
    let constrained_type = Type::constrained(
        vec![show_constraint],
        Type::function(
            vec![Type::Variable(var_a.clone())],
            Type::String,
        ),
    );
    
    let scheme = TypeScheme::polymorphic(
        vec![var_a],
        vec![Constraint {
            class: "Show".to_string(),
            type_: Type::Variable(TypeVar::with_name("a")),
        }],
        constrained_type,
    );
    
    type_env.bind("to_string".to_string(), scheme);
    
    // Verify the binding exists
    assert!(type_env.lookup("to_string").is_some());
    
    // Test instantiation
    if let Some(scheme) = type_env.lookup("to_string") {
        let instantiated = scheme.instantiate();
        assert!(instantiated.is_function());
    }
}