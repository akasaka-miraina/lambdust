;; Bootstrap Core Library
;; 
;; This library implements fundamental Scheme operations using only the minimal
;; set of Rust primitives. All operations here are built from the primitive
;; bridge system and provide the foundation for higher-level standard library
;; functions.
;;
;; Design principles:
;; 1. Use only minimal primitives (prefixed with %)
;; 2. Implement everything else in pure Scheme
;; 3. Maintain R7RS semantics
;; 4. Provide clean abstraction layers

;; ============= BASIC OPERATIONS =============

;; Fundamental list operations (built from minimal primitives)
(define cons %cons)
(define car %car)
(define cdr %cdr)

;; Basic type predicates (built from minimal primitives)
(define null? %null?)
(define pair? %pair?)
(define symbol? %symbol?)
(define number? %number?)
(define string? %string?)
(define char? %char?)
(define vector? %vector?)
(define procedure? %procedure?)
(define port? %port?)

;; Basic arithmetic (built from minimal primitives)
(define + %+)
(define - %-)
(define * %*)
(define / %/)
(define = %=)
(define < %<)
(define > %>)

;; String operations (built from minimal primitives)
(define string-length %string-length)
(define string-ref %string-ref)
(define make-string %make-string)
(define string->symbol %string->symbol)
(define symbol->string %symbol->string)

;; Vector operations (built from minimal primitives)
(define vector-length %vector-length)
(define vector-ref %vector-ref)
(define vector-set! %vector-set!)
(define make-vector %make-vector)

;; Error handling (built from minimal primitives)
(define error %error)

;; ============= DERIVED OPERATIONS =============

;; Additional comparison operators
(define (<= x y)
  (or (< x y) (= x y)))

(define (>= x y)
  (or (> x y) (= x y)))

;; Logical operations
(define (not obj)
  (if obj #f #t))

;; Additional list predicates and operations
(define (list? obj)
  (cond
    ((null? obj) #t)
    ((pair? obj) (list? (cdr obj)))
    (else #f)))

(define (length lst)
  (if (null? lst)
      0
      (+ 1 (length (cdr lst)))))

;; Basic list construction
(define (list . args)
  (define (build-list items)
    (if (null? items)
        '()
        (cons (car items) (build-list (cdr items)))))
  (build-list args))

;; List reversal
(define (reverse lst)
  (define (reverse-helper lst acc)
    (if (null? lst)
        acc
        (reverse-helper (cdr lst) (cons (car lst) acc))))
  (reverse-helper lst '()))

;; List append
(define (append . lists)
  (define (append-two lst1 lst2)
    (if (null? lst1)
        lst2
        (cons (car lst1) (append-two (cdr lst1) lst2))))
  (define (append-all lists)
    (if (null? lists)
        '()
        (if (null? (cdr lists))
            (car lists)
            (append-two (car lists) (append-all (cdr lists))))))
  (append-all lists))

;; List membership
(define (memq obj lst)
  (cond
    ((null? lst) #f)
    ((eq? obj (car lst)) lst)
    (else (memq obj (cdr lst)))))

(define (memv obj lst)
  (cond
    ((null? lst) #f)
    ((eqv? obj (car lst)) lst)
    (else (memv obj (cdr lst)))))

(define (member obj lst)
  (cond
    ((null? lst) #f)
    ((equal? obj (car lst)) lst)
    (else (member obj (cdr lst)))))

;; Association lists
(define (assq obj alist)
  (cond
    ((null? alist) #f)
    ((eq? obj (car (car alist))) (car alist))
    (else (assq obj (cdr alist)))))

(define (assv obj alist)
  (cond
    ((null? alist) #f)
    ((eqv? obj (car (car alist))) (car alist))
    (else (assv obj (cdr alist)))))

(define (assoc obj alist)
  (cond
    ((null? alist) #f)
    ((equal? obj (car (car alist))) (car alist))
    (else (assoc obj (cdr alist)))))

;; ============= EQUALITY PREDICATES =============

;; Basic equality - build on primitive comparison where possible
(define (eq? obj1 obj2)
  ;; Identity equality - for now use primitive comparison
  ;; In full implementation, this would check reference equality
  (cond
    ((and (symbol? obj1) (symbol? obj2))
     ;; Symbols with same name are eq?
     (string=? (symbol->string obj1) (symbol->string obj2)))
    ((and (number? obj1) (number? obj2))
     (= obj1 obj2))
    ((and (char? obj1) (char? obj2))
     (char=? obj1 obj2))
    ((and (null? obj1) (null? obj2)) #t)
    (else #f))) ; Conservative - would need runtime support for true identity

(define (eqv? obj1 obj2)
  ;; Operational equivalence
  (cond
    ((eq? obj1 obj2) #t)
    ((and (number? obj1) (number? obj2))
     (= obj1 obj2))
    (else #f)))

(define (equal? obj1 obj2)
  ;; Structural equality
  (cond
    ((eqv? obj1 obj2) #t)
    ((and (pair? obj1) (pair? obj2))
     (and (equal? (car obj1) (car obj2))
          (equal? (cdr obj1) (cdr obj2))))
    ((and (string? obj1) (string? obj2))
     (string=? obj1 obj2))
    ((and (vector? obj1) (vector? obj2))
     (and (= (vector-length obj1) (vector-length obj2))
          (vector-equal-elements? obj1 obj2 0)))
    (else #f)))

;; Helper for vector equality
(define (vector-equal-elements? vec1 vec2 index)
  (cond
    ((= index (vector-length vec1)) #t)
    ((equal? (vector-ref vec1 index) (vector-ref vec2 index))
     (vector-equal-elements? vec1 vec2 (+ index 1)))
    (else #f)))

;; ============= STRING OPERATIONS =============

;; String comparison
(define (string=? str1 str2)
  (define (string-equal-chars? s1 s2 i len)
    (cond
      ((= i len) #t)
      ((char=? (string-ref s1 i) (string-ref s2 i))
       (string-equal-chars? s1 s2 (+ i 1) len))
      (else #f)))
  (and (= (string-length str1) (string-length str2))
       (string-equal-chars? str1 str2 0 (string-length str1))))

(define (string<? str1 str2)
  (define (string-less-chars? s1 s2 i len1 len2)
    (cond
      ((= i len1) (< len1 len2))
      ((= i len2) #f)
      ((char<? (string-ref s1 i) (string-ref s2 i)) #t)
      ((char>? (string-ref s1 i) (string-ref s2 i)) #f)
      (else (string-less-chars? s1 s2 (+ i 1) len1 len2))))
  (string-less-chars? str1 str2 0 (string-length str1) (string-length str2)))

(define (string>? str1 str2)
  (string<? str2 str1))

(define (string<=? str1 str2)
  (not (string>? str1 str2)))

(define (string>=? str1 str2)
  (not (string<? str1 str2)))

;; ============= CHARACTER OPERATIONS =============

;; Character comparison (assuming ASCII for now)
(define (char=? char1 char2)
  ;; This would need primitive support or numeric conversion
  ;; For now, simplified implementation
  #f) ; Placeholder - needs runtime support

(define (char<? char1 char2)
  ;; This would need primitive support or numeric conversion
  ;; For now, simplified implementation
  #f) ; Placeholder - needs runtime support

(define (char>? char1 char2)
  (char<? char2 char1))

(define (char<=? char1 char2)
  (not (char>? char1 char2)))

(define (char>=? char1 char2)
  (not (char<? char1 char2)))

;; ============= CONTROL OPERATIONS =============

;; Basic control flow - these would typically be special forms
;; but we implement what we can as procedures

(define (identity x) x)

(define (const x) (lambda (y) x))

;; ============= NUMERIC OPERATIONS =============

;; Additional numeric predicates and operations
(define (zero? x) (= x 0))
(define (positive? x) (> x 0))
(define (negative? x) (< x 0))

(define (abs x)
  (if (negative? x) (- x) x))

(define (min x . rest)
  (define (min-two a b) (if (< a b) a b))
  (define (min-list lst current)
    (if (null? lst)
        current
        (min-list (cdr lst) (min-two current (car lst)))))
  (if (null? rest)
      x
      (min-list rest x)))

(define (max x . rest)
  (define (max-two a b) (if (> a b) a b))
  (define (max-list lst current)
    (if (null? lst)
        current
        (max-list (cdr lst) (max-two current (car lst)))))
  (if (null? rest)
      x
      (max-list rest x)))

;; ============= BOOLEAN OPERATIONS =============

(define (boolean? obj)
  (or (eq? obj #t) (eq? obj #f)))

(define (boolean=? . args)
  (define (all-equal? lst first)
    (cond
      ((null? lst) #t)
      ((not (boolean? (car lst)))
       (error "boolean=?: argument is not a boolean"))
      ((eq? (car lst) first)
       (all-equal? (cdr lst) first))
      (else #f)))
  (cond
    ((null? args) (error "boolean=?: requires at least one argument"))
    ((not (boolean? (car args)))
     (error "boolean=?: argument is not a boolean"))
    (else (all-equal? (cdr args) (car args)))))

;; ============= VECTOR OPERATIONS =============

;; Additional vector operations
(define (vector . args)
  (define vec (make-vector (length args)))
  (define (fill-vector! v lst index)
    (if (not (null? lst))
        (begin
          (vector-set! v index (car lst))
          (fill-vector! v (cdr lst) (+ index 1)))))
  (fill-vector! vec args 0)
  vec)

(define (vector->list vec)
  (define (vector->list-helper v i len acc)
    (if (= i len)
        (reverse acc)
        (vector->list-helper v (+ i 1) len 
                           (cons (vector-ref v i) acc))))
  (vector->list-helper vec 0 (vector-length vec) '()))

(define (list->vector lst)
  (define vec (make-vector (length lst)))
  (define (fill-from-list! v l i)
    (if (not (null? l))
        (begin
          (vector-set! v i (car l))
          (fill-from-list! v (cdr l) (+ i 1)))))
  (fill-from-list! vec lst 0)
  vec)

;; ============= UTILITY PROCEDURES =============

;; Apply procedure - this would need evaluator integration in practice
(define apply %apply) ; Use primitive for now

;; Compose procedure
(define (compose f g)
  (lambda (x) (f (g x))))

;; Partial application
(define (curry f)
  (lambda (x) (lambda (y) (f x y))))

;; ============= MISCELLANEOUS =============

;; Void procedure
(define (void) (if #f #f))

;; Values procedure (simplified - full implementation needs multiple values support)
(define (values . args)
  (if (null? args)
      (void)
      (if (null? (cdr args))
          (car args)
          args))) ; Simplified - real implementation needs multiple values

;; Call with values (simplified)
(define (call-with-values producer consumer)
  (consumer (producer)))

;; ============= EXPORT LIST =============
;; In a full module system, we would export these bindings
;; For now, they are available in the global environment

;; Core list operations: cons, car, cdr, null?, pair?, list?, length, 
;;                      reverse, append, member, memq, memv, assoc, assq, assv
;; Core predicates: symbol?, number?, string?, char?, vector?, procedure?, port?
;; Core arithmetic: +, -, *, /, =, <, >, <=, >=, zero?, positive?, negative?, abs, min, max
;; Core equality: eq?, eqv?, equal?
;; Core strings: string-length, string-ref, make-string, string->symbol, symbol->string,
;;              string=?, string<?, string>?, string<=?, string>=?
;; Core vectors: vector-length, vector-ref, vector-set!, make-vector, vector,
;;              vector->list, list->vector
;; Core control: identity, const, compose, curry
;; Core boolean: boolean?, boolean=?, not
;; Core misc: void, values, call-with-values, apply

;; This provides a solid foundation for implementing more complex
;; standard library functions in pure Scheme.