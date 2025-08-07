;; Lambdust R7RS-Compliant Vector Module
;; Provides complete R7RS Section 6.8 vector operations with extensions

(define-module (:: vector)
  (metadata
    (version "2.0.0")
    (description "R7RS-compliant vector operations with practical extensions")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.8"))
  
  (export 
    ;; === R7RS Section 6.8 Vector Procedures ===
    
    ;; Core vector operations
    vector? make-vector vector vector-length
    
    ;; Vector access and mutation
    vector-ref vector-set!
    
    ;; Vector copying and manipulation
    vector-copy vector-copy! vector-fill!
    
    ;; Vector construction
    vector-append
    
    ;; Vector/list conversions
    vector->list list->vector
    
    ;; Vector/string conversions
    vector->string string->vector
    
    ;; Vector iteration (higher-order functions)
    vector-for-each vector-map
    
    ;; === Additional Lambdust Extensions ===
    ;; (Beyond R7RS but commonly useful)
    
    ;; Extended vector predicates
    vector-empty? vector-equal?
    
    ;; Vector manipulation
    vector-take vector-drop vector-reverse vector-reverse!
    
    ;; Vector searching and filtering
    vector-find vector-filter vector-remove
    vector-any vector-every vector-count
    vector-index vector-index-right
    
    ;; Vector transformation
    vector-fold vector-fold-right)

  ;; ============= R7RS Core Vector Operations =============

  (define (vector? obj)
    "Returns #t if obj is a vector, #f otherwise.
     
     R7RS: (vector? obj) procedure
     The vector? predicate returns #t if obj is a vector, and otherwise returns #f."
    (builtin:vector? obj))

  (define (make-vector k . fill)
    "Returns a newly allocated vector of k elements.
     
     R7RS: (make-vector k) procedure
           (make-vector k fill) procedure
     The make-vector procedure returns a newly allocated vector of k elements.
     If a second argument is given, then each element is initialized to fill.
     Otherwise the initial contents of each element is unspecified."
    (let ((fill-val (if (null? fill) #f (car fill))))
      (builtin:make-vector k fill-val)))

  (define (vector . objs)
    "Returns a vector whose elements are the given arguments.
     
     R7RS: (vector obj ...) procedure
     The vector procedure returns a newly allocated vector whose elements
     contain the given arguments. It is analogous to list."
    (list->vector objs))

  (define (vector-length vec)
    "Returns the number of elements in vec.
     
     R7RS: (vector-length vector) procedure
     Returns the number of elements in vector as an exact integer."
    (builtin:vector-length vec))

  ;; ============= R7RS Vector Access and Mutation =============

  (define (vector-ref vec k)
    "Returns the kth element of vec.
     
     R7RS: (vector-ref vector k) procedure
     The vector-ref procedure returns the contents of element k of vector.
     It is an error if k is not a valid index of vector."
    (builtin:vector-ref vec k))

  (define (vector-set! vec k obj)
    "Sets the kth element of vec to obj.
     
     R7RS: (vector-set! vector k obj) procedure
     The vector-set! procedure stores obj in element k of vector.
     It is an error if k is not a valid index of vector.
     This procedure returns an unspecified value."
    (builtin:vector-set! vec k obj))

  ;; ============= R7RS Vector Copying and Manipulation =============

  (define (vector-copy vec . start-end)
    "Returns a copy of vec.
     
     R7RS: (vector-copy vector) procedure
           (vector-copy vector start) procedure
           (vector-copy vector start end) procedure
     Returns a newly allocated copy of the elements of the given vector
     between start and end. The end index is exclusive."
    (if (null? start-end)
        (builtin:vector-copy vec)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (vector-length vec))))
          (builtin:vector-copy vec start end))))

  (define (vector-copy! to at from . start-end)
    "Copies elements from from to to.
     
     R7RS: (vector-copy! to at from) procedure
           (vector-copy! to at from start) procedure
           (vector-copy! to at from start end) procedure
     Copies the elements of vector from between start and end to vector to,
     starting at at. The order in which elements are copied is unspecified,
     except that if the source and destination overlap, copying takes place
     as if the source is first copied into a temporary vector and then into
     the destination. This procedure returns an unspecified value."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length from)
                   (cadr start-end))))
      (builtin:vector-copy! to at from start end)))

  (define (vector-fill! vec fill . start-end)
    "Fills vec with fill.
     
     R7RS: (vector-fill! vector fill) procedure
           (vector-fill! vector fill start) procedure
           (vector-fill! vector fill start end) procedure
     Stores fill in the elements of vector between start and end.
     This procedure returns an unspecified value."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (builtin:vector-fill! vec fill start end)))

  ;; ============= R7RS Vector Construction =============

  (define (vector-append . vectors)
    "Returns a vector formed by concatenating the vectors.
     
     R7RS: (vector-append vector ...) procedure
     Returns a newly allocated vector whose elements are the concatenation
     of the elements of the given vectors."
    (if (null? vectors)
        (vector)
        (builtin:vector-append-list vectors)))

  ;; ============= R7RS Vector/List Conversions =============

  (define (vector->list vec . start-end)
    "Returns a list of the elements of vec.
     
     R7RS: (vector->list vector) procedure
           (vector->list vector start) procedure
           (vector->list vector start end) procedure
     The vector->list procedure returns a newly allocated list of the
     objects contained in the elements of vector between start and end.
     The list->vector procedure returns a newly allocated vector whose
     elements contain the given objects in the same order."
    (if (null? start-end)
        (builtin:vector->list vec)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (vector-length vec))))
          (builtin:vector->list vec start end))))

  (define (list->vector lst)
    "Returns a vector formed from the elements of lst.
     
     R7RS: (list->vector list) procedure
     The list->vector procedure returns a newly allocated vector whose
     elements contain the given objects in the same order. It is analogous
     to the string->list and list->string procedures."
    (builtin:list->vector lst))

  ;; ============= R7RS Vector/String Conversions =============

  (define (vector->string vec . start-end)
    "Returns a string formed from the character elements of vec.
     
     R7RS: (vector->string vector) procedure
           (vector->string vector start) procedure
           (vector->string vector start end) procedure
     The vector->string procedure returns a newly allocated string of the
     objects contained in the elements of vector between start and end.
     It is an error if any element of vector between start and end is not a character."
    (if (null? start-end)
        (builtin:vector->string vec)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (vector-length vec))))
          (builtin:vector->string vec start end))))

  (define (string->vector str . start-end)
    "Returns a vector formed from the characters of str.
     
     R7RS: (string->vector string) procedure
           (string->vector string start) procedure
           (string->vector string start end) procedure
     The string->vector procedure returns a newly allocated vector of the
     characters that make up the given string between start and end."
    (if (null? start-end)
        (builtin:string->vector str)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (string-length str))))
          (builtin:string->vector str start end))))

  ;; ============= R7RS Vector Iteration (Higher-Order Functions) =============

  (define (vector-for-each proc vec . vectors)
    "Applies proc to each element of the vectors.
     
     R7RS: (vector-for-each proc vector1 vector2 ...) procedure
     The vector-for-each procedure applies proc element-wise to the elements
     of the vectors for its side effects, in order from the first element(s)
     to the last. The proc procedure is always called with the same number
     of arguments as there are vectors. The vector-for-each procedure returns
     an unspecified value."
    (define (vector-for-each-helper vectors index)
      (when (< index (apply min (map vector-length vectors)))
        (apply proc (map (lambda (v) (vector-ref v index)) vectors))
        (vector-for-each-helper vectors (+ index 1))))
    (vector-for-each-helper (cons vec vectors) 0))

  (define (vector-map proc vec . vectors)
    "Returns a vector formed by applying proc to each element.
     
     R7RS: (vector-map proc vector1 vector2 ...) procedure
     The vector-map procedure applies proc element-wise to the elements of the
     vectors and returns a vector of the results, in order. The proc procedure
     is always called with the same number of arguments as there are vectors.
     If more than one vector is given and not all vectors have the same length,
     vector-map terminates when the shortest vector runs out."
    (define (vector-map-helper vectors index acc)
      (if (< index (apply min (map vector-length vectors)))
          (let ((result (apply proc (map (lambda (v) (vector-ref v index)) vectors))))
            (vector-map-helper vectors (+ index 1) (cons result acc)))
          (reverse acc)))
    (list->vector (vector-map-helper (cons vec vectors) 0 '())))

  ;; ============= Additional Lambdust Vector Extensions =============
  ;; (Beyond R7RS but commonly useful)

  (define (vector-empty? vec)
    "Returns #t if vec is empty.
     
     Extension: Convenience predicate for empty vectors."
    (zero? (vector-length vec)))

  (define (vector-equal? vec1 vec2 . element=?)
    "Returns #t if vectors are equal element-wise.
     
     Extension: Element-wise vector comparison with custom predicate."
    (let ((elem-equal (if (null? element=?) equal? (car element=?))))
      (and (= (vector-length vec1) (vector-length vec2))
           (vector-equal-helper vec1 vec2 elem-equal 0))))

  (define (vector-equal-helper vec1 vec2 elem-equal i)
    "Helper for vector-equal?."
    (cond
      ((>= i (vector-length vec1)) #t)
      ((elem-equal (vector-ref vec1 i) (vector-ref vec2 i))
       (vector-equal-helper vec1 vec2 elem-equal (+ i 1)))
      (else #f)))

  ;; === Vector Manipulation Extensions ===

  (define (vector-take vec k)
    "Returns a vector containing the first k elements of vec.
     
     Extension: Prefix operation for vectors."
    (vector-copy vec 0 k))

  (define (vector-drop vec k)
    "Returns a vector with the first k elements removed.
     
     Extension: Suffix operation for vectors."
    (vector-copy vec k))

  (define (vector-reverse vec)
    "Returns a vector with elements in reverse order.
     
     Extension: Functional vector reversal."
    (let* ((len (vector-length vec))
           (result (make-vector len)))
      (do ((i 0 (+ i 1)))
          ((= i len) result)
        (vector-set! result i (vector-ref vec (- len 1 i))))))

  (define (vector-reverse! vec)
    "Reverses vec in place.
     
     Extension: Destructive vector reversal."
    (let ((len (vector-length vec)))
      (do ((i 0 (+ i 1)))
          ((>= i (quotient len 2)) vec)
        (let ((temp (vector-ref vec i)))
          (vector-set! vec i (vector-ref vec (- len 1 i)))
          (vector-set! vec (- len 1 i) temp)))))

  ;; ============= Vector Searching =============

  (define (vector-index pred vec . start-end)
    "Returns the index of the first element satisfying pred."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-index-helper pred vec start end)))

  (define (vector-index-helper pred vec start end)
    "Helper for vector-index."
    (cond
      ((>= start end) #f)
      ((pred (vector-ref vec start)) start)
      (else (vector-index-helper pred vec (+ start 1) end))))

  (define (vector-index-right pred vec . start-end)
    "Returns the index of the last element satisfying pred."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-index-right-helper pred vec start (- end 1))))

  (define (vector-index-right-helper pred vec start end)
    "Helper for vector-index-right."
    (cond
      ((< end start) #f)
      ((pred (vector-ref vec end)) end)
      (else (vector-index-right-helper pred vec start (- end 1)))))

  (define (vector-skip pred vec . start-end)
    "Returns the index of the first element not satisfying pred."
    (apply vector-index (lambda (x) (not (pred x))) vec start-end))

  (define (vector-skip-right pred vec . start-end)
    "Returns the index of the last element not satisfying pred."
    (apply vector-index-right (lambda (x) (not (pred x))) vec start-end))

  (define (vector-binary-search vec value compare . start-end)
    "Binary search for value in sorted vec."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-binary-search-helper vec value compare start end)))

  (define (vector-binary-search-helper vec value compare start end)
    "Helper for vector-binary-search."
    (if (>= start end)
        #f
        (let* ((mid (quotient (+ start end) 2))
               (mid-val (vector-ref vec mid))
               (cmp (compare value mid-val)))
          (cond
            ((zero? cmp) mid)
            ((negative? cmp) (vector-binary-search-helper vec value compare start mid))
            (else (vector-binary-search-helper vec value compare (+ mid 1) end))))))

  ;; ============= Vector Higher-order Functions =============

  (define (vector-any pred vec . start-end)
    "Returns #t if pred is true for any element."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-any-helper pred vec start end)))

  (define (vector-any-helper pred vec start end)
    "Helper for vector-any."
    (cond
      ((>= start end) #f)
      ((pred (vector-ref vec start)) #t)
      (else (vector-any-helper pred vec (+ start 1) end))))

  (define (vector-every pred vec . start-end)
    "Returns #t if pred is true for every element."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-every-helper pred vec start end)))

  (define (vector-every-helper pred vec start end)
    "Helper for vector-every."
    (cond
      ((>= start end) #t)
      ((pred (vector-ref vec start)) (vector-every-helper pred vec (+ start 1) end))
      (else #f)))

  (define (vector-count pred vec . start-end)
    "Returns the number of elements satisfying pred."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-count-helper pred vec start end 0)))

  (define (vector-count-helper pred vec start end count)
    "Helper for vector-count."
    (if (>= start end)
        count
        (vector-count-helper pred vec (+ start 1) end
                           (if (pred (vector-ref vec start)) (+ count 1) count))))

  (define (vector-fold proc init vec . start-end)
    "Left fold over vector elements."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-fold-helper proc init vec start end)))

  (define (vector-fold-helper proc init vec start end)
    "Helper for vector-fold."
    (if (>= start end)
        init
        (vector-fold-helper proc (proc (vector-ref vec start) init) vec (+ start 1) end)))

  (define (vector-fold-right proc init vec . start-end)
    "Right fold over vector elements."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-fold-right-helper proc init vec start (- end 1))))

  (define (vector-fold-right-helper proc init vec start end)
    "Helper for vector-fold-right."
    (if (< end start)
        init
        (proc (vector-ref vec end) 
              (vector-fold-right-helper proc init vec start (- end 1)))))

  ;; === Missing Extension Functions ===

  (define (vector-find pred vec . start-end)
    "Returns the first element satisfying pred, or #f if none.
     
     Extension: Element search operation."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (vector-find-helper pred vec start end)))

  (define (vector-find-helper pred vec start end)
    "Helper for vector-find."
    (cond
      ((>= start end) #f)
      ((pred (vector-ref vec start)) (vector-ref vec start))
      (else (vector-find-helper pred vec (+ start 1) end))))

  (define (vector-filter pred vec . start-end)
    "Returns a vector containing elements that satisfy pred.
     
     Extension: Functional filtering for vectors."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (vector-length vec)
                   (cadr start-end))))
      (list->vector (filter-helper pred (vector->list vec start end)))))

  (define (vector-remove pred vec . start-end)
    "Returns a vector containing elements that do not satisfy pred.
     
     Extension: Complement of vector-filter."
    (vector-filter (lambda (x) (not (pred x))) vec 
                   (if (null? start-end) 0 (car start-end))
                   (if (or (null? start-end) (< (length start-end) 2))
                       (vector-length vec)
                       (cadr start-end))))

  ;; === Helper function for filter ===
  (define (filter-helper pred lst)
    "Filter helper for vector operations."
    (cond
      ((null? lst) '())
      ((pred (car lst)) (cons (car lst) (filter-helper pred (cdr lst))))
      (else (filter-helper pred (cdr lst)))))

)