//! Core Operation Benchmarks
//!
//! Comprehensive benchmarks for core Lambdust operations including
//! lists, strings, vectors, I/O, arithmetic, and control structures.
//! These benchmarks measure the performance of both migrated and 
//! non-migrated operations to ensure acceptable performance characteristics.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime as TokioRuntime;

/// Core operation categories for benchmarking
#[derive(Debug, Clone, Copy)]
pub enum CoreOperationCategory {
    Arithmetic,
    List,
    String,
    Vector,
    IO,
    Control,
    Memory,
    Concurrency,
}

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct CoreBenchmarkConfig {
    pub categories: Vec<CoreOperationCategory>,
    pub data_sizes: Vec<usize>,
    pub thread_counts: Vec<usize>,
    pub bootstrap_modes: Vec<BootstrapMode>,
}

impl Default for CoreBenchmarkConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                CoreOperationCategory::Arithmetic,
                CoreOperationCategory::List,
                CoreOperationCategory::String,
                CoreOperationCategory::Vector,
                CoreOperationCategory::IO,
                CoreOperationCategory::Control,
            ],
            data_sizes: vec![10, 100, 1000, 10000],
            thread_counts: vec![1, 2, 4, 8],
            bootstrap_modes: vec![BootstrapMode::Minimal, BootstrapMode::Hybrid],
        }
    }
}

// ============================================================================
// ARITHMETIC OPERATION BENCHMARKS
// ============================================================================

fn bench_arithmetic_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("arithmetic_operations");
    group.measurement_time(Duration::from_secs(12));

    let arithmetic_operations = vec![
        ("basic_addition", "fold", "(fold + 0 (range 1 {}))", vec![100, 1000, 10000]),
        ("multiplication", "fold", "(fold * 1 (range 1 {}))", vec![50, 100, 200]),
        ("mixed_arithmetic", "map", "(map (lambda (x) (+ (* x x) (/ x 2))) (range 1 {}))", vec![100, 1000, 5000]),
        ("modular_arithmetic", "map", "(map (lambda (x) (modulo (* x 17) 31)) (range 1 {}))", vec![1000, 5000, 10000]),
        ("gcd_operations", "map", "(let ((pairs (map (lambda (x) (list x (+ x 1))) (range 1 {})))) (map (lambda (p) (gcd (car p) (cadr p))) pairs))", vec![100, 500, 1000]),
        ("exponentiation", "map", "(map (lambda (x) (expt x 3)) (range 1 {}))", vec![100, 500, 1000]),
        ("sqrt_operations", "map", "(map sqrt (map (lambda (x) (* x x)) (range 1 {})))", vec![100, 1000, 5000]),
        ("trigonometric", "map", "(map sin (map (lambda (x) (/ x 100.0)) (range 1 {})))", vec![100, 500, 1000]),
    ];

    for (name, _op_type, expression_template, sizes) in arithmetic_operations {
        for &size in &sizes {
            let expression = expression_template.replace("{}", &size.to_string());
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// LIST OPERATION BENCHMARKS
// ============================================================================

fn bench_list_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("list_operations");
    group.measurement_time(Duration::from_secs(15));

    let list_operations = vec![
        ("construction", "(list {})", vec![100, 1000, 10000]),
        ("append_chain", "(fold append '() (map (lambda (x) (list x)) (range 1 {})))", vec![100, 500, 1000]),
        ("mapping", "(map (lambda (x) (* x x)) (range 1 {}))", vec![1000, 5000, 10000]),
        ("filtering", "(filter (lambda (x) (> x {})) (range 1 {}))", vec![1000, 5000, 10000]),
        ("folding", "(fold + 0 (range 1 {}))", vec![1000, 10000, 50000]),
        ("reversal", "(reverse (range 1 {}))", vec![1000, 5000, 10000]),
        ("member_search", "(member {} (range 1 {}))", vec![500, 1000, 2000]),
        ("assoc_lookup", "(let ((alist (map (lambda (x) (cons x (* x x))) (range 1 {})))) (assoc {} alist))", vec![100, 500, 1000]),
        ("list_sorting", "(sort (reverse (range 1 {})) <)", vec![100, 500, 1000]),
        ("nested_mapping", "(map (lambda (lst) (map (lambda (x) (* x 2)) lst)) (map (lambda (x) (list x (+ x 1) (+ x 2))) (range 1 {})))", vec![50, 100, 200]),
    ];

    for (name, expression_template, sizes) in list_operations {
        for &size in &sizes {
            let expression = if name == "filtering" || name == "member_search" || name == "assoc_lookup" {
                expression_template.replace("{}", &(size / 2).to_string()).replace("{}", &size.to_string())
            } else if name == "construction" {
                format!("(list {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "))
            } else {
                expression_template.replace("{}", &size.to_string())
            };
            
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// STRING OPERATION BENCHMARKS
// ============================================================================

fn bench_string_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("string_operations");
    group.measurement_time(Duration::from_secs(12));

    let string_operations = vec![
        ("concatenation", r#"(fold string-append "" (map (lambda (x) (string-append "item-" (number->string x) " ")) (range 1 {})))"#, vec![100, 500, 1000]),
        ("comparison", r#"(let ((strings (map (lambda (x) (string-append "string-" (number->string x))) (range 1 {})))) (filter (lambda (s) (string=? s "string-{}")) strings))"#, vec![100, 500, 1000]),
        ("case_conversion", r#"(map string-upcase (map string-downcase (map (lambda (x) (string-append "Test-" (number->string x))) (range 1 {}))))"#, vec![100, 500, 1000]),
        ("substring_operations", r#"(map (lambda (s) (substring s 0 (min 5 (string-length s)))) (map (lambda (x) (string-append "prefix-" (number->string x) "-suffix")) (range 1 {})))"#, vec![100, 500, 1000]),
        ("string_searching", r#"(let ((strings (map (lambda (x) (string-append "item-" (number->string x) "-test")) (range 1 {})))) (filter (lambda (s) (string-contains s "test")) strings))"#, vec![100, 500, 1000]),
        ("string_replacement", r#"(map (lambda (s) (string-replace s "item" "element")) (map (lambda (x) (string-append "item-" (number->string x))) (range 1 {})))"#, vec![100, 500, 1000]),
        ("string_splitting", r#"(map (lambda (s) (string-split s "-")) (map (lambda (x) (string-append "part1-part2-" (number->string x))) (range 1 {})))"#, vec![100, 500, 1000]),
        ("string_trimming", r#"(map string-trim (map (lambda (x) (string-append "  spaced-" (number->string x) "  ")) (range 1 {})))"#, vec![100, 500, 1000]),
    ];

    for (name, expression_template, sizes) in string_operations {
        for &size in &sizes {
            let expression = if name == "comparison" {
                expression_template.replace("{}", &size.to_string()).replace("{}", &(size / 2).to_string())
            } else {
                expression_template.replace("{}", &size.to_string())
            };
            
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// VECTOR OPERATION BENCHMARKS
// ============================================================================

fn bench_vector_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("vector_operations");
    group.measurement_time(Duration::from_secs(10));

    let vector_operations = vec![
        ("construction", "(vector {})", vec![100, 1000, 10000]),
        ("list_conversion", "(list->vector (range 1 {}))", vec![1000, 5000, 10000]),
        ("access_pattern", "(let ((v (list->vector (range 1 {})))) (map (lambda (i) (vector-ref v i)) (range 0 (- (vector-length v) 1))))", vec![1000, 5000, 10000]),
        ("mapping", "(vector-map (lambda (x) (* x x)) (list->vector (range 1 {})))", vec![1000, 5000, 10000]),
        ("folding", "(vector-fold + 0 (list->vector (range 1 {})))", vec![1000, 10000, 50000]),
        ("append_operations", "(vector-append (list->vector (range 1 {})) (list->vector (range {} {})))", vec![500, 1000, 2000]),
        ("fill_operations", "(let ((v (make-vector {}))) (vector-fill! v 42) v)", vec![1000, 5000, 10000]),
        ("copy_operations", "(vector-copy (list->vector (range 1 {})))", vec![1000, 5000, 10000]),
    ];

    for (name, expression_template, sizes) in vector_operations {
        for &size in &sizes {
            let expression = match name {
                "construction" => format!("(vector {})", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")),
                "append_operations" => expression_template.replace("{}", &size.to_string()).replace("{}", &(size + 1).to_string()).replace("{}", &(size * 2).to_string()),
                _ => expression_template.replace("{}", &size.to_string()),
            };
            
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// I/O OPERATION BENCHMARKS
// ============================================================================

fn bench_io_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("io_operations");
    group.measurement_time(Duration::from_secs(15));

    let io_operations = vec![
        ("string_port_creation", r#"(map (lambda (x) (open-output-string)) (range 1 {}))"#, vec![100, 500, 1000]),
        ("string_port_writing", r#"(let ((ports (map (lambda (x) (open-output-string)) (range 1 {})))) (map (lambda (port) (write "test data" port)) ports))"#, vec![100, 500, 1000]),
        ("string_port_reading", r#"(let ((port (open-input-string "test data to read"))) (map (lambda (x) (read-char port)) (range 1 {})))"#, vec![10, 50, 100]),
        ("port_predicates", r#"(let ((ports (map (lambda (x) (open-output-string)) (range 1 {})))) (map input-port? ports))"#, vec![100, 500, 1000]),
        ("output_operations", r#"(let ((port (open-output-string))) (map (lambda (x) (write x port)) (range 1 {})) (get-output-string port))"#, vec![100, 500, 1000]),
        ("current_port_operations", r#"(let ((original (current-output-port))) (map (lambda (x) (with-output-to-string (lambda () (write x)))) (range 1 {})))"#, vec![100, 500, 1000]),
        ("eof_detection", r#"(let ((port (open-input-string (fold string-append "" (map number->string (range 1 {})))))) (let loop ((count 0)) (if (eof-object? (peek-char port)) count (begin (read-char port) (loop (+ count 1))))))"#, vec![100, 500, 1000]),
    ];

    for (name, expression_template, sizes) in io_operations {
        for &size in &sizes {
            let expression = expression_template.replace("{}", &size.to_string());
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// CONTROL STRUCTURE BENCHMARKS
// ============================================================================

fn bench_control_structures(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("control_structures");
    group.measurement_time(Duration::from_secs(12));

    let control_operations = vec![
        ("conditional_chains", "(map (lambda (x) (cond ((< x {}) 'small) ((< x {}) 'medium) (else 'large))) (range 1 {}))", vec![100, 500, 1000]),
        ("case_statements", "(map (lambda (x) (case (modulo x 5) ((0) 'zero) ((1) 'one) ((2) 'two) ((3) 'three) (else 'other))) (range 1 {}))", vec![100, 500, 1000]),
        ("boolean_operations", "(map (lambda (x) (and (> x 0) (< x {}) (not (= (modulo x 3) 0)))) (range 1 {}))", vec![100, 500, 1000]),
        ("let_bindings", "(map (lambda (x) (let ((a (* x 2)) (b (+ x 1)) (c (- x 1))) (+ a b c))) (range 1 {}))", vec![100, 1000, 5000]),
        ("let_star_bindings", "(map (lambda (x) (let* ((a x) (b (* a 2)) (c (+ b a))) c)) (range 1 {}))", vec![100, 1000, 5000]),
        ("letrec_bindings", "(letrec ((countdown (lambda (n) (if (= n 0) 'done (countdown (- n 1)))))) (map countdown (range 1 {})))", vec![10, 50, 100]),
        ("do_loops", "(map (lambda (n) (do ((i 0 (+ i 1)) (sum 0 (+ sum i))) ((= i n) sum))) (range 1 {}))", vec![10, 50, 100]),
        ("apply_operations", "(map (lambda (lst) (apply + lst)) (map (lambda (n) (range 1 n)) (range 1 {})))", vec![10, 50, 100]),
    ];

    for (name, expression_template, sizes) in control_operations {
        for &size in &sizes {
            let expression = match name {
                "conditional_chains" => expression_template.replace("{}", &(size / 3).to_string()).replace("{}", &(size * 2 / 3).to_string()).replace("{}", &size.to_string()),
                "boolean_operations" => expression_template.replace("{}", &size.to_string()).replace("{}", &size.to_string()),
                _ => expression_template.replace("{}", &size.to_string()),
            };
            
            let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}", name, size));
            
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(benchmark_id, &expression, |b, expr| {
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
    }
    
    group.finish();
}

// ============================================================================
// CONCURRENT OPERATION BENCHMARKS
// ============================================================================

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(15));

    let concurrent_operations = vec![
        ("parallel_arithmetic", "(fold + 0 (range 1 {}))", vec![1000, 5000, 10000]),
        ("parallel_mapping", "(map (lambda (x) (* x x x)) (range 1 {}))", vec![1000, 5000, 10000]),
        ("parallel_filtering", "(filter (lambda (x) (> x {})) (range 1 {}))", vec![1000, 5000, 10000]),
    ];

    let thread_counts = vec![1, 2, 4, 8];

    for (name, expression_template, sizes) in concurrent_operations {
        for &size in &sizes {
            for &thread_count in &thread_counts {
                let expression = if name == "parallel_filtering" {
                    expression_template.replace("{}", &(size / 2).to_string()).replace("{}", &size.to_string())
                } else {
                    expression_template.replace("{}", &size.to_string())
                };
                
                let benchmark_id = BenchmarkId::from_parameter(format!("{}_{}_{}_threads", name, size, thread_count));
                
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(benchmark_id, &(expression, thread_count), |b, (expr, threads)| {
                    b.to_async(&rt).iter(|| async move {
                        let config = BootstrapIntegrationConfig {
                            mode: BootstrapMode::Minimal,
                            verbose: false,
                            ..Default::default()
                        };
                        
                        let runtime = LambdustRuntime::with_bootstrap_config(Some(*threads), config).unwrap();
                        let lambdust = MultithreadedLambdust::with_runtime(runtime);
                        let result = lambdust.eval(expr, Some("benchmark")).await.unwrap();
                        let _ = lambdust.shutdown().await;
                        result
                    });
                });
            }
        }
    }
    
    group.finish();
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    arithmetic_benches,
    bench_arithmetic_operations
);

criterion_group!(
    list_benches,
    bench_list_operations
);

criterion_group!(
    string_benches,
    bench_string_operations
);

criterion_group!(
    vector_benches,
    bench_vector_operations
);

criterion_group!(
    io_benches,
    bench_io_operations
);

criterion_group!(
    control_benches,
    bench_control_structures
);

criterion_group!(
    concurrent_benches,
    bench_concurrent_operations
);

criterion_main!(
    arithmetic_benches,
    list_benches,
    string_benches,
    vector_benches,
    io_benches,
    control_benches,
    concurrent_benches
);