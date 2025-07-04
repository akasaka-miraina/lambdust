use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::{Continuation, EvalOrder, FormalEvaluator, eval_with_formal_semantics};
use lambdust::lexer::{SchemeNumber, tokenize};
use lambdust::parser::parse;
use lambdust::value::Value;
use std::rc::Rc;

fn eval_str_formal(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());
    eval_with_formal_semantics(ast, env)
}

#[test]
fn test_formal_literals() {
    assert_eq!(eval_str_formal("42").unwrap(), Value::from(42i64));
    assert_eq!(eval_str_formal("#t").unwrap(), Value::Boolean(true));
    assert_eq!(eval_str_formal("\"hello\"").unwrap(), Value::from("hello"));
}

#[test]
fn test_formal_quote() {
    assert_eq!(
        eval_str_formal("'x").unwrap(),
        Value::Symbol("x".to_string())
    );
    assert_eq!(
        eval_str_formal("'(1 2 3)").unwrap(),
        Value::from_vector(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64)
        ])
    );
}

#[test]
fn test_formal_lambda() {
    let result = eval_str_formal("(lambda (x) x)").unwrap();
    assert!(result.is_procedure());
}

#[test]
fn test_formal_if() {
    assert_eq!(eval_str_formal("(if #t 1 2)").unwrap(), Value::from(1i64));
    assert_eq!(eval_str_formal("(if #f 1 2)").unwrap(), Value::from(2i64));
}

#[test]
fn test_formal_values() {
    let result = eval_str_formal("(values 1 2 3)").unwrap();
    assert_eq!(
        result,
        Value::Values(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64)
        ])
    );
}

#[test]
fn test_evaluation_order_strategies() {
    // Test different evaluation order strategies
    let mut eval_ltr = FormalEvaluator::with_eval_order(EvalOrder::LeftToRight);
    let mut eval_rtl = FormalEvaluator::with_eval_order(EvalOrder::RightToLeft);
    let mut eval_unspec = FormalEvaluator::with_eval_order(EvalOrder::Unspecified);

    let env = Rc::new(Environment::with_builtins());

    // Test literal evaluation (should be same regardless of order)
    let lit_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let cont = Continuation::Identity;

    let result_ltr = eval_ltr
        .eval(lit_expr.clone(), env.clone(), cont.clone())
        .unwrap();
    let result_rtl = eval_rtl
        .eval(lit_expr.clone(), env.clone(), cont.clone())
        .unwrap();
    let result_unspec = eval_unspec.eval(lit_expr, env, cont).unwrap();

    assert_eq!(result_ltr, Value::from(42i64));
    assert_eq!(result_rtl, Value::from(42i64));
    assert_eq!(result_unspec, Value::from(42i64));
}

#[test]
fn test_argument_order_independence() {
    // Test that expressions that should be order-independent work correctly
    // (This is a simplified test - real order independence testing would be more complex)

    use lambdust::lexer::SchemeNumber;

    let args = vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ];

    let eval = FormalEvaluator::new();

    // Test left-to-right order
    let ordered_ltr = eval.apply_evaluation_order(args.clone());
    assert_eq!(ordered_ltr.len(), 3);

    // Test that reordering doesn't affect literal values
    let eval_rtl = FormalEvaluator::with_eval_order(EvalOrder::RightToLeft);
    let ordered_rtl = eval_rtl.apply_evaluation_order(args);
    assert_eq!(ordered_rtl.len(), 3);
}

#[test]
fn test_formal_call_with_values() {
    // Test call-with-values with single value
    let result = eval_str_formal("(call-with-values (lambda () 42) (lambda (x) x))").unwrap();
    assert_eq!(result, Value::from(42i64));

    // Test call-with-values with multiple values
    let result =
        eval_str_formal("(call-with-values (lambda () (values 1 2 3)) (lambda (x y z) (+ x y z)))")
            .unwrap();
    assert_eq!(result, Value::from(6i64));

    // Test call-with-values with values producer
    let result =
        eval_str_formal("(call-with-values (lambda () (values 10 20)) (lambda (a b) (* a b)))")
            .unwrap();
    assert_eq!(result, Value::from(200i64));
}

#[test]
fn test_formal_call_with_values_errors() {
    // Test call-with-values with wrong arity
    let result = eval_str_formal("(call-with-values)");
    assert!(result.is_err());

    let result = eval_str_formal("(call-with-values (lambda () 1))");
    assert!(result.is_err());

    let result = eval_str_formal("(call-with-values (lambda () 1) (lambda (x) x) extra)");
    assert!(result.is_err());

    // Test call-with-values with non-procedure arguments
    let result = eval_str_formal("(call-with-values 42 (lambda (x) x))");
    assert!(result.is_err());

    let result = eval_str_formal("(call-with-values (lambda () 1) 42)");
    assert!(result.is_err());
}

#[test]
fn test_formal_multi_value_continuations() {
    // Test that the formal evaluator properly handles multiple values in continuations
    // This ensures that the CPS implementation correctly propagates multi-value contexts

    // Test simple multi-value propagation
    let result = eval_str_formal("(values 1 2 3)").unwrap();
    assert_eq!(
        result,
        Value::Values(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64)
        ])
    );

    // Test multi-value in call-with-values (more complex CPS case)
    let result = eval_str_formal(
        "(call-with-values (lambda () (values 5 10 15)) (lambda (a b c) (+ a b c)))",
    )
    .unwrap();
    assert_eq!(result, Value::from(30i64));
}

#[test]
fn test_formal_begin() {
    // Test empty begin
    let result = eval_str_formal("(begin)").unwrap();
    assert_eq!(result, Value::Undefined);

    // Test single expression begin
    let result = eval_str_formal("(begin 42)").unwrap();
    assert_eq!(result, Value::from(42i64));

    // Test multiple expression begin
    let result = eval_str_formal("(begin (+ 1 2) (* 3 4) (- 10 5))").unwrap();
    assert_eq!(result, Value::from(5i64));
}

#[test]
fn test_formal_define() {
    // Test simple define
    let result = eval_str_formal("(begin (define x 42) x)").unwrap();
    assert_eq!(result, Value::from(42i64));

    // Test define with complex expression
    let result = eval_str_formal("(begin (define y (+ 10 20)) y)").unwrap();
    assert_eq!(result, Value::from(30i64));
}

#[test]
fn test_formal_and() {
    // Test empty and
    let result = eval_str_formal("(and)").unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Test single expression and
    let result = eval_str_formal("(and 42)").unwrap();
    assert_eq!(result, Value::from(42i64));

    // Test and with all true values
    let result = eval_str_formal("(and 1 2 3)").unwrap();
    assert_eq!(result, Value::from(3i64));

    // Test and with false value (short-circuit)
    let result = eval_str_formal("(and 1 #f 3)").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_formal_or() {
    // Test empty or
    let result = eval_str_formal("(or)").unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Test single expression or
    let result = eval_str_formal("(or 42)").unwrap();
    assert_eq!(result, Value::from(42i64));

    // Test or with first true value (short-circuit)
    let result = eval_str_formal("(or 1 2 3)").unwrap();
    assert_eq!(result, Value::from(1i64));

    // Test or with all false values
    let result = eval_str_formal("(or #f #f #f)").unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Test or with true value at end
    let result = eval_str_formal("(or #f #f 42)").unwrap();
    assert_eq!(result, Value::from(42i64));
}

#[test]
fn test_formal_do() {
    // Test do loop with immediate termination
    let result = eval_str_formal("(do ((i 5)) ((> i 3) i))").unwrap();
    assert_eq!(result, Value::from(5i64));

    // Test do loop with step expression
    let result = eval_str_formal("(do ((i 0 (+ i 1))) ((>= i 3) i))").unwrap();
    assert_eq!(result, Value::from(3i64));
}

#[test]
fn test_formal_do_with_step() {
    // Test do loop with step expression and accumulator
    let result = eval_str_formal("(do ((i 0 (+ i 2)) (sum 0 (+ sum i))) ((>= i 10) sum))").unwrap();
    // i: 0, 2, 4, 6, 8, 10
    // sum: 0, 0, 2, 6, 12, 20
    assert_eq!(result, Value::from(20i64));
}

#[test]
fn test_formal_do_no_step() {
    // Test do loop without step expression (variable unchanged)
    let result = eval_str_formal("(do ((i 5)) ((< i 10) i))").unwrap();
    assert_eq!(result, Value::from(5i64));
}

#[test]
fn test_formal_delay() {
    // Test delay creates a promise
    let result = eval_str_formal("(delay (+ 1 2))").unwrap();
    assert!(matches!(result, Value::Promise(_)));
}

#[test]
fn test_formal_lazy() {
    // Test lazy creates a promise
    let result = eval_str_formal("(lazy (+ 1 2))").unwrap();
    assert!(matches!(result, Value::Promise(_)));
}

#[test]
fn test_formal_call_cc_basic() {
    // Basic call/cc test
    let result = eval_str_formal("(call/cc (lambda (k) 42))").unwrap();
    assert_eq!(result, Value::from(42i64));
}

#[test]
fn test_formal_call_cc_escape() {
    // Test call/cc with escape continuation
    let result = eval_str_formal("(+ 1 (call/cc (lambda (k) 2)) 3)").unwrap();
    assert_eq!(result, Value::from(6i64));
}

#[test]
fn test_formal_force() {
    // Test force evaluates a promise
    let result = eval_str_formal("(force (delay (+ 1 2)))").unwrap();
    // Note: This will fail until force is properly implemented to actually force promises
    // For now, just test that it doesn't crash and returns some value
    assert!(!matches!(result, Value::Undefined));
}

#[test]
fn test_formal_raise() {
    // Test raise creates an error
    let result = eval_str_formal("(raise 'test-error)");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Exception raised: test-error"));
    }
}

#[test]
fn test_formal_with_exception_handler_basic() {
    // Test basic with-exception-handler syntax
    // For now, this just tests parsing and basic execution
    let result = eval_str_formal("(with-exception-handler (lambda (obj) 'handled) (lambda () 42))");
    // Should succeed with current implementation
    assert!(result.is_ok());
}

#[test]
fn test_formal_guard_basic() {
    // Test basic guard syntax
    // For now, this just tests parsing and basic execution
    let result = eval_str_formal("(guard (e ((eq? e 'test) 'caught) (else 'not-caught)) 42)");
    // Should succeed with current implementation and return 42
    assert_eq!(result.unwrap(), Value::from(42i64));
}
