;; SRFI-13: String Library
;; 
;; This library provides comprehensive string processing procedures that extend
;; the basic string operations of R7RS. It includes string comparison, searching,
;; case conversion, and various string manipulation utilities.
;;
;; Reference: https://srfi.schemers.org/srfi-13/srfi-13.html

(define-library (srfi 13)
  (import (scheme base)
          (scheme char)
          (scheme case-lambda))
  
  (export
    ;; === Predicates ===
    string-null? string-every string-any
    
    ;; === Constructors ===
    make-string string string-tabulate
    
    ;; === List & string conversion ===
    string->list list->string reverse-list->string
    string-join
    
    ;; === Selection ===
    string-length string-ref string-copy substring/shared
    string-copy! string-take string-drop 
    string-take-right string-drop-right
    string-pad string-pad-right string-trim string-trim-right string-trim-both
    
    ;; === Modification ===
    string-set! string-fill!
    
    ;; === Comparison ===
    string-compare string-compare-ci
    string= string<> string< string> string<= string>=
    string-ci= string-ci<> string-ci< string-ci> string-ci<= string-ci>=
    string-hash string-hash-ci
    
    ;; === Searching ===
    string-prefix-length string-suffix-length
    string-prefix? string-suffix?
    string-index string-index-right
    string-skip string-skip-right
    string-count
    string-contains string-contains-ci
    
    ;; === Alphabetic case mapping ===
    string-upcase string-downcase string-titlecase
    string-upcase! string-downcase! string-titlecase!
    
    ;; === Reverse & append ===
    string-reverse string-reverse!
    string-append string-concatenate string-concatenate-reverse
    string-concatenate/shared
    
    ;; === Fold, unfold & map ===
    string-map string-map! string-fold string-fold-right
    string-unfold string-unfold-right
    string-for-each string-for-each-index
    
    ;; === Replicate & rotate ===
    xsubstring string-xcopy!
    
    ;; === Miscellaneous ===
    string-tokenize string-filter string-delete)

  (begin
    ;; ============= PREDICATES =============
    
    (define (string-null? s)
      (= (string-length s) 0))
    
    (define string-every
      (case-lambda
        ((pred s) (string-every pred s 0 (string-length s)))
        ((pred s start) (string-every pred s start (string-length s)))
        ((pred s start end)
         (let loop ((i start))
           (cond
             ((>= i end) #t)
             ((pred (string-ref s i)) (loop (+ i 1)))
             (else #f))))))
    
    (define string-any
      (case-lambda
        ((pred s) (string-any pred s 0 (string-length s)))
        ((pred s start) (string-any pred s start (string-length s)))
        ((pred s start end)
         (let loop ((i start))
           (cond
             ((>= i end) #f)
             ((pred (string-ref s i)) #t)
             (else (loop (+ i 1))))))))
    
    ;; ============= CONSTRUCTORS =============
    
    (define string-tabulate
      (case-lambda
        ((len proc) (string-tabulate len proc))
        ((len proc)
         (let ((result (make-string len)))
           (do ((i 0 (+ i 1)))
               ((= i len) result)
             (string-set! result i (proc i)))))))
    
    ;; ============= LIST & STRING CONVERSION =============
    
    (define reverse-list->string
      (lambda (char-list)
        (list->string (reverse char-list))))
    
    (define string-join
      (case-lambda
        ((string-list) (string-join string-list " "))
        ((string-list delimiter) (string-join string-list delimiter 'infix))
        ((string-list delimiter grammar)
         (case grammar
           ((infix)
            (if (null? string-list)
                ""
                (let loop ((strings (cdr string-list))
                          (result (car string-list)))
                  (if (null? strings)
                      result
                      (loop (cdr strings)
                            (string-append result delimiter (car strings)))))))
           ((strict-infix)
            (if (null? string-list)
                (error "strict-infix requires non-empty list")
                (string-join string-list delimiter 'infix)))
           ((suffix)
            (let loop ((strings string-list) (result ""))
              (if (null? strings)
                  result
                  (loop (cdr strings)
                        (string-append result (car strings) delimiter)))))
           ((prefix)
            (let loop ((strings string-list) (result ""))
              (if (null? strings)
                  result
                  (loop (cdr strings)
                        (string-append result delimiter (car strings))))))
           (else (error "Invalid grammar" grammar))))))
    
    ;; ============= SELECTION =============
    
    (define substring/shared substring)  ; Simplified - no sharing optimization
    
    (define string-copy!
      (case-lambda
        ((to tstart from) (string-copy! to tstart from 0 (string-length from)))
        ((to tstart from fstart) (string-copy! to tstart from fstart (string-length from)))
        ((to tstart from fstart fend)
         (do ((i fstart (+ i 1))
              (j tstart (+ j 1)))
             ((>= i fend))
           (string-set! to j (string-ref from i))))))
    
    (define string-take
      (case-lambda
        ((s n) (substring s 0 n))))
    
    (define string-drop
      (case-lambda
        ((s n) (substring s n (string-length s)))))
    
    (define string-take-right
      (case-lambda
        ((s n) 
         (let ((len (string-length s)))
           (substring s (- len n) len)))))
    
    (define string-drop-right
      (case-lambda
        ((s n)
         (let ((len (string-length s)))
           (substring s 0 (- len n))))))
    
    (define string-pad
      (case-lambda
        ((s len) (string-pad s len #\space))
        ((s len char) (string-pad s len char 0 (string-length s)))
        ((s len char start) (string-pad s len char start (string-length s)))
        ((s len char start end)
         (let* ((slen (- end start))
                (result (make-string len char)))
           (if (< slen len)
               (string-copy! result (- len slen) s start end)
               (string-copy! result 0 s (- end len) end))
           result))))
    
    (define string-pad-right
      (case-lambda
        ((s len) (string-pad-right s len #\space))
        ((s len char) (string-pad-right s len char 0 (string-length s)))
        ((s len char start) (string-pad-right s len char start (string-length s)))
        ((s len char start end)
         (let* ((slen (- end start))
                (result (make-string len char)))
           (string-copy! result 0 s start (min end (+ start len)))
           result))))
    
    (define string-trim
      (case-lambda
        ((s) (string-trim s char-whitespace?))
        ((s pred) (string-trim s pred 0 (string-length s)))
        ((s pred start) (string-trim s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i start))
           (cond
             ((>= i end) "")
             ((pred (string-ref s i)) (loop (+ i 1)))
             (else (substring s i end)))))))
    
    (define string-trim-right
      (case-lambda
        ((s) (string-trim-right s char-whitespace?))
        ((s pred) (string-trim-right s pred 0 (string-length s)))
        ((s pred start) (string-trim-right s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i (- end 1)))
           (cond
             ((< i start) "")
             ((pred (string-ref s i)) (loop (- i 1)))
             (else (substring s start (+ i 1))))))))
    
    (define string-trim-both
      (case-lambda
        ((s) (string-trim-both s char-whitespace?))
        ((s pred) (string-trim-both s pred 0 (string-length s)))
        ((s pred start) (string-trim-both s pred start (string-length s)))
        ((s pred start end)
         (string-trim-right (string-trim s pred start end) pred))))
    
    ;; ============= COMPARISON =============
    
    (define (string-compare s1 s2 proc< proc= proc>)
      (let ((len1 (string-length s1))
            (len2 (string-length s2)))
        (let loop ((i 0))
          (cond
            ((and (= i len1) (= i len2)) (proc= i))
            ((= i len1) (proc< i))
            ((= i len2) (proc> i))
            (else
             (let ((c1 (string-ref s1 i))
                   (c2 (string-ref s2 i)))
               (cond
                 ((char<? c1 c2) (proc< i))
                 ((char>? c1 c2) (proc> i))
                 (else (loop (+ i 1))))))))))
    
    (define (string-compare-ci s1 s2 proc< proc= proc>)
      (string-compare (string-downcase s1) (string-downcase s2) proc< proc= proc>))
    
    (define (string<> s1 s2) (not (string= s1 s2)))
    (define (string-ci<> s1 s2) (not (string-ci= s1 s2)))
    
    (define (string-hash s . maybe-bound)
      (let ((bound (if (null? maybe-bound) 268435456 (car maybe-bound))))
        (let loop ((i (- (string-length s) 1)) (hash 0))
          (if (< i 0)
              (modulo hash bound)
              (loop (- i 1)
                    (+ (* hash 37) (char->integer (string-ref s i))))))))
    
    (define (string-hash-ci s . maybe-bound)
      (apply string-hash (string-downcase s) maybe-bound))
    
    ;; ============= SEARCHING =============
    
    (define string-prefix-length
      (case-lambda
        ((s1 s2) (string-prefix-length s1 s2 0 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1) (string-prefix-length s1 s2 start1 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1 end1) (string-prefix-length s1 s2 start1 end1 0 (string-length s2)))
        ((s1 s2 start1 end1 start2) (string-prefix-length s1 s2 start1 end1 start2 (string-length s2)))
        ((s1 s2 start1 end1 start2 end2)
         (let loop ((i1 start1) (i2 start2) (count 0))
           (cond
             ((or (>= i1 end1) (>= i2 end2)) count)
             ((char=? (string-ref s1 i1) (string-ref s2 i2))
              (loop (+ i1 1) (+ i2 1) (+ count 1)))
             (else count))))))
    
    (define string-suffix-length
      (case-lambda
        ((s1 s2) (string-suffix-length s1 s2 0 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1) (string-suffix-length s1 s2 start1 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1 end1) (string-suffix-length s1 s2 start1 end1 0 (string-length s2)))
        ((s1 s2 start1 end1 start2) (string-suffix-length s1 s2 start1 end1 start2 (string-length s2)))
        ((s1 s2 start1 end1 start2 end2)
         (let loop ((i1 (- end1 1)) (i2 (- end2 1)) (count 0))
           (cond
             ((or (< i1 start1) (< i2 start2)) count)
             ((char=? (string-ref s1 i1) (string-ref s2 i2))
              (loop (- i1 1) (- i2 1) (+ count 1)))
             (else count))))))
    
    (define string-prefix?
      (case-lambda
        ((s1 s2) (string-prefix? s1 s2 0 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1) (string-prefix? s1 s2 start1 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1 end1) (string-prefix? s1 s2 start1 end1 0 (string-length s2)))
        ((s1 s2 start1 end1 start2) (string-prefix? s1 s2 start1 end1 start2 (string-length s2)))
        ((s1 s2 start1 end1 start2 end2)
         (let ((len1 (- end1 start1))
               (len2 (- end2 start2)))
           (and (<= len1 len2)
                (= (string-prefix-length s1 s2 start1 end1 start2 end2) len1))))))
    
    (define string-suffix?
      (case-lambda
        ((s1 s2) (string-suffix? s1 s2 0 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1) (string-suffix? s1 s2 start1 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1 end1) (string-suffix? s1 s2 start1 end1 0 (string-length s2)))
        ((s1 s2 start1 end1 start2) (string-suffix? s1 s2 start1 end1 start2 (string-length s2)))
        ((s1 s2 start1 end1 start2 end2)
         (let ((len1 (- end1 start1))
               (len2 (- end2 start2)))
           (and (<= len1 len2)
                (= (string-suffix-length s1 s2 start1 end1 start2 end2) len1))))))
    
    (define string-index
      (case-lambda
        ((s pred) (string-index s pred 0 (string-length s)))
        ((s pred start) (string-index s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i start))
           (cond
             ((>= i end) #f)
             ((if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i))) i)
             (else (loop (+ i 1))))))))
    
    (define string-index-right
      (case-lambda
        ((s pred) (string-index-right s pred 0 (string-length s)))
        ((s pred start) (string-index-right s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i (- end 1)))
           (cond
             ((< i start) #f)
             ((if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i))) i)
             (else (loop (- i 1))))))))
    
    (define string-skip
      (case-lambda
        ((s pred) (string-skip s pred 0 (string-length s)))
        ((s pred start) (string-skip s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i start))
           (cond
             ((>= i end) #f)
             ((not (if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i)))) i)
             (else (loop (+ i 1))))))))
    
    (define string-skip-right
      (case-lambda
        ((s pred) (string-skip-right s pred 0 (string-length s)))
        ((s pred start) (string-skip-right s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i (- end 1)))
           (cond
             ((< i start) #f)
             ((not (if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i)))) i)
             (else (loop (- i 1))))))))
    
    (define string-count
      (case-lambda
        ((s pred) (string-count s pred 0 (string-length s)))
        ((s pred start) (string-count s pred start (string-length s)))
        ((s pred start end)
         (let loop ((i start) (count 0))
           (cond
             ((>= i end) count)
             ((if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i)))
              (loop (+ i 1) (+ count 1)))
             (else (loop (+ i 1) count)))))))
    
    (define string-contains
      (case-lambda
        ((s1 s2) (string-contains s1 s2 0 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1) (string-contains s1 s2 start1 (string-length s1) 0 (string-length s2)))
        ((s1 s2 start1 end1) (string-contains s1 s2 start1 end1 0 (string-length s2)))
        ((s1 s2 start1 end1 start2) (string-contains s1 s2 start1 end1 start2 (string-length s2)))
        ((s1 s2 start1 end1 start2 end2)
         (let ((len2 (- end2 start2)))
           (let loop ((i start1))
             (cond
               ((> (+ i len2) end1) #f)
               ((string-prefix? s2 s1 start2 end2 i (+ i len2)) i)
               (else (loop (+ i 1)))))))))
    
    (define string-contains-ci
      (case-lambda
        ((s1 s2) (string-contains (string-downcase s1) (string-downcase s2)))
        ((s1 s2 start1) (string-contains (string-downcase s1) (string-downcase s2) start1))
        ((s1 s2 start1 end1) (string-contains (string-downcase s1) (string-downcase s2) start1 end1))
        ((s1 s2 start1 end1 start2) (string-contains (string-downcase s1) (string-downcase s2) start1 end1 start2))
        ((s1 s2 start1 end1 start2 end2) (string-contains (string-downcase s1) (string-downcase s2) start1 end1 start2 end2))))
    
    ;; ============= ALPHABETIC CASE MAPPING =============
    
    (define string-upcase!
      (lambda (s)
        (do ((i 0 (+ i 1)))
            ((= i (string-length s)) s)
          (string-set! s i (char-upcase (string-ref s i))))))
    
    (define string-downcase!
      (lambda (s)
        (do ((i 0 (+ i 1)))
            ((= i (string-length s)) s)
          (string-set! s i (char-downcase (string-ref s i))))))
    
    (define string-titlecase!
      (lambda (s)
        (let ((len (string-length s)))
          (when (> len 0)
            (string-set! s 0 (char-upcase (string-ref s 0)))
            (do ((i 1 (+ i 1)))
                ((= i len) s)
              (let ((ch (string-ref s i)))
                (string-set! s i
                            (if (char-whitespace? (string-ref s (- i 1)))
                                (char-upcase ch)
                                (char-downcase ch)))))))))
    
    ;; ============= REVERSE & APPEND =============
    
    (define (string-reverse s)
      (let* ((len (string-length s))
             (result (make-string len)))
        (do ((i 0 (+ i 1)))
            ((= i len) result)
          (string-set! result i (string-ref s (- len i 1))))))
    
    (define (string-reverse! s)
      (let ((len (string-length s)))
        (do ((i 0 (+ i 1))
             (j (- len 1) (- j 1)))
            ((>= i j) s)
          (let ((temp (string-ref s i)))
            (string-set! s i (string-ref s j))
            (string-set! s j temp)))))
    
    (define (string-concatenate string-list)
      (apply string-append string-list))
    
    (define (string-concatenate-reverse string-list . maybe-final-string)
      (let ((final (if (null? maybe-final-string) "" (car maybe-final-string))))
        (string-concatenate (append (reverse string-list) (list final)))))
    
    (define string-concatenate/shared string-concatenate)  ; No sharing optimization
    
    ;; ============= FOLD, UNFOLD & MAP =============
    
    (define string-map
      (case-lambda
        ((proc s) (list->string (map proc (string->list s))))
        ((proc s1 s2 . strings)
         (let ((char-lists (map string->list (cons s1 (cons s2 strings)))))
           (list->string (apply map proc char-lists))))))
    
    (define string-map!
      (case-lambda
        ((proc s)
         (do ((i 0 (+ i 1)))
             ((= i (string-length s)) s)
           (string-set! s i (proc (string-ref s i)))))
        ((proc s1 s2 . strings)
         (let ((all-strings (cons s1 (cons s2 strings)))
               (min-len (apply min (map string-length (cons s1 (cons s2 strings))))))
           (do ((i 0 (+ i 1)))
               ((= i min-len) s1)
             (string-set! s1 i (apply proc (map (lambda (s) (string-ref s i)) all-strings))))))))
    
    (define string-fold
      (case-lambda
        ((kons knil s) (string-fold kons knil s 0 (string-length s)))
        ((kons knil s start) (string-fold kons knil s start (string-length s)))
        ((kons knil s start end)
         (let loop ((i start) (acc knil))
           (if (>= i end)
               acc
               (loop (+ i 1) (kons (string-ref s i) acc)))))))
    
    (define string-fold-right
      (case-lambda
        ((kons knil s) (string-fold-right kons knil s 0 (string-length s)))
        ((kons knil s start) (string-fold-right kons knil s start (string-length s)))
        ((kons knil s start end)
         (let loop ((i (- end 1)) (acc knil))
           (if (< i start)
               acc
               (loop (- i 1) (kons (string-ref s i) acc)))))))
    
    (define (string-unfold p f g seed . maybe-base-string)
      (let ((base (if (null? maybe-base-string) "" (car maybe-base-string))))
        (string-append base
                       (list->string
                        (let loop ((seed seed) (chars '()))
                          (if (p seed)
                              (reverse chars)
                              (loop (g seed) (cons (f seed) chars))))))))
    
    (define (string-unfold-right p f g seed . maybe-base-string)
      (let ((base (if (null? maybe-base-string) "" (car maybe-base-string))))
        (string-append (list->string
                        (let loop ((seed seed) (chars '()))
                          (if (p seed)
                              chars
                              (loop (g seed) (cons (f seed) chars)))))
                       base)))
    
    (define string-for-each
      (case-lambda
        ((proc s) (string-for-each proc s 0 (string-length s)))
        ((proc s start) (string-for-each proc s start (string-length s)))
        ((proc s start end)
         (do ((i start (+ i 1)))
             ((>= i end))
           (proc (string-ref s i))))))
    
    (define (string-for-each-index proc s . maybe-start-end)
      (let ((start (if (null? maybe-start-end) 0 (car maybe-start-end)))
            (end (if (or (null? maybe-start-end) (null? (cdr maybe-start-end)))
                     (string-length s)
                     (cadr maybe-start-end))))
        (do ((i start (+ i 1)))
            ((>= i end))
          (proc i))))
    
    ;; ============= REPLICATE & ROTATE =============
    
    (define xsubstring
      (case-lambda
        ((s from) (xsubstring s from (+ from (string-length s))))
        ((s from to)
         (let* ((len (string-length s))
                (result-len (- to from))
                (result (make-string result-len)))
           (do ((i 0 (+ i 1))
                (j from (+ j 1)))
               ((= i result-len) result)
             (string-set! result i (string-ref s (modulo j len))))))))
    
    (define string-xcopy!
      (case-lambda
        ((target tstart s sfrom) (string-xcopy! target tstart s sfrom (+ sfrom (string-length s))))
        ((target tstart s sfrom sto)
         (let ((slen (string-length s)))
           (do ((i tstart (+ i 1))
                (j sfrom (+ j 1)))
               ((= j sto))
             (string-set! target i (string-ref s (modulo j slen))))))))
    
    ;; ============= MISCELLANEOUS =============
    
    (define string-tokenize
      (case-lambda
        ((s) (string-tokenize s char-graphic?))
        ((s token-chars) (string-tokenize s token-chars 0 (string-length s)))
        ((s token-chars start) (string-tokenize s token-chars start (string-length s)))
        ((s token-chars start end)
         (let loop ((i start) (tokens '()) (current-start #f))
           (cond
             ((>= i end)
              (if current-start
                  (reverse (cons (substring s current-start end) tokens))
                  (reverse tokens)))
             ((if (procedure? token-chars)
                  (token-chars (string-ref s i))
                  (char=? (string-ref s i) token-chars))
              (if current-start
                  (loop (+ i 1) tokens current-start)
                  (loop (+ i 1) tokens i)))
             (else
              (if current-start
                  (loop (+ i 1) (cons (substring s current-start i) tokens) #f)
                  (loop (+ i 1) tokens #f))))))))
    
    (define string-filter
      (case-lambda
        ((pred s) (string-filter pred s 0 (string-length s)))
        ((pred s start) (string-filter pred s start (string-length s)))
        ((pred s start end)
         (list->string
          (let loop ((i start) (chars '()))
            (cond
              ((>= i end) (reverse chars))
              ((if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i)))
               (loop (+ i 1) (cons (string-ref s i) chars)))
              (else (loop (+ i 1) chars))))))))
    
    (define string-delete
      (case-lambda
        ((pred s) (string-delete pred s 0 (string-length s)))
        ((pred s start) (string-delete pred s start (string-length s)))
        ((pred s start end)
         (list->string
          (let loop ((i start) (chars '()))
            (cond
              ((>= i end) (reverse chars))
              ((not (if (char? pred) (char=? (string-ref s i) pred) (pred (string-ref s i))))
               (loop (+ i 1) (cons (string-ref s i) chars)))
              (else (loop (+ i 1) chars))))))))))