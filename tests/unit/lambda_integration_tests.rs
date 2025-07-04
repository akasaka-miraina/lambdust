//! Tests for lambda function integration with higher-order functions

use lambdust::evaluator::FormalEvaluator;
use lambdust::value::Value;

fn eval_str_formal(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut evaluator = FormalEvaluator::new();
    Ok(evaluator.eval_string(input)?)
}

#[test]
fn test_map_with_lambda() {
    let result = eval_str_formal("(map (lambda (x) (* x 2)) '(1 2 3 4))").unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(2i64),
            Value::from(4i64),
            Value::from(6i64),
            Value::from(8i64)
        ])
    );
}

#[test]
fn test_apply_with_lambda() {
    let result = eval_str_formal("(apply (lambda (x y) (+ x y)) '(10 20))").unwrap();
    assert_eq!(result, Value::from(30i64));
}

#[test]
fn test_fold_with_lambda() {
    let result = eval_str_formal("(fold (lambda (acc x) (+ acc x)) 0 '(1 2 3 4))").unwrap();
    assert_eq!(result, Value::from(10i64));
}

#[test]
fn test_nested_lambda_with_map() {
    let result =
        eval_str_formal("(map (lambda (x) (+ x 1)) (map (lambda (x) (* x 2)) '(1 2 3)))").unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(3i64),
            Value::from(5i64),
            Value::from(7i64)
        ])
    );
}

#[test]
fn test_lambda_with_closure() {
    let result = eval_str_formal(
        r#"
        (define make-adder
          (lambda (n)
            (lambda (x) (+ x n))))
        (define add5 (make-adder 5))
        (map add5 '(1 2 3))
    "#,
    )
    .unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(6i64),
            Value::from(7i64),
            Value::from(8i64)
        ])
    );
}

#[test]
fn test_complex_lambda_expression() {
    let result = eval_str_formal(
        r#"
        (fold (lambda (acc x) 
                (if (> x 0) 
                    (+ acc x) 
                    acc)) 
              0 
              '(-1 2 -3 4 5))
    "#,
    )
    .unwrap();
    assert_eq!(result, Value::from(11i64)); // 2 + 4 + 5 = 11
}

#[test]
fn test_lambda_with_multiple_arguments() {
    let result = eval_str_formal(
        r#"
        (map (lambda (x y) (+ x y)) '(1 2 3) '(10 20 30))
    "#,
    )
    .unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(11i64),
            Value::from(22i64),
            Value::from(33i64)
        ])
    );
}

#[test]
fn test_fold_right_with_lambda() {
    let result =
        eval_str_formal("(fold-right (lambda (x acc) (cons x acc)) '() '(1 2 3))").unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64)
        ])
    );
}

#[test]
fn test_filter_with_lambda() {
    let result = eval_str_formal("(filter (lambda (x) (> x 2)) '(1 2 3 4 5))").unwrap();
    assert_eq!(
        result,
        Value::from_vector(vec![
            Value::from(3i64),
            Value::from(4i64),
            Value::from(5i64)
        ])
    );
}

#[test]
fn test_fold_vs_fold_right() {
    // Left fold: (- (- (- 0 1) 2) 3) = -6
    let left_result = eval_str_formal("(fold (lambda (acc x) (- acc x)) 0 '(1 2 3))").unwrap();
    assert_eq!(left_result, Value::from(-6i64));

    // Right fold: (- 1 (- 2 (- 3 0))) = 2
    let right_result =
        eval_str_formal("(fold-right (lambda (x acc) (- x acc)) 0 '(1 2 3))").unwrap();
    assert_eq!(right_result, Value::from(2i64));
}
