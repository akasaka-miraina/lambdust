use lambdust::Interpreter;
use std::time::Instant;

fn main() {
    let mut interpreter = Interpreter::new();

    // ベンチマークスイート
    let benchmarks = vec![
        ("Simple arithmetic", "(+ 1 2 3 4 5)"),
        ("Nested arithmetic", "(+ (* 2 3) (- 10 5) (/ 20 4))"),
        (
            "Deep recursion (factorial 10)",
            r#"
(begin
  (define (factorial n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
  (factorial 10))"#,
        ),
        (
            "Fibonacci 20",
            r#"
(begin
  (define (fib n)
    (if (< n 2)
        n
        (+ (fib (- n 1)) (fib (- n 2)))))
  (fib 20))"#,
        ),
        (
            "List operations",
            r#"
(begin
  (define lst '(1 2 3 4 5 6 7 8 9 10))
  (define (sum-list l)
    (if (null? l)
        0
        (+ (car l) (sum-list (cdr l)))))
  (sum-list lst))"#,
        ),
        (
            "Large list creation",
            r#"
(begin
  (define (make-range n)
    (if (<= n 0)
        '()
        (cons n (make-range (- n 1)))))
  (length (make-range 1000)))"#,
        ),
        (
            "Higher-order functions",
            r#"
(begin
  (define (map f lst)
    (if (null? lst)
        '()
        (cons (f (car lst)) (map f (cdr lst)))))
  (define (square x) (* x x))
  (map square '(1 2 3 4 5 6 7 8 9 10)))"#,
        ),
        (
            "Complex lambda expressions",
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
  (lambda () (values 1 2 3 4 5))
  (lambda (a b c d e) (+ a b c d e)))"#,
        ),
        (
            "Deep continuation stack",
            r#"
(begin
  (define (deep-call n)
    (if (= n 0)
        42
        (+ 1 (deep-call (- n 1)))))
  (deep-call 100))"#,
        ),
    ];

    println!("lambdust CPS Evaluator Performance Benchmark");
    println!("===========================================");

    for (name, code) in benchmarks {
        print!("{:<30} ... ", name);

        // ウォームアップ
        for _ in 0..3 {
            let _ = interpreter.eval(code);
        }

        // 実際の測定
        let start = Instant::now();
        let iterations = match name {
            "Fibonacci 20" => 5, // 重い処理は少なめ
            "Large list creation" => 10,
            "Deep continuation stack" => 50,
            _ => 100,
        };

        for _ in 0..iterations {
            match interpreter.eval(code) {
                Ok(_) => {}
                Err(e) => {
                    println!("ERROR: {}", e);
                    continue;
                }
            }
        }

        let elapsed = start.elapsed();
        let per_iteration = elapsed.as_micros() as f64 / iterations as f64;

        println!("{:>8.2} μs/op ({} iterations)", per_iteration, iterations);
    }
}
