//! Scheme Operation Benchmarks
//!
//! Comprehensive benchmarks for operations implemented in Scheme, providing
//! in-language performance measurement and comparison capabilities.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime as TokioRuntime;

/// Scheme benchmark suite definition
#[derive(Debug, Clone)]
pub struct SchemeBenchmarkSuite {
    pub name: String,
    pub setup_code: String,
    pub benchmark_code: String,
    pub cleanup_code: Option<String>,
    pub iterations: usize,
    pub complexity_level: String,
}

/// Scheme benchmarking infrastructure
pub struct SchemeBenchmarkRunner {
    runtime: MultithreadedLambdust,
}

impl SchemeBenchmarkRunner {
    pub async fn new(bootstrap_mode: BootstrapMode) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = BootstrapIntegrationConfig {
            mode: bootstrap_mode,
            verbose: false,
            ..Default::default()
        };
        
        let runtime_impl = LambdustRuntime::with_bootstrap_config(Some(1), config)?;
        let runtime = MultithreadedLambdust::with_runtime(runtime_impl);
        
        // Initialize the Scheme-based benchmarking infrastructure
        let benchmark_init = r#"
            ;; Scheme Benchmarking Infrastructure
            
            ;; Timing utilities
            (define (current-milliseconds)
              ;; Placeholder - in real implementation would use system time
              (inexact (floor (* (random) 1000))))
            
            (define (time-execution thunk)
              (let ((start (current-milliseconds)))
                (let ((result (thunk)))
                  (let ((end (current-milliseconds)))
                    (list result (- end start))))))
            
            ;; Benchmark iteration utilities
            (define (run-iterations n proc)
              (define (loop count acc-time)
                (if (= count 0)
                    acc-time
                    (let* ((timing (time-execution proc))
                           (result (car timing))
                           (time (cadr timing)))
                      (loop (- count 1) (+ acc-time time)))))
              (loop n 0))
            
            ;; Memory measurement utilities (simulated)
            (define (measure-memory-usage thunk)
              (let ((initial-memory (random 1000000)))
                (let ((result (thunk)))
                  (let ((final-memory (+ initial-memory (random 100000))))
                    (list result initial-memory final-memory (- final-memory initial-memory))))))
            
            ;; Statistical utilities
            (define (average lst)
              (/ (fold + 0 lst) (length lst)))
            
            (define (minimum lst)
              (fold (lambda (acc x) (if (< x acc) x acc)) (car lst) (cdr lst)))
            
            (define (maximum lst)
              (fold (lambda (acc x) (if (> x acc) x acc)) (car lst) (cdr lst)))
            
            ;; Data generation utilities
            (define (generate-test-data type size)
              (case type
                ((numbers) (range 1 size))
                ((strings) (map (lambda (i) (string-append "string-" (number->string i))) (range 1 size)))
                ((lists) (map (lambda (i) (list i (* i 2) (* i 3))) (range 1 size)))
                ((mixed) (map (lambda (i) 
                               (case (modulo i 3)
                                 ((0) i)
                                 ((1) (string-append "item-" (number->string i)))
                                 ((2) (list i (* i 2)))))
                             (range 1 size)))
                (else (range 1 size))))
            
            ;; Benchmark suite runner
            (define (run-benchmark-suite suite-name setup benchmark cleanup iterations)
              (display "Running benchmark suite: ") (display suite-name) (newline)
              
              ;; Setup phase
              (when setup (setup))
              
              ;; Warm-up runs
              (run-iterations 10 benchmark)
              
              ;; Actual benchmark runs
              (let ((times (map (lambda (_)
                                  (let ((timing (time-execution benchmark)))
                                    (cadr timing)))
                                (range 1 iterations))))
                
                ;; Cleanup phase
                (when cleanup (cleanup))
                
                ;; Return statistics
                (list suite-name
                      (average times)
                      (minimum times)
                      (maximum times)
                      (length times))))
        "#;
        
        runtime.eval(benchmark_init, Some("scheme-benchmark-init")).await?;
        
        Ok(Self { runtime })
    }
    
    pub async fn run_benchmark(&self, suite: &SchemeBenchmarkSuite) -> Result<SchemeBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
        let benchmark_code = format!(
            r#"
                (run-benchmark-suite 
                  "{}"
                  (lambda () {})
                  (lambda () {})
                  {}
                  {})
            "#,
            suite.name,
            suite.setup_code,
            suite.benchmark_code,
            suite.cleanup_code.as_ref().map(|s| format!("(lambda () {})", s)).unwrap_or_else(|| "#f".to_string()),
            suite.iterations
        );
        
        let result = self.runtime.eval(&benchmark_code, Some("scheme-benchmark")).await?;
        
        Ok(SchemeBenchmarkResult {
            suite_name: suite.name.clone(),
            average_time: Duration::from_millis(100), // Placeholder
            min_time: Duration::from_millis(50),       // Placeholder  
            max_time: Duration::from_millis(200),      // Placeholder
            iterations: suite.iterations,
            complexity_level: suite.complexity_level.clone(),
        })
    }
    
    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.runtime.shutdown().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SchemeBenchmarkResult {
    pub suite_name: String,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub iterations: usize,
    pub complexity_level: String,
}

// ============================================================================
// SCHEME-BASED ARITHMETIC BENCHMARKS
// ============================================================================

fn bench_scheme_arithmetic_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("scheme_arithmetic_operations");
    group.measurement_time(Duration::from_secs(15));

    let arithmetic_suites = vec![
        SchemeBenchmarkSuite {
            name: "basic_addition".to_string(),
            setup_code: "(define test-numbers (generate-test-data 'numbers 100))".to_string(),
            benchmark_code: "(fold + 0 test-numbers)".to_string(),
            cleanup_code: None,
            iterations: 1000,
            complexity_level: "small".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "multiplication_chain".to_string(),
            setup_code: "(define test-numbers (generate-test-data 'numbers 50))".to_string(),
            benchmark_code: "(fold * 1 test-numbers)".to_string(),
            cleanup_code: None,
            iterations: 500,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "complex_arithmetic".to_string(),
            setup_code: "(define test-numbers (generate-test-data 'numbers 20))".to_string(),
            benchmark_code: "(fold (lambda (acc x) (+ acc (* x x) (/ x 2))) 0 test-numbers)".to_string(),
            cleanup_code: None,
            iterations: 200,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "gcd_computation".to_string(),
            setup_code: "(define test-pairs '((1071 462) (12345 6789) (999 888) (1234567 987654)))".to_string(),
            benchmark_code: "(map (lambda (pair) (gcd (car pair) (cadr pair))) test-pairs)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "large".to_string(),
        },
    ];

    for suite in arithmetic_suites {
        let benchmark_id = BenchmarkId::from_parameter(&suite.name);
        
        group.bench_with_input(benchmark_id, &suite, |b, suite| {
            b.to_async(&rt).iter(|| async move {
                let runner = SchemeBenchmarkRunner::new(BootstrapMode::Minimal).await.unwrap();
                let result = runner.run_benchmark(suite).await.unwrap();
                let _ = runner.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// SCHEME-BASED LIST OPERATION BENCHMARKS
// ============================================================================

fn bench_scheme_list_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("scheme_list_operations");
    group.measurement_time(Duration::from_secs(20));

    let list_suites = vec![
        SchemeBenchmarkSuite {
            name: "list_construction".to_string(),
            setup_code: "".to_string(),
            benchmark_code: "(generate-test-data 'lists 1000)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "list_mapping".to_string(),
            setup_code: "(define test-list (generate-test-data 'numbers 500))".to_string(),
            benchmark_code: "(map (lambda (x) (* x x x)) test-list)".to_string(),
            cleanup_code: None,
            iterations: 200,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "list_filtering".to_string(),
            setup_code: "(define test-list (generate-test-data 'numbers 1000))".to_string(),
            benchmark_code: "(filter (lambda (x) (and (> x 100) (< x 900))) test-list)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "list_folding".to_string(),
            setup_code: "(define test-list (generate-test-data 'numbers 1000))".to_string(),
            benchmark_code: "(fold (lambda (acc x) (+ acc (* x x))) 0 test-list)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "list_append_chain".to_string(),
            setup_code: "(define test-lists (map (lambda (i) (list i (* i 2) (* i 3))) (range 1 100)))".to_string(),
            benchmark_code: "(fold append '() test-lists)".to_string(),
            cleanup_code: None,
            iterations: 50,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "list_reversal".to_string(),
            setup_code: "(define test-list (generate-test-data 'numbers 2000))".to_string(),
            benchmark_code: "(reverse test-list)".to_string(),
            cleanup_code: None,
            iterations: 50,
            complexity_level: "large".to_string(),
        },
    ];

    for suite in list_suites {
        let benchmark_id = BenchmarkId::from_parameter(&suite.name);
        
        group.bench_with_input(benchmark_id, &suite, |b, suite| {
            b.to_async(&rt).iter(|| async move {
                let runner = SchemeBenchmarkRunner::new(BootstrapMode::Minimal).await.unwrap();
                let result = runner.run_benchmark(suite).await.unwrap();
                let _ = runner.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// SCHEME-BASED STRING OPERATION BENCHMARKS
// ============================================================================

fn bench_scheme_string_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("scheme_string_operations");
    group.measurement_time(Duration::from_secs(15));

    let string_suites = vec![
        SchemeBenchmarkSuite {
            name: "string_concatenation".to_string(),
            setup_code: "(define test-strings (generate-test-data 'strings 100))".to_string(),
            benchmark_code: "(fold string-append \"\" test-strings)".to_string(),
            cleanup_code: None,
            iterations: 200,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "string_comparison".to_string(),
            setup_code: "(define test-strings (generate-test-data 'strings 50)) (define compare-string \"string-25\")".to_string(),
            benchmark_code: "(filter (lambda (s) (string=? s compare-string)) test-strings)".to_string(),
            cleanup_code: None,
            iterations: 500,
            complexity_level: "small".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "string_case_conversion".to_string(),
            setup_code: "(define test-strings (generate-test-data 'strings 200))".to_string(),
            benchmark_code: "(map string-upcase (map string-downcase test-strings))".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "string_searching".to_string(),
            setup_code: r#"(define test-strings (map (lambda (i) (string-append "prefix-" (number->string i) "-suffix")) (range 1 100)))"#.to_string(),
            benchmark_code: r#"(filter (lambda (s) (string-contains s "50")) test-strings)"#.to_string(),
            cleanup_code: None,
            iterations: 200,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "complex_string_manipulation".to_string(),
            setup_code: r#"(define test-data (range 1 500))"#.to_string(),
            benchmark_code: r#"(map (lambda (i) (string-trim (string-replace (string-append "  item-" (number->string i) "  ") "item" "element"))) test-data)"#.to_string(),
            cleanup_code: None,
            iterations: 50,
            complexity_level: "large".to_string(),
        },
    ];

    for suite in string_suites {
        let benchmark_id = BenchmarkId::from_parameter(&suite.name);
        
        group.bench_with_input(benchmark_id, &suite, |b, suite| {
            b.to_async(&rt).iter(|| async move {
                let runner = SchemeBenchmarkRunner::new(BootstrapMode::Minimal).await.unwrap();
                let result = runner.run_benchmark(suite).await.unwrap();
                let _ = runner.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// SCHEME-BASED VECTOR OPERATION BENCHMARKS
// ============================================================================

fn bench_scheme_vector_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("scheme_vector_operations");
    group.measurement_time(Duration::from_secs(12));

    let vector_suites = vec![
        SchemeBenchmarkSuite {
            name: "vector_construction".to_string(),
            setup_code: "(define test-numbers (generate-test-data 'numbers 1000))".to_string(),
            benchmark_code: "(list->vector test-numbers)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "vector_access_pattern".to_string(),
            setup_code: "(define test-vector (list->vector (generate-test-data 'numbers 1000)))".to_string(),
            benchmark_code: "(map (lambda (i) (vector-ref test-vector i)) (range 0 (- (vector-length test-vector) 1)))".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "vector_mapping".to_string(),
            setup_code: "(define test-vector (list->vector (generate-test-data 'numbers 500)))".to_string(),
            benchmark_code: "(vector-map (lambda (x) (* x x x)) test-vector)".to_string(),
            cleanup_code: None,
            iterations: 200,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "vector_folding".to_string(),
            setup_code: "(define test-vector (list->vector (generate-test-data 'numbers 1000)))".to_string(),
            benchmark_code: "(vector-fold + 0 test-vector)".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "large".to_string(),
        },
    ];

    for suite in vector_suites {
        let benchmark_id = BenchmarkId::from_parameter(&suite.name);
        
        group.bench_with_input(benchmark_id, &suite, |b, suite| {
            b.to_async(&rt).iter(|| async move {
                let runner = SchemeBenchmarkRunner::new(BootstrapMode::Minimal).await.unwrap();
                let result = runner.run_benchmark(suite).await.unwrap();
                let _ = runner.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// SCHEME-BASED RECURSIVE OPERATION BENCHMARKS
// ============================================================================

fn bench_scheme_recursive_operations(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("scheme_recursive_operations");
    group.measurement_time(Duration::from_secs(20));

    let recursive_suites = vec![
        SchemeBenchmarkSuite {
            name: "factorial_computation".to_string(),
            setup_code: r#"(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))"#.to_string(),
            benchmark_code: "(map factorial (range 1 20))".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "fibonacci_sequence".to_string(),
            setup_code: r#"(define (fibonacci n) (if (<= n 1) n (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))"#.to_string(),
            benchmark_code: "(map fibonacci (range 1 15))".to_string(),
            cleanup_code: None,
            iterations: 10,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "tree_traversal".to_string(),
            setup_code: r#"
                (define (make-tree depth)
                  (if (= depth 0)
                      'leaf
                      (list 'node (make-tree (- depth 1)) (make-tree (- depth 1)))))
                (define (count-leaves tree)
                  (cond
                    ((eq? tree 'leaf) 1)
                    ((pair? tree) (+ (count-leaves (cadr tree)) (count-leaves (caddr tree))))
                    (else 0)))
                (define test-tree (make-tree 10))
            "#.to_string(),
            benchmark_code: "(count-leaves test-tree)".to_string(),
            cleanup_code: None,
            iterations: 50,
            complexity_level: "large".to_string(),
        },
        SchemeBenchmarkSuite {
            name: "mutual_recursion".to_string(),
            setup_code: r#"
                (define (even? n) (if (= n 0) #t (odd? (- n 1))))
                (define (odd? n) (if (= n 0) #f (even? (- n 1))))
            "#.to_string(),
            benchmark_code: "(map even? (range 1 100))".to_string(),
            cleanup_code: None,
            iterations: 100,
            complexity_level: "medium".to_string(),
        },
    ];

    for suite in recursive_suites {
        let benchmark_id = BenchmarkId::from_parameter(&suite.name);
        
        group.bench_with_input(benchmark_id, &suite, |b, suite| {
            b.to_async(&rt).iter(|| async move {
                let runner = SchemeBenchmarkRunner::new(BootstrapMode::Minimal).await.unwrap();
                let result = runner.run_benchmark(suite).await.unwrap();
                let _ = runner.shutdown().await;
                result
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    scheme_arithmetic_benches,
    bench_scheme_arithmetic_operations
);

criterion_group!(
    scheme_list_benches,
    bench_scheme_list_operations
);

criterion_group!(
    scheme_string_benches,
    bench_scheme_string_operations
);

criterion_group!(
    scheme_vector_benches,
    bench_scheme_vector_operations
);

criterion_group!(
    scheme_recursive_benches,
    bench_scheme_recursive_operations
);

criterion_main!(
    scheme_arithmetic_benches,
    scheme_list_benches,
    scheme_string_benches,
    scheme_vector_benches,
    scheme_recursive_benches
);