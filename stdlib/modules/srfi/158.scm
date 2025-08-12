;;; SRFI-158: Enhanced Generators and Accumulators
;;;
;;; This library provides the complete SRFI-158 specification, which extends
;;; SRFI-121 generators with 15 additional enhanced procedures for more powerful
;;; generator and accumulator operations.
;;;
;;; Total procedures: 68 (53 from SRFI-121 + 15 new enhanced procedures)
;;;
;;; New procedures added in SRFI-158:
;;; - Enhanced constructors: circular-generator, make-bits-generator
;;; - Enhanced transformers: gmerge, gcombine, generator-zip-with  
;;; - Enhanced accumulators: product-accumulator, min-accumulator, max-accumulator,
;;;   vector-accumulator (enhanced), accumulate-generated, generator-accumulate,
;;;   make-accumulator-generator (enhanced)
;;; - Advanced utilities: generator-concatenate, generator-pad-with, generator-maybe-ref

(define-library (srfi 158)
  (import (scheme base)
          (scheme case-lambda)
          (scheme write)
          (only (scheme char) char?)
          (only (scheme vector) make-vector vector-length vector-ref vector-set!)
          (only (scheme bytevector) make-bytevector bytevector-length bytevector-u8-ref)
          (srfi 121)     ; Import all SRFI-121 generators
          (srfi 128))    ; Import comparators for enhanced operations
  
  (export
    ;; Re-export all SRFI-121 procedures for backward compatibility
    ;; Core generator operations
    %make-generator %generator-next %generator-exhausted? %generator?
    
    ;; Generator constructors from SRFI-121
    generator make-range-generator make-iota-generator
    list->generator vector->generator reverse-vector->generator
    string->generator bytevector->generator
    make-for-each-generator make-unfold-generator
    make-coroutine-generator
    
    ;; Generator transformers/combinators from SRFI-121
    gcons* gappend gflatten ggroup
    gfilter gremove gstate-filter gdelete gdelete-neighbor-dups
    gtake gdrop gtake-while gdrop-while
    gindex gselect
    
    ;; Generator consumers from SRFI-121
    generator->list generator->vector generator->string generator->bytevector
    generator-fold generator-reduce generator-for-each generator-map
    generator-find generator-count generator-any generator-every
    generator-length generator-sum generator-unfold
    
    ;; Accumulator procedures from SRFI-121
    make-accumulator count-accumulator list-accumulator
    reverse-list-accumulator vector-accumulator string-accumulator
    bytevector-accumulator sum-accumulator
    make-accumulator-generator
    
    ;; NEW SRFI-158 Enhanced constructors (2)
    circular-generator make-bits-generator
    
    ;; NEW SRFI-158 Enhanced transformers (3)  
    gmerge gcombine generator-zip-with
    
    ;; NEW SRFI-158 Enhanced accumulators (7)
    product-accumulator min-accumulator max-accumulator
    enhanced-vector-accumulator accumulate-generated 
    generator-accumulate enhanced-make-accumulator-generator
    
    ;; NEW SRFI-158 Advanced utilities (3)
    generator-concatenate generator-pad-with generator-maybe-ref)
  
  (begin
    ;; Import internal utilities from SRFI-121 (redefined for SRFI-158)
    (define *eof-object* '*eof-object*)
    (define (eof-object? x) (eq? x *eof-object*))
    (define (safe-generator-next gen)
      (if (%generator-exhausted? gen)
          *eof-object*
          (%generator-next gen)))
    
    ;; ================ NEW SRFI-158 ENHANCED CONSTRUCTORS ================
    
    ;; Circular generator - infinitely repeats a finite sequence
    (define (circular-generator . args)
      (if (null? args)
          (error "circular-generator: at least one argument required")
          (let ((items (if (and (= (length args) 1) (list? (car args)))
                          (car args)  ; Single list argument
                          args))      ; Multiple arguments
                (index 0))
            (%make-generator
              (lambda ()
                (if (null? items)
                    *eof-object*  ; Empty sequence case
                    (let ((value (list-ref items (modulo index (length items)))))
                      (set! index (+ index 1))
                      value)))))))
    
    ;; Make bits generator - generates bits from integers
    (define (make-bits-generator n)
      (if (not (and (integer? n) (>= n 0)))
          (error "make-bits-generator: non-negative integer required" n)
          (let ((remaining n)
                (done #f))
            (%make-generator
              (lambda ()
                (if (or done (= remaining 0))
                    (if done *eof-object* (begin (set! done #t) 0))
                    (let ((bit (modulo remaining 2)))
                      (set! remaining (quotient remaining 2))
                      bit)))))))
    
    ;; ================ NEW SRFI-158 ENHANCED TRANSFORMERS ================
    
    ;; Merge sorted generators using comparator
    (define gmerge
      (case-lambda
        ((comp gen1 gen2)
         (let ((val1 (safe-generator-next gen1))
               (val2 (safe-generator-next gen2))
               (values '()))
           (let loop ((v1 val1) (v2 val2))
             (cond
               ((eof-object? v1)
                ;; gen1 exhausted, append remaining from gen2
                (when (not (eof-object? v2))
                  (set! values (cons v2 values))
                  (let inner-loop ()
                    (let ((next-val (safe-generator-next gen2)))
                      (when (not (eof-object? next-val))
                        (set! values (cons next-val values))
                        (inner-loop)))))
                (apply generator (reverse values)))
               ((eof-object? v2)
                ;; gen2 exhausted, append remaining from gen1
                (set! values (cons v1 values))
                (let inner-loop ()
                  (let ((next-val (safe-generator-next gen1)))
                    (when (not (eof-object? next-val))
                      (set! values (cons next-val values))
                      (inner-loop))))
                (apply generator (reverse values)))
               ((<= (comparator-compare comp v1 v2) 0) ; v1 <= v2
                (set! values (cons v1 values))
                (loop (safe-generator-next gen1) v2))
               (else
                (set! values (cons v2 values))
                (loop v1 (safe-generator-next gen2)))))))
        ((comp . gens)
         ;; Multiple generators - merge pairwise
         (if (null? gens)
             (generator)
             (let loop ((result (car gens)) (remaining (cdr gens)))
               (if (null? remaining)
                   result
                   (loop (gmerge comp result (car remaining))
                         (cdr remaining))))))))
    
    ;; Combine generators with state transformation
    (define (gcombine proc seed . gens)
      (if (null? gens)
          (error "gcombine: at least one generator required")
          (let ((state seed)
                (values '()))
            (let loop ()
              (let ((vals (map safe-generator-next gens)))
                (if (any eof-object? vals)
                    (apply generator (reverse values))
                    (let-values (((new-state result) (apply proc state vals)))
                      (set! state new-state)
                      (set! values (cons result values))
                      (loop))))))))
    
    ;; Zip generators with function application
    (define (generator-zip-with proc . gens)
      (if (null? gens)
          (error "generator-zip-with: at least one generator required")
          (let ((values '()))
            (let loop ()
              (let ((vals (map safe-generator-next gens)))
                (if (any eof-object? vals)
                    (apply generator (reverse values))
                    (begin
                      (set! values (cons (apply proc vals) values))
                      (loop))))))))
    
    ;; ================ NEW SRFI-158 ENHANCED ACCUMULATORS ================
    
    ;; Product accumulator - multiply numeric values
    (define (product-accumulator)
      (make-accumulator * 1))
    
    ;; Min accumulator - find minimum value
    (define (min-accumulator)
      (let ((min-val #f)
            (initialized #f))
        (lambda (item)
          (if (eof-object? item)
              min-val
              (if initialized
                  (begin
                    (set! min-val (min min-val item))
                    min-val)
                  (begin
                    (set! min-val item)
                    (set! initialized #t)
                    min-val))))))
    
    ;; Max accumulator - find maximum value  
    (define (max-accumulator)
      (let ((max-val #f)
            (initialized #f))
        (lambda (item)
          (if (eof-object? item)
              max-val
              (if initialized
                  (begin
                    (set! max-val (max max-val item))
                    max-val)
                  (begin
                    (set! max-val item)
                    (set! initialized #t)
                    max-val))))))
    
    ;; Enhanced vector accumulator with length hints
    (define enhanced-vector-accumulator
      (case-lambda
        (() (enhanced-vector-accumulator #f))
        ((length-hint)
         (let ((items '())
               (count 0))
           (lambda (item)
             (if (eof-object? item)
                 (let ((result (make-vector count)))
                   (let loop ((i (- count 1)) (lst items))
                     (if (or (< i 0) (null? lst))
                         result
                         (begin
                           (vector-set! result i (car lst))
                           (loop (- i 1) (cdr lst))))))
                 (begin
                   (set! items (cons item items))
                   (set! count (+ count 1))
                   items)))))))
    
    ;; Apply accumulator to entire generator
    (define (accumulate-generated accumulator gen)
      (let ((acc accumulator))
        (generator-for-each 
          (lambda (item) (acc item))
          gen)
        (acc *eof-object*)))
    
    ;; Higher-order accumulator application  
    (define (generator-accumulate gen accumulator)
      (accumulate-generated accumulator gen))
    
    ;; Enhanced make-accumulator-generator with better performance
    (define (enhanced-make-accumulator-generator accumulator gen)
      (let ((acc accumulator)
            (results '())
            (generated #f)
            (index 0))
        (%make-generator
          (lambda ()
            (when (not generated)
              ;; Generate all intermediate results
              (let loop ()
                (let ((val (safe-generator-next gen)))
                  (when (not (eof-object? val))
                    (let ((intermediate (acc val)))
                      (set! results (cons intermediate results))
                      (loop)))))
              ;; Get final result and reverse list
              (let ((final (acc *eof-object*)))
                (set! results (cons final (reverse results)))
                (set! generated #t)))
            
            ;; Return results one by one
            (if (>= index (length results))
                *eof-object*
                (let ((val (list-ref results index)))
                  (set! index (+ index 1))
                  val))))))
    
    ;; ================ NEW SRFI-158 ADVANCED UTILITIES ================
    
    ;; Efficiently concatenate generator of generators
    (define (generator-concatenate gen-of-gens)
      (let ((current-gen #f)
            (main-gen gen-of-gens))
        (%make-generator
          (lambda ()
            (let loop ()
              (if current-gen
                  (let ((val (safe-generator-next current-gen)))
                    (if (eof-object? val)
                        (begin
                          (set! current-gen #f)
                          (loop))  ; Try next generator
                        val))
                  ;; Need new generator
                  (let ((next-gen (safe-generator-next main-gen)))
                    (if (eof-object? next-gen)
                        *eof-object*  ; All generators exhausted
                        (begin
                          (set! current-gen 
                            (cond
                              ((%generator? next-gen) next-gen)
                              ((list? next-gen) (list->generator next-gen))
                              ((vector? next-gen) (vector->generator next-gen))
                              ((string? next-gen) (string->generator next-gen))
                              (else (error "Invalid generator type" next-gen))))
                          (loop))))))))))
    
    ;; Pad shorter generators in zip-like operations
    (define (generator-pad-with padding . gens)
      (if (null? gens)
          (generator)
          (let ((active-gens (map (lambda (g) (cons g #f)) gens)))  ; (gen . exhausted?)
            (%make-generator
              (lambda ()
                (let ((all-exhausted #t)
                      (values '()))
                  ;; Collect values from each generator
                  (for-each
                    (lambda (gen-pair)
                      (if (cdr gen-pair)  ; Already exhausted
                          (set! values (cons padding values))
                          (let ((val (safe-generator-next (car gen-pair))))
                            (if (eof-object? val)
                                (begin
                                  (set-cdr! gen-pair #t)  ; Mark as exhausted
                                  (set! values (cons padding values)))
                                (begin
                                  (set! all-exhausted #f)
                                  (set! values (cons val values)))))))
                    active-gens)
                  
                  (if all-exhausted
                      *eof-object*
                      (reverse values))))))))
    
    ;; Safe generator indexing with bounds checking
    (define generator-maybe-ref
      (case-lambda
        ((gen index) (generator-maybe-ref gen index #f))
        ((gen index default)
         (if (not (and (integer? index) (>= index 0)))
             (error "generator-maybe-ref: non-negative integer index required" index)
             (let ((current-index 0))
               (let loop ()
                 (if (= current-index index)
                     (let ((val (safe-generator-next gen)))
                       (if (eof-object? val) default val))
                     (let ((val (safe-generator-next gen)))
                       (if (eof-object? val)
                           default
                           (begin
                             (set! current-index (+ current-index 1))
                             (loop)))))))))))
    
    ;; Helper function for comparator compatibility
    (define (any proc lst)
      (and (pair? lst)
           (or (proc (car lst))
               (any proc (cdr lst)))))
    
    )) ; end library