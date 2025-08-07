//! R7RS String Operations Tests
//!
//! Tests for R7RS-small section 6.7 (Strings) including:
//! - String construction and manipulation
//! - String comparison operations
//! - String access and mutation
//! - String conversion operations
//! - String predicates and properties
//!
//! This module comprehensively tests string operations
//! required by the R7RS-small standard.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value, utils::SymbolId};
use std::sync::Arc;

/// Run all string operations tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS String Operations tests...");
    
    test_string_construction(suite)?;
    test_string_predicates(suite)?;
    test_string_comparison(suite)?;
    test_string_access(suite)?;
    test_string_mutation(suite)?;
    test_string_conversion(suite)?;
    test_string_case_operations(suite)?;
    test_string_edge_cases(suite)?;
    
    println!("✓ String operations tests passed");
    Ok(())
}

/// Test string construction operations
fn test_string_construction(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // make-string
    suite.assert_eval_eq("(string-length (make-string 5))", 
                       Value::Literal(Literal::integer(5)))?;
    suite.assert_eval_eq("(string-length (make-string 0))", 
                       Value::Literal(Literal::integer(0)))?;
    
    if !suite.skip_if_unimplemented("make-string with fill character") {
        suite.assert_eval_eq("(make-string 5 #\\x)", 
                           Value::Literal(Literal::String("xxxxx".to_string())))?;
        suite.assert_eval_eq("(make-string 3 #\\a)", 
                           Value::Literal(Literal::String("aaa".to_string())))?;
    }
    
    // string constructor
    suite.assert_eval_eq("(string)", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(string #\\h #\\e #\\l #\\l #\\o)", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(string #\\a)", 
                       Value::Literal(Literal::String("a".to_string())))?;
    
    // string-append
    suite.assert_eval_eq("(string-append)", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(string-append \"hello\")", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(string-append \"hello\" \" \" \"world\")", 
                       Value::Literal(Literal::String("hello world".to_string())))?;
    suite.assert_eval_eq("(string-append \"a\" \"b\" \"c\" \"d\")", 
                       Value::Literal(Literal::String("abcd".to_string())))?;
    suite.assert_eval_eq("(string-append \"\" \"hello\" \"\")", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    
    Ok(())
}

/// Test string predicates
fn test_string_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // string? predicate (already tested in basic_data_types, but comprehensive here)
    suite.assert_eval_true("(string? \"\")")?;
    suite.assert_eval_true("(string? \"hello\")")?;
    suite.assert_eval_true("(string? \"Hello, World!\")")?;
    suite.assert_eval_true("(string? \"123\")")?;
    suite.assert_eval_true("(string? \"with spaces\")")?;
    suite.assert_eval_true("(string? \"with\\nnewlines\")")?;
    
    // Non-strings
    suite.assert_eval_false("(string? 'symbol)")?;
    suite.assert_eval_false("(string? 42)")?;
    suite.assert_eval_false("(string? #\\c)")?;
    suite.assert_eval_false("(string? '())")?;
    suite.assert_eval_false("(string? '(\"hello\"))")?;
    suite.assert_eval_false("(string? #t)")?;
    
    Ok(())
}

/// Test string comparison operations
fn test_string_comparison(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // string=? (equality)
    suite.assert_eval_true("(string=? \"hello\" \"hello\")")?;
    suite.assert_eval_true("(string=? \"\" \"\")")?;
    suite.assert_eval_false("(string=? \"hello\" \"Hello\")")?;  // Case sensitive
    suite.assert_eval_false("(string=? \"hello\" \"world\")")?;
    suite.assert_eval_false("(string=? \"hello\" \"hello \")")?;  // Trailing space
    
    // Multiple argument equality
    suite.assert_eval_true("(string=? \"test\" \"test\" \"test\")")?;
    suite.assert_eval_false("(string=? \"test\" \"test\" \"Test\")")?;
    
    // string<? (lexicographic less than)
    suite.assert_eval_true("(string<? \"apple\" \"banana\")")?;
    suite.assert_eval_true("(string<? \"\" \"a\")")?;
    suite.assert_eval_true("(string<? \"a\" \"aa\")")?;
    suite.assert_eval_false("(string<? \"banana\" \"apple\")")?;
    suite.assert_eval_false("(string<? \"hello\" \"hello\")")?;
    suite.assert_eval_true("(string<? \"A\" \"a\")")?;  // Uppercase < lowercase in ASCII
    
    // string>? (lexicographic greater than)
    suite.assert_eval_true("(string>? \"banana\" \"apple\")")?;
    suite.assert_eval_true("(string>? \"a\" \"\")")?;
    suite.assert_eval_true("(string>? \"aa\" \"a\")")?;
    suite.assert_eval_false("(string>? \"apple\" \"banana\")")?;
    suite.assert_eval_false("(string>? \"hello\" \"hello\")")?;
    
    // string<=? and string>=?
    suite.assert_eval_true("(string<=? \"apple\" \"banana\")")?;
    suite.assert_eval_true("(string<=? \"hello\" \"hello\")")?;
    suite.assert_eval_false("(string<=? \"banana\" \"apple\")")?;
    
    suite.assert_eval_true("(string>=? \"banana\" \"apple\")")?;
    suite.assert_eval_true("(string>=? \"hello\" \"hello\")")?;
    suite.assert_eval_false("(string>=? \"apple\" \"banana\")")?;
    
    // Case-insensitive comparison (if implemented)
    if !suite.skip_if_unimplemented("case-insensitive string comparison") {
        suite.assert_eval_true("(string-ci=? \"Hello\" \"hello\")")?;
        suite.assert_eval_true("(string-ci=? \"HELLO\" \"hello\")")?;
        suite.assert_eval_false("(string-ci=? \"hello\" \"world\")")?;
        
        suite.assert_eval_true("(string-ci<? \"apple\" \"BANANA\")")?;
        suite.assert_eval_true("(string-ci>? \"BANANA\" \"apple\")")?;
        suite.assert_eval_true("(string-ci<=? \"Apple\" \"APPLE\")")?;
        suite.assert_eval_true("(string-ci>=? \"Apple\" \"APPLE\")")?;
    }
    
    Ok(())
}

/// Test string access operations
fn test_string_access(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // string-length
    suite.assert_eval_eq("(string-length \"\")", 
                       Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(string-length \"hello\")", 
                       Value::Literal(Literal::integer(5)))?;
    suite.assert_eval_eq("(string-length \"Hello, World!\")", 
                       Value::Literal(Literal::integer(13)))?;
    suite.assert_eval_eq("(string-length \"a\")", 
                       Value::Literal(Literal::integer(1)))?;
    
    // string-ref
    suite.assert_eval_eq("(string-ref \"hello\" 0)", 
                       Value::Literal(Literal::Character('h')))?;
    suite.assert_eval_eq("(string-ref \"hello\" 1)", 
                       Value::Literal(Literal::Character('e')))?;
    suite.assert_eval_eq("(string-ref \"hello\" 4)", 
                       Value::Literal(Literal::Character('o')))?;
    
    // Out of bounds should error
    suite.assert_eval_error("(string-ref \"hello\" 5)")?;
    suite.assert_eval_error("(string-ref \"hello\" -1)")?;
    suite.assert_eval_error("(string-ref \"\" 0)")?;
    
    // substring
    suite.assert_eval_eq("(substring \"hello world\" 0 5)", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(substring \"hello world\" 6 11)", 
                       Value::Literal(Literal::String("world".to_string())))?;
    suite.assert_eval_eq("(substring \"hello\" 1 4)", 
                       Value::Literal(Literal::String("ell".to_string())))?;
    suite.assert_eval_eq("(substring \"hello\" 0 0)", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(substring \"hello\" 5 5)", 
                       Value::Literal(Literal::String("".to_string())))?;
    
    // substring with single argument (from start to end)
    suite.assert_eval_eq("(substring \"hello\" 2)", 
                       Value::Literal(Literal::String("llo".to_string())))?;
    suite.assert_eval_eq("(substring \"hello\" 0)", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    
    // Invalid substring indices
    suite.assert_eval_error("(substring \"hello\" 6 7)")?;  // Start > length
    suite.assert_eval_error("(substring \"hello\" 3 2)")?;  // Start > end
    suite.assert_eval_error("(substring \"hello\" -1 3)")?; // Negative start
    
    Ok(())
}

/// Test string mutation operations
fn test_string_mutation(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("string mutation") {
        return Ok(());
    }
    
    // string-set! (mutates existing string)
    // Note: This test depends on how strings are implemented (mutable vs immutable)
    // In many Scheme implementations, string literals are immutable
    
    // Test with mutable string created by make-string
    suite.eval("(define test-str (make-string 5 #\\a))")?;
    suite.assert_eval_eq("test-str", 
                       Value::Literal(Literal::String("aaaaa".to_string())))?;
    
    suite.eval("(string-set! test-str 2 #\\x)")?;
    suite.assert_eval_eq("test-str", 
                       Value::Literal(Literal::String("aaxaa".to_string())))?;
    
    // Out of bounds should error
    suite.assert_eval_error("(string-set! test-str 5 #\\y)")?;
    suite.assert_eval_error("(string-set! test-str -1 #\\y)")?;
    
    // string-fill! (fills entire string with character)
    suite.eval("(define fill-str (make-string 3 #\\a))")?;
    suite.eval("(string-fill! fill-str #\\x)")?;
    suite.assert_eval_eq("fill-str", 
                       Value::Literal(Literal::String("xxx".to_string())))?;
    
    // string-copy (creates mutable copy)
    suite.eval("(define orig \"hello\")")?;
    suite.eval("(define copy (string-copy orig))")?;
    suite.assert_eval_eq("copy", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    
    // Verify copy is independent
    suite.eval("(string-set! copy 0 #\\H)")?;
    suite.assert_eval_eq("copy", 
                       Value::Literal(Literal::String("Hello".to_string())))?;
    suite.assert_eval_eq("orig", 
                       Value::Literal(Literal::String("hello".to_string())))?;  // Original unchanged
    
    Ok(())
}

/// Test string conversion operations
fn test_string_conversion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // string->list
    suite.assert_eval_eq("(string->list \"\")", 
                       Value::Nil)?;  // Empty list
    suite.assert_eval_eq("(string->list \"a\")", 
                       Value::Pair(
                           Arc::new(Value::Literal(Literal::Character('a'))),
                           Arc::new(Value::Nil)
                       ))?;
    
    // For longer strings, we'll test length and first element
    let result = suite.eval("(string->list \"hello\")")?;
    // This should be a list of characters: (#\h #\e #\l #\l #\o)
    suite.assert_eval_eq("(length (string->list \"hello\"))", 
                       Value::Literal(Literal::integer(5)))?;
    suite.assert_eval_eq("(car (string->list \"hello\"))", 
                       Value::Literal(Literal::Character('h')))?;
    
    // list->string
    suite.assert_eval_eq("(list->string '())", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(list->string '(#\\h #\\e #\\l #\\l #\\o))", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(list->string (list #\\a #\\b #\\c))", 
                       Value::Literal(Literal::String("abc".to_string())))?;
    
    // Round-trip conversion
    suite.assert_eval_eq("(list->string (string->list \"test\"))", 
                       Value::Literal(Literal::String("test".to_string())))?;
    suite.assert_eval_eq("(string->list (list->string '(#\\x #\\y #\\z)))", 
                       Value::Literal(Literal::String("xyz".to_string())))?;
    
    // Invalid list->string (non-character elements should error)
    suite.assert_eval_error("(list->string '(#\\a 42 #\\c))")?;
    suite.assert_eval_error("(list->string '(\"hello\"))")?;
    
    // symbol->string and string->symbol
    suite.assert_eval_eq("(symbol->string 'hello)", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(symbol->string '+)", 
                       Value::Literal(Literal::String("+".to_string())))?;
    
    suite.assert_eval_eq("(string->symbol \"hello\")", 
                       Value::Symbol(SymbolId::new(0)))?;  // Note: actual symbol ID depends on implementation
    
    // Round-trip symbol conversion
    suite.assert_eval_true("(eq? 'test (string->symbol (symbol->string 'test)))")?;
    
    Ok(())
}

/// Test string case operations
fn test_string_case_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("string case operations") {
        return Ok(());
    }
    
    // string-upcase
    suite.assert_eval_eq("(string-upcase \"hello\")", 
                       Value::Literal(Literal::String("HELLO".to_string())))?;
    suite.assert_eval_eq("(string-upcase \"Hello World\")", 
                       Value::Literal(Literal::String("HELLO WORLD".to_string())))?;
    suite.assert_eval_eq("(string-upcase \"\")", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(string-upcase \"123abc\")", 
                       Value::Literal(Literal::String("123ABC".to_string())))?;
    suite.assert_eval_eq("(string-upcase \"ALREADY\")", 
                       Value::Literal(Literal::String("ALREADY".to_string())))?;
    
    // string-downcase
    suite.assert_eval_eq("(string-downcase \"HELLO\")", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(string-downcase \"Hello World\")", 
                       Value::Literal(Literal::String("hello world".to_string())))?;
    suite.assert_eval_eq("(string-downcase \"\")", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(string-downcase \"123ABC\")", 
                       Value::Literal(Literal::String("123abc".to_string())))?;
    suite.assert_eval_eq("(string-downcase \"already\")", 
                       Value::Literal(Literal::String("already".to_string())))?;
    
    // string-foldcase (Unicode case folding - may be same as downcase for ASCII)
    suite.assert_eval_eq("(string-foldcase \"Hello\")", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    
    // Case operations should not modify original string
    suite.eval("(define orig \"Hello\")")?;
    suite.eval("(define upper (string-upcase orig))")?;
    suite.assert_eval_eq("orig", 
                       Value::Literal(Literal::String("Hello".to_string())))?;  // Original unchanged
    suite.assert_eval_eq("upper", 
                       Value::Literal(Literal::String("HELLO".to_string())))?;
    
    Ok(())
}

/// Test string edge cases and error conditions
fn test_string_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Empty string operations
    suite.assert_eval_eq("(string-length \"\")", 
                       Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(string-append \"\" \"\" \"\")", 
                       Value::Literal(Literal::String("".to_string())))?;
    suite.assert_eval_eq("(substring \"\" 0 0)", 
                       Value::Literal(Literal::String("".to_string())))?;
    
    // Very large strings (within reasonable limits)
    if !suite.skip_if_unimplemented("large strings") {
        suite.eval("(define big-str (make-string 1000 #\\x))")?;
        suite.assert_eval_eq("(string-length big-str)", 
                           Value::Literal(Literal::integer(1000)))?;
        suite.assert_eval_eq("(string-ref big-str 500)", 
                           Value::Literal(Literal::Character('x')))?;
    }
    
    // Strings with special characters
    suite.assert_eval_eq("(string-length \"\\n\\t\\r\")", 
                       Value::Literal(Literal::integer(3)))?;  // Escape sequences
    suite.assert_eval_eq("(string-ref \"\\\"quote\\\"\" 0)", 
                       Value::Literal(Literal::Character('"')))?;
    
    // Type errors
    suite.assert_eval_error("(string-length 42)")?;
    suite.assert_eval_error("(string-ref 'not-string 0)")?;
    suite.assert_eval_error("(string-append \"hello\" 42)")?;
    suite.assert_eval_error("(substring 123 0 2)")?;
    
    // Arity errors
    suite.assert_eval_error("(string-length)")?;
    suite.assert_eval_error("(string-length \"a\" \"b\")")?;
    suite.assert_eval_error("(string-ref \"hello\")")?;  // Missing index
    suite.assert_eval_error("(string-ref \"hello\" 0 1)")?;  // Too many args
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_operations_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("String operations tests should pass");
    }
    
    #[test]
    fn test_string_construction_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_string_construction(&mut suite).expect("String construction tests should pass");
    }
    
    #[test]
    fn test_string_comparison_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_string_comparison(&mut suite).expect("String comparison tests should pass");
    }
    
    #[test]
    fn test_string_access_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_string_access(&mut suite).expect("String access tests should pass");
    }
    
    #[test]
    fn test_string_conversion_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_string_conversion(&mut suite).expect("String conversion tests should pass");
    }
}