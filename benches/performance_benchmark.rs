//! Performance benchmarks for lambdust CPS evaluator optimization
//! 
//! Tests various scenarios to measure the impact of tail call optimization 
//! and continuation inlining improvements.

use criterion::{criterion_group, criterion_main, Criterion};
use lambdust::Interpreter;

fn bench_simple_arithmetic(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    c.bench_function("simple_arithmetic", |b| {
        b.iter(|| {
            interpreter.eval("(+ 1 2 3)")
        })
    });
}

fn bench_nested_arithmetic(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    c.bench_function("nested_arithmetic", |b| {
        b.iter(|| {
            interpreter.eval("(+ (* 2 3) (- 10 5) (/ 8 2))")
        })
    });
}

fn bench_factorial_recursive(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    // Define factorial function
    interpreter.eval("(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))").unwrap();
    
    c.bench_function("factorial_5", |b| {
        b.iter(|| {
            interpreter.eval("(factorial 5)")
        })
    });
    
    c.bench_function("factorial_10", |b| {
        b.iter(|| {
            interpreter.eval("(factorial 10)")
        })
    });
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    // Define fibonacci function
    interpreter.eval("(define (fib n) (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2)))))").unwrap();
    
    c.bench_function("fibonacci_8", |b| {
        b.iter(|| {
            interpreter.eval("(fib 8)")
        })
    });
    
    c.bench_function("fibonacci_10", |b| {
        b.iter(|| {
            interpreter.eval("(fib 10)")
        })
    });
}

fn bench_list_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    c.bench_function("list_creation", |b| {
        b.iter(|| {
            interpreter.eval("(list 1 2 3 4 5 6 7 8 9 10)")
        })
    });
    
    c.bench_function("list_append", |b| {
        b.iter(|| {
            interpreter.eval("(append '(1 2 3) '(4 5 6) '(7 8 9))")
        })
    });
    
    c.bench_function("list_length", |b| {
        b.iter(|| {
            interpreter.eval("(length '(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15))")
        })
    });
}

fn bench_tail_call_optimization(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    // Define tail-recursive function
    interpreter.eval(r#"
        (define (sum-tail n acc)
          (if (<= n 0)
              acc
              (sum-tail (- n 1) (+ acc n))))
    "#).unwrap();
    
    // Define non-tail-recursive function  
    interpreter.eval(r#"
        (define (sum-normal n)
          (if (<= n 0)
              0
              (+ n (sum-normal (- n 1)))))
    "#).unwrap();
    
    c.bench_function("tail_recursive_sum_100", |b| {
        b.iter(|| {
            interpreter.eval("(sum-tail 100 0)")
        })
    });
    
    c.bench_function("normal_recursive_sum_100", |b| {
        b.iter(|| {
            interpreter.eval("(sum-normal 100)")
        })
    });
}

fn bench_continuation_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    c.bench_function("call_cc_simple", |b| {
        b.iter(|| {
            interpreter.eval("(call/cc (lambda (k) 42))")
        })
    });
    
    c.bench_function("call_cc_arithmetic", |b| {
        b.iter(|| {
            interpreter.eval("(+ 1 (call/cc (lambda (k) 2)) 3)")
        })
    });
    
    c.bench_function("values_operation", |b| {
        b.iter(|| {
            interpreter.eval("(values 1 2 3 4 5)")
        })
    });
}

fn bench_builtin_functions(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    c.bench_function("builtin_plus", |b| {
        b.iter(|| {
            interpreter.eval("(+ 1 2 3 4 5 6 7 8 9 10)")
        })
    });
    
    c.bench_function("builtin_multiply", |b| {
        b.iter(|| {
            interpreter.eval("(* 2 3 4 5)")
        })
    });
    
    c.bench_function("builtin_comparison", |b| {
        b.iter(|| {
            interpreter.eval("(< 1 2 3 4 5)")
        })
    });
}

fn bench_environment_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();
    
    // Setup some variables
    interpreter.eval("(define x 42)").unwrap();
    interpreter.eval("(define y 24)").unwrap();
    interpreter.eval("(define z 12)").unwrap();
    
    c.bench_function("variable_access", |b| {
        b.iter(|| {
            interpreter.eval("(+ x y z)")
        })
    });
    
    c.bench_function("lambda_creation", |b| {
        b.iter(|| {
            interpreter.eval("(lambda (a b c) (+ a b c))")
        })
    });
    
    c.bench_function("lambda_invocation", |b| {
        b.iter(|| {
            interpreter.eval("((lambda (a b c) (+ a b c)) 1 2 3)")
        })
    });
}

criterion_group!(
    benches,
    bench_simple_arithmetic,
    bench_nested_arithmetic,
    bench_factorial_recursive,
    bench_fibonacci,
    bench_list_operations,
    bench_tail_call_optimization,
    bench_continuation_operations,
    bench_builtin_functions,
    bench_environment_operations
);

criterion_main!(benches);