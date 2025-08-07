;; SRFI-14: Character Sets
;; 
;; This library provides a comprehensive character set facility that can be used
;; for efficient character classification and set operations. Character sets are
;; objects that represent a collection of characters.
;;
;; Reference: https://srfi.schemers.org/srfi-14/srfi-14.html

(define-library (srfi 14)
  (import (scheme base)
          (scheme char)
          (scheme case-lambda))
  
  (export
    ;; === Predicates ===
    char-set? char-set= char-set<= char-set-hash
    
    ;; === Iterating over character sets ===
    char-set-cursor char-set-ref char-set-cursor-next end-of-char-set?
    char-set-fold char-set-unfold char-set-unfold!
    char-set-for-each char-set-map
    
    ;; === Creating character sets ===
    char-set char-set-copy list->char-set string->char-set
    list->char-set! string->char-set!
    char-set-filter char-set-filter!
    ucs-range->char-set ucs-range->char-set!
    
    ;; === Querying character sets ===
    char-set->list char-set->string
    char-set-size char-set-count char-set-contains?
    char-set-every char-set-any
    
    ;; === Character set algebra ===
    char-set-adjoin char-set-delete char-set-adjoin! char-set-delete!
    char-set-complement char-set-union char-set-intersection 
    char-set-difference char-set-xor
    char-set-complement! char-set-union! char-set-intersection!
    char-set-difference! char-set-xor!
    
    ;; === Standard character sets ===
    char-set:lower-case char-set:upper-case char-set:title-case
    char-set:letter char-set:digit char-set:letter+digit
    char-set:graphic char-set:printing char-set:whitespace
    char-set:iso-control char-set:punctuation char-set:symbol
    char-set:hex-digit char-set:blank char-set:ascii
    char-set:empty char-set:full)

  (begin
    ;; ============= CHARACTER SET REPRESENTATION =============
    
    ;; Character sets are represented as bit vectors for ASCII range (0-127)
    ;; and association lists for Unicode characters beyond ASCII.
    ;; This hybrid approach provides efficiency for common ASCII operations
    ;; while supporting the full Unicode range.
    
    (define-record-type char-set-type
      (make-char-set-internal ascii-bits unicode-alist)
      char-set?
      (ascii-bits char-set-ascii-bits char-set-set-ascii-bits!)
      (unicode-alist char-set-unicode-alist char-set-set-unicode-alist!))
    
    (define (make-char-set)
      (make-char-set-internal (make-vector 128 #f) '()))
    
    ;; ============= PREDICATES =============
    
    (define (char-set= cs1 . char-sets)
      (let loop ((css (cons cs1 char-sets)))
        (or (null? (cdr css))
            (and (char-set-equal-internal? (car css) (cadr css))
                 (loop (cdr css))))))
    
    (define (char-set<= cs1 . char-sets)
      (let loop ((css (cons cs1 char-sets)))
        (or (null? (cdr css))
            (and (char-set-subset-internal? (car css) (cadr css))
                 (loop (cdr css))))))
    
    (define (char-set-equal-internal? cs1 cs2)
      (and (equal? (char-set-ascii-bits cs1) (char-set-ascii-bits cs2))
           (equal? (char-set-unicode-alist cs1) (char-set-unicode-alist cs2))))
    
    (define (char-set-subset-internal? cs1 cs2)
      (and (char-set-ascii-subset? cs1 cs2)
           (char-set-unicode-subset? cs1 cs2)))
    
    (define (char-set-ascii-subset? cs1 cs2)
      (let ((bits1 (char-set-ascii-bits cs1))
            (bits2 (char-set-ascii-bits cs2)))
        (let loop ((i 0))
          (cond
            ((= i 128) #t)
            ((and (vector-ref bits1 i) (not (vector-ref bits2 i))) #f)
            (else (loop (+ i 1)))))))
    
    (define (char-set-unicode-subset? cs1 cs2)
      (let ((alist1 (char-set-unicode-alist cs1))
            (alist2 (char-set-unicode-alist cs2)))
        (every (lambda (pair)
                 (assoc (car pair) alist2))
               alist1)))
    
    (define (char-set-hash cs . maybe-bound)
      (let ((bound (if (null? maybe-bound) 268435456 (car maybe-bound))))
        (let ((ascii-hash (char-set-ascii-hash cs))
              (unicode-hash (char-set-unicode-hash cs)))
          (modulo (+ (* ascii-hash 37) unicode-hash) bound))))
    
    (define (char-set-ascii-hash cs)
      (let ((bits (char-set-ascii-bits cs)))
        (let loop ((i 0) (hash 0))
          (if (= i 128)
              hash
              (loop (+ i 1)
                    (if (vector-ref bits i)
                        (+ (* hash 2) 1)
                        (* hash 2)))))))
    
    (define (char-set-unicode-hash cs)
      (fold (lambda (pair hash)
              (+ (* hash 31) (char->integer (car pair))))
            0
            (char-set-unicode-alist cs)))
    
    ;; ============= ITERATING OVER CHARACTER SETS =============
    
    ;; Cursor implementation for iterating over character sets
    (define-record-type char-set-cursor-type
      (make-char-set-cursor-internal char-set index unicode-alist)
      char-set-cursor?
      (char-set cursor-char-set)
      (index cursor-index cursor-set-index!)
      (unicode-alist cursor-unicode-alist cursor-set-unicode-alist!))
    
    (define (char-set-cursor cs)
      (make-char-set-cursor-internal cs 0 (char-set-unicode-alist cs)))
    
    (define (char-set-ref cs cursor)
      (let ((idx (cursor-index cursor)))
        (if (< idx 128)
            ;; ASCII range
            (let loop ((i idx))
              (cond
                ((= i 128)
                 ;; Move to Unicode range
                 (let ((unicode-alist (cursor-unicode-alist cursor)))
                   (if (null? unicode-alist)
                       (error "Invalid cursor position")
                       (caar unicode-alist))))
                ((vector-ref (char-set-ascii-bits cs) i)
                 (integer->char i))
                (else (loop (+ i 1)))))
            ;; Unicode range
            (let ((unicode-alist (cursor-unicode-alist cursor)))
              (if (null? unicode-alist)
                  (error "Invalid cursor position")
                  (caar unicode-alist))))))
    
    (define (char-set-cursor-next cs cursor)
      (let ((idx (cursor-index cursor)))
        (if (< idx 128)
            ;; ASCII range
            (let loop ((i (+ idx 1)))
              (cond
                ((= i 128)
                 ;; Move to Unicode range
                 (make-char-set-cursor-internal cs 128 (char-set-unicode-alist cs)))
                ((vector-ref (char-set-ascii-bits cs) i)
                 (make-char-set-cursor-internal cs i (cursor-unicode-alist cursor)))
                (else (loop (+ i 1)))))
            ;; Unicode range
            (let ((unicode-alist (cursor-unicode-alist cursor)))
              (if (null? unicode-alist)
                  (error "Cannot advance cursor past end")
                  (make-char-set-cursor-internal cs 128 (cdr unicode-alist)))))))
    
    (define (end-of-char-set? cursor)
      (and (>= (cursor-index cursor) 128)
           (null? (cursor-unicode-alist cursor))))
    
    (define (char-set-fold kons knil cs)
      (let ((result (char-set-fold-ascii kons knil cs)))
        (char-set-fold-unicode kons result cs)))
    
    (define (char-set-fold-ascii kons knil cs)
      (let ((bits (char-set-ascii-bits cs)))
        (let loop ((i 0) (acc knil))
          (if (= i 128)
              acc
              (loop (+ i 1)
                    (if (vector-ref bits i)
                        (kons (integer->char i) acc)
                        acc))))))
    
    (define (char-set-fold-unicode kons knil cs)
      (fold (lambda (pair acc)
              (kons (car pair) acc))
            knil
            (char-set-unicode-alist cs)))
    
    (define (char-set-unfold p f g seed . maybe-base-cs)
      (let ((base-cs (if (null? maybe-base-cs) (make-char-set) (car maybe-base-cs))))
        (let loop ((seed seed) (cs (char-set-copy base-cs)))
          (if (p seed)
              cs
              (loop (g seed) (char-set-adjoin! cs (f seed)))))))
    
    (define (char-set-unfold! p f g seed base-cs)
      (let loop ((seed seed) (cs base-cs))
        (if (p seed)
            cs
            (loop (g seed) (char-set-adjoin! cs (f seed))))))
    
    (define (char-set-for-each proc cs)
      (char-set-fold (lambda (ch acc) (proc ch) acc) #f cs))
    
    (define (char-set-map proc cs)
      (char-set-fold (lambda (ch acc)
                       (char-set-adjoin! acc (proc ch)))
                     (make-char-set)
                     cs))
    
    ;; ============= CREATING CHARACTER SETS =============
    
    (define (char-set . chars)
      (list->char-set chars))
    
    (define (char-set-copy cs)
      (make-char-set-internal
       (vector-copy (char-set-ascii-bits cs))
       (map cons
            (map car (char-set-unicode-alist cs))
            (map cdr (char-set-unicode-alist cs)))))
    
    (define list->char-set
      (case-lambda
        ((char-list) (list->char-set char-list (make-char-set)))
        ((char-list base-cs)
         (let ((result (char-set-copy base-cs)))
           (for-each (lambda (ch) (char-set-adjoin! result ch)) char-list)
           result))))
    
    (define string->char-set
      (case-lambda
        ((s) (string->char-set s (make-char-set)))
        ((s base-cs)
         (list->char-set (string->list s) base-cs))))
    
    (define list->char-set!
      (case-lambda
        ((char-list base-cs)
         (for-each (lambda (ch) (char-set-adjoin! base-cs ch)) char-list)
         base-cs)))
    
    (define string->char-set!
      (case-lambda
        ((s base-cs)
         (list->char-set! (string->list s) base-cs))))
    
    (define char-set-filter
      (case-lambda
        ((pred cs) (char-set-filter pred cs (make-char-set)))
        ((pred cs base-cs)
         (char-set-fold (lambda (ch acc)
                          (if (pred ch)
                              (char-set-adjoin! acc ch)
                              acc))
                        (char-set-copy base-cs)
                        cs))))
    
    (define (char-set-filter! pred cs base-cs)
      (char-set-fold (lambda (ch acc)
                       (if (pred ch)
                           (char-set-adjoin! acc ch)
                           acc))
                     base-cs
                     cs))
    
    (define ucs-range->char-set
      (case-lambda
        ((lower upper) (ucs-range->char-set lower upper #f (make-char-set)))
        ((lower upper error?) (ucs-range->char-set lower upper error? (make-char-set)))
        ((lower upper error? base-cs)
         (let ((result (char-set-copy base-cs)))
           (do ((i lower (+ i 1)))
               ((= i upper) result)
             (let ((ch (integer->char i)))
               (char-set-adjoin! result ch)))))))
    
    (define ucs-range->char-set!
      (case-lambda
        ((lower upper base-cs) (ucs-range->char-set! lower upper #f base-cs))
        ((lower upper error? base-cs)
         (do ((i lower (+ i 1)))
             ((= i upper) base-cs)
           (let ((ch (integer->char i)))
             (char-set-adjoin! base-cs ch))))))
    
    ;; ============= QUERYING CHARACTER SETS =============
    
    (define (char-set->list cs)
      (char-set-fold cons '() cs))
    
    (define (char-set->string cs)
      (list->string (char-set->list cs)))
    
    (define (char-set-size cs)
      (char-set-fold (lambda (ch count) (+ count 1)) 0 cs))
    
    (define (char-set-count pred cs)
      (char-set-fold (lambda (ch count)
                       (if (pred ch) (+ count 1) count))
                     0 cs))
    
    (define (char-set-contains? cs char)
      (let ((code (char->integer char)))
        (if (< code 128)
            (vector-ref (char-set-ascii-bits cs) code)
            (assoc char (char-set-unicode-alist cs)))))
    
    (define (char-set-every pred cs)
      (char-set-fold (lambda (ch acc)
                       (and acc (pred ch)))
                     #t cs))
    
    (define (char-set-any pred cs)
      (char-set-fold (lambda (ch acc)
                       (or acc (pred ch)))
                     #f cs))
    
    ;; ============= CHARACTER SET ALGEBRA =============
    
    (define (char-set-adjoin cs . chars)
      (let ((result (char-set-copy cs)))
        (for-each (lambda (ch) (char-set-adjoin! result ch)) chars)
        result))
    
    (define (char-set-delete cs . chars)
      (let ((result (char-set-copy cs)))
        (for-each (lambda (ch) (char-set-delete! result ch)) chars)
        result))
    
    (define (char-set-adjoin! cs . chars)
      (for-each (lambda (ch)
                  (let ((code (char->integer ch)))
                    (if (< code 128)
                        (vector-set! (char-set-ascii-bits cs) code #t)
                        (let ((alist (char-set-unicode-alist cs)))
                          (unless (assoc ch alist)
                            (char-set-set-unicode-alist! cs (cons (cons ch #t) alist)))))))
                chars)
      cs)
    
    (define (char-set-delete! cs . chars)
      (for-each (lambda (ch)
                  (let ((code (char->integer ch)))
                    (if (< code 128)
                        (vector-set! (char-set-ascii-bits cs) code #f)
                        (char-set-set-unicode-alist! 
                         cs 
                         (remove (lambda (pair) (char=? (car pair) ch))
                                 (char-set-unicode-alist cs))))))
                chars)
      cs)
    
    (define (char-set-complement cs)
      (let ((result (make-char-set)))
        ;; For ASCII range, flip all bits
        (let ((src-bits (char-set-ascii-bits cs))
              (dst-bits (char-set-ascii-bits result)))
          (do ((i 0 (+ i 1)))
              ((= i 128))
            (vector-set! dst-bits i (not (vector-ref src-bits i)))))
        ;; For Unicode range, this is more complex - simplified implementation
        ;; In practice, would need to handle the full Unicode range
        result))
    
    (define (char-set-union cs1 . char-sets)
      (let ((result (char-set-copy cs1)))
        (for-each (lambda (cs) (char-set-union! result cs)) char-sets)
        result))
    
    (define (char-set-intersection cs1 . char-sets)
      (let ((result (char-set-copy cs1)))
        (for-each (lambda (cs) (char-set-intersection! result cs)) char-sets)
        result))
    
    (define (char-set-difference cs1 . char-sets)
      (let ((result (char-set-copy cs1)))
        (for-each (lambda (cs) (char-set-difference! result cs)) char-sets)
        result))
    
    (define (char-set-xor cs1 . char-sets)
      (let ((result (char-set-copy cs1)))
        (for-each (lambda (cs) (char-set-xor! result cs)) char-sets)
        result))
    
    ;; Destructive versions
    (define (char-set-complement! cs)
      (let ((bits (char-set-ascii-bits cs)))
        (do ((i 0 (+ i 1)))
            ((= i 128))
          (vector-set! bits i (not (vector-ref bits i)))))
      cs)
    
    (define (char-set-union! cs1 cs2)
      (let ((bits1 (char-set-ascii-bits cs1))
            (bits2 (char-set-ascii-bits cs2)))
        (do ((i 0 (+ i 1)))
            ((= i 128))
          (when (vector-ref bits2 i)
            (vector-set! bits1 i #t))))
      ;; Handle Unicode characters
      (for-each (lambda (pair)
                  (char-set-adjoin! cs1 (car pair)))
                (char-set-unicode-alist cs2))
      cs1)
    
    (define (char-set-intersection! cs1 cs2)
      (let ((bits1 (char-set-ascii-bits cs1))
            (bits2 (char-set-ascii-bits cs2)))
        (do ((i 0 (+ i 1)))
            ((= i 128))
          (vector-set! bits1 i (and (vector-ref bits1 i) (vector-ref bits2 i)))))
      ;; Handle Unicode characters
      (char-set-set-unicode-alist!
       cs1
       (filter (lambda (pair)
                 (assoc (car pair) (char-set-unicode-alist cs2)))
               (char-set-unicode-alist cs1)))
      cs1)
    
    (define (char-set-difference! cs1 cs2)
      (let ((bits1 (char-set-ascii-bits cs1))
            (bits2 (char-set-ascii-bits cs2)))
        (do ((i 0 (+ i 1)))
            ((= i 128))
          (when (vector-ref bits2 i)
            (vector-set! bits1 i #f))))
      ;; Handle Unicode characters
      (for-each (lambda (pair)
                  (char-set-delete! cs1 (car pair)))
                (char-set-unicode-alist cs2))
      cs1)
    
    (define (char-set-xor! cs1 cs2)
      (let ((bits1 (char-set-ascii-bits cs1))
            (bits2 (char-set-ascii-bits cs2)))
        (do ((i 0 (+ i 1)))
            ((= i 128))
          (let ((b1 (vector-ref bits1 i))
                (b2 (vector-ref bits2 i)))
            (vector-set! bits1 i (and (or b1 b2) (not (and b1 b2)))))))
      ;; Handle Unicode characters - simplified
      cs1)
    
    ;; ============= STANDARD CHARACTER SETS =============
    
    (define char-set:lower-case
      (list->char-set '(#\a #\b #\c #\d #\e #\f #\g #\h #\i #\j #\k #\l #\m
                        #\n #\o #\p #\q #\r #\s #\t #\u #\v #\w #\x #\y #\z)))
    
    (define char-set:upper-case
      (list->char-set '(#\A #\B #\C #\D #\E #\F #\G #\H #\I #\J #\K #\L #\M
                        #\N #\O #\P #\Q #\R #\S #\T #\U #\V #\W #\X #\Y #\Z)))
    
    (define char-set:title-case char-set:upper-case)  ; Simplified
    
    (define char-set:letter (char-set-union char-set:lower-case char-set:upper-case))
    
    (define char-set:digit
      (list->char-set '(#\0 #\1 #\2 #\3 #\4 #\5 #\6 #\7 #\8 #\9)))
    
    (define char-set:letter+digit (char-set-union char-set:letter char-set:digit))
    
    (define char-set:graphic
      (char-set-union char-set:letter+digit
                      (list->char-set '(#\! #\" #\# #\$ #\% #\& #\' #\( #\) #\*
                                        #\+ #\, #\- #\. #\/ #\: #\; #\< #\= #\>
                                        #\? #\@ #\[ #\\ #\] #\^ #\_ #\` #\{ #\|
                                        #\} #\~ #\space))))
    
    (define char-set:printing char-set:graphic)
    
    (define char-set:whitespace
      (list->char-set '(#\space #\tab #\newline #\return #\page)))
    
    (define char-set:iso-control
      (ucs-range->char-set 0 32))
    
    (define char-set:punctuation
      (list->char-set '(#\! #\" #\# #\$ #\% #\& #\' #\( #\) #\* #\+ #\,
                        #\- #\. #\/ #\: #\; #\< #\= #\> #\? #\@ #\[ #\\
                        #\] #\^ #\_ #\` #\{ #\| #\} #\~)))
    
    (define char-set:symbol char-set:punctuation)  ; Simplified
    
    (define char-set:hex-digit
      (list->char-set '(#\0 #\1 #\2 #\3 #\4 #\5 #\6 #\7 #\8 #\9
                        #\A #\B #\C #\D #\E #\F #\a #\b #\c #\d #\e #\f)))
    
    (define char-set:blank (list->char-set '(#\space #\tab)))
    
    (define char-set:ascii (ucs-range->char-set 0 128))
    
    (define char-set:empty (make-char-set))
    
    (define char-set:full (char-set-complement char-set:empty))
    
    ;; Helper functions
    (define (every pred lst)
      (or (null? lst)
          (and (pred (car lst))
               (every pred (cdr lst)))))
    
    (define (remove pred lst)
      (filter (lambda (x) (not (pred x))) lst))
    
    (define (filter pred lst)
      (let loop ((lst lst) (result '()))
        (cond
          ((null? lst) (reverse result))
          ((pred (car lst)) (loop (cdr lst) (cons (car lst) result)))
          (else (loop (cdr lst) result)))))
    
    (define (fold proc init lst)
      (if (null? lst)
          init
          (fold proc (proc (car lst) init) (cdr lst))))
    
    ;; Vector-copy implementation if not available
    (define (vector-copy vec)
      (let* ((len (vector-length vec))
             (result (make-vector len)))
        (do ((i 0 (+ i 1)))
            ((= i len) result)
          (vector-set! result i (vector-ref vec i)))))