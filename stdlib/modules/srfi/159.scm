;;; SRFI-158: Enhanced Generators and Accumulators - Minimal Implementation
;;;
;;; This is a minimal implementation to test the core functionality

(define-library (srfi 159)
  (import (scheme base)
          (srfi 121))
  
  (export
    ;; Re-export all SRFI-121 procedures
    %make-generator %generator-next %generator-exhausted? %generator?
    generator make-range-generator make-iota-generator
    list->generator vector->generator reverse-vector->generator
    string->generator bytevector->generator
    make-for-each-generator make-unfold-generator
    make-coroutine-generator
    gcons* gappend gflatten ggroup
    gfilter gremove gstate-filter gdelete gdelete-neighbor-dups
    gtake gdrop gtake-while gdrop-while
    gindex gselect
    generator->list generator->vector generator->string generator->bytevector
    generator-fold generator-reduce generator-for-each generator-map
    generator-find generator-count generator-any generator-every
    generator-length generator-sum generator-unfold
    make-accumulator count-accumulator list-accumulator
    reverse-list-accumulator vector-accumulator string-accumulator
    bytevector-accumulator sum-accumulator
    make-accumulator-generator
    
    ;; NEW SRFI-158 procedures
    circular-generator make-bits-generator
    product-accumulator)
  
  (begin
    ;; Internal utilities
    (define *eof-object* '*eof-object*)
    (define (eof-object? x) (eq? x *eof-object*))
    (define (safe-generator-next gen)
      (if (%generator-exhausted? gen)
          *eof-object*
          (%generator-next gen)))
    
    ;; Circular generator - simple version
    (define (circular-generator . args)
      (if (null? args)
          (error "circular-generator: at least one argument required")
          (let ((items args)
                (index 0))
            (%make-generator
              (lambda ()
                (if (null? items)
                    *eof-object*
                    (let ((value (list-ref items (modulo index (length items)))))
                      (set! index (+ index 1))
                      value)))))))
    
    ;; Make bits generator - simple version
    (define (make-bits-generator n)
      (if (not (and (integer? n) (>= n 0)))
          (error "make-bits-generator: non-negative integer required" n)
          (if (= n 0)
              (generator 0)
              (let ((bits '()))
                ;; Convert to binary (LSB first)
                (let loop ((num n))
                  (when (> num 0)
                    (set! bits (cons (modulo num 2) bits))
                    (loop (quotient num 2))))
                (apply generator (reverse bits))))))
    
    ;; Product accumulator
    (define (product-accumulator)
      (make-accumulator * 1))
    
    )) ; end library