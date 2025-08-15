//! Memory Optimization Baseline Benchmarks
//!
//! This benchmark suite establishes performance and memory usage baselines
//! before applying memory layout optimizations to the Lambdust codebase.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lambdust::eval::{Value, Environment, ThreadSafeEnvironment, Generation};
use lambdust::ast::{Literal, Expr, Formals};
use lambdust::diagnostics::Spanned;
use lambdust::utils::SymbolId;
use std::sync::Arc;
use std::collections::HashMap;
use std::mem;

/// Benchmark memory layout and allocation patterns for Value enum
fn bench_value_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_creation");
    
    // Benchmark different Value variants creation
    group.bench_function("literal_number", |b| {
        b.iter(|| {
            let val = black_box(Value::number(42.0));
            mem::forget(val);
        })
    });
    
    group.bench_function("literal_string", |b| {
        b.iter(|| {
            let val = black_box(Value::string("hello world"));
            mem::forget(val);
        })
    });
    
    group.bench_function("symbol", |b| {
        b.iter(|| {
            let val = black_box(Value::symbol(SymbolId::new(123)));
            mem::forget(val);
        })
    });
    
    group.bench_function("pair", |b| {
        b.iter(|| {
            let val = black_box(Value::pair(Value::number(1.0), Value::number(2.0)));
            mem::forget(val);
        })
    });
    
    group.bench_function("vector_small", |b| {
        b.iter(|| {
            let val = black_box(Value::vector(vec![
                Value::number(1.0),
                Value::number(2.0),
                Value::number(3.0),
            ]));
            mem::forget(val);
        })
    });
    
    group.bench_function("vector_large", |b| {
        b.iter(|| {
            let values: Vec<Value> = (0..100).map(|i| Value::number(i as f64)).collect();
            let val = black_box(Value::vector(values));
            mem::forget(val);
        })
    });
    
    group.bench_function("procedure", |b| {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        b.iter(|| {
            let proc = lambdust::eval::value::Procedure {
                formals: Formals::Fixed(vec!["x".to_string()]),
                body: vec![Spanned::new(
                    Expr::Literal(Literal::ExactInteger(42)),
                    lambdust::diagnostics::Span::new(0, 2)
                )],
                environment: env.clone(),
                name: Some("test-proc".to_string()),
                metadata: HashMap::new(),
                source: None,
            };
            let val = black_box(Value::procedure(proc));
            mem::forget(val);
        })
    });
    
    group.finish();
}

/// Benchmark Value enum size and alignment
fn bench_value_size_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_size_analysis");
    
    // Report memory layout information
    println!("\n=== Value Memory Layout Analysis ===");
    println!("Value size: {} bytes", mem::size_of::<Value>());
    println!("Value alignment: {} bytes", mem::align_of::<Value>());
    
    // Analyze different variant sizes
    let test_values = vec![
        ("Literal", Value::number(42.0)),
        ("Symbol", Value::symbol(SymbolId::new(123))),
        ("Nil", Value::Nil),
        ("Pair", Value::pair(Value::number(1.0), Value::number(2.0))),
        ("Vector", Value::vector(vec![Value::number(1.0)])),
        ("String", Value::string("test")),
    ];
    
    for (name, value) in &test_values {
        println!("{}: discriminant = {:?}", name, mem::discriminant(value));
    }
    
    // Benchmark discriminant matching (enum dispatch cost)
    group.bench_function("discriminant_matching", |b| {
        let values = vec![
            Value::number(42.0),
            Value::string("hello"),
            Value::symbol(SymbolId::new(123)),
            Value::Nil,
            Value::pair(Value::number(1.0), Value::number(2.0)),
        ];
        
        b.iter(|| {
            let mut sum = 0;
            for val in &values {
                sum += match val {
                    Value::Literal(_) => 1,
                    Value::Symbol(_) => 2,
                    Value::Nil => 3,
                    Value::Pair(_, _) => 4,
                    _ => 5,
                };
            }
            black_box(sum)
        })
    });
    
    group.finish();
}

/// Benchmark memory-intensive operations
fn bench_memory_intensive_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_intensive_ops");
    
    // Test list construction (heavy on pairs)
    group.bench_function("list_construction_small", |b| {
        b.iter(|| {
            let values: Vec<Value> = (0..10).map(|i| Value::number(i as f64)).collect();
            let list = black_box(Value::list(values));
            mem::forget(list);
        })
    });
    
    group.bench_function("list_construction_large", |b| {
        b.iter(|| {
            let values: Vec<Value> = (0..1000).map(|i| Value::number(i as f64)).collect();
            let list = black_box(Value::list(values));
            mem::forget(list);
        })
    });
    
    // Test environment operations
    group.bench_function("environment_lookup", |b| {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        for i in 0..100 {
            env.define(format!("var{}", i), Value::number(i as f64));
        }
        
        b.iter(|| {
            let mut sum = 0.0;
            for i in 0..100 {
                if let Some(val) = env.lookup(&format!("var{}", i)) {
                    if let Some(n) = val.as_number() {
                        sum += n;
                    }
                }
            }
            black_box(sum)
        })
    });
    
    // Test deep nesting scenarios
    group.bench_function("deep_nesting", |b| {
        b.iter(|| {
            let mut current = Value::Nil;
            for i in 0..100 {
                current = Value::pair(Value::number(i as f64), current);
            }
            black_box(current)
        })
    });
    
    group.finish();
}

/// Benchmark Value clone operations (very common in functional programming)
fn bench_value_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_cloning");
    
    let test_values = vec![
        ("number", Value::number(42.0)),
        ("string", Value::string("hello world")),
        ("symbol", Value::symbol(SymbolId::new(123))),
        ("pair", Value::pair(Value::number(1.0), Value::number(2.0))),
        ("vector", Value::vector((0..10).map(|i| Value::number(i as f64)).collect())),
        ("large_vector", Value::vector((0..1000).map(|i| Value::number(i as f64)).collect())),
    ];
    
    for (name, value) in test_values {
        group.bench_with_input(BenchmarkId::new("clone", name), &value, |b, val| {
            b.iter(|| {
                let cloned = black_box(val.clone());
                mem::forget(cloned);
            })
        });
    }
    
    group.finish();
}

/// Benchmark pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");
    
    let mixed_values: Vec<Value> = vec![
        Value::number(42.0),
        Value::string("hello"),
        Value::symbol(SymbolId::new(123)),
        Value::Nil,
        Value::pair(Value::number(1.0), Value::number(2.0)),
        Value::vector(vec![Value::number(1.0), Value::number(2.0)]),
        Value::boolean(true),
        Value::boolean(false),
    ];
    
    group.bench_function("is_methods", |b| {
        b.iter(|| {
            let mut counts = [0; 8];
            for val in &mixed_values {
                if val.is_number() { counts[0] += 1; }
                if val.is_string() { counts[1] += 1; }
                if val.is_symbol() { counts[2] += 1; }
                if val.is_nil() { counts[3] += 1; }
                if val.is_pair() { counts[4] += 1; }
                if val.is_vector() { counts[5] += 1; }
                if val.is_procedure() { counts[6] += 1; }
                if val.is_list() { counts[7] += 1; }
            }
            black_box(counts)
        })
    });
    
    group.bench_function("match_expr", |b| {
        b.iter(|| {
            let mut sum = 0;
            for val in &mixed_values {
                sum += match val {
                    Value::Literal(Literal::ExactInteger(n)) => *n as usize,
                    Value::Literal(Literal::InexactReal(n)) => *n as usize,
                    Value::Literal(Literal::String(s)) => s.len(),
                    Value::Symbol(_) => 1,
                    Value::Nil => 0,
                    Value::Pair(_, _) => 2,
                    Value::Vector(v) => {
                        if let Ok(vec) = v.read() {
                            vec.len()
                        } else {
                            0
                        }
                    },
                    _ => 99,
                };
            }
            black_box(sum)
        })
    });
    
    group.finish();
}

/// Benchmark equality operations
fn bench_equality_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality_ops");
    
    let val1 = Value::number(42.0);
    let val2 = Value::number(42.0);
    let val3 = Value::string("hello");
    let val4 = Value::string("hello");
    let pair1 = Value::pair(Value::number(1.0), Value::number(2.0));
    let pair2 = Value::pair(Value::number(1.0), Value::number(2.0));
    
    group.bench_function("number_equality", |b| {
        b.iter(|| {
            black_box(val1 == val2)
        })
    });
    
    group.bench_function("string_equality", |b| {
        b.iter(|| {
            black_box(val3 == val4)
        })
    });
    
    group.bench_function("pair_equality", |b| {
        b.iter(|| {
            black_box(pair1 == pair2)
        })
    });
    
    group.bench_function("mixed_inequality", |b| {
        b.iter(|| {
            black_box(val1 != val3)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_value_creation,
    bench_value_size_analysis,
    bench_memory_intensive_ops,
    bench_value_cloning,
    bench_pattern_matching,
    bench_equality_ops
);

criterion_main!(benches);