;; Lambdust Bootstrap Library
;; Pure Scheme implementations of essential functions using minimal Rust primitives
;;
;; This is the main bootstrap module that provides all essential higher-order functions
;; and list utilities implemented in pure Scheme. It serves as the foundation for the
;; migration from Rust implementations to Scheme implementations.
;;
;; Architecture:
;; - Uses only minimal Rust primitives as defined in MINIMAL_PRIMITIVE_SYSTEM_DESIGN.md
;; - Provides exact R7RS semantics and error handling
;; - Serves as template for future migrations
;; - Maintains performance characteristics where possible

(define-module (:: bootstrap)
  (metadata
    (version "1.0.0")
    (description "Bootstrap library with pure Scheme implementations of essential functions")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.4")
    (bootstrap-level "foundation")
    (migration-status "complete"))
  
  (export 
    ;; ============= HIGHER-ORDER FUNCTIONS =============
    ;; Core functional programming primitives
    map for-each filter fold-left fold-right
    
    ;; ============= LIST UTILITIES =============
    ;; Essential list manipulation functions
    append reverse length
    
    ;; ============= LIST SEARCHING =============
    ;; Membership and searching functions
    member memq memv
    
    ;; ============= UTILITY PREDICATES =============
    ;; Helper functions for bootstrap system
    %proper-list? %list-length
    
    ;; ============= COMPATIBILITY LAYER =============
    ;; Functions to ease migration
    %bootstrap-version %migration-info
    
    ;; ============= I/O OPERATIONS =============
    ;; Re-export I/O operations from io-operations module
    read-line write-string newline
    read write display
    char-ready? u8-ready? close-port
    call-with-input-file call-with-output-file
    with-input-from-file with-output-to-file))

;; ============= BOOTSTRAP METADATA =============

(define (%bootstrap-version)
  "Returns version information for the bootstrap system."
  '((version . "1.0.0")
    (migration-date . "2025-08-05")
    (functions-migrated . (map for-each filter fold-left fold-right 
                           append reverse length member memq memv))
    (primitives-used . (cons car cdr null? pair? procedure? eq? eqv? equal? 
                        + - * / = < > error begin if cond let lambda))))

(define (%migration-info)
  "Returns information about the migration process."
  '((migration-strategy . "minimal-primitives-plus-scheme")
    (performance-target . "maintain-or-improve")
    (compatibility . "exact-r7rs-semantics")
    (error-handling . "identical-to-rust-implementation")
    (testing-strategy . "comprehensive-validation")))

;; ============= UTILITY FUNCTIONS =============
;; Core utilities needed by multiple functions

(define (%proper-list? obj)
  "Internal: Check if object is a proper list using Floyd's cycle detection.
   
   This function uses Floyd's tortoise and hare algorithm to detect cycles
   and ensure the list is properly terminated with nil. It's used throughout
   the bootstrap system for argument validation.
   
   Returns: #t if obj is a proper finite list, #f otherwise."
  (cond
    ;; Empty list is proper
    ((null? obj) #t)
    ;; Non-pairs are not lists
    ((not (pair? obj)) #f)
    ;; Use Floyd's algorithm for cycle detection
    (else
      (let loop ((tortoise obj)
                 (hare obj)
                 (moved-hare? #f))
        (cond
          ;; Hare reached non-pair (improper list)
          ((not (pair? hare)) #f)
          ;; Hare reached proper termination
          ((null? (cdr hare)) #t)
          ;; Hare's next is non-pair (improper termination)
          ((not (pair? (cdr hare))) #f)
          ;; Hare reached proper termination in two steps
          ((null? (cddr hare)) #t)
          ;; Cycle detected (tortoise caught up to hare)
          ((and moved-hare? (eq? tortoise hare)) #f)
          ;; Continue algorithm
          (else
            (loop (cdr tortoise) (cddr hare) #t)))))))

(define (%list-length obj)
  "Internal: Get length of proper list, or #f if not proper.
   
   This function efficiently computes the length of a list after verifying
   it's proper. Used for optimization and validation throughout the system.
   
   Returns: Non-negative integer length, or #f if not a proper list."
  (if (not (%proper-list? obj))
      #f
      (let loop ((lst obj) (count 0))
        (if (null? lst)
            count
            (loop (cdr lst) (+ count 1))))))

(define (%all-lists-proper? lists)
  "Internal: Check if all arguments are proper lists."
  (let loop ((lists lists))
    (cond
      ((null? lists) #t)
      ((%proper-list? (car lists)) (loop (cdr lists)))
      (else #f))))

(define (%validate-procedure proc name)
  "Internal: Validate that argument is a procedure."
  (unless (procedure? proc)
    (error (string-append name ": first argument must be a procedure") proc)))

(define (%validate-proper-list lst name)
  "Internal: Validate that argument is a proper list."
  (unless (%proper-list? lst)
    (error (string-append name ": argument must be a proper list") lst)))

;; ============= HIGHER-ORDER FUNCTIONS =============

(define (map proc list1 . lists)
  "Apply procedure element-wise to elements of lists, returning list of results.
   
   R7RS: (map proc list1 list2 ...) procedure
   
   This is the core functional programming primitive that applies a procedure
   to corresponding elements of one or more lists, collecting the results.
   
   Single list optimization: When only one list is provided, uses a simple
   recursive implementation for better performance.
   
   Multiple lists: Handles multiple lists correctly, terminating when the
   shortest list is exhausted. All lists must be proper lists.
   
   Error handling: Validates procedure and list arguments with descriptive
   error messages matching the Rust implementation exactly."
  
  ;; Argument validation
  (%validate-procedure proc "map")
  (%validate-proper-list list1 "map")
  (unless (%all-lists-proper? lists)
    (error "map: all list arguments must be proper lists" lists))
  
  ;; Single list case (optimized)
  (if (null? lists)
      (map-single proc list1)
      (map-multiple proc (cons list1 lists))))

(define (map-single proc list)
  "Internal: Optimized map for single list case."
  (if (null? list)
      '()
      (cons (proc (car list))
            (map-single proc (cdr list)))))

(define (map-multiple proc lists)
  "Internal: Map for multiple lists case."
  (if (any-null? lists)
      '()
      (cons (apply proc (map car lists))
            (map-multiple proc (map cdr lists)))))

(define (for-each proc list1 . lists)
  "Apply procedure element-wise to elements of lists for side effects.
   
   R7RS: (for-each proc list1 list2 ...) procedure
   
   Like map, but called for side effects only. The return value is unspecified
   (this implementation returns #f to match the Rust implementation).
   
   Guarantees left-to-right evaluation order for side effects."
  
  ;; Argument validation
  (%validate-procedure proc "for-each")
  (%validate-proper-list list1 "for-each")
  (unless (%all-lists-proper? lists)
    (error "for-each: all list arguments must be proper lists" lists))
  
  ;; Single list case (optimized)
  (if (null? lists)
      (for-each-single proc list1)
      (for-each-multiple proc (cons list1 lists)))
  
  ;; Return unspecified value (using #f)
  (if #f #f))

(define (for-each-single proc list)
  "Internal: Optimized for-each for single list case."
  (unless (null? list)
    (proc (car list))
    (for-each-single proc (cdr list))))

(define (for-each-multiple proc lists)
  "Internal: For-each for multiple lists case."
  (unless (any-null? lists)
    (apply proc (map car lists))
    (for-each-multiple proc (map cdr lists))))

(define (filter pred list)
  "Return list of elements from list that satisfy predicate.
   
   R7RS: (filter pred list) procedure
   
   Returns a newly allocated list of elements from list for which pred
   returns a true value, in the same order as they appeared in list."
  
  ;; Argument validation
  (%validate-procedure pred "filter")
  (%validate-proper-list list "filter")
  
  (filter-internal pred list))

(define (filter-internal pred list)
  "Internal: Core filter logic using tail recursion."
  (if (null? list)
      '()
      (let ((element (car list))
            (rest (filter-internal pred (cdr list))))
        (if (pred element)
            (cons element rest)
            rest))))

(define (fold-left proc init list1 . lists)
  "Left fold over lists with initial accumulator value.
   
   R7RS: (fold-left proc init list1 list2 ...) procedure
   
   Applies proc to elements from left to right with accumulator.
   Single list: (proc element accumulator)
   Multiple lists: (proc element1 element2 ... accumulator)"
  
  ;; Argument validation
  (%validate-procedure proc "fold-left")
  (%validate-proper-list list1 "fold-left")
  (unless (%all-lists-proper? lists)
    (error "fold-left: all list arguments must be proper lists" lists))
  
  ;; Single list case (optimized)
  (if (null? lists)
      (fold-left-single proc init list1)
      (fold-left-multiple proc init (cons list1 lists))))

(define (fold-left-single proc acc list)
  "Internal: Optimized fold-left for single list case."
  (if (null? list)
      acc
      (fold-left-single proc 
                        (proc (car list) acc) 
                        (cdr list))))

(define (fold-left-multiple proc acc lists)
  "Internal: Fold-left for multiple lists case."
  (if (any-null? lists)
      acc
      (fold-left-multiple proc 
                          (apply proc (append (map car lists) (list acc)))
                          (map cdr lists))))

(define (fold-right proc init list1 . lists)
  "Right fold over lists with initial accumulator value.
   
   R7RS: (fold-right proc init list1 list2 ...) procedure
   
   Applies proc to elements from right to left with accumulator.
   Ensures proper right-associative evaluation order."
  
  ;; Argument validation
  (%validate-procedure proc "fold-right")
  (%validate-proper-list list1 "fold-right")
  (unless (%all-lists-proper? lists)
    (error "fold-right: all list arguments must be proper lists" lists))
  
  ;; Single list case (optimized)
  (if (null? lists)
      (fold-right-single proc init list1)
      (fold-right-multiple proc init (cons list1 lists))))

(define (fold-right-single proc init list)
  "Internal: Optimized fold-right for single list case."
  (if (null? list)
      init
      (proc (car list) 
            (fold-right-single proc init (cdr list)))))

(define (fold-right-multiple proc init lists)
  "Internal: Fold-right for multiple lists case."
  (if (any-null? lists)
      init
      (apply proc (append (map car lists) 
                         (list (fold-right-multiple proc init (map cdr lists)))))))

;; ============= LIST UTILITIES =============

(define (length list)
  "Return the length of list.
   
   R7RS: (length list) procedure
   
   Returns the length of list. It is an error if list is not a proper list."
  
  ;; Handle empty list immediately
  (cond
    ((null? list) 0)
    ((not (pair? list))
     (error "length: argument must be a list" list))
    ((not (%proper-list? list))
     (error "length: argument must be a proper list" list)))
  
  ;; Count elements using tail recursion
  (let loop ((lst list) (count 0))
    (if (null? lst)
        count
        (loop (cdr lst) (+ count 1)))))

(define (append . lists)
  "Return list consisting of elements of lists concatenated.
   
   R7RS: (append list ...) procedure
   
   The last argument can be of any type. All other arguments must be proper lists.
   Returns a newly allocated list except it shares structure with the last argument."
  
  (cond
    ;; No arguments
    ((null? lists) '())
    ;; Single argument
    ((null? (cdr lists)) (car lists))
    ;; Multiple arguments
    (else
      ;; Validate all but last argument are proper lists
      (let ((all-but-last (reverse (cdr (reverse lists)))))
        (for-each (lambda (arg)
                    (unless (%proper-list? arg)
                      (error "append: all arguments except the last must be proper lists" arg)))
                  all-but-last))
      
      ;; Perform append operation
      (append-internal lists))))

(define (append-internal lists)
  "Internal: Core append logic with validated arguments."
  (if (null? (cdr lists))
      (car lists)  ; Last argument
      (append-two (car lists) (append-internal (cdr lists)))))

(define (append-two list1 list2)
  "Internal: Append two lists efficiently."
  (if (null? list1)
      list2
      (cons (car list1) (append-two (cdr list1) list2))))

(define (reverse list)
  "Return newly allocated list with elements of list in reverse order.
   
   R7RS: (reverse list) procedure
   
   It is an error if list is not a proper list."
  
  ;; Argument validation
  (cond
    ((null? list) '())
    ((not (pair? list))
     (error "reverse: argument must be a list" list))
    ((not (%proper-list? list))
     (error "reverse: argument must be a proper list" list)))
  
  ;; Iterative reversal
  (let loop ((remaining list) (accumulated '()))
    (if (null? remaining)
        accumulated
        (loop (cdr remaining) (cons (car remaining) accumulated)))))

;; ============= LIST SEARCHING =============

(define (member obj list . compare)
  "Return first sublist of list whose car is obj.
   
   R7RS: (member obj list) procedure
         (member obj list compare) procedure
   
   Uses equal? by default, or custom comparison procedure if provided."
  
  ;; Argument validation
  (%validate-proper-list list "member")
  (when (and (not (null? compare)) (not (procedure? (car compare))))
    (error "member: comparison argument must be a procedure" (car compare)))
  (when (> (length compare) 1)
    (error "member: too many arguments" (cons obj (cons list compare))))
  
  ;; Determine comparison procedure
  (let ((cmp-proc (if (null? compare) equal? (car compare))))
    (member-internal obj list cmp-proc)))

(define (member-internal obj list cmp-proc)
  "Internal: Core member logic."
  (cond
    ((null? list) #f)
    ((cmp-proc obj (car list)) list)
    (else (member-internal obj (cdr list) cmp-proc))))

(define (memq obj list)
  "Return first sublist of list whose car is obj (using eq?).
   
   R7RS: (memq obj list) procedure"
  
  (%validate-proper-list list "memq")
  (memq-internal obj list))

(define (memq-internal obj list)
  "Internal: Core memq logic."
  (cond
    ((null? list) #f)
    ((eq? obj (car list)) list)
    (else (memq-internal obj (cdr list)))))

(define (memv obj list)
  "Return first sublist of list whose car is obj (using eqv?).
   
   R7RS: (memv obj list) procedure"
  
  (%validate-proper-list list "memv")
  (memv-internal obj list))

(define (memv-internal obj list)
  "Internal: Core memv logic."
  (cond
    ((null? list) #f)
    ((eqv? obj (car list)) list)
    (else (memv-internal obj (cdr list)))))

;; ============= UTILITY HELPERS =============

(define (any-null? lists)
  "Internal: Check if any list in lists is null."
  (and (not (null? lists))
       (or (null? (car lists))
           (any-null? (cdr lists)))))

(define (unless condition . body)
  "Execute body unless condition is true."
  (if (not condition)
      (begin . body)))

(define (when condition . body)
  "Execute body when condition is true."
  (if condition
      (begin . body)))

;; ============= MODULE INITIALIZATION =============

;; Validate that we have the required primitives
(unless (procedure? cons) (error "Bootstrap requires 'cons' primitive"))
(unless (procedure? car) (error "Bootstrap requires 'car' primitive"))
(unless (procedure? cdr) (error "Bootstrap requires 'cdr' primitive"))
(unless (procedure? null?) (error "Bootstrap requires 'null?' primitive"))
(unless (procedure? pair?) (error "Bootstrap requires 'pair?' primitive"))
(unless (procedure? procedure?) (error "Bootstrap requires 'procedure?' primitive"))
(unless (procedure? eq?) (error "Bootstrap requires 'eq?' primitive"))
(unless (procedure? eqv?) (error "Bootstrap requires 'eqv?' primitive"))
(unless (procedure? equal?) (error "Bootstrap requires 'equal?' primitive"))
(unless (procedure? error) (error "Bootstrap requires 'error' primitive"))

;; ============= I/O OPERATIONS INTEGRATION =============
;; Import and re-export I/O operations from the io-operations module

;; Import I/O operations (these will be resolved at load time)
(import (:: bootstrap io-operations))

;; Re-export all I/O operations for unified bootstrap interface
;; The actual implementations are provided by the io-operations module

;; Bootstrap initialization complete
(display "Bootstrap library loaded successfully\n")
(display "Pure Scheme implementations available for:\n")
(display "  - Higher-order: map, for-each, filter, fold-left, fold-right\n")
(display "  - List utilities: append, reverse, length\n")
(display "  - List searching: member, memq, memv\n")
(display "  - I/O operations: read-line, write-string, newline, read, write, display\n")
(display "  - File utilities: call-with-input-file, call-with-output-file\n")
(display "  - Port management: char-ready?, u8-ready?, close-port\n")