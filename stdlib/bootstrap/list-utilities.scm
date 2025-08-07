;; Bootstrap List Utilities
;; Pure Scheme implementations using minimal Rust primitives
;; 
;; This module provides essential list utility functions that complement the
;; higher-order functions. These implementations use only the minimal primitive
;; set defined in the Lambdust bootstrap system.
;;
;; R7RS Compliance: All functions follow R7RS specifications exactly, including
;; error handling, edge cases, and performance characteristics.

(define-module (:: bootstrap list-utilities)
  (metadata
    (version "1.0.0")
    (description "Bootstrap list utilities implemented in pure Scheme")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.4")
    (bootstrap-level "primitive"))
  
  (export 
    ;; Core list utilities
    append reverse length
    ;; List searching and membership
    member memq memv
    ;; Utility predicates
    %proper-list? %list-length))

;; ============= UTILITY FUNCTIONS =============
;; Import utility functions from higher-order module or redefine if needed

(define (%proper-list? obj)
  "Internal: Check if object is a proper list (finite, nil-terminated).
   Uses Floyd's cycle detection algorithm for efficiency and correctness."
  (cond
    ((null? obj) #t)
    ((not (pair? obj)) #f)
    (else
      (let loop ((tortoise obj)
                 (hare obj)
                 (moved-hare? #f))
        (cond
          ((not (pair? hare)) #f)  ; Improper list
          ((null? (cdr hare)) #t)  ; Proper termination
          ((not (pair? (cdr hare))) #f)  ; Improper termination
          ((null? (cddr hare)) #t)  ; Proper termination
          ((and moved-hare? (eq? tortoise hare)) #f)  ; Circular list detected
          (else
            (loop (cdr tortoise) (cddr hare) #t)))))))

(define (%list-length obj)
  "Internal: Get length of proper list, or #f if not a proper list."
  (if (not (%proper-list? obj))
      #f
      (let loop ((lst obj) (count 0))
        (if (null? lst)
            count
            (loop (cdr lst) (+ count 1))))))

;; ============= LENGTH IMPLEMENTATION =============

(define (length list)
  "Returns the length of list.
   
   R7RS: (length list) procedure
   
   Returns the length of list. It is an error if list is not a proper list.
   
   This implementation uses tail recursion for efficiency and provides exact R7RS
   error handling for improper lists."
  
  ;; Argument validation
  (cond
    ((null? list) 0)  ; Empty list has length 0
    ((not (pair? list))
     (error "length: argument must be a list" list))
    ((not (%proper-list? list))
     (error "length: argument must be a proper list" list)))
  
  ;; Count elements using tail recursion
  (let loop ((lst list) (count 0))
    (if (null? lst)
        count
        (loop (cdr lst) (+ count 1)))))

;; ============= APPEND IMPLEMENTATION =============

(define (append . lists)
  "Returns a list consisting of elements of first list followed by elements of other lists.
   
   R7RS: (append list ...) procedure
   
   The last argument, if there is one, can be of any type. Returns a list consisting
   of the elements of the first list followed by the elements of the other lists.
   If there are no arguments, the empty list is returned. If there is exactly one
   argument, it is returned. Otherwise the resulting list is always newly allocated,
   except that it shares structure with the last argument.
   
   This implementation provides exact R7RS semantics including proper handling
   of the last argument which need not be a list."
  
  (cond
    ;; No arguments case
    ((null? lists) '())
    
    ;; Single argument case
    ((null? (cdr lists)) (car lists))
    
    ;; Multiple arguments case
    (else
      ;; Validate all but last argument are proper lists
      (let loop ((args (reverse (cdr (reverse lists)))))  ; All but last
        (unless (null? args)
          (unless (%proper-list? (car args))
            (error "append: all arguments except the last must be proper lists" (car args)))
          (loop (cdr args))))
      
      ;; Perform the append operation
      (append-internal lists))))

(define (append-internal lists)
  "Internal: Core append logic with validated arguments."
  (if (null? (cdr lists))
      (car lists)  ; Last argument (can be any type)
      (append-two (car lists) (append-internal (cdr lists)))))

(define (append-two list1 list2)
  "Internal: Append two lists, where list1 must be proper and list2 can be anything."
  (if (null? list1)
      list2
      (cons (car list1) (append-two (cdr list1) list2))))

;; ============= REVERSE IMPLEMENTATION =============

(define (reverse list)
  "Returns a newly allocated list consisting of elements of list in reverse order.
   
   R7RS: (reverse list) procedure
   
   Returns a newly allocated list consisting of the elements of list in reverse order.
   It is an error if list is not a proper list.
   
   This implementation uses iterative reversal for optimal performance and
   provides exact R7RS error handling."
  
  ;; Argument validation
  (cond
    ((null? list) '())  ; Empty list reverses to empty list
    ((not (pair? list))
     (error "reverse: argument must be a list" list))
    ((not (%proper-list? list))
     (error "reverse: argument must be a proper list" list)))
  
  ;; Iterative reversal using accumulator
  (let loop ((remaining list) (accumulated '()))
    (if (null? remaining)
        accumulated
        (loop (cdr remaining) (cons (car remaining) accumulated)))))

;; ============= MEMBER IMPLEMENTATION =============

(define (member obj list . compare)
  "Returns first sublist of list whose car is obj, using equal? or custom comparison.
   
   R7RS: (member obj list) procedure
         (member obj list compare) procedure
   
   The member procedure returns the first sublist of list whose car is obj,
   where the sublists of list are the non-empty lists returned by (list-tail list k)
   for k less than the length of list. If obj does not occur in list, then #f
   (not the empty list) is returned. The member procedure uses equal? to compare
   obj with the elements of list, unless a custom comparison procedure is provided.
   
   This implementation provides exact R7RS semantics with optional comparison predicate."
  
  ;; Argument validation
  (cond
    ((not (%proper-list? list))
     (error "member: second argument must be a proper list" list))
    ((and (not (null? compare)) (not (procedure? (car compare))))
     (error "member: comparison argument must be a procedure" (car compare)))
    ((> (length compare) 1)
     (error "member: too many arguments" (cons obj (cons list compare)))))
  
  ;; Determine comparison procedure
  (let ((cmp-proc (if (null? compare) equal? (car compare))))
    (member-internal obj list cmp-proc)))

(define (member-internal obj list cmp-proc)
  "Internal: Core member logic with validated arguments."
  (cond
    ((null? list) #f)
    ((cmp-proc obj (car list)) list)
    (else (member-internal obj (cdr list) cmp-proc))))

;; ============= MEMQ IMPLEMENTATION =============

(define (memq obj list)
  "Returns first sublist of list whose car is obj, using eq? comparison.
   
   R7RS: (memq obj list) procedure
   
   The memq procedure uses eq? to compare obj with the elements of list.
   Otherwise it is identical to member.
   
   This implementation provides exact R7RS semantics using eq? comparison."
  
  ;; Argument validation
  (unless (%proper-list? list)
    (error "memq: second argument must be a proper list" list))
  
  (memq-internal obj list))

(define (memq-internal obj list)
  "Internal: Core memq logic with validated arguments."
  (cond
    ((null? list) #f)
    ((eq? obj (car list)) list)
    (else (memq-internal obj (cdr list)))))

;; ============= MEMV IMPLEMENTATION =============

(define (memv obj list)
  "Returns first sublist of list whose car is obj, using eqv? comparison.
   
   R7RS: (memv obj list) procedure
   
   The memv procedure uses eqv? to compare obj with the elements of list.
   Otherwise it is identical to member.
   
   This implementation provides exact R7RS semantics using eqv? comparison."
  
  ;; Argument validation
  (unless (%proper-list? list)
    (error "memv: second argument must be a proper list" list))
  
  (memv-internal obj list))

(define (memv-internal obj list)
  "Internal: Core memv logic with validated arguments."
  (cond
    ((null? list) #f)
    ((eqv? obj (car list)) list)
    (else (memv-internal obj (cdr list)))))

;; ============= UTILITY HELPERS =============

(define (unless condition . body)
  "Execute body unless condition is true."
  (if (not condition)
      (begin . body)))