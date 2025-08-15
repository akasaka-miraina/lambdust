//! Simplified JIT performance benchmarks for Lambdust Scheme
//!
//! This benchmark suite provides a basic performance comparison between
//! interpreter-only execution and JIT-enabled execution.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lambdust::eval::{Evaluator, Environment};
use lambdust::ast::{Expr, Literal};
use lambdust::diagnostics::Spanned;
use std::rc::Rc;

/// Helper function to create a spanned expression
fn spanned_expr(expr: Expr) -> Spanned<Expr> {
    Spanned {
        inner: expr,
        span: lambdust::diagnostics::Span::new(0, 0),
    }
}

/// Simple arithmetic expression: (+ 1 2)
fn create_simple_arithmetic() -> Expr {
    Expr::Application {
        operator: Box::new(spanned_expr(Expr::Identifier("+".to_string()))),
        operands: vec![
            spanned_expr(Expr::Literal(Literal::ExactInteger(1))),
            spanned_expr(Expr::Literal(Literal::ExactInteger(2))),
        ],
    }
}

/// Complex arithmetic expression with nested operations
fn create_complex_arithmetic() -> Expr {
    // (+ (* 3 4) (- 10 5))
    Expr::Application {
        operator: Box::new(spanned_expr(Expr::Identifier("+".to_string()))),
        operands: vec![
            spanned_expr(Expr::Application {
                operator: Box::new(spanned_expr(Expr::Identifier("*".to_string()))),
                operands: vec![
                    spanned_expr(Expr::Literal(Literal::ExactInteger(3))),
                    spanned_expr(Expr::Literal(Literal::ExactInteger(4))),
                ],
            }),
            spanned_expr(Expr::Application {
                operator: Box::new(spanned_expr(Expr::Identifier("-".to_string()))),
                operands: vec![
                    spanned_expr(Expr::Literal(Literal::ExactInteger(10))),
                    spanned_expr(Expr::Literal(Literal::ExactInteger(5))),
                ],
            }),
        ],
    }
}

/// Benchmark simple interpreter execution
fn benchmark_interpreter_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter_simple");
    
    group.bench_function("arithmetic", |b| {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_simple_arithmetic();
        
        b.iter(|| {
            black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
        });
    });
    
    group.bench_function("complex_arithmetic", |b| {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_complex_arithmetic();
        
        b.iter(|| {
            black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
        });
    });
    
    group.finish();
}

/// Benchmark JIT-enabled execution
fn benchmark_jit_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_simple");
    
    group.bench_function("arithmetic", |b| {
        let mut evaluator = Evaluator::new();
        evaluator.enable_jit().unwrap_or(()); // Ignore errors if JIT not available
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_simple_arithmetic();
        
        b.iter(|| {
            black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
        });
    });
    
    group.bench_function("complex_arithmetic", |b| {
        let mut evaluator = Evaluator::new();
        evaluator.enable_jit().unwrap_or(()); // Ignore errors if JIT not available
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_complex_arithmetic();
        
        b.iter(|| {
            black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
        });
    });
    
    group.finish();
}

/// Benchmark repeated execution to test cache effectiveness
fn benchmark_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");
    
    group.bench_function("interpreter_repeated", |b| {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_simple_arithmetic();
        
        b.iter(|| {
            for _ in 0..10 {
                black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
            }
        });
    });
    
    group.bench_function("jit_repeated", |b| {
        let mut evaluator = Evaluator::new();
        evaluator.enable_jit().unwrap_or(()); // Ignore errors if JIT not available
        let env = Rc::new(Environment::new(None, 0));
        let expr = create_simple_arithmetic();
        
        b.iter(|| {
            for _ in 0..10 {
                black_box(evaluator.eval(&spanned_expr(expr.clone()), env.clone()).unwrap());
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_interpreter_simple,
    benchmark_jit_simple,
    benchmark_cache_effectiveness
);

criterion_main!(benches);