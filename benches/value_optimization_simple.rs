//! Simple Value Optimization Benchmarks
//!
//! Minimal benchmark to test Value enum performance before optimization.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lambdust::ast::Literal;
use lambdust::eval::Value;
use std::mem;

/// Benchmark basic Value operations
fn bench_value_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_basic");

    // Test literal creation
    group.bench_function("create_number", |b| {
        b.iter(|| {
            let val = black_box(Value::number(42.0));
            mem::drop(val);
        })
    });

    group.bench_function("create_boolean", |b| {
        b.iter(|| {
            let val = black_box(Value::boolean(true));
            mem::drop(val);
        })
    });

    group.bench_function("create_string", |b| {
        b.iter(|| {
            let val = black_box(Value::string("test"));
            mem::drop(val);
        })
    });

    group.bench_function("create_nil", |b| {
        b.iter(|| {
            let val = black_box(Value::nil());
            mem::drop(val);
        })
    });

    group.finish();
}

/// Benchmark Value memory size
fn bench_value_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_size");

    group.bench_function("size_check", |b| {
        b.iter(|| {
            let size = black_box(std::mem::size_of::<Value>());
            black_box(size);
        })
    });

    group.finish();
}

/// Benchmark Value clone operations
fn bench_value_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_clone");

    let number_val = Value::number(42.0);
    let string_val = Value::string("hello world");
    let boolean_val = Value::boolean(true);

    group.bench_function("clone_number", |b| {
        b.iter(|| {
            let cloned = black_box(number_val.clone());
            mem::drop(cloned);
        })
    });

    group.bench_function("clone_string", |b| {
        b.iter(|| {
            let cloned = black_box(string_val.clone());
            mem::drop(cloned);
        })
    });

    group.bench_function("clone_boolean", |b| {
        b.iter(|| {
            let cloned = black_box(boolean_val.clone());
            mem::drop(cloned);
        })
    });

    group.finish();
}

criterion_group!(
    value_benches,
    bench_value_basic,
    bench_value_size,
    bench_value_clone
);
criterion_main!(value_benches);
