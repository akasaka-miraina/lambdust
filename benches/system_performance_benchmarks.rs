//! System Performance Benchmarks
//!
//! Benchmarks for system-level performance characteristics including:
//! - REPL startup time and responsiveness
//! - Module loading and compilation performance
//! - Memory allocation and garbage collection behavior
//! - Thread safety and concurrent operation performance
//! - Bootstrap system performance across different modes

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust, Lambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime as TokioRuntime;

/// System benchmark configuration
#[derive(Debug, Clone)]
pub struct SystemBenchmarkConfig {
    pub bootstrap_modes: Vec<BootstrapMode>,
    pub thread_counts: Vec<usize>,
    pub module_sizes: Vec<usize>,
    pub memory_pressure_levels: Vec<MemoryPressureLevel>,
    pub repl_interaction_patterns: Vec<REPLInteractionPattern>,
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryPressureLevel {
    Low,     // Small allocations, infrequent GC
    Medium,  // Moderate allocations, regular GC
    High,    // Large allocations, frequent GC
    Extreme, // Stress testing with maximum pressure
}

#[derive(Debug, Clone)]
pub enum REPLInteractionPattern {
    SimpleExpressions,    // Basic arithmetic and function calls
    ComplexExpressions,   // Nested operations and data structures
    InteractiveSession,   // Mixed expressions with variable definitions
    LibraryLoading,      // Module imports and library usage
    DebuggingSession,    // Error handling and recovery
}

impl Default for SystemBenchmarkConfig {
    fn default() -> Self {
        Self {
            bootstrap_modes: vec![BootstrapMode::Minimal, BootstrapMode::Hybrid, BootstrapMode::Fallback],
            thread_counts: vec![1, 2, 4, 8],
            module_sizes: vec![10, 50, 100, 500],
            memory_pressure_levels: vec![
                MemoryPressureLevel::Low,
                MemoryPressureLevel::Medium,
                MemoryPressureLevel::High,
            ],
            repl_interaction_patterns: vec![
                REPLInteractionPattern::SimpleExpressions,
                REPLInteractionPattern::ComplexExpressions,
                REPLInteractionPattern::InteractiveSession,
            ],
        }
    }
}

// ============================================================================
// REPL STARTUP AND RESPONSIVENESS BENCHMARKS
// ============================================================================

fn bench_repl_startup_time(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("repl_startup_time");
    group.measurement_time(Duration::from_secs(15));

    let bootstrap_modes = vec![
        (BootstrapMode::Minimal, "minimal"),
        (BootstrapMode::Hybrid, "hybrid"),
        (BootstrapMode::Fallback, "fallback"),
    ];

    for (mode, mode_name) in bootstrap_modes {
        // Single-threaded REPL startup
        group.bench_with_input(BenchmarkId::new("single_threaded", mode_name), &mode, |b, &bootstrap_mode| {
            b.iter(|| {
                let config = BootstrapIntegrationConfig {
                    mode: bootstrap_mode,
                    verbose: false,
                    ..Default::default()
                };
                
                let start = Instant::now();
                let _runtime = lambdust::runtime::Runtime::with_bootstrap_config(config).unwrap();
                let _lambdust = Lambdust::with_runtime(_runtime);
                start.elapsed()
            });
        });

        // Multi-threaded REPL startup
        for &thread_count in &[2, 4, 8] {
            let param_name = format!("{}_{}_threads", mode_name, thread_count);
            group.bench_with_input(BenchmarkId::from_parameter(&param_name), &(mode, thread_count), |b, &(bootstrap_mode, threads)| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: bootstrap_mode,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let start = Instant::now();
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(threads), config).unwrap();
                    let _lambdust = MultithreadedLambdust::with_runtime(runtime);
                    let startup_time = start.elapsed();
                    let _ = _lambdust.shutdown().await;
                    startup_time
                });
            });
        }
    }
    
    group.finish();
}

fn bench_repl_response_time(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("repl_response_time");
    group.measurement_time(Duration::from_secs(20));

    let interaction_patterns = vec![
        (REPLInteractionPattern::SimpleExpressions, vec![
            "(+ 1 2 3)",
            "(* 4 5 6)",
            "(define x 42)",
            "x",
            "(if (> x 40) 'big 'small)",
        ]),
        (REPLInteractionPattern::ComplexExpressions, vec![
            "(map (lambda (x) (* x x)) (range 1 100))",
            "(filter (lambda (x) (> x 50)) (range 1 100))",
            "(fold + 0 (map (lambda (x) (* x x)) (range 1 50)))",
            "(let ((lst (range 1 20))) (reverse (sort lst >)))",
        ]),
        (REPLInteractionPattern::InteractiveSession, vec![
            "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))",
            "(factorial 10)",
            "(define test-list (range 1 50))",
            "(map factorial (filter (lambda (x) (< x 8)) test-list))",
        ]),
    ];

    for (pattern, expressions) in interaction_patterns {
        let pattern_name = format!("{:?}", pattern);
        
        for &bootstrap_mode in &[BootstrapMode::Minimal, BootstrapMode::Hybrid] {
            let mode_name = format!("{:?}", bootstrap_mode);
            let benchmark_id = BenchmarkId::new(&pattern_name, &mode_name);
            
            group.bench_with_input(benchmark_id, &(bootstrap_mode, expressions.clone()), |b, (mode, exprs)| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: *mode,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let mut total_response_time = Duration::ZERO;
                    let mut response_count = 0;
                    
                    for expr in exprs {
                        let start = Instant::now();
                        let _result = lambdust.eval(expr, Some("repl")).await.unwrap_or_default();
                        total_response_time += start.elapsed();
                        response_count += 1;
                    }
                    
                    let _ = lambdust.shutdown().await;
                    
                    if response_count > 0 {
                        total_response_time / response_count as u32
                    } else {
                        Duration::ZERO
                    }
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// MODULE LOADING AND COMPILATION BENCHMARKS
// ============================================================================

fn bench_module_loading_performance(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("module_loading_performance");
    group.measurement_time(Duration::from_secs(20));

    let module_types = vec![
        ("simple_arithmetic", generate_arithmetic_module, vec![10, 50, 100]),
        ("list_utilities", generate_list_utilities_module, vec![10, 30, 50]),
        ("string_processing", generate_string_processing_module, vec![10, 30, 50]),
        ("recursive_functions", generate_recursive_functions_module, vec![5, 15, 25]),
    ];

    for (module_type, generator, sizes) in module_types {
        for &size in &sizes {
            let module_code = generator(size);
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_size_{}", module_type, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &module_code, |b, code| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let start = Instant::now();
                    let _result = lambdust.eval(code, Some("module")).await.unwrap();
                    let loading_time = start.elapsed();
                    
                    let _ = lambdust.shutdown().await;
                    loading_time
                });
            });
        }
    }
    
    group.finish();
}

fn bench_standard_library_loading(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("standard_library_loading");
    group.measurement_time(Duration::from_secs(15));

    let library_imports = vec![
        ("base_library", "(import (scheme base))"),
        ("char_library", "(import (scheme char))"),
        ("list_library", "(import (scheme list))"),
        ("string_library", "(import (scheme string))"),
        ("vector_library", "(import (scheme vector))"),
        ("multiple_libraries", "(import (scheme base) (scheme char) (scheme list))"),
        ("srfi_libraries", "(import (srfi 1) (srfi 13) (srfi 26))"),
    ];

    for &bootstrap_mode in &[BootstrapMode::Minimal, BootstrapMode::Hybrid] {
        let mode_name = format!("{:?}", bootstrap_mode);
        
        for (library_name, import_statement) in &library_imports {
            let benchmark_id = BenchmarkId::new(&mode_name, library_name);
            
            group.bench_with_input(benchmark_id, &(bootstrap_mode, import_statement), |b, &(mode, import)| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let start = Instant::now();
                    let _result = lambdust.eval(import, Some("library-import")).await.unwrap_or_default();
                    let import_time = start.elapsed();
                    
                    let _ = lambdust.shutdown().await;
                    import_time
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// MEMORY ALLOCATION AND GC BENCHMARKS
// ============================================================================

fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_allocation_patterns");
    group.measurement_time(Duration::from_secs(20));

    let allocation_patterns = vec![
        ("small_objects", MemoryPressureLevel::Low, "(map (lambda (x) (list x (* x 2))) (range 1 {}))", vec![1000, 5000, 10000]),
        ("medium_objects", MemoryPressureLevel::Medium, "(map (lambda (x) (list->vector (range x (+ x 50)))) (range 1 {}))", vec![100, 500, 1000]),
        ("large_objects", MemoryPressureLevel::High, "(map (lambda (x) (make-string (* x 100) #\\a)) (range 1 {}))", vec![10, 50, 100]),
        ("mixed_allocation", MemoryPressureLevel::Medium, generate_mixed_allocation_pattern, vec![500, 1000, 2000]),
    ];

    for (pattern_name, pressure_level, pattern_template, sizes) in allocation_patterns {
        for &size in &sizes {
            let expression = if pattern_name == "mixed_allocation" {
                generate_mixed_allocation_pattern(size)
            } else {
                pattern_template.replace("{}", &size.to_string())
            };
            
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{:?}_{}", pattern_name, pressure_level, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let start_memory = get_memory_usage_estimate();
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let _result = lambdust.eval(expr, Some("memory-benchmark")).await.unwrap();
                    let peak_memory = get_memory_usage_estimate();
                    
                    let _ = lambdust.shutdown().await;
                    let final_memory = get_memory_usage_estimate();
                    
                    MemoryUsageReport {
                        initial: start_memory,
                        peak: peak_memory,
                        final_usage: final_memory,
                        allocation_delta: peak_memory.saturating_sub(start_memory),
                    }
                });
            });
        }
    }
    
    group.finish();
}

fn bench_garbage_collection_performance(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("garbage_collection_performance");
    group.measurement_time(Duration::from_secs(25));

    let gc_scenarios = vec![
        ("cyclic_references", generate_cyclic_reference_workload, vec![100, 500, 1000]),
        ("temporary_objects", generate_temporary_object_workload, vec![1000, 5000, 10000]),
        ("long_lived_objects", generate_long_lived_object_workload, vec![500, 1000, 2000]),
        ("mixed_lifetimes", generate_mixed_lifetime_workload, vec![500, 1000, 2000]),
    ];

    for (scenario_name, generator, sizes) in gc_scenarios {
        for &size in &sizes {
            let workload = generator(size);
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", scenario_name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &workload, |b, work| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(4), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let start = Instant::now();
                    let _result = lambdust.eval(work, Some("gc-benchmark")).await.unwrap();
                    let gc_time = start.elapsed();
                    
                    let _ = lambdust.shutdown().await;
                    gc_time
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// THREAD SAFETY AND CONCURRENT PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_thread_safety_performance(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("thread_safety_performance");
    group.measurement_time(Duration::from_secs(20));

    let concurrent_scenarios = vec![
        ("shared_data_access", "(define shared-counter 0) (define (increment!) (set! shared-counter (+ shared-counter 1))) (map (lambda (x) (increment!)) (range 1 {}))"),
        ("parallel_computation", "(map (lambda (x) (fold + 0 (map (lambda (y) (* x y)) (range 1 100)))) (range 1 {}))"),
        ("concurrent_list_ops", "(let ((shared-list (range 1 100))) (map (lambda (x) (filter (lambda (y) (> y x)) shared-list)) (range 1 {})))"),
    ];

    let thread_counts = vec![2, 4, 8, 16];

    for (scenario_name, expression_template) in concurrent_scenarios {
        for &thread_count in &thread_counts {
            for &workload_size in &[50, 100, 200] {
                let expression = expression_template.replace("{}", &workload_size.to_string());
                let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}_{}_threads", scenario_name, workload_size, thread_count));
                
                group.throughput(Throughput::Elements(workload_size as u64));
                group.bench_with_input(benchmark_id, &(expression, thread_count), |b, (expr, threads)| {
                    b.to_async(&rt).iter(|| async move {
                        let config = BootstrapIntegrationConfig {
                            mode: BootstrapMode::Minimal,
                            verbose: false,
                            ..Default::default()
                        };
                        
                        let runtime = LambdustRuntime::with_bootstrap_config(Some(*threads), config).unwrap();
                        let lambdust = MultithreadedLambdust::with_runtime(runtime);
                        
                        let start = Instant::now();
                        let _result = lambdust.eval(expr, Some("thread-safety-benchmark")).await.unwrap();
                        let execution_time = start.elapsed();
                        
                        let _ = lambdust.shutdown().await;
                        execution_time
                    });
                });
            }
        }
    }
    
    group.finish();
}

// ============================================================================
// BOOTSTRAP SYSTEM PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_bootstrap_system_performance(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("bootstrap_system_performance");
    group.measurement_time(Duration::from_secs(20));

    let bootstrap_scenarios = vec![
        ("minimal_bootstrap", BootstrapMode::Minimal, "Bootstrap with minimal Scheme stdlib"),
        ("hybrid_bootstrap", BootstrapMode::Hybrid, "Bootstrap with hybrid Rust/Scheme stdlib"),
        ("fallback_bootstrap", BootstrapMode::Fallback, "Bootstrap with fallback Rust stdlib"),
    ];

    let workloads = vec![
        ("arithmetic_heavy", "(fold * 1 (range 1 100))"),
        ("list_heavy", "(fold append '() (map (lambda (x) (list x (* x 2) (* x 3))) (range 1 50)))"),
        ("string_heavy", "(fold string-append \"\" (map (lambda (x) (string-append \"item-\" (number->string x) \"-\")) (range 1 100)))"),
        ("mixed_workload", "(let ((nums (range 1 50))) (fold string-append \"\" (map (lambda (x) (string-append (number->string (* x x)) \"-\")) (filter (lambda (x) (> x 10)) nums))))"),
    ];

    for (mode_name, bootstrap_mode, _description) in bootstrap_scenarios {
        for (workload_name, workload_expr) in &workloads {
            let benchmark_id = BenchmarkId::new(mode_name, workload_name);
            
            group.bench_with_input(benchmark_id, &(bootstrap_mode, workload_expr), |b, &(mode, expr)| {
                b.to_async(&rt).iter(|| async move {
                    let config = BootstrapIntegrationConfig {
                        mode,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let bootstrap_start = Instant::now();
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                    let bootstrap_time = bootstrap_start.elapsed();
                    
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let execution_start = Instant::now();
                    let _result = lambdust.eval(expr, Some("bootstrap-benchmark")).await.unwrap();
                    let execution_time = execution_start.elapsed();
                    
                    let _ = lambdust.shutdown().await;
                    
                    BootstrapPerformanceReport {
                        bootstrap_time,
                        execution_time,
                        total_time: bootstrap_time + execution_time,
                    }
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS AND TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub struct MemoryUsageReport {
    pub initial: usize,
    pub peak: usize,
    pub final_usage: usize,
    pub allocation_delta: usize,
}

#[derive(Debug, Clone)]
pub struct BootstrapPerformanceReport {
    pub bootstrap_time: Duration,
    pub execution_time: Duration,
    pub total_time: Duration,
}

fn get_memory_usage_estimate() -> usize {
    // Placeholder memory usage estimation
    // In a real implementation, this would use system APIs or profiling tools
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    (timestamp % 10000000) as usize
}

fn generate_arithmetic_module(size: usize) -> String {
    let mut module = String::new();
    module.push_str(";; Arithmetic module\n");
    
    for i in 1..=size {
        module.push_str(&format!(
            "(define (func-{} x) (+ (* x {}) (/ x 2)))\n",
            i, i
        ));
    }
    
    module.push_str(&format!(
        "(define (combined-arithmetic x) (+ {}))\n",
        (1..=size).map(|i| format!("(func-{} x)", i)).collect::<Vec<_>>().join(" ")
    ));
    
    module
}

fn generate_list_utilities_module(size: usize) -> String {
    let mut module = String::new();
    module.push_str(";; List utilities module\n");
    
    for i in 1..=size {
        module.push_str(&format!(
            "(define (list-util-{} lst) (filter (lambda (x) (> x {})) (map (lambda (x) (* x {})) lst)))\n",
            i, i, i
        ));
    }
    
    module
}

fn generate_string_processing_module(size: usize) -> String {
    let mut module = String::new();
    module.push_str(";; String processing module\n");
    
    for i in 1..=size {
        module.push_str(&format!(
            "(define (string-proc-{} s) (string-append s \"-processed-{}\" (number->string {})))\n",
            i, i, i
        ));
    }
    
    module
}

fn generate_recursive_functions_module(size: usize) -> String {
    let mut module = String::new();
    module.push_str(";; Recursive functions module\n");
    
    for i in 1..=size {
        module.push_str(&format!(
            "(define (recursive-{} n) (if (<= n 1) {} (+ {} (recursive-{} (- n 1)))))\n",
            i, i, i, i
        ));
    }
    
    module
}

fn generate_mixed_allocation_pattern(size: usize) -> String {
    format!(
        "(let ((data (range 1 {}))) \
         (append \
           (map (lambda (x) (list x (* x 2))) data) \
           (map (lambda (x) (make-string (+ x 10) #\\a)) (range 1 {})) \
           (map (lambda (x) (list->vector (range x (+ x 10)))) (range 1 {}))))",
        size, size / 10, size / 20
    )
}

fn generate_cyclic_reference_workload(size: usize) -> String {
    format!(
        "(let ((objects (map (lambda (x) (list x)) (range 1 {})))) \
         (map (lambda (i) \
                (let ((obj (list-ref objects i)) \
                      (next-obj (list-ref objects (modulo (+ i 1) {})))) \
                  (set-cdr! obj next-obj))) \
              (range 0 (- {} 1))) \
         objects)",
        size, size, size
    )
}

fn generate_temporary_object_workload(size: usize) -> String {
    format!(
        "(fold (lambda (acc x) \
                (let ((temp (list x (* x 2) (* x 3)))) \
                  (+ acc (fold + 0 temp)))) \
              0 (range 1 {}))",
        size
    )
}

fn generate_long_lived_object_workload(size: usize) -> String {
    format!(
        "(let ((long-lived (map (lambda (x) (list x (* x x))) (range 1 {})))) \
         (map (lambda (obj) (fold + 0 obj)) long-lived))",
        size
    )
}

fn generate_mixed_lifetime_workload(size: usize) -> String {
    format!(
        "(let ((long-lived (range 1 {}))) \
         (fold (lambda (acc x) \
                 (let ((temp (list x (* x 2)))) \
                   (+ acc x (fold + 0 temp)))) \
               0 long-lived))",
        size
    )
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    repl_benches,
    bench_repl_startup_time,
    bench_repl_response_time
);

criterion_group!(
    module_benches,
    bench_module_loading_performance,
    bench_standard_library_loading
);

criterion_group!(
    memory_benches,
    bench_memory_allocation_patterns,
    bench_garbage_collection_performance
);

criterion_group!(
    thread_safety_benches,
    bench_thread_safety_performance
);

criterion_group!(
    bootstrap_benches,
    bench_bootstrap_system_performance
);

criterion_main!(
    repl_benches,
    module_benches,
    memory_benches,
    thread_safety_benches,
    bootstrap_benches
);