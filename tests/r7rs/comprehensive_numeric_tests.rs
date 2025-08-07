//! Comprehensive R7RS Numeric Operations Tests
//!
//! Tests for R7RS-small section 6.2 (Numbers) with complete coverage
//!
//! This module provides exhaustive testing of:
//! - All numeric types (exact/inexact integers, rationals, reals, complex)
//! - All arithmetic operations with proper exactness handling
//! - Number predicates and type queries
//! - Comparison operations
//! - Mathematical functions
//! - Number parsing and formatting
//! - Edge cases and error conditions
//!
//! All tests follow R7RS specification requirements exactly.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all comprehensive numeric tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Comprehensive Numeric Operations tests...");
    
    test_number_types_and_predicates(suite)?;
    test_exactness_system(suite)?;
    test_arithmetic_operations(suite)?;
    test_comparison_operations(suite)?;
    test_mathematical_functions(suite)?;
    test_rational_numbers(suite)?;
    test_complex_numbers(suite)?;
    test_number_conversion(suite)?;
    test_number_parsing(suite)?;
    test_numeric_edge_cases(suite)?;
    test_numeric_errors(suite)?;
    
    println!("✓ Comprehensive numeric operations tests passed");
    Ok(())
}

/// Test all number types and their predicates
fn test_number_types_and_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Integer tests
    suite.assert_eval_true("(integer? 0)")?;
    suite.assert_eval_true("(integer? 42)")?;
    suite.assert_eval_true("(integer? -17)")?;
    suite.assert_eval_true("(integer? 123456789012345678901234567890)")?; // Bigint
    
    // Real number tests
    suite.assert_eval_true("(real? 42)")?; // Integers are real
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(real? 3.14)")?;
        suite.assert_eval_true("(real? -2.718)")?;
        suite.assert_eval_true("(real? 1.0)")?;
        suite.assert_eval_true("(real? 0.0)")?;
        suite.assert_eval_true("(real? -0.0)")?;
    }
    
    // Rational number tests
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(rational? 42)")?; // Integers are rational
        suite.assert_eval_true("(rational? 22/7)")?;
        suite.assert_eval_true("(rational? -1/3)")?;
        suite.assert_eval_true("(rational? 0/1)")?;
        suite.assert_eval_true("(rational? 1/1)")?;
    }
    
    // Complex number tests
    if !suite.skip_if_unimplemented("complex numbers") {
        suite.assert_eval_true("(complex? 42)")?; // All numbers are complex
        suite.assert_eval_true("(complex? 3.14)")?;
        suite.assert_eval_true("(complex? 3+4i)")?;
        suite.assert_eval_true("(complex? 0+1i)")?;
        suite.assert_eval_true("(complex? 5+0i)")?;
        suite.assert_eval_true("(complex? 0+0i)")?;
    }
    
    // Number? predicate (most general)
    suite.assert_eval_true("(number? 42)")?;
    suite.assert_eval_true("(number? -17)")?;
    suite.assert_eval_true("(number? 0)")?;
    
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(number? 3.14)")?;
        suite.assert_eval_true("(number? -2.5)")?;
    }
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(number? 22/7)")?;
    }
    
    if !suite.skip_if_unimplemented("complex numbers") {
        suite.assert_eval_true("(number? 3+4i)")?;
    }
    
    // Non-numbers
    suite.assert_eval_false("(number? #t)")?;
    suite.assert_eval_false("(number? \"42\")")?;
    suite.assert_eval_false("(number? 'forty-two)")?;
    suite.assert_eval_false("(number? '())")?;
    suite.assert_eval_false("(number? (cons 1 2))")?;
    
    // Type hierarchy tests
    suite.assert_eval_true("(rational? 42)")?; // integer -> rational
    suite.assert_eval_true("(real? 42)")?; // integer -> real
    suite.assert_eval_true("(complex? 42)")?; // integer -> complex
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(real? 22/7)")?; // rational -> real
        suite.assert_eval_true("(complex? 22/7)")?; // rational -> complex
    }
    
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(complex? 3.14)")?; // real -> complex
    }
    
    Ok(())
}

/// Test exactness system
fn test_exactness_system(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("exact/inexact") {
        return Ok(());
    }
    
    // Exact predicates
    suite.assert_eval_true("(exact? 42)")?;
    suite.assert_eval_true("(exact? -17)")?;
    suite.assert_eval_true("(exact? 0)")?;
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(exact? 1/3)")?;
        suite.assert_eval_true("(exact? -22/7)")?;
    }
    
    // Inexact predicates
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(inexact? 3.14)")?;
        suite.assert_eval_true("(inexact? -2.5)")?;
        suite.assert_eval_true("(inexact? 1.0)")?;
        suite.assert_eval_false("(inexact? 42)")?;
    }
    
    // Exactness conversion
    suite.assert_eval_true("(exact? (inexact->exact 3.0))")?;
    suite.assert_eval_true("(inexact? (exact->inexact 42))")?;
    
    // Exactness preservation in arithmetic
    suite.assert_eval_true("(exact? (+ 1 2))")?;
    suite.assert_eval_true("(exact? (* 3 4))")?;
    suite.assert_eval_true("(exact? (- 10 5))")?;
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(exact? (/ 1 3))")?;
        suite.assert_eval_true("(exact? (+ 1/2 1/3))")?;
    }
    
    // Exactness contamination
    if !suite.skip_if_unimplemented("floating-point numbers") {
        suite.assert_eval_true("(inexact? (+ 1 2.0))")?;
        suite.assert_eval_true("(inexact? (* 3 4.0))")?;
    }
    
    Ok(())
}

/// Test arithmetic operations
fn test_arithmetic_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Addition
    suite.assert_eval_true("(= (+ 1 2 3) 6)")?;
    suite.assert_eval_true("(= (+) 0)")?; // Identity
    suite.assert_eval_true("(= (+ 5) 5)")?; // Single argument
    suite.assert_eval_true("(= (+ -3 3) 0)")?;
    
    // Subtraction
    suite.assert_eval_true("(= (- 10 3 2) 5)")?;
    suite.assert_eval_true("(= (- 5) -5)")?; // Negation
    suite.assert_eval_true("(= (- 0) 0)")?;
    
    // Multiplication
    suite.assert_eval_true("(= (* 2 3 4) 24)")?;
    suite.assert_eval_true("(= (*) 1)")?; // Identity
    suite.assert_eval_true("(= (* 7) 7)")?; // Single argument
    suite.assert_eval_true("(= (* -2 3) -6)")?;
    
    // Division
    suite.assert_eval_true("(= (/ 12 3) 4)")?;
    suite.assert_eval_true("(= (/ 20 4 2) 2.5)")?; // Multiple divisors
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(= (/ 1 3) 1/3)")?;
        suite.assert_eval_true("(= (/ 22 7) 22/7)")?;
    }
    
    // Reciprocal
    if !suite.skip_if_unimplemented("reciprocal") {
        suite.assert_eval_true("(= (/ 4) 1/4)")?;
        suite.assert_eval_true("(= (/ -2) -1/2)")?;
    }
    
    // Division by zero
    suite.assert_eval_error("(/ 1 0)")?;
    suite.assert_eval_error("(/ 0)")?;
    
    // Modulo and remainder
    suite.assert_eval_true("(= (modulo 13 4) 1)")?;
    suite.assert_eval_true("(= (modulo -13 4) 3)")?;
    suite.assert_eval_true("(= (modulo 13 -4) -3)")?;
    suite.assert_eval_true("(= (modulo -13 -4) -1)")?;
    
    suite.assert_eval_true("(= (remainder 13 4) 1)")?;
    suite.assert_eval_true("(= (remainder -13 4) -1)")?;
    suite.assert_eval_true("(= (remainder 13 -4) 1)")?;
    suite.assert_eval_true("(= (remainder -13 -4) -1)")?;
    
    // quotient
    suite.assert_eval_true("(= (quotient 13 4) 3)")?;
    suite.assert_eval_true("(= (quotient -13 4) -3)")?;
    suite.assert_eval_true("(= (quotient 13 -4) -3)")?;
    suite.assert_eval_true("(= (quotient -13 -4) 3)")?;
    
    Ok(())
}

/// Test comparison operations
fn test_comparison_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Equality
    suite.assert_eval_true("(= 1 1)")?;
    suite.assert_eval_true("(= 1 1.0)")?; // Different representations
    suite.assert_eval_true("(= 2 2 2 2)")?; // Multiple arguments
    suite.assert_eval_false("(= 1 2)")?;
    suite.assert_eval_false("(= 1 1 2)")?;
    
    if !suite.skip_if_unimplemented("rational numbers") {
        suite.assert_eval_true("(= 1/2 0.5)")?;
        suite.assert_eval_true("(= 3/4 0.75)")?;
    }
    
    // Less than
    suite.assert_eval_true("(< 1 2)")?;
    suite.assert_eval_true("(< 1 2 3 4)")?; // Multiple arguments
    suite.assert_eval_false("(< 2 1)")?;
    suite.assert_eval_false("(< 1 1)")?;
    suite.assert_eval_false("(< 1 2 2 3)")?; // Not strictly increasing
    
    // Less than or equal
    suite.assert_eval_true("(<= 1 2)")?;
    suite.assert_eval_true("(<= 1 1)")?; // Equal is OK
    suite.assert_eval_true("(<= 1 2 2 3)")?; // Non-decreasing
    suite.assert_eval_false("(<= 2 1)")?;
    
    // Greater than
    suite.assert_eval_true("(> 2 1)")?;
    suite.assert_eval_true("(> 4 3 2 1)")?; // Multiple arguments
    suite.assert_eval_false("(> 1 2)")?;
    suite.assert_eval_false("(> 1 1)")?;
    
    // Greater than or equal
    suite.assert_eval_true("(>= 2 1)")?;
    suite.assert_eval_true("(>= 2 2)")?; // Equal is OK
    suite.assert_eval_true("(>= 3 2 2 1)")?; // Non-increasing
    suite.assert_eval_false("(>= 1 2)")?;
    
    // Mixed type comparisons
    if !suite.skip_if_unimplemented("mixed type comparisons") {
        suite.assert_eval_true("(< 1 1.5 2)")?;
        suite.assert_eval_true("(= 2.0 2)")?;
        
        if !suite.skip_if_unimplemented("rational numbers") {
            suite.assert_eval_true("(< 1/2 0.6 2/3)")?;
        }
    }
    
    Ok(())
}

/// Test mathematical functions
fn test_mathematical_functions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic functions always required
    suite.assert_eval_true("(= (abs 5) 5)")?;
    suite.assert_eval_true("(= (abs -5) 5)")?;
    suite.assert_eval_true("(= (abs 0) 0)")?;
    
    // Floor, ceiling, truncate, round
    if !suite.skip_if_unimplemented("rounding functions") {
        suite.assert_eval_true("(= (floor 3.7) 3)")?;
        suite.assert_eval_true("(= (floor -3.7) -4)")?;
        
        suite.assert_eval_true("(= (ceiling 3.2) 4)")?;
        suite.assert_eval_true("(= (ceiling -3.2) -3)")?;
        
        suite.assert_eval_true("(= (truncate 3.7) 3)")?;
        suite.assert_eval_true("(= (truncate -3.7) -3)")?;
        
        suite.assert_eval_true("(= (round 3.2) 3)")?;
        suite.assert_eval_true("(= (round 3.7) 4)")?;
        suite.assert_eval_true("(= (round 3.5) 4)")?; // Banker's rounding
        suite.assert_eval_true("(= (round 2.5) 2)")?; // Banker's rounding
    }
    
    // Min and max
    suite.assert_eval_true("(= (min 1 2 3) 1)")?;
    suite.assert_eval_true("(= (min 3 1 2) 1)")?;
    suite.assert_eval_true("(= (min -5 -2 -10) -10)")?;
    
    suite.assert_eval_true("(= (max 1 2 3) 3)")?;
    suite.assert_eval_true("(= (max 3 1 2) 3)")?;
    suite.assert_eval_true("(= (max -5 -2 -10) -2)")?;
    
    // GCD and LCM
    if !suite.skip_if_unimplemented("gcd and lcm") {
        suite.assert_eval_true("(= (gcd 12 18) 6)")?;
        suite.assert_eval_true("(= (gcd 12 18 24) 6)")?;
        suite.assert_eval_true("(= (gcd 17 19) 1)")?; // Coprime
        suite.assert_eval_true("(= (gcd 0 5) 5)")?;
        suite.assert_eval_true("(= (gcd) 0)")?; // No arguments
        
        suite.assert_eval_true("(= (lcm 12 18) 36)")?;
        suite.assert_eval_true("(= (lcm 12 18 24) 72)")?;
        suite.assert_eval_true("(= (lcm 17 19) 323)")?; // Coprime
        suite.assert_eval_true("(= (lcm) 1)")?; // No arguments
        
        // GCD/LCM with negative numbers
        suite.assert_eval_true("(= (gcd -12 18) 6)")?;
        suite.assert_eval_true("(= (lcm -12 18) 36)")?;
    }
    
    // Exponentiation
    if !suite.skip_if_unimplemented("expt") {
        suite.assert_eval_true("(= (expt 2 3) 8)")?;
        suite.assert_eval_true("(= (expt 2 0) 1)")?;
        suite.assert_eval_true("(= (expt 0 5) 0)")?;
        suite.assert_eval_true("(= (expt 1 100) 1)")?;
        
        if !suite.skip_if_unimplemented("negative exponents") {
            suite.assert_eval_true("(= (expt 2 -3) 1/8)")?;
            suite.assert_eval_true("(= (expt 4 -1/2) 1/2)")?;
        }
    }
    
    // Square root
    if !suite.skip_if_unimplemented("sqrt") {
        suite.assert_eval_true("(= (sqrt 4) 2)")?;
        suite.assert_eval_true("(= (sqrt 9) 3)")?;
        suite.assert_eval_true("(= (sqrt 0) 0)")?;
        suite.assert_eval_true("(= (sqrt 1) 1)")?;
        
        // Negative square root should give complex result
        if !suite.skip_if_unimplemented("complex sqrt") {
            suite.assert_eval_true("(= (sqrt -1) 0+1i)")?;
            suite.assert_eval_true("(= (sqrt -4) 0+2i)")?;
        }
    }
    
    Ok(())
}

/// Test rational number operations
fn test_rational_numbers(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("rational numbers") {
        return Ok(());
    }
    
    // Basic rational arithmetic
    suite.assert_eval_true("(= (+ 1/2 1/3) 5/6)")?;
    suite.assert_eval_true("(= (- 3/4 1/4) 1/2)")?;
    suite.assert_eval_true("(= (* 2/3 3/4) 1/2)")?;
    suite.assert_eval_true("(= (/ 1/2 1/3) 3/2)")?;
    
    // Rational simplification
    suite.assert_eval_true("(= 6/8 3/4)")?;
    suite.assert_eval_true("(= 10/5 2)")?;
    suite.assert_eval_true("(= -4/8 -1/2)")?;
    
    // Rational predicates
    suite.assert_eval_true("(rational? 22/7)")?;
    suite.assert_eval_true("(rational? -1/3)")?;
    suite.assert_eval_true("(rational? 5)")?; // Integers are rational
    
    // Numerator and denominator
    if !suite.skip_if_unimplemented("numerator and denominator") {
        suite.assert_eval_true("(= (numerator 3/4) 3)")?;
        suite.assert_eval_true("(= (denominator 3/4) 4)")?;
        suite.assert_eval_true("(= (numerator 5) 5)")?;
        suite.assert_eval_true("(= (denominator 5) 1)")?;
        suite.assert_eval_true("(= (numerator -3/4) -3)")?;
        suite.assert_eval_true("(= (denominator -3/4) 4)")?;
    }
    
    // Mixed rational and integer arithmetic
    suite.assert_eval_true("(= (+ 1/2 2) 5/2)")?;
    suite.assert_eval_true("(= (* 3 2/3) 2)")?;
    
    Ok(())
}

/// Test complex number operations
fn test_complex_numbers(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("complex numbers") {
        return Ok(());
    }
    
    // Complex arithmetic
    suite.assert_eval_true("(= (+ 3+4i 1+2i) 4+6i)")?;
    suite.assert_eval_true("(= (- 5+3i 2+1i) 3+2i)")?;
    suite.assert_eval_true("(= (* 2+3i 1+1i) -1+5i)")?; // (2+3i)(1+i) = 2+2i+3i+3i² = 2+5i-3 = -1+5i
    
    // Complex with real arithmetic
    suite.assert_eval_true("(= (+ 3+4i 2) 5+4i)")?;
    suite.assert_eval_true("(= (* 2+3i 2) 4+6i)")?;
    
    // Real and imaginary parts
    if !suite.skip_if_unimplemented("real-part and imag-part") {
        suite.assert_eval_true("(= (real-part 3+4i) 3)")?;
        suite.assert_eval_true("(= (imag-part 3+4i) 4)")?;
        suite.assert_eval_true("(= (real-part 5) 5)")?;
        suite.assert_eval_true("(= (imag-part 5) 0)")?;
        suite.assert_eval_true("(= (real-part 0+7i) 0)")?;
        suite.assert_eval_true("(= (imag-part 0+7i) 7)")?;
    }
    
    // Magnitude and angle
    if !suite.skip_if_unimplemented("magnitude and angle") {
        suite.assert_eval_true("(= (magnitude 3+4i) 5)")?; // √(3²+4²) = 5
        suite.assert_eval_true("(= (magnitude 5) 5)")?;
        suite.assert_eval_true("(= (magnitude 0+1i) 1)")?;
        
        // Angle tests (approximate for floating point)
        suite.eval("(define pi-approx 3.141592653589793)")?;
        suite.assert_eval_true("(< (abs (- (angle 1+1i) (/ pi-approx 4))) 0.0001)")?; // 45 degrees
    }
    
    // Complex conjugate
    if !suite.skip_if_unimplemented("conjugate") {
        suite.assert_eval_true("(= (conjugate 3+4i) 3-4i)")?;
        suite.assert_eval_true("(= (conjugate 3-4i) 3+4i)")?;
        suite.assert_eval_true("(= (conjugate 5) 5)")?; // Real number
        suite.assert_eval_true("(= (conjugate 0+7i) 0-7i)")?;
    }
    
    // Make rectangular and polar
    if !suite.skip_if_unimplemented("make-rectangular and make-polar") {
        suite.assert_eval_true("(= (make-rectangular 3 4) 3+4i)")?;
        suite.assert_eval_true("(= (make-rectangular 5 0) 5)")?;
        suite.assert_eval_true("(= (make-rectangular 0 3) 0+3i)")?;
        
        // Polar form (approximate for floating point)
        suite.eval("(define c (make-polar 5 (/ pi-approx 4)))")?; // Should be ≈ 3.536+3.536i
        suite.assert_eval_true("(< (abs (- (real-part c) 3.536)) 0.001)")?;
        suite.assert_eval_true("(< (abs (- (imag-part c) 3.536)) 0.001)")?;
    }
    
    Ok(())
}

/// Test number conversion functions
fn test_number_conversion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Exactness conversion
    if !suite.skip_if_unimplemented("exactness conversion") {
        suite.assert_eval_true("(exact? (inexact->exact 3.0))")?;
        suite.assert_eval_true("(= (inexact->exact 3.0) 3)")?;
        suite.assert_eval_true("(inexact? (exact->inexact 3))")?;
        suite.assert_eval_true("(= (exact->inexact 3) 3.0)")?;
        
        if !suite.skip_if_unimplemented("rational inexact conversion") {
            suite.assert_eval_true("(= (inexact->exact 0.5) 1/2)")?;
            suite.assert_eval_true("(= (exact->inexact 1/2) 0.5)")?;
        }
    }
    
    // Number to string conversion
    if !suite.skip_if_unimplemented("number->string") {
        suite.assert_eval_true("(equal? (number->string 123) \"123\")")?;
        suite.assert_eval_true("(equal? (number->string -456) \"-456\")")?;
        suite.assert_eval_true("(equal? (number->string 0) \"0\")")?;
        
        // With radix
        suite.assert_eval_true("(equal? (number->string 15 16) \"f\")")?;
        suite.assert_eval_true("(equal? (number->string 15 8) \"17\")")?;
        suite.assert_eval_true("(equal? (number->string 15 2) \"1111\")")?;
        
        if !suite.skip_if_unimplemented("rational number->string") {
            suite.assert_eval_true("(equal? (number->string 1/2) \"1/2\")")?;
        }
        
        if !suite.skip_if_unimplemented("floating-point number->string") {
            suite.assert_eval_true("(equal? (number->string 3.14) \"3.14\")")?;
        }
        
        if !suite.skip_if_unimplemented("complex number->string") {
            suite.assert_eval_true("(equal? (number->string 3+4i) \"3+4i\")")?;
        }
    }
    
    // String to number conversion
    if !suite.skip_if_unimplemented("string->number") {
        suite.assert_eval_true("(= (string->number \"123\") 123)")?;
        suite.assert_eval_true("(= (string->number \"-456\") -456)")?;
        suite.assert_eval_true("(= (string->number \"0\") 0)")?;
        
        // With radix
        suite.assert_eval_true("(= (string->number \"f\" 16) 15)")?;
        suite.assert_eval_true("(= (string->number \"17\" 8) 15)")?;
        suite.assert_eval_true("(= (string->number \"1111\" 2) 15)")?;
        
        // Invalid strings
        suite.assert_eval_false("(string->number \"not-a-number\")")?;
        suite.assert_eval_false("(string->number \"12.34.56\")")?;
        suite.assert_eval_false("(string->number \"\")")?;
        
        if !suite.skip_if_unimplemented("rational string->number") {
            suite.assert_eval_true("(= (string->number \"1/2\") 1/2)")?;
            suite.assert_eval_true("(= (string->number \"22/7\") 22/7)")?;
        }
        
        if !suite.skip_if_unimplemented("floating-point string->number") {
            suite.assert_eval_true("(= (string->number \"3.14\") 3.14)")?;
            suite.assert_eval_true("(= (string->number \"1e3\") 1000.0)")?;
        }
        
        if !suite.skip_if_unimplemented("complex string->number") {
            suite.assert_eval_true("(= (string->number \"3+4i\") 3+4i)")?;
            suite.assert_eval_true("(= (string->number \"0+1i\") 0+1i)")?;
        }
    }
    
    Ok(())
}

/// Test number parsing edge cases
fn test_number_parsing(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic integer parsing
    suite.assert_eval_true("(= 42 42)")?;
    suite.assert_eval_true("(= -17 -17)")?;
    suite.assert_eval_true("(= +25 25)")?;
    
    // Hexadecimal literals
    if !suite.skip_if_unimplemented("hex literals") {
        suite.assert_eval_true("(= #x10 16)")?;
        suite.assert_eval_true("(= #xff 255)")?;
        suite.assert_eval_true("(= #xDEADBEEF 3735928559)")?;
    }
    
    // Octal literals
    if !suite.skip_if_unimplemented("octal literals") {
        suite.assert_eval_true("(= #o10 8)")?;
        suite.assert_eval_true("(= #o77 63)")?;
    }
    
    // Binary literals
    if !suite.skip_if_unimplemented("binary literals") {
        suite.assert_eval_true("(= #b10 2)")?;
        suite.assert_eval_true("(= #b1111 15)")?;
    }
    
    // Exactness prefixes
    if !suite.skip_if_unimplemented("exactness prefixes") {
        suite.assert_eval_true("(exact? #e3.14)")?;
        suite.assert_eval_true("(inexact? #i42)")?;
        suite.assert_eval_true("(= #e3.14 314/100)")?; // Exact representation
    }
    
    // Scientific notation
    if !suite.skip_if_unimplemented("scientific notation") {
        suite.assert_eval_true("(= 1e3 1000.0)")?;
        suite.assert_eval_true("(= 1.5e2 150.0)")?;
        suite.assert_eval_true("(= 1e-3 0.001)")?;
        suite.assert_eval_true("(= 2.5e-1 0.25)")?;
    }
    
    Ok(())
}

/// Test numeric edge cases
fn test_numeric_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Zero variations
    suite.assert_eval_true("(= 0 0)")?;
    suite.assert_eval_true("(= 0 -0)")?;
    suite.assert_eval_true("(= 0 +0)")?;
    
    if !suite.skip_if_unimplemented("floating-point zero") {
        suite.assert_eval_true("(= 0.0 -0.0)")?;
        suite.assert_eval_true("(= 0 0.0)")?;
    }
    
    // Large integers
    suite.eval("(define large-int 123456789012345678901234567890)")?;
    suite.assert_eval_true("(integer? large-int)")?;
    suite.assert_eval_true("(exact? large-int)")?;
    
    // Very small and large rational numbers
    if !suite.skip_if_unimplemented("extreme rationals") {
        suite.eval("(define tiny-rational 1/123456789012345678901234567890)")?;
        suite.assert_eval_true("(rational? tiny-rational)")?;
        suite.assert_eval_true("(exact? tiny-rational)")?;
        
        suite.eval("(define huge-rational 123456789012345678901234567890/1)")?;
        suite.assert_eval_true("(= huge-rational large-int)")?;
    }
    
    // Infinity and NaN (if supported)
    if !suite.skip_if_unimplemented("infinity and nan") {
        suite.assert_eval_true("(infinite? +inf.0)")?;
        suite.assert_eval_true("(infinite? -inf.0)")?;
        suite.assert_eval_false("(infinite? 42)")?;
        
        suite.assert_eval_true("(nan? +nan.0)")?;
        suite.assert_eval_false("(nan? 42)")?;
        suite.assert_eval_false("(nan? +inf.0)")?;
        
        // NaN is not equal to anything, including itself
        suite.assert_eval_false("(= +nan.0 +nan.0)")?;
        
        // Infinity arithmetic
        suite.assert_eval_true("(= (+ +inf.0 42) +inf.0)")?;
        suite.assert_eval_true("(= (* +inf.0 2) +inf.0)")?;
    }
    
    Ok(())
}

/// Test numeric error conditions
fn test_numeric_errors(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Division by zero
    suite.assert_eval_error("(/ 1 0)")?;
    suite.assert_eval_error("(/ 0)")?;
    suite.assert_eval_error("(quotient 1 0)")?;
    suite.assert_eval_error("(remainder 1 0)")?;
    suite.assert_eval_error("(modulo 1 0)")?;
    
    // Invalid operations
    suite.assert_eval_error("(+ 1 \"not-a-number\")")?;
    suite.assert_eval_error("(* #t 2)")?;
    suite.assert_eval_error("(< 1 'symbol)")?;
    
    // Type errors
    suite.assert_eval_error("(abs \"five\")")?;
    suite.assert_eval_error("(min 1 'two 3)")?;
    suite.assert_eval_error("(max #t #f)")?;
    
    // Invalid string->number conversions
    if !suite.skip_if_unimplemented("string->number") {
        suite.assert_eval_false("(string->number \"not-a-number\")")?;
        suite.assert_eval_false("(string->number \"1.2.3\")")?;
        suite.assert_eval_false("(string->number \"\")")?;
        suite.assert_eval_false("(string->number \"1/0\")")?; // Division by zero in rational
    }
    
    // Inexact->exact errors for non-integral inexacts
    if !suite.skip_if_unimplemented("inexact->exact errors") {
        suite.assert_eval_error("(inexact->exact +inf.0)")?;
        suite.assert_eval_error("(inexact->exact +nan.0)")?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comprehensive_numeric_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Comprehensive numeric tests should pass");
    }
    
    #[test]
    fn test_number_types_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_number_types_and_predicates(&mut suite).expect("Number type tests should pass");
    }
    
    #[test]
    fn test_arithmetic_operations_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_arithmetic_operations(&mut suite).expect("Arithmetic operation tests should pass");
    }
}