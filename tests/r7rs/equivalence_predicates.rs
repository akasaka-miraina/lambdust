//! R7RS Equivalence Predicates Tests (Section 6.1)
//!
//! Comprehensive tests for R7RS-small section 6.1 equivalence predicates:
//! - eq? (identity/pointer equivalence)
//! - eqv? (operational equivalence) 
//! - equal? (structural equivalence)
//!
//! These predicates form the foundation of Scheme's equality system
//! and must be implemented correctly for proper R7RS compliance.
//! This module tests all edge cases and interactions between the
//! three equivalence predicates.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all equivalence predicate tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Equivalence Predicates tests...");
    
    test_eq_predicate(suite)?;
    test_eqv_predicate(suite)?;
    test_equal_predicate(suite)?;
    test_equivalence_hierarchy(suite)?;
    test_equivalence_with_all_types(suite)?;
    test_equivalence_edge_cases(suite)?;
    test_equivalence_performance(suite)?;
    
    println!("✓ Equivalence predicates tests passed");
    Ok(())
}

/// Test eq? predicate (identity/pointer equivalence)
fn test_eq_predicate(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // eq? should be true for identical objects
    
    // Symbols (interned, so should be eq?)
    suite.assert_eval_true("(eq? 'a 'a)")?;
    suite.assert_eval_true("(eq? 'hello 'hello)")?;
    suite.assert_eval_false("(eq? 'a 'b)")?;
    
    // Booleans (singleton values)
    suite.assert_eval_true("(eq? #t #t)")?;
    suite.assert_eval_true("(eq? #f #f)")?;
    suite.assert_eval_false("(eq? #t #f)")?;
    
    // Empty list (singleton value)
    suite.assert_eval_true("(eq? '() '())")?;
    
    // Unspecified value (if accessible)
    if !suite.skip_if_unimplemented("unspecified value access") {
        suite.assert_eval_true("(eq? (if #f #f) (if #f #f))")?;
    }
    
    // Small integers (often cached/interned)
    suite.assert_eval_true("(eq? 0 0)")?;
    suite.assert_eval_true("(eq? 1 1)")?;
    suite.assert_eval_true("(eq? -1 -1)")?;
    suite.assert_eval_true("(eq? 42 42)")?;
    
    // Numbers that might not be eq? (implementation dependent)
    // R7RS allows but doesn't require eq? to be true for numbers
    let large_num = 1000000;
    let eq_result = suite.eval(&format!("(eq? {} {})", large_num, large_num))?;
    // We don't assert the result since it's implementation-dependent
    println!("  eq? for large numbers: {:?}", eq_result);
    
    // Characters (might be cached)
    suite.assert_eval_true("(eq? #\\a #\\a)")?;
    suite.assert_eval_false("(eq? #\\a #\\b)")?;
    
    // eq? should be false for different types
    suite.assert_eval_false("(eq? 42 '42)")?;
    suite.assert_eval_false("(eq? 42 \"42\")")?;
    suite.assert_eval_false("(eq? #t 1)")?;
    suite.assert_eval_false("(eq? '() #f)")?;
    
    // Compound objects should not be eq? unless same instance
    suite.assert_eval_false("(eq? '(a) '(a))")?;  // Different pairs
    suite.assert_eval_false("(eq? \"hello\" \"hello\")")?;  // Different strings (usually)
    
    // Same reference should be eq?
    suite.eval("(define x '(a b c))")?;
    suite.eval("(define y x)")?;
    suite.assert_eval_true("(eq? x y)")?;
    suite.assert_eval_true("(eq? x x)")?;
    
    // Procedures 
    suite.eval("(define proc1 (lambda (x) x))")?;
    suite.eval("(define proc2 (lambda (x) x))")?;
    suite.eval("(define proc3 proc1)")?;
    
    suite.assert_eval_false("(eq? proc1 proc2)")?;  // Different procedures
    suite.assert_eval_true("(eq? proc1 proc3)")?;   // Same procedure
    suite.assert_eval_true("(eq? proc1 proc1)")?;   // Same procedure
    
    // Built-in procedures
    suite.assert_eval_true("(eq? car car)")?;
    suite.assert_eval_true("(eq? + +)")?;
    suite.assert_eval_false("(eq? car cdr)")?;
    
    Ok(())
}

/// Test eqv? predicate (operational equivalence)
fn test_eqv_predicate(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // eqv? includes everything eq? does, plus operational equivalence
    
    // Everything that's eq? should be eqv?
    suite.assert_eval_true("(eqv? 'a 'a)")?;
    suite.assert_eval_true("(eqv? #t #t)")?;
    suite.assert_eval_true("(eqv? #f #f)")?;
    suite.assert_eval_true("(eqv? '() '())")?;
    
    // Numbers: eqv? should be true for same values regardless of identity
    suite.assert_eval_true("(eqv? 42 42)")?;
    suite.assert_eval_true("(eqv? 0 0)")?;
    suite.assert_eval_true("(eqv? -17 -17)")?;
    suite.assert_eval_false("(eqv? 42 43)")?;
    
    // Large numbers should be eqv? even if not eq?
    suite.assert_eval_true("(eqv? 1000000 1000000)")?;
    suite.assert_eval_true("(eqv? -999999 -999999)")?;
    
    // Floating-point numbers (if supported)
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(eqv? 3.14 3.14)")?;
        suite.assert_eval_true("(eqv? 0.0 0.0)")?;
        suite.assert_eval_false("(eqv? 3.14 3.15)")?;
        
        // Special floating-point values (if supported)
        if !suite.skip_if_unimplemented("special float values") {
            suite.assert_eval_true("(eqv? +inf.0 +inf.0)")?;
            suite.assert_eval_true("(eqv? -inf.0 -inf.0)")?;
            suite.assert_eval_false("(eqv? +inf.0 -inf.0)")?;
            
            // NaN is not eqv? to itself in IEEE 754
            suite.assert_eval_false("(eqv? +nan.0 +nan.0)")?;
        }
    }
    
    // Rational numbers (if supported) 
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(eqv? 1/2 1/2)")?;
        suite.assert_eval_true("(eqv? 22/7 22/7)")?;
        suite.assert_eval_false("(eqv? 1/2 1/3)")?;
        
        // Reduced rationals should be equivalent
        suite.assert_eval_true("(eqv? 2/4 1/2)")?;
        suite.assert_eval_true("(eqv? 6/9 2/3)")?;
    }
    
    // Complex numbers (if supported)
    if !suite.skip_if_unimplemented("complex numbers") {
        suite.assert_eval_true("(eqv? 3+4i 3+4i)")?;
        suite.assert_eval_true("(eqv? 0+1i 0+1i)")?;
        suite.assert_eval_false("(eqv? 3+4i 4+3i)")?;
        
        // Real numbers as complex numbers
        suite.assert_eval_true("(eqv? 5 5+0i)")?;
        suite.assert_eval_true("(eqv? 0 0+0i)")?;
    }
    
    // Characters
    suite.assert_eval_true("(eqv? #\\a #\\a)")?;
    suite.assert_eval_true("(eqv? #\\space #\\space)")?;
    suite.assert_eval_false("(eqv? #\\a #\\A)")?;
    suite.assert_eval_false("(eqv? #\\a #\\b)")?;
    
    // Different types
    suite.assert_eval_false("(eqv? 42 '42)")?;
    suite.assert_eval_false("(eqv? 42 \"42\")")?;
    suite.assert_eval_false("(eqv? #\\0 0)")?;
    suite.assert_eval_false("(eqv? #t 1)")?;
    
    // Mutable objects (should be false unless same object)
    suite.assert_eval_false("(eqv? '(a) '(a))")?;
    suite.assert_eval_false("(eqv? \"hello\" \"hello\")")?;
    
    if !suite.skip_if_unimplemented("vectors") {
        suite.assert_eval_false("(eqv? #(1 2 3) #(1 2 3))")?;
    }
    
    // Same object should be eqv?
    suite.eval("(define lst '(a b c))")?;
    suite.eval("(define same-lst lst)")?;
    suite.assert_eval_true("(eqv? lst same-lst)")?;
    
    // Procedures
    suite.eval("(define f1 (lambda (x) x))")?;
    suite.eval("(define f2 (lambda (x) x))")?;
    suite.eval("(define f3 f1)")?;
    
    suite.assert_eval_false("(eqv? f1 f2)")?;  // Different procedures
    suite.assert_eval_true("(eqv? f1 f3)")?;   // Same procedure
    
    Ok(())
}

/// Test equal? predicate (structural equivalence)
fn test_equal_predicate(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // equal? includes everything eqv? does, plus structural equivalence
    
    // Everything that's eqv? should be equal?
    suite.assert_eval_true("(equal? 42 42)")?;
    suite.assert_eval_true("(equal? 'hello 'hello)")?;
    suite.assert_eval_true("(equal? #t #t)")?;
    suite.assert_eval_true("(equal? #\\a #\\a)")?;
    
    // Strings: equal? compares content
    suite.assert_eval_true("(equal? \"hello\" \"hello\")")?;
    suite.assert_eval_true("(equal? \"\" \"\")")?;
    suite.assert_eval_false("(equal? \"hello\" \"world\")")?;
    suite.assert_eval_false("(equal? \"Hello\" \"hello\")")?; // Case sensitive
    
    // Lists: equal? compares structure recursively
    suite.assert_eval_true("(equal? '() '())")?;
    suite.assert_eval_true("(equal? '(a) '(a))")?;
    suite.assert_eval_true("(equal? '(a b c) '(a b c))")?;
    suite.assert_eval_false("(equal? '(a b c) '(a b d))")?;
    suite.assert_eval_false("(equal? '(a b) '(a b c))")?;
    
    // Nested lists
    suite.assert_eval_true("(equal? '((a b) (c d)) '((a b) (c d)))")?;
    suite.assert_eval_true("(equal? '(a (b (c d) e) f) '(a (b (c d) e) f))")?;
    suite.assert_eval_false("(equal? '((a b) (c d)) '((a b) (c e)))")?;
    
    // Improper lists (dotted pairs)
    suite.assert_eval_true("(equal? '(a . b) '(a . b))")?;
    suite.assert_eval_true("(equal? '(a b . c) '(a b . c))")?;
    suite.assert_eval_false("(equal? '(a . b) '(a . c))")?;
    suite.assert_eval_false("(equal? '(a b c) '(a b . c))")?; // Proper vs improper
    
    // Mixed structures
    suite.assert_eval_true("(equal? '(a \"hello\" 42 #t) '(a \"hello\" 42 #t))")?;
    suite.assert_eval_false("(equal? '(a \"hello\" 42 #t) '(a \"world\" 42 #t))")?;
    
    // Vectors (if supported)
    if !suite.skip_if_unimplemented("vectors") {
        suite.assert_eval_true("(equal? #() #())")?;
        suite.assert_eval_true("(equal? #(a b c) #(a b c))")?;
        suite.assert_eval_false("(equal? #(a b c) #(a b d))")?;
        suite.assert_eval_false("(equal? #(a b) #(a b c))")?;
        
        // Nested vectors
        suite.assert_eval_true("(equal? #(#(1 2) #(3 4)) #(#(1 2) #(3 4)))")?;
        suite.assert_eval_false("(equal? #(#(1 2) #(3 4)) #(#(1 2) #(3 5)))")?;
        
        // Vectors vs lists (different types)
        suite.assert_eval_false("(equal? '(a b c) #(a b c))")?;
        suite.assert_eval_false("(equal? #(a b c) '(a b c))")?;
    }
    
    // Mixed list/vector structures (if vectors supported)
    if !suite.skip_if_unimplemented("vectors") {
        suite.assert_eval_true("(equal? '(a #(b c) d) '(a #(b c) d))")?;
        suite.assert_eval_true("(equal? #('(a b) c) #('(a b) c))")?;
        suite.assert_eval_false("(equal? '(a #(b c) d) '(a #(b d) d))")?;
    }
    
    // Bytevectors (if supported)
    if !suite.skip_if_unimplemented("bytevectors") {
        suite.assert_eval_true("(equal? #u8() #u8())")?;
        suite.assert_eval_true("(equal? #u8(1 2 3) #u8(1 2 3))")?;
        suite.assert_eval_false("(equal? #u8(1 2 3) #u8(1 2 4))")?;
        suite.assert_eval_false("(equal? #u8(1 2) #u8(1 2 3))")?;
    }
    
    // Procedures are not equal? unless same object
    suite.eval("(define p1 (lambda (x) x))")?;
    suite.eval("(define p2 (lambda (x) x))")?;
    suite.eval("(define p3 p1)")?;
    
    suite.assert_eval_false("(equal? p1 p2)")?;  // Different procedures
    suite.assert_eval_true("(equal? p1 p3)")?;   // Same procedure
    
    // Different types
    suite.assert_eval_false("(equal? 42 \"42\")")?;
    suite.assert_eval_false("(equal? '42 42)")?;
    suite.assert_eval_false("(equal? #t 1)")?;
    suite.assert_eval_false("(equal? '() #f)")?;
    
    Ok(())
}

/// Test equivalence predicate hierarchy (eq? implies eqv? implies equal?)
fn test_equivalence_hierarchy(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // If (eq? x y) is true, then (eqv? x y) should be true
    // If (eqv? x y) is true, then (equal? x y) should be true
    
    // Test with symbols
    suite.eval("(define test-hierarchy\n                (lambda (x y)\n                  (let ((eq-result (eq? x y))\n                        (eqv-result (eqv? x y))\n                        (equal-result (equal? x y)))\n                    (and (or (not eq-result) eqv-result)\n                         (or (not eqv-result) equal-result)))))")?;
    
    // Test hierarchy with various values
    suite.assert_eval_true("(test-hierarchy 'a 'a)")?;
    suite.assert_eval_true("(test-hierarchy 'a 'b)")?;
    suite.assert_eval_true("(test-hierarchy 42 42)")?;
    suite.assert_eval_true("(test-hierarchy 42 43)")?;
    suite.assert_eval_true("(test-hierarchy #t #t)")?;
    suite.assert_eval_true("(test-hierarchy #t #f)")?;
    suite.assert_eval_true("(test-hierarchy '() '())")?;
    suite.assert_eval_true("(test-hierarchy '(a) '(a))")?;
    suite.assert_eval_true("(test-hierarchy \"hello\" \"hello\")")?;
    suite.assert_eval_true("(test-hierarchy \"hello\" \"world\")")?;
    
    // Specific hierarchy examples
    
    // These should be eq?, eqv?, and equal?
    suite.assert_eval_true("(and (eq? 'symbol 'symbol)\n                           (eqv? 'symbol 'symbol)\n                           (equal? 'symbol 'symbol))")?;
    
    // These should be eqv? and equal?, but not necessarily eq?
    suite.assert_eval_true("(and (eqv? 1000000 1000000)\n                           (equal? 1000000 1000000))")?;
    
    // These should be equal? but not eqv? or eq?
    suite.assert_eval_true("(equal? '(a b c) '(a b c))")?;
    suite.assert_eval_false("(eqv? '(a b c) '(a b c))")?;
    suite.assert_eval_false("(eq? '(a b c) '(a b c))")?;
    
    suite.assert_eval_true("(equal? \"hello\" \"hello\")")?;
    // Note: strings might be eqv? or eq? in some implementations, but not required
    
    Ok(())
}

/// Test equivalence predicates with all data types
fn test_equivalence_with_all_types(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Create a comprehensive test with mixed data structures
    
    let complex_data1 = r#"'(numbers (42 3.14 1/2) 
                                    symbols (hello world) 
                                    strings ("foo" "bar")
                                    booleans (#t #f)
                                    lists ((a b) (c d) ())
                                    char #\x)"#;
    
    let complex_data2 = r#"'(numbers (42 3.14 1/2) 
                                    symbols (hello world) 
                                    strings ("foo" "bar")
                                    booleans (#t #f)
                                    lists ((a b) (c d) ())
                                    char #\x)"#;
    
    let complex_data3 = r#"'(numbers (42 3.14 1/2) 
                                    symbols (hello world) 
                                    strings ("foo" "bar")
                                    booleans (#t #f)
                                    lists ((a b) (c d) ())
                                    char #\y)"#;  // Different character
    
    suite.eval(&format!("(define data1 {})", complex_data1))?;
    suite.eval(&format!("(define data2 {})", complex_data2))?;
    suite.eval(&format!("(define data3 {})", complex_data3))?;
    
    // Should be structurally equal
    suite.assert_eval_true("(equal? data1 data2)")?;
    suite.assert_eval_false("(equal? data1 data3)")?;
    
    // Should not be eqv? (different objects)
    suite.assert_eval_false("(eqv? data1 data2)")?;
    suite.assert_eval_false("(eqv? data1 data3)")?;
    
    // Should not be eq? (different objects)
    suite.assert_eval_false("(eq? data1 data2)")?;
    suite.assert_eval_false("(eq? data1 data3)")?;
    
    // Same reference should be all three
    suite.eval("(define data1-ref data1)")?;
    suite.assert_eval_true("(equal? data1 data1-ref)")?;
    suite.assert_eval_true("(eqv? data1 data1-ref)")?;
    suite.assert_eval_true("(eq? data1 data1-ref)")?;
    
    Ok(())
}

/// Test equivalence predicate edge cases
fn test_equivalence_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Circular structures (if supported)
    if !suite.skip_if_unimplemented("circular structures") {
        suite.eval("(define circ1 (list 'a 'b))")?;
        suite.eval("(set-cdr! (cdr circ1) circ1)")?; // Make circular
        suite.eval("(define circ2 (list 'a 'b))")?;
        suite.eval("(set-cdr! (cdr circ2) circ2)")?; // Make circular
        
        // equal? should handle circular structures
        suite.assert_eval_true("(equal? circ1 circ1)")?;
        // circ1 and circ2 have same structure, so should be equal? if implementation handles it
        // (This is implementation-dependent)\n        let circ_result = suite.eval(\"(equal? circ1 circ2)\");\n        println!(\"  Circular structure equal? result: {:?}\", circ_result);\n    }\n    \n    // Very deep structures\n    suite.eval(\"(define deep1 '(((((((((a))))))))))\")?\n    suite.eval(\"(define deep2 '(((((((((a))))))))))\")?\n    suite.eval(\"(define deep3 '(((((((((b))))))))))\")?\n    \n    suite.assert_eval_true(\"(equal? deep1 deep2)\")?\n    suite.assert_eval_false(\"(equal? deep1 deep3)\")?\n    \n    // Empty structures\n    suite.assert_eval_true(\"(equal? '() '())\")?\n    suite.assert_eval_true(\"(equal? \\\"\\\" \\\"\\\")\")?\n    \n    if !suite.skip_if_unimplemented(\"vectors\") {\n        suite.assert_eval_true(\"(equal? #() #())\")?\n    }\n    \n    // Arity tests - all predicates should take exactly 2 arguments\n    suite.assert_eval_error(\"(eq? 'a)\")?\n    suite.assert_eval_error(\"(eq? 'a 'b 'c)\")?\n    suite.assert_eval_error(\"(eqv?)\")?\n    suite.assert_eval_error(\"(eqv? 'a 'b 'c)\")?\n    suite.assert_eval_error(\"(equal?)\")?\n    suite.assert_eval_error(\"(equal? 'a 'b 'c)\")?\n    \n    Ok(())\n}\n\n/// Test equivalence predicate performance characteristics\nfn test_equivalence_performance(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {\n    // This test is more about ensuring the predicates work correctly\n    // with large data structures, not actual performance measurement\n    \n    // Large list\n    suite.eval(\"(define large-list (make-list 1000 'x))\")?\n    suite.eval(\"(define large-list2 (make-list 1000 'x))\")?\n    suite.eval(\"(define large-list3 (make-list 1000 'y))\")?\n    \n    // Should work correctly even with large structures\n    suite.assert_eval_true(\"(equal? large-list large-list2)\")?\n    suite.assert_eval_false(\"(equal? large-list large-list3)\")?\n    \n    // eq? and eqv? should be fast regardless of structure size\n    suite.assert_eval_true(\"(eq? large-list large-list)\")?\n    suite.assert_eval_false(\"(eq? large-list large-list2)\")?\n    suite.assert_eval_true(\"(eqv? large-list large-list)\")?\n    suite.assert_eval_false(\"(eqv? large-list large-list2)\")?\n    \n    // Wide structure (many elements at same level)\n    let wide_list = (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(\" \");\n    suite.eval(&format!(\"(define wide1 '({}))\", wide_list))?\n    suite.eval(&format!(\"(define wide2 '({}))\", wide_list))?\n    suite.eval(&format!(\"(define wide3 '({} 999))\", wide_list))? // Extra element\n    \n    suite.assert_eval_true(\"(equal? wide1 wide2)\")?\n    suite.assert_eval_false(\"(equal? wide1 wide3)\")?\n    \n    println!(\"  Performance tests completed (functional correctness verified)\");\n    Ok(())\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    \n    #[test]\n    fn test_equivalence_predicates_suite() {\n        let mut suite = R7RSTestSuite::new();\n        run_tests(&mut suite).expect(\"Equivalence predicates tests should pass\");\n    }\n    \n    #[test]\n    fn test_eq_predicate_isolated() {\n        let mut suite = R7RSTestSuite::new();\n        test_eq_predicate(&mut suite).expect(\"eq? predicate tests should pass\");\n    }\n    \n    #[test]\n    fn test_eqv_predicate_isolated() {\n        let mut suite = R7RSTestSuite::new();\n        test_eqv_predicate(&mut suite).expect(\"eqv? predicate tests should pass\");\n    }\n    \n    #[test]\n    fn test_equal_predicate_isolated() {\n        let mut suite = R7RSTestSuite::new();\n        test_equal_predicate(&mut suite).expect(\"equal? predicate tests should pass\");\n    }\n    \n    #[test]\n    fn test_equivalence_hierarchy_isolated() {\n        let mut suite = R7RSTestSuite::new();\n        test_equivalence_hierarchy(&mut suite).expect(\"Equivalence hierarchy tests should pass\");\n    }\n}"
}]