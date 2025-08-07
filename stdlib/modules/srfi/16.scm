;; SRFI-16: Syntax for procedures of variable arity (case-lambda)
;; 
;; This library provides case-lambda, which creates a procedure that can have
;; a variable number of arguments. The different cases are selected based on
;; the number of arguments passed to the procedure.
;;
;; Reference: https://srfi.schemers.org/srfi-16/srfi-16.html

(define-library (srfi 16)
  (import (scheme base))
  
  (export case-lambda)

  (begin  
    ;; case-lambda creates a procedure that dispatches based on argument count
    ;;
    ;; Syntax: (case-lambda
    ;;           (formals body ...)
    ;;           (formals body ...)
    ;;           ...)
    ;;
    ;; Each clause has the form (formals body ...) where:
    ;; - formals can be a proper list of variables (fixed arity)
    ;; - formals can be a single variable (variadic, receives all arguments as a list)
    ;; - formals can be an improper list (fixed + variadic)
    ;;
    ;; Examples:
    ;; (define plus
    ;;   (case-lambda
    ;;     (() 0)
    ;;     ((x) x)
    ;;     ((x y) (+ x y))
    ;;     ((x y z) (+ x y z))
    ;;     (args (apply + args))))
    
    (define-syntax case-lambda
      (syntax-rules ()
        ;; Single clause case
        ((_ (formals body ...))
         (lambda formals body ...))
        
        ;; Multiple clauses - generate dispatcher
        ((_ (formals1 body1 ...) (formals2 body2 ...) ...)
         (lambda args
           (case-lambda-dispatch args
                                 (formals1 body1 ...)
                                 (formals2 body2 ...) ...)))))
    
    ;; Helper macro to generate the dispatch logic
    (define-syntax case-lambda-dispatch
      (syntax-rules ()
        ;; Base case: no more clauses, generate error
        ((_ args)
         (error "No matching case for arguments" args))
        
        ;; Single clause remaining
        ((_ args (formals body ...))
         (case-lambda-try-clause args (formals body ...) (error "No matching case for arguments" args)))
        
        ;; Multiple clauses remaining
        ((_ args (formals1 body1 ...) (formals2 body2 ...) ...)
         (case-lambda-try-clause args 
                                 (formals1 body1 ...)
                                 (case-lambda-dispatch args (formals2 body2 ...) ...)))))
    
    ;; Helper macro to try a single clause
    (define-syntax case-lambda-try-clause
      (syntax-rules ()
        ;; Empty formals (no arguments)
        ((_ args (() body ...) else-expr)
         (if (null? args)
             (begin body ...)
             else-expr))
        
        ;; Single variable (variadic - all arguments)
        ((_ args (var body ...) else-expr)
         (let ((var args)) body ...))
        
        ;; Proper list of variables (fixed arity)
        ((_ args ((var1 var2 ...) body ...) else-expr)
         (case-lambda-match-fixed args (var1 var2 ...) (begin body ...) else-expr))
        
        ;; Improper list (fixed + variadic)
        ((_ args ((var1 var2 ... . rest-var) body ...) else-expr)
         (case-lambda-match-improper args (var1 var2 ...) rest-var (begin body ...) else-expr))))
    
    ;; Helper macro for fixed arity matching
    (define-syntax case-lambda-match-fixed
      (syntax-rules ()
        ;; No variables - check for empty args
        ((_ args () body else-expr)
         (if (null? args) body else-expr))
        
        ;; One variable
        ((_ args (var) body else-expr)
         (if (and (pair? args) (null? (cdr args)))
             (let ((var (car args))) body)
             else-expr))
        
        ;; Multiple variables
        ((_ args (var1 var2 ...) body else-expr)
         (case-lambda-match-fixed-helper args (var1 var2 ...) () body else-expr))))
    
    ;; Helper for matching fixed arity with accumulator
    (define-syntax case-lambda-match-fixed-helper
      (syntax-rules ()
        ;; All variables matched
        ((_ args () (var ...) body else-expr)
         (if (= (length args) (length '(var ...)))
             (apply (lambda (var ...) body) args)
             else-expr))
        
        ;; More variables to match
        ((_ args (var1 var2 ...) (matched ...) body else-expr)
         (case-lambda-match-fixed-helper args (var2 ...) (matched ... var1) body else-expr))))
    
    ;; Helper macro for improper list matching (fixed + variadic)
    (define-syntax case-lambda-match-improper
      (syntax-rules ()
        ;; No fixed variables - all go to rest
        ((_ args () rest-var body else-expr)
         (let ((rest-var args)) body))
        
        ;; One or more fixed variables
        ((_ args (var1 ...) rest-var body else-expr)
         (let ((fixed-count (length '(var1 ...))))
           (if (>= (length args) fixed-count)
               (let-values (((fixed-args rest-args) (split-at args fixed-count)))
                 (apply (lambda (var1 ...)
                          (let ((rest-var rest-args)) body))
                        fixed-args))
               else-expr)))))
    
    ;; Utility function to split a list at a given position
    ;; This should be available from SRFI-1, but we define it here for independence
    (define (split-at lst k)
      (let loop ((lst lst) (k k) (prefix '()))
        (if (= k 0)
            (values (reverse prefix) lst)
            (loop (cdr lst) (- k 1) (cons (car lst) prefix)))))
    
    ;; Alternative implementation using explicit length checking
    ;; This version might be more efficient in some implementations
    
    (define-syntax case-lambda-alt
      (syntax-rules ()
        ((_ clause ...)
         (lambda args
           (let ((len (length args)))
             (case-lambda-alt-dispatch len args clause ...))))))
    
    (define-syntax case-lambda-alt-dispatch
      (syntax-rules ()
        ;; No more clauses
        ((_ len args)
         (error "No matching case for" len "arguments"))
        
        ;; Try each clause
        ((_ len args (formals body ...) rest ...)
         (case-lambda-alt-try len args (formals body ...) 
                              (case-lambda-alt-dispatch len args rest ...)))))
    
    (define-syntax case-lambda-alt-try
      (syntax-rules ()
        ;; Empty formals
        ((_ len args (() body ...) else-expr)
         (if (= len 0) (begin body ...) else-expr))
        
        ;; Single variable (variadic)
        ((_ len args (var body ...) else-expr)
         (let ((var args)) body ...))
        
        ;; Fixed arity
        ((_ len args ((var ...) body ...) else-expr)
         (if (= len (length '(var ...)))
             (apply (lambda (var ...) body ...) args)
             else-expr))
        
        ;; Improper list (fixed + rest)
        ((_ len args ((var1 ... . rest-var) body ...) else-expr)
         (let ((min-args (length '(var1 ...))))
           (if (>= len min-args)
               (let-values (((fixed rest) (split-at args min-args)))
                 (apply (lambda (var1 ...)
                          (let ((rest-var rest)) body ...))
                        fixed))
               else-expr))))
    
    ;; Simple version for common cases
    (define-syntax simple-case-lambda
      (syntax-rules ()
        ;; Two cases: specific arity and variadic
        ((_ (() zero-body ...)
            (args variadic-body ...))
         (lambda args
           (if (null? args)
               (begin zero-body ...)
               (begin variadic-body ...))))
        
        ;; Three cases: specific arities
        ((_ ((var1) one-body ...)
            ((var1-2 var2-2) two-body ...)
            (args variadic-body ...))
         (lambda args
           (case (length args)
             ((1) (let ((var1 (car args))) one-body ...))
             ((2) (let ((var1-2 (car args)) (var2-2 (cadr args))) two-body ...))
             (else (begin variadic-body ...)))))))
    
    ;; Examples and test cases for the implementation
    
    ;; Example 1: Simple arithmetic function
    (define plus-example
      (case-lambda
        (() 0)
        ((x) x)
        ((x y) (+ x y))
        ((x y z) (+ x y z))
        (args (apply + args))))
    
    ;; Example 2: List processing with different behaviors
    (define list-processor
      (case-lambda
        (() '())
        ((x) (list x))
        ((x y) (list x y))
        ((x . rest) (cons x rest))))
    
    ;; Example 3: String formatting with variable arguments
    (define format-string
      (case-lambda
        ((template) template)
        ((template arg) (string-append template (if (string? arg) arg (object->string arg))))
        ((template . args) 
         (fold-left (lambda (acc arg)
                      (string-append acc " " (if (string? arg) arg (object->string arg))))
                    template args))))
    
    ;; Helper function for the format-string example
    (define (object->string obj)
      (cond
        ((string? obj) obj)
        ((number? obj) (number->string obj))
        ((symbol? obj) (symbol->string obj))
        (else "#<object>")))
    
    (define (fold-left proc init lst)
      (if (null? lst)
          init
          (fold-left proc (proc init (car lst)) (cdr lst))))))