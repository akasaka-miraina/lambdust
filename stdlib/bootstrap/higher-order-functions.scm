;; Bootstrap Higher-Order Functions
;; Pure Scheme implementations using minimal Rust primitives
;; 
;; This module provides essential higher-order functions implemented in pure Scheme
;; using only the minimal primitive set. These functions form the foundation for
;; functional programming patterns in the standard library.

;; ============= CORE HIGHER-ORDER FUNCTIONS =============

;; MAP - Apply procedure to corresponding elements of lists
(define (map proc lst . lsts)
  "Apply proc to corresponding elements of lists, return list of results."
  (if (null? lsts)
      ;; Single list case (most common)
      (if (null? lst)
          '()
          (%cons (proc (%car lst)) (map proc (%cdr lst))))
      ;; Multiple lists case
      (if (or (null? lst) (any-null? lsts))
          '()
          (%cons (apply proc (%cons (%car lst) (map-cars lsts)))
                 (apply map proc (%cons (%cdr lst) (map-cdrs lsts)))))))

;; FOR-EACH - Apply procedure for side effects
(define (for-each proc lst . lsts)
  "Apply proc to corresponding elements of lists for side effects."
  (if (null? lsts)
      ;; Single list case
      (unless (null? lst)
        (proc (%car lst))
        (for-each proc (%cdr lst)))
      ;; Multiple lists case
      (unless (or (null? lst) (any-null? lsts))
        (apply proc (%cons (%car lst) (map-cars lsts)))
        (apply for-each proc (%cons (%cdr lst) (map-cdrs lsts)))))
  ;; Return unspecified value
  (if #f #f))

;; FILTER - Select elements satisfying predicate
(define (filter pred lst)
  "Return list of elements from lst that satisfy predicate pred."
  (cond
    ((null? lst) '())
    ((pred (%car lst)) (%cons (%car lst) (filter pred (%cdr lst))))
    (else (filter pred (%cdr lst)))))

;; FOLD-LEFT - Left fold with accumulator
(define (fold-left proc init lst . lsts)
  "Left fold: accumulate from left to right."
  (if (null? lsts)
      ;; Single list case
      (fold-left-single proc init lst)
      ;; Multiple lists case
      (fold-left-multiple proc init (%cons lst lsts))))

(define (fold-left-single proc acc lst)
  "Internal: fold-left for single list."
  (if (null? lst)
      acc
      (fold-left-single proc (proc (%car lst) acc) (%cdr lst))))

(define (fold-left-multiple proc acc lsts)
  "Internal: fold-left for multiple lists."
  (if (any-null? lsts)
      acc
      (fold-left-multiple proc 
                          (apply proc (append (map-cars lsts) (list acc)))
                          (map-cdrs lsts))))

;; FOLD-RIGHT - Right fold with accumulator
(define (fold-right proc init lst . lsts)
  "Right fold: accumulate from right to left."
  (if (null? lsts)
      ;; Single list case
      (fold-right-single proc init lst)
      ;; Multiple lists case
      (fold-right-multiple proc init (%cons lst lsts))))

(define (fold-right-single proc init lst)
  "Internal: fold-right for single list."
  (if (null? lst)
      init
      (proc (%car lst) (fold-right-single proc init (%cdr lst)))))

(define (fold-right-multiple proc init lsts)
  "Internal: fold-right for multiple lists."
  (if (any-null? lsts)
      init
      (apply proc (append (map-cars lsts) 
                         (list (fold-right-multiple proc init (map-cdrs lsts)))))))

;; ============= UTILITY FUNCTIONS =============

;; Helper: check if any list is null
(define (any-null? lsts)
  "Check if any list in lsts is null."
  (if (null? lsts)
      #f
      (or (null? (%car lsts)) (any-null? (%cdr lsts)))))

;; Helper: get car of each list
(define (map-cars lsts)
  "Get car of each list in lsts."
  (if (null? lsts)
      '()
      (%cons (%car (%car lsts)) (map-cars (%cdr lsts)))))

;; Helper: get cdr of each list
(define (map-cdrs lsts)
  "Get cdr of each list in lsts."
  (if (null? lsts)
      '()
      (%cons (%cdr (%car lsts)) (map-cdrs (%cdr lsts)))))

;; Helper: unless construct
(define (unless condition . body)
  "Execute body unless condition is true."
  (if (not condition)
      (begin . body)))

;; ============= ADDITIONAL HIGHER-ORDER FUNCTIONS =============

;; FIND - Find first element satisfying predicate
(define (find pred lst)
  "Find first element in lst that satisfies pred, or #f if none."
  (cond
    ((null? lst) #f)
    ((pred (%car lst)) (%car lst))
    (else (find pred (%cdr lst)))))

;; ANY - Test if any element satisfies predicate
(define (any pred lst)
  "Return #t if pred returns true for any element in lst."
  (and (not (null? lst))
       (or (pred (%car lst))
           (any pred (%cdr lst)))))

;; EVERY - Test if all elements satisfy predicate
(define (every pred lst)
  "Return #t if pred returns true for all elements in lst."
  (or (null? lst)
      (and (pred (%car lst))
           (every pred (%cdr lst)))))

;; PARTITION - Split list based on predicate
(define (partition pred lst)
  "Return two lists: elements satisfying pred and those that don't."
  (let loop ((lst lst) (true-acc '()) (false-acc '()))
    (cond
      ((null? lst) (values (reverse true-acc) (reverse false-acc)))
      ((pred (%car lst))
       (loop (%cdr lst) (%cons (%car lst) true-acc) false-acc))
      (else
       (loop (%cdr lst) true-acc (%cons (%car lst) false-acc))))))

;; REMOVE - Remove all elements satisfying predicate
(define (remove pred lst)
  "Return list with all elements satisfying pred removed."
  (filter (lambda (x) (not (pred x))) lst))

;; COUNT - Count elements satisfying predicate
(define (count pred lst)
  "Count number of elements in lst that satisfy pred."
  (fold-left (lambda (x acc) (if (pred x) (%+ acc 1) acc)) 0 lst))

;; TAKE - Take first n elements
(define (take lst n)
  "Return list of first n elements from lst."
  (if (or (%<= n 0) (null? lst))
      '()
      (%cons (%car lst) (take (%cdr lst) (%- n 1)))))

;; DROP - Drop first n elements
(define (drop lst n)
  "Return list with first n elements removed."
  (if (or (%<= n 0) (null? lst))
      lst
      (drop (%cdr lst) (%- n 1))))

;; REVERSE - Reverse a list
(define (reverse lst)
  "Return list with elements in reverse order."
  (fold-left %cons '() lst))

;; APPEND - Concatenate lists
(define (append . lsts)
  "Concatenate all lists."
  (fold-right (lambda (lst acc)
                (fold-right %cons acc lst))
              '() lsts))

;; ============= FUNCTIONAL COMPOSITION =============

;; COMPOSE - Function composition
(define (compose f g)
  "Return composition of functions f and g: (f (g x))."
  (lambda (x) (f (g x))))

;; CURRY - Curry a two-argument function
(define (curry f)
  "Convert two-argument function to curried form."
  (lambda (x) (lambda (y) (f x y))))

;; FLIP - Flip arguments of two-argument function
(define (flip f)
  "Return function with arguments flipped."
  (lambda (x y) (f y x)))

;; CONST - Constant function
(define (const x)
  "Return function that always returns x."
  (lambda (y) x))

;; IDENTITY - Identity function
(define (identity x)
  "Return argument unchanged."
  x)

;; ============= LIST PREDICATES =============

;; LENGTH - Get list length
(define (length lst)
  "Return length of proper list."
  (fold-left (lambda (x acc) (%+ acc 1)) 0 lst))

;; LIST? - Test if object is a proper list
(define (list? obj)
  "Test if obj is a proper list."
  (or (null? obj)
      (and (%pair? obj) (list? (%cdr obj)))))

;; ============= COMPARISON UTILITIES =============

(define (%<= x y) (or (%< x y) (%= x y)))
(define (%>= x y) (or (%> x y) (%= x y)))

;; This completes the essential higher-order functions for the bootstrap system.
;; All functions use only minimal primitives and provide the foundation for
;; more complex standard library operations.