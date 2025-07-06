;; Lambdust REPL Advanced Features Demo
;;
;; This file demonstrates the enhanced REPL features including:
;; - Tab completion
;; - Syntax highlighting
;; - Debug mode
;; - Error handling

;; Basic arithmetic with syntax highlighting
(define (factorial n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

;; Test the factorial function
(factorial 5)  ; Should return 120

;; String operations with completion
(define greeting "Hello, World!")
(string-length greeting)
(substring greeting 0 5)

;; List operations demonstrating completion
(define numbers '(1 2 3 4 5))
(map (lambda (x) (* x x)) numbers)  ; Square each number
(filter (lambda (x) (> x 2)) numbers)  ; Filter numbers > 2

;; Higher-order functions
(define (compose f g)
  (lambda (x) (f (g x))))

(define square (lambda (x) (* x x)))
(define double (lambda (x) (* 2 x)))
(define square-then-double (compose double square))

(square-then-double 3)  ; Should return 18

;; Record types (SRFI 9)
(define-record-type person
  (make-person name age)
  person?
  (name person-name)
  (age person-age))

(define alice (make-person "Alice" 30))
(person-name alice)
(person? alice)

;; Hash tables (SRFI 69)
(define contacts (make-hash-table))
(hash-table-set! contacts "alice" alice)
(hash-table-ref contacts "alice")
(hash-table-size contacts)

;; Error handling demonstration
(define (safe-divide x y)
  (if (= y 0)
      (raise "Division by zero!")
      (/ x y)))

;; Test error handling
(with-exception-handler
  (lambda (e) (display "Caught error: ") (display e) (newline))
  (lambda () (safe-divide 10 0)))

;; Continuations example (for debugging)
(define (test-call/cc)
  (+ 1 (call/cc (lambda (k) (+ 2 (k 3))))))

(test-call/cc)  ; Should return 4

;; Complex data structures
(define nested-list '((1 2) (3 4) (5 6)))
(map car nested-list)  ; Extract first elements

;; Vector operations
(define vec (vector 'a 'b 'c 'd))
(vector-ref vec 1)
(vector-set! vec 2 'x)
vec

;; String libraries (SRFI 13) demonstration
(string-prefix? "Hello" "Hello, World!")
(string-contains "Hello, World!" "World")
(string-take "Hello, World!" 5)

;; Lazy evaluation (SRFI 45)
(define lazy-computation
  (delay (begin
           (display "Computing...")
           (newline)
           42)))

(force lazy-computation)  ; Will display message and return 42

;; Debug mode test - set breakpoint here
(define (debug-test x)
  (let ((y (* x 2)))
    (let ((z (+ y 1)))
      (* z z))))

(debug-test 3)  ; Use (debug on) and (break) to debug this

;; Multi-line expression for testing completion
(define complicated-function
  (lambda (lst)
    (fold (lambda (acc x)
            (if (number? x)
                (+ acc x)
                acc))
          0
          lst)))

(complicated-function '(1 "hello" 2 'symbol 3))  ; Should return 6

;; Test tab completion with partial inputs:
;; Try typing these in REPL:
;; - (str<TAB>    -> should complete string functions
;; - (vec<TAB>    -> should complete vector functions
;; - (def<TAB>    -> should complete define, etc.
;; - (car<TAB>    -> should complete car, etc.

;; Test debug commands:
;; 1. (debug on)     - Enable debug mode
;; 2. (break)        - Set breakpoint
;; 3. Evaluate any expression to trigger breakpoint
;; 4. (backtrace)    - Show call stack
;; 5. (continue)     - Continue execution
;; 6. (debug off)    - Disable debug mode