use lambdust::Interpreter;
use std::time::Instant;

fn main() {
    let mut interpreter = Interpreter::new();

    // 軽量なベンチマークスイート（スタックオーバーフローを避ける）
    let benchmarks = vec![
        ("Simple arithmetic", "(+ 1 2 3 4 5)"),
        ("Nested arithmetic", "(+ (* 2 3) (- 10 5) (/ 20 4))"),
        (
            "Simple recursion (factorial 5)",
            r#"
(begin
  (define (factorial n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
  (factorial 5))"#,
        ),
        (
            "Small Fibonacci (10)",
            r#"
(begin
  (define (fib n)
    (if (< n 2)
        n
        (+ (fib (- n 1)) (fib (- n 2)))))
  (fib 10))"#,
        ),
        (
            "List operations",
            r#"
(begin
  (define lst '(1 2 3 4 5))
  (define (sum-list l)
    (if (null? l)
        0
        (+ (car l) (sum-list (cdr l)))))
  (sum-list lst))"#,
        ),
        (
            "Small list creation",
            r#"
(begin
  (define (make-range n)
    (if (<= n 0)
        '()
        (cons n (make-range (- n 1)))))
  (length (make-range 10)))"#,
        ),
        (
            "Lambda expressions",
            r#"
(begin
  (define make-adder
    (lambda (x)
      (lambda (y) (+ x y))))
  (define add5 (make-adder 5))
  (add5 10))"#,
        ),
        (
            "Multiple values",
            r#"
(call-with-values
  (lambda () (values 1 2 3))
  (lambda (a b c) (+ a b c)))"#,
        ),
        (
            "Small continuation stack",
            r#"
(begin
  (define (deep-call n)
    (if (= n 0)
        42
        (+ 1 (deep-call (- n 1)))))
  (deep-call 10))"#,
        ),
    ];

    println!("lambdust CPS Evaluator Performance Benchmark (Safe)");
    println!("==================================================");

    for (name, code) in benchmarks {
        print!("{:<32} ... ", name);

        // ウォームアップ
        for _ in 0..3 {
            let _ = interpreter.eval(code);
        }

        // 実際の測定
        let start = Instant::now();
        let iterations = 1000;

        let mut success_count = 0;
        for _ in 0..iterations {
            match interpreter.eval(code) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    println!("ERROR: {}", e);
                    break;
                }
            }
        }

        if success_count > 0 {
            let elapsed = start.elapsed();
            let per_iteration = elapsed.as_micros() as f64 / success_count as f64;

            println!(
                "{:>8.2} μs/op ({} iterations)",
                per_iteration, success_count
            );
        }
    }

    // メモリ使用量のテスト
    println!("\nMemory allocation tests:");

    // Box<Continuation>の深いネストのテスト
    let complex_expr = r#"
(begin
  (define (test-continuations n)
    (if (= n 0)
        42
        (+ 1 (test-continuations (- n 1)))))
  (test-continuations 5))"#;

    print!("Continuation nesting test      ... ");
    let start = Instant::now();
    match interpreter.eval(complex_expr) {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("{:>8.2} μs", elapsed.as_micros() as f64);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }

    // Environment cloning のテスト
    let env_test = r#"
(begin
  (define x 10)
  (define (make-closure y)
    (lambda (z) (+ x y z)))
  (define f (make-closure 20))
  (f 30))"#;

    print!("Environment cloning test       ... ");
    let start = Instant::now();
    match interpreter.eval(env_test) {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("{:>8.2} μs", elapsed.as_micros() as f64);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}
