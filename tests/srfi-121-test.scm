;;; Test suite for SRFI-121: Generators
;;;
;;; This test suite verifies the complete SRFI-121 implementation
;;; including constructors, transformers, consumers, and accumulators.

(import (srfi 121)
        (scheme base)
        (scheme write))

;;; Helper test framework
(define test-count 0)
(define fail-count 0)

(define (test-equal description expected actual)
  (set! test-count (+ test-count 1))
  (if (equal? expected actual)
      (begin
        (display "✓ ")
        (display description)
        (newline))
      (begin
        (set! fail-count (+ fail-count 1))
        (display "✗ ")
        (display description)
        (display " - Expected: ")
        (write expected)
        (display ", Got: ")
        (write actual)
        (newline))))

(define (test-true description actual)
  (test-equal description #t actual))

(define (test-false description actual)
  (test-equal description #f actual))

;;; Test Phase 1 primitives (Rust-based)
(display "Testing Phase 1 primitives...\n")

;; Test generator predicate
(let ((gen (generator 1 2 3)))
  (test-true "generator? on generator" (%generator? gen))
  (test-false "generator? on non-generator" (%generator? 42)))

;; Test basic generator operations
(let ((gen (generator 1 2 3)))
  (test-equal "First value" 1 (%generator-next gen))
  (test-equal "Second value" 2 (%generator-next gen))
  (test-equal "Third value" 3 (%generator-next gen))
  (test-true "Exhausted after all values" (%generator-exhausted? gen)))

;; Test range generator
(let ((gen (make-range-generator 0 3)))
  (test-equal "Range 0" 0.0 (%generator-next gen))
  (test-equal "Range 1" 1.0 (%generator-next gen))
  (test-equal "Range 2" 2.0 (%generator-next gen)))

;; Test iota generator
(let ((gen (make-iota-generator 3 5 2)))
  (test-equal "Iota first" 5 (%generator-next gen))
  (test-equal "Iota second" 7 (%generator-next gen))
  (test-equal "Iota third" 9 (%generator-next gen)))

;; Test list->generator
(let ((gen (list->generator '(a b c))))
  (test-equal "List gen first" 'a (%generator-next gen))
  (test-equal "List gen second" 'b (%generator-next gen))
  (test-equal "List gen third" 'c (%generator-next gen)))

;;; Test Phase 2-4 Scheme implementations
(display "Testing Phase 2-4 constructors...\n")

;; Test vector->generator
(let ((gen (vector->generator #(x y z))))
  (test-equal "Vector gen first" 'x (%generator-next gen))
  (test-equal "Vector gen second" 'y (%generator-next gen))
  (test-equal "Vector gen third" 'z (%generator-next gen)))

;; Test string->generator
(let ((gen (string->generator "abc")))
  (test-equal "String gen first" #\a (%generator-next gen))
  (test-equal "String gen second" #\b (%generator-next gen))
  (test-equal "String gen third" #\c (%generator-next gen)))

;; Test reverse-vector->generator
(let ((gen (reverse-vector->generator #(1 2 3))))
  (test-equal "Reverse vector first" 3 (%generator-next gen))
  (test-equal "Reverse vector second" 2 (%generator-next gen))
  (test-equal "Reverse vector third" 1 (%generator-next gen)))

;; Test make-unfold-generator
(let ((gen (make-unfold-generator 
             (lambda (x) (> x 5))
             (lambda (x) (* x x))
             (lambda (x) (+ x 1))
             2)))
  (test-equal "Unfold first" 4 (%generator-next gen))   ; 2*2
  (test-equal "Unfold second" 9 (%generator-next gen))  ; 3*3
  (test-equal "Unfold third" 16 (%generator-next gen))  ; 4*4
  (test-equal "Unfold fourth" 25 (%generator-next gen))) ; 5*5

(display "Testing transformers/combinators...\n")

;; Test gcons*
(let ((gen (gcons* 'x 'y (generator 1 2 3))))
  (test-equal "gcons* first" 'x (%generator-next gen))
  (test-equal "gcons* second" 'y (%generator-next gen))
  (test-equal "gcons* third" 1 (%generator-next gen)))

;; Test gappend
(let ((gen (gappend (generator 1 2) (generator 3 4))))
  (test-equal "gappend first" 1 (%generator-next gen))
  (test-equal "gappend second" 2 (%generator-next gen))
  (test-equal "gappend third" 3 (%generator-next gen))
  (test-equal "gappend fourth" 4 (%generator-next gen)))

;; Test gfilter
(let ((gen (gfilter odd? (generator 1 2 3 4 5))))
  (test-equal "gfilter first odd" 1 (%generator-next gen))
  (test-equal "gfilter second odd" 3 (%generator-next gen))
  (test-equal "gfilter third odd" 5 (%generator-next gen)))

;; Test gtake
(let ((gen (gtake (generator 1 2 3 4 5) 3)))
  (test-equal "gtake first" 1 (%generator-next gen))
  (test-equal "gtake second" 2 (%generator-next gen))
  (test-equal "gtake third" 3 (%generator-next gen))
  (test-true "gtake exhausted" (%generator-exhausted? gen)))

;; Test gtake-while
(let ((gen (gtake-while (lambda (x) (< x 4)) (generator 1 2 3 4 5))))
  (test-equal "gtake-while first" 1 (%generator-next gen))
  (test-equal "gtake-while second" 2 (%generator-next gen))
  (test-equal "gtake-while third" 3 (%generator-next gen))
  (test-true "gtake-while stops at 4" (%generator-exhausted? gen)))

(display "Testing consumers...\n")

;; Test generator->list
(test-equal "generator->list" '(1 2 3) 
            (generator->list (generator 1 2 3)))

;; Test generator->vector
(test-equal "generator->vector" #(a b c)
            (generator->vector (generator 'a 'b 'c)))

;; Test generator-fold
(test-equal "generator-fold sum" 15
            (generator-fold + 0 (generator 1 2 3 4 5)))

;; Test generator-map
(let ((gen (generator-map (lambda (x) (* x x)) (generator 1 2 3))))
  (test-equal "generator-map first" 1 (%generator-next gen))
  (test-equal "generator-map second" 4 (%generator-next gen))
  (test-equal "generator-map third" 9 (%generator-next gen)))

;; Test generator-find
(test-equal "generator-find even" 2
            (generator-find even? (generator 1 2 3 4 5)))

;; Test generator-count
(test-equal "generator-count evens" 2
            (generator-count even? (generator 1 2 3 4 5)))

;; Test generator-any
(test-true "generator-any even" 
           (generator-any even? (generator 1 2 3 4)))

;; Test generator-every
(test-false "generator-every even"
            (generator-every even? (generator 1 2 3 4)))
(test-true "generator-every positive"
           (generator-every positive? (generator 1 2 3 4)))

;; Test generator-length
(test-equal "generator-length" 5
            (generator-length (generator 'a 'b 'c 'd 'e)))

;; Test generator-sum
(test-equal "generator-sum" 10
            (generator-sum (generator 1 2 3 4)))

(display "Testing accumulators...\n")

;; Test count-accumulator
(let ((acc (count-accumulator)))
  (acc 'x)
  (acc 'y)
  (acc 'z)
  (test-equal "count-accumulator" 3 (acc '*eof-object*)))

;; Test list-accumulator
(let ((acc (list-accumulator)))
  (acc 1)
  (acc 2)
  (acc 3)
  (test-equal "list-accumulator" '(3 2 1) (acc '*eof-object*)))

;; Test reverse-list-accumulator
(let ((acc (reverse-list-accumulator)))
  (acc 1)
  (acc 2) 
  (acc 3)
  (test-equal "reverse-list-accumulator" '(1 2 3) (acc '*eof-object*)))

;; Test sum-accumulator
(let ((acc (sum-accumulator)))
  (acc 10)
  (acc 20)
  (acc 30)
  (test-equal "sum-accumulator" 60 (acc '*eof-object*)))

;;; Summary
(newline)
(display "Test Results: ")
(display (- test-count fail-count))
(display "/")
(display test-count)
(display " passed")
(if (> fail-count 0)
    (begin
      (display " (")
      (display fail-count)
      (display " failed)"))
    (display " (all passed)"))
(newline)

(if (= fail-count 0)
    (display "🎉 All SRFI-121 tests passed!\n")
    (display "❌ Some tests failed. Check implementation.\n"))