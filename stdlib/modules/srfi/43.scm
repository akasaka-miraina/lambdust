;; SRFI-43: Vector Library
;; 
;; This library provides comprehensive vector manipulation utilities, similar to
;; SRFI-1 for lists but specialized for vectors. It includes constructors, 
;; predicates, selectors, iteration, searching, and mutating operations.
;;
;; Reference: https://srfi.schemers.org/srfi-43/srfi-43.html

(define-library (srfi 43)
  (import (scheme base)
          (scheme case-lambda))
  
  (export
    ;; === Constructors ===
    make-vector vector vector-unfold vector-unfold-right vector-copy
    vector-reverse-copy vector-append vector-concatenate
    
    ;; === Predicates ===
    vector? vector-empty? vector=
    
    ;; === Selectors ===
    vector-ref vector-length
    
    ;; === Iteration ===
    vector-fold vector-fold-right vector-reduce vector-reduce-right
    vector-map vector-map! vector-for-each vector-count
    
    ;; === Searching ===
    vector-index vector-index-right vector-skip vector-skip-right
    vector-binary-search vector-any vector-every
    
    ;; === Mutators ===
    vector-set! vector-swap! vector-fill! vector-reverse!
    vector-copy! vector-reverse-copy!
    
    ;; === Conversion ===
    vector->list reverse-vector->list list->vector reverse-list->vector
    
    ;; === Advanced ===
    vector-partition vector-filter vector-remove)

  (begin
    ;; ============= PREDICATES =============
    
    (define (vector-empty? vec)
      "Test whether the vector is empty."
      (= (vector-length vec) 0))
    
    (define vector=
      (case-lambda
        ((elt=) #t)
        ((elt= vec1) #t)
        ((elt= vec1 vec2)
         (let ((len1 (vector-length vec1))
               (len2 (vector-length vec2)))
           (and (= len1 len2)
                (let loop ((i 0))
                  (or (= i len1)
                      (and (elt= (vector-ref vec1 i) (vector-ref vec2 i))
                           (loop (+ i 1))))))))
        ((elt= vec1 vec2 . vecs)
         (let loop ((vecs (cons vec1 (cons vec2 vecs))))
           (or (null? (cdr vecs))
               (and (vector= elt= (car vecs) (cadr vecs))
                    (loop (cdr vecs))))))))
    
    ;; ============= CONSTRUCTORS =============
    
    (define vector-unfold
      (case-lambda
        ((f length initial-seed)
         (vector-unfold f length initial-seed values))
        ((f length initial-seed next-seed)
         (let ((result (make-vector length)))
           (let loop ((i 0) (seed initial-seed))
             (if (= i length)
                 result
                 (begin
                   (vector-set! result i (f i seed))
                   (loop (+ i 1) (next-seed seed)))))))))
    
    (define vector-unfold-right
      (case-lambda
        ((f length initial-seed)
         (vector-unfold-right f length initial-seed values))
        ((f length initial-seed next-seed)
         (let ((result (make-vector length)))
           (let loop ((i (- length 1)) (seed initial-seed))
             (if (< i 0)
                 result
                 (begin
                   (vector-set! result i (f i seed))
                   (loop (- i 1) (next-seed seed)))))))))
    
    (define vector-copy
      (case-lambda
        ((vec) (vector-copy vec 0 (vector-length vec)))
        ((vec start) (vector-copy vec start (vector-length vec)))
        ((vec start end)
         (let* ((len (- end start))
                (result (make-vector len)))
           (let loop ((i 0))
             (if (= i len)
                 result
                 (begin
                   (vector-set! result i (vector-ref vec (+ start i)))
                   (loop (+ i 1)))))))))
    
    (define vector-reverse-copy
      (case-lambda
        ((vec) (vector-reverse-copy vec 0 (vector-length vec)))
        ((vec start) (vector-reverse-copy vec start (vector-length vec)))
        ((vec start end)
         (let* ((len (- end start))
                (result (make-vector len)))
           (let loop ((i 0))
             (if (= i len)
                 result
                 (begin
                   (vector-set! result i (vector-ref vec (- end i 1)))
                   (loop (+ i 1)))))))))
    
    (define (vector-append . vecs)
      "Concatenate vectors into a new vector."
      (if (null? vecs)
          (vector)
          (let* ((lengths (map vector-length vecs))
                 (total-length (apply + lengths))
                 (result (make-vector total-length)))
            (let loop ((vecs vecs) (lengths lengths) (pos 0))
              (if (null? vecs)
                  result
                  (let ((vec (car vecs))
                        (len (car lengths)))
                    (let copy-loop ((i 0))
                      (if (= i len)
                          (loop (cdr vecs) (cdr lengths) (+ pos len))
                          (begin
                            (vector-set! result (+ pos i) (vector-ref vec i))
                            (copy-loop (+ i 1)))))))))))
    
    (define (vector-concatenate list-of-vectors)
      "Concatenate a list of vectors into a single vector."
      (apply vector-append list-of-vectors))
    
    ;; ============= ITERATION =============
    
    (define vector-fold
      (case-lambda
        ((kons knil vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0) (acc knil))
             (if (= i len)
                 acc
                 (loop (+ i 1) (kons i (vector-ref vec i) acc))))))
        ((kons knil vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0) (acc knil))
             (if (= i min-len)
                 acc
                 (loop (+ i 1)
                       (apply kons i 
                              (append (map (lambda (v) (vector-ref v i)) all-vecs)
                                      (list acc))))))))))
    
    (define vector-fold-right
      (case-lambda
        ((kons knil vec)
         (let ((len (vector-length vec)))
           (let loop ((i (- len 1)) (acc knil))
             (if (< i 0)
                 acc
                 (loop (- i 1) (kons i (vector-ref vec i) acc))))))
        ((kons knil vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i (- min-len 1)) (acc knil))
             (if (< i 0)
                 acc
                 (loop (- i 1)
                       (apply kons i
                              (append (map (lambda (v) (vector-ref v i)) all-vecs)
                                      (list acc))))))))))
    
    (define (vector-reduce f ridentity vec)
      "Reduce vector using binary operator f, with ridentity for empty vector."
      (let ((len (vector-length vec)))
        (cond
          ((= len 0) ridentity)
          ((= len 1) (vector-ref vec 0))
          (else
           (let loop ((i 1) (acc (vector-ref vec 0)))
             (if (= i len)
                 acc
                 (loop (+ i 1) (f acc (vector-ref vec i)))))))))
    
    (define (vector-reduce-right f ridentity vec)
      "Reduce vector from right using binary operator f."
      (let ((len (vector-length vec)))
        (cond
          ((= len 0) ridentity)
          ((= len 1) (vector-ref vec 0))
          (else
           (let loop ((i (- len 2)) (acc (vector-ref vec (- len 1))))
             (if (< i 0)
                 acc
                 (loop (- i 1) (f (vector-ref vec i) acc))))))))
    
    (define vector-map
      (case-lambda
        ((f vec)
         (let* ((len (vector-length vec))
                (result (make-vector len)))
           (let loop ((i 0))
             (if (= i len)
                 result
                 (begin
                   (vector-set! result i (f (vector-ref vec i)))
                   (loop (+ i 1)))))))
        ((f vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs)))
                (result (make-vector min-len)))
           (let loop ((i 0))
             (if (= i min-len)
                 result
                 (begin
                   (vector-set! result i
                                (apply f (map (lambda (v) (vector-ref v i)) all-vecs)))
                   (loop (+ i 1)))))))))
    
    (define vector-map!
      (case-lambda
        ((f vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0))
             (if (= i len)
                 vec
                 (begin
                   (vector-set! vec i (f (vector-ref vec i)))
                   (loop (+ i 1)))))))
        ((f vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0))
             (if (= i min-len)
                 vec1
                 (begin
                   (vector-set! vec1 i
                                (apply f (map (lambda (v) (vector-ref v i)) all-vecs)))
                   (loop (+ i 1)))))))))
    
    (define vector-for-each
      (case-lambda
        ((f vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0))
             (if (= i len)
                 (if #f #f)  ; Return unspecified value
                 (begin
                   (f (vector-ref vec i))
                   (loop (+ i 1)))))))
        ((f vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0))
             (if (= i min-len)
                 (if #f #f)  ; Return unspecified value
                 (begin
                   (apply f (map (lambda (v) (vector-ref v i)) all-vecs))
                   (loop (+ i 1)))))))))
    
    (define vector-count
      (case-lambda
        ((pred vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0) (count 0))
             (if (= i len)
                 count
                 (loop (+ i 1) 
                       (if (pred (vector-ref vec i)) (+ count 1) count))))))
        ((pred vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0) (count 0))
             (if (= i min-len)
                 count
                 (loop (+ i 1)
                       (if (apply pred (map (lambda (v) (vector-ref v i)) all-vecs))
                           (+ count 1)
                           count))))))))
    
    ;; ============= SEARCHING =============
    
    (define vector-index
      (case-lambda
        ((pred vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0))
             (if (= i len)
                 #f
                 (if (pred (vector-ref vec i))
                     i
                     (loop (+ i 1)))))))
        ((pred vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0))
             (if (= i min-len)
                 #f
                 (if (apply pred (map (lambda (v) (vector-ref v i)) all-vecs))
                     i
                     (loop (+ i 1)))))))))
    
    (define vector-index-right
      (case-lambda
        ((pred vec)
         (let ((len (vector-length vec)))
           (let loop ((i (- len 1)))
             (if (< i 0)
                 #f
                 (if (pred (vector-ref vec i))
                     i
                     (loop (- i 1)))))))
        ((pred vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i (- min-len 1)))
             (if (< i 0)
                 #f
                 (if (apply pred (map (lambda (v) (vector-ref v i)) all-vecs))
                     i
                     (loop (- i 1)))))))))
    
    (define vector-skip
      (case-lambda
        ((pred vec) (vector-index (lambda (x) (not (pred x))) vec))
        ((pred vec1 vec2 . vecs)
         (apply vector-index (lambda args (not (apply pred args))) vec1 vec2 vecs))))
    
    (define vector-skip-right
      (case-lambda
        ((pred vec) (vector-index-right (lambda (x) (not (pred x))) vec))
        ((pred vec1 vec2 . vecs)
         (apply vector-index-right (lambda args (not (apply pred args))) vec1 vec2 vecs))))
    
    (define (vector-binary-search vec value cmp)
      "Binary search in sorted vector. Returns index or #f if not found."
      (let ((len (vector-length vec)))
        (let loop ((low 0) (high (- len 1)))
          (if (> low high)
              #f
              (let* ((mid (quotient (+ low high) 2))
                     (mid-val (vector-ref vec mid))
                     (comparison (cmp value mid-val)))
                (cond
                  ((= comparison 0) mid)
                  ((< comparison 0) (loop low (- mid 1)))
                  (else (loop (+ mid 1) high))))))))
    
    (define vector-any
      (case-lambda
        ((pred vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0))
             (if (= i len)
                 #f
                 (or (pred (vector-ref vec i))
                     (loop (+ i 1)))))))
        ((pred vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0))
             (if (= i min-len)
                 #f
                 (or (apply pred (map (lambda (v) (vector-ref v i)) all-vecs))
                     (loop (+ i 1)))))))))
    
    (define vector-every
      (case-lambda
        ((pred vec)
         (let ((len (vector-length vec)))
           (let loop ((i 0))
             (if (= i len)
                 #t
                 (and (pred (vector-ref vec i))
                      (loop (+ i 1)))))))
        ((pred vec1 vec2 . vecs)
         (let* ((all-vecs (cons vec1 (cons vec2 vecs)))
                (min-len (apply min (map vector-length all-vecs))))
           (let loop ((i 0))
             (if (= i min-len)
                 #t
                 (and (apply pred (map (lambda (v) (vector-ref v i)) all-vecs))
                      (loop (+ i 1)))))))))
    
    ;; ============= MUTATORS =============
    
    (define (vector-swap! vec i j)
      "Swap elements at indices i and j in vector."
      (let ((temp (vector-ref vec i)))
        (vector-set! vec i (vector-ref vec j))
        (vector-set! vec j temp)))
    
    (define vector-fill!
      (case-lambda
        ((vec fill) (vector-fill! vec fill 0 (vector-length vec)))
        ((vec fill start) (vector-fill! vec fill start (vector-length vec)))
        ((vec fill start end)
         (let loop ((i start))
           (if (= i end)
               vec
               (begin
                 (vector-set! vec i fill)
                 (loop (+ i 1))))))))
    
    (define (vector-reverse! vec . maybe-start-end)
      "Reverse vector in place."
      (let* ((start (if (null? maybe-start-end) 0 (car maybe-start-end)))
             (end (if (or (null? maybe-start-end) (null? (cdr maybe-start-end)))
                      (vector-length vec)
                      (cadr maybe-start-end))))
        (let loop ((i start) (j (- end 1)))
          (if (>= i j)
              vec
              (begin
                (vector-swap! vec i j)
                (loop (+ i 1) (- j 1)))))))
    
    (define vector-copy!
      (case-lambda
        ((to at from) (vector-copy! to at from 0 (vector-length from)))
        ((to at from start) (vector-copy! to at from start (vector-length from)))
        ((to at from start end)
         (let ((len (- end start)))
           (if (< at (+ start len))
               ;; Overlapping copy, copy backwards
               (let loop ((i (- len 1)))
                 (if (< i 0)
                     to
                     (begin
                       (vector-set! to (+ at i) (vector-ref from (+ start i)))
                       (loop (- i 1)))))
               ;; Non-overlapping copy, copy forwards
               (let loop ((i 0))
                 (if (= i len)
                     to
                     (begin
                       (vector-set! to (+ at i) (vector-ref from (+ start i)))
                       (loop (+ i 1))))))))))
    
    (define vector-reverse-copy!
      (case-lambda
        ((to at from) (vector-reverse-copy! to at from 0 (vector-length from)))
        ((to at from start) (vector-reverse-copy! to at from start (vector-length from)))
        ((to at from start end)
         (let ((len (- end start)))
           (let loop ((i 0))
             (if (= i len)
                 to
                 (begin
                   (vector-set! to (+ at i) (vector-ref from (- end i 1)))
                   (loop (+ i 1)))))))))
    
    ;; ============= CONVERSION =============
    
    (define vector->list
      (case-lambda
        ((vec) (vector->list vec 0 (vector-length vec)))
        ((vec start) (vector->list vec start (vector-length vec)))
        ((vec start end)
         (let loop ((i (- end 1)) (result '()))
           (if (< i start)
               result
               (loop (- i 1) (cons (vector-ref vec i) result)))))))
    
    (define reverse-vector->list
      (case-lambda
        ((vec) (reverse-vector->list vec 0 (vector-length vec)))
        ((vec start) (reverse-vector->list vec start (vector-length vec)))
        ((vec start end)
         (let loop ((i start) (result '()))
           (if (= i end)
               result
               (loop (+ i 1) (cons (vector-ref vec i) result)))))))
    
    (define list->vector
      (case-lambda
        ((lst) (apply vector lst))
        ((lst start) (list->vector (list-tail lst start)))
        ((lst start end)
         (let ((sub-list (let loop ((l lst) (i 0) (result '()))
                           (cond
                             ((= i end) (reverse result))
                             ((< i start) (loop (cdr l) (+ i 1) result))
                             (else (loop (cdr l) (+ i 1) (cons (car l) result)))))))
           (apply vector sub-list)))))
    
    (define (reverse-list->vector lst . maybe-start-end)
      "Convert list to vector with elements in reverse order."
      (let* ((start (if (null? maybe-start-end) 0 (car maybe-start-end)))
             (end (if (or (null? maybe-start-end) (null? (cdr maybe-start-end)))
                      (length lst)
                      (cadr maybe-start-end)))
             (sub-list (let loop ((l lst) (i 0) (result '()))
                         (cond
                           ((= i end) result)
                           ((< i start) (loop (cdr l) (+ i 1) result))
                           (else (loop (cdr l) (+ i 1) (cons (car l) result)))))))
        (apply vector sub-list)))
    
    ;; ============= ADVANCED OPERATIONS =============
    
    (define (vector-partition pred vec)
      "Partition vector into two vectors based on predicate."
      (let* ((len (vector-length vec))
             (trues '())
             (falses '()))
        (let loop ((i 0))
          (if (= i len)
              (values (apply vector (reverse trues))
                      (apply vector (reverse falses)))
              (let ((elem (vector-ref vec i)))
                (if (pred elem)
                    (begin
                      (set! trues (cons elem trues))
                      (loop (+ i 1)))
                    (begin
                      (set! falses (cons elem falses))
                      (loop (+ i 1)))))))))
    
    (define (vector-filter pred vec)
      "Create new vector containing elements that satisfy predicate."
      (let* ((len (vector-length vec))
             (result '()))
        (let loop ((i 0))
          (if (= i len)
              (apply vector (reverse result))
              (let ((elem (vector-ref vec i)))
                (if (pred elem)
                    (begin
                      (set! result (cons elem result))
                      (loop (+ i 1)))
                    (loop (+ i 1))))))))
    
    (define (vector-remove pred vec)
      "Create new vector containing elements that don't satisfy predicate."
      (vector-filter (lambda (x) (not (pred x))) vec))
    
    ;; ============= UTILITIES AND EXAMPLES =============
    
    ;; Example usage patterns:
    
    ;; Creating vectors with patterns:
    ;; (vector-unfold (lambda (i x) (* i x)) 5 2 (lambda (x) (+ x 1)))
    ;; => #(0 2 6 12 20)
    
    ;; Functional-style vector processing:
    ;; (vector-map (lambda (x) (* x x)) #(1 2 3 4 5))
    ;; => #(1 4 9 16 25)
    
    ;; Searching and testing:
    ;; (vector-any positive? #(-1 -2 3 -4))  => #t
    ;; (vector-every number? #(1 2 3 4))     => #t
    ;; (vector-index even? #(1 3 4 7))       => 2
    
    ;; Efficient accumulation:
    ;; (vector-fold (lambda (i elem acc) (+ elem acc)) 0 #(1 2 3 4))
    ;; => 10
    
    ;; Binary search (vector must be sorted):
    ;; (vector-binary-search #(1 3 5 7 9) 5 (lambda (a b) (- a b)))
    ;; => 2
    ))