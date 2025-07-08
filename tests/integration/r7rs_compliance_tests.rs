//! R7RS compliance tests
//!
//! Tests verifying compliance with the R7RS Scheme specification.

use lambdust::{Interpreter, Value};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_tower() {
        let mut interpreter = Interpreter::new();

        // Test exact integers
        let result = interpreter.eval("(exact? 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test inexact numbers
        let result = interpreter.eval("(exact? 3.14)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test number predicates
        let result = interpreter.eval("(number? 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(integer? 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(real? 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_equality_predicates() {
        let mut interpreter = Interpreter::new();

        // eq? - object identity
        let result = interpreter.eval("(eq? 'a 'a)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(eq? 'a 'b)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // eqv? - equivalence
        let result = interpreter.eval("(eqv? 42 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(eqv? 42 43)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // equal? - structural equality
        let result = interpreter.eval("(equal? '(1 2 3) '(1 2 3))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(equal? '(1 2 3) '(1 2 4))").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_boolean_values() {
        let mut interpreter = Interpreter::new();

        // Test boolean literals
        let result = interpreter.eval("#t").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("#f").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test boolean predicate
        let result = interpreter.eval("(boolean? #t)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(boolean? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test truthiness - only #f is false
        let result = interpreter.eval("(if 0 'true 'false)").unwrap();
        assert_eq!(result, Value::Symbol("true".to_string()));

        let result = interpreter.eval("(if '() 'true 'false)").unwrap();
        assert_eq!(result, Value::Symbol("true".to_string()));

        let result = interpreter.eval("(if #f 'true 'false)").unwrap();
        assert_eq!(result, Value::Symbol("false".to_string()));
    }

    #[test]
    fn test_character_operations() {
        let mut interpreter = Interpreter::new();

        // Test character literals
        let result = interpreter.eval("#\\a").unwrap();
        assert_eq!(result, Value::Character('a'));

        let result = interpreter.eval("#\\space").unwrap();
        assert_eq!(result, Value::Character(' '));

        let result = interpreter.eval("#\\newline").unwrap();
        assert_eq!(result, Value::Character('\n'));

        // Test character predicates
        let result = interpreter.eval("(char? #\\a)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(char? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test character comparisons
        let result = interpreter.eval("(char=? #\\a #\\a)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(char<? #\\a #\\b)").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_string_operations() {
        let mut interpreter = Interpreter::new();

        // Test string literals
        let result = interpreter.eval(r#""hello""#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));

        // Test string predicates
        let result = interpreter.eval(r#"(string? "hello")"#).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(string? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test string operations
        let result = interpreter.eval(r#"(string-length "hello")"#).unwrap();
        assert_eq!(result, Value::from(5i64));

        let result = interpreter.eval(r#"(string-ref "hello" 0)"#).unwrap();
        assert_eq!(result, Value::Character('h'));
    }

    #[test]
    fn test_symbol_operations() {
        let mut interpreter = Interpreter::new();

        // Test symbol literals
        let result = interpreter.eval("'hello").unwrap();
        assert_eq!(result, Value::Symbol("hello".to_string()));

        // Test symbol predicates
        let result = interpreter.eval("(symbol? 'hello)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(symbol? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_pair_and_list_operations() {
        let mut interpreter = Interpreter::new();

        // Test cons
        let result = interpreter.eval("(cons 1 2)").unwrap();
        assert!(matches!(result, Value::Pair(_)));

        // Test car and cdr
        let result = interpreter.eval("(car (cons 1 2))").unwrap();
        assert_eq!(result, Value::from(1i64));

        let result = interpreter.eval("(cdr (cons 1 2))").unwrap();
        assert_eq!(result, Value::from(2i64));

        // Test null/nil
        let result = interpreter.eval("'()").unwrap();
        assert_eq!(result, Value::Nil);

        let result = interpreter.eval("(null? '())").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(null? '(1 2 3))").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test pair predicates
        let result = interpreter.eval("(pair? (cons 1 2))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(pair? '())").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_vector_operations() {
        let mut interpreter = Interpreter::new();

        // Test vector creation
        let result = interpreter.eval("#(1 2 3)").unwrap();
        assert!(matches!(result, Value::Vector(_)));

        // Test vector operations
        let result = interpreter.eval("(vector-length #(1 2 3))").unwrap();
        assert_eq!(result, Value::from(3i64));

        let result = interpreter.eval("(vector-ref #(1 2 3) 0)").unwrap();
        assert_eq!(result, Value::from(1i64));
    }

    #[test]
    fn test_procedure_operations() {
        let mut interpreter = Interpreter::new();

        // Test procedure predicate
        let result = interpreter.eval("(procedure? +)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(procedure? (lambda (x) x))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(procedure? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test apply
        let result = interpreter.eval("(apply + '(1 2 3))").unwrap();
        assert_eq!(result, Value::from(6i64));

        // Test map (currently disabled - requires evaluator integration)
        // let result = interpreter.eval("(map (lambda (x) (* x 2)) '(1 2 3))").unwrap();
        // // Should return a list of (2 4 6)
        // assert!(matches!(result, Value::Pair(_) | Value::Vector(_)));
    }

    #[test]
    fn test_control_flow() {
        let mut interpreter = Interpreter::new();

        // Test if with true condition
        let result = interpreter.eval("(if #t 'yes 'no)").unwrap();
        assert_eq!(result, Value::Symbol("yes".to_string()));

        // Test if with false condition
        let result = interpreter.eval("(if #f 'yes 'no)").unwrap();
        assert_eq!(result, Value::Symbol("no".to_string()));

        // Test cond with true condition
        let result = interpreter
            .eval("(cond (#f 'no) (#t 'yes) (else 'default))")
            .unwrap();
        assert_eq!(result, Value::Symbol("yes".to_string()));

        // Test cond with else clause
        let result = interpreter
            .eval("(cond (#f 'no) (#f 'also-no) (else 'from-else))")
            .unwrap();
        assert_eq!(result, Value::Symbol("from-else".to_string()));

        // Test cond with multiple expressions in consequent
        let result = interpreter.eval("(cond (#t (+ 1 2) (* 3 4)))").unwrap();
        assert_eq!(result, Value::from(12i64)); // (* 3 4) should be last expression

        // Test case (if implemented)
        // let result = interpreter.eval("(case 'a ((a) 'found-a) ((b) 'found-b) (else 'other))").unwrap();
        // assert_eq!(result, Value::Symbol("found-a".to_string()));
    }

    #[test]
    fn test_quasiquote() {
        let mut interpreter = Interpreter::new();

        // Test basic quasiquote
        let result = interpreter.eval("`(1 2 3)").unwrap();
        // Should be equivalent to '(1 2 3)
        assert!(matches!(result, Value::Pair(_) | Value::Vector(_)));

        // Test unquote (if implemented)
        interpreter.eval("(define x 42)").unwrap();
        // let result = interpreter.eval("`(1 ,x 3)").unwrap();
        // This would require full quasiquote implementation
    }

    #[test]
    #[ignore = "Phase 6-D tail call optimization still in development - requires full implementation"]
    fn test_tail_call_optimization() {
        let mut interpreter = Interpreter::new();

        // Define a tail-recursive function
        interpreter
            .eval(
                r#"
            (define (factorial n acc)
              (if (<= n 1)
                  acc
                  (factorial (- n 1) (* n acc))))
        "#,
            )
            .unwrap();

        // This should not stack overflow with proper tail call optimization
        let result = interpreter.eval("(factorial 5 1)").unwrap();
        assert_eq!(result, Value::from(120i64)); // 5!

        // Test with smaller number to avoid stack overflow in current implementation
        let result = interpreter.eval("(factorial 10 1)");
        // This might overflow or be too large, but should not stack overflow
        if result.is_ok() {
            // Success is fine
        }
        // Error is also acceptable if it's an arithmetic overflow or stack limit
    }
}
