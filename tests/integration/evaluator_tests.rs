//! Evaluator-specific integration tests
//!
//! Tests focusing on the formal evaluator implementation and R7RS compliance.

use lambdust::{Interpreter, Value};

#[cfg(test)]
mod formal_evaluator_tests {
    use super::*;

    #[test]
    fn test_special_forms() {
        let mut interpreter = Interpreter::new();

        // Test begin
        let result = interpreter
            .eval("(begin (define x 1) (define y 2) (+ x y))")
            .unwrap();
        assert_eq!(result, Value::from(3i64));

        // Test and
        let result = interpreter.eval("(and #t #t #t)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(and #t #f #t)").unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test or
        let result = interpreter.eval("(or #f #f #t)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(or #f #f #f)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_do_loops() {
        let mut interpreter = Interpreter::new();

        // Simple counting loop
        let result = interpreter
            .eval("(do ((i 0 (+ i 1))) ((>= i 5) i))")
            .unwrap();
        assert_eq!(result, Value::from(5i64));

        // Loop with accumulator
        let result = interpreter
            .eval("(do ((i 0 (+ i 1)) (sum 0 (+ sum i))) ((>= i 5) sum))")
            .unwrap();
        assert_eq!(result, Value::from(10i64)); // 0+1+2+3+4 = 10

        // Loop without step (variable unchanged)
        let result = interpreter.eval("(do ((x 42)) ((> x 40) x))").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_lazy_evaluation() {
        let mut interpreter = Interpreter::new();

        // Test delay creates a promise
        let result = interpreter.eval("(delay (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));

        // Test lazy creates a promise
        let result = interpreter.eval("(lazy (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));

        // Test promise predicate
        let result = interpreter.eval("(promise? (delay 42))").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(promise? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_call_with_current_continuation() {
        let mut interpreter = Interpreter::new();

        // Basic call/cc test
        let result = interpreter.eval("(call/cc (lambda (k) 42))").unwrap();
        assert_eq!(result, Value::from(42i64));

        // call-with-current-continuation alias
        let result = interpreter
            .eval("(call-with-current-continuation (lambda (k) 100))")
            .unwrap();
        assert_eq!(result, Value::from(100i64));

        // call/cc in arithmetic context
        let result = interpreter
            .eval("(+ 1 (call/cc (lambda (k) 2)) 3)")
            .unwrap();
        assert_eq!(result, Value::from(6i64));
        
        // Test escape continuation - the key test for true call/cc behavior
        let result = interpreter.eval("(+ 1 (call/cc (lambda (k) (k 10) 2)) 3)").unwrap();
        assert_eq!(result, Value::from(14i64)); // Should be 1 + 10 + 3, not 1 + 2 + 3
        
        // Test continuation escape from nested computation
        let result = interpreter.eval(r#"
            (call/cc (lambda (escape)
                (+ 1 (* 2 (escape 42)) 3)))
        "#).unwrap();
        assert_eq!(result, Value::from(42i64)); // Should escape with 42, not compute the arithmetic
        
        // Test that continuation can be called with any value
        let result = interpreter.eval(r#"
            (call/cc (lambda (k) (k "hello")))
        "#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_multi_value_system() {
        let mut interpreter = Interpreter::new();

        // Test values expression
        let result = interpreter.eval("(values 1 2 3)").unwrap();
        assert!(matches!(result, Value::Values(_)));

        // Test call-with-values basic
        let result = interpreter
            .eval("(call-with-values (lambda () (values 1 2)) +)")
            .unwrap();
        assert_eq!(result, Value::from(3i64));
    }

    #[test]
    fn test_evaluation_order_independence() {
        let mut interpreter = Interpreter::new();

        // Test that argument evaluation order doesn't affect result
        // (though the specific order is unspecified per R7RS)
        let result = interpreter.eval("(+ 1 2 3 4)").unwrap();
        assert_eq!(result, Value::from(10i64));

        // Function calls with side effects
        interpreter.eval("(define counter 0)").unwrap();
        interpreter
            .eval("(define (inc!) (set! counter (+ counter 1)) counter)")
            .unwrap();

        // Multiple function calls (order unspecified but should work)
        let _result = interpreter.eval("(list (inc!) (inc!) (inc!))").unwrap();
        let counter_value = interpreter.eval("counter").unwrap();
        assert_eq!(counter_value, Value::from(3i64));
    }
}
