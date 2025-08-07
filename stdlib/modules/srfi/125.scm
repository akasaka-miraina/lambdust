;; SRFI-125: Intermediate hash tables
;; 
;; This library provides comprehensive hash table operations that extend
;; beyond basic hash table interfaces. It offers a rich set of convenience
;; procedures for creating, querying, updating, and manipulating hash tables
;; with various customization options for hash and equality functions.
;;
;; Reference: https://srfi.schemers.org/srfi-125/srfi-125.html

(define-library (srfi 125)
  (import (scheme base)
          (scheme case-lambda)
          (srfi 1))  ; List library for various operations
  
  (export
    ;; === Constructors ===
    make-hash-table hash-table hash-table-unfold alist->hash-table
    
    ;; === Predicates ===
    hash-table? hash-table-contains? hash-table-empty?
    hash-table=? hash-table-mutable?
    
    ;; === Accessors ===
    hash-table-ref hash-table-ref/default
    
    ;; === Mutators ===
    hash-table-set! hash-table-delete! hash-table-intern!
    hash-table-update! hash-table-update!/default hash-table-pop!
    hash-table-clear!
    
    ;; === The whole hash table ===
    hash-table-size hash-table-keys hash-table-values hash-table-entries
    hash-table-find hash-table-count
    
    ;; === Mapping and folding ===
    hash-table-map hash-table-for-each hash-table-map!
    hash-table-map->list hash-table-fold hash-table-prune!
    
    ;; === Copying and conversion ===
    hash-table-copy hash-table->alist
    
    ;; === Hash table merge ===
    hash-table-union! hash-table-intersection! hash-table-difference!
    hash-table-xor!
    
    ;; === Hash functions ===
    hash-table-equivalence-function hash-table-hash-function
    
    ;; === Deprecated names ===
    hash-table-walk hash-table-merge!)

  (begin
    ;; ============= INTERNAL REPRESENTATION =============
    
    ;; Hash table structure:
    ;; - vector of buckets (each bucket is a list of (key . value) pairs)
    ;; - current size (number of key-value pairs)
    ;; - capacity (number of buckets)
    ;; - hash function
    ;; - equality function
    ;; - mutable flag
    
    (define-record-type <hash-table>
      (%make-hash-table buckets size capacity hash-func equal-func mutable?)
      hash-table?
      (buckets hash-table-buckets set-hash-table-buckets!)
      (size hash-table-size set-hash-table-size!)
      (capacity hash-table-capacity set-hash-table-capacity!)
      (hash-func hash-table-hash-function)
      (equal-func hash-table-equivalence-function)
      (mutable? hash-table-mutable?))
    
    ;; Default hash function
    (define (default-hash-function obj)
      (cond
        ((number? obj) (abs (exact (floor obj))))
        ((string? obj) (string-hash obj))
        ((symbol? obj) (string-hash (symbol->string obj)))
        ((char? obj) (char->integer obj))
        ((boolean? obj) (if obj 1 0))
        ((null? obj) 0)
        ((pair? obj) (+ (default-hash-function (car obj))
                       (* 37 (default-hash-function (cdr obj)))))
        ((vector? obj) (vector-hash obj))
        (else (string-hash (object->string obj)))))
    
    ;; String hash function (simple polynomial hash)
    (define (string-hash str)
      (let ((len (string-length str)))
        (let loop ((i 0) (hash 0))
          (if (>= i len)
              (abs hash)
              (loop (+ i 1)
                    (+ (* hash 31) (char->integer (string-ref str i))))))))
    
    ;; Vector hash function
    (define (vector-hash vec)
      (let ((len (vector-length vec)))
        (let loop ((i 0) (hash 0))
          (if (>= i len)
              (abs hash)
              (loop (+ i 1)
                    (+ (* hash 31) (default-hash-function (vector-ref vec i))))))))
    
    ;; Object to string conversion for hashing
    (define (object->string obj)
      (call-with-output-string
        (lambda (port) (write obj port))))
    
    ;; ============= CONSTRUCTORS =============
    
    ;; Main constructor
    (define make-hash-table
      (case-lambda
        (() (make-hash-table equal? default-hash-function))
        ((equal-func) (make-hash-table equal-func default-hash-function))
        ((equal-func hash-func)
         (make-hash-table equal-func hash-func 16))
        ((equal-func hash-func initial-capacity)
         (let ((capacity (max 1 initial-capacity)))
           (%make-hash-table (make-vector capacity '())
                            0
                            capacity
                            hash-func
                            equal-func
                            #t)))))
    
    ;; Hash table literal constructor
    (define (hash-table equal-func hash-func . key-value-pairs)
      (let ((ht (make-hash-table equal-func hash-func)))
        (let loop ((pairs key-value-pairs))
          (cond
            ((null? pairs) ht)
            ((null? (cdr pairs)) (error "hash-table: odd number of arguments"))
            (else
             (hash-table-set! ht (car pairs) (cadr pairs))
             (loop (cddr pairs)))))))
    
    ;; Unfold constructor
    (define (hash-table-unfold stop? mapper successor seed equal-func hash-func)
      (let ((ht (make-hash-table equal-func hash-func)))
        (let loop ((seed seed))
          (if (stop? seed)
              ht
              (let-values (((key value) (mapper seed)))
                (hash-table-set! ht key value)
                (loop (successor seed)))))))
    
    ;; Convert alist to hash table
    (define alist->hash-table
      (case-lambda
        ((alist) (alist->hash-table alist equal? default-hash-function))
        ((alist equal-func) (alist->hash-table alist equal-func default-hash-function))
        ((alist equal-func hash-func)
         (let ((ht (make-hash-table equal-func hash-func)))
           (for-each (lambda (pair)
                      (hash-table-set! ht (car pair) (cdr pair)))
                    alist)
           ht))))
    
    ;; ============= INTERNAL UTILITIES =============
    
    ;; Get bucket index for key
    (define (get-bucket-index ht key)
      (modulo ((hash-table-hash-function ht) key)
              (hash-table-capacity ht)))
    
    ;; Find key in bucket
    (define (find-in-bucket bucket key equal-func)
      (let loop ((items bucket))
        (cond
          ((null? items) #f)
          ((equal-func key (caar items)) (car items))
          (else (loop (cdr items))))))
    
    ;; Remove key from bucket
    (define (remove-from-bucket bucket key equal-func)
      (let loop ((items bucket) (result '()))
        (cond
          ((null? items) (reverse result))
          ((equal-func key (caar items)) 
           (append (reverse result) (cdr items)))
          (else (loop (cdr items) (cons (car items) result))))))
    
    ;; Resize hash table when load factor exceeds 0.75
    (define (maybe-resize-hash-table! ht)
      (when (> (hash-table-size ht) (* 0.75 (hash-table-capacity ht)))
        (resize-hash-table! ht (* 2 (hash-table-capacity ht)))))
    
    ;; Resize hash table to new capacity
    (define (resize-hash-table! ht new-capacity)
      (let ((old-buckets (hash-table-buckets ht))
            (new-buckets (make-vector new-capacity '())))
        (set-hash-table-buckets! ht new-buckets)
        (set-hash-table-capacity! ht new-capacity)
        (set-hash-table-size! ht 0)
        ;; Rehash all existing entries
        (vector-for-each
          (lambda (bucket)
            (for-each (lambda (pair)
                       (hash-table-set! ht (car pair) (cdr pair)))
                     bucket))
          old-buckets)))
    
    ;; Vector for-each (if not available)
    (define (vector-for-each proc vec)
      (let ((len (vector-length vec)))
        (do ((i 0 (+ i 1)))
            ((= i len))
          (proc (vector-ref vec i)))))
    
    ;; ============= PREDICATES =============
    
    ;; Check if hash table contains key
    (define (hash-table-contains? ht key)
      (let* ((index (get-bucket-index ht key))
             (bucket (vector-ref (hash-table-buckets ht) index)))
        (and (find-in-bucket bucket key (hash-table-equivalence-function ht)) #t)))
    
    ;; Check if hash table is empty
    (define (hash-table-empty? ht)
      (= (hash-table-size ht) 0))
    
    ;; Compare hash tables for equality
    (define (hash-table=? value-equal-func ht1 ht2 . more-hts)
      (define (tables-equal? ht1 ht2)
        (and (= (hash-table-size ht1) (hash-table-size ht2))
             (hash-table-fold
               (lambda (key value1 acc)
                 (and acc
                      (hash-table-contains? ht2 key)
                      (value-equal-func value1 (hash-table-ref ht2 key))))
               #t
               ht1)))
      
      (let loop ((hts (cons ht2 more-hts)))
        (or (null? hts)
            (and (tables-equal? ht1 (car hts))
                 (loop (cdr hts))))))
    
    ;; ============= ACCESSORS =============
    
    ;; Get value for key, with optional default or failure thunk
    (define hash-table-ref
      (case-lambda
        ((ht key) 
         (hash-table-ref ht key #f (lambda () (error "hash-table-ref: key not found" key))))
        ((ht key default)
         (hash-table-ref ht key default (lambda () default)))
        ((ht key default failure)
         (let* ((index (get-bucket-index ht key))
                (bucket (vector-ref (hash-table-buckets ht) index))
                (entry (find-in-bucket bucket key (hash-table-equivalence-function ht))))
           (if entry
               (cdr entry)
               (if (procedure? failure)
                   (failure)
                   failure))))))
    
    ;; Get value with default
    (define (hash-table-ref/default ht key default)
      (hash-table-ref ht key default))
    
    ;; ============= MUTATORS =============
    
    ;; Set key-value pair
    (define (hash-table-set! ht . key-value-pairs)
      (unless (hash-table-mutable? ht)
        (error "hash-table-set!: hash table is immutable"))
      (let loop ((pairs key-value-pairs))
        (cond
          ((null? pairs) (void))
          ((null? (cdr pairs)) (error "hash-table-set!: odd number of arguments"))
          (else
           (let* ((key (car pairs))
                  (value (cadr pairs))
                  (index (get-bucket-index ht key))
                  (buckets (hash-table-buckets ht))
                  (bucket (vector-ref buckets index))
                  (entry (find-in-bucket bucket key (hash-table-equivalence-function ht))))
             (if entry
                 (set-cdr! entry value)  ; Update existing
                 (begin
                   (vector-set! buckets index (cons (cons key value) bucket))
                   (set-hash-table-size! ht (+ (hash-table-size ht) 1))
                   (maybe-resize-hash-table! ht))))
           (loop (cddr pairs))))))
    
    ;; Delete key
    (define (hash-table-delete! ht . keys)
      (unless (hash-table-mutable? ht)
        (error "hash-table-delete!: hash table is immutable"))
      (let ((count 0))
        (for-each
          (lambda (key)
            (let* ((index (get-bucket-index ht key))
                   (buckets (hash-table-buckets ht))
                   (bucket (vector-ref buckets index))
                   (entry (find-in-bucket bucket key (hash-table-equivalence-function ht))))
              (when entry
                (vector-set! buckets index 
                           (remove-from-bucket bucket key (hash-table-equivalence-function ht)))
                (set-hash-table-size! ht (- (hash-table-size ht) 1))
                (set! count (+ count 1)))))
          keys)
        count))
    
    ;; Intern key with default value
    (define (hash-table-intern! ht key failure)
      (unless (hash-table-mutable? ht)
        (error "hash-table-intern!: hash table is immutable"))
      (if (hash-table-contains? ht key)
          (hash-table-ref ht key)
          (let ((value (if (procedure? failure) (failure) failure)))
            (hash-table-set! ht key value)
            value)))
    
    ;; Update value for key
    (define (hash-table-update! ht key updater . maybe-default-failure)
      (unless (hash-table-mutable? ht)
        (error "hash-table-update!: hash table is immutable"))
      (let ((default (if (null? maybe-default-failure) #f (car maybe-default-failure)))
            (failure (if (or (null? maybe-default-failure) (null? (cdr maybe-default-failure)))
                        (lambda () (error "hash-table-update!: key not found" key))
                        (cadr maybe-default-failure))))
        (let ((current (hash-table-ref ht key default failure)))
          (hash-table-set! ht key (updater current)))))
    
    ;; Update with default
    (define (hash-table-update!/default ht key updater default)
      (hash-table-update! ht key updater default))
    
    ;; Pop key-value pair
    (define (hash-table-pop! ht key . maybe-default-failure)
      (unless (hash-table-mutable? ht)
        (error "hash-table-pop!: hash table is immutable"))
      (let ((value (apply hash-table-ref ht key maybe-default-failure)))
        (hash-table-delete! ht key)
        value))
    
    ;; Clear all entries
    (define (hash-table-clear! ht)
      (unless (hash-table-mutable? ht)
        (error "hash-table-clear!: hash table is immutable"))
      (let ((capacity (hash-table-capacity ht)))
        (set-hash-table-buckets! ht (make-vector capacity '()))
        (set-hash-table-size! ht 0)))
    
    ;; ============= THE WHOLE HASH TABLE =============
    
    ;; Get all keys
    (define (hash-table-keys ht)
      (hash-table-fold (lambda (key value keys) (cons key keys)) '() ht))
    
    ;; Get all values  
    (define (hash-table-values ht)
      (hash-table-fold (lambda (key value values) (cons value values)) '() ht))
    
    ;; Get all entries as (key . value) pairs
    (define (hash-table-entries ht)
      (hash-table-fold (lambda (key value entries) 
                        (cons (cons key value) entries)) '() ht))
    
    ;; Find entry satisfying predicate
    (define (hash-table-find ht proc)
      (call-with-current-continuation
        (lambda (return)
          (hash-table-for-each
            (lambda (key value)
              (when (proc key value)
                (return key value)))
            ht)
          (values #f #f))))
    
    ;; Count entries satisfying predicate
    (define (hash-table-count ht pred)
      (hash-table-fold
        (lambda (key value count)
          (if (pred key value) (+ count 1) count))
        0
        ht))
    
    ;; ============= MAPPING AND FOLDING =============
    
    ;; Map to new hash table
    (define (hash-table-map proc equal-func hash-func ht)
      (let ((result (make-hash-table equal-func hash-func)))
        (hash-table-for-each
          (lambda (key value)
            (let-values (((new-key new-value) (proc key value)))
              (hash-table-set! result new-key new-value)))
          ht)
        result))
    
    ;; For each entry
    (define (hash-table-for-each proc ht)
      (vector-for-each
        (lambda (bucket)
          (for-each (lambda (pair) (proc (car pair) (cdr pair))) bucket))
        (hash-table-buckets ht)))
    
    ;; Map in place (deprecated, use hash-table-map)
    (define (hash-table-map! proc ht)
      (unless (hash-table-mutable? ht)
        (error "hash-table-map!: hash table is immutable"))
      (hash-table-for-each
        (lambda (key value)
          (hash-table-set! ht key (proc key value)))
        ht))
    
    ;; Map to list
    (define (hash-table-map->list proc ht)
      (hash-table-fold
        (lambda (key value acc) (cons (proc key value) acc))
        '()
        ht))
    
    ;; Fold over hash table
    (define (hash-table-fold proc init ht)
      (let ((result init))
        (hash-table-for-each
          (lambda (key value)
            (set! result (proc key value result)))
          ht)
        result))
    
    ;; Remove entries not satisfying predicate
    (define (hash-table-prune! proc ht)
      (unless (hash-table-mutable? ht)
        (error "hash-table-prune!: hash table is immutable"))
      (let ((keys-to-delete
             (hash-table-fold
               (lambda (key value acc)
                 (if (proc key value) acc (cons key acc)))
               '()
               ht)))
        (for-each (lambda (key) (hash-table-delete! ht key)) keys-to-delete)
        ht))
    
    ;; ============= COPYING AND CONVERSION =============
    
    ;; Copy hash table
    (define hash-table-copy
      (case-lambda
        ((ht) (hash-table-copy ht #f))
        ((ht mutable?)
         (let ((new-ht (%make-hash-table (vector-copy (hash-table-buckets ht))
                                        (hash-table-size ht)
                                        (hash-table-capacity ht)
                                        (hash-table-hash-function ht)
                                        (hash-table-equivalence-function ht)
                                        mutable?)))
           ;; Deep copy the bucket contents
           (let ((buckets (hash-table-buckets new-ht)))
             (do ((i 0 (+ i 1)))
                 ((= i (vector-length buckets)))
               (vector-set! buckets i (map (lambda (pair) (cons (car pair) (cdr pair)))
                                          (vector-ref buckets i)))))
           new-ht))))
    
    ;; Vector copy (if not available)
    (define (vector-copy vec)
      (let* ((len (vector-length vec))
             (new-vec (make-vector len)))
        (do ((i 0 (+ i 1)))
            ((= i len) new-vec)
          (vector-set! new-vec i (vector-ref vec i)))))
    
    ;; Convert to association list
    (define (hash-table->alist ht)
      (hash-table-fold
        (lambda (key value acc) (cons (cons key value) acc))
        '()
        ht))
    
    ;; ============= HASH TABLE MERGE =============
    
    ;; Union (destructive)
    (define (hash-table-union! ht1 ht2)
      (unless (hash-table-mutable? ht1)
        (error "hash-table-union!: first hash table is immutable"))
      (hash-table-for-each
        (lambda (key value)
          (unless (hash-table-contains? ht1 key)
            (hash-table-set! ht1 key value)))
        ht2)
      ht1)
    
    ;; Intersection (destructive)
    (define (hash-table-intersection! ht1 ht2)
      (unless (hash-table-mutable? ht1)
        (error "hash-table-intersection!: first hash table is immutable"))
      (let ((keys-to-keep
             (hash-table-fold
               (lambda (key value acc)
                 (if (hash-table-contains? ht2 key) (cons key acc) acc))
               '()
               ht1)))
        (hash-table-clear! ht1)
        (for-each (lambda (key)
                   (hash-table-set! ht1 key (hash-table-ref ht2 key)))
                 keys-to-keep))
      ht1)
    
    ;; Difference (destructive)
    (define (hash-table-difference! ht1 ht2)
      (unless (hash-table-mutable? ht1)
        (error "hash-table-difference!: first hash table is immutable"))
      (hash-table-for-each
        (lambda (key value)
          (when (hash-table-contains? ht1 key)
            (hash-table-delete! ht1 key)))
        ht2)
      ht1)
    
    ;; Exclusive or (destructive)
    (define (hash-table-xor! ht1 ht2)
      (unless (hash-table-mutable? ht1)
        (error "hash-table-xor!: first hash table is immutable"))
      (let ((keys-in-both '()))
        ;; Find keys in both tables
        (hash-table-for-each
          (lambda (key value)
            (when (hash-table-contains? ht1 key)
              (set! keys-in-both (cons key keys-in-both))))
          ht2)
        ;; Remove keys that are in both
        (for-each (lambda (key) (hash-table-delete! ht1 key)) keys-in-both)
        ;; Add keys that are only in ht2
        (hash-table-for-each
          (lambda (key value)
            (unless (member key keys-in-both)
              (hash-table-set! ht1 key value)))
          ht2))
      ht1)
    
    ;; ============= DEPRECATED NAMES =============
    
    (define hash-table-walk hash-table-for-each)
    (define (hash-table-merge! ht1 ht2)
      (hash-table-union! ht1 ht2))
    
    ;; ============= EXAMPLES AND USAGE PATTERNS =============
    
    ;; Example: Word frequency counter
    ;; (define word-counts (make-hash-table string=? string-hash))
    ;; (define (count-word! word)
    ;;   (hash-table-update!/default word-counts word (lambda (n) (+ n 1)) 0))
    
    ;; Example: Memoization cache
    ;; (define (make-memoized-function f)
    ;;   (let ((cache (make-hash-table equal? default-hash-function)))
    ;;     (lambda args
    ;;       (hash-table-intern! cache args (lambda () (apply f args))))))
    
    ;; Example: Multi-map (multiple values per key)
    ;; (define (multi-hash-table-add! ht key value)
    ;;   (hash-table-update!/default ht key (lambda (lst) (cons value lst)) '()))
    
    ;; Example: Index of objects by property
    ;; (define (make-property-index objects property-proc)
    ;;   (let ((index (make-hash-table equal? default-hash-function)))
    ;;     (for-each (lambda (obj)
    ;;                (multi-hash-table-add! index (property-proc obj) obj))
    ;;              objects)
    ;;     index))
    ))