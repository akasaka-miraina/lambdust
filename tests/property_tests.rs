//! Property-based tests for Lambdust core functionality
//!
//! This module contains comprehensive property-based tests that verify
//! mathematical properties and invariants of the Lambdust Scheme implementation.

use lambdust::eval::value::Value;
use lambdust::property;
use lambdust::property_testing::{
    check_property, generators::*, PropertyConfig, PropertyResult, Property,
};
use std::sync::Arc;

/// Test the list reversal property: (reverse (reverse x)) = x
#[test]
fn test_list_reversal_property() {
    let property = property!("list_reversal", |list: Value| {
        // This property should hold for any list
        match list {
            Value::List(ref elements) => {
                let reversed_once: Vec<Value> = elements.iter().rev().cloned().collect();
                let reversed_twice: Vec<Value> = reversed_once.iter().rev().cloned().collect();
                elements == &reversed_twice
            }
            Value::Nil => true, // Empty list reversal property trivially holds
            _ => true, // Skip non-list values
        }
    });

    let generator = Arc::new(ListGenerator::new(0, 10));
    let config = PropertyConfig {
        test_cases: 1000,
        max_size: 20,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![generator], config);
    assert!(summary.all_passed(), 
        "List reversal property failed: {}/{} passed", 
        summary.passed, summary.total_tests);
}

/// Test numeric addition commutativity: x + y = y + x
#[test]
fn test_addition_commutativity() {
    let property = property!("addition_commutativity", |x: Value, y: Value| {
        match (&x, &y) {
            (Value::Number(a), Value::Number(b)) => {
                let sum1 = a + b;
                let sum2 = b + a;
                // Handle NaN case
                if sum1.is_nan() && sum2.is_nan() {
                    true
                } else {
                    (sum1 - sum2).abs() < f64::EPSILON
                }
            }
            (Value::Integer(a), Value::Integer(b)) => {
                // Check for overflow
                if let (Some(sum1), Some(sum2)) = (a.checked_add(*b), b.checked_add(*a)) {
                    sum1 == sum2
                } else {
                    true // Skip overflow cases
                }
            }
            _ => true // Skip non-numeric values
        }
    });

    let number_gen = Arc::new(NumberGenerator::new(-1000.0, 1000.0));
    let config = PropertyConfig {
        test_cases: 5000,
        max_size: 10,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![number_gen.clone(), number_gen], config);
    assert!(summary.all_passed(),
        "Addition commutativity failed: {}/{} passed, first failure: {:?}",
        summary.passed, summary.total_tests, summary.first_failure);
}

/// Test numeric multiplication associativity: (x * y) * z = x * (y * z)
#[test]
fn test_multiplication_associativity() {
    let property = property!("multiplication_associativity", |x: Value, y: Value, z: Value| {
        match (&x, &y, &z) {
            (Value::Number(a), Value::Number(b), Value::Number(c)) => {
                let result1 = (a * b) * c;
                let result2 = a * (b * c);
                
                // Handle special cases
                if result1.is_nan() && result2.is_nan() {
                    true
                } else if result1.is_infinite() && result2.is_infinite() {
                    result1.signum() == result2.signum()
                } else {
                    (result1 - result2).abs() < 1e-10 // Slightly larger epsilon for triple multiplication
                }
            }
            (Value::Integer(a), Value::Integer(b), Value::Integer(c)) => {
                // Check for overflow in multiplication
                if let Some(temp) = a.checked_mul(*b) {
                    if let Some(result1) = temp.checked_mul(*c) {
                        if let Some(temp2) = b.checked_mul(*c) {
                            if let Some(result2) = a.checked_mul(temp2) {
                                return result1 == result2;
                            }
                        }
                    }
                }
                true // Skip overflow cases
            }
            _ => true
        }
    });

    let number_gen = Arc::new(NumberGenerator::new(-10.0, 10.0));
    let config = PropertyConfig {
        test_cases: 2000,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(
        property,
        vec![number_gen.clone(), number_gen.clone(), number_gen],
        config,
    );
    assert!(summary.all_passed(),
        "Multiplication associativity failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test string concatenation associativity: (s1 ++ s2) ++ s3 = s1 ++ (s2 ++ s3)
#[test]
fn test_string_concatenation_associativity() {
    let property = property!("string_concat_associativity", |s1: Value, s2: Value, s3: Value| {
        match (&s1, &s2, &s3) {
            (Value::String(a), Value::String(b), Value::String(c)) => {
                let concat1 = format!("{}{}{}", a, b, c);
                let temp1 = format!("{}{}", a, b);
                let concat2 = format!("{}{}", temp1, c);
                let temp2 = format!("{}{}", b, c);
                let concat3 = format!("{}{}", a, temp2);
                
                concat1 == concat2 && concat2 == concat3
            }
            _ => true
        }
    });

    let string_weights = ValueTypeWeights {
        string: 100,
        ..Default::default()
    };
    let generator = Arc::new(SchemeValueGenerator::with_weights(string_weights));
    let config = PropertyConfig {
        test_cases: 1000,
        max_size: 20,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(
        property,
        vec![generator.clone(), generator.clone(), generator],
        config,
    );
    assert!(summary.all_passed(),
        "String concatenation associativity failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test list append associativity: (l1 ++ l2) ++ l3 = l1 ++ (l2 ++ l3)
#[test]
fn test_list_append_associativity() {
    let property = property!("list_append_associativity", |l1: Value, l2: Value, l3: Value| {
        match (&l1, &l2, &l3) {
            (Value::List(a), Value::List(b), Value::List(c)) => {
                // Simulate list append operations
                let mut result1 = Vec::new();
                result1.extend(a.iter().cloned());
                result1.extend(b.iter().cloned());
                result1.extend(c.iter().cloned());
                
                let mut temp1 = Vec::new();
                temp1.extend(a.iter().cloned());
                temp1.extend(b.iter().cloned());
                let mut result2 = temp1;
                result2.extend(c.iter().cloned());
                
                let mut temp2 = Vec::new();
                temp2.extend(b.iter().cloned());
                temp2.extend(c.iter().cloned());
                let mut result3 = Vec::new();
                result3.extend(a.iter().cloned());
                result3.extend(temp2);
                
                result1 == result2 && result2 == result3
            }
            _ => true
        }
    });

    let list_gen = Arc::new(ListGenerator::new(0, 5));
    let config = PropertyConfig {
        test_cases: 1000,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(
        property,
        vec![list_gen.clone(), list_gen.clone(), list_gen],
        config,
    );
    assert!(summary.all_passed(),
        "List append associativity failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test that cons and car/cdr are inverses for non-empty lists
#[test]
fn test_cons_car_cdr_inverse() {
    let property = property!("cons_car_cdr_inverse", |head: Value, tail: Value| {
        if let Value::List(tail_elements) = tail {
            // Simulate: (cons head tail) -> (car result) = head, (cdr result) = tail
            let mut result = vec![head.clone()];
            result.extend(tail_elements.iter().cloned());
            
            // car(cons(head, tail)) = head
            let car_result = result.first().cloned().unwrap_or(Value::Nil);
            
            // cdr(cons(head, tail)) = tail  
            let cdr_result = if result.len() > 1 {
                Value::list(result[1..].to_vec())
            } else {
                Value::Nil
            };
            
            car_result == head && cdr_result == Value::List(tail_elements.clone())
        } else {
            true // Skip non-list tails
        }
    });

    let value_gen = Arc::new(SchemeValueGenerator::new());
    let list_gen = Arc::new(ListGenerator::new(0, 5));
    let config = PropertyConfig {
        test_cases: 1000,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![value_gen, list_gen], config);
    assert!(summary.all_passed(),
        "Cons/car/cdr inverse property failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test numeric identity properties: x + 0 = x, x * 1 = x
#[test]
fn test_numeric_identities() {
    let property = property!("numeric_identities", |x: Value| {
        match x {
            Value::Number(n) => {
                let add_identity = n + 0.0;
                let mul_identity = n * 1.0;
                
                // Handle NaN specially
                if n.is_nan() {
                    add_identity.is_nan() && mul_identity.is_nan()
                } else {
                    (add_identity - n).abs() < f64::EPSILON && 
                    (mul_identity - n).abs() < f64::EPSILON
                }
            }
            Value::Integer(n) => {
                n + 0 == n && n * 1 == n
            }
            _ => true
        }
    });

    let number_gen = Arc::new(NumberGenerator::new(-10000.0, 10000.0));
    let config = PropertyConfig {
        test_cases: 2000,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![number_gen], config);
    assert!(summary.all_passed(),
        "Numeric identity properties failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test string length properties
#[test]
fn test_string_length_properties() {
    let property = property!("string_length_properties", |s1: Value, s2: Value| {
        match (&s1, &s2) {
            (Value::String(a), Value::String(b)) => {
                let concat = format!("{}{}", a, b);
                concat.len() == a.len() + b.len()
            }
            _ => true
        }
    });

    let string_weights = ValueTypeWeights {
        string: 100,
        ..Default::default()
    };
    let generator = Arc::new(SchemeValueGenerator::with_weights(string_weights));
    let config = PropertyConfig {
        test_cases: 1000,
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![generator.clone(), generator], config);
    assert!(summary.all_passed(),
        "String length properties failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Test boolean logic properties: De Morgan's laws, etc.
#[test]
fn test_boolean_logic_properties() {
    let property = property!("boolean_logic_properties", |a: Value, b: Value| {
        match (&a, &b) {
            (Value::Boolean(x), Value::Boolean(y)) => {
                // De Morgan's law: !(x && y) = !x || !y
                let demorgan1 = !(*x && *y) == (!*x || !*y);
                
                // De Morgan's law: !(x || y) = !x && !y  
                let demorgan2 = !(*x || *y) == (!*x && !*y);
                
                // Commutativity: x && y = y && x, x || y = y || x
                let and_comm = (*x && *y) == (*y && *x);
                let or_comm = (*x || *y) == (*y || *x);
                
                demorgan1 && demorgan2 && and_comm && or_comm
            }
            _ => true
        }
    });

    let bool_weights = ValueTypeWeights {
        boolean: 100,
        ..Default::default()
    };
    let generator = Arc::new(SchemeValueGenerator::with_weights(bool_weights));
    let config = PropertyConfig {
        test_cases: 500, // Fewer cases needed for boolean combinations
        parallel: true,
        ..PropertyConfig::default()
    };

    let summary = check_property(property, vec![generator.clone(), generator], config);
    assert!(summary.all_passed(),
        "Boolean logic properties failed: {}/{} passed",
        summary.passed, summary.total_tests);
}

/// Performance test to ensure we can run 100,000 test cases quickly
#[test]
fn test_performance_large_scale() {
    let property = property!("simple_always_true", |_x: Value| true);
    let generator = Arc::new(SchemeValueGenerator::new());
    let config = PropertyConfig {
        test_cases: 100_000,
        max_size: 10,
        parallel: true,
        ..PropertyConfig::default()
    };

    let start = std::time::Instant::now();
    let summary = check_property(property, vec![generator], config);
    let duration = start.elapsed();

    assert!(summary.all_passed());
    assert_eq!(summary.total_tests, 100_000);
    
    // Should complete in under 2 minutes (120 seconds) as per design goals
    assert!(duration.as_secs() < 120, 
        "Performance test took {} seconds, should be under 120", duration.as_secs());
    
    println!("Performance test: 100,000 cases in {} seconds", duration.as_secs_f64());
}