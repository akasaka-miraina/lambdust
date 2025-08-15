//! Comprehensive Value Optimization Benchmarks
//!
//! This benchmark suite provides detailed performance measurement for the Value enum
//! optimization implementation. It measures Arc reduction, memory savings, performance
//! improvements, and semantic equivalence to validate the optimization strategy.
//!
//! Benchmark Categories:
//! - Hot path operations (creation, cloning, access)
//! - Memory allocation patterns
//! - Arc usage optimization
//! - Cache performance and locality
//! - Semantic equivalence validation
//! - Production load testing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use lambdust::eval::value::Value;
use lambdust::eval::value_optimization_core::{ValueOptimizer, OptimizationConfig};
use lambdust::eval::comprehensive_performance_verification::{
    PerformanceVerificationSuite, VerificationConfig
};
use lambdust::eval::arc_allocation_tracker::{
    enable_global_tracking, disable_global_tracking, reset_global_tracking, get_global_stats
};
use lambdust::ast::Literal;
use lambdust::utils::SymbolId;
use std::time::{Duration, Instant};
use std::sync::Arc;

/// Benchmarks immediate value creation (hot path)
fn bench_immediate_value_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("immediate_value_creation");
    group.throughput(Throughput::Elements(1));
    
    let optimizer = ValueOptimizer::default();
    
    // Boolean values
    group.bench_function("legacy_boolean", |b| {
        b.iter(|| {
            let val = black_box(Value::boolean(true));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_boolean", |b| {
        b.iter(|| {
            let val = black_box(optimizer.boolean(true));
            std::mem::drop(val);
        })
    });
    
    // Integer values
    group.bench_function("legacy_integer", |b| {
        b.iter(|| {
            let val = black_box(Value::integer(42));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_integer", |b| {
        b.iter(|| {
            let val = black_box(optimizer.integer(42));
            std::mem::drop(val);
        })
    });
    
    // Character values
    group.bench_function("legacy_character", |b| {
        b.iter(|| {
            let val = black_box(Value::Literal(Literal::Character('A')));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_character", |b| {
        b.iter(|| {
            let val = black_box(optimizer.character('A'));
            std::mem::drop(val);
        })
    });
    
    // Nil values
    group.bench_function("legacy_nil", |b| {
        b.iter(|| {
            let val = black_box(Value::Nil);
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_nil", |b| {
        b.iter(|| {
            let val = black_box(ValueOptimizer::nil());
            std::mem::drop(val);
        })
    });
    
    group.finish();
}

/// Benchmarks compound value creation (pairs, lists)
fn bench_compound_value_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("compound_value_creation");
    group.throughput(Throughput::Elements(1));
    
    let optimizer = ValueOptimizer::default();
    
    // Simple pairs
    group.bench_function("legacy_pair", |b| {
        b.iter(|| {
            let val = black_box(Value::pair(Value::integer(1), Value::integer(2)));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_pair", |b| {
        b.iter(|| {
            let val = black_box(optimizer.pair(optimizer.integer(1), optimizer.integer(2)));
            std::mem::drop(val);
        })
    });
    
    // Short lists
    group.bench_function("legacy_short_list", |b| {
        b.iter(|| {
            let val = black_box(Value::list(vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3)
            ]));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_short_list", |b| {
        b.iter(|| {
            let val = black_box(optimizer.list(vec![
                optimizer.integer(1),
                optimizer.integer(2),
                optimizer.integer(3)
            ]));
            std::mem::drop(val);
        })
    });
    
    // String values
    group.bench_function("legacy_string", |b| {
        b.iter(|| {
            let val = black_box(Value::string("hello world"));
            std::mem::drop(val);
        })
    });
    
    group.bench_function("optimized_string", |b| {
        b.iter(|| {
            let val = black_box(optimizer.string("hello world"));
            std::mem::drop(val);
        })
    });
    
    group.finish();
}

/// Benchmarks value cloning operations
fn bench_value_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_cloning");
    group.throughput(Throughput::Elements(1));
    
    let optimizer = ValueOptimizer::default();
    
    // Immediate value cloning
    let legacy_int = Value::integer(42);
    let optimized_int = optimizer.integer(42);
    
    group.bench_function("legacy_integer_clone", |b| {
        b.iter(|| {
            let cloned = black_box(legacy_int.clone());
            std::mem::drop(cloned);
        })
    });
    
    group.bench_function("optimized_integer_clone", |b| {
        b.iter(|| {
            let cloned = black_box(optimized_int.clone());
            std::mem::drop(cloned);
        })
    });
    
    // Compound value cloning
    let legacy_pair = Value::pair(Value::integer(1), Value::integer(2));
    let optimized_pair = optimizer.pair(optimizer.integer(1), optimizer.integer(2));
    
    group.bench_function("legacy_pair_clone", |b| {
        b.iter(|| {
            let cloned = black_box(legacy_pair.clone());
            std::mem::drop(cloned);
        })
    });
    
    group.bench_function("optimized_pair_clone", |b| {
        b.iter(|| {
            let cloned = black_box(optimized_pair.clone());
            std::mem::drop(cloned);
        })
    });
    
    // List cloning
    let legacy_list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
    let optimized_list = optimizer.list(vec![optimizer.integer(1), optimizer.integer(2), optimizer.integer(3)]);
    
    group.bench_function("legacy_list_clone", |b| {
        b.iter(|| {
            let cloned = black_box(legacy_list.clone());
            std::mem::drop(cloned);
        })
    });
    
    group.bench_function("optimized_list_clone", |b| {
        b.iter(|| {
            let cloned = black_box(optimized_list.clone());
            std::mem::drop(cloned);
        })
    });
    
    group.finish();
}

/// Benchmarks value access operations
fn bench_value_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_access");
    group.throughput(Throughput::Elements(1));
    
    let optimizer = ValueOptimizer::default();
    
    // Type predicate checks
    let values = vec![
        (Value::boolean(true), optimizer.boolean(true)),
        (Value::integer(42), optimizer.integer(42)),
        (Value::Nil, ValueOptimizer::nil()),
        (Value::string("test"), optimizer.string("test")),
    ];
    
    group.bench_function("legacy_type_predicates", |b| {
        b.iter(|| {
            for (val, _) in &values {
                black_box(val.is_boolean());
                black_box(val.is_number());
                black_box(val.is_nil());
                black_box(val.is_string());
            }
        })
    });
    
    group.bench_function("optimized_type_predicates", |b| {
        b.iter(|| {
            for (_, val) in &values {
                black_box(val.is_boolean());
                black_box(val.is_number());
                black_box(val.is_nil());
                black_box(val.is_string());
            }
        })
    });
    
    // List operations
    let legacy_list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
    let optimized_list = optimizer.list(vec![optimizer.integer(1), optimizer.integer(2), optimizer.integer(3)]);
    
    group.bench_function("legacy_list_access", |b| {
        b.iter(|| {
            black_box(legacy_list.as_list());
            black_box(legacy_list.car());
            black_box(legacy_list.cdr());
        })
    });
    
    group.bench_function("optimized_list_access", |b| {
        b.iter(|| {
            black_box(optimized_list.as_list());
            black_box(optimized_list.car());
            black_box(optimized_list.cdr());
        })
    });
    
    group.finish();
}

/// Benchmarks Arc allocation patterns
fn bench_arc_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("arc_allocation_patterns");
    group.throughput(Throughput::Elements(100));
    
    enable_global_tracking();
    
    group.bench_function("legacy_allocation_pattern", |b| {
        b.iter(|| {
            reset_global_tracking();
            
            // Create a variety of values
            let mut values = Vec::new();
            for i in 0..100 {
                values.push(Value::integer(i));
                if i % 10 == 0 {
                    values.push(Value::pair(Value::integer(i), Value::integer(i + 1)));
                }
                if i % 20 == 0 {
                    values.push(Value::string(&format!("string_{}", i)));
                }
            }
            
            black_box(values);
        })
    });
    
    group.bench_function("optimized_allocation_pattern", |b| {
        b.iter(|| {
            reset_global_tracking();
            let optimizer = ValueOptimizer::default();
            
            // Create the same variety of values with optimizer
            let mut values = Vec::new();
            for i in 0..100 {
                values.push(optimizer.integer(i));
                if i % 10 == 0 {
                    values.push(optimizer.pair(optimizer.integer(i), optimizer.integer(i + 1)));
                }
                if i % 20 == 0 {
                    values.push(optimizer.string(&format!("string_{}", i)));
                }
            }
            
            black_box(values);
        })
    });
    
    disable_global_tracking();
    group.finish();
}

/// Benchmarks memory usage patterns with different sizes
fn bench_memory_usage_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_scaling");
    
    let optimizer = ValueOptimizer::default();
    
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("legacy_list", size), size, |b, &size| {
            b.iter(|| {
                let values: Vec<Value> = (0..size).map(|i| Value::integer(i as i64)).collect();
                let list = black_box(Value::list(values));
                std::mem::drop(list);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("optimized_list", size), size, |b, &size| {
            b.iter(|| {
                let values: Vec<Value> = (0..size).map(|i| optimizer.integer(i as i64)).collect();
                let list = black_box(optimizer.list(values));
                std::mem::drop(list);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("legacy_vector", size), size, |b, &size| {
            b.iter(|| {
                let values: Vec<Value> = (0..size).map(|i| Value::integer(i as i64)).collect();
                let vector = black_box(Value::vector(values));
                std::mem::drop(vector);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("optimized_vector", size), size, |b, &size| {
            b.iter(|| {
                let values: Vec<Value> = (0..size).map(|i| optimizer.integer(i as i64)).collect();
                let vector = black_box(optimizer.vector(values));
                std::mem::drop(vector);
            })
        });
    }
    
    group.finish();
}

/// Benchmarks complex nested value structures
fn bench_nested_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_structures");
    group.throughput(Throughput::Elements(1));
    
    let optimizer = ValueOptimizer::default();
    
    group.bench_function("legacy_nested_pairs", |b| {
        b.iter(|| {
            let mut current = Value::Nil;
            for i in 0..50 {
                current = Value::pair(Value::integer(i), current);
            }
            black_box(current);
        })
    });
    
    group.bench_function("optimized_nested_pairs", |b| {
        b.iter(|| {
            let mut current = ValueOptimizer::nil();
            for i in 0..50 {
                current = optimizer.pair(optimizer.integer(i), current);
            }
            black_box(current);
        })
    });
    
    group.bench_function("legacy_mixed_structure", |b| {
        b.iter(|| {
            let vector = Value::vector(vec![
                Value::integer(1),
                Value::string("test"),
                Value::pair(Value::integer(2), Value::integer(3))
            ]);
            let list = Value::list(vec![
                vector,
                Value::boolean(true),
                Value::Nil
            ]);
            black_box(list);
        })
    });
    
    group.bench_function("optimized_mixed_structure", |b| {
        b.iter(|| {
            let vector = optimizer.vector(vec![
                optimizer.integer(1),
                optimizer.string("test"),
                optimizer.pair(optimizer.integer(2), optimizer.integer(3))
            ]);
            let list = optimizer.list(vec![
                vector,
                optimizer.boolean(true),
                ValueOptimizer::nil()
            ]);
            black_box(list);
        })
    });
    
    group.finish();
}

/// Benchmarks cache efficiency and memory locality
fn bench_cache_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_efficiency");
    group.throughput(Throughput::Elements(1000));
    
    let optimizer = ValueOptimizer::default();
    
    // Create arrays of values for cache testing
    let legacy_values: Vec<Value> = (0..1000).map(|i| Value::integer(i)).collect();
    let optimized_values: Vec<Value> = (0..1000).map(|i| optimizer.integer(i)).collect();
    
    group.bench_function("legacy_sequential_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for value in &legacy_values {
                if let Some(n) = value.as_integer() {
                    sum += n;
                }
            }
            black_box(sum);
        })
    });
    
    group.bench_function("optimized_sequential_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for value in &optimized_values {
                if let Some(n) = value.as_integer() {
                    sum += n;
                }
            }
            black_box(sum);
        })
    });
    
    group.bench_function("legacy_random_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for i in (0..1000).step_by(7) { // Irregular access pattern
                if let Some(n) = legacy_values[i % 1000].as_integer() {
                    sum += n;
                }
            }
            black_box(sum);
        })
    });
    
    group.bench_function("optimized_random_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for i in (0..1000).step_by(7) { // Irregular access pattern
                if let Some(n) = optimized_values[i % 1000].as_integer() {
                    sum += n;
                }
            }
            black_box(sum);
        })
    });
    
    group.finish();
}

/// Comprehensive performance verification benchmark
fn bench_comprehensive_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_verification");
    group.measurement_time(Duration::from_secs(30)); // Allow more time for comprehensive testing
    group.sample_size(10); // Fewer samples since this is expensive
    
    group.bench_function("full_verification_suite", |b| {
        b.iter(|| {
            let config = VerificationConfig {
                benchmark_iterations: 1000, // Reduced for benchmark
                enable_memory_tracking: true,
                enable_arc_tracking: true,
                enable_semantic_verification: true,
                enable_cache_analysis: false, // Disable to reduce runtime
                enable_production_assessment: false, // Disable to reduce runtime
                test_timeout: Duration::from_secs(10),
            };
            
            let suite = PerformanceVerificationSuite::new(config);
            let report = black_box(suite.run_comprehensive_verification());
            
            // Verify we got meaningful results
            assert!(report.memory_analysis.total_values_tested > 0);
            assert!(report.arc_analysis.total_arc_usage >= 0);
        })
    });
    
    group.finish();
}

/// Benchmarks string caching effectiveness
fn bench_string_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_caching");
    group.throughput(Throughput::Elements(100));
    
    let mut config = OptimizationConfig::default();
    config.enable_caching = true;
    config.max_cache_size = 1000;
    let optimizer = ValueOptimizer::new(config);
    
    // Common strings that should benefit from caching
    let test_strings = vec![
        "hello", "world", "test", "cache", "optimization",
        "hello", "world", "test", "cache", "optimization", // Repeats for cache hits
    ];
    
    group.bench_function("cached_string_creation", |b| {
        b.iter(|| {
            for s in &test_strings {
                let val = black_box(optimizer.string(*s));
                std::mem::drop(val);
            }
        })
    });
    
    group.bench_function("uncached_string_creation", |b| {
        b.iter(|| {
            for s in &test_strings {
                let val = black_box(Value::string(*s));
                std::mem::drop(val);
            }
        })
    });
    
    group.finish();
}

/// Load testing benchmark to assess production readiness
fn bench_load_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_testing");
    group.throughput(Throughput::Elements(10000));
    group.measurement_time(Duration::from_secs(10));
    
    let optimizer = ValueOptimizer::default();
    
    group.bench_function("legacy_high_load", |b| {
        b.iter(|| {
            let mut values = Vec::with_capacity(10000);
            for i in 0..10000 {
                match i % 4 {
                    0 => values.push(Value::integer(i as i64)),
                    1 => values.push(Value::boolean(i % 2 == 0)),
                    2 => values.push(Value::string(&format!("item_{}", i))),
                    3 => values.push(Value::pair(Value::integer(i as i64), Value::Nil)),
                    _ => unreachable!(),
                }
            }
            black_box(values);
        })
    });
    
    group.bench_function("optimized_high_load", |b| {
        b.iter(|| {
            let mut values = Vec::with_capacity(10000);
            for i in 0..10000 {
                match i % 4 {
                    0 => values.push(optimizer.integer(i as i64)),
                    1 => values.push(optimizer.boolean(i % 2 == 0)),
                    2 => values.push(optimizer.string(&format!("item_{}", i))),
                    3 => values.push(optimizer.pair(optimizer.integer(i as i64), ValueOptimizer::nil())),
                    _ => unreachable!(),
                }
            }
            black_box(values);
        })
    });
    
    group.finish();
}

criterion_group!(
    value_optimization_benches,
    bench_immediate_value_creation,
    bench_compound_value_creation,
    bench_value_cloning,
    bench_value_access,
    bench_arc_allocation_patterns,
    bench_memory_usage_scaling,
    bench_nested_structures,
    bench_cache_efficiency,
    bench_string_caching,
    bench_load_testing,
    bench_comprehensive_verification
);

criterion_main!(value_optimization_benches);