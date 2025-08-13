;; SRFI-128: Comparators
;;
;; This library provides a facility for creating and manipulating comparators.
;; Comparators bundle together type test, equality, comparison, and hash
;; functions to provide a uniform way to compare values.
;;
;; Reference: https://srfi.schemers.org/srfi-128/srfi-128.html

(define-library (srfi 128)
  (import (scheme base)
          (scheme case-lambda))
  
  (export
    ;; Constructor
    make-comparator
    
    ;; Standard comparators
    boolean-comparator
    char-comparator
    string-comparator
    string-ci-comparator
    symbol-comparator
    number-comparator
    default-comparator
    
    ;; Predicates
    comparator?
    comparator-ordered?
    comparator-hashable?
    
    ;; Comparison procedures
    comparator-test-type
    comparator-equal?
    comparator-compare
    comparator-hash
    
    ;; Derived comparison procedures
    =? <? >? <=? >=?
    
    ;; Utilities
    make-comparison
    make-hash-function
    comparator-register-default!
    )
  
  (begin
    
    ;; ============= COMPARATOR DATA STRUCTURE =============
    
    ;; A comparator is a record with four functions:
    ;; 1. type-test: predicate that returns #t for supported types
    ;; 2. equality: equality predicate
    ;; 3. comparison: comparison function (returns -1, 0, 1)
    ;; 4. hash: hash function (optional, returns integer)
    (define-record-type &lt;comparator&gt;
      (make-raw-comparator type-test equality comparison hash ordered? hashable?)
      comparator?
      (type-test comparator-type-test)
      (equality comparator-equality)
      (comparison comparator-comparison)
      (hash comparator-hash-function)
      (ordered? comparator-ordered?)
      (hashable? comparator-hashable?))
    
    ;; ============= CONSTRUCTOR =============
    
    ;; Main constructor for comparators
    (define make-comparator
      (case-lambda
        ;; Four functions: type-test, equality, comparison, hash
        ((type-test equality comparison hash)
         (make-raw-comparator type-test equality comparison hash #t #t))
        
        ;; Three functions: type-test, equality, comparison (no hash)
        ((type-test equality comparison)
         (make-raw-comparator type-test equality comparison #f #t #f))
        
        ;; Two functions: type-test, equality (no comparison or hash)
        ((type-test equality)
         (make-raw-comparator type-test equality #f #f #f #f))))
    
    ;; ============= BUILT-IN COMPARATORS =============
    
    ;; Boolean comparator
    (define boolean-comparator
      (make-comparator
        boolean?
        eq?
        (lambda (a b)
          (cond
            ((and a b) 0)
            ((and (not a) (not b)) 0)
            (a 1)
            (else -1)))
        (lambda (x)
          (if x 1 0))))
    
    ;; Character comparator
    (define char-comparator
      (make-comparator
        char?
        char=?
        (lambda (a b)
          (cond
            ((char&lt;? a b) -1)
            ((char&gt;? a b) 1)
            (else 0)))
        char-&gt;integer))
    
    ;; String comparator (case-sensitive)
    (define string-comparator
      (make-comparator
        string?
        string=?
        (lambda (a b)
          (cond
            ((string&lt;? a b) -1)
            ((string&gt;? a b) 1)
            (else 0)))
        string-hash))
    
    ;; Case-insensitive string comparator
    (define string-ci-comparator
      (make-comparator
        string?
        string-ci=?
        (lambda (a b)
          (cond
            ((string-ci&lt;? a b) -1)
            ((string-ci&gt;? a b) 1)
            (else 0)))
        string-ci-hash))
    
    ;; Symbol comparator
    (define symbol-comparator
      (make-comparator
        symbol?
        eq?
        (lambda (a b)
          (let ((sa (symbol-&gt;string a))
                (sb (symbol-&gt;string b)))
            (cond
              ((string&lt;? sa sb) -1)
              ((string&gt;? sa sb) 1)
              (else 0))))
        (lambda (x)
          (string-hash (symbol-&gt;string x)))))
    
    ;; Number comparator
    (define number-comparator
      (make-comparator
        number?
        =
        (lambda (a b)
          (cond
            ((&lt; a b) -1)
            ((&gt; a b) 1)
            (else 0)))
        exact-integer-sqrt)) ; Simple hash for numbers
    
    ;; Default comparator (handles multiple types)
    (define default-comparator
      (make-comparator
        (lambda (x) #t) ; Accepts any type
        default-equal?
        default-compare
        default-hash))
    
    ;; ============= COMPARISON PROCEDURES =============
    
    ;; Test if a value is of the type handled by the comparator
    (define (comparator-test-type comparator obj)
      ((comparator-type-test comparator) obj))
    
    ;; Test equality using the comparator
    (define (comparator-equal? comparator a b)
      ((comparator-equality comparator) a b))
    
    ;; Compare two values using the comparator
    (define (comparator-compare comparator a b)
      (let ((compare-fn (comparator-comparison comparator)))
        (if compare-fn
          (compare-fn a b)
          (error "comparator does not support comparison" comparator))))
    
    ;; Hash a value using the comparator
    (define (comparator-hash comparator obj)
      (let ((hash-fn (comparator-hash-function comparator)))
        (if hash-fn
          (hash-fn obj)
          (error "comparator does not support hashing" comparator))))
    
    ;; ============= DERIVED COMPARISON PROCEDURES =============
    
    ;; Equality test
    (define (=? comparator . objs)
      (if (null? objs)
        #t
        (let loop ((first (car objs)) (rest (cdr objs)))
          (if (null? rest)
            #t
            (and (comparator-equal? comparator first (car rest))
                 (loop (car rest) (cdr rest)))))))
    
    ;; Less than test
    (define (&lt;? comparator . objs)
      (if (null? objs)
        #t
        (let loop ((first (car objs)) (rest (cdr objs)))
          (if (null? rest)
            #t
            (and (&lt; (comparator-compare comparator first (car rest)) 0)
                 (loop (car rest) (cdr rest)))))))
    
    ;; Greater than test
    (define (&gt;? comparator . objs)
      (if (null? objs)
        #t
        (let loop ((first (car objs)) (rest (cdr objs)))
          (if (null? rest)
            #t
            (and (&gt; (comparator-compare comparator first (car rest)) 0)
                 (loop (car rest) (cdr rest)))))))
    
    ;; Less than or equal test
    (define (<=? comparator . objs)
      (if (null? objs)
        #t
        (let loop ((first (car objs)) (rest (cdr objs)))
          (if (null? rest)
            #t
            (and (<=  (comparator-compare comparator first (car rest)) 0)
                 (loop (car rest) (cdr rest)))))))
    
    ;; Greater than or equal test
    (define (>=? comparator . objs)
      (if (null? objs)
        #t
        (let loop ((first (car objs)) (rest (cdr objs)))
          (if (null? rest)
            #t
            (and (>= (comparator-compare comparator first (car rest)) 0)
                 (loop (car rest) (cdr rest)))))))
    
    ;; ============= HELPER FUNCTIONS =============
    
    ;; Default equality function (similar to equal?)
    (define (default-equal? a b)
      (cond
        ((and (boolean? a) (boolean? b)) (eq? a b))
        ((and (number? a) (number? b)) (= a b))
        ((and (char? a) (char? b)) (char=? a b))
        ((and (string? a) (string? b)) (string=? a b))
        ((and (symbol? a) (symbol? b)) (eq? a b))
        ((and (null? a) (null? b)) #t)
        ((and (pair? a) (pair? b))
         (and (default-equal? (car a) (car b))
              (default-equal? (cdr a) (cdr b))))
        ((and (vector? a) (vector? b))
         (let ((len-a (vector-length a))
               (len-b (vector-length b)))
           (and (= len-a len-b)
                (let loop ((i 0))
                  (if (= i len-a)
                    #t
                    (and (default-equal? (vector-ref a i) (vector-ref b i))
                         (loop (+ i 1))))))))
        (else (eqv? a b))))
    
    ;; Default comparison function
    (define (default-compare a b)
      (cond
        ;; Numbers
        ((and (number? a) (number? b))
         (cond
           ((&lt; a b) -1)
           ((&gt; a b) 1)
           (else 0)))
        
        ;; Characters
        ((and (char? a) (char? b))
         (cond
           ((char&lt;? a b) -1)
           ((char&gt;? a b) 1)
           (else 0)))
        
        ;; Strings
        ((and (string? a) (string? b))
         (cond
           ((string&lt;? a b) -1)
           ((string&gt;? a b) 1)
           (else 0)))
        
        ;; Symbols
        ((and (symbol? a) (symbol? b))
         (let ((sa (symbol-&gt;string a))
               (sb (symbol-&gt;string b)))
           (cond
             ((string&lt;? sa sb) -1)
             ((string&gt;? sa sb) 1)
             (else 0))))
        
        ;; Booleans
        ((and (boolean? a) (boolean? b))
         (cond
           ((and a b) 0)
           ((and (not a) (not b)) 0)
           (a 1)
           (else -1)))
        
        ;; Different types - order by type precedence
        (else
          (let ((type-a (value-type-order a))
                (type-b (value-type-order b)))
            (cond
              ((&lt; type-a type-b) -1)
              ((&gt; type-a type-b) 1)
              (else 0))))))
    
    ;; Type ordering for mixed-type comparison
    (define (value-type-order obj)
      (cond
        ((null? obj) 0)
        ((boolean? obj) 1)
        ((number? obj) 2)
        ((char? obj) 3)
        ((string? obj) 4)
        ((symbol? obj) 5)
        ((pair? obj) 6)
        ((vector? obj) 7)
        (else 8)))
    
    ;; Default hash function
    (define (default-hash obj)
      (cond
        ((boolean? obj) (if obj 1 0))
        ((number? obj) (modulo (exact (abs obj)) 1000000))
        ((char? obj) (char-&gt;integer obj))
        ((string? obj) (string-hash obj))
        ((symbol? obj) (string-hash (symbol-&gt;string obj)))
        ((null? obj) 0)
        ((pair? obj) (+ (default-hash (car obj)) (* 31 (default-hash (cdr obj)))))
        ((vector? obj)
         (let ((len (vector-length obj)))
           (if (= len 0)
             0
             (let loop ((i 0) (hash 0))
               (if (= i len)
                 hash
                 (loop (+ i 1) (+ hash (* 31 (default-hash (vector-ref obj i))))))))))
        (else 42))) ; Fallback for other types
    
    ;; String hash function (simple implementation)
    (define (string-hash str)
      (let ((len (string-length str)))
        (if (= len 0)
          0
          (let loop ((i 0) (hash 0))
            (if (= i len)
              (modulo hash 1000000)
              (loop (+ i 1) (+ hash (* 31 (char-&gt;integer (string-ref str i))))))))))
    
    ;; Case-insensitive string hash
    (define (string-ci-hash str)
      (string-hash (string-downcase str)))
    
    ;; ============= UTILITIES =============
    
    ;; Create a comparison function from a less-than predicate
    (define (make-comparison &lt;)
      (lambda (a b)
        (cond
          ((&lt; a b) -1)
          ((&lt; b a) 1)
          (else 0))))
    
    ;; Create a hash function (placeholder - could be more sophisticated)
    (define (make-hash-function type-test)
      (lambda (obj)
        (if (type-test obj)
          (default-hash obj)
          (error "object not supported by hash function" obj))))
    
    ;; Registry for default comparators (minimal implementation)
    (define *default-registry* '())
    
    (define (comparator-register-default! comparator)
      (set! *default-registry* (cons comparator *default-registry*)))
    
    ;; Register built-in comparators
    (comparator-register-default! boolean-comparator)
    (comparator-register-default! char-comparator)
    (comparator-register-default! string-comparator)
    (comparator-register-default! symbol-comparator)
    (comparator-register-default! number-comparator)
    
    )) ; end define-library