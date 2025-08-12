;;; SRFI-121: Generators - Simplified for debugging

(define-library (srfi 120)
  (import (scheme base)
          (scheme case-lambda))
  
  (export
    ;; Core generator operations  
    %make-generator %generator-next %generator-exhausted? %generator?
    
    ;; Basic constructors
    generator list->generator
    
    ;; Basic consumers
    generator->list)
  
  (begin
    ;; EOF object
    (define *eof-object* '*eof-object*)
    (define (eof-object? x) (eq? x *eof-object*))
    
    ;; Safe next operation
    (define (safe-generator-next gen)
      (if (%generator-exhausted? gen)
          *eof-object*
          (%generator-next gen)))
    
    )) ; end library