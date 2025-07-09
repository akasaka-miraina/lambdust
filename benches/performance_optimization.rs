use criterion::{criterion_group, criterion_main, Criterion};
use lambdust::Interpreter;

fn fibonacci_recursive(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let fibonacci_code = r#"
    (define fib
      (lambda (n)
        (if (<= n 1)
            n
            (+ (fib (- n 1)) (fib (- n 2))))))
    "#;

    interpreter.eval(fibonacci_code).unwrap();

    c.bench_function("fibonacci_recursive_8", |b| {
        b.iter(|| interpreter.eval("(fib 8)").unwrap())
    });
}

fn complex_list_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let setup_code = r#"
    (define test-list (list 1 2 3 4 5 6 7 8 9 10))
    (define (square x) (* x x))
    (define (sum-list lst)
      (if (null? lst)
          0
          (+ (car lst) (sum-list (cdr lst)))))
    "#;

    interpreter.eval(setup_code).unwrap();

    c.bench_function("complex_list_operations", |b| {
        b.iter(|| {
            interpreter
                .eval("(sum-list (map square test-list))")
                .unwrap()
        })
    });
}

fn nested_function_calls(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let nested_code = r#"
    (define (f x) (+ x 1))
    (define (g x) (* x 2))
    (define (h x) (- x 3))
    (define (nested-call x)
      (f (g (h (f (g (h x)))))))
    "#;

    interpreter.eval(nested_code).unwrap();

    c.bench_function("nested_function_calls", |b| {
        b.iter(|| interpreter.eval("(nested-call 42)").unwrap())
    });
}

fn vector_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let vector_code = r#"
    (define test-vector (vector 1 2 3 4 5 6 7 8 9 10))
    (define (sum-vector-elements vec)
      (define len (vector-length vec))
      (define (sum-loop i acc)
        (if (>= i len)
            acc
            (sum-loop (+ i 1) (+ acc (vector-ref vec i)))))
      (sum-loop 0 0))
    "#;

    interpreter.eval(vector_code).unwrap();

    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            interpreter
                .eval("(sum-vector-elements test-vector)")
                .unwrap()
        })
    });
}

fn continuation_heavy_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let continuation_code = r#"
    (define (factorial n)
      (if (<= n 1)
          1
          (* n (factorial (- n 1)))))
    
    (define (deep-recursion n)
      (if (<= n 0)
          0
          (+ 1 (deep-recursion (- n 1)))))
    "#;

    interpreter.eval(continuation_code).unwrap();

    c.bench_function("continuation_heavy_factorial", |b| {
        b.iter(|| interpreter.eval("(factorial 8)").unwrap())
    });

    c.bench_function("continuation_heavy_deep_recursion", |b| {
        b.iter(|| interpreter.eval("(deep-recursion 30)").unwrap())
    });
}

fn expression_evaluation_patterns(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    // Test split_first optimization patterns
    c.bench_function("multiple_argument_function", |b| {
        b.iter(|| interpreter.eval("(+ 1 2 3 4 5 6 7 8 9 10)").unwrap())
    });

    c.bench_function("nested_begin_expressions", |b| {
        b.iter(|| {
            interpreter
                .eval("(begin (+ 1 2) (+ 3 4) (+ 5 6) (+ 7 8) (+ 9 10))")
                .unwrap()
        })
    });

    c.bench_function("complex_expression_evaluation", |b| {
        b.iter(|| {
            interpreter
                .eval("(+ (* 2 3) (- 10 5) (/ 20 4) (quotient 15 3))")
                .unwrap()
        })
    });
}

criterion_group!(
    benches,
    fibonacci_recursive,
    complex_list_operations,
    nested_function_calls,
    vector_operations,
    continuation_heavy_operations,
    expression_evaluation_patterns
);

criterion_main!(benches);
