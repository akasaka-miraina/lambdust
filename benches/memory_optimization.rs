use criterion::{Criterion, criterion_group, criterion_main};
use lambdust::Interpreter;

fn memory_pool_allocation_test(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    // Test memory pool allocation patterns
    let allocation_code = r#"
    (define (create-many-lists n)
      (if (<= n 0)
          '()
          (cons (list 1 2 3 4 5) (create-many-lists (- n 1)))))
    "#;

    interpreter.eval(allocation_code).unwrap();

    c.bench_function("memory_pool_allocations", |b| {
        b.iter(|| {
            // Allocate many small lists to test memory pool efficiency
            interpreter.eval("(create-many-lists 20)").unwrap()
        })
    });
}

fn memory_intensive_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let setup_code = r#"
    (define (create-vectors n)
      (if (<= n 0)
          (vector)
          (vector (create-vectors (- n 1)) n (+ n 1) (* n 2))))
          
    (define (process-data data)
      (map (lambda (x) (* x x)) data))
    "#;

    interpreter.eval(setup_code).unwrap();

    c.bench_function("memory_intensive_vectors", |b| {
        b.iter(|| interpreter.eval("(create-vectors 8)").unwrap())
    });

    c.bench_function("memory_intensive_processing", |b| {
        b.iter(|| {
            interpreter
                .eval("(process-data (list 1 2 3 4 5 6 7 8 9 10))")
                .unwrap()
        })
    });
}

fn garbage_collection_stress_test(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let gc_stress_code = r#"
    (define (allocate-and-discard n)
      (if (<= n 0)
          'done
          (begin
            (vector 1 2 3 4 5)
            (list 'a 'b 'c 'd 'e)
            (allocate-and-discard (- n 1)))))
    "#;

    interpreter.eval(gc_stress_code).unwrap();

    c.bench_function("gc_stress_test", |b| {
        b.iter(|| interpreter.eval("(allocate-and-discard 15)").unwrap())
    });
}

fn continuation_memory_patterns(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let continuation_code = r#"
    (define (nested-calls n acc)
      (if (<= n 0)
          acc
          (nested-calls (- n 1) (+ acc n))))
          
    (define (tail-recursive-sum lst acc)
      (if (null? lst)
          acc
          (tail-recursive-sum (cdr lst) (+ acc (car lst)))))
    "#;

    interpreter.eval(continuation_code).unwrap();

    c.bench_function("continuation_memory_nested", |b| {
        b.iter(|| interpreter.eval("(nested-calls 25 0)").unwrap())
    });

    c.bench_function("continuation_memory_tail_recursive", |b| {
        b.iter(|| {
            interpreter
                .eval("(tail-recursive-sum (list 1 2 3 4 5 6 7 8 9 10) 0)")
                .unwrap()
        })
    });
}

fn string_memory_operations(c: &mut Criterion) {
    let mut interpreter = Interpreter::new();

    let string_code = r#"
    (define (create-strings n)
      (if (<= n 0)
          ""
          (string-append "test" (number->string n) (create-strings (- n 1)))))
          
    (define test-string "hello world scheme programming language")
    "#;

    interpreter.eval(string_code).unwrap();

    c.bench_function("string_memory_operations", |b| {
        b.iter(|| interpreter.eval("(create-strings 8)").unwrap())
    });

    c.bench_function("string_memory_length_ops", |b| {
        b.iter(|| interpreter.eval("(string-length test-string)").unwrap())
    });
}

fn raii_vs_gc_comparison(c: &mut Criterion) {
    // Test different allocation patterns that benefit from either RAII or GC
    let mut interpreter = Interpreter::new();

    let mixed_allocation_code = r#"
    (define (mixed-allocations n)
      (if (<= n 0)
          '()
          (cons 
            (vector (list n) (+ n 1) (* n 2))
            (mixed-allocations (- n 1)))))
    "#;

    interpreter.eval(mixed_allocation_code).unwrap();

    c.bench_function("mixed_allocation_patterns", |b| {
        b.iter(|| interpreter.eval("(mixed-allocations 12)").unwrap())
    });
}

criterion_group!(
    memory_benches,
    memory_pool_allocation_test,
    memory_intensive_operations,
    garbage_collection_stress_test,
    continuation_memory_patterns,
    string_memory_operations,
    raii_vs_gc_comparison
);

criterion_main!(memory_benches);
