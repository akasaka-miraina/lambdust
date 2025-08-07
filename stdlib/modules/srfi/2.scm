;; SRFI-2: and-let*
;; 
;; This library provides the and-let* macro, which is useful for sequential binding
;; and testing where any test failure short-circuits the entire expression.
;; It's particularly useful for nested null/false checks in functional programming.
;;
;; Reference: https://srfi.schemers.org/srfi-2/srfi-2.html

(define-library (srfi 2)
  (import (scheme base))
  
  (export and-let*)

  (begin
    ;; The and-let* macro performs sequential binding and testing
    ;;
    ;; Syntax: (and-let* (clause1 clause2 ... clauseN) body1 body2 ...)
    ;;
    ;; Each clause can be one of:
    ;; - (variable expression) - bind variable to expression, fail if #f
    ;; - (expression) - test expression, fail if #f
    ;; - variable - test variable, fail if #f
    ;;
    ;; The macro evaluates clauses left to right. If any clause evaluates to #f,
    ;; the entire and-let* expression evaluates to #f. Otherwise, the body is
    ;; evaluated in the extended lexical environment.
    ;;
    ;; Examples:
    ;; (and-let* ((x (find-something))
    ;;            (y (process x))
    ;;            ((valid? y)))
    ;;   (use y))
    ;;
    ;; (and-let* ((alist (get-alist))
    ;;            (entry (assoc key alist))
    ;;            (value (cdr entry)))
    ;;   value)
    
    (define-syntax and-let*
      (syntax-rules ()
        ;; Base case: empty clause list, evaluate body
        ((_ () body1 body2 ...)
         (begin body1 body2 ...))
        
        ;; Single variable clause: test it
        ((_ (var) body1 body2 ...)
         (and var (begin body1 body2 ...)))
        
        ;; Single expression clause: test it
        ((_ ((expr)) body1 body2 ...)
         (and expr (begin body1 body2 ...)))
        
        ;; Single binding clause: bind and test
        ((_ ((var expr)) body1 body2 ...)
         (let ((var expr))
           (and var (begin body1 body2 ...))))
        
        ;; Multiple clauses: handle first, recurse on rest
        ;; Variable clause followed by more
        ((_ (var clause2 ... ) body1 body2 ...)
         (and var (and-let* (clause2 ...) body1 body2 ...)))
        
        ;; Expression clause followed by more
        ((_ ((expr) clause2 ...) body1 body2 ...)
         (and expr (and-let* (clause2 ...) body1 body2 ...)))
        
        ;; Binding clause followed by more
        ((_ ((var expr) clause2 ...) body1 body2 ...)
         (let ((var expr))
           (and var (and-let* (clause2 ...) body1 body2 ...))))))
    
    ;; Examples and usage patterns:
    
    ;; Example 1: Safe nested access
    ;; (define (safe-caar x)
    ;;   (and-let* ((p (pair? x))
    ;;              (car-x (car x))
    ;;              (car-car-x (pair? car-x)))
    ;;     (car car-x)))
    
    ;; Example 2: Database-style queries
    ;; (define (find-user-email userid)
    ;;   (and-let* ((user (find-user userid))
    ;;              (profile (user-profile user))
    ;;              (email (profile-email profile))
    ;;              ((string? email))
    ;;              ((not (string-null? email))))
    ;;     email))
    
    ;; Example 3: File processing
    ;; (define (process-config-file filename)
    ;;   (and-let* ((exists? (file-exists? filename))
    ;;              (port (open-input-file filename))
    ;;              (config (read-config port)))
    ;;     (close-input-port port)
    ;;     (validate-and-use-config config)))
    
    ;; Example 4: Mathematical operations with bounds checking
    ;; (define (safe-divide-and-sqrt x y)
    ;;   (and-let* (((number? x))
    ;;              ((number? y))
    ;;              ((not (zero? y)))
    ;;              (quotient (/ x y))
    ;;              ((>= quotient 0)))
    ;;     (sqrt quotient)))
    
    ;; Utility functions that work well with and-let*
    
    (define (non-empty-string? obj)
      "Test if obj is a non-empty string."
      (and (string? obj) (not (string=? obj ""))))
    
    (define (non-null-list? obj)
      "Test if obj is a non-null list."
      (and (list? obj) (not (null? obj))))
    
    (define (positive-number? obj)
      "Test if obj is a positive number."
      (and (number? obj) (positive? obj)))
    
    (define (non-negative-number? obj)
      "Test if obj is a non-negative number."
      (and (number? obj) (>= obj 0)))
    
    ;; Advanced example: JSON-like data processing
    ;; (define (extract-nested-value data . keys)
    ;;   (and-let* ((current data))
    ;;     (let loop ((keys keys) (current current))
    ;;       (cond
    ;;         ((null? keys) current)
    ;;         ((not (pair? current)) #f)
    ;;         (else
    ;;          (and-let* ((entry (assoc (car keys) current))
    ;;                     (value (cdr entry)))
    ;;            (loop (cdr keys) value)))))))
    
    ;; Pattern: Optional chaining (similar to modern languages)
    ;; (define-syntax optional->
    ;;   (syntax-rules ()
    ;;     ((_ value) value)
    ;;     ((_ value (proc arg ...) more ...)
    ;;      (and-let* ((result value)
    ;;                 (next (proc result arg ...)))
    ;;        (optional-> next more ...)))))
    
    ;; Integration with other SRFI libraries
    ;; Works particularly well with:
    ;; - SRFI-1 list operations
    ;; - SRFI-13 string operations  
    ;; - SRFI-43 vector operations
    
    ;; Example combining with SRFI-1:
    ;; (define (process-non-empty-list lst)
    ;;   (and-let* (((list? lst))
    ;;              ((not (null? lst)))
    ;;              (first-valid (find positive? lst))
    ;;              (doubled-list (map (lambda (x) (* x 2)) lst)))
    ;;     (cons first-valid doubled-list)))
    
    ;; Performance notes:
    ;; - and-let* is implemented as nested lets and ands
    ;; - Short-circuits on first #f value
    ;; - Lexical binding means no runtime overhead for variable lookup
    ;; - Compile-time macro expansion means no runtime interpretation
    
    ;; Error handling pattern:
    ;; (define (safe-operation x)
    ;;   (or (and-let* (((valid-input? x))
    ;;                  (processed (process x))
    ;;                  ((valid-output? processed)))
    ;;         processed)
    ;;       (error "Invalid operation" x)))
    
    ;; Common pitfalls to avoid:
    ;; 1. Don't use and-let* for simple and expressions
    ;; 2. Remember that #f terminates, but 0, '(), "" do not
    ;; 3. Variable bindings are only visible to subsequent clauses and body
    ;; 4. Empty body with empty clauses returns unspecified value
    ))