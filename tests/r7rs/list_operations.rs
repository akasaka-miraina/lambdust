//! R7RS List Operations Tests
//!
//! Tests for R7RS-small section 6.4 (Pairs and lists) including:
//! - Pair construction and access (cons, car, cdr)
//! - List construction and manipulation
//! - List predicates and length operations
//! - List traversal and search operations
//! - List modification operations
//! - Association lists and property lists
//!
//! This module comprehensively tests list and pair operations
//! required by the R7RS-small standard.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value, utils::intern_symbol};
use std::sync::Arc;

/// Run all list operations tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS List Operations tests...");
    
    test_pair_construction(suite)?;
    test_pair_access(suite)?;
    test_list_construction(suite)?;
    test_list_predicates(suite)?;
    test_list_length(suite)?;
    test_list_access(suite)?;
    test_list_traversal(suite)?;
    test_list_search(suite)?;
    test_list_modification(suite)?;
    test_association_lists(suite)?;
    test_list_edge_cases(suite)?;
    
    println!("✓ List operations tests passed");
    Ok(())
}

/// Test pair construction operations
fn test_pair_construction(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // cons - basic pair construction
    suite.assert_eval_eq("(cons 'a 'b)", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),
                           Arc::new(Value::symbol(intern_symbol("b")))
                       ))?;
    
    suite.assert_eval_eq("(cons 1 2)", 
                       Value::Pair(
                           Arc::new(Value::Literal(Literal::integer(1))),
                           Arc::new(Value::Literal(Literal::integer(2)))
                       ))?;
    
    suite.assert_eval_eq("(cons #t #f)", 
                       Value::Pair(
                           Arc::new(Value::Literal(Literal::Boolean(true))),
                           Arc::new(Value::Literal(Literal::Boolean(false)))
                       ))?;
    
    // cons with empty list creates single-element list
    suite.assert_eval_eq("(cons 'a '())", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),
                           Arc::new(Value::Nil)
                       ))?;
    
    // cons with existing list extends the list
    let result = suite.eval("(cons 'a '(b c))")?;
    // This should be '(a b c)
    suite.assert_eval_eq("(length (cons 'a '(b c)))", 
                       Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(car (cons 'a '(b c)))", 
                       Value::symbol(intern_symbol("a")))?;  // 'a
    
    // Nested cons
    suite.assert_eval_eq("(cons (cons 1 2) (cons 3 4))", 
                       Value::Pair(
                           Arc::new(Value::Pair(
                               Arc::new(Value::Literal(Literal::integer(1))),
                               Arc::new(Value::Literal(Literal::integer(2)))
                           )),
                           Arc::new(Value::Pair(
                               Arc::new(Value::Literal(Literal::integer(3))),
                               Arc::new(Value::Literal(Literal::integer(4)))
                           ))
                       ))?;
    
    Ok(())
}

/// Test pair access operations
fn test_pair_access(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // car - first element access
    suite.assert_eval_eq("(car '(a . b))", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(car '(1 . 2))", Value::Literal(Literal::integer(1)))?;
    suite.assert_eval_eq("(car '(a b c))", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(car (cons 'x 'y))", Value::symbol(intern_symbol("x")))?;  // 'x
    
    // cdr - rest element access
    suite.assert_eval_eq("(cdr '(a . b))", Value::symbol(intern_symbol("b")))?;  // 'b
    suite.assert_eval_eq("(cdr '(1 . 2))", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("(cdr '(a))", Value::Nil)?;  // cdr of single-element list is '()
    
    // For multi-element lists, cdr returns the rest of the list
    let result = suite.eval("(cdr '(a b c))")?;
    // This should be '(b c)
    suite.assert_eval_eq("(length (cdr '(a b c)))", 
                       Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("(car (cdr '(a b c)))", Value::symbol(intern_symbol("b")))?;  // 'b
    
    // Composite car/cdr operations
    suite.assert_eval_eq("(caar '((a b) c d))", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(cadr '(a b c d))", Value::symbol(intern_symbol("b")))?;    // 'b
    suite.assert_eval_eq("(cdar '((a b) c d))", Value::Nil)?;         // '(b) -> Check first
    suite.assert_eval_eq("(cddr '(a b c d))", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("c"))),  // 'c
                           Arc::new(Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("d"))),  // 'd
                               Arc::new(Value::Nil)
                           ))
                       ))?;  // '(c d)
    
    // More complex combinations
    if !suite.skip_if_unimplemented("complex car/cdr combinations") {
        suite.assert_eval_eq("(caddr '(a b c d))", Value::symbol(intern_symbol("c")))?;   // 'c
        suite.assert_eval_eq("(cadddr '(a b c d e))", Value::symbol(intern_symbol("d")))?; // 'd
        suite.assert_eval_eq("(cdaddr '(a b (c d e) f))", 
                           Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("d"))),  // 'd
                               Arc::new(Value::Pair(
                                   Arc::new(Value::symbol(intern_symbol("e"))),  // 'e
                                   Arc::new(Value::Nil)
                               ))
                           ))?;  // '(d e)
    }
    
    // Error cases - car/cdr on non-pairs
    suite.assert_eval_error("(car '())")?;      // car of empty list
    suite.assert_eval_error("(cdr '())")?;      // cdr of empty list
    suite.assert_eval_error("(car 42)")?;       // car of number
    suite.assert_eval_error("(cdr \"hello\")")?; // cdr of string
    suite.assert_eval_error("(car #t)")?;       // car of boolean
    
    Ok(())
}

/// Test list construction operations
fn test_list_construction(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // list constructor
    suite.assert_eval_eq("(list)", Value::Nil)?;
    
    suite.assert_eval_eq("(list 'a)", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),
                           Arc::new(Value::Nil)
                       ))?;
    
    // Multi-element lists
    let result = suite.eval("(list 'a 'b 'c)")?;
    suite.assert_eval_eq("(length (list 'a 'b 'c))", 
                       Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(car (list 'a 'b 'c))", Value::symbol(intern_symbol("a")))?;  // 'a
    
    // Mixed types
    suite.assert_eval_eq("(length (list 1 'a \"hello\" #t))", 
                       Value::Literal(Literal::integer(4)))?;
    
    // Nested lists
    suite.assert_eval_eq("(length (list '(a b) '(c d)))", 
                       Value::Literal(Literal::integer(2)))?;
    
    // Quote notation equivalence
    suite.assert_eval_true("(equal? '(a b c) (list 'a 'b 'c))")?;
    suite.assert_eval_true("(equal? '() (list))")?;
    
    Ok(())
}

/// Test list predicates
fn test_list_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // pair? predicate
    suite.assert_eval_true("(pair? '(a . b))")?;
    suite.assert_eval_true("(pair? '(a b c))")?;
    suite.assert_eval_true("(pair? (cons 1 2))")?;
    suite.assert_eval_false("(pair? '())")?;
    suite.assert_eval_false("(pair? 42)")?;
    suite.assert_eval_false("(pair? \"hello\")")?;
    suite.assert_eval_false("(pair? 'symbol)")?;
    
    // list? predicate
    suite.assert_eval_true("(list? '())")?;
    suite.assert_eval_true("(list? '(a))")?;
    suite.assert_eval_true("(list? '(a b c))")?;
    suite.assert_eval_true("(list? (list 1 2 3))")?;
    suite.assert_eval_false("(list? '(a . b))")?;  // Improper list
    suite.assert_eval_false("(list? 42)")?;
    suite.assert_eval_false("(list? \"hello\")")?;
    
    // null? predicate  
    suite.assert_eval_true("(null? '())")?;
    suite.assert_eval_true("(null? (list))")?;
    suite.assert_eval_false("(null? '(a))")?;
    suite.assert_eval_false("(null? '(a . b))")?;
    suite.assert_eval_false("(null? 42)")?;
    suite.assert_eval_false("(null? #f)")?;  // #f is not null
    suite.assert_eval_false("(null? \"\")")?;  // Empty string is not null
    
    Ok(())
}

/// Test list length operations
fn test_list_length(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // length function
    suite.assert_eval_eq("(length '())", Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(length '(a))", Value::Literal(Literal::integer(1)))?;
    suite.assert_eval_eq("(length '(a b))", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("(length '(a b c))", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(length (list 1 2 3 4 5))", Value::Literal(Literal::integer(5)))?;
    
    // Nested lists - length only counts top-level elements
    suite.assert_eval_eq("(length '((a b) (c d)))", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("(length '((a b c) (d) (e f g h)))", Value::Literal(Literal::integer(3)))?;
    
    // Error case - length on improper list
    suite.assert_eval_error("(length '(a . b))")?;
    suite.assert_eval_error("(length 42)")?;
    suite.assert_eval_error("(length \"hello\")")?;
    
    Ok(())
}

/// Test list access operations  
fn test_list_access(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // list-ref - access by index
    suite.assert_eval_eq("(list-ref '(a b c d) 0)", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(list-ref '(a b c d) 1)", Value::symbol(intern_symbol("b")))?;  // 'b
    suite.assert_eval_eq("(list-ref '(a b c d) 2)", Value::symbol(intern_symbol("c")))?;  // 'c
    suite.assert_eval_eq("(list-ref '(a b c d) 3)", Value::symbol(intern_symbol("d")))?;  // 'd
    
    // list-ref with numbers
    suite.assert_eval_eq("(list-ref '(10 20 30) 1)", Value::Literal(Literal::integer(20)))?;
    
    // list-ref with nested lists
    let result = suite.eval("(list-ref '((a b) (c d) (e f)) 1)")?;
    // This should be '(c d)
    suite.assert_eval_eq("(car (list-ref '((a b) (c d) (e f)) 1))", Value::symbol(intern_symbol("c")))?;  // 'c
    
    // Error cases
    suite.assert_eval_error("(list-ref '(a b c) 3)")?;   // Index too large
    suite.assert_eval_error("(list-ref '(a b c) -1)")?;  // Negative index
    suite.assert_eval_error("(list-ref '() 0)")?;        // Empty list
    suite.assert_eval_error("(list-ref 42 0)")?;         // Not a list
    
    // list-tail - get suffix starting at index
    let result = suite.eval("(list-tail '(a b c d) 0)")?;
    suite.assert_eval_eq("(length (list-tail '(a b c d) 0))", 
                       Value::Literal(Literal::integer(4)))?;  // Whole list
    
    suite.assert_eval_eq("(length (list-tail '(a b c d) 2))", 
                       Value::Literal(Literal::integer(2)))?;  // '(c d)
    suite.assert_eval_eq("(car (list-tail '(a b c d) 2))", Value::symbol(intern_symbol("c")))?;  // 'c
    
    suite.assert_eval_eq("(list-tail '(a b c d) 4)", Value::Nil)?;  // Empty list
    
    // Error cases for list-tail
    suite.assert_eval_error("(list-tail '(a b c) 4)")?;   // Index too large
    suite.assert_eval_error("(list-tail '(a b c) -1)")?;  // Negative index
    suite.assert_eval_error("(list-tail 42 0)")?;         // Not a list
    
    Ok(())
}

/// Test list traversal operations
fn test_list_traversal(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // append - concatenate lists
    suite.assert_eval_eq("(append)", Value::Nil)?;
    suite.assert_eval_eq("(append '(a b))", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
                           Arc::new(Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
                               Arc::new(Value::Nil)
                           ))
                       ))?;
    
    let result = suite.eval("(append '(a b) '(c d))")?;
    suite.assert_eval_eq("(length (append '(a b) '(c d)))", 
                       Value::Literal(Literal::integer(4)))?;
    suite.assert_eval_eq("(car (append '(a b) '(c d)))", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(car (list-tail (append '(a b) '(c d)) 2))", Value::symbol(intern_symbol("c")))?;  // 'c
    
    // Multiple list append
    suite.assert_eval_eq("(length (append '(a) '(b) '(c) '(d)))", 
                       Value::Literal(Literal::integer(4)))?;
    
    // Append with empty lists
    suite.assert_eval_eq("(append '() '(a b))", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
                           Arc::new(Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
                               Arc::new(Value::Nil)
                           ))
                       ))?;
    
    suite.assert_eval_eq("(append '(a b) '())", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
                           Arc::new(Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
                               Arc::new(Value::Nil)
                           ))
                       ))?;
    
    // reverse - reverse list order
    suite.assert_eval_eq("(reverse '())", Value::Nil)?;
    
    let result = suite.eval("(reverse '(a b c))")?;
    suite.assert_eval_eq("(length (reverse '(a b c)))", 
                       Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(car (reverse '(a b c)))", Value::symbol(intern_symbol("c")))?;  // 'c
    suite.assert_eval_eq("(car (list-tail (reverse '(a b c)) 2))", Value::symbol(intern_symbol("a")))?;  // 'a
    
    // Reverse single element
    suite.assert_eval_eq("(car (reverse '(x)))", Value::symbol(intern_symbol("x")))?;  // 'x
    suite.assert_eval_eq("(length (reverse '(x)))", Value::Literal(Literal::integer(1)))?;
    
    // Error cases
    suite.assert_eval_error("(append 42 '(a b))")?;  // First arg not list
    suite.assert_eval_error("(reverse 42)")?;        // Not a list
    
    Ok(())
}

/// Test list search operations
fn test_list_search(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // memq - membership test with eq?
    let result = suite.eval("(memq 'b '(a b c d))")?;
    suite.assert_eval_eq("(car (memq 'b '(a b c d)))", Value::symbol(intern_symbol("b")))?;  // 'b
    suite.assert_eval_eq("(length (memq 'b '(a b c d)))", 
                       Value::Literal(Literal::integer(3)))?;  // '(b c d)
    
    suite.assert_eval_eq("(memq 'x '(a b c d))", Value::Literal(Literal::Boolean(false)))?;
    suite.assert_eval_eq("(memq 'a '(a b c d))", 
                       Value::Pair(
                           Arc::new(Value::symbol(intern_symbol("a"))),  // Returns the sublist starting with 'a
                           Arc::new(Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("b"))),
                               Arc::new(Value::Pair(
                                   Arc::new(Value::symbol(intern_symbol("c"))),
                                   Arc::new(Value::Pair(
                                       Arc::new(Value::symbol(intern_symbol("d"))),
                                       Arc::new(Value::Nil)
                                   ))
                               ))
                           ))
                       ))?;
    
    // memv - membership test with eqv?
    suite.assert_eval_eq("(car (memv 2 '(1 2 3 4)))", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("(memv 5 '(1 2 3 4))", Value::Literal(Literal::Boolean(false)))?;
    
    // member - membership test with equal?
    suite.assert_eval_eq("(car (member \"hello\" '(\"hi\" \"hello\" \"world\")))", 
                       Value::Literal(Literal::String("hello".to_string())))?;
    suite.assert_eval_eq("(member \"missing\" '(\"hi\" \"hello\" \"world\"))", 
                       Value::Literal(Literal::Boolean(false)))?;
    
    // Member with nested structures
    let result = suite.eval("(member '(a b) '((x y) (a b) (c d)))")?;
    // Should return '((a b) (c d))
    suite.assert_eval_eq("(length (member '(a b) '((x y) (a b) (c d))))", 
                       Value::Literal(Literal::integer(2)))?;
    
    Ok(())
}

/// Test list modification operations
fn test_list_modification(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("list mutation") {
        return Ok(());
    }
    
    // set-car! and set-cdr! modify existing pairs
    suite.eval("(define test-pair (cons 'a 'b))")?;
    suite.assert_eval_eq("(car test-pair)", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(cdr test-pair)", Value::symbol(intern_symbol("b")))?;  // 'b
    
    suite.eval("(set-car! test-pair 'x)")?;
    suite.assert_eval_eq("(car test-pair)", Value::symbol(intern_symbol("x")))?;  // 'x (symbol ID might differ)
    suite.assert_eval_eq("(cdr test-pair)", Value::symbol(intern_symbol("b")))?;  // 'b (unchanged)
    
    suite.eval("(set-cdr! test-pair 'y)")?;
    suite.assert_eval_eq("(car test-pair)", Value::symbol(intern_symbol("x")))?;  // 'x
    suite.assert_eval_eq("(cdr test-pair)", Value::symbol(intern_symbol("y")))?;  // 'y
    
    // Modifying lists
    suite.eval("(define test-list (list 1 2 3))")?;
    suite.assert_eval_eq("(car test-list)", Value::Literal(Literal::integer(1)))?;
    
    suite.eval("(set-car! test-list 10)")?;
    suite.assert_eval_eq("(car test-list)", Value::Literal(Literal::integer(10)))?;
    suite.assert_eval_eq("(car (cdr test-list))", Value::Literal(Literal::integer(2)))?;  // Unchanged
    
    // Error cases - modifying immutable structures or non-pairs
    suite.assert_eval_error("(set-car! '() 'x)")?;     // Empty list
    suite.assert_eval_error("(set-car! 42 'x)")?;      // Not a pair
    suite.assert_eval_error("(set-cdr! \"hello\" 'x)")?; // Not a pair
    
    Ok(())
}

/// Test association list operations
fn test_association_lists(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // assq - association with eq?
    let result = suite.eval("(assq 'b '((a 1) (b 2) (c 3)))")?;
    suite.assert_eval_eq("(car (assq 'b '((a 1) (b 2) (c 3))))", Value::symbol(intern_symbol("b")))?;  // 'b
    suite.assert_eval_eq("(car (cdr (assq 'b '((a 1) (b 2) (c 3)))))", 
                       Value::Literal(Literal::integer(2)))?;
    
    suite.assert_eval_eq("(assq 'x '((a 1) (b 2) (c 3)))", 
                       Value::Literal(Literal::Boolean(false)))?;
    
    // assv - association with eqv?
    suite.assert_eval_eq("(car (cdr (assv 2 '((1 \"one\") (2 \"two\") (3 \"three\")))))", 
                       Value::Literal(Literal::String("two".to_string())))?;
    suite.assert_eval_eq("(assv 4 '((1 \"one\") (2 \"two\") (3 \"three\")))", 
                       Value::Literal(Literal::Boolean(false)))?;
    
    // assoc - association with equal?
    let result = suite.eval("(assoc \"key\" '((\"hello\" 1) (\"key\" 2) (\"world\" 3)))")?;
    suite.assert_eval_eq("(car (cdr (assoc \"key\" '((\"hello\" 1) (\"key\" 2) (\"world\" 3)))))", 
                       Value::Literal(Literal::integer(2)))?;
    
    // Association with complex keys
    let result = suite.eval("(assoc '(a b) '(((x y) 1) ((a b) 2) ((c d) 3)))")?;
    suite.assert_eval_eq("(car (cdr (assoc '(a b) '(((x y) 1) ((a b) 2) ((c d) 3)))))", 
                       Value::Literal(Literal::integer(2)))?;
    
    // Empty association list
    suite.assert_eval_eq("(assq 'a '())", Value::Literal(Literal::Boolean(false)))?;
    
    // Malformed association list (should still work for valid entries)
    suite.assert_eval_eq("(car (cdr (assq 'a '((a 1) b (c 3)))))", 
                       Value::Literal(Literal::integer(1)))?;
    
    Ok(())
}

/// Test list edge cases and error conditions
fn test_list_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Empty list operations
    suite.assert_eval_eq("(length '())", Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(append '())", Value::Nil)?;
    suite.assert_eval_eq("(reverse '())", Value::Nil)?;
    
    // Single element lists
    suite.assert_eval_eq("(car (reverse '(x)))", Value::symbol(intern_symbol("x")))?;  // 'x
    suite.assert_eval_eq("(cdr '(x))", Value::Nil)?;
    
    // Improper lists (dotted pairs that aren't proper lists)
    suite.assert_eval_true("(pair? '(a . b))")?;
    suite.assert_eval_false("(list? '(a . b))")?;
    suite.assert_eval_eq("(car '(a . b))", Value::symbol(intern_symbol("a")))?;  // 'a
    suite.assert_eval_eq("(cdr '(a . b))", Value::symbol(intern_symbol("b")))?;  // 'b
    
    // Long lists (within reasonable limits)
    if !suite.skip_if_unimplemented("long lists") {
        // Create a list of 100 elements
        suite.eval("(define long-list (let loop ((n 100) (acc '())) (if (= n 0) acc (loop (- n 1) (cons n acc)))))")?;
        suite.assert_eval_eq("(length long-list)", Value::Literal(Literal::integer(100)))?;
        suite.assert_eval_eq("(car long-list)", Value::Literal(Literal::integer(1)))?;
        suite.assert_eval_eq("(list-ref long-list 50)", Value::Literal(Literal::integer(51)))?;
    }
    
    // Circular lists (if detected)
    if !suite.skip_if_unimplemented("circular list detection") {
        suite.eval("(define circular (list 'a 'b 'c))")?;
        suite.eval("(set-cdr! (cddr circular) circular)")?;  // Make it circular
        // length should either error or detect the cycle
        suite.assert_eval_error("(length circular)")?;
    }
    
    // Type errors
    suite.assert_eval_error("(car 42)")?;
    suite.assert_eval_error("(cdr \"hello\")")?;
    suite.assert_eval_error("(length 'not-a-list)")?;
    suite.assert_eval_error("(list-ref 42 0)")?;
    suite.assert_eval_error("(append 'not-a-list '(a b))")?;
    suite.assert_eval_error("(reverse 'not-a-list)")?;
    
    // Arity errors
    suite.assert_eval_error("(cons 1)")?;          // cons needs 2 args
    suite.assert_eval_error("(cons 1 2 3)")?;      // cons takes exactly 2 args
    suite.assert_eval_error("(car)")?;             // car needs 1 arg
    suite.assert_eval_error("(car 'a 'b)")?;       // car takes exactly 1 arg
    suite.assert_eval_error("(list-ref '(a b) 0 1)")?; // list-ref takes exactly 2 args
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_operations_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("List operations tests should pass");
    }
    
    #[test]
    fn test_pair_construction_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_pair_construction(&mut suite).expect("Pair construction tests should pass");
    }
    
    #[test]
    fn test_pair_access_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_pair_access(&mut suite).expect("Pair access tests should pass");
    }
    
    #[test]
    fn test_list_construction_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_list_construction(&mut suite).expect("List construction tests should pass");
    }
    
    #[test]
    fn test_list_search_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_list_search(&mut suite).expect("List search tests should pass");
    }
    
    #[test]
    fn test_association_lists_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_association_lists(&mut suite).expect("Association list tests should pass");
    }
}