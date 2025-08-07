//! R7RS Vector Operations Tests (Section 6.8)
//!
//! Tests for R7RS-small section 6.8 (Vectors) including:
//! - Vector creation and manipulation
//! - Vector predicates and type checking
//! - Vector indexing and element access
//! - Vector conversion to/from lists
//! - Vector fill and copy operations
//! - Vector equality and comparison
//!
//! This module comprehensively tests vector operations as specified
//! in the R7RS-small standard.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all vector operations tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Vector Operations tests...");
    
    test_vector_predicates(suite)?;
    test_vector_creation(suite)?;
    test_vector_access(suite)?;
    test_vector_mutation(suite)?;
    test_vector_conversion(suite)?;
    test_vector_operations(suite)?;
    test_vector_equality(suite)?;
    test_vector_edge_cases(suite)?;
    
    println!("✓ Vector operations tests passed");
    Ok(())
}

/// Test vector predicates and type checking
fn test_vector_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector? predicate
    suite.assert_eval_true("(vector? #(1 2 3))")?;
    suite.assert_eval_true("(vector? #())")?; // Empty vector
    suite.assert_eval_true("(vector? (vector 'a 'b 'c))")?;
    suite.assert_eval_true("(vector? (make-vector 5))")?;
    
    // Test non-vectors
    suite.assert_eval_false("(vector? '(1 2 3))")?; // List is not vector
    suite.assert_eval_false("(vector? \"hello\")")?; // String is not vector
    suite.assert_eval_false("(vector? 42)")?;
    suite.assert_eval_false("(vector? #t)")?;
    suite.assert_eval_false("(vector? '())")?; // Empty list is not vector
    suite.assert_eval_false("(vector? (cons 1 2))")?; // Pair is not vector
    
    Ok(())
}

/// Test vector creation
fn test_vector_creation(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector literal syntax
    suite.assert_eval_eq("#()", Value::Vector(vec![]))?;
    suite.assert_eval_eq("#(1 2 3)", Value::Vector(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3))
    ]))?;
    
    // Test vector procedure
    suite.assert_eval_eq("(vector)", Value::Vector(vec![]))?;
    suite.assert_eval_eq("(vector 1 2 3)", Value::Vector(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3))
    ]))?;
    suite.assert_eval_eq("(vector 'a #t \"hello\")", Value::Vector(vec![
        Value::Symbol("a".to_string()),
        Value::Literal(Literal::Boolean(true)),
        Value::Literal(Literal::String("hello".to_string()))
    ]))?;
    
    // Test make-vector
    suite.assert_eval_eq("(make-vector 0)", Value::Vector(vec![]))?;
    suite.assert_eval_eq("(make-vector 3)", Value::Vector(vec![
        Value::Literal(Literal::Unspecified),
        Value::Literal(Literal::Unspecified),
        Value::Literal(Literal::Unspecified)
    ]))?;
    
    // Test make-vector with fill value
    suite.assert_eval_eq("(make-vector 3 'x)", Value::Vector(vec![
        Value::Symbol("x".to_string()),
        Value::Symbol("x".to_string()),
        Value::Symbol("x".to_string())
    ]))?;
    suite.assert_eval_eq("(make-vector 4 42)", Value::Vector(vec![
        Value::Literal(Literal::integer(42)),
        Value::Literal(Literal::integer(42)),
        Value::Literal(Literal::integer(42)),
        Value::Literal(Literal::integer(42))
    ]))?;
    
    // Test negative length error
    suite.assert_eval_error("(make-vector -1)")?;
    
    Ok(())
}

/// Test vector access operations
fn test_vector_access(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector-length
    suite.assert_eval_eq("(vector-length #())", Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(vector-length #(a b c))", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(vector-length (make-vector 10))", Value::Literal(Literal::integer(10)))?;
    
    // Test vector-ref
    suite.assert_eval_eq("(vector-ref #(a b c) 0)", Value::Symbol("a".to_string()))?;
    suite.assert_eval_eq("(vector-ref #(a b c) 1)", Value::Symbol("b".to_string()))?;
    suite.assert_eval_eq("(vector-ref #(a b c) 2)", Value::Symbol("c".to_string()))?;
    
    // Test vector-ref with numbers
    suite.assert_eval_eq("(vector-ref #(10 20 30) 1)", Value::Literal(Literal::integer(20)))?;
    
    // Test vector-ref bounds checking
    suite.assert_eval_error("(vector-ref #(a b c) 3)")?; // Index too large
    suite.assert_eval_error("(vector-ref #(a b c) -1)")?; // Negative index
    suite.assert_eval_error("(vector-ref #() 0)")?; // Empty vector
    
    // Test vector-ref with non-vector
    suite.assert_eval_error("(vector-ref '(a b c) 0)")?; // List is not vector
    suite.assert_eval_error("(vector-ref \"abc\" 0)")?; // String is not vector
    
    Ok(())
}

/// Test vector mutation operations
fn test_vector_mutation(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector-set!
    suite.eval("(define v1 (vector 'a 'b 'c))")?;
    suite.eval("(vector-set! v1 1 'x)")?;
    suite.assert_eval_eq("(vector-ref v1 1)", Value::Symbol("x".to_string()))?;
    
    // Verify other elements unchanged
    suite.assert_eval_eq("(vector-ref v1 0)", Value::Symbol("a".to_string()))?;
    suite.assert_eval_eq("(vector-ref v1 2)", Value::Symbol("c".to_string()))?;
    
    // Test vector-set! bounds checking
    suite.assert_eval_error("(vector-set! v1 3 'y)")?; // Index too large
    suite.assert_eval_error("(vector-set! v1 -1 'y)")?; // Negative index
    
    // Test vector-fill!
    suite.eval("(define v2 (vector 1 2 3 4 5))")?;
    suite.eval("(vector-fill! v2 'filled)")?;
    suite.assert_eval_eq("v2", Value::Vector(vec![
        Value::Symbol("filled".to_string()),
        Value::Symbol("filled".to_string()),
        Value::Symbol("filled".to_string()),
        Value::Symbol("filled".to_string()),
        Value::Symbol("filled".to_string())
    ]))?;
    
    // Test vector-fill! on empty vector
    suite.eval("(define v3 (vector))")?;
    suite.eval("(vector-fill! v3 'x)")?; // Should succeed but do nothing
    suite.assert_eval_eq("v3", Value::Vector(vec![]))?;
    
    Ok(())
}

/// Test vector conversion operations
fn test_vector_conversion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector->list
    suite.assert_eval_eq("(vector->list #())", Value::List(vec![]))?;
    suite.assert_eval_eq("(vector->list #(a b c))", Value::List(vec![
        Value::Symbol("a".to_string()),
        Value::Symbol("b".to_string()),
        Value::Symbol("c".to_string())
    ]))?;
    suite.assert_eval_eq("(vector->list #(1 2 3 4 5))", Value::List(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3)),
        Value::Literal(Literal::integer(4)),
        Value::Literal(Literal::integer(5))
    ]))?;
    
    // Test list->vector
    suite.assert_eval_eq("(list->vector '())", Value::Vector(vec![]))?;
    suite.assert_eval_eq("(list->vector '(a b c))", Value::Vector(vec![
        Value::Symbol("a".to_string()),
        Value::Symbol("b".to_string()),
        Value::Symbol("c".to_string())
    ]))?;
    suite.assert_eval_eq("(list->vector '(1 2 3))", Value::Vector(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3))
    ]))?;
    
    // Test round-trip conversion
    suite.assert_eval_eq("(list->vector (vector->list #(x y z)))", Value::Vector(vec![
        Value::Symbol("x".to_string()),
        Value::Symbol("y".to_string()),
        Value::Symbol("z".to_string())
    ]))?;
    suite.assert_eval_eq("(vector->list (list->vector '(p q r)))", Value::List(vec![
        Value::Symbol("p".to_string()),
        Value::Symbol("q".to_string()),
        Value::Symbol("r".to_string())
    ]))?;
    
    // Test conversion with improper list (should error)
    suite.assert_eval_error("(list->vector '(a . b))")?;
    
    Ok(())
}

/// Test additional vector operations
fn test_vector_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test vector-copy (if supported)
    if !suite.skip_if_unimplemented("vector-copy") {
        suite.eval("(define orig (vector 1 2 3))")?;
        suite.eval("(define copy (vector-copy orig))")?;
        
        // Verify copy has same contents
        suite.assert_eval_eq("copy", Value::Vector(vec![
            Value::Literal(Literal::integer(1)),
            Value::Literal(Literal::integer(2)),
            Value::Literal(Literal::integer(3))
        ]))?;
        
        // Verify they are independent
        suite.eval("(vector-set! orig 0 'changed)")?;
        suite.assert_eval_eq("(vector-ref orig 0)", Value::Symbol("changed".to_string()))?;
        suite.assert_eval_eq("(vector-ref copy 0)", Value::Literal(Literal::integer(1)))?;
    }
    
    // Test vector-copy! (if supported)
    if !suite.skip_if_unimplemented("vector-copy!") {
        suite.eval("(define dest (make-vector 5 'x))")?;
        suite.eval("(define src (vector 'a 'b 'c))")?;
        suite.eval("(vector-copy! dest 1 src)")?;
        
        suite.assert_eval_eq("dest", Value::Vector(vec![
            Value::Symbol("x".to_string()),
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("c".to_string()),
            Value::Symbol("x".to_string())
        ]))?;
    }
    
    // Test vector-append (if supported)
    if !suite.skip_if_unimplemented("vector-append") {
        suite.assert_eval_eq("(vector-append)", Value::Vector(vec![]))?;
        suite.assert_eval_eq("(vector-append #(a b))", Value::Vector(vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string())
        ]))?;
        suite.assert_eval_eq("(vector-append #(a b) #(c d))", Value::Vector(vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("c".to_string()),
            Value::Symbol("d".to_string())
        ]))?;
        suite.assert_eval_eq("(vector-append #(1) #() #(2 3))", Value::Vector(vec![
            Value::Literal(Literal::integer(1)),
            Value::Literal(Literal::integer(2)),
            Value::Literal(Literal::integer(3))
        ]))?;
    }
    
    Ok(())
}

/// Test vector equality and comparison
fn test_vector_equality(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test equal? with vectors
    suite.assert_eval_true("(equal? #() #())")?;
    suite.assert_eval_true("(equal? #(a b c) #(a b c))")?;
    suite.assert_eval_true("(equal? #(1 2 3) #(1 2 3))")?;
    
    suite.assert_eval_false("(equal? #(a b c) #(a b d))")?;
    suite.assert_eval_false("(equal? #(1 2) #(1 2 3))")?;
    suite.assert_eval_false("(equal? #(a b c) '(a b c))")?; // Vector != list
    
    // Test nested vectors
    suite.assert_eval_true("(equal? #(#(1 2) #(3 4)) #(#(1 2) #(3 4)))")?;
    suite.assert_eval_false("(equal? #(#(1 2) #(3 4)) #(#(1 2) #(3 5)))")?;
    
    // Test eqv? with vectors (should be false unless same object)
    suite.eval("(define vec1 #(a b c))")?;
    suite.eval("(define vec2 #(a b c))")?;
    suite.eval("(define vec3 vec1)")?;
    
    suite.assert_eval_false("(eqv? vec1 vec2)")?; // Different objects
    suite.assert_eval_true("(eqv? vec1 vec3)")?; // Same object
    
    // Test eq? with vectors (identity)
    suite.assert_eval_false("(eq? #(a) #(a))")?; // Literals might be different objects
    suite.assert_eval_true("(eq? vec1 vec3)")?; // Same object
    
    Ok(())
}

/// Test vector edge cases and error conditions
fn test_vector_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // Test very large vectors (if memory allows)
    let large_size = 1000;
    suite.eval(&format!("(define large-vec (make-vector {} 'x))", large_size))?;
    suite.assert_eval_eq(&format!("(vector-length large-vec)"), 
                       Value::Literal(Literal::integer(large_size)))?;
    suite.assert_eval_eq("(vector-ref large-vec 999)", Value::Symbol("x".to_string()))?;
    
    // Test vector with mixed types
    suite.assert_eval_eq("(vector 1 'a \"hello\" #t '(x y) #(nested))", Value::Vector(vec![
        Value::Literal(Literal::integer(1)),
        Value::Symbol("a".to_string()),
        Value::Literal(Literal::String("hello".to_string())),
        Value::Literal(Literal::Boolean(true)),
        Value::List(vec![Value::Symbol("x".to_string()), Value::Symbol("y".to_string())]),
        Value::Vector(vec![Value::Symbol("nested".to_string())])
    ]))?;
    
    // Test vector operations on empty vector
    suite.assert_eval_eq("(vector-length #())", Value::Literal(Literal::integer(0)))?;
    suite.eval("(vector-fill! #() 'x)")?; // Should succeed (no-op)
    
    // Test type errors
    suite.assert_eval_error("(vector-length 'not-a-vector)")?;
    suite.assert_eval_error("(vector-ref '(a b c) 0)")?;
    suite.assert_eval_error("(vector-set! '(a b c) 0 'x)")?;
    suite.assert_eval_error("(vector-fill! \"string\" 'x)")?;
    
    // Test argument count errors
    suite.assert_eval_error("(vector-ref #(a b c))")?; // Missing index
    suite.assert_eval_error("(vector-set! #(a b c) 0)")?; // Missing value
    suite.assert_eval_error("(vector-fill!)")?; // Missing arguments
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_operations_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Vector operations tests should pass");
    }
    
    #[test]
    fn test_vector_creation_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_vector_creation(&mut suite).expect("Vector creation tests should pass");
    }
    
    #[test]
    fn test_vector_access_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_vector_access(&mut suite).expect("Vector access tests should pass");
    }
    
    #[test]
    fn test_vector_conversion_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_vector_conversion(&mut suite).expect("Vector conversion tests should pass");
    }
}