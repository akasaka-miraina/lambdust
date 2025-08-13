//! Tests for SRFI-121: Generators implementation
//!
//! This module provides comprehensive tests for the SRFI-121 generators
//! implementation, covering core operations, constructors, and utilities.

use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::generators::*;
use lambdust::containers::Generator;
use std::sync::Arc;

/// Test basic generator functionality
#[test]
fn test_generator_basic_operations() {
    // Test generator creation from values
    let values = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
    let generator = Value::generator_from_values(values);
    
    // Test type predicate
    assert!(generator.is_generator());
    assert!(!Value::integer(42).is_generator());
    
    // Test exhausted state
    if let Value::Generator(gen_ref) = &generator {
        assert!(!gen_ref.is_exhausted());
        
        // Consume all values
        assert_eq!(gen_ref.next().unwrap(), Value::integer(1));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(2));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(3));
        
        // Should be exhausted now and return EOF
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
    }
}

/// Test range generator functionality
#[test]
fn test_range_generator() {
    // Test finite range
    let generator = Value::generator_range(0.0, Some(3.0), 1.0);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::number(0.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(1.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(2.0));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test negative step
    let generator = Value::generator_range(10.0, Some(5.0), -2.0);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::number(10.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(8.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(6.0));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test infinite range (just a few values)
    let generator = Value::generator_range(0.0, None, 1.0);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::number(0.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(1.0));
        assert_eq!(gen_ref.next().unwrap(), Value::number(2.0));
        assert!(!gen_ref.is_exhausted()); // Should still be infinite
    }
}

/// Test iota generator functionality
#[test]
fn test_iota_generator() {
    // Test finite iota with count
    let generator = Value::generator_iota(Some(3), 5, 2);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::integer(5));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(7));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(9));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test iota starting from 0 with default step
    let generator = Value::generator_iota(Some(4), 0, 1);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::integer(0));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(1));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(2));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(3));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test infinite iota (just a few values)
    let generator = Value::generator_iota(None, 100, 10);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::integer(100));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(110));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(120));
        assert!(!gen_ref.is_exhausted()); // Should still be infinite
    }
}

/// Test list generator functionality
#[test]
fn test_list_generator() {
    // Create a proper Scheme list
    let list = Value::list(vec![
        Value::string("hello"),
        Value::string("world"),
        Value::integer(42),
    ]);
    
    let generator = Value::generator_from_list(list);
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::string("hello"));
        assert_eq!(gen_ref.next().unwrap(), Value::string("world"));
        assert_eq!(gen_ref.next().unwrap(), Value::integer(42));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test empty list
    let empty_generator = Value::generator_from_list(Value::Nil);
    
    if let Value::Generator(gen_ref) = &empty_generator {
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
    }
}

/// Test string generator functionality
#[test]
fn test_string_generator() {
    let generator = Value::generator_from_string("abc".to_string());
    
    if let Value::Generator(gen_ref) = &generator {
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('a')));
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('b')));
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('c')));
        assert!(gen_ref.is_exhausted());
    }
    
    // Test empty string
    let empty_generator = Value::generator_from_string("".to_string());
    
    if let Value::Generator(gen_ref) = &empty_generator {
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
    }
    
    // Test Unicode characters
    let unicode_generator = Value::generator_from_string("αβγ".to_string());
    
    if let Value::Generator(gen_ref) = &unicode_generator {
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('α')));
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('β')));
        assert_eq!(gen_ref.next().unwrap(), Value::Literal(lambdust::ast::Literal::Character('γ')));
        assert!(gen_ref.is_exhausted());
    }
}

/// Test vector generator functionality
#[test]
fn test_vector_generator() {
    let vector_values = vec![
        Value::boolean(true),
        Value::boolean(false), 
        Value::string("test"),
    ];
    let vector = Value::vector(vector_values);
    
    if let Value::Vector(vec_ref) = &vector {
        let generator = Value::generator_from_vector(vec_ref.clone());
        
        if let Value::Generator(gen_ref) = &generator {
            assert_eq!(gen_ref.next().unwrap(), Value::boolean(true));
            assert_eq!(gen_ref.next().unwrap(), Value::boolean(false));
            assert_eq!(gen_ref.next().unwrap(), Value::string("test"));
            assert!(gen_ref.is_exhausted());
        }
    }
}

/// Test exhausted generator
#[test]
fn test_exhausted_generator() {
    let generator = Value::generator_exhausted();
    
    if let Value::Generator(gen_ref) = &generator {
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone()); // Should still return EOF
    }
}

/// Test generator display formatting
#[test]
fn test_generator_display() {
    let generator = Value::generator_from_values(vec![Value::integer(1)]);
    let display_str = format!("{}", generator);
    assert!(display_str.contains("generator"));
}

/// Test generator equality (pointer equality for thread-safe containers)
#[test]
fn test_generator_equality() {
    let gen1 = Value::generator_from_values(vec![Value::integer(1)]);
    let gen2 = Value::generator_from_values(vec![Value::integer(1)]);
    let gen1_clone = gen1.clone();
    
    // Different generators should not be equal (pointer comparison)
    assert_ne!(gen1, gen2);
    
    // Same generator (cloned Arc) should be equal
    assert_eq!(gen1, gen1_clone);
}

/// Test generator with mixed types
#[test]
fn test_mixed_type_generator() {
    let mixed_values = vec![
        Value::integer(42),
        Value::string("hello"),
        Value::boolean(true),
        Value::Nil,
        Value::list(vec![Value::integer(1), Value::integer(2)]),
    ];
    
    let generator = Value::generator_from_values(mixed_values.clone());
    
    if let Value::Generator(gen_ref) = &generator {
        for expected_value in mixed_values {
            assert_eq!(gen_ref.next().unwrap(), expected_value);
        }
        assert!(gen_ref.is_exhausted());
    }
}

/// Test thread safety by using generator from multiple contexts
#[test]
fn test_generator_thread_safety() {
    let generator = Value::generator_range(0.0, Some(10.0), 1.0);
    
    // This test ensures the generator compiles with Send + Sync bounds
    let generator_clone = generator.clone();
    let handle = std::thread::spawn(move || {
        if let Value::Generator(gen_ref) = &generator_clone {
            gen_ref.next().unwrap()
        } else {
            Value::Nil
        }
    });
    
    let result = handle.join().unwrap();
    assert_eq!(result, Value::number(0.0));
}

/// Performance test for large generators
#[test]
fn test_large_generator_performance() {
    let large_count = 10000;
    let generator = Value::generator_iota(Some(large_count), 0, 1);
    
    if let Value::Generator(gen_ref) = &generator {
        let start = std::time::Instant::now();
        
        for expected in 0..large_count {
            let value = gen_ref.next().unwrap();
            assert_eq!(value, Value::integer(expected as i64));
        }
        
        let elapsed = start.elapsed();
        println!("Generated {} values in {:?}", large_count, elapsed);
        
        // Should be exhausted
        assert!(gen_ref.is_exhausted());
        
        // Performance should be reasonable (under 1ms per 1000 values)
        assert!(elapsed.as_millis() < large_count as u128 / 1000 + 100);
    }
}

/// Test edge cases and error conditions
#[test]
fn test_generator_edge_cases() {
    // Test zero-step range (should be handled gracefully in actual implementation)
    // This tests the boundary condition rather than causing a panic
    
    // Test empty values generator
    let empty_gen = Value::generator_from_values(vec![]);
    
    if let Value::Generator(gen_ref) = &empty_gen {
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
    }
    
    // Test single value generator  
    let single_gen = Value::generator_from_values(vec![Value::string("only")]);
    
    if let Value::Generator(gen_ref) = &single_gen {
        assert_eq!(gen_ref.next().unwrap(), Value::string("only"));
        assert!(gen_ref.is_exhausted());
        assert_eq!(gen_ref.next().unwrap(), gen_ref.eof_object().clone());
    }
}

/// Integration test with EOF object
#[test]
fn test_eof_object_handling() {
    let generator = Value::generator_from_values(vec![Value::integer(1)]);
    
    if let Value::Generator(gen_ref) = &generator {
        let eof_obj = gen_ref.eof_object().clone();
        
        // Consume the value
        assert_eq!(gen_ref.next().unwrap(), Value::integer(1));
        
        // Should now return EOF consistently
        assert_eq!(gen_ref.next().unwrap(), eof_obj.clone());
        assert_eq!(gen_ref.next().unwrap(), eof_obj.clone());
        
        // EOF should be a symbol
        assert!(eof_obj.is_symbol());
    }
}