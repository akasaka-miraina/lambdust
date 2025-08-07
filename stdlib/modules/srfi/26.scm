;; SRFI-26: Notation for Specializing Parameters (cut/cute)
;; 
;; This library provides the `cut` and `cute` macros for creating specialized
;; procedures by "cutting" slots in procedure calls. This is extremely useful
;; for functional programming and creating concise, readable code.
;;
;; Reference: https://srfi.schemers.org/srfi-26/srfi-26.html

(define-library (srfi 26)
  (import (scheme base))
  
  (export cut cute <> <...>)

  (begin
    ;; The slot placeholder symbols
    (define <> '<>)        ; Slot placeholder for single arguments
    (define <...> '<...>)  ; Rest slot placeholder for remaining arguments
    
    ;; Main cut macro - creates a procedure with slots filled lazily
    ;;
    ;; Syntax: (cut <slot-or-expr> <slot-or-expr> ...)
    ;;
    ;; Where <slot-or-expr> can be:
    ;; - <> : a slot to be filled when the resulting procedure is called
    ;; - <...> : a rest slot to receive all remaining arguments
    ;; - any expression : evaluated when cut is evaluated (not when called)
    ;;
    ;; Examples:
    ;; (cut + <> 5)        => (lambda (x) (+ x 5))
    ;; (cut list <> <> <>) => (lambda (x y z) (list x y z))
    ;; (cut + <...>)       => (lambda args (apply + args))
    ;; (cut list 'a <> 'c <...>) => (lambda (x . rest) (apply list 'a x 'c rest))
    
    (define-syntax cut
      (syntax-rules (<> <...>)
        ;; No arguments - just call the procedure
        ((_ proc)
         (lambda () (proc)))
        
        ;; Single slot
        ((_ proc <>)
         (lambda (x) (proc x)))
        
        ;; Two slots
        ((_ proc <> <>)
         (lambda (x y) (proc x y)))
        
        ;; Three slots  
        ((_ proc <> <> <>)
         (lambda (x y z) (proc x y z)))
        
        ;; Four slots
        ((_ proc <> <> <> <>)
         (lambda (w x y z) (proc w x y z)))
        
        ;; Expression followed by slot
        ((_ proc expr <>)
         (lambda (x) (proc expr x)))
        
        ;; Slot followed by expression
        ((_ proc <> expr)
         (lambda (x) (proc x expr)))
        
        ;; Expression, slot, expression
        ((_ proc expr1 <> expr2)
         (lambda (x) (proc expr1 x expr2)))
        
        ;; Two expressions, one slot
        ((_ proc expr1 expr2 <>)
         (lambda (x) (proc expr1 expr2 x)))
        
        ;; Rest arguments only
        ((_ proc <...>)
         (lambda args (apply proc args)))
        
        ;; Slot followed by rest
        ((_ proc <> <...>)
         (lambda (x . rest) (apply proc x rest)))
        
        ;; Two slots followed by rest
        ((_ proc <> <> <...>)
         (lambda (x y . rest) (apply proc x y rest)))
        
        ;; Expression followed by rest
        ((_ proc expr <...>)
         (lambda rest (apply proc expr rest)))
        
        ;; Slot, expression, rest
        ((_ proc <> expr <...>)
         (lambda (x . rest) (apply proc x expr rest)))
        
        ;; Expression, slot, rest
        ((_ proc expr <> <...>)
         (lambda (x . rest) (apply proc expr x rest)))
        
        ;; General case - use helper macro
        ((_ proc arg ...)
         (cut-general proc (arg ...)))))
    
    ;; Helper macro for general cut cases
    (define-syntax cut-general
      (syntax-rules (<> <...>)
        ((_ proc args)
         (cut-analyze-args proc args () () #f))))
    
    ;; Analyze arguments and build the appropriate lambda
    (define-syntax cut-analyze-args
      (syntax-rules (<> <...>)
        ;; End of arguments - no rest slot
        ((_ proc () (param ...) (arg ...) #f)
         (lambda (param ...) (proc arg ...)))
        
        ;; End of arguments - with rest slot
        ((_ proc () (param ...) (arg ...) #t)
         (lambda (param ... . rest-args) (apply proc arg ... rest-args)))
        
        ;; Process slot placeholder
        ((_ proc (<> . rest-args) (param ...) (arg ...) rest-flag)
         (cut-analyze-args proc rest-args (param ... new-param) (arg ... new-param) rest-flag))
        
        ;; Process rest slot placeholder
        ((_ proc (<...> . rest-args) (param ...) (arg ...) #f)
         (cut-analyze-args proc rest-args (param ...) (arg ...) #t))
        
        ;; Process regular expression
        ((_ proc (expr . rest-args) (param ...) (arg ...) rest-flag)
         (cut-analyze-args proc rest-args (param ...) (arg ... expr) rest-flag))))
    
    ;; Macro to generate unique parameter names
    (define-syntax new-param
      (syntax-rules ()
        ((_) (gensym))))
    
    ;; cute macro - like cut, but non-slot expressions are evaluated immediately
    ;;
    ;; Syntax: (cute <slot-or-expr> <slot-or-expr> ...)
    ;;
    ;; The difference from cut is that non-slot expressions are evaluated
    ;; when cute is evaluated, not when the resulting procedure is called.
    ;; This can be important for side effects and for "capturing" current values.
    ;;
    ;; Examples:
    ;; (cute + <> (random 10))  ; random is called once when cute is evaluated
    ;; (cut + <> (random 10))   ; random is called each time the procedure is called
    
    (define-syntax cute
      (syntax-rules (<> <...>)
        ;; No arguments
        ((_ proc)
         (lambda () (proc)))
        
        ;; Single slot - same as cut
        ((_ proc <>)
         (lambda (x) (proc x)))
        
        ;; Two slots - same as cut
        ((_ proc <> <>)
         (lambda (x y) (proc x y)))
        
        ;; Expression followed by slot - evaluate expression immediately
        ((_ proc expr <>)
         (let ((val expr)) (lambda (x) (proc val x))))
        
        ;; Slot followed by expression - evaluate expression immediately
        ((_ proc <> expr)
         (let ((val expr)) (lambda (x) (proc x val))))
        
        ;; Two expressions and a slot - evaluate both immediately
        ((_ proc expr1 expr2 <>)
         (let ((val1 expr1) (val2 expr2)) (lambda (x) (proc val1 val2 x))))
        
        ;; Expression, slot, expression - evaluate expressions immediately
        ((_ proc expr1 <> expr2)
         (let ((val1 expr1) (val2 expr2)) (lambda (x) (proc val1 x val2))))
        
        ;; Rest arguments only - same as cut
        ((_ proc <...>)
         (lambda args (apply proc args)))
        
        ;; Expression followed by rest - evaluate expression immediately
        ((_ proc expr <...>)
         (let ((val expr)) (lambda rest (apply proc val rest))))
        
        ;; Slot followed by rest - same as cut
        ((_ proc <> <...>)
         (lambda (x . rest) (apply proc x rest)))
        
        ;; Expression, slot, rest - evaluate expression immediately
        ((_ proc expr <> <...>)
         (let ((val expr)) (lambda (x . rest) (apply proc val x rest))))
        
        ;; General case - use helper macro
        ((_ proc arg ...)
         (cute-general proc (arg ...)))))
    
    ;; Helper macro for general cute cases
    (define-syntax cute-general
      (syntax-rules (<> <...>)
        ((_ proc args)
         (cute-analyze-args proc args () () () #f))))
    
    ;; Analyze arguments for cute - separate expressions from slots
    (define-syntax cute-analyze-args
      (syntax-rules (<> <...>)
        ;; End of arguments - no rest slot
        ((_ proc () (param ...) (binding ...) (arg ...) #f)
         (let (binding ...) (lambda (param ...) (proc arg ...))))
        
        ;; End of arguments - with rest slot
        ((_ proc () (param ...) (binding ...) (arg ...) #t)
         (let (binding ...) (lambda (param ... . rest-args) (apply proc arg ... rest-args))))
        
        ;; Process slot placeholder - no binding needed
        ((_ proc (<> . rest-args) (param ...) (binding ...) (arg ...) rest-flag)
         (cute-analyze-args proc rest-args (param ... new-param) (binding ...) (arg ... new-param) rest-flag))
        
        ;; Process rest slot placeholder
        ((_ proc (<...> . rest-args) (param ...) (binding ...) (arg ...) #f)
         (cute-analyze-args proc rest-args (param ...) (binding ...) (arg ...) #t))
        
        ;; Process regular expression - create binding
        ((_ proc (expr . rest-args) (param ...) (binding ...) (arg ...) rest-flag)
         (cute-analyze-args proc rest-args (param ...) (binding ... (temp-var expr)) (arg ... temp-var) rest-flag))))
    
    ;; Macro to generate unique temporary variable names
    (define-syntax temp-var
      (syntax-rules ()
        ((_) (gensym))))
    
    ;; Simple gensym implementation for generating unique symbols
    ;; In a full implementation, this would be provided by the system
    (define gensym-counter 0)
    
    (define (gensym . prefix)
      (set! gensym-counter (+ gensym-counter 1))
      (if (null? prefix)
          (string->symbol (string-append "g" (number->string gensym-counter)))
          (string->symbol (string-append 
                          (symbol->string (car prefix))
                          (number->string gensym-counter)))))
    
    ;; Simplified implementations for common cases
    ;; These are more efficient and easier to understand
    
    (define-syntax cut-simple
      (syntax-rules (<> <...>)
        ;; Most common patterns with direct implementations
        
        ;; Single slot
        ((_ proc <>)
         (lambda (x) (proc x)))
        
        ;; Two slots
        ((_ proc <> <>)
         (lambda (x y) (proc x y)))
        
        ;; Expression and slot
        ((_ proc expr <>)
         (lambda (x) (proc expr x)))
        
        ;; Slot and expression
        ((_ proc <> expr)
         (lambda (x) (proc x expr)))
        
        ;; Variadic
        ((_ proc <...>)
         (lambda args (apply proc args)))
        
        ;; Slot and variadic
        ((_ proc <> <...>)
         (lambda (x . rest) (apply proc x rest)))))
    
    (define-syntax cute-simple
      (syntax-rules (<> <...>)
        ;; Most common patterns with direct implementations
        
        ;; Single slot
        ((_ proc <>)
         (lambda (x) (proc x)))
        
        ;; Two slots
        ((_ proc <> <>)
         (lambda (x y) (proc x y)))
        
        ;; Expression and slot - evaluate expression immediately
        ((_ proc expr <>)
         (let ((val expr)) (lambda (x) (proc val x))))
        
        ;; Slot and expression - evaluate expression immediately
        ((_ proc <> expr)
         (let ((val expr)) (lambda (x) (proc x val))))
        
        ;; Variadic
        ((_ proc <...>)
         (lambda args (apply proc args)))
        
        ;; Expression and variadic - evaluate expression immediately
        ((_ proc expr <...>)
         (let ((val expr)) (lambda args (apply proc val args))))))
    
    ;; Example usage and test cases
    
    ;; Arithmetic examples
    (define add-one (cut + <> 1))
    (define multiply-by-two (cut * <> 2))
    (define subtract-from-ten (cut - 10 <>))
    
    ;; List processing examples
    (define get-first (cut car <>))
    (define get-rest (cut cdr <>))
    (define prepend-hello (cut cons 'hello <>))
    
    ;; String examples (assuming string procedures are available)
    ;; (define add-suffix (cute string-append <> ".txt"))
    ;; (define starts-with-a? (cut string-prefix? "a" <>))
    
    ;; Higher-order function examples
    (define map-add-one (cut map add-one <>))
    (define filter-positive (cut filter positive? <>))
    
    ;; Variadic examples
    (define my-list (cut list <...>))
    (define sum-all (cut apply + <...>))
    (define max-of-all (cut apply max <...>))
    
    ;; Demonstrating cute vs cut difference
    (define current-time-cut (cut cons 'timestamp <>))      ; evaluates current time each call
    ;; (define current-time-cute (cute cons (current-time) <>))  ; evaluates current time once
    
    ;; Complex examples showing multiple slots and expressions
    (define between? 
      (lambda (min max)
        (cut and (>= <> min) (<= <> max))))
    
    (define make-range-checker
      (lambda (min max)
        (cute and (>= <> min) (<= <> max))))  ; captures min/max values immediately
    
    ;; Composition examples
    (define compose-functions
      (lambda (f g)
        (cut f (g <>))))
    
    ;; Pipeline-style processing
    (define process-number
      (lambda (x)
        ((compose-functions 
          (cut * <> 2)
          (cut + <> 1)) x)))