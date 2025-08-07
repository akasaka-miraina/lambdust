//! Migration Impact Benchmarks
//!
//! Comprehensive benchmarks measuring the performance impact of the Rust-to-Scheme migration.
//! These benchmarks compare:
//! - Original Rust implementations vs migrated Scheme implementations
//! - Bootstrap modes (Minimal, Hybrid, Fallback)
//! - Memory usage patterns between Rust and Scheme code
//! - Hot path performance to ensure acceptable degradation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust, Lambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime as TokioRuntime;

/// Configuration for migration impact benchmarks
#[derive(Debug, Clone)]
pub struct MigrationBenchmarkConfig {
    pub bootstrap_modes: Vec<BootstrapMode>,
    pub operation_types: Vec<MigrationOperationType>,
    pub sample_sizes: Vec<usize>,
    pub complexity_levels: Vec<ComplexityLevel>,
}

#[derive(Debug, Clone, Copy)]
pub enum MigrationOperationType {
    // Arithmetic operations (migrated to Scheme)
    ArithmeticBasic,
    ArithmeticComplex,
    ArithmeticGCD,
    
    // List operations (migrated to Scheme)
    ListConstruction,
    ListTraversal,
    ListMapping,
    ListFiltering,
    
    // String operations (migrated to Scheme)
    StringConcatenation,
    StringComparison,
    StringSearching,
    
    // Vector operations (migrated to Scheme)
    VectorConstruction,
    VectorAccess,
    VectorMapping,
    
    // I/O operations (migrated to Scheme)
    IOPortOperations,
    IOFileOperations,
    IOStringPorts,
    
    // Control operations (migrated to Scheme)
    ConditionalEvaluation,
    LoopConstruction,
    RecursiveCalls,
    
    // Exception operations (migrated to Scheme)
    ExceptionHandling,
    ErrorPropagation,
}

#[derive(Debug, Clone, Copy)]
pub enum ComplexityLevel {
    Trivial,   // 1-10 operations
    Small,     // 10-100 operations
    Medium,    // 100-1000 operations
    Large,     // 1000-10000 operations
}

impl Default for MigrationBenchmarkConfig {
    fn default() -> Self {
        Self {
            bootstrap_modes: vec![
                BootstrapMode::Minimal,
                BootstrapMode::Hybrid,
                BootstrapMode::Fallback,
            ],
            operation_types: vec![
                MigrationOperationType::ArithmeticBasic,
                MigrationOperationType::ArithmeticComplex,
                MigrationOperationType::ListConstruction,
                MigrationOperationType::ListMapping,
                MigrationOperationType::StringConcatenation,
                MigrationOperationType::VectorConstruction,
                MigrationOperationType::ConditionalEvaluation,
                MigrationOperationType::RecursiveCalls,
            ],
            sample_sizes: vec![100, 500, 1000],
            complexity_levels: vec![
                ComplexityLevel::Small,
                ComplexityLevel::Medium,
                ComplexityLevel::Large,
            ],
        }
    }
}

/// Migration performance measurement result
#[derive(Debug, Clone)]
pub struct MigrationPerformanceMeasurement {
    pub operation_type: MigrationOperationType,
    pub bootstrap_mode: BootstrapMode,
    pub complexity_level: ComplexityLevel,
    pub rust_time: Option<Duration>,
    pub scheme_time: Duration,
    pub performance_ratio: f64, // scheme_time / rust_time
    pub memory_usage: MemoryUsageMeasurement,
}

#[derive(Debug, Clone)]
pub struct MemoryUsageMeasurement {
    pub initial_memory: usize,
    pub peak_memory: usize,
    pub final_memory: usize,
    pub total_allocations: usize,
}

// ============================================================================
// BOOTSTRAP MODE COMPARISON BENCHMARKS
// ============================================================================

fn bench_bootstrap_mode_comparison(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("bootstrap_mode_comparison");
    group.measurement_time(Duration::from_secs(15));

    let bootstrap_modes = vec![
        (BootstrapMode::Minimal, "minimal"),
        (BootstrapMode::Hybrid, "hybrid"),
        (BootstrapMode::Fallback, "fallback"),
    ];

    let test_operations = vec![
        ("arithmetic", "(fold + 0 (map (lambda (x) (* x x)) (range 1 100)))"),
        ("list_ops", "(filter (lambda (x) (> x 5)) (map (lambda (x) (+ x 1)) (range 1 20)))"),
        ("string_ops", "(string-append (string-upcase \"hello\") \" \" (string-downcase \"WORLD\"))"),
        ("conditionals", "(cond ((> 10 5) 'yes) ((< 10 5) 'no) (else 'maybe))"),
    ];

    for (mode, mode_name) in bootstrap_modes {
        for (op_name, operation) in &test_operations {
            let benchmark_id = BenchmarkId::new(mode_name, op_name);
            
            group.bench_with_input(benchmark_id, &(mode, operation), |b, &(bootstrap_mode, expr)| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: bootstrap_mode,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let start = Instant::now();
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(4), config).unwrap();
                    let bootstrap_time = start.elapsed();
                    
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    let evaluation_start = Instant::now();
                    let _result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                    let evaluation_time = evaluation_start.elapsed();
                    
                    let _ = lambdust.shutdown().await;
                    
                    (bootstrap_time, evaluation_time)
                });
            });
        }
    }
    
    group.finish();
}

fn bench_startup_time_comparison(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("startup_time_comparison");
    group.measurement_time(Duration::from_secs(10));

    let modes = vec![
        (BootstrapMode::Minimal, "minimal"),
        (BootstrapMode::Hybrid, "hybrid"), 
        (BootstrapMode::Fallback, "fallback"),
    ];

    for (mode, mode_name) in modes {
        // Single-threaded startup
        group.bench_with_input(BenchmarkId::new("single_threaded", mode_name), &mode, |b, &bootstrap_mode| {
            b.iter(|| {
                let config = BootstrapIntegrationConfig {
                    mode: bootstrap_mode,
                    verbose: false,
                    ..Default::default()
                };
                
                let _runtime = lambdust::runtime::Runtime::with_bootstrap_config(config).unwrap();
            });
        });

        // Multi-threaded startup
        group.bench_with_input(BenchmarkId::new("multi_threaded", mode_name), &mode, |b, &bootstrap_mode| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: bootstrap_mode,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(4), config).unwrap();
                let _ = runtime.shutdown().await;
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// RUST VS SCHEME IMPLEMENTATION BENCHMARKS
// ============================================================================

fn bench_arithmetic_migration_impact(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("arithmetic_migration_impact");
    group.measurement_time(Duration::from_secs(12));

    let test_cases = vec![
        ("basic_arithmetic", "(+ 1 2 3 4 5)", ComplexityLevel::Trivial),
        ("complex_arithmetic", "(* (+ 1 2 3) (- 10 5) (/ 20 4))", ComplexityLevel::Small),
        ("gcd_computation", "(gcd 1071 462)", ComplexityLevel::Medium),
        ("fibonacci", "(define (fib n) (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 15)", ComplexityLevel::Large),
    ];

    for (name, expression, complexity) in test_cases {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_{:?}", name, complexity));
        
        // Benchmark with fallback mode (mostly Rust implementation)
        group.bench_with_input(BenchmarkId::new("rust_fallback", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Fallback,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });

        // Benchmark with minimal mode (Scheme implementation)
        group.bench_with_input(BenchmarkId::new("scheme_minimal", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Minimal,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

fn bench_list_operations_migration_impact(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("list_operations_migration_impact");
    group.measurement_time(Duration::from_secs(15));

    let test_cases = vec![
        ("list_construction", "(list 1 2 3 4 5)", ComplexityLevel::Trivial),
        ("list_append", "(append (list 1 2 3) (list 4 5 6) (list 7 8 9))", ComplexityLevel::Small),
        ("list_mapping", "(map (lambda (x) (* x x)) (range 1 50))", ComplexityLevel::Medium),
        ("list_filtering", "(filter (lambda (x) (and (> x 10) (< x 90))) (range 1 100))", ComplexityLevel::Medium),
        ("list_folding", "(fold + 0 (map (lambda (x) (* x x x)) (range 1 100)))", ComplexityLevel::Large),
        ("list_reversal", "(reverse (range 1 1000))", ComplexityLevel::Large),
    ];

    for (name, expression, complexity) in test_cases {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_{:?}", name, complexity));
        
        // Benchmark with fallback mode (Rust implementation)
        group.bench_with_input(BenchmarkId::new("rust_fallback", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Fallback,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });

        // Benchmark with minimal mode (Scheme implementation)
        group.bench_with_input(BenchmarkId::new("scheme_minimal", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Minimal,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

fn bench_string_operations_migration_impact(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("string_operations_migration_impact");
    group.measurement_time(Duration::from_secs(12));

    let test_cases = vec![
        ("string_append", r#"(string-append "hello" " " "world")"#, ComplexityLevel::Trivial),
        ("string_comparison", r#"(string=? "test" "test")"#, ComplexityLevel::Trivial),
        ("string_case_conversion", r#"(string-upcase (string-downcase "Hello World"))"#, ComplexityLevel::Small),
        ("string_searching", r#"(string-contains "hello world" "world")"#, ComplexityLevel::Small),
        ("string_manipulation", r#"(string-trim (string-replace "  hello world  " "world" "universe"))"#, ComplexityLevel::Medium),
        ("string_complex", 
         r#"(fold string-append "" (map (lambda (i) (string-append "item-" (number->string i) " ")) (range 1 100)))"#, 
         ComplexityLevel::Large),
    ];

    for (name, expression, complexity) in test_cases {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_{:?}", name, complexity));
        
        // Benchmark with fallback mode (Rust implementation)
        group.bench_with_input(BenchmarkId::new("rust_fallback", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Fallback,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });

        // Benchmark with minimal mode (Scheme implementation)
        group.bench_with_input(BenchmarkId::new("scheme_minimal", &benchmark_id), &expression, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Minimal,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// HOT PATH PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_hot_path_performance(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("hot_path_performance");
    group.measurement_time(Duration::from_secs(20));

    // These are operations likely to be called frequently
    let hot_path_operations = vec![
        ("car_cdr", "(car (cdr (list 1 2 3 4 5)))", 1000),
        ("cons_operations", "(cons 1 (cons 2 (cons 3 ())))", 1000),
        ("arithmetic_chain", "(+ (* 2 3) (- 10 5) (/ 20 4))", 500),
        ("conditional_simple", "(if (> 5 3) 'yes 'no)", 1000),
        ("lambda_application", "((lambda (x y) (+ x y)) 10 20)", 500),
        ("variable_lookup", "(define x 42) x", 1000),
        ("function_call", "(define (square x) (* x x)) (square 7)", 500),
    ];

    for (name, operation, iterations) in hot_path_operations {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}_iterations", name, iterations));
        
        // Create a benchmark program that repeats the operation
        let benchmark_program = format!(
            "(define (benchmark-loop n op) (if (= n 0) 'done (begin op (benchmark-loop (- n 1) op)))) {} (benchmark-loop {} '{}')",
            operation, iterations, operation
        );
        
        // Test with fallback mode
        group.bench_with_input(BenchmarkId::new("rust_fallback", &benchmark_id), &benchmark_program, |b, expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Fallback,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });

        // Test with minimal mode
        group.bench_with_input(BenchmarkId::new("scheme_minimal", &benchmark_id), &benchmark_program, |b, expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Minimal,
                    verbose: false,
                    ..Default::default()
                };
                
                let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let _ = lambdust.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// MEMORY USAGE PATTERN BENCHMARKS
// ============================================================================

fn bench_memory_usage_patterns(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_usage_patterns");
    group.measurement_time(Duration::from_secs(15));

    let memory_intensive_operations = vec![
        ("large_list_construction", "(range 1 10000)", "Creates large list"),
        ("deep_recursion", "(define (deep-list n) (if (= n 0) () (cons n (deep-list (- n 1))))) (deep-list 1000)", "Deep recursive construction"),
        ("many_small_objects", "(map (lambda (x) (list x (* x 2) (* x 3))) (range 1 1000))", "Many small allocations"),
        ("string_concatenation", "(fold string-append \"\" (map number->string (range 1 1000)))", "String memory pressure"),
        ("closure_retention", "(map (lambda (x) (lambda () x)) (range 1 500))", "Closure memory retention"),
    ];

    for (name, operation, _description) in memory_intensive_operations {
        // Test with fallback mode
        group.bench_with_input(BenchmarkId::new("rust_fallback", name), &operation, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Fallback,
                    verbose: false,
                    ..Default::default()
                };
                
                let start_memory = get_memory_usage();
                let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let peak_memory = get_memory_usage();
                let _ = lambdust.shutdown().await;
                let final_memory = get_memory_usage();
                
                MemoryUsageMeasurement {
                    initial_memory: start_memory,
                    peak_memory,
                    final_memory,
                    total_allocations: 0, // Placeholder
                }
            });
        });

        // Test with minimal mode
        group.bench_with_input(BenchmarkId::new("scheme_minimal", name), &operation, |b, &expr| {
            b.to_async(&rt).iter(|| async move {
                let config = BootstrapIntegrationConfig {
                    mode: BootstrapMode::Minimal,
                    verbose: false,
                    ..Default::default()
                };
                
                let start_memory = get_memory_usage();
                let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                let lambdust = MultithreadedLambdust::with_runtime(runtime);
                let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                let peak_memory = get_memory_usage();
                let _ = lambdust.shutdown().await;
                let final_memory = get_memory_usage();
                
                MemoryUsageMeasurement {
                    initial_memory: start_memory,
                    peak_memory,
                    final_memory,
                    total_allocations: 0, // Placeholder
                }
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_memory_usage() -> usize {
    // Placeholder memory usage measurement
    // In a real implementation, this would use system APIs or libraries like `jemalloc` or `procfs`
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    (timestamp % 1000000) as usize
}

fn generate_complexity_expression(op_type: MigrationOperationType, complexity: ComplexityLevel) -> String {
    let size = match complexity {
        ComplexityLevel::Trivial => 5,
        ComplexityLevel::Small => 50,
        ComplexityLevel::Medium => 500,
        ComplexityLevel::Large => 5000,
    };

    match op_type {
        MigrationOperationType::ArithmeticBasic => {
            format!("(+ {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "))
        }
        MigrationOperationType::ArithmeticComplex => {
            format!("(* (+ {}) (- {}) (/ {} 2))", 
                (1..=size/3).map(|i| i.to_string()).collect::<Vec<_>>().join(" "),
                (size/3+1..=2*size/3).map(|i| i.to_string()).collect::<Vec<_>>().join(" "),
                (2*size/3+1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")
            )
        }
        MigrationOperationType::ListConstruction => {
            format!("(list {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "))
        }
        MigrationOperationType::ListMapping => {
            format!("(map (lambda (x) (* x x)) (range 1 {}))", size)
        }
        MigrationOperationType::StringConcatenation => {
            format!("(fold string-append \"\" (map number->string (range 1 {})))", size)
        }
        MigrationOperationType::VectorConstruction => {
            format!("(vector {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "))
        }
        MigrationOperationType::ConditionalEvaluation => {
            format!("(fold (lambda (acc x) (if (> x {}) (+ acc x) acc)) 0 (range 1 {}))", size / 2, size)
        }
        MigrationOperationType::RecursiveCalls => {
            format!("(define (sum-to n) (if (= n 0) 0 (+ n (sum-to (- n 1))))) (sum-to {})", size.min(100))
        }
        _ => format!("(+ {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")), // Default fallback
    }
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    bootstrap_benches,
    bench_bootstrap_mode_comparison,
    bench_startup_time_comparison
);

criterion_group!(
    migration_impact_benches,
    bench_arithmetic_migration_impact,
    bench_list_operations_migration_impact,
    bench_string_operations_migration_impact
);

criterion_group!(
    hot_path_benches,
    bench_hot_path_performance
);

criterion_group!(
    memory_pattern_benches,
    bench_memory_usage_patterns
);

criterion_main!(
    bootstrap_benches,
    migration_impact_benches,
    hot_path_benches,
    memory_pattern_benches
);