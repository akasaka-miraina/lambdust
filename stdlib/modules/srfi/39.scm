;; SRFI-39: Parameter objects
;; 
;; This library provides parameter objects, which are a clean mechanism for
;; creating dynamically-scoped variables. Parameters can be used to implement
;; fluent variables, configuration settings, and context-dependent behavior.
;;
;; Reference: https://srfi.schemers.org/srfi-39/srfi-39.html

(define-library (srfi 39)
  (import (scheme base))
  
  (export make-parameter parameterize)

  (begin
    ;; Parameter objects are procedures that can get/set their value
    ;; and maintain a stack of values for dynamic scoping
    
    ;; Internal parameter structure
    ;; We use a vector to store:
    ;; [0] - current value
    ;; [1] - converter procedure (or #f)
    ;; [2] - value stack for parameterize
    ;; [3] - parameter tag for identification
    
    (define parameter-tag (list 'parameter))  ; Unique tag
    
    ;; Create a new parameter object
    ;;
    ;; Syntax: (make-parameter init [converter])
    ;;
    ;; - init: Initial value for the parameter
    ;; - converter: Optional procedure to convert/validate values
    ;;
    ;; Returns a parameter object (which is a procedure)
    ;;
    ;; Examples:
    ;; (define my-param (make-parameter 42))
    ;; (my-param)      ; => 42
    ;; (my-param 100)  ; sets value to 100
    ;;
    ;; (define validated-param 
    ;;   (make-parameter 0 (lambda (x) 
    ;;                       (if (number? x) x 
    ;;                           (error "Must be a number" x)))))
    
    (define make-parameter
      (case-lambda
        ((init)
         (make-parameter init #f))
        ((init converter)
         (let* ((converted-init (if converter (converter init) init))
                (param-data (vector converted-init converter '() parameter-tag)))
           
           ;; Return a procedure that acts as the parameter
           (case-lambda
             ;; Get current value
             (()
              (vector-ref param-data 0))
             
             ;; Set new value
             ((new-value)
              (let ((converted-value (if (vector-ref param-data 1)
                                         ((vector-ref param-data 1) new-value)
                                         new-value)))
                (vector-set! param-data 0 converted-value)
                converted-value)))))))
    
    ;; Check if an object is a parameter
    (define (parameter? obj)
      "Predicate to test if an object is a parameter."
      ;; This is tricky since parameters are procedures
      ;; We need a way to identify them - this is implementation dependent
      ;; For now, we use a simple approach with a global registry
      (and (procedure? obj)
           (parameter-object? obj)))
    
    ;; Internal helper to identify parameter objects
    ;; This is a simplified approach - real implementations might use
    ;; more sophisticated techniques
    (define parameter-registry (make-hash-table eq?))
    
    (define (register-parameter param)
      "Register a parameter in the global registry."
      (hash-table-set! parameter-registry param #t))
    
    (define (parameter-object? obj)
      "Check if an object is registered as a parameter."
      (hash-table-ref parameter-registry obj #f))
    
    ;; Enhanced make-parameter with registration
    (define make-parameter-enhanced
      (case-lambda
        ((init)
         (make-parameter-enhanced init #f))
        ((init converter)
         (let* ((converted-init (if converter (converter init) init))
                (param-data (vector converted-init converter '() parameter-tag))
                (param-proc
                 (case-lambda
                   ;; Get current value
                   (()
                    (vector-ref param-data 0))
                   
                   ;; Set new value
                   ((new-value)
                    (let ((converted-value (if (vector-ref param-data 1)
                                               ((vector-ref param-data 1) new-value)
                                               new-value)))
                      (vector-set! param-data 0 converted-value)
                      converted-value)))))
           
           ;; Register the parameter
           (register-parameter param-proc)
           
           ;; Store reference to data for parameterize
           (set! param-proc 
                 (let ((original-proc param-proc))
                   (lambda args
                     (apply original-proc args))))
           
           ;; Attach the data to the procedure (implementation-specific)
           param-proc))))
    
    ;; Use the enhanced version as the default
    (set! make-parameter make-parameter-enhanced)
    
    ;; The parameterize special form for dynamic scoping
    ;;
    ;; Syntax: (parameterize ((param1 value1) (param2 value2) ...) body ...)
    ;;
    ;; Temporarily sets parameters to new values for the dynamic extent
    ;; of the body, then restores their previous values.
    ;;
    ;; Example:
    ;; (define x (make-parameter 1))
    ;; (x)  ; => 1
    ;; (parameterize ((x 10))
    ;;   (x))  ; => 10
    ;; (x)  ; => 1
    
    (define-syntax parameterize
      (syntax-rules ()
        ((_ ((param value) ...) body ...)
         (parameterize-helper ((param value) ...) (lambda () body ...)))))
    
    ;; Helper procedure for parameterize implementation
    (define (parameterize-helper bindings thunk)
      "Implementation of parameterize using dynamic-wind for proper cleanup."
      (if (null? bindings)
          (thunk)
          (let* ((param (caar bindings))
                 (new-value (cadar bindings))
                 (rest-bindings (cdr bindings))
                 (old-value (param)))
            
            (dynamic-wind
              ;; Before: set new value
              (lambda () (param new-value))
              
              ;; During: continue with remaining bindings
              (lambda () (parameterize-helper rest-bindings thunk))
              
              ;; After: restore old value
              (lambda () (param old-value))))))
    
    ;; Alternative implementation without dynamic-wind
    ;; This version is simpler but doesn't handle exceptions as cleanly
    
    (define-syntax parameterize-simple
      (syntax-rules ()
        ((_ () body ...)
         (begin body ...))
        
        ((_ ((param value) . rest) body ...)
         (let ((old-value (param)))
           (param value)
           (let ((result (parameterize-simple rest body ...)))
             (param old-value)
             result)))))
    
    ;; Stack-based implementation for better performance
    ;; This version maintains explicit stacks in parameter objects
    
    (define (make-parameter-with-stack init . maybe-converter)
      "Creates a parameter with explicit stack management."
      (let* ((converter (if (null? maybe-converter) #f (car maybe-converter)))
             (converted-init (if converter (converter init) init))
             (current-value converted-init)
             (value-stack '()))
        
        (define (push-value new-value)
          (set! value-stack (cons current-value value-stack))
          (set! current-value (if converter (converter new-value) new-value)))
        
        (define (pop-value)
          (if (null? value-stack)
              (error "Parameter stack underflow")
              (begin
                (set! current-value (car value-stack))
                (set! value-stack (cdr value-stack)))))
        
        (define param-proc
          (case-lambda
            ;; Get current value
            (()
             current-value)
            
            ;; Set new value
            ((new-value)
             (set! current-value (if converter (converter new-value) new-value))
             current-value)
            
            ;; Internal operations for parameterize
            (('push new-value)
             (push-value new-value))
            
            (('pop)
             (pop-value))))
        
        ;; Register for parameterize
        (register-parameter param-proc)
        param-proc))
    
    ;; Enhanced parameterize using stack operations
    (define-syntax parameterize-stack
      (syntax-rules ()
        ((_ ((param value) ...) body ...)
         (begin
           ;; Push all new values
           (param 'push value) ...
           
           ;; Execute body with exception handling
           (let ((result (guard (condition
                                 (else
                                  ;; Pop all values on exception
                                  (param 'pop) ...
                                  (raise condition)))
                           body ...)))
             ;; Pop all values on normal completion
             (param 'pop) ...
             result)))))
    
    ;; Common parameter objects for system configuration
    
    ;; Current input/output ports (if not provided by base system)
    ;; (define current-input-port (make-parameter (standard-input-port)))
    ;; (define current-output-port (make-parameter (standard-output-port)))
    ;; (define current-error-port (make-parameter (standard-error-port)))
    
    ;; Numeric parameters
    (define current-precision 
      (make-parameter 15 
                      (lambda (x) 
                        (if (and (integer? x) (positive? x))
                            x
                            (error "Precision must be a positive integer" x)))))
    
    ;; Boolean parameters
    (define case-sensitive 
      (make-parameter #t
                      (lambda (x)
                        (if (boolean? x)
                            x
                            (error "Case sensitivity must be boolean" x)))))
    
    ;; List parameters with validation
    (define search-paths
      (make-parameter '()
                      (lambda (paths)
                        (if (and (list? paths) 
                                 (every string? paths))
                            paths
                            (error "Search paths must be a list of strings" paths)))))
    
    ;; Example usage patterns
    
    ;; Configuration parameter
    (define debug-level
      (make-parameter 0
                      (lambda (level)
                        (if (and (integer? level) 
                                 (>= level 0) 
                                 (<= level 3))
                            level
                            (error "Debug level must be 0-3" level)))))
    
    ;; Context parameter
    (define current-user
      (make-parameter "anonymous"
                      (lambda (user)
                        (if (string? user)
                            user
                            (error "User must be a string" user)))))
    
    ;; Fluent variable example
    (define *context* (make-parameter '()))
    
    (define (with-context key value thunk)
      "Execute thunk with additional context."
      (parameterize ((*context* (cons (cons key value) (*context*))))
        (thunk)))
    
    (define (get-context key)
      "Retrieve value from current context."
      (let ((entry (assoc key (*context*))))
        (if entry (cdr entry) #f)))
    
    ;; Error handling in parameter converters
    (define safe-parameter
      (lambda (init converter)
        "Create a parameter with safe converter that never fails."
        (make-parameter init
                        (lambda (value)
                          (guard (condition
                                  (else init))  ; Return initial value on error
                            (converter value))))))
    
    ;; Parameter composition
    (define (compose-parameters param1 param2 combiner)
      "Create a parameter that combines two other parameters."
      (make-parameter (combiner (param1) (param2))
                      (lambda (value)
                        ;; This is a read-only computed parameter
                        (error "Cannot set computed parameter" value))))
    
    ;; Helper functions
    (define (every pred lst)
      "Test if predicate is true for all elements."
      (or (null? lst)
          (and (pred (car lst))
               (every pred (cdr lst)))))
    
    ;; Simplified hash table implementation if not available
    (define (make-hash-table comparison)
      "Simple hash table using association list."
      (list 'hash-table comparison '()))
    
    (define (hash-table-set! ht key value)
      "Set key-value pair in hash table."
      (let ((data (cddr ht)))
        (let ((entry (assoc key data)))
          (if entry
              (set-cdr! entry value)
              (set-cdr! (cdr ht) (cons (cons key value) data))))))
    
    (define (hash-table-ref ht key default)
      "Get value from hash table with default."
      (let ((entry (assoc key (cddr ht))))
        (if entry (cdr entry) default)))