;; R7RS Compliance Verification Test for Container System
;; Tests core R7RS features that depend on the container system

;; Basic data structures that use containers internally
(define test-list '(1 2 3 4 5))
(define test-vector #(1 2 3 4 5))

;; Symbol operations (uses ordered sets for symbol tables)
(define sym1 'test-symbol)
(define sym2 'test-symbol)

;; Basic environment and binding tests
(let ((x 42))
  (define inner-test 
    (lambda () x)))

;; Lexical scoping test (relies on environment chains)
(define outer-var 100)
(define scoping-test
  (lambda ()
    (let ((outer-var 200))
      (+ outer-var 1))))

;; Simple set operations if available
(define test-set-basic
  (lambda ()
    (let ((s1 (make-ordered-set))
          (s2 (make-ordered-set)))
      (ordered-set-insert! s1 1)
      (ordered-set-insert! s1 2)
      (ordered-set-insert! s2 2)
      (ordered-set-insert! s2 3)
      (list
        (ordered-set-size s1)
        (ordered-set-size s2)
        (ordered-set-contains? s1 1)
        (ordered-set-contains? s2 1)))))

;; Test script output
(display "=== R7RS Container Compliance Verification ===")
(newline)

(display "1. Symbol identity test: ")
(display (eq? sym1 sym2))
(newline)

(display "2. Lexical scoping test: ")
(display (= (scoping-test) 201))
(newline)

(display "3. List operations: ")
(display (= (length test-list) 5))
(newline)

(display "4. Vector operations: ")
(display (= (vector-length test-vector) 5))
(newline)

(display "5. Closure capture test: ")
(display (= (inner-test) 42))
(newline)

(display "6. Basic container operations: ")
(display (if (procedure? make-ordered-set)
             (equal? (test-set-basic) '(2 2 #t #f))
             "Containers not available"))
(newline)

(display "=== Verification Complete ===")
(newline)