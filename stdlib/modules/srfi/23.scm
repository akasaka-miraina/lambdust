;; SRFI-23: Error reporting mechanism
;; 
;; This library provides a simple error reporting mechanism that is compatible
;; with R7RS error handling. The main procedure is `error` which raises an
;; exception with a message and optional irritants.
;;
;; Reference: https://srfi.schemers.org/srfi-23/srfi-23.html

(define-library (srfi 23)
  (import (scheme base))
  
  (export error)

  (begin
    ;; The error procedure raises an exception with a message and optional irritants
    ;;
    ;; Syntax: (error message [irritant1 irritant2 ...])
    ;;
    ;; - message: A string describing the error
    ;; - irritants: Optional objects that provide additional context
    ;;
    ;; The error procedure does not return normally - it raises an exception
    ;; that can be caught by exception handlers.
    ;;
    ;; Examples:
    ;; (error "Division by zero")
    ;; (error "Invalid argument" x)
    ;; (error "Type mismatch" 'expected 'string 'got x)
    
    (define (error message . irritants)
      "Raises an exception with the given message and optional irritants.
       
       This is the standard SRFI-23 error reporting procedure. It creates
       an exception object containing the error message and any additional
       objects that might help diagnose the problem."
      
      ;; Create a comprehensive error message
      (let ((full-message 
             (if (null? irritants)
                 message
                 (string-append message 
                               ": " 
                               (irritants->string irritants)))))
        
        ;; In R7RS systems, we use raise to signal the exception
        ;; The exception object includes both the message and irritants
        (raise (make-error-object message irritants full-message))))
    
    ;; Helper function to create error objects
    ;; This creates a structured error object that can be inspected by handlers
    (define (make-error-object message irritants full-message)
      "Creates a structured error object for SRFI-23 compliance."
      (vector 'srfi-23-error message irritants full-message))
    
    ;; Helper function to convert irritants to a readable string
    (define (irritants->string irritants)
      "Converts a list of irritants to a human-readable string representation."
      (if (null? irritants)
          ""
          (let loop ((items irritants) (result ""))
            (cond
              ((null? items) result)
              ((null? (cdr items))
               (string-append result (object->display-string (car items))))
              (else
               (loop (cdr items)
                     (string-append result 
                                    (object->display-string (car items))
                                    " ")))))))
    
    ;; Helper function to convert objects to display strings
    (define (object->display-string obj)
      "Converts an object to a string suitable for error display."
      (cond
        ((string? obj) obj)
        ((symbol? obj) (symbol->string obj))
        ((number? obj) (number->string obj))
        ((boolean? obj) (if obj "#t" "#f"))
        ((null? obj) "()")
        ((pair? obj) (list->display-string obj))
        ((vector? obj) (vector->display-string obj))
        ((procedure? obj) "#<procedure>")
        (else "#<object>")))
    
    ;; Helper function for list display
    (define (list->display-string lst)
      "Converts a list to a display string with proper formatting."
      (define (list->string-helper lst)
        (cond
          ((null? lst) "")
          ((null? (cdr lst))
           (object->display-string (car lst)))
          ((pair? (cdr lst))
           (string-append (object->display-string (car lst))
                          " "
                          (list->string-helper (cdr lst))))
          (else
           (string-append (object->display-string (car lst))
                          " . "
                          (object->display-string (cdr lst))))))
      
      (string-append "(" (list->string-helper lst) ")"))
    
    ;; Helper function for vector display
    (define (vector->display-string vec)
      "Converts a vector to a display string."
      (let* ((len (vector-length vec))
             (elements (let loop ((i 0) (acc '()))
                         (if (= i len)
                             (reverse acc)
                             (loop (+ i 1)
                                   (cons (object->display-string (vector-ref vec i))
                                         acc))))))
        (string-append "#(" (string-join elements " ") ")")))
    
    ;; String join utility
    (define (string-join strings delimiter)
      "Joins a list of strings with the given delimiter."
      (if (null? strings)
          ""
          (let loop ((strs (cdr strings)) (result (car strings)))
            (if (null? strs)
                result
                (loop (cdr strs)
                      (string-append result delimiter (car strs)))))))
    
    ;; Error object predicates and accessors for compatibility
    
    (define (srfi-23-error? obj)
      "Predicate to test if an object is a SRFI-23 error object."
      (and (vector? obj)
           (> (vector-length obj) 0)
           (eq? (vector-ref obj 0) 'srfi-23-error)))
    
    (define (srfi-23-error-message error-obj)
      "Extracts the message from a SRFI-23 error object."
      (if (srfi-23-error? error-obj)
          (vector-ref error-obj 1)
          (error "Not a SRFI-23 error object" error-obj)))
    
    (define (srfi-23-error-irritants error-obj)
      "Extracts the irritants from a SRFI-23 error object."
      (if (srfi-23-error? error-obj)
          (vector-ref error-obj 2)
          (error "Not a SRFI-23 error object" error-obj)))
    
    (define (srfi-23-error-full-message error-obj)
      "Extracts the full formatted message from a SRFI-23 error object."
      (if (srfi-23-error? error-obj)
          (vector-ref error-obj 3)
          (error "Not a SRFI-23 error object" error-obj)))
    
    ;; Common error patterns and utilities
    
    (define (type-error expected-type actual-value . context)
      "Convenience function for type errors."
      (apply error 
             "Type error"
             'expected expected-type
             'got (if (procedure? actual-value) 
                      "#<procedure>"
                      actual-value)
             context))
    
    (define (arity-error procedure-name expected-arity actual-arity . context)
      "Convenience function for arity errors."
      (apply error
             "Arity error"
             'procedure procedure-name
             'expected expected-arity
             'got actual-arity
             context))
    
    (define (range-error value minimum maximum . context)
      "Convenience function for range errors."
      (apply error
             "Range error"
             'value value
             'valid-range (list minimum maximum)
             context))
    
    (define (not-implemented-error feature . context)
      "Convenience function for not-implemented errors."
      (apply error
             "Not implemented"
             'feature feature
             context))
    
    ;; Examples of error usage patterns
    
    ;; Example 1: Simple error with message only
    ;; (error "Something went wrong")
    
    ;; Example 2: Error with context information
    ;; (error "Invalid argument" 'function 'my-proc 'argument x)
    
    ;; Example 3: Using convenience functions
    ;; (type-error 'number "not a number" 'in 'addition)
    ;; (arity-error 'my-function 2 5)
    ;; (range-error -1 0 100 'in 'array-access)
    
    ;; Integration with R7RS exception system
    ;; The error procedure works with R7RS guard expressions:
    ;;
    ;; (guard (condition
    ;;         ((srfi-23-error? condition)
    ;;          (display (srfi-23-error-full-message condition))
    ;;          'handled))
    ;;   (error "Test error" 'with 'irritants))
    
    ;; For systems that need to maintain compatibility with older Scheme
    ;; standards, we also provide a simplified version
    
    (define (simple-error message . irritants)
      "Simplified error procedure that just displays and exits.
       This is for compatibility with systems that don't have
       proper exception handling."
      (display "Error: ")
      (display message)
      (unless (null? irritants)
        (display ": ")
        (display (irritants->string irritants)))
      (newline)
      ;; In a real implementation, this might call (exit 1) or similar
      ;; For now, we still raise the exception
      (raise (make-error-object message irritants 
                                (string-append message 
                                               (if (null? irritants) 
                                                   "" 
                                                   (string-append ": " (irritants->string irritants)))))))
    
    ;; Error handling utilities
    
    (define (with-error-context context thunk)
      "Executes thunk and adds context to any SRFI-23 errors that occur."
      (guard (condition
              ((srfi-23-error? condition)
               (let ((msg (srfi-23-error-message condition))
                     (irritants (srfi-23-error-irritants condition)))
                 (apply error msg 'context context irritants)))
              (else (raise condition)))
        (thunk)))
    
    (define (ignore-errors thunk . default)
      "Executes thunk and returns its result, or the default value if an error occurs."
      (guard (condition
              (else (if (null? default) #f (car default))))
        (thunk)))