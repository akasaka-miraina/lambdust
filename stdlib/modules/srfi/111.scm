;; SRFI-111: Boxes
;; 
;; This library provides boxes, which are single-cell mutable containers.
;; Boxes provide a simple and efficient way to create mutable references
;; that can be shared between different parts of a program. They are
;; particularly useful in functional programming for controlled mutability.
;;
;; Reference: https://srfi.schemers.org/srfi-111/srfi-111.html

(define-library (srfi 111)
  (import (scheme base))
  
  (export
    ;; === Box Type ===
    box box? unbox set-box!
    
    ;; === Convenience Procedures ===
    box-cas! box-swap!)

  (begin
    ;; ============= BOX TYPE AND OPERATIONS =============
    
    ;; Box type tag for runtime type checking
    (define box-type-tag 'box)
    
    ;; Box constructor
    ;; Creates a new box containing the given value
    ;; Usage: (box value) -> box
    (define (box value)
      "Create a new box containing value."
      (vector box-type-tag value))
    
    ;; Box type predicate
    ;; Tests whether an object is a box
    ;; Usage: (box? obj) -> boolean
    (define (box? obj)
      "Test whether obj is a box."
      (and (vector? obj)
           (= (vector-length obj) 2)
           (eq? (vector-ref obj 0) box-type-tag)))
    
    ;; Box accessor
    ;; Returns the value contained in the box
    ;; Usage: (unbox box) -> any
    (define (unbox box)
      "Return the value contained in box."
      (if (box? box)
          (vector-ref box 1)
          (error "unbox: not a box" box)))
    
    ;; Box mutator
    ;; Changes the value contained in the box
    ;; Usage: (set-box! box new-value) -> unspecified
    (define (set-box! box new-value)
      "Set the value contained in box to new-value."
      (if (box? box)
          (vector-set! box 1 new-value)
          (error "set-box!: not a box" box)))
    
    ;; ============= CONVENIENCE PROCEDURES =============
    
    ;; Compare-and-swap operation
    ;; Atomically compares the box's current value with expected-value
    ;; and if they are the same, sets the box to new-value
    ;; Returns #t if the swap occurred, #f otherwise
    ;; Usage: (box-cas! box expected-value new-value) -> boolean
    ;;
    ;; Note: In a single-threaded implementation, this is straightforward.
    ;; In a multi-threaded implementation, this would need to be atomic.
    (define (box-cas! box expected-value new-value)
      "Compare-and-swap: if box contains expected-value, set it to new-value."
      (if (not (box? box))
          (error "box-cas!: not a box" box)
          (let ((current (unbox box)))
            (if (equal? current expected-value)
                (begin
                  (set-box! box new-value)
                  #t)
                #f))))
    
    ;; Swap operation with function
    ;; Applies the function to the current value and stores the result
    ;; Returns the old value
    ;; Usage: (box-swap! box proc) -> any
    (define (box-swap! box proc)
      "Apply proc to box's value, store result, return old value."
      (if (not (box? box))
          (error "box-swap!: not a box" box)
          (let ((old-value (unbox box)))
            (set-box! box (proc old-value))
            old-value)))
    
    ;; ============= USAGE EXAMPLES =============
    
    ;; Example 1: Simple counter
    ;; (define counter (box 0))
    ;; (define (increment!) (box-swap! counter (lambda (n) (+ n 1))))
    ;; (define (get-count) (unbox counter))
    ;; (increment!) ; returns 0, counter now contains 1
    ;; (get-count)  ; returns 1
    
    ;; Example 2: Shared mutable state
    ;; (define shared-data (box '()))
    ;; (define (add-item! item)
    ;;   (set-box! shared-data (cons item (unbox shared-data))))
    ;; (define (get-items) (unbox shared-data))
    ;; (add-item! 'a)
    ;; (add-item! 'b)
    ;; (get-items) ; returns '(b a)
    
    ;; Example 3: Memoization
    ;; (define (make-memoized-proc proc)
    ;;   (let ((cache (box '())))
    ;;     (lambda (arg)
    ;;       (let ((cached (assoc arg (unbox cache))))
    ;;         (if cached
    ;;             (cdr cached)
    ;;             (let ((result (proc arg)))
    ;;               (set-box! cache (cons (cons arg result) (unbox cache)))
    ;;               result))))))
    
    ;; Example 4: Optional reference
    ;; (define maybe-value (box #f))
    ;; (define (set-value! v) (set-box! maybe-value v))
    ;; (define (has-value?) (not (eq? #f (unbox maybe-value))))
    ;; (define (get-value) 
    ;;   (let ((v (unbox maybe-value)))
    ;;     (if (eq? v #f)
    ;;         (error "No value set")
    ;;         v)))
    
    ;; Example 5: State machine
    ;; (define (make-state-machine initial-state transitions)
    ;;   (let ((current-state (box initial-state)))
    ;;     (lambda (event)
    ;;       (let* ((state (unbox current-state))
    ;;              (transition (assoc (cons state event) transitions)))
    ;;         (if transition
    ;;             (let ((new-state (cdr transition)))
    ;;               (set-box! current-state new-state)
    ;;               new-state)
    ;;             (error "Invalid transition" state event))))))
    
    ;; Example 6: Lazy evaluation with boxes
    ;; (define (make-lazy-box thunk)
    ;;   (let ((computed? (box #f))
    ;;         (value (box #f)))
    ;;     (lambda ()
    ;;       (if (unbox computed?)
    ;;           (unbox value)
    ;;           (let ((result (thunk)))
    ;;             (set-box! value result)
    ;;             (set-box! computed? #t)
    ;;             result)))))
    
    ;; ============= INTEGRATION PATTERNS =============
    
    ;; Pattern: Boxes with SRFI-1 list operations
    ;; (define (box-map proc box-list)
    ;;   (map (lambda (b) (set-box! b (proc (unbox b)))) box-list))
    
    ;; Pattern: Boxes with SRFI-26 cut/cute
    ;; (define increment-box! (cute box-swap! <> (cut + <> 1)))
    ;; (define reset-box! (cut set-box! <> 0))
    
    ;; Pattern: Boxes as accumulators
    ;; (define (make-accumulator initial)
    ;;   (let ((acc (box initial)))
    ;;     (lambda (value)
    ;;       (box-swap! acc (lambda (current) (+ current value))))))
    
    ;; Pattern: Boxes for error handling
    ;; (define error-box (box #f))
    ;; (define (with-error-capture thunk)
    ;;   (guard (condition
    ;;           (else (set-box! error-box condition) #f))
    ;;     (let ((result (thunk)))
    ;;       (set-box! error-box #f)
    ;;       result)))
    
    ;; ============= PERFORMANCE NOTES =============
    
    ;; Boxes in this implementation use vectors for storage, providing:
    ;; - Constant-time access and mutation operations
    ;; - Minimal memory overhead (just the type tag)
    ;; - Efficient representation compatible with most Scheme systems
    
    ;; Performance characteristics:
    ;; - box: O(1) - creates a 2-element vector
    ;; - box?: O(1) - vector type check and length check  
    ;; - unbox: O(1) - vector-ref operation
    ;; - set-box!: O(1) - vector-set! operation
    ;; - box-cas!: O(1) - unbox + equal? + set-box!
    ;; - box-swap!: O(1) + time complexity of proc
    
    ;; Memory considerations:
    ;; - Each box uses 2 vector slots plus vector overhead
    ;; - Boxes themselves are small; contained values determine memory usage
    ;; - No automatic garbage collection of box contents until box is GC'd
    ;; - Shared boxes can prevent GC of large data structures
    
    ;; Thread safety notes:
    ;; - In single-threaded implementations, all operations are safe
    ;; - box-cas! provides compare-and-swap semantics for eventual MT support
    ;; - In multi-threaded contexts, external synchronization may be needed
    ;; - Future implementations might provide atomic operations
    
    ;; ============= TYPE SYSTEM INTEGRATION =============
    
    ;; For Lambdust's type system, boxes could be typed as:
    ;; (box-of T) where T is the type of contained value
    ;; This would enable static type checking of box operations
    
    ;; Example type annotations (hypothetical):
    ;; (: counter (box-of integer))
    ;; (: increment! (-> (box-of integer) integer))
    ;; (: make-accumulator (-> number (-> number number)))
    
    ;; ============= ERROR HANDLING =============
    
    ;; All box operations include proper error checking:
    ;; - Type errors for non-box arguments
    ;; - Informative error messages
    ;; - Consistent error reporting across operations
    
    ;; Common error scenarios:
    ;; - Passing non-box to unbox, set-box!, box-cas!, or box-swap!
    ;; - These are programming errors that should be caught during development
    ;; - Runtime type checking helps catch these early
    
    ;; ============= COMPATIBILITY =============
    
    ;; This implementation is compatible with:
    ;; - R7RS-small (uses only basic vector operations)
    ;; - Most Scheme implementations
    ;; - Both interpreted and compiled environments
    ;; - Environments with or without threads (different semantics for box-cas!)
    
    ;; Portability considerations:
    ;; - Uses standard vector operations for maximum compatibility
    ;; - No implementation-specific features required
    ;; - Works with any Scheme supporting R7RS-small
    ;; - Type tag system is simple and reliable across implementations
    ))