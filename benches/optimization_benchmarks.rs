//! Optimization Benchmarks
//!
//! Benchmarks comparing optimized vs regular implementations to measure
//! the performance improvements from string interning, memory pooling,
//! and other optimizations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::lexer::{Lexer, OptimizedLexer};
use lambdust::utils::{StringInterner, memory_pool::global_pools};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// LEXER OPTIMIZATION BENCHMARKS
// ============================================================================

fn bench_lexer_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_optimization_comparison");
    group.measurement_time(Duration::from_secs(10));
    
    let test_cases = vec![
        ("simple", "(+ 1 2 3)"),
        ("identifiers", "define lambda if cond case let let* letrec"),
        ("repeated_identifiers", "foo bar foo baz foo bar"),
        ("complex_program", r#"
            (define (factorial n)
              (if (= n 0)
                  1
                  (* n (factorial (- n 1)))))
            
            (define (fibonacci n)
              (if (<= n 1)
                  n
                  (+ (fibonacci (- n 1))
                     (fibonacci (- n 2)))))
                     
            (define pi 3.14159)
            (define e 2.71828)
            
            (factorial 10)
            (fibonacci 10)
        "#),
        ("large_repetitive", {
            let mut s = String::new();
            for i in 0..100 {
                s.push_str(&format!("(define var-{} {})\n", i % 10, i));
            }
            s
        }.leak()), // Leak to get &'static str
    ];
    
    for (name, source) in test_cases {
        group.throughput(Throughput::Bytes(source.len() as u64));
        
        // Benchmark regular lexer
        group.bench_with_input(
            BenchmarkId::new("regular", name), 
            &source, 
            |b, &src| {
                b.iter(|| {
                    let mut lexer = Lexer::new(src, Some("benchmark"));
                    lexer.tokenize().unwrap()
                });
            }
        );
        
        // Benchmark optimized lexer
        group.bench_with_input(
            BenchmarkId::new("optimized", name), 
            &source, 
            |b, &src| {
                b.iter(|| {
                    let mut lexer = OptimizedLexer::new(src, Some("benchmark"));
                    lexer.tokenize().unwrap()
                });
            }
        );
    }
    
    group.finish();
}

fn bench_string_interning_benefits(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning_benefits");
    group.measurement_time(Duration::from_secs(8));
    
    // Test with different amounts of repetition
    let repetition_counts = vec![10, 50, 100, 500];
    
    for &count in &repetition_counts {
        let source = generate_repetitive_source(count);
        group.throughput(Throughput::Elements(count as u64));
        
        // Without shared interner (each lexer gets its own)
        group.bench_with_input(
            BenchmarkId::new("no_sharing", count),
            &source,
            |b, src| {
                b.iter(|| {
                    let mut lexer = OptimizedLexer::new(src, Some("benchmark"));
                    lexer.tokenize_optimized().unwrap()
                });
            }
        );
        
        // With shared interner across multiple lexer instances
        group.bench_with_input(
            BenchmarkId::new("shared_interner", count),
            &source,
            |b, src| {
                let shared_interner = Arc::new(StringInterner::new());
                b.iter(|| {
                    let mut lexer = OptimizedLexer::with_interner(
                        src, 
                        Some("benchmark"), 
                        shared_interner.clone()
                    );
                    lexer.tokenize_optimized().unwrap()
                });
            }
        );
    }
    
    group.finish();
}

fn bench_memory_pool_benefits(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool_benefits");
    group.measurement_time(Duration::from_secs(8));
    
    let _source = generate_large_program(1000);
    
    // Benchmark without using pools (direct Vec allocation)
    group.bench_function("direct_allocation", |b| {
        b.iter(|| {
            let mut tokens = Vec::new(); // Direct allocation each time
            tokens.reserve(100);
            for i in 0..100 {
                tokens.push(format!("token_{}", i));
            }
            tokens
        });
    });
    
    // Benchmark with memory pools
    group.bench_function("pooled_allocation", |b| {
        b.iter(|| {
            let mut tokens = global_pools::get_string_vec(); // From pool
            for i in 0..100 {
                tokens.push(format!("token_{}", i));
            }
            tokens // Automatically returned to pool on drop
        });
    });
    
    group.finish();
}

// ============================================================================
// ALGORITHM OPTIMIZATION BENCHMARKS
// ============================================================================

fn bench_parsing_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_algorithms");
    group.measurement_time(Duration::from_secs(10));
    
    // Test different nesting depths to identify O(n²) issues
    let depths = vec![10, 20, 50, 100];
    
    for &depth in &depths {
        let nested_source = generate_deeply_nested(depth);
        group.throughput(Throughput::Elements(depth as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}", depth)),
            &nested_source,
            |b, src| {
                b.iter(|| {
                    let mut lexer = OptimizedLexer::new(src, Some("benchmark"));
                    lexer.tokenize().unwrap()
                });
            }
        );
    }
    
    group.finish();
}

// ============================================================================
// COMPREHENSIVE OPTIMIZATION ANALYSIS
// ============================================================================

fn bench_optimization_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_analysis");
    group.measurement_time(Duration::from_secs(15));
    
    let test_program = r#"
        ; This is a comprehensive test program
        (define (quicksort lst)
          (if (null? lst)
              '()
              (let ((pivot (car lst))
                    (rest (cdr lst)))
                (append (quicksort (filter (lambda (x) (< x pivot)) rest))
                        (list pivot)
                        (quicksort (filter (lambda (x) (>= x pivot)) rest))))))
        
        (define (fibonacci n)
          (cond
            ((= n 0) 0)
            ((= n 1) 1)
            (else (+ (fibonacci (- n 1))
                     (fibonacci (- n 2))))))
        
        (define test-data (list 5 2 8 1 9 3))
        (define sorted-data (quicksort test-data))
        (define fib-10 (fibonacci 10))
        
        (display "Sorted: ") (display sorted-data) (newline)
        (display "Fibonacci 10: ") (display fib-10) (newline)
    "#;
    
    group.bench_function("baseline_regular", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(test_program, Some("test"));
            lexer.tokenize().unwrap()
        });
    });
    
    group.bench_function("all_optimizations", |b| {
        let shared_interner = Arc::new(StringInterner::new());
        b.iter(|| {
            let mut lexer = OptimizedLexer::with_interner(
                test_program, 
                Some("test"), 
                shared_interner.clone()
            );
            lexer.tokenize_optimized().unwrap()
        });
    });
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn generate_repetitive_source(count: usize) -> String {
    let mut source = String::new();
    let identifiers = vec!["define", "lambda", "if", "cond", "let", "car", "cdr", "cons"];
    
    for i in 0..count {
        let ident = identifiers[i % identifiers.len()];
        source.push_str(&format!("({} var-{} {})\n", ident, i, i * 2));
    }
    
    source
}

fn generate_large_program(target_lines: usize) -> String {
    let mut source = String::new();
    
    for i in 0..target_lines {
        let line = match i % 5 {
            0 => format!("(+ {} {})", i, i + 1),
            1 => format!("(define var-{} {})", i, i * 2),
            2 => format!("(if (> {} 0) {} 0)", i, i + 1),
            3 => format!("(lambda (x) (+ x {}))", i),
            4 => format!("(cons {} (cons {} ()))", i, i + 1),
            _ => unreachable!(),
        };
        source.push_str(&line);
        source.push('\n');
    }
    
    source
}

fn generate_deeply_nested(depth: usize) -> String {
    let mut source = String::new();
    
    // Create deeply nested expressions
    for _ in 0..depth {
        source.push('(');
    }
    source.push_str("+ 1 2");
    for _ in 0..depth {
        source.push(')');
    }
    
    source
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    optimization_benches,
    bench_lexer_comparison,
    bench_string_interning_benefits,
    bench_memory_pool_benefits
);

criterion_group!(
    algorithm_benches,
    bench_parsing_algorithms
);

criterion_group!(
    analysis_benches,
    bench_optimization_analysis
);

criterion_main!(
    optimization_benches,
    algorithm_benches,
    analysis_benches
);