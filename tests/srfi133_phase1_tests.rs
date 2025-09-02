//! Comprehensive test suite for SRFI-133 Vector Library Phase 1
//!
//! This test suite validates all 8 critical procedures implemented in Phase 1,
//! including performance optimizations and error handling.

use lambdust::diagnostics::Result;
use lambdust::effects::Effect;
use lambdust::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi133_vectors::create_srfi133_phase1_bindings;
use std::sync::Arc;

/// Creates a test environment with SRFI-133 Phase 1 bindings
fn create_test_env() -> Arc<ThreadSafeEnvironment> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_srfi133_phase1_bindings(&env);

    // Add some utility procedures for testing
    add_test_procedures(&env);

    env
}

/// Adds utility procedures for testing
fn add_test_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // equal? procedure for vector= testing
    env.define(
        "equal?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "equal?".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(|args| Ok(Value::boolean(args[0] == args[1]))),
            effects: &[Effect::Pure],
        })),
    );

    // + procedure for fold testing
    env.define(
        "+".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "+".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let sum = args
                    .iter()
                    .filter_map(|v| v.as_number())
                    .fold(0.0, |acc, n| acc + n);
                Ok(Value::number(sum))
            }),
            effects: &[Effect::Pure],
        })),
    );

    // * procedure for fold testing
    env.define(
        "*".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "*".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let product = args
                    .iter()
                    .filter_map(|v| v.as_number())
                    .fold(1.0, |acc, n| acc * n);
                Ok(Value::number(product))
            }),
            effects: &[Effect::Pure],
        })),
    );

    // even? predicate for search testing
    env.define(
        "even?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "even?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_number() {
                    Ok(Value::boolean(n as i64 % 2 == 0))
                } else {
                    Ok(Value::boolean(false))
                }
            }),
            effects: &[Effect::Pure],
        })),
    );

    // positive? predicate for search testing
    env.define(
        "positive?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "positive?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_number() {
                    Ok(Value::boolean(n > 0.0))
                } else {
                    Ok(Value::boolean(false))
                }
            }),
            effects: &[Effect::Pure],
        })),
    );

    // double procedure for map! testing
    env.define(
        "double".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "double".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_number() {
                    Ok(Value::number(n * 2.0))
                } else {
                    Ok(args[0].clone())
                }
            }),
            effects: &[Effect::Pure],
        })),
    );
}

/// Helper function to call a procedure from the environment
fn call_procedure(env: &Arc<ThreadSafeEnvironment>, name: &str, args: &[Value]) -> Result<Value> {
    let proc = env.get(name).ok_or_else(|| {
        Box::new(lambdust::diagnostics::Error::runtime_error(
            format!("Procedure {} not found", name),
            None,
        ))
    })?;

    match &proc {
        Value::Primitive(prim) => match &prim.implementation {
            PrimitiveImpl::RustFn(func) => func(args),
            PrimitiveImpl::Native(func) => func(args),
            _ => Err(Box::new(lambdust::diagnostics::Error::runtime_error(
                "Unsupported primitive type in test".to_string(),
                None,
            ))),
        },
        _ => Err(Box::new(lambdust::diagnostics::Error::runtime_error(
            "Expected primitive procedure".to_string(),
            None,
        ))),
    }
}

// ============= ENHANCED PREDICATES TESTS =============

#[test]
fn test_vector_empty_comprehensive() {
    let env = create_test_env();

    // Test empty vector
    let empty_vec = Value::vector(Vec::new());
    let result = call_procedure(&env, "vector-empty?", &[empty_vec]).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test non-empty vectors of various sizes
    let small_vec = Value::vector(&[Value::integer(1)]);
    let result = call_procedure(&env, "vector-empty?", &[small_vec]).unwrap();
    assert_eq!(result, Value::boolean(false));

    let large_vec = Value::vector((0..1000).map(|i| Value::integer(i)).collect());
    let result = call_procedure(&env, "vector-empty?", &[large_vec]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_empty_errors() {
    let env = create_test_env();

    // Test wrong number of arguments
    let result = call_procedure(&env, "vector-empty?", &[]);
    assert!(result.is_err());

    let result = call_procedure(
        &env,
        "vector-empty?",
        &[Value::integer(1), Value::integer(2)],
    );
    assert!(result.is_err());

    // Test non-vector argument
    let result = call_procedure(&env, "vector-empty?", &[Value::integer(42)]);
    assert!(result.is_err());
}

#[test]
fn test_vector_equal_comprehensive() {
    let env = create_test_env();

    // Test equal vectors
    let vec1 = Value::vector(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    let vec2 = Value::vector(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    let equal_proc = env.lookup("equal?").unwrap();

    let result =
        call_procedure(&env, "vector=", &[equal_proc.clone(), vec1.clone(), vec2]).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test unequal vectors (different elements)
    let vec3 = Value::vector(&[Value::integer(1), Value::integer(2), Value::integer(4)]);
    let result =
        call_procedure(&env, "vector=", &[equal_proc.clone(), vec1.clone(), vec3]).unwrap();
    assert_eq!(result, Value::boolean(false));

    // Test unequal vectors (different lengths)
    let vec4 = Value::vector(&[Value::integer(1), Value::integer(2)]);
    let result =
        call_procedure(&env, "vector=", &[equal_proc.clone(), vec1.clone(), vec4]).unwrap();
    assert_eq!(result, Value::boolean(false));

    // Test multiple equal vectors
    let vec5 = Value::vector(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    let result = call_procedure(
        &env,
        "vector=",
        &[equal_proc.clone(), vec1.clone(), vec1.clone(), vec5],
    )
    .unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test single vector (should be true)
    let result = call_procedure(&env, "vector=", &[equal_proc, vec1]).unwrap();
    assert_eq!(result, Value::boolean(true));
}

#[test]
fn test_vector_equal_empty_vectors() {
    let env = create_test_env();
    let equal_proc = env.lookup("equal?").unwrap();

    // Test empty vectors are equal
    let empty1 = Value::vector(Vec::new());
    let empty2 = Value::vector(Vec::new());
    let result = call_procedure(&env, "vector=", &[equal_proc, empty1, empty2]).unwrap();
    assert_eq!(result, Value::boolean(true));
}

// ============= FOLD OPERATIONS TESTS =============

#[test]
fn test_vector_fold_basic() {
    let env = create_test_env();
    let add_proc = env.lookup("+").unwrap();

    // Test basic left fold (sum)
    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
        Value::number(4.0),
    ]);

    let result =
        call_procedure(&env, "vector-fold", &[add_proc, Value::number(0.0), vector]).unwrap();

    assert_eq!(result, Value::number(10.0));
}

#[test]
fn test_vector_fold_multiple_vectors() {
    let env = create_test_env();
    let add_proc = env.lookup("+").unwrap();

    // Test fold over multiple vectors
    let vec1 = Value::vector(&[Value::number(1.0), Value::number(2.0)]);
    let vec2 = Value::vector(&[Value::number(3.0), Value::number(4.0)]);

    let result = call_procedure(
        &env,
        "vector-fold",
        &[add_proc, Value::number(0.0), vec1, vec2],
    )
    .unwrap();

    // Should be 0 + 1 + 3 + 2 + 4 = 10
    assert_eq!(result, Value::number(10.0));
}

#[test]
fn test_vector_fold_empty() {
    let env = create_test_env();
    let add_proc = env.lookup("+").unwrap();

    // Test fold over empty vector
    let empty_vec = Value::vector(Vec::new());
    let result = call_procedure(
        &env,
        "vector-fold",
        &[add_proc, Value::number(42.0), empty_vec],
    )
    .unwrap();

    // Should return initial value
    assert_eq!(result, Value::number(42.0));
}

#[test]
fn test_vector_fold_right_basic() {
    let env = create_test_env();

    // Use a non-associative operation to test right-fold behavior
    let subtract_proc = Arc::new(PrimitiveProcedure {
        name: "-".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(|args| {
            if args.is_empty() {
                return Ok(Value::number(0.0));
            }

            let first = args[0].as_number().unwrap_or(0.0);
            if args.len() == 1 {
                return Ok(Value::number(-first));
            }

            let rest_sum = args[1..]
                .iter()
                .filter_map(|v| v.as_number())
                .fold(0.0, |acc, n| acc + n);
            Ok(Value::number(first - rest_sum))
        }),
        effects: &[Effect::Pure],
    });

    // Add to environment for testing
    let test_env = create_test_env();
    test_env.define("-".to_string(), Value::Primitive(subtract_proc));

    let minus_proc = test_env.lookup("-").unwrap();
    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
    ]);

    let result = call_procedure(
        &test_env,
        "vector-fold-right",
        &[minus_proc, Value::number(0.0), vector],
    )
    .unwrap();

    // For right fold: 0 - 3 - 2 - 1 = -6
    assert_eq!(result, Value::number(-6.0));
}

// ============= SEARCH OPERATIONS TESTS =============

#[test]
fn test_vector_index_found() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0), // First even at index 1
        Value::number(3.0),
        Value::number(4.0),
    ]);

    let result = call_procedure(&env, "vector-index", &[even_proc, vector]).unwrap();
    assert_eq!(result, Value::integer(1));
}

#[test]
fn test_vector_index_not_found() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(3.0),
        Value::number(5.0),
    ]);

    let result = call_procedure(&env, "vector-index", &[even_proc, vector]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_index_empty() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();
    let empty_vec = Value::vector(Vec::new());

    let result = call_procedure(&env, "vector-index", &[even_proc, empty_vec]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_any_found() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
    ]);

    let result = call_procedure(&env, "vector-any", &[even_proc, vector]).unwrap();
    assert_eq!(result, Value::boolean(true));
}

#[test]
fn test_vector_any_not_found() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(3.0),
        Value::number(5.0),
    ]);

    let result = call_procedure(&env, "vector-any", &[even_proc, vector]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_any_empty() {
    let env = create_test_env();
    let even_proc = env.lookup("even?").unwrap();
    let empty_vec = Value::vector(Vec::new());

    let result = call_procedure(&env, "vector-any", &[even_proc, empty_vec]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_every_all_true() {
    let env = create_test_env();
    let positive_proc = env.lookup("positive?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
    ]);

    let result = call_procedure(&env, "vector-every", &[positive_proc, vector]).unwrap();
    assert_eq!(result, Value::boolean(true));
}

#[test]
fn test_vector_every_some_false() {
    let env = create_test_env();
    let positive_proc = env.lookup("positive?").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(-2.0), // Not positive
        Value::number(3.0),
    ]);

    let result = call_procedure(&env, "vector-every", &[positive_proc, vector]).unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_vector_every_empty() {
    let env = create_test_env();
    let positive_proc = env.lookup("positive?").unwrap();
    let empty_vec = Value::vector(Vec::new());

    let result = call_procedure(&env, "vector-every", &[positive_proc, empty_vec]).unwrap();
    assert_eq!(result, Value::boolean(true)); // Every over empty set is true
}

// ============= PERFORMANCE SHOWCASE TESTS =============

#[test]
fn test_vector_map_inplace_basic() {
    let env = create_test_env();
    let double_proc = env.lookup("double").unwrap();

    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
    ]);

    let result = call_procedure(&env, "vector-map!", &[double_proc, vector.clone()]).unwrap();
    assert_eq!(result, Value::Unspecified);

    // Verify the vector was modified in place
    if let Value::Vector(vec_ref) = &vector {
        let vec_data = vec_ref.borrow();
        assert_eq!(vec_data[0], Value::number(2.0));
        assert_eq!(vec_data[1], Value::number(4.0));
        assert_eq!(vec_data[2], Value::number(6.0));
    } else {
        panic!("Expected vector");
    }
}

#[test]
fn test_vector_map_inplace_multiple_vectors() {
    let env = create_test_env();
    let add_proc = env.lookup("+").unwrap();

    let target_vec = Value::vector(vec![Value::number(1.0), Value::number(2.0)]);
    let source_vec = Value::vector(vec![Value::number(10.0), Value::number(20.0)]);

    let result = call_procedure(
        &env,
        "vector-map!",
        &[add_proc, target_vec.clone(), source_vec],
    )
    .unwrap();
    assert_eq!(result, Value::Unspecified);

    // Verify the target vector was modified
    if let Value::Vector(vec_ref) = &target_vec {
        let vec_data = vec_ref.borrow();
        assert_eq!(vec_data[0], Value::number(11.0)); // 1 + 10
        assert_eq!(vec_data[1], Value::number(22.0)); // 2 + 20
    } else {
        panic!("Expected vector");
    }
}

#[test]
fn test_vector_map_inplace_empty() {
    let env = create_test_env();
    let double_proc = env.lookup("double").unwrap();
    let empty_vec = Value::vector(Vec::new());

    let result = call_procedure(&env, "vector-map!", &[double_proc, empty_vec]).unwrap();
    assert_eq!(result, Value::Unspecified);
}

// ============= STRESS TESTS =============

#[test]
fn test_large_vector_performance() {
    let env = create_test_env();

    // Create a large vector to test performance optimizations
    let large_vector = Value::vector((0..10000).map(|i| Value::number(i as f64)).collect());

    // Test vector-empty? on large vector (should be O(1))
    let result = call_procedure(&env, "vector-empty?", &[large_vector.clone()]).unwrap();
    assert_eq!(result, Value::boolean(false));

    // Test vector-any with early termination
    let even_proc = env.lookup("even?").unwrap();
    let result = call_procedure(&env, "vector-any", &[even_proc, large_vector.clone()]).unwrap();
    assert_eq!(result, Value::boolean(true)); // Should find 0 at index 0

    // Test vector-fold on large vector
    let add_proc = env.lookup("+").unwrap();
    let result = call_procedure(
        &env,
        "vector-fold",
        &[add_proc, Value::number(0.0), large_vector],
    )
    .unwrap();

    // Sum of 0 to 9999 = 9999 * 10000 / 2 = 49995000
    assert_eq!(result, Value::number(49995000.0));
}

#[test]
fn test_cache_friendly_processing() {
    let env = create_test_env();

    // Create vector with size that exercises cache block processing
    let medium_vector = Value::vector((0..100).map(|i| Value::number(i as f64)).collect());
    let double_proc = env.lookup("double").unwrap();

    // Test in-place mapping which should use cache-friendly block processing
    let result =
        call_procedure(&env, "vector-map!", &[double_proc, medium_vector.clone()]).unwrap();
    assert_eq!(result, Value::Unspecified);

    // Verify results
    if let Value::Vector(vec_ref) = &medium_vector {
        let vec_data = vec_ref.borrow();
        assert_eq!(vec_data[0], Value::number(0.0));
        assert_eq!(vec_data[1], Value::number(2.0));
        assert_eq!(vec_data[50], Value::number(100.0));
        assert_eq!(vec_data[99], Value::number(198.0));
    } else {
        panic!("Expected vector");
    }
}

// ============= ERROR HANDLING TESTS =============

#[test]
fn test_comprehensive_error_handling() {
    let env = create_test_env();

    // Test all procedures with wrong argument counts
    let procedures = [
        "vector-empty?",
        "vector=",
        "vector-fold",
        "vector-fold-right",
        "vector-index",
        "vector-any",
        "vector-every",
        "vector-map!",
    ];

    for proc_name in &procedures {
        // Test with no arguments (should fail for all)
        let result = call_procedure(&env, proc_name, &[]);
        assert!(
            result.is_err(),
            "Procedure {} should fail with no arguments",
            proc_name
        );
    }

    // Test with non-procedure arguments where procedures are expected
    let non_proc = Value::integer(42);
    let test_vector = Value::vector(&[Value::integer(1)]);

    for proc_name in &[
        "vector=",
        "vector-fold",
        "vector-fold-right",
        "vector-index",
        "vector-any",
        "vector-every",
        "vector-map!",
    ] {
        let result = call_procedure(&env, proc_name, &[non_proc.clone(), test_vector.clone()]);
        assert!(
            result.is_err(),
            "Procedure {} should fail with non-procedure argument",
            proc_name
        );
    }

    // Test with non-vector arguments where vectors are expected
    let non_vector = Value::integer(42);
    let test_proc = env.lookup("even?").unwrap();

    for proc_name in &[
        "vector-empty?",
        "vector-index",
        "vector-any",
        "vector-every",
        "vector-map!",
    ] {
        let args = if proc_name == &"vector-empty?" {
            &[non_vector.clone()]
        } else {
            &[test_proc.clone(), non_vector.clone()]
        };

        let result = call_procedure(&env, proc_name, &args);
        assert!(
            result.is_err(),
            "Procedure {} should fail with non-vector argument",
            proc_name
        );
    }
}

// ============= INTEGRATION TESTS =============

#[test]
fn test_procedure_composition() {
    let env = create_test_env();

    // Test chaining operations
    let vector = Value::vector(vec![
        Value::number(1.0),
        Value::number(2.0),
        Value::number(3.0),
        Value::number(4.0),
    ]);

    // First, check if any element is even
    let even_proc = env.lookup("even?").unwrap();
    let has_even =
        call_procedure(&env, "vector-any", &[even_proc.clone(), vector.clone()]).unwrap();
    assert_eq!(has_even, Value::boolean(true));

    // Find the index of first even element
    let even_index = call_procedure(&env, "vector-index", &[even_proc, vector.clone()]).unwrap();
    assert_eq!(even_index, Value::integer(1));

    // Check if vector is empty
    let is_empty = call_procedure(&env, "vector-empty?", &[vector.clone()]).unwrap();
    assert_eq!(is_empty, Value::boolean(false));

    // Sum all elements
    let add_proc = env.lookup("+").unwrap();
    let sum = call_procedure(&env, "vector-fold", &[add_proc, Value::number(0.0), vector]).unwrap();
    assert_eq!(sum, Value::number(10.0));
}

#[test]
fn test_mixed_data_types() {
    let env = create_test_env();

    // Test with vectors containing mixed data types
    let mixed_vector = Value::vector(vec![
        Value::number(1.0),
        Value::string("hello"),
        Value::boolean(true),
        Value::number(2.0),
    ]);

    // vector-empty? should work with any vector
    let result = call_procedure(&env, "vector-empty?", &[mixed_vector.clone()]).unwrap();
    assert_eq!(result, Value::boolean(false));

    // Test equality with mixed types
    let mixed_vector2 = Value::vector(vec![
        Value::number(1.0),
        Value::string("hello"),
        Value::boolean(true),
        Value::number(2.0),
    ]);

    let equal_proc = env.lookup("equal?").unwrap();
    let result = call_procedure(
        &env,
        "vector=",
        &[equal_proc, mixed_vector.clone(), mixed_vector2],
    )
    .unwrap();
    assert_eq!(result, Value::boolean(true));
}
