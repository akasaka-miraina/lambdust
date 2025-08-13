;; SRFI-132: Sort Libraries
;;
;; This library provides a comprehensive set of sorting and merging procedures
;; for both lists and vectors. It includes stable and unstable sorting algorithms,
;; merge operations, and utilities for working with sorted sequences.
;;
;; Reference: https://srfi.schemers.org/srfi-132/srfi-132.html

(define-library (srfi 132)
  (import (scheme base)
          (srfi 128)) ; Comparators
  
  (export
    ;; List sorting procedures
    list-sort
    list-stable-sort
    list-sort!
    list-stable-sort!
    list-sorted?
    
    ;; Vector sorting procedures
    vector-sort
    vector-stable-sort
    vector-sort!
    vector-stable-sort!
    vector-sorted?
    
    ;; Merge procedures
    list-merge
    list-merge!
    vector-merge
    vector-merge!
    
    ;; Delete duplicate procedures
    list-delete-neighbor-dups
    list-delete-neighbor-dups!
    vector-delete-neighbor-dups
    vector-delete-neighbor-dups!
    
    ;; Advanced procedures
    vector-find-median
    vector-find-median!
    vector-select!
    vector-separate!
    )
  
  (begin
    
    ;; ============= ALGORITHM CONSTANTS =============
    
    ;; Threshold for switching from merge sort to insertion sort
    (define *insertion-sort-threshold* 32)
    
    ;; Threshold for switching from quicksort to insertion sort
    (define *quicksort-threshold* 16)
    
    ;; ============= UTILITY FUNCTIONS =============
    
    ;; Helper function to copy a subrange of a vector
    (define (vector-copy-range! target target-start source source-start source-end)
      (let loop ((i source-start) (j target-start))
        (if (< i source-end)
          (begin
            (vector-set! target j (vector-ref source i))
            (loop (+ i 1) (+ j 1))))))
    
    ;; Helper function to create a vector copy of a subrange
    (define (vector-subrange source start end)
      (let ((len (- end start)))
        (if (<= len 0)
          (vector)
          (let ((result (make-vector len)))
            (vector-copy-range! result 0 source start end)
            result))))
    
    ;; Helper function to parse optional start/end arguments for vectors
    (define (parse-vector-range vec args default-start default-end)
      (let ((len (vector-length vec)))
        (cond
          ((null? args) (values default-start (min default-end len)))
          ((null? (cdr args)) (values (car args) (min default-end len)))
          (else (values (car args) (min (cadr args) len))))))
    
    ;; Helper function to validate vector range arguments
    (define (validate-vector-range vec start end)
      (let ((len (vector-length vec)))
        (unless (and (<= 0 start) (<= start end) (<= end len))
          (error "invalid vector range" start end len))))
    
    ;; Helper function to swap elements in a vector
    (define (vector-swap! vec i j)
      (let ((temp (vector-ref vec i)))
        (vector-set! vec i (vector-ref vec j))
        (vector-set! vec j temp)))
    
    ;; Helper function to convert list to vector (if needed)
    (define (ensure-vector obj)
      (if (list? obj)
        (list->vector obj)
        obj))
    
    ;; Helper function to convert vector to list (if needed)
    (define (ensure-list obj)
      (if (vector? obj)
        (vector->list obj)
        obj))
    
    ;; Helper function for three-way comparison
    (define (compare-3way comparator a b)
      (comparator-compare comparator a b))
    
    ;; ============= INSERTION SORT ALGORITHMS =============
    
    ;; Insertion sort for vectors (in-place)
    ;; Efficient for small arrays (n < 32)
    (define (vector-insertion-sort! comparator vec start end)
      (validate-vector-range vec start end)
      (let loop ((i (+ start 1)))
        (when (< i end)
          (let ((key (vector-ref vec i))
                (j i))
            ;; Shift elements greater than key to the right
            (let shift-loop ((k (- j 1)))
              (when (and (>= k start)
                        (> (compare-3way comparator (vector-ref vec k) key) 0))
                (vector-set! vec (+ k 1) (vector-ref vec k))
                (set! j k)
                (shift-loop (- k 1))))
            ;; Insert key at correct position
            (vector-set! vec j key)
            (loop (+ i 1))))))
    
    ;; Insertion sort for lists (returns new list)
    ;; More complex due to list structure, but still O(n²)
    (define (list-insertion-sort comparator lst)
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let loop ((sorted '()) (unsorted lst))
          (if (null? unsorted)
            sorted
            (loop (insert-sorted comparator (car unsorted) sorted)
                  (cdr unsorted))))))
    
    ;; Helper function to insert an element into a sorted list
    (define (insert-sorted comparator element sorted-list)
      (cond
        ((null? sorted-list) (list element))
        ((<= (compare-3way comparator element (car sorted-list)) 0)
         (cons element sorted-list))
        (else
         (cons (car sorted-list)
               (insert-sorted comparator element (cdr sorted-list))))))
    
    ;; ============= MERGE SORT ALGORITHMS =============
    
    ;; Stable merge sort for vectors (out-of-place)
    ;; Uses auxiliary arrays for stability and O(n log n) performance
    (define (vector-merge-sort! comparator vec start end)
      (validate-vector-range vec start end)
      (let ((len (- end start)))
        (when (> len 1)
          (if (<= len *insertion-sort-threshold*)
            ;; Use insertion sort for small arrays
            (vector-insertion-sort! comparator vec start end)
            ;; Use merge sort for larger arrays
            (let* ((mid (+ start (quotient len 2)))
                   (temp (make-vector len)))
              ;; Recursively sort both halves
              (vector-merge-sort! comparator vec start mid)
              (vector-merge-sort! comparator vec mid end)
              ;; Merge the sorted halves
              (vector-merge-ranges! comparator vec start mid end temp))))))
    
    ;; Helper function to merge two sorted ranges in a vector
    (define (vector-merge-ranges! comparator vec start mid end temp)
      ;; Copy to temporary array
      (vector-copy-range! temp 0 vec start end)
      (let ((left-start 0)
            (left-end (- mid start))
            (right-start (- mid start))
            (right-end (- end start)))
        ;; Merge back to original array
        (let loop ((i left-start) (j right-start) (k start))
          (cond
            ;; Left side exhausted
            ((>= i left-end)
             (when (< j right-end)
               (vector-set! vec k (vector-ref temp j))
               (loop i (+ j 1) (+ k 1))))
            ;; Right side exhausted
            ((>= j right-end)
             (vector-set! vec k (vector-ref temp i))
             (loop (+ i 1) j (+ k 1)))
            ;; Compare and merge
            (else
             (let ((cmp (compare-3way comparator 
                                     (vector-ref temp i)
                                     (vector-ref temp j))))
               (if (<= cmp 0) ; Stable: use <= for left side preference
                 (begin
                   (vector-set! vec k (vector-ref temp i))
                   (loop (+ i 1) j (+ k 1)))
                 (begin
                   (vector-set! vec k (vector-ref temp j))
                   (loop i (+ j 1) (+ k 1))))))))))
    
    ;; Merge sort for lists (returns new list)
    ;; Classic divide-and-conquer approach
    (define (list-merge-sort comparator lst)
      (define (length-and-split lst)
        ;; Returns (length . (first-half . second-half))
        (let ((len (length lst)))
          (if (<= len 1)
            (values len lst '())
            (let ((mid (quotient len 2)))
              (let split-loop ((remaining lst) (first-half '()) (count 0))
                (if (< count mid)
                  (split-loop (cdr remaining) 
                             (cons (car remaining) first-half) 
                             (+ count 1))
                  (values len (reverse first-half) remaining)))))))
      
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let-values (((len first-half second-half) (length-and-split lst)))
          (if (<= len *insertion-sort-threshold*)
            ;; Use insertion sort for small lists
            (list-insertion-sort comparator lst)
            ;; Use merge sort for larger lists
            (list-merge comparator
                       (list-merge-sort comparator first-half)
                       (list-merge-sort comparator second-half))))))
    
    ;; Core list merge operation (used by merge sort)
    (define (list-merge comparator lst1 lst2)
      (cond
        ((null? lst1) lst2)
        ((null? lst2) lst1)
        (else
         (let ((cmp (compare-3way comparator (car lst1) (car lst2))))
           (if (<= cmp 0) ; Stable: prefer first list on equality
             (cons (car lst1) (list-merge comparator (cdr lst1) lst2))
             (cons (car lst2) (list-merge comparator lst1 (cdr lst2)))))))
    
    ;; ============= QUICKSORT ALGORITHMS =============
    
    ;; Unstable quicksort for vectors (in-place)
    ;; Efficient average case O(n log n), worst case O(n²)
    (define (vector-quicksort! comparator vec start end)
      (validate-vector-range vec start end)
      (let ((len (- end start)))
        (when (> len 1)
          (if (<= len *quicksort-threshold*)
            ;; Use insertion sort for small arrays
            (vector-insertion-sort! comparator vec start end)
            ;; Use quicksort for larger arrays
            (let ((pivot-index (vector-partition! comparator vec start end)))
              (vector-quicksort! comparator vec start pivot-index)
              (vector-quicksort! comparator vec (+ pivot-index 1) end))))))
    
    ;; Partition function for quicksort (Lomuto partition scheme)
    ;; Returns the final position of the pivot
    (define (vector-partition! comparator vec start end)
      (let* ((pivot-index (vector-choose-pivot vec start end))
             (pivot-value (vector-ref vec pivot-index)))
        ;; Move pivot to end
        (vector-swap! vec pivot-index (- end 1))
        (let ((store-index start))
          ;; Partition around pivot
          (let loop ((i start))
            (when (< i (- end 1))
              (when (<= (compare-3way comparator (vector-ref vec i) pivot-value) 0)
                (vector-swap! vec i store-index)
                (set! store-index (+ store-index 1)))
              (loop (+ i 1))))
          ;; Move pivot to its final position
          (vector-swap! vec store-index (- end 1))
          store-index)))
    
    ;; Choose pivot for quicksort (median-of-three)
    (define (vector-choose-pivot vec start end)
      (let* ((len (- end start))
             (mid (+ start (quotient len 2))))
        (if (< len 3)
          start
          ;; Choose median of first, middle, last
          (let ((first (vector-ref vec start))
                (middle (vector-ref vec mid))
                (last (vector-ref vec (- end 1))))
            (cond
              ((<= first middle last) mid)
              ((<= first last middle) (- end 1))
              ((<= middle first last) start)
              ((<= middle last first) (- end 1))
              ((<= last first middle) start)
              (else mid))))))
    
    ;; Quicksort for lists (returns new list, not in-place)
    ;; Less efficient than vector version but still average O(n log n)
    (define (list-quicksort comparator lst)
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let* ((pivot (car lst))
               (rest (cdr lst)))
          (let partition-loop ((remaining rest) (less '()) (equal '()) (greater '()))
            (if (null? remaining)
              ;; Concatenate: quicksorted-less + pivot + equal + quicksorted-greater
              (append (list-quicksort comparator less)
                     (cons pivot equal)
                     (list-quicksort comparator greater))
              (let ((item (car remaining))
                    (tail (cdr remaining)))
                (let ((cmp (compare-3way comparator item pivot)))
                  (cond
                    ((< cmp 0) (partition-loop tail (cons item less) equal greater))
                    ((> cmp 0) (partition-loop tail less equal (cons item greater)))
                    (else (partition-loop tail less (cons item equal) greater)))))))))))
    
    ;; ============= ALGORITHM SELECTION LOGIC =============
    
    ;; Choose the best sorting algorithm for vectors based on requirements
    (define (vector-sort-algorithm! comparator vec start end stable?)
      (validate-vector-range vec start end)
      (let ((len (- end start)))
        (cond
          ;; Small arrays: always use insertion sort
          ((<= len *insertion-sort-threshold*)
           (vector-insertion-sort! comparator vec start end))
          ;; Large arrays: choose based on stability requirement
          (stable?
           (vector-merge-sort! comparator vec start end))
          (else
           (vector-quicksort! comparator vec start end)))))
    
    ;; Choose the best sorting algorithm for lists based on requirements  
    (define (list-sort-algorithm comparator lst stable?)
      (let ((len (length lst)))
        (cond
          ;; Small lists: use insertion sort
          ((<= len *insertion-sort-threshold*)
           (list-insertion-sort comparator lst))
          ;; Large lists: choose based on stability requirement
          (stable?
           (list-merge-sort comparator lst))
          (else
           (list-quicksort comparator lst)))))
    
    ;; ============= LIST SORTING PROCEDURES =============
    
    ;; Sort list using unstable algorithm (typically quicksort for large lists)
    (define (list-sort comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (list-sort-algorithm comparator lst #f))
    
    ;; Sort list using stable algorithm (always merge sort for large lists)
    (define (list-stable-sort comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (list-sort-algorithm comparator lst #t))
    
    ;; Destructively sort list (returns same list, modified)
    ;; Note: This is somewhat artificial for lists since we can't truly sort in-place
    ;; We implement it by converting to vector, sorting, and converting back
    (define (list-sort! comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let* ((vec (list->vector lst))
               (len (vector-length vec)))
          (vector-sort-algorithm! comparator vec 0 len #f)
          ;; Replace list contents with sorted values
          (let loop ((i 0) (current lst))
            (when (< i len)
              (set-car! current (vector-ref vec i))
              (loop (+ i 1) (cdr current))))
          lst)))
    
    ;; Destructively stable sort list
    (define (list-stable-sort! comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let* ((vec (list->vector lst))
               (len (vector-length vec)))
          (vector-sort-algorithm! comparator vec 0 len #t)
          ;; Replace list contents with sorted values
          (let loop ((i 0) (current lst))
            (when (< i len)
              (set-car! current (vector-ref vec i))
              (loop (+ i 1) (cdr current))))
          lst))
    
    ;; ============= VECTOR SORTING PROCEDURES =============
    
    ;; Sort vector using unstable algorithm (returns new vector)
    (define (vector-sort comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((result (vector-copy vec start end)))
          (vector-sort-algorithm! comparator result 0 (- end start) #f)
          result)))
    
    ;; Sort vector using stable algorithm (returns new vector)
    (define (vector-stable-sort comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((result (vector-copy vec start end)))
          (vector-sort-algorithm! comparator result 0 (- end start) #t)
          result)))
    
    ;; Destructively sort vector using unstable algorithm
    (define (vector-sort! comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (vector-sort-algorithm! comparator vec start end #f)
        vec))
    
    ;; Destructively sort vector using stable algorithm
    (define (vector-stable-sort! comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (vector-sort-algorithm! comparator vec start end #t)
        vec))
    
    ;; ============= MERGE PROCEDURES =============
    
    ;; Merge two sorted lists (already implemented above as part of merge sort)
    ;; This is the public interface version
    (define (list-merge comparator lst1 lst2)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (cond
        ((null? lst1) lst2)
        ((null? lst2) lst1)
        (else
         (let ((cmp (compare-3way comparator (car lst1) (car lst2))))
           (if (<= cmp 0) ; Stable: prefer first list on equality
             (cons (car lst1) (list-merge comparator (cdr lst1) lst2))
             (cons (car lst2) (list-merge comparator lst1 (cdr lst2))))))))
    
    ;; Destructively merge two sorted lists
    (define (list-merge! comparator lst1 lst2)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      ;; For simplicity, we implement this as non-destructive merge
      ;; A truly destructive version would reuse cons cells
      (list-merge comparator lst1 lst2))
    
    ;; Merge two sorted vectors (returns new vector)
    (define (vector-merge comparator vec1 vec2 . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start1 end1) (parse-vector-range vec1 args 0 (vector-length vec1)))
                   ((start2 end2) (if (>= (length args) 2)
                                    (parse-vector-range vec2 (cddr args) 0 (vector-length vec2))
                                    (values 0 (vector-length vec2)))))
        (let* ((len1 (- end1 start1))
               (len2 (- end2 start2))
               (result (make-vector (+ len1 len2))))
          (vector-merge-into! comparator result 0 
                             vec1 start1 end1 
                             vec2 start2 end2)
          result)))
    
    ;; Helper function to merge two vector ranges into a target vector
    (define (vector-merge-into! comparator target target-start 
                               vec1 start1 end1 
                               vec2 start2 end2)
      (let loop ((i start1) (j start2) (k target-start))
        (cond
          ;; First vector exhausted
          ((>= i end1)
           (when (< j end2)
             (vector-set! target k (vector-ref vec2 j))
             (loop i (+ j 1) (+ k 1))))
          ;; Second vector exhausted
          ((>= j end2)
           (vector-set! target k (vector-ref vec1 i))
           (loop (+ i 1) j (+ k 1)))
          ;; Compare and merge
          (else
           (let ((cmp (compare-3way comparator 
                                   (vector-ref vec1 i)
                                   (vector-ref vec2 j))))
             (if (<= cmp 0) ; Stable: prefer first vector on equality
               (begin
                 (vector-set! target k (vector-ref vec1 i))
                 (loop (+ i 1) j (+ k 1)))
               (begin
                 (vector-set! target k (vector-ref vec2 j))
                 (loop i (+ j 1) (+ k 1)))))))))
    
    ;; Destructively merge vectors (target receives merged result)
    (define (vector-merge! comparator target vec1 vec2 . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((target-start target-end) (parse-vector-range target args 0 (vector-length target)))
                   ((start1 end1) (if (>= (length args) 2)
                                    (parse-vector-range vec1 (cddr args) 0 (vector-length vec1))
                                    (values 0 (vector-length vec1))))
                   ((start2 end2) (if (>= (length args) 4)
                                    (parse-vector-range vec2 (cdddr (cdr args)) 0 (vector-length vec2))
                                    (values 0 (vector-length vec2)))))
        (vector-merge-into! comparator target target-start 
                           vec1 start1 end1 
                           vec2 start2 end2)
        target))
    
    ;; ============= SORTED? PREDICATE PROCEDURES =============
    
    ;; Check if list is sorted according to comparator
    (define (list-sorted? comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (or (null? lst)
          (null? (cdr lst))
          (let loop ((current (car lst)) (rest (cdr lst)))
            (if (null? rest)
              #t
              (let ((next (car rest)))
                (and (<= (compare-3way comparator current next) 0)
                     (loop next (cdr rest))))))))
    
    ;; Check if vector is sorted according to comparator
    (define (vector-sorted? comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (if (<= (- end start) 1)
          #t
          (let loop ((i start))
            (if (>= i (- end 1))
              #t
              (and (<= (compare-3way comparator 
                                    (vector-ref vec i) 
                                    (vector-ref vec (+ i 1))) 0)
                   (loop (+ i 1))))))))
    
    ;; ============= DUPLICATE DELETION PROCEDURES =============
    
    ;; Remove adjacent duplicates from list (returns new list)
    (define (list-delete-neighbor-dups comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (if (or (null? lst) (null? (cdr lst)))
        lst
        (let loop ((current (car lst)) (rest (cdr lst)) (result '()))
          (if (null? rest)
            (reverse (cons current result))
            (let ((next (car rest)))
              (if (= (compare-3way comparator current next) 0)
                ;; Skip duplicate
                (loop current (cdr rest) result)
                ;; Keep current, move to next
                (loop next (cdr rest) (cons current result))))))))
    
    ;; Destructively remove adjacent duplicates from list
    (define (list-delete-neighbor-dups! comparator lst)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      ;; For simplicity, implement as non-destructive
      ;; A truly destructive version would modify list structure in-place
      (list-delete-neighbor-dups comparator lst))
    
    ;; Remove adjacent duplicates from vector (returns new vector)
    (define (vector-delete-neighbor-dups comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((len (- end start)))
          (if (<= len 1)
            (vector-subrange vec start end)
            ;; First pass: count unique elements
            (let ((unique-count
                   (let loop ((i (+ start 1)) (count 1))
                     (if (>= i end)
                       count
                       (if (= (compare-3way comparator 
                                           (vector-ref vec (- i 1))
                                           (vector-ref vec i)) 0)
                         (loop (+ i 1) count)
                         (loop (+ i 1) (+ count 1)))))))
              ;; Second pass: build result vector
              (let ((result (make-vector unique-count)))
                (vector-set! result 0 (vector-ref vec start))
                (let loop ((i (+ start 1)) (j 1))
                  (if (>= i end)
                    result
                    (if (= (compare-3way comparator 
                                        (vector-ref vec (- i 1))
                                        (vector-ref vec i)) 0)
                      (loop (+ i 1) j)
                      (begin
                        (vector-set! result j (vector-ref vec i))
                        (loop (+ i 1) (+ j 1))))))))))))
    
    ;; Destructively remove adjacent duplicates from vector
    (define (vector-delete-neighbor-dups! comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((len (- end start)))
          (if (<= len 1)
            vec
            ;; Compact duplicates in-place
            (let ((write-pos start))
              (let loop ((read-pos (+ start 1)))
                (when (< read-pos end)
                  (unless (= (compare-3way comparator 
                                          (vector-ref vec (- read-pos 1))
                                          (vector-ref vec read-pos)) 0)
                    (set! write-pos (+ write-pos 1))
                    (vector-set! vec write-pos (vector-ref vec read-pos)))
                  (loop (+ read-pos 1))))
              ;; Return truncated vector (conceptually)
              ;; In practice, we can't resize vectors in Scheme
              vec)))))
    
    ;; ============= ADVANCED PROCEDURES =============
    
    ;; Find median element in vector (returns new sorted vector)
    (define (vector-find-median comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((len (- end start)))
          (if (<= len 0)
            (vector)
            (let ((sorted-vec (vector-subrange vec start end)))
              (vector-sort-algorithm! comparator sorted-vec 0 len #f)
              sorted-vec)))))
    
    ;; Find median element in vector (modifies vector in place)  
    (define (vector-find-median! comparator vec . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (vector-sort-algorithm! comparator vec start end #f)
        vec))
    
    ;; Select kth smallest element using quickselect algorithm
    (define (vector-select! comparator vec k . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((len (- end start)))
          (unless (and (>= k 0) (< k len))
            (error "k out of range" k len))
          (vector-quickselect! comparator vec start end k)
          (vector-ref vec (+ start k)))))
    
    ;; Quickselect algorithm: partially sort to find kth element
    (define (vector-quickselect! comparator vec start end k)
      (let loop ((lo start) (hi (- end 1)))
        (when (< lo hi)
          (let ((pivot-pos (vector-partition-select! comparator vec lo (+ hi 1))))
            (let ((pivot-index (- pivot-pos start)))
              (cond
                ((< k pivot-index)
                 ;; kth element is in left partition
                 (loop lo (- pivot-pos 1)))
                ((> k pivot-index)
                 ;; kth element is in right partition  
                 (loop (+ pivot-pos 1) hi))
                ;; else: kth element found at pivot position
                ))))))
    
    ;; Partition function for quickselect (similar to quicksort partition)
    (define (vector-partition-select! comparator vec start end)
      (let* ((pivot-index (vector-choose-pivot vec start end))
             (pivot-value (vector-ref vec pivot-index)))
        ;; Move pivot to end
        (vector-swap! vec pivot-index (- end 1))
        (let ((store-index start))
          ;; Partition around pivot
          (let loop ((i start))
            (when (< i (- end 1))
              (when (<= (compare-3way comparator (vector-ref vec i) pivot-value) 0)
                (vector-swap! vec i store-index)
                (set! store-index (+ store-index 1)))
              (loop (+ i 1))))
          ;; Move pivot to its final position
          (vector-swap! vec store-index (- end 1))
          store-index)))
    
    ;; Separate vector into elements < kth, = kth, > kth
    (define (vector-separate! comparator vec k . args)
      (unless (comparator-ordered? comparator)
        (error "comparator does not support ordering" comparator))
      (let-values (((start end) (parse-vector-range vec args 0 (vector-length vec))))
        (let ((len (- end start)))
          (unless (and (>= k 0) (< k len))
            (error "k out of range" k len))
          ;; First, find the kth element
          (let ((kth-element (vector-select! comparator vec k start end)))
            ;; Then, partition around that element using three-way partitioning
            (vector-three-way-partition! comparator vec start end kth-element)
            vec))))
    
    ;; Three-way partitioning: < pivot | = pivot | > pivot
    (define (vector-three-way-partition! comparator vec start end pivot-value)
      (let ((lt start)      ; boundary for < pivot
            (gt (- end 1))  ; boundary for > pivot  
            (i start))      ; current position
        (let loop ()
          (when (<= i gt)
            (let ((cmp (compare-3way comparator (vector-ref vec i) pivot-value)))
              (cond
                ((< cmp 0)
                 ;; Element < pivot: swap with lt boundary and advance both
                 (vector-swap! vec i lt)
                 (set! lt (+ lt 1))
                 (set! i (+ i 1))
                 (loop))
                ((> cmp 0)
                 ;; Element > pivot: swap with gt boundary, retreat gt
                 (vector-swap! vec i gt)
                 (set! gt (- gt 1))
                 (loop))  ; don't advance i, recheck swapped element
                (else
                 ;; Element = pivot: just advance i
                 (set! i (+ i 1))
                 (loop)))))))))
    
    )) ; end define-library