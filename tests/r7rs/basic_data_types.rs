//! R7RS Basic Data Types and Predicates Tests
//!
//! Tests for R7RS-small section 6.1 (Equivalence predicates),
//! section 6.2 (Numbers), section 6.3 (Booleans), section 6.4 (Pairs and lists),
//! section 6.5 (Symbols), section 6.6 (Characters), section 6.7 (Strings),
//! section 6.8 (Vectors), and section 6.9 (Bytevectors).
//!
//! This module verifies that the basic Scheme data types are properly
//! implemented with correct type predicates and basic operations.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all basic data type and predicate tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Basic Data Types tests...");
    
    test_boolean_predicates(suite)?;
    test_number_predicates(suite)?;
    test_character_predicates(suite)?;
    test_string_predicates(suite)?;
    test_symbol_predicates(suite)?;
    test_pair_list_predicates(suite)?;
    test_vector_predicates(suite)?;
    test_procedure_predicates(suite)?;
    test_equivalence_predicates(suite)?;
    
    println!("✓ Basic data types tests passed");
    Ok(())
}

/// Test boolean type and predicates
fn test_boolean_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // boolean? predicate tests
    suite.assert_eval_true("(boolean? #t)")?;
    suite.assert_eval_true("(boolean? #f)")?;
    suite.assert_eval_false("(boolean? 0)")?;
    suite.assert_eval_false("(boolean? 1)")?;
    suite.assert_eval_false("(boolean? '())")?;
    suite.assert_eval_false("(boolean? \"#f\")")?;
    suite.assert_eval_false("(boolean? 'true)")?;
    
    // Boolean values should be self-evaluating
    suite.assert_eval_eq("#t", Value::Literal(Literal::Boolean(true)))?;
    suite.assert_eval_eq("#f", Value::Literal(Literal::Boolean(false)))?;
    
    // Boolean truthiness (R7RS: only #f is false, everything else is true)
    suite.assert_eval_true("(if #t #t #f)")?;
    suite.assert_eval_false("(if #f #t #f)")?;
    suite.assert_eval_true("(if 0 #t #f)")?;  // 0 is truthy in Scheme
    suite.assert_eval_true("(if '() #t #f)")?;  // '() is truthy in Scheme
    suite.assert_eval_true("(if \"\" #t #f)")?;  // Empty string is truthy
    
    Ok(())
}

/// Test number type and predicates
fn test_number_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // number? predicate tests
    suite.assert_eval_true("(number? 42)")?;
    suite.assert_eval_true("(number? -17)")?;
    suite.assert_eval_true("(number? 0)")?;
    
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(number? 3.14)")?;
        suite.assert_eval_true("(number? -2.5)")?;
    }
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(number? 22/7)")?;
        suite.assert_eval_true("(number? -1/3)")?;
    }
    
    if !suite.skip_if_unimplemented("complex numbers") {
        suite.assert_eval_true("(number? 3+4i)")?;
        suite.assert_eval_true("(number? 0+1i)")?;
    }
    
    // Non-numbers
    suite.assert_eval_false("(number? #t)")?;
    suite.assert_eval_false("(number? \"42\")")?;
    suite.assert_eval_false("(number? 'forty-two)")?;
    suite.assert_eval_false("(number? '())")?;
    suite.assert_eval_false("(number? (cons 1 2))")?;
    
    // Numeric subtype predicates
    suite.assert_eval_true("(integer? 42)")?;
    suite.assert_eval_true("(integer? -17)")?;
    suite.assert_eval_true("(integer? 0)")?;
    suite.assert_eval_false("(integer? #t)")?;
    
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(real? 3.14)")?;
        suite.assert_eval_true("(real? 42)")?;  // Integers are real
        suite.assert_eval_false("(real? #f)")?;
    }
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(rational? 22/7)")?;
        suite.assert_eval_true("(rational? 42)")?;  // Integers are rational
        suite.assert_eval_false("(rational? #f)")?;
    }
    
    if !suite.skip_if_unimplemented("complex numbers") {
        suite.assert_eval_true("(complex? 3+4i)")?;
        suite.assert_eval_true("(complex? 42)")?;  // All numbers are complex
        suite.assert_eval_false("(complex? #f)")?;
    }
    
    // Exactness predicates
    if !suite.skip_if_unimplemented("exact/inexact") {
        suite.assert_eval_true("(exact? 42)")?;
        suite.assert_eval_true("(exact? 1/3)")?;
        suite.assert_eval_false("(exact? 3.14)")?;
        suite.assert_eval_true("(inexact? 3.14)")?;
        suite.assert_eval_false("(inexact? 42)")?;
    }
    
    Ok(())
}

/// Test character type and predicates
fn test_character_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // char? predicate tests
    suite.assert_eval_true("(char? #\\a)")?;
    suite.assert_eval_true("(char? #\\A)")?;
    suite.assert_eval_true("(char? #\\1)")?;
    suite.assert_eval_true("(char? #\\space)")?;
    suite.assert_eval_true("(char? #\\newline)")?;
    
    // Non-characters
    suite.assert_eval_false("(char? \"a\")")?;
    suite.assert_eval_false("(char? 'a)")?;
    suite.assert_eval_false("(char? 97)")?;  // ASCII value of 'a'
    suite.assert_eval_false("(char? #t)")?;
    suite.assert_eval_false("(char? '())")?;
    
    Ok(())
}

/// Test string type and predicates
fn test_string_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // string? predicate tests
    suite.assert_eval_true("(string? \"hello\")")?;
    suite.assert_eval_true("(string? \"\")")?;  // Empty string
    suite.assert_eval_true("(string? \"Hello, World!\")")?;
    suite.assert_eval_true("(string? \"42\")")?;
    
    // Non-strings
    suite.assert_eval_false("(string? 'hello)")?;
    suite.assert_eval_false("(string? #\\h)")?;
    suite.assert_eval_false("(string? 42)")?;
    suite.assert_eval_false("(string? #t)")?;
    suite.assert_eval_false("(string? '())")?;
    suite.assert_eval_false("(string? '(#\\h #\\e #\\l #\\l #\\o))")?;
    
    Ok(())
}

/// Test symbol type and predicates
fn test_symbol_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // symbol? predicate tests
    suite.assert_eval_true("(symbol? 'hello)")?;
    suite.assert_eval_true("(symbol? 'HELLO)")?;
    suite.assert_eval_true("(symbol? 'hello-world)")?;
    suite.assert_eval_true("(symbol? '+)")?;
    suite.assert_eval_true("(symbol? '42x)")?;  // Symbols can start with numbers
    
    // Non-symbols
    suite.assert_eval_false("(symbol? \"hello\")")?;
    suite.assert_eval_false("(symbol? #\\h)")?;
    suite.assert_eval_false("(symbol? 42)")?;
    suite.assert_eval_false("(symbol? #t)")?;
    suite.assert_eval_false("(symbol? '())")?;
    
    Ok(())
}

/// Test pair and list predicates
fn test_pair_list_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // pair? predicate tests
    suite.assert_eval_true("(pair? '(a . b))")?;
    suite.assert_eval_true("(pair? '(a b c))")?;  // Lists are pairs
    suite.assert_eval_true("(pair? (cons 1 2))")?;
    
    // Non-pairs
    suite.assert_eval_false("(pair? '())")?;  // Empty list is not a pair
    suite.assert_eval_false("(pair? 42)")?;
    suite.assert_eval_false("(pair? \"hello\")")?;
    suite.assert_eval_false("(pair? 'symbol)")?;
    suite.assert_eval_false("(pair? #t)")?;
    
    // list? predicate tests
    suite.assert_eval_true("(list? '())")?;  // Empty list
    suite.assert_eval_true("(list? '(a))")?;
    suite.assert_eval_true("(list? '(a b c))")?;
    suite.assert_eval_true("(list? (list 1 2 3))")?;
    
    // Non-lists (including improper lists)
    suite.assert_eval_false("(list? '(a . b))")?;  // Improper list
    suite.assert_eval_false("(list? 42)")?;
    suite.assert_eval_false("(list? \"hello\")")?;
    suite.assert_eval_false("(list? 'symbol)")?;
    
    // null? predicate tests
    suite.assert_eval_true("(null? '())")?;
    suite.assert_eval_false("(null? '(a))")?;
    suite.assert_eval_false("(null? '(a . b))")?;
    suite.assert_eval_false("(null? 42)")?;
    suite.assert_eval_false("(null? #f)")?;  // #f is not null
    suite.assert_eval_false("(null? \"\")")?;  // Empty string is not null
    
    Ok(())
}

/// Test vector predicates
fn test_vector_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("vectors") {
        return Ok(());
    }
    
    // vector? predicate tests
    suite.assert_eval_true("(vector? #(1 2 3))")?;
    suite.assert_eval_true("(vector? #())")?;  // Empty vector
    suite.assert_eval_true("(vector? (vector 'a 'b 'c))")?;
    suite.assert_eval_true("(vector? (make-vector 5))")?;
    
    // Non-vectors
    suite.assert_eval_false("(vector? '(1 2 3))")?;  // List is not vector
    suite.assert_eval_false("(vector? \"hello\")")?;
    suite.assert_eval_false("(vector? 42)")?;
    suite.assert_eval_false("(vector? #t)")?;
    suite.assert_eval_false("(vector? '())")?;
    
    Ok(())
}

/// Test procedure predicates
fn test_procedure_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // procedure? predicate tests
    suite.assert_eval_true("(procedure? car)")?;
    suite.assert_eval_true("(procedure? +)")?;
    suite.assert_eval_true("(procedure? (lambda (x) x))")?;
    
    if !suite.skip_if_unimplemented("continuations") {
        suite.assert_eval_true("(procedure? (call/cc (lambda (k) k)))")?;
    }
    
    // Non-procedures
    suite.assert_eval_false("(procedure? 42)")?;
    suite.assert_eval_false("(procedure? \"hello\")")?;
    suite.assert_eval_false("(procedure? 'symbol)")?;
    suite.assert_eval_false("(procedure? #t)")?;
    suite.assert_eval_false("(procedure? '())")?;
    suite.assert_eval_false("(procedure? '(lambda (x) x))")?;  // Quoted lambda is a list
    
    Ok(())
}

/// Test equivalence predicates (eq?, eqv?, equal?)
fn test_equivalence_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // eq? tests - identity equality
    suite.assert_eval_true("(eq? 'a 'a)")?;
    suite.assert_eval_true("(eq? #t #t)")?;
    suite.assert_eval_true("(eq? #f #f)")?;
    suite.assert_eval_true("(eq? '() '())")?;
    suite.assert_eval_false("(eq? 'a 'b)")?;
    suite.assert_eval_false("(eq? #t #f)")?;
    
    // Numbers and eq? (implementation-dependent, but small integers often eq?)
    suite.assert_eval_true("(eq? 0 0)")?;
    suite.assert_eval_true("(eq? 1 1)")?;
    
    // eqv? tests - operational equivalence
    suite.assert_eval_true("(eqv? 'a 'a)")?;
    suite.assert_eval_true("(eqv? #t #t)")?;
    suite.assert_eval_true("(eqv? #f #f)")?;
    suite.assert_eval_true("(eqv? '() '())")?;
    suite.assert_eval_true("(eqv? 42 42)")?;
    suite.assert_eval_false("(eqv? 'a 'b)")?;
    suite.assert_eval_false("(eqv? 42 43)")?;
    
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(eqv? 3.14 3.14)")?;
        suite.assert_eval_false("(eqv? 3.14 3.15)")?;
    }
    
    // equal? tests - structural equality
    suite.assert_eval_true("(equal? 'a 'a)")?;
    suite.assert_eval_true("(equal? #t #t)")?;
    suite.assert_eval_true("(equal? 42 42)")?;
    suite.assert_eval_true("(equal? \"hello\" \"hello\")")?;
    suite.assert_eval_true("(equal? '(a b c) '(a b c))")?;
    suite.assert_eval_true("(equal? '(a (b c) d) '(a (b c) d))")?;
    
    suite.assert_eval_false("(equal? 'a 'b)")?;
    suite.assert_eval_false("(equal? 42 43)")?;
    suite.assert_eval_false("(equal? \"hello\" \"world\")")?;
    suite.assert_eval_false("(equal? '(a b c) '(a b d))")?;
    
    if !suite.skip_if_unimplemented("vectors") {
        suite.assert_eval_true("(equal? #(a b c) #(a b c))")?;
        suite.assert_eval_false("(equal? #(a b c) #(a b d))")?;
        suite.assert_eval_false("(equal? '(a b c) #(a b c))")?;  // List != vector
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_data_types_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Basic data types tests should pass");
    }
    
    #[test] 
    fn test_boolean_predicates_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_boolean_predicates(&mut suite).expect("Boolean predicate tests should pass");
    }
    
    #[test]
    fn test_equivalence_predicates_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_equivalence_predicates(&mut suite).expect("Equivalence predicate tests should pass");
    }
}