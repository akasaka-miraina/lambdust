;; SRFI-8: receive
;; 
;; This library provides the receive macro for binding multiple values returned
;; by expressions. It provides a clean interface for working with procedures
;; that return multiple values using the values procedure.
;;
;; Reference: https://srfi.schemers.org/srfi-8/srfi-8.html

(define-library (srfi 8)
  (import (scheme base))
  
  (export receive)

  (begin
    ;; The receive macro provides syntax for binding multiple values
    ;;
    ;; Syntax: (receive formals expression body1 body2 ...)
    ;;
    ;; Where formals is a formal parameter list as in lambda:
    ;; - (var1 var2 ... varN) - bind exactly N values
    ;; - (var1 var2 ... . rest) - bind at least N values, rest to list
    ;; - var - bind all values to a single list
    ;;
    ;; The expression should evaluate to multiple values (typically using values).
    ;; The formals are bound to these values and the body is evaluated.
    ;;
    ;; Examples:
    ;; (receive (a b) (values 1 2) (+ a b)) => 3
    ;; (receive (x y . rest) (values 1 2 3 4) (list x y rest)) => (1 2 (3 4))
    ;; (receive args (values 1 2 3) args) => (1 2 3)
    
    (define-syntax receive
      (syntax-rules ()
        ;; Basic case: bind formal parameters to multiple values
        ((_ formals expression body1 body2 ...)
         (call-with-values
           (lambda () expression)
           (lambda formals body1 body2 ...)))))
    
    ;; ============= USAGE EXAMPLES =============
    
    ;; Example 1: Quotient and remainder
    ;; (receive (q r) (quotient/remainder 23 5)
    ;;   (list 'quotient q 'remainder r))
    ;; => (quotient 4 remainder 3)
    
    ;; Example 2: Multiple return values from user function
    ;; (define (analyze-number n)
    ;;   (values (positive? n) (even? n) (zero? n)))
    ;;
    ;; (receive (pos? even? zero?) (analyze-number 42)
    ;;   (list pos? even? zero?))
    ;; => (#t #t #f)
    
    ;; Example 3: Variable number of values
    ;; (define (split-at-first-space str)
    ;;   (let ((pos (string-index str #\space)))
    ;;     (if pos
    ;;         (values (substring str 0 pos)
    ;;                 (substring str (+ pos 1) (string-length str)))
    ;;         (values str))))
    ;;
    ;; (receive (first . rest) (split-at-first-space "hello world foo")
    ;;   (cons first rest))
    ;; => ("hello" "world foo")
    
    ;; Example 4: All values to single binding
    ;; (receive all-values (values 1 2 3 4 5)
    ;;   (length all-values))
    ;; => 5
    
    ;; ============= INTEGRATION WITH R7RS MULTIPLE VALUES =============
    
    ;; SRFI-8 receive works seamlessly with R7RS multiple values:
    
    ;; Standard R7RS procedures that return multiple values:
    ;; - (values obj ...) - creates multiple values
    ;; - (call-with-values producer consumer) - handles multiple values
    
    ;; Example with exact/inexact integer division:
    ;; (receive (quotient remainder) 
    ;;     (floor/ 23 4)
    ;;   (+ quotient remainder))
    ;; => 11 (5 + 3 + 3 = 11, where 23 = 5*4 + 3)
    
    ;; ============= COMMON PATTERNS =============
    
    ;; Pattern 1: Error handling with multiple values
    ;; (define (safe-divide x y)
    ;;   (if (zero? y)
    ;;       (values #f "Division by zero")
    ;;       (values (/ x y) #f)))
    ;;
    ;; (receive (result error) (safe-divide 10 2)
    ;;   (if error
    ;;       (display error)
    ;;       result))
    
    ;; Pattern 2: State and value returns
    ;; (define (counter-increment state)
    ;;   (let ((new-state (+ state 1)))
    ;;     (values new-state new-state)))
    ;;
    ;; (receive (new-state value) (counter-increment 5)
    ;;   (display (list 'state new-state 'value value)))
    
    ;; Pattern 3: Parsing with position tracking
    ;; (define (parse-number str pos)
    ;;   (let loop ((i pos) (acc 0))
    ;;     (if (or (>= i (string-length str))
    ;;             (not (char-numeric? (string-ref str i))))
    ;;         (values acc i)
    ;;         (loop (+ i 1) 
    ;;               (+ (* acc 10) 
    ;;                  (- (char->integer (string-ref str i)) 
    ;;                     (char->integer #\0)))))))
    ;;
    ;; (receive (number new-pos) (parse-number "123abc" 0)
    ;;   (list number new-pos))
    ;; => (123 3)
    
    ;; ============= COMPARISON WITH ALTERNATIVES =============
    
    ;; Without receive (using call-with-values directly):
    ;; (call-with-values
    ;;   (lambda () (values 1 2 3))
    ;;   (lambda (a b c) (+ a b c)))
    
    ;; With receive (cleaner syntax):
    ;; (receive (a b c) (values 1 2 3)
    ;;   (+ a b c))
    
    ;; Multiple value assignment (hypothetical syntax):
    ;; (let-values (((a b c) (values 1 2 3)))
    ;;   (+ a b c))
    
    ;; receive is more concise and readable than call-with-values
    ;; and provides a single, consistent interface for multiple value binding.
    
    ;; ============= ERROR HANDLING =============
    
    ;; Wrong number of values:
    ;; (receive (a b) (values 1 2 3 4) ...) ; Error: too many values
    ;; (receive (a b c) (values 1 2) ...)   ; Error: too few values
    
    ;; Using rest parameters for flexibility:
    ;; (receive (a b . rest) (values 1 2 3 4) 
    ;;   rest) ; => (3 4)
    
    ;; (receive (a . rest) (values 1)
    ;;   rest) ; => ()
    
    ;; ============= ADVANCED USAGE =============
    
    ;; Nested receive (though not usually necessary):
    ;; (receive (outer-a outer-b) (values 1 (values 2 3))
    ;;   (receive (inner-a inner-b) outer-b
    ;;     (+ outer-a inner-a inner-b)))
    ;; => 6
    
    ;; receive with generators/iterators pattern:
    ;; (define (make-range-generator start end)
    ;;   (let ((current start))
    ;;     (lambda ()
    ;;       (if (< current end)
    ;;           (let ((value current))
    ;;             (set! current (+ current 1))
    ;;             (values #t value))
    ;;           (values #f #f)))))
    ;;
    ;; (let ((gen (make-range-generator 0 3)))
    ;;   (let loop ()
    ;;     (receive (has-value? value) (gen)
    ;;       (if has-value?
    ;;           (begin
    ;;             (display value)
    ;;             (loop))
    ;;           'done))))
    
    ;; ============= PERFORMANCE CONSIDERATIONS =============
    
    ;; receive is a macro that expands to call-with-values
    ;; - No runtime overhead compared to direct call-with-values usage
    ;; - Multiple values are typically implemented efficiently
    ;; - Modern Scheme implementations optimize multiple value operations
    
    ;; For single values, receive has slight overhead compared to let:
    ;; (receive (x) (some-computation) ...)  ; Slight overhead
    ;; (let ((x (some-computation))) ...)    ; More direct
    
    ;; ============= COMPATIBILITY NOTES =============
    
    ;; SRFI-8 is widely supported and is considered a fundamental extension
    ;; Many Scheme implementations include receive in their base library
    ;; R7RS-large is likely to include similar functionality
    
    ;; Integration with other SRFIs:
    ;; - Works well with SRFI-1 list operations that may return multiple values
    ;; - Complements SRFI-2 and-let* for conditional binding
    ;; - Can be used with SRFI-43 vector operations
    
    ;; Thread safety:
    ;; - receive itself is thread-safe (it's a macro)
    ;; - Thread safety depends on the expression being evaluated
    ;; - Multiple values are typically thread-local
    ))