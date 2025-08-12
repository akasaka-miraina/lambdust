;;; SRFI-158 Enhanced Generators and Accumulators - Comprehensive Test Suite
;;;
;;; This test suite validates the complete SRFI-158 implementation
;;; including all 15 new enhanced procedures and backward compatibility
;;; with SRFI-121.

(import (scheme base)
        (scheme write)
        (srfi 158)     ; Enhanced generators
        (srfi 128))    ; Comparators for gmerge

;; Test framework
(define test-count 0)
(define pass-count 0)

(define (test name expected actual)
  (set! test-count (+ test-count 1))
  (let ((passed (equal? expected actual)))
    (if passed
        (begin
          (set! pass-count (+ pass-count 1))
          (display "✓ "))
        (display "✗ "))
    (display name)
    (newline)
    (unless passed
      (display "  Expected: ")
      (write expected)
      (newline)
      (display "  Actual:   ")
      (write actual)
      (newline))))

(define (test-generator name expected-list gen)
  (test name expected-list (generator->list gen)))

;; ================ SRFI-121 BACKWARD COMPATIBILITY TESTS ================

(display "Testing SRFI-121 backward compatibility...\n")

;; Basic generator construction
(test-generator "basic generator" 
  '(1 2 3) 
  (generator 1 2 3))

(test-generator "list->generator"
  '(a b c d)
  (list->generator '(a b c d)))

;; Generator transformers  
(test-generator "gfilter"
  '(2 4 6)
  (gfilter even? (generator 1 2 3 4 5 6)))

(test-generator "gtake"
  '(1 2 3)
  (gtake (generator 1 2 3 4 5) 3))

;; Generator consumers
(test "generator-fold"
  15
  (generator-fold + 0 (generator 1 2 3 4 5)))

(test "generator-count"
  3
  (generator-count even? (generator 1 2 3 4 5 6)))

;; Accumulators
(let ((acc (list-accumulator)))
  (acc 'a)
  (acc 'b)  
  (acc 'c)
  (test "list-accumulator" '(c b a) (acc '*eof-object*)))

;; ================ NEW SRFI-158 ENHANCED CONSTRUCTORS ================

(display "\nTesting SRFI-158 enhanced constructors...\n")

;; Circular generator
(test-generator "circular-generator finite"
  '(a b c a b c a b c)
  (gtake (circular-generator 'a 'b 'c) 9))

(test-generator "circular-generator list"
  '(1 2 1 2 1 2)
  (gtake (circular-generator '(1 2)) 6))

;; Make bits generator  
(test-generator "make-bits-generator 0"
  '(0)
  (make-bits-generator 0))

(test-generator "make-bits-generator 5"
  '(1 0 1)  ; 5 = 101 in binary, LSB first
  (make-bits-generator 5))

(test-generator "make-bits-generator 8" 
  '(0 0 0 1)  ; 8 = 1000 in binary, LSB first
  (make-bits-generator 8))

;; ================ NEW SRFI-158 ENHANCED TRANSFORMERS ================

(display "\nTesting SRFI-158 enhanced transformers...\n")

;; gmerge with numeric comparator
(test-generator "gmerge simple"
  '(1 2 3 4 5 6)
  (gmerge default-comparator 
          (generator 1 3 5)
          (generator 2 4 6)))

;; gcombine with state
(test-generator "gcombine sum with counter"
  '(1 3 6 10 15)  ; Running sums: 1, 1+2, 1+2+3, 1+2+3+4, 1+2+3+4+5
  (gcombine 
    (lambda (state value)
      (let ((new-state (+ state value)))
        (values new-state new-state)))
    0
    (generator 1 2 3 4 5)))

;; generator-zip-with
(test-generator "generator-zip-with +"
  '(5 7 9)
  (generator-zip-with +
                      (generator 1 2 3)
                      (generator 4 5 6)))

(test-generator "generator-zip-with list"
  '((a 1) (b 2) (c 3))
  (generator-zip-with list
                      (generator 'a 'b 'c)
                      (generator 1 2 3)))

;; ================ NEW SRFI-158 ENHANCED ACCUMULATORS ================

(display "\nTesting SRFI-158 enhanced accumulators...\n")

;; Product accumulator
(let ((acc (product-accumulator)))
  (acc 2)
  (acc 3)
  (acc 4)
  (test "product-accumulator" 24 (acc '*eof-object*)))

;; Min accumulator
(let ((acc (min-accumulator)))
  (acc 5)
  (acc 2)
  (acc 8)
  (acc 1)
  (test "min-accumulator" 1 (acc '*eof-object*)))

;; Max accumulator  
(let ((acc (max-accumulator)))
  (acc 3)
  (acc 7)
  (acc 2)
  (acc 9)
  (test "max-accumulator" 9 (acc '*eof-object*)))

;; Enhanced vector accumulator
(let ((acc (enhanced-vector-accumulator)))
  (acc 'a)
  (acc 'b)
  (acc 'c)
  (test "enhanced-vector-accumulator" 
        '#(a b c)
        (acc '*eof-object*)))

;; Accumulate generated
(test "accumulate-generated with sum"
  15
  (accumulate-generated (sum-accumulator)
                       (generator 1 2 3 4 5)))

(test "generator-accumulate with count"
  4
  (generator-accumulate (generator 'a 'b 'c 'd)
                       (count-accumulator)))

;; Enhanced make-accumulator-generator
(test-generator "enhanced-make-accumulator-generator"
  '(1 3 6 10 10)  ; Running sum + final result
  (enhanced-make-accumulator-generator (sum-accumulator)
                                      (generator 1 2 3 4)))

;; ================ NEW SRFI-158 ADVANCED UTILITIES ================

(display "\nTesting SRFI-158 advanced utilities...\n")

;; Generator concatenate
(test-generator "generator-concatenate"
  '(1 2 3 a b c 10 20)
  (generator-concatenate 
    (generator 
      (generator 1 2 3)
      (generator 'a 'b 'c)
      (generator 10 20))))

;; Generator pad with
(test-generator "generator-pad-with"
  '((1 a) (2 b) (3 default) (4 default))
  (generator-pad-with 'default
                     (generator 1 2 3 4)
                     (generator 'a 'b)))

;; Generator maybe ref
(test "generator-maybe-ref found"
  'c
  (generator-maybe-ref (generator 'a 'b 'c 'd) 2))

(test "generator-maybe-ref not-found"
  'default
  (generator-maybe-ref (generator 'a 'b) 5 'default))

(test "generator-maybe-ref boundary"
  'default
  (generator-maybe-ref (generator 'a 'b 'c) 3 'default))

;; ================ COMPLEX INTEGRATION TESTS ================

(display "\nTesting complex integrations...\n")

;; Chain multiple SRFI-158 operations
(test-generator "complex chain 1"
  '(2 4)
  (gfilter even? 
           (generator-zip-with +
                              (circular-generator 1 2 3)
                              (generator 1 2 3 4 5))))

;; Merge multiple sorted streams
(test-generator "complex merge"
  '(1 2 3 4 5 6 7 8 9)
  (gmerge default-comparator
          (generator 1 4 7)
          (generator 2 5 8)  
          (generator 3 6 9)))

;; Accumulator with generator transformation
(test "accumulator with transformation"
  120  ; 1*2*3*4*5
  (accumulate-generated (product-accumulator)
                       (gfilter (lambda (x) (<= x 5))
                               (circular-generator 1 2 3 4 5 6 7))))

;; Generator concatenation with transformations
(test-generator "concatenation with filters"
  '(2 4 6 8 10)
  (gtake 
    (gfilter even?
             (generator-concatenate
               (generator (generator 1 2 3 4 5)
                         (generator 6 7 8 9 10 11 12))))
    5))

;; ================ EDGE CASES AND ERROR HANDLING ================

(display "\nTesting edge cases...\n")

;; Empty generators
(test-generator "circular-generator empty" 
  '()
  (gtake (circular-generator) 3))

(test-generator "make-bits-generator zero"
  '(0)
  (make-bits-generator 0))

;; Single element operations
(test-generator "gmerge single elements"
  '(1 2)
  (gmerge default-comparator 
          (generator 1)
          (generator 2)))

;; Boundary conditions for generator-maybe-ref
(test "generator-maybe-ref index 0"
  'first
  (generator-maybe-ref (generator 'first 'second) 0))

;; ================ RESULTS ================

(display "\n" "=== SRFI-158 Test Results ===\n")
(display "Tests passed: ")
(display pass-count)
(display "/")
(display test-count)
(newline)

(if (= pass-count test-count)
    (display "🎉 All tests passed! SRFI-158 implementation is working correctly.\n")
    (begin
      (display "❌ Some tests failed. Implementation needs review.\n")
      (display "Success rate: ")
      (display (inexact (/ pass-count test-count)))
      (newline)))

(display "\nSRFI-158 provides ")
(display "68 total procedures")
(display " (53 from SRFI-121 + 15 new enhanced procedures)\n")

(display "Enhanced constructors: circular-generator, make-bits-generator\n")
(display "Enhanced transformers: gmerge, gcombine, generator-zip-with\n") 
(display "Enhanced accumulators: product-, min-, max-accumulator, enhanced-vector-accumulator\n")
(display "                      accumulate-generated, generator-accumulate, enhanced-make-accumulator-generator\n")
(display "Advanced utilities: generator-concatenate, generator-pad-with, generator-maybe-ref\n")