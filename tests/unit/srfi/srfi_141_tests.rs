//! SRFI 141: Integer Division Tests
//!
//! Comprehensive test suite for SRFI 141 integer division operations covering:
//! - Floor division family (floor-quotient, floor-remainder, floor/)
//! - Ceiling division family (ceiling-quotient, ceiling-remainder, ceiling/)
//! - Truncate division family (truncate-quotient, truncate-remainder, truncate/)
//! - Round division family (round-quotient, round-remainder, round/)
//! - Euclidean division family (euclidean-quotient, euclidean-remainder, euclidean/)
//! - Balanced division family (balanced-quotient, balanced-remainder, balanced/)

use lambdust::builtins::arithmetic::register_arithmetic_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

/// Test floor division family functions
#[test]
fn test_floor_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test floor-quotient
    let floor_quotient = builtins.get("floor-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_quotient {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-4)));
        
        // Test negative divisor
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(-4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-4)));
        
        // Test both negative
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(-4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
    }

    // Test floor-remainder
    let floor_remainder = builtins.get("floor-remainder").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_remainder {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
    }

    // Test floor/ (combined operation)
    let floor_div = builtins.get("floor/").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_div {
        
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        
        if let Value::Values(values) = result {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Number(SchemeNumber::Integer(3))); // quotient
            assert_eq!(values[1], Value::Number(SchemeNumber::Integer(1))); // remainder
        } else {
            panic!("Expected Values for floor/, got {:?}", result);
        }
    }
}

/// Test ceiling division family functions
#[test]
fn test_ceiling_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test ceiling-quotient
    let ceiling_quotient = builtins.get("ceiling-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = ceiling_quotient {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(4)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-3)));
    }

    // Test ceiling-remainder
    let ceiling_remainder = builtins.get("ceiling-remainder").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = ceiling_remainder {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-3)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-1)));
    }
}

/// Test truncate division family functions  
#[test]
fn test_truncate_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test truncate-quotient (equivalent to quotient)
    let truncate_quotient = builtins.get("truncate-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = truncate_quotient {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-3)));
    }

    // Test truncate-remainder (equivalent to remainder)
    let truncate_remainder = builtins.get("truncate-remainder").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = truncate_remainder {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
        
        // Test negative dividend
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-1)));
    }
}

/// Test round division family functions
#[test]
fn test_round_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test round-quotient
    let round_quotient = builtins.get("round-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = round_quotient {
        
        // Test case where rounding makes a difference
        let result = func(&[
            Value::Number(SchemeNumber::Integer(11)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3))); // 11/4 = 2.75, rounds to 3
        
        let result = func(&[
            Value::Number(SchemeNumber::Integer(10)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(2))); // 10/4 = 2.5, rounds to 2 (even)
    }
}

/// Test euclidean division family functions
#[test]
fn test_euclidean_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test euclidean-quotient
    let euclidean_quotient = builtins.get("euclidean-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = euclidean_quotient {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        
        // Test negative dividend (euclidean remainder is always non-negative)
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-4)));
    }

    // Test euclidean-remainder
    let euclidean_remainder = builtins.get("euclidean-remainder").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = euclidean_remainder {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
        
        // Test negative dividend (remainder should be non-negative)
        let result = func(&[
            Value::Number(SchemeNumber::Integer(-13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
    }
}

/// Test balanced division family functions
#[test]
fn test_balanced_division_family() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test balanced-quotient
    let balanced_quotient = builtins.get("balanced-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = balanced_quotient {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        
        // Test case where remainder would be >= divisor/2
        let result = func(&[
            Value::Number(SchemeNumber::Integer(15)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(4))); // Balanced rounding
    }

    // Test balanced-remainder
    let balanced_remainder = builtins.get("balanced-remainder").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = balanced_remainder {
        
        // Test positive numbers
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
        
        // Test case where remainder would be >= divisor/2
        let result = func(&[
            Value::Number(SchemeNumber::Integer(15)),
            Value::Number(SchemeNumber::Integer(4)),
        ]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(-1))); // Balanced remainder
    }
}

/// Test division by zero error handling
#[test]
fn test_division_by_zero_errors() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test floor-quotient division by zero
    let floor_quotient = builtins.get("floor-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_quotient {
        
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(0)),
        ]);
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, LambdustError::DivisionByZero { .. }));
        }
    }
}

/// Test arity error handling
#[test]
fn test_arity_errors() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test floor-quotient with wrong number of arguments
    let floor_quotient = builtins.get("floor-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_quotient {
        
        // Too few arguments
        let result = func(&[Value::Number(SchemeNumber::Integer(13))]);
        assert!(result.is_err());
        
        // Too many arguments
        let result = func(&[
            Value::Number(SchemeNumber::Integer(13)),
            Value::Number(SchemeNumber::Integer(4)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        assert!(result.is_err());
    }
}

/// Test type error handling
#[test]
fn test_type_errors() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test floor-quotient with non-integer arguments
    let floor_quotient = builtins.get("floor-quotient").unwrap().clone();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = floor_quotient {
        
        let result = func(&[
            Value::String("not-a-number".to_string()),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        assert!(result.is_err());
    }
}

/// Test mathematical correctness of division algorithms
#[test]
fn test_mathematical_correctness() {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);

    // Test the fundamental property: x = q*y + r
    let test_cases = vec![
        (13, 4),
        (-13, 4),
        (13, -4),
        (-13, -4),
        (17, 5),
        (-17, 5),
        (17, -5),
        (-17, -5),
    ];

    for (x, y) in test_cases {
        // Test floor division
        let floor_quotient = builtins.get("floor-quotient").unwrap().clone();
        let floor_remainder = builtins.get("floor-remainder").unwrap().clone();
        
        if let (Value::Procedure(Procedure::Builtin { func: q_func, .. }), Value::Procedure(Procedure::Builtin { func: r_func, .. })) = (floor_quotient, floor_remainder) {
            
            let q_result = q_func(&[
                Value::Number(SchemeNumber::Integer(x)),
                Value::Number(SchemeNumber::Integer(y)),
            ]).unwrap();
            
            let r_result = r_func(&[
                Value::Number(SchemeNumber::Integer(x)),
                Value::Number(SchemeNumber::Integer(y)),
            ]).unwrap();
            
            if let (Value::Number(SchemeNumber::Integer(q)), Value::Number(SchemeNumber::Integer(r))) = (q_result, r_result) {
                // Verify x = q*y + r
                assert_eq!(x, q * y + r, "Division property failed for x={}, y={}, q={}, r={}", x, y, q, r);
                
                // For floor division, remainder should have same sign as divisor (or be zero)
                if r != 0 {
                    assert_eq!(r.signum(), y.signum(), "Floor remainder sign incorrect for x={}, y={}, r={}", x, y, r);
                }
                assert!(r.abs() < y.abs(), "Floor remainder too large for x={}, y={}, r={}", x, y, r);
            }
        }
    }
}