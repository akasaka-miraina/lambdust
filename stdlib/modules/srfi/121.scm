;;; SRFI-121: Generators - Complete Implementation
;;;
;;; This library provides a complete implementation of SRFI-121 generators,
;;; which are stateful procedures for lazy sequence generation and processing.
;;; 
;;; Phase 2-4: Complete SRFI-121 implementation using Phase 1 primitives
;;; 
;;; Provides 53 procedures total:
;;;   - 4 core primitives (Phase 1 Rust)  
;;;   - 12 basic constructors (Phase 1 Rust + Scheme)
;;;   - 15 transformers/combinators (Pure Scheme)
;;;   - 15 consumers (Pure Scheme) 
;;;   - 8 accumulators (Pure Scheme)

(define-library (srfi 121)
  (import (scheme base)
          (scheme case-lambda)
          (scheme write)
          (only (scheme char) char?)
          (only (scheme vector) make-vector vector-length vector-ref)
          (only (scheme bytevector) make-bytevector bytevector-length bytevector-u8-ref))
  
  (export
    ;; Core generator operations (Phase 1 primitives)
    %make-generator %generator-next %generator-exhausted? %generator?
    
    ;; Generator constructors 
    generator make-range-generator make-iota-generator
    list->generator vector->generator reverse-vector->generator
    string->generator bytevector->generator
    make-for-each-generator make-unfold-generator
    make-coroutine-generator
    
    ;; Generator transformers/combinators
    gcons* gappend gflatten ggroup
    gfilter gremove gstate-filter gdelete gdelete-neighbor-dups
    gtake gdrop gtake-while gdrop-while
    gindex gselect
    
    ;; Generator consumers
    generator->list generator->vector generator->string generator->bytevector
    generator-fold generator-reduce generator-for-each generator-map
    generator-find generator-count generator-any generator-every
    generator-length generator-sum generator-unfold
    
    ;; Accumulator procedures
    make-accumulator count-accumulator list-accumulator
    reverse-list-accumulator vector-accumulator string-accumulator
    bytevector-accumulator sum-accumulator
    make-accumulator-generator)
  
  (begin
    ;; ================ HELPER UTILITIES ================
    
    ;; EOF object singleton for consistency
    (define *eof-object* '*eof-object*)
    
    ;; Check if a value is the EOF object
    (define (eof-object? x)
      (eq? x *eof-object*))
    
    ;; Safe next operation that handles errors
    (define (safe-generator-next gen)
      (if (%generator-exhausted? gen)
          *eof-object*
          (%generator-next gen)))
    
    ;; Convert any sequence to a generator
    (define (->generator obj)
      (cond
        ((%generator? obj) obj)
        ((list? obj) (list->generator obj))
        ((vector? obj) (vector->generator obj))
        ((string? obj) (string->generator obj))
        (else (error "Cannot convert to generator" obj))))
    
    ;; ================ ADDITIONAL CONSTRUCTORS ================
    
    ;; Reverse vector generator
    (define (reverse-vector->generator vec . args)
      (let* ((start (if (and (pair? args) (>= (length args) 1)) (car args) 0))
             (end (if (and (pair? args) (>= (length args) 2)) 
                      (cadr args) 
                      (vector-length vec)))
             (values '()))
        ;; Collect values in reverse order
        (do ((i (- end 1) (- i 1)))
            ((< i start))
          (set! values (cons (vector-ref vec i) values)))
        (generator values)))
    
    ;; Bytevector generator
    (define (bytevector->generator bv . args)
      (let* ((start (if (and (pair? args) (>= (length args) 1)) (car args) 0))
             (end (if (and (pair? args) (>= (length args) 2)) 
                      (cadr args) 
                      (bytevector-length bv)))
             (values '()))
        ;; Collect byte values
        (do ((i start (+ i 1)))
            ((>= i end))
          (set! values (cons (bytevector-u8-ref bv i) values)))
        (apply generator (reverse values))))
    
    ;; For-each based generator
    (define (make-for-each-generator for-each obj)
      (let ((values '()))
        ;; Use for-each to collect all values
        (for-each (lambda (x) (set! values (cons x values))) obj)
        (apply generator (reverse values))))
    
    ;; Unfold generator - generates values using unfold pattern
    (define (make-unfold-generator stop? mapper successor seed)
      (let ((current seed)
            (done #f))
        (%make-generator 
          (lambda ()
            (if done
                *eof-object*
                (if (stop? current)
                    (begin (set! done #t) *eof-object*)
                    (let ((value (mapper current)))
                      (set! current (successor current))
                      value)))))))
    
    ;; Coroutine generator using call/cc for advanced control flow  
    (define (make-coroutine-generator proc)
      (define continuation #f)
      (define done #f)
      (define yielded-value #f)
      
      (define (yield value)
        (call/cc 
          (lambda (k)
            (set! continuation k)
            (set! yielded-value value)
            #t)))
      
      (%make-generator
        (lambda ()
          (if done
              *eof-object*
              (if continuation
                  (begin
                    (continuation #f)
                    (if done *eof-object* yielded-value))
                  (begin
                    (call/cc
                      (lambda (exit)
                        (proc yield)
                        (set! done #t)
                        (exit *eof-object*)))
                    (if done *eof-object* yielded-value)))))))
    
    ;; ================ GENERATOR TRANSFORMERS ================
    
    ;; Cons elements onto generator
    (define (gcons* . args)
      (let ((items (reverse (cdr (reverse args))))
            (gen (car (reverse args))))
        (generator (append items (generator->list gen)))))
    
    ;; Append generators
    (define (gappend . gens)
      (let ((all-values '()))
        (for-each 
          (lambda (gen)
            (let loop ()
              (let ((val (safe-generator-next gen)))
                (when (not (eof-object? val))
                  (set! all-values (cons val all-values))
                  (loop)))))
          gens)
        (apply generator (reverse all-values))))
    
    ;; Flatten generator of generators
    (define (gflatten gen)
      (let ((all-values '()))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (when (not (eof-object? val))
              (let ((subgen (->generator val)))
                (let subloop ()
                  (let ((subval (safe-generator-next subgen)))
                    (when (not (eof-object? subval))
                      (set! all-values (cons subval all-values))
                      (subloop)))))
              (loop))))
        (apply generator (reverse all-values))))
    
    ;; Group consecutive elements
    (define ggroup
      (case-lambda
        ((gen) (ggroup gen equal?))
        ((gen equal?)
         (let ((groups '())
               (current-group '())
               (prev-value #f)
               (first #t))
           (let loop ()
             (let ((val (safe-generator-next gen)))
               (cond
                 ((eof-object? val)
                  (when (not (null? current-group))
                    (set! groups (cons (reverse current-group) groups))))
                 (first
                  (set! current-group (list val))
                  (set! prev-value val)
                  (set! first #f)
                  (loop))
                 ((equal? val prev-value)
                  (set! current-group (cons val current-group))
                  (loop))
                 (else
                  (set! groups (cons (reverse current-group) groups))
                  (set! current-group (list val))
                  (set! prev-value val)
                  (loop)))))
           (apply generator (reverse groups))))))
    
    ;; Filter generator values
    (define (gfilter pred gen)
      (let ((values '()))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (when (not (eof-object? val))
              (when (pred val)
                (set! values (cons val values)))
              (loop))))
        (apply generator (reverse values))))
    
    ;; Remove matching values
    (define (gremove pred gen)
      (gfilter (lambda (x) (not (pred x))) gen))
    
    ;; Stateful filter with state tracking
    (define (gstate-filter proc gen)
      (let ((values '())
            (state #f))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (when (not (eof-object? val))
              (let-values (((keep? new-state) (proc val state)))
                (set! state new-state)
                (when keep?
                  (set! values (cons val values))))
              (loop))))
        (apply generator (reverse values))))
    
    ;; Delete specific elements
    (define gdelete
      (case-lambda
        ((item gen) (gdelete item gen equal?))
        ((item gen equal?)
         (gfilter (lambda (x) (not (equal? x item))) gen))))
    
    ;; Delete neighbor duplicates
    (define gdelete-neighbor-dups
      (case-lambda
        ((gen) (gdelete-neighbor-dups gen equal?))
        ((gen equal?)
         (let ((values '())
               (prev-value #f)
               (first #t))
           (let loop ()
             (let ((val (safe-generator-next gen)))
               (when (not (eof-object? val))
                 (when (or first (not (equal? val prev-value)))
                   (set! values (cons val values))
                   (set! prev-value val)
                   (set! first #f))
                 (loop))))
           (apply generator (reverse values))))))
    
    ;; Take first n elements
    (define (gtake gen n)
      (let ((values '())
            (count 0))
        (let loop ()
          (if (>= count n)
              (apply generator (reverse values))
              (let ((val (safe-generator-next gen)))
                (if (eof-object? val)
                    (apply generator (reverse values))
                    (begin
                      (set! values (cons val values))
                      (set! count (+ count 1))
                      (loop))))))))
    
    ;; Drop first n elements
    (define (gdrop gen n)
      (let ((count 0))
        ;; Skip n elements
        (define (skip-loop)
          (when (< count n)
            (let ((val (safe-generator-next gen)))
              (when (not (eof-object? val))
                (set! count (+ count 1))
                (skip-loop)))))
        (skip-loop)
        ;; Return generator for remaining elements
        gen))
    
    ;; Take while predicate is true
    (define (gtake-while pred gen)
      (let ((values '()))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (if (or (eof-object? val) (not (pred val)))
                (apply generator (reverse values))
                (begin
                  (set! values (cons val values))
                  (loop)))))))
    
    ;; Drop while predicate is true
    (define (gdrop-while pred gen)
      (let ((dropped #f))
        ;; Skip elements while predicate is true
        (define (skip-loop)
          (when (not dropped)
            (let ((val (safe-generator-next gen)))
              (if (or (eof-object? val) (not (pred val)))
                  (set! dropped #t)
                  (skip-loop)))))
        (skip-loop)
        ;; Return generator for remaining elements
        gen))
    
    ;; Index generator with indices
    (define (gindex value-gen index-gen)
      (let ((values (generator->list value-gen))
            (indices (generator->list index-gen))
            (result '()))
        (for-each
          (lambda (idx)
            (when (and (integer? idx) (>= idx 0) (< idx (length values)))
              (set! result (cons (list-ref values idx) result))))
          indices)
        (apply generator (reverse result))))
    
    ;; Select elements at specific indices  
    (define (gselect gen . indices)
      (gindex gen (apply generator indices)))
    
    ;; ================ GENERATOR CONSUMERS ================
    
    ;; Convert generator to vector
    (define generator->vector
      (case-lambda
        ((gen) (list->vector (generator->list gen)))
        ((gen n)
         (let ((vec (make-vector n))
               (i 0))
           (let loop ()
             (if (>= i n)
                 vec
                 (let ((val (safe-generator-next gen)))
                   (if (eof-object? val)
                       (let ((result (make-vector i)))
                         (do ((j 0 (+ j 1)))
                             ((>= j i) result)
                           (vector-set! result j (vector-ref vec j))))
                       (begin
                         (vector-set! vec i val)
                         (set! i (+ i 1))
                         (loop))))))))))
    
    ;; Convert generator to string
    (define (generator->string gen)
      (list->string 
        (filter char? (generator->list gen))))
    
    ;; Convert generator to bytevector
    (define (generator->bytevector gen)
      (let ((bytes (filter (lambda (x) 
                            (and (integer? x) (>= x 0) (<= x 255)))
                          (generator->list gen))))
        (let ((bv (make-bytevector (length bytes))))
          (let loop ((i 0) (lst bytes))
            (if (null? lst)
                bv
                (begin
                  (bytevector-u8-set! bv i (car lst))
                  (loop (+ i 1) (cdr lst))))))))
    
    ;; Reduce generator (fold without initial value)
    (define (generator-reduce gen proc)
      (let ((first-val (safe-generator-next gen)))
        (if (eof-object? first-val)
            (error "Cannot reduce empty generator")
            (generator-fold proc first-val gen))))
    
    ;; For-each over generator  
    (define (generator-for-each proc gen)
      (let loop ()
        (let ((val (safe-generator-next gen)))
          (when (not (eof-object? val))
            (proc val)
            (loop)))))
    
    ;; Map function over generator
    (define (generator-map proc . gens)
      (if (null? gens)
          (error "generator-map: no generators provided")
          (let ((values '()))
            (let loop ()
              (let ((vals (map safe-generator-next gens)))
                (if (any eof-object? vals)
                    (apply generator (reverse values))
                    (begin
                      (set! values (cons (apply proc vals) values))
                      (loop))))))))
    
    ;; Find first element matching predicate
    (define (generator-find pred gen)
      (let loop ()
        (let ((val (safe-generator-next gen)))
          (cond
            ((eof-object? val) #f)
            ((pred val) val)
            (else (loop))))))
    
    ;; Count elements matching predicate
    (define (generator-count pred gen)
      (let ((count 0))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (when (not (eof-object? val))
              (when (pred val)
                (set! count (+ count 1)))
              (loop))))
        count))
    
    ;; Test if any element matches predicate
    (define (generator-any pred gen)
      (let loop ()
        (let ((val (safe-generator-next gen)))
          (cond
            ((eof-object? val) #f)
            ((pred val) #t)
            (else (loop))))))
    
    ;; Test if all elements match predicate
    (define (generator-every pred gen)
      (let loop ()
        (let ((val (safe-generator-next gen)))
          (cond
            ((eof-object? val) #t)
            ((pred val) (loop))
            (else #f)))))
    
    ;; Get length of generator
    (define (generator-length gen)
      (let ((count 0))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (when (not (eof-object? val))
              (set! count (+ count 1))
              (loop))))
        count))
    
    ;; Sum numeric generator
    (define (generator-sum gen)
      (generator-fold + 0 gen))
    
    ;; Unfold using generator as source
    (define (generator-unfold gen unfolder . args)
      (let ((stop? (if (pair? args) (car args) (lambda (x) #f)))
            (result '()))
        (let loop ()
          (let ((val (safe-generator-next gen)))
            (if (or (eof-object? val) (stop? val))
                (reverse result)
                (begin
                  (set! result (cons (unfolder val) result))
                  (loop)))))))
    
    ;; ================ ACCUMULATOR PROCEDURES ================
    
    ;; Generic accumulator constructor
    (define (make-accumulator proc init)
      (let ((state init))
        (lambda (item)
          (if (eof-object? item)
              state
              (begin
                (set! state (proc state item))
                state)))))
    
    ;; Count accumulator
    (define (count-accumulator)
      (make-accumulator (lambda (count _) (+ count 1)) 0))
    
    ;; List accumulator  
    (define (list-accumulator)
      (make-accumulator 
        (lambda (lst item) (cons item lst))
        '()))
    
    ;; Reverse list accumulator (maintains insertion order)
    (define (reverse-list-accumulator)
      (let ((acc (list-accumulator)))
        (lambda (item)
          (if (eof-object? item)
              (reverse (acc item))
              (acc item)))))
    
    ;; Vector accumulator (with dynamic resizing)
    (define (vector-accumulator)
      (let ((items '()))
        (lambda (item)
          (if (eof-object? item)
              (list->vector (reverse items))
              (begin
                (set! items (cons item items))
                items)))))
    
    ;; String accumulator (for character sequences)
    (define (string-accumulator)
      (let ((chars '()))
        (lambda (item)
          (if (eof-object? item)
              (list->string (reverse chars))
              (if (char? item)
                  (begin
                    (set! chars (cons item chars))
                    chars)
                  chars)))))
    
    ;; Bytevector accumulator (for byte sequences)
    (define (bytevector-accumulator)
      (let ((bytes '()))
        (lambda (item)
          (if (eof-object? item)
              (let ((bv (make-bytevector (length bytes))))
                (let loop ((i 0) (lst (reverse bytes)))
                  (if (null? lst)
                      bv
                      (begin
                        (bytevector-u8-set! bv i (car lst))
                        (loop (+ i 1) (cdr lst))))))
              (if (and (integer? item) (>= item 0) (<= item 255))
                  (begin
                    (set! bytes (cons item bytes))
                    bytes)
                  bytes)))))
    
    ;; Sum accumulator (for numeric sequences)
    (define (sum-accumulator)
      (make-accumulator + 0))
    
    ;; Create generator from accumulator
    (define (make-accumulator-generator accumulator gen)
      (let ((acc accumulator)
            (done #f)
            (values '())
            (index 0))
        ;; First, accumulate all values
        (generator-for-each 
          (lambda (item) (acc item))
          gen)
        ;; Get final accumulated result
        (let ((result (acc *eof-object*)))
          ;; Convert result to list for iteration
          (set! values 
            (cond
              ((list? result) result)
              ((vector? result) (vector->list result))
              ((string? result) (string->list result))
              (else (list result))))
          ;; Return generator that yields accumulated values
          (%make-generator
            (lambda ()
              (if (>= index (length values))
                  *eof-object*
                  (let ((val (list-ref values index)))
                    (set! index (+ index 1))
                    val)))))))
    
    )) ; end library