;; SRFI-41: Streams
;; 
;; This library provides streams, which are lazy sequences that allow for
;; efficient processing of potentially infinite data structures. Streams
;; are fundamental to functional programming and provide elegant solutions
;; to many algorithmic problems.
;;
;; Reference: https://srfi.schemers.org/srfi-41/srfi-41.html

(define-library (srfi 41)
  (import (scheme base)
          (scheme case-lambda))
  
  (export
    ;; === Stream Type ===
    stream-null stream-cons stream? stream-null? stream-pair?
    
    ;; === Stream Primitives ===
    stream-car stream-cdr stream-lambda
    
    ;; === Stream Library ===
    define-stream list->stream port->stream stream->list stream-append
    stream-concat stream-constant stream-drop stream-drop-while
    stream-filter stream-fold stream-for-each stream-from
    stream-iterate stream-length stream-let stream-map stream-match
    stream-of stream-range stream-ref stream-reverse stream-scan
    stream-take stream-take-while stream-unfold stream-unfolds
    stream-zip)

  (begin
    ;; ============= STREAM TYPE AND CONSTRUCTORS =============
    
    ;; The empty stream
    (define stream-null '())
    
    ;; Stream constructor - creates a lazy stream cell
    ;; Usage: (stream-cons expr stream-expr)
    (define-syntax stream-cons
      (syntax-rules ()
        ((_ expr stream-expr)
         (cons 'stream-pair (delay (cons expr stream-expr))))))
    
    ;; Stream type predicate
    (define (stream? obj)
      (or (null? obj)
          (and (pair? obj) (eq? (car obj) 'stream-pair))))
    
    ;; Empty stream predicate
    (define (stream-null? obj)
      (null? obj))
    
    ;; Non-empty stream predicate
    (define (stream-pair? obj)
      (and (pair? obj) (eq? (car obj) 'stream-pair)))
    
    ;; ============= STREAM PRIMITIVES =============
    
    ;; Get first element of stream
    (define (stream-car stream)
      (cond
        ((stream-null? stream)
         (error "stream-car: empty stream"))
        ((stream-pair? stream)
         (car (force (cdr stream))))
        (else
         (error "stream-car: not a stream" stream))))
    
    ;; Get rest of stream
    (define (stream-cdr stream)
      (cond
        ((stream-null? stream)
         (error "stream-cdr: empty stream"))
        ((stream-pair? stream)
         (cdr (force (cdr stream))))
        (else
         (error "stream-cdr: not a stream" stream))))
    
    ;; Stream lambda - creates a procedure that returns a stream
    (define-syntax stream-lambda
      (syntax-rules ()
        ((_ formals body0 body1 ...)
         (lambda formals body0 body1 ...))))
    
    ;; ============= STREAM LIBRARY =============
    
    ;; Define a stream variable
    (define-syntax define-stream
      (syntax-rules ()
        ((_ var expr)
         (define var expr))
        ((_ (name . formals) body0 body1 ...)
         (define name (stream-lambda formals body0 body1 ...)))))
    
    ;; Convert list to stream
    (define (list->stream lst)
      (if (null? lst)
          stream-null
          (stream-cons (car lst) (list->stream (cdr lst)))))
    
    ;; Convert port to stream of characters
    (define (port->stream port)
      (let ((ch (read-char port)))
        (if (eof-object? ch)
            stream-null
            (stream-cons ch (port->stream port)))))
    
    ;; Convert stream to list (potentially infinite - use with care!)
    (define stream->list
      (case-lambda
        ((stream) (stream->list stream #f))
        ((stream n)
         (let loop ((stream stream) (n n) (result '()))
           (cond
             ((and n (<= n 0)) (reverse result))
             ((stream-null? stream) (reverse result))
             (else
              (loop (stream-cdr stream)
                    (and n (- n 1))
                    (cons (stream-car stream) result))))))))
    
    ;; Append streams
    (define (stream-append . streams)
      (let loop ((streams streams))
        (cond
          ((null? streams) stream-null)
          ((stream-null? (car streams)) (loop (cdr streams)))
          (else
           (stream-cons (stream-car (car streams))
                        (stream-append (stream-cdr (car streams))
                                      (apply stream-append (cdr streams))))))))
    
    ;; Concatenate a stream of streams
    (define (stream-concat stream-of-streams)
      (if (stream-null? stream-of-streams)
          stream-null
          (stream-append (stream-car stream-of-streams)
                        (stream-concat (stream-cdr stream-of-streams)))))
    
    ;; Create infinite constant stream
    (define (stream-constant . objs)
      (if (null? objs)
          (error "stream-constant: no arguments")
          (let loop ((objs objs))
            (stream-cons (car objs)
                        (if (null? (cdr objs))
                            (loop objs)
                            (loop (cdr objs)))))))
    
    ;; Drop n elements from stream
    (define (stream-drop n stream)
      (cond
        ((< n 0) (error "stream-drop: negative argument" n))
        ((= n 0) stream)
        ((stream-null? stream) stream-null)
        (else (stream-drop (- n 1) (stream-cdr stream)))))
    
    ;; Drop elements while predicate is true
    (define (stream-drop-while pred stream)
      (cond
        ((stream-null? stream) stream-null)
        ((pred (stream-car stream))
         (stream-drop-while pred (stream-cdr stream)))
        (else stream)))
    
    ;; Filter stream elements
    (define (stream-filter pred stream)
      (cond
        ((stream-null? stream) stream-null)
        ((pred (stream-car stream))
         (stream-cons (stream-car stream)
                      (stream-filter pred (stream-cdr stream))))
        (else (stream-filter pred (stream-cdr stream)))))
    
    ;; Fold stream from left
    (define (stream-fold kons knil stream)
      (if (stream-null? stream)
          knil
          (stream-fold kons
                      (kons (stream-car stream) knil)
                      (stream-cdr stream))))
    
    ;; Apply procedure to each stream element (for side effects)
    (define (stream-for-each proc stream . streams)
      (if (null? streams)
          (let loop ((stream stream))
            (unless (stream-null? stream)
              (proc (stream-car stream))
              (loop (stream-cdr stream))))
          (let loop ((streams (cons stream streams)))
            (unless (any stream-null? streams)
              (apply proc (map stream-car streams))
              (loop (map stream-cdr streams))))))
    
    ;; Create arithmetic stream starting from n with step
    (define stream-from
      (case-lambda
        ((n) (stream-from n 1))
        ((n step)
         (stream-cons n (stream-from (+ n step) step)))))
    
    ;; Create stream by iterating function
    (define (stream-iterate f x)
      (stream-cons x (stream-iterate f (f x))))
    
    ;; Get length of finite stream
    (define (stream-length stream)
      (let loop ((stream stream) (count 0))
        (if (stream-null? stream)
            count
            (loop (stream-cdr stream) (+ count 1)))))
    
    ;; Stream let binding
    (define-syntax stream-let
      (syntax-rules ()
        ((_ tag ((var init) ...) body1 body2 ...)
         ((letrec ((tag (lambda (var ...) body1 body2 ...))) tag)
          init ...))))
    
    ;; Map function over streams
    (define (stream-map proc stream . streams)
      (if (null? streams)
          (if (stream-null? stream)
              stream-null
              (stream-cons (proc (stream-car stream))
                          (stream-map proc (stream-cdr stream))))
          (let loop ((streams (cons stream streams)))
            (if (any stream-null? streams)
                stream-null
                (stream-cons (apply proc (map stream-car streams))
                            (loop (map stream-cdr streams)))))))
    
    ;; Pattern matching for streams (simplified)
    (define-syntax stream-match
      (syntax-rules ()
        ((_ stream-expr
            (() body1 body2 ...)
            ((head-pat . tail-pat) body3 body4 ...)
            clause ...)
         (let ((s stream-expr))
           (cond
             ((stream-null? s) (begin body1 body2 ...))
             ((stream-pair? s)
              (let ((head-pat (stream-car s))
                    (tail-pat (stream-cdr s)))
                (begin body3 body4 ...)))
             (else (stream-match s clause ...)))))))
    
    ;; Stream comprehension (simplified)
    (define-syntax stream-of
      (syntax-rules (in is)
        ((_ expr (var in stream))
         (stream-map (lambda (var) expr) stream))
        ((_ expr (var in stream) (pred is boolean-expr))
         (stream-map (lambda (var) expr)
                    (stream-filter (lambda (var) boolean-expr) stream)))))
    
    ;; Create numeric range stream
    (define stream-range
      (case-lambda
        ((first past) (stream-range first past 1))
        ((first past step)
         (if (if (positive? step) (>= first past) (<= first past))
             stream-null
             (stream-cons first (stream-range (+ first step) past step))))))
    
    ;; Get nth element of stream (0-indexed)
    (define (stream-ref stream n)
      (cond
        ((< n 0) (error "stream-ref: negative index" n))
        ((stream-null? stream) (error "stream-ref: index out of bounds" n))
        ((= n 0) (stream-car stream))
        (else (stream-ref (stream-cdr stream) (- n 1)))))
    
    ;; Reverse finite stream
    (define (stream-reverse stream)
      (let loop ((stream stream) (result stream-null))
        (if (stream-null? stream)
            result
            (loop (stream-cdr stream)
                  (stream-cons (stream-car stream) result)))))
    
    ;; Scan stream (like fold but returns stream of intermediate results)
    (define (stream-scan kons knil stream)
      (if (stream-null? stream)
          (stream-cons knil stream-null)
          (stream-cons knil
                      (stream-scan kons
                                  (kons (stream-car stream) knil)
                                  (stream-cdr stream)))))
    
    ;; Take n elements from stream
    (define (stream-take n stream)
      (cond
        ((< n 0) (error "stream-take: negative argument" n))
        ((= n 0) stream-null)
        ((stream-null? stream) stream-null)
        (else
         (stream-cons (stream-car stream)
                      (stream-take (- n 1) (stream-cdr stream))))))
    
    ;; Take elements while predicate is true
    (define (stream-take-while pred stream)
      (cond
        ((stream-null? stream) stream-null)
        ((pred (stream-car stream))
         (stream-cons (stream-car stream)
                      (stream-take-while pred (stream-cdr stream))))
        (else stream-null)))
    
    ;; Unfold stream from seed
    (define (stream-unfold map pred gen seed)
      (if (pred seed)
          stream-null
          (stream-cons (map seed)
                      (stream-unfold map pred gen (gen seed)))))
    
    ;; Multiple-value unfold
    (define (stream-unfolds gen seed)
      (call-with-values
        (lambda () (gen seed))
        (lambda (next . results)
          (if (null? results)
              stream-null
              (apply values
                    (map (lambda (result-stream)
                           (stream-cons (car result-stream)
                                       (stream-unfolds gen next)))
                         results))))))
    
    ;; Zip streams together
    (define (stream-zip . streams)
      (if (any stream-null? streams)
          stream-null
          (stream-cons (map stream-car streams)
                      (apply stream-zip (map stream-cdr streams)))))
    
    ;; Helper function for any
    (define (any pred list)
      (and (not (null? list))
           (or (pred (car list))
               (any pred (cdr list)))))
    
    ;; ============= EXAMPLE USAGE =============
    
    ;; Example stream definitions for testing and demonstration
    
    ;; Natural numbers starting from 0
    (define nats (stream-from 0))
    
    ;; Even numbers
    (define evens (stream-filter even? nats))
    
    ;; Fibonacci sequence
    (define fibs
      (stream-cons 0
        (stream-cons 1
          (stream-map + fibs (stream-cdr fibs)))))
    
    ;; Prime numbers using sieve of Eratosthenes
    (define (sieve stream)
      (let ((first (stream-car stream)))
        (stream-cons first
          (sieve (stream-filter
                  (lambda (n) (not (zero? (modulo n first))))
                  (stream-cdr stream))))))
    
    (define primes (sieve (stream-from 2)))
    
    ;; Example: Powers of 2
    (define powers-of-2 (stream-iterate (lambda (x) (* x 2)) 1))
    
    ;; Example: Factorial sequence
    (define factorials
      (stream-cons 1
        (stream-map * (stream-from 1) factorials)))
    
    ;; ============= PERFORMANCE NOTES =============
    
    ;; Streams in this implementation use Scheme's delay/force mechanism
    ;; for lazy evaluation. This provides:
    ;; - Memoization: computed values are cached
    ;; - Space efficiency: only computed elements are stored
    ;; - Time efficiency: elements are computed only once
    
    ;; Best practices:
    ;; 1. Use stream-take or stream->list with a limit for infinite streams
    ;; 2. Be careful with stream-length on infinite streams
    ;; 3. stream-reverse requires the entire stream to be realized
    ;; 4. Streams work well with tail recursion optimization
    
    ;; Memory considerations:
    ;; - Holding a reference to the head of a long stream retains all elements
    ;; - Use stream-drop or recreate streams to allow garbage collection
    ;; - Very long finite streams may cause stack overflow in some operations
    
    ;; Integration with other SRFIs:
    ;; - Works well with SRFI-1 list procedures via stream->list/list->stream
    ;; - Complements SRFI-26 (cut/cute) for concise stream operations
    ;; - Can be used with SRFI-158 generators as an alternative lazy structure
    ))