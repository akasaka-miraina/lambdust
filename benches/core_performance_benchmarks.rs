//! Core Performance Benchmarks
//!
//! Comprehensive benchmarks for the core components of the Lambdust language implementation:
//! - Lexer performance (tokenization speed)
//! - Parser performance (AST construction efficiency)
//! - Evaluator performance (runtime execution speed)
//! - Memory usage patterns
//!
//! These benchmarks establish baseline performance metrics and identify optimization opportunities.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{Lambdust, Lexer, Parser};
use std::time::Duration;

// ============================================================================
// LEXER BENCHMARKS
// ============================================================================

fn bench_lexer_simple_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_simple_expressions");
    group.measurement_time(Duration::from_secs(10));
    
    let test_cases = vec![
        ("arithmetic", "(+ 1 2 3)"),
        ("function_call", "((lambda (x) (* x x)) 42)"),
        ("nested_lists", "(list (list 1 2) (list 3 4) (list 5 6))"),
        ("string_literals", "\"hello world\" \"with\\nescapes\""),
        ("identifiers", "define lambda if cond case let let* letrec"),
        ("numbers", "42 3.14 22/7 1+2i -5.2e-10"),
        ("mixed", "(define (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))"),
    ];
    
    for (name, source) in test_cases {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &source, |b, &src| {
            b.iter(|| {
                let mut lexer = Lexer::new(src, Some("benchmark"));
                lexer.tokenize().unwrap()
            });
        });
    }
    
    group.finish();
}

fn bench_lexer_large_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_large_input");
    group.measurement_time(Duration::from_secs(15));
    
    let sizes = vec![1_000, 5_000, 10_000, 50_000];
    
    for &size in &sizes {
        // Generate large input with mixed token types
        let source = generate_large_program(size);
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{}_chars", size)), &source, |b, src| {
            b.iter(|| {
                let mut lexer = Lexer::new(src, Some("benchmark"));
                lexer.tokenize().unwrap()
            });
        });
    }
    
    group.finish();
}

fn bench_lexer_token_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_token_types");
    group.measurement_time(Duration::from_secs(8));
    
    let token_tests = vec![
        ("integers", "1 2 3 42 -17 +123"),
        ("reals", "3.14 -2.71 1.0e10 -5.2e-3"),
        ("rationals", "1/2 3/4 -22/7 123/456"),
        ("complex", "1+2i -3+4i 5-6i +i -i"),
        ("strings", "\"hello\" \"world\\n\" \"unicode: \\x41;\""),
        ("identifiers", "foo bar baz-qux list->vector string-length"),
        ("keywords", "#:key #:value #:test-keyword"),
        ("characters", "#\\a #\\space #\\newline #\\x41"),
        ("booleans", "#t #f #true #false"),
    ];
    
    for (name, source) in token_tests {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &source, |b, &src| {
            b.iter(|| {
                let mut lexer = Lexer::new(src, Some("benchmark"));
                lexer.tokenize().unwrap()
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// PARSER BENCHMARKS
// ============================================================================

fn bench_parser_simple_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_simple_expressions");
    group.measurement_time(Duration::from_secs(10));
    
    let test_cases = vec![
        ("literal", "42"),
        ("identifier", "variable-name"),
        ("quote", "'(a b c)"),
        ("application", "(+ 1 2 3)"),
        ("lambda", "(lambda (x y) (+ x y))"),
        ("if_expression", "(if (> x 0) x (- x))"),
        ("define", "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))"),
        ("nested", "((lambda (f) (f (f 42))) (lambda (x) (* x 2)))"),
    ];
    
    for (name, source) in test_cases {
        // Pre-tokenize for fair comparison
        let mut lexer = Lexer::new(source, Some("benchmark"));
        let tokens = lexer.tokenize().unwrap();
        
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &tokens, |b, toks| {
            b.iter(|| {
                let mut parser = Parser::new(toks.clone());
                parser.parse().unwrap()
            });
        });
    }
    
    group.finish();
}

fn bench_parser_complex_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_complex_structures");
    group.measurement_time(Duration::from_secs(15));
    
    let depths = vec![5, 10, 20, 50];
    
    for &depth in &depths {
        let source = generate_nested_expression(depth);
        let mut lexer = Lexer::new(&source, Some("benchmark"));
        let tokens = lexer.tokenize().unwrap();
        
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("depth_{}", depth)), &tokens, |b, toks| {
            b.iter(|| {
                let mut parser = Parser::new(toks.clone());
                parser.parse().unwrap()
            });
        });
    }
    
    group.finish();
}

fn bench_parser_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_error_recovery");
    group.measurement_time(Duration::from_secs(12));
    
    let error_cases = vec![
        ("missing_paren", "(+ 1 2"),
        ("extra_paren", "(+ 1 2))"),
        ("invalid_syntax", "(define (42) x)"),
        ("mixed_errors", "(+ 1 2 } (define x 42) )"),
    ];
    
    for (name, source) in error_cases {
        let mut lexer = Lexer::new(source, Some("benchmark"));
        let tokens = lexer.tokenize().unwrap_or_default();
        
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &tokens, |b, toks| {
            b.iter(|| {
                let mut parser = Parser::new(toks.clone());
                let _ = parser.parse(); // Allow errors
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// END-TO-END EVALUATION BENCHMARKS
// ============================================================================

fn bench_evaluation_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_simple");
    group.measurement_time(Duration::from_secs(8));
    
    let test_cases = vec![
        ("arithmetic", "(+ 1 2 3)"),
        ("function_call", "((lambda (x) (* x x)) 42)"),
        ("conditional", "(if #t 1 2)"),
        ("list_operations", "(cons 1 (cons 2 (cons 3 ())))"),
        ("recursion_small", "(define (fact n) (if (= n 0) 1 (* n (fact (- n 1))))) (fact 5)"),
    ];
    
    for (name, source) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &source, |b, &src| {
            b.iter(|| {
                let mut lambdust = Lambdust::new();
                // Note: This will fail until the runtime is fully implemented
                // For now, just measure parsing + setup overhead
                let _ = lambdust.eval(src, Some("benchmark"));
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// MEMORY ALLOCATION BENCHMARKS
// ============================================================================

fn bench_memory_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocations");
    group.measurement_time(Duration::from_secs(10));
    
    let allocation_patterns: Vec<(&str, usize, fn(usize) -> Vec<String>)> = vec![
        ("small_objects", 1000, generate_small_objects),
        ("large_objects", 100, generate_large_objects),
        ("mixed_objects", 500, generate_mixed_objects),
    ];
    
    for (name, count, generator) in allocation_patterns {
        let sources = generator(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &sources, |b, srcs| {
            b.iter(|| {
                for source in srcs {
                    let mut lexer = Lexer::new(source, Some("benchmark"));
                    let tokens = lexer.tokenize().unwrap();
                    let mut parser = Parser::new(tokens);
                    let _ = parser.parse().unwrap();
                }
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn generate_large_program(target_chars: usize) -> String {
    let mut result = String::new();
    let mut counter = 0;
    
    while result.len() < target_chars {
        let line = match counter % 8 {
            0 => format!("(+ {} {})", counter, counter + 1),
            1 => format!("(* {} {})", counter, counter + 1),
            2 => format!("(define var-{} {})", counter, counter * 2),
            3 => format!("(if (> {} {}) {} {})", counter, counter + 1, counter * 2, counter + 3),
            4 => format!("(lambda (x{}) (+ x{} {}))", counter, counter, counter * 3),
            5 => format!("\"string-literal-{}\"", counter),
            6 => format!("'(symbol-{})", counter),
            7 => format!("#:keyword-{}", counter),
            _ => unreachable!(),
        };
        result.push_str(&line);
        result.push('\n');
        counter += 1;
    }
    
    result
}

fn generate_nested_expression(depth: usize) -> String {
    if depth == 0 {
        "42".to_string()
    } else {
        format!("(+ {} {})", generate_nested_expression(depth - 1), depth)
    }
}

fn generate_small_objects(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| match i % 4 {
            0 => format!("{}", i),
            1 => format!("var-{}", i),
            2 => format!("\"str-{}\"", i),
            3 => format!("(+ {} {})", i, i + 1),
            _ => unreachable!(),
        })
        .collect()
}

fn generate_large_objects(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let elements: Vec<String> = (0..50).map(|j| format!("{}", i * 50 + j)).collect();
            format!("(list {})", elements.join(" "))
        })
        .collect()
}

fn generate_mixed_objects(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| match i % 6 {
            0 => format!("{}", i),
            1 => format!("(+ {} {})", i, i + 1),
            2 => format!("(lambda (x) (* x {}))", i),
            3 => format!("(list {})", (0..5).map(|j| (i + j).to_string()).collect::<Vec<_>>().join(" ")),
            4 => format!("\"string-with-content-{}\"", i),
            5 => format!("(if (> {} 10) {} {})", i, i * 2, i / 2),
            _ => unreachable!(),
        })
        .collect()
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    lexer_benches,
    bench_lexer_simple_expressions,
    bench_lexer_large_input,
    bench_lexer_token_types
);

criterion_group!(
    parser_benches,
    bench_parser_simple_expressions,
    bench_parser_complex_structures,
    bench_parser_error_recovery
);

criterion_group!(
    evaluation_benches,
    bench_evaluation_simple
);

criterion_group!(
    memory_benches,
    bench_memory_allocations
);

criterion_main!(
    lexer_benches,
    parser_benches,
    evaluation_benches,
    memory_benches
);