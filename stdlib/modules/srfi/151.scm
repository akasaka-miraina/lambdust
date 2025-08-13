;;; SRFI-151: Bitwise Operations
;;; 
;;; Complete implementation of SRFI-151 using pure Scheme
;;; All 38 procedures implemented with R7RS-small primitives

(define-library (srfi 151)
  
  ;; Export all 38 SRFI-151 procedures
  (export
    ;; Basic bitwise operations (11 procedures)
    bitwise-not bitwise-and bitwise-ior bitwise-xor bitwise-eqv
    bitwise-nand bitwise-nor bitwise-andc1 bitwise-andc2 bitwise-orc1 bitwise-orc2
    
    ;; Integer operations (4 procedures)
    arithmetic-shift bit-count integer-length bitwise-if
    
    ;; Single-bit operations (6 procedures)
    bit-set? copy-bit bit-swap any-bit-set? every-bit-set? first-set-bit
    
    ;; Bit field operations (9 procedures) 
    bit-field bit-field-any? bit-field-every? bit-field-clear bit-field-set
    bit-field-replace bit-field-replace-same bit-field-rotate bit-field-reverse
    
    ;; Conversion operations (5 procedures)
    bits->list list->bits bits->vector vector->bits bits
    
    ;; Higher-order operations (3 procedures)
    bitwise-fold bitwise-for-each bitwise-unfold)
  
  (import (scheme base))
  
  (begin
    
    ;; ============= HELPER FUNCTIONS =============
    
    (define (exact-integer? obj)
      (and (integer? obj) (exact? obj)))
    
    (define (validate-exact-integer obj proc-name)
      (if (not (exact-integer? obj))
        (error (string-append proc-name ": expected exact integer") obj)))
    
    (define (validate-bit-index index proc-name)
      (validate-exact-integer index proc-name)
      (if (not (>= index 0))
        (error (string-append proc-name ": bit index must be non-negative") index)))
    
    (define (validate-bit-range start end proc-name)
      (validate-bit-index start proc-name)
      (validate-bit-index end proc-name)
      (if (not (>= end start))
        (error (string-append proc-name ": end must be >= start") start end)))
    
    (define (validate-bit-value bit proc-name)
      (validate-exact-integer bit proc-name)
      (if (not (or (= bit 0) (= bit 1)))
        (error (string-append proc-name ": bit value must be 0 or 1") bit)))
    
    ;; ============= CORE PRIMITIVES =============
    
    ;; Bitwise NOT using two's complement: ~n = -1 - n
    (define %bitwise-not
      (lambda (n) (- -1 n)))
    
    ;; Bitwise AND using recursive bit-by-bit approach
    (define %bitwise-and
      (lambda (n1 n2)
        (cond
          ((= n1 0) 0)
          ((= n2 0) 0)
          ((= n1 -1) n2)
          ((= n2 -1) n1)
          (else
            (+ (* 2 (%bitwise-and (quotient n1 2) (quotient n2 2)))
               (if (and (odd? n1) (odd? n2)) 1 0))))))
    
    ;; Bitwise IOR using recursive bit-by-bit approach
    (define %bitwise-ior
      (lambda (n1 n2)
        (cond
          ((= n1 0) n2)
          ((= n2 0) n1) 
          ((= n1 -1) -1)
          ((= n2 -1) -1)
          (else
            (+ (* 2 (%bitwise-ior (quotient n1 2) (quotient n2 2)))
               (if (or (odd? n1) (odd? n2)) 1 0))))))
    
    ;; Bitwise XOR using recursive bit-by-bit approach
    (define %bitwise-xor
      (lambda (n1 n2)
        (cond
          ((= n1 n2) 0)
          ((= n1 0) n2)
          ((= n2 0) n1)
          (else
            (+ (* 2 (%bitwise-xor (quotient n1 2) (quotient n2 2)))
               (if (not (eq? (odd? n1) (odd? n2))) 1 0))))))
               
    ;; Arithmetic shift
    (define %arithmetic-shift
      (lambda (n count)
        (cond
          ((= count 0) n)
          ((> count 0) (* n (expt 2 count)))
          (else (quotient n (expt 2 (- count)))))))
          
    ;; Bit count
    (define %bit-count
      (lambda (n)
        (cond
          ((= n 0) 0)
          ((= n -1) -1)
          ((< n 0) (- (%bit-count (%bitwise-not n))))
          (else (+ (remainder n 2) (%bit-count (quotient n 2)))))))
          
    ;; Integer length
    (define %integer-length
      (lambda (n)
        (if (= n 0) 0
            (+ 1 (%integer-length (quotient (abs n) 2))))))
            
    ;; First set bit
    (define %first-set-bit
      (lambda (n)
        (cond
          ((= n 0) -1)
          ((odd? n) 0)
          (else (+ 1 (%first-set-bit (quotient n 2)))))))
    
    ;; ============= BASIC BITWISE OPERATIONS =============
    
    (define (bitwise-not n)
      (validate-exact-integer n "bitwise-not")
      (%bitwise-not n))
    
    ;; Multi-argument bitwise-and
    (define (bitwise-and . args)
      (define (bitwise-and-2 n1 n2)
        (validate-exact-integer n1 "bitwise-and")
        (validate-exact-integer n2 "bitwise-and")
        (%bitwise-and n1 n2))
      (cond
        ((null? args) -1)
        ((null? (cdr args)) 
         (validate-exact-integer (car args) "bitwise-and")
         (car args))
        (else
          (let loop ((result (car args)) (rest (cdr args)))
            (if (null? rest)
                result
                (loop (bitwise-and-2 result (car rest)) (cdr rest)))))))
    
    ;; Multi-argument bitwise-ior
    (define (bitwise-ior . args)
      (define (bitwise-ior-2 n1 n2)
        (validate-exact-integer n1 "bitwise-ior")
        (validate-exact-integer n2 "bitwise-ior")
        (%bitwise-ior n1 n2))
      (cond
        ((null? args) 0)
        ((null? (cdr args)) 
         (validate-exact-integer (car args) "bitwise-ior")
         (car args))
        (else
          (let loop ((result (car args)) (rest (cdr args)))
            (if (null? rest)
                result
                (loop (bitwise-ior-2 result (car rest)) (cdr rest)))))))
    
    ;; Multi-argument bitwise-xor
    (define (bitwise-xor . args)
      (define (bitwise-xor-2 n1 n2)
        (validate-exact-integer n1 "bitwise-xor")
        (validate-exact-integer n2 "bitwise-xor")
        (%bitwise-xor n1 n2))
      (cond
        ((null? args) 0)
        ((null? (cdr args)) 
         (validate-exact-integer (car args) "bitwise-xor")
         (car args))
        (else
          (let loop ((result (car args)) (rest (cdr args)))
            (if (null? rest)
                result
                (loop (bitwise-xor-2 result (car rest)) (cdr rest)))))))
    
    ;; Derived operations
    (define (bitwise-eqv . ns)
      (bitwise-not (apply bitwise-xor ns)))
    
    (define (bitwise-nand . ns)
      (bitwise-not (apply bitwise-and ns)))
    
    (define (bitwise-nor . ns)
      (bitwise-not (apply bitwise-ior ns)))
    
    (define (bitwise-andc1 n1 n2)
      (validate-exact-integer n1 "bitwise-andc1")
      (validate-exact-integer n2 "bitwise-andc1")
      (bitwise-and (bitwise-not n1) n2))
    
    (define (bitwise-andc2 n1 n2)
      (validate-exact-integer n1 "bitwise-andc2")
      (validate-exact-integer n2 "bitwise-andc2")
      (bitwise-and n1 (bitwise-not n2)))
    
    (define (bitwise-orc1 n1 n2)
      (validate-exact-integer n1 "bitwise-orc1")
      (validate-exact-integer n2 "bitwise-orc1")
      (bitwise-ior (bitwise-not n1) n2))
    
    (define (bitwise-orc2 n1 n2)
      (validate-exact-integer n1 "bitwise-orc2")
      (validate-exact-integer n2 "bitwise-orc2")
      (bitwise-ior n1 (bitwise-not n2)))
    
    ;; ============= INTEGER OPERATIONS =============
    
    (define (arithmetic-shift n count)
      (validate-exact-integer n "arithmetic-shift")
      (validate-exact-integer count "arithmetic-shift")
      (%arithmetic-shift n count))
    
    (define (bit-count n)
      (validate-exact-integer n "bit-count")
      (%bit-count n))
    
    (define (integer-length n)
      (validate-exact-integer n "integer-length")
      (%integer-length n))
    
    (define (bitwise-if mask n0 n1)
      (validate-exact-integer mask "bitwise-if")
      (validate-exact-integer n0 "bitwise-if")
      (validate-exact-integer n1 "bitwise-if")
      (bitwise-ior (bitwise-and mask n1) 
                   (bitwise-andc1 mask n0)))
    
    ;; ============= SINGLE-BIT OPERATIONS =============
    
    (define (bit-set? index n)
      (validate-bit-index index "bit-set?")
      (validate-exact-integer n "bit-set?")
      (not (= 0 (bitwise-and n (arithmetic-shift 1 index)))))
    
    (define (copy-bit index n bit)
      (validate-bit-index index "copy-bit")
      (validate-exact-integer n "copy-bit")
      (validate-bit-value bit "copy-bit")
      (let ((mask (arithmetic-shift 1 index)))
        (if (= 0 bit)
            (bitwise-andc2 n mask)
            (bitwise-ior n mask))))
    
    (define (bit-swap index1 index2 n)
      (validate-bit-index index1 "bit-swap")
      (validate-bit-index index2 "bit-swap")
      (validate-exact-integer n "bit-swap")
      (let ((bit1 (if (bit-set? index1 n) 1 0))
            (bit2 (if (bit-set? index2 n) 1 0)))
        (copy-bit index2 (copy-bit index1 n bit2) bit1)))
    
    (define (any-bit-set? test-bits n)
      (validate-exact-integer test-bits "any-bit-set?")
      (validate-exact-integer n "any-bit-set?")
      (not (= 0 (bitwise-and test-bits n))))
    
    (define (every-bit-set? test-bits n)
      (validate-exact-integer test-bits "every-bit-set?")
      (validate-exact-integer n "every-bit-set?")
      (= test-bits (bitwise-and test-bits n)))
    
    (define (first-set-bit n)
      (validate-exact-integer n "first-set-bit")
      (%first-set-bit n))
    
    ;; ============= BIT FIELD OPERATIONS =============
    
    (define (bit-field n start end)
      (validate-exact-integer n "bit-field")
      (validate-bit-range start end "bit-field")
      (let ((width (- end start)))
        (if (= width 0)
            0
            (let ((mask (- (arithmetic-shift 1 width) 1)))
              (bitwise-and mask (arithmetic-shift n (- start)))))))
    
    (define (bit-field-any? n start end)
      (validate-exact-integer n "bit-field-any?")
      (validate-bit-range start end "bit-field-any?")
      (not (= 0 (bit-field n start end))))
    
    (define (bit-field-every? n start end)
      (validate-exact-integer n "bit-field-every?")
      (validate-bit-range start end "bit-field-every?")
      (let ((width (- end start)))
        (if (= width 0)
            #t
            (let ((expected-field (- (arithmetic-shift 1 width) 1))
                  (actual-field (bit-field n start end)))
              (= expected-field actual-field)))))
    
    (define (bit-field-replace n newfield start end)
      (validate-exact-integer n "bit-field-replace")
      (validate-exact-integer newfield "bit-field-replace")
      (validate-bit-range start end "bit-field-replace")
      (let ((width (- end start)))
        (if (= width 0)
            n
            (let* ((mask (- (arithmetic-shift 1 width) 1))
                   (cleared-n (bitwise-and n (bitwise-not (arithmetic-shift mask start))))
                   (shifted-newfield (arithmetic-shift (bitwise-and newfield mask) start)))
              (bitwise-ior cleared-n shifted-newfield)))))
    
    (define (bit-field-clear n start end)
      (validate-exact-integer n "bit-field-clear")
      (validate-bit-range start end "bit-field-clear")
      (bit-field-replace n 0 start end))
    
    (define (bit-field-set n start end)
      (validate-exact-integer n "bit-field-set")
      (validate-bit-range start end "bit-field-set")
      (let ((width (- end start)))
        (if (= width 0)
            n
            (let ((mask (- (arithmetic-shift 1 width) 1)))
              (bit-field-replace n mask start end)))))
    
    (define (bit-field-replace-same to from start end)
      (validate-exact-integer to "bit-field-replace-same")
      (validate-exact-integer from "bit-field-replace-same")
      (validate-bit-range start end "bit-field-replace-same")
      (let ((field (bit-field from start end)))
        (bit-field-replace to field start end)))
    
    (define (bit-field-rotate n count start end)
      (validate-exact-integer n "bit-field-rotate")
      (validate-exact-integer count "bit-field-rotate")
      (validate-bit-range start end "bit-field-rotate")
      (let ((width (- end start)))
        (if (<= width 1)
            n
            (let* ((field (bit-field n start end))
                   (shift (modulo count width))
                   (rotated (bitwise-ior 
                            (arithmetic-shift field shift)
                            (arithmetic-shift field (- shift width))))
                   (mask (- (arithmetic-shift 1 width) 1)))
              (bit-field-replace n (bitwise-and rotated mask) start end)))))
    
    (define (bit-field-reverse n start end)
      (validate-exact-integer n "bit-field-reverse")
      (validate-bit-range start end "bit-field-reverse")
      (let ((width (- end start)))
        (if (<= width 1)
            n
            (let ((field (bit-field n start end)))
              (let loop ((i 0) (reversed 0))
                (if (= i width)
                    (bit-field-replace n reversed start end)
                    (let ((bit (if (bit-set? i field) 1 0)))
                      (loop (+ i 1)
                            (bitwise-ior reversed 
                                       (arithmetic-shift bit (- width i 1)))))))))))
    
    ;; ============= CONVERSION OPERATIONS =============
    
    ;; Simplified bits->list without optional length parameter
    (define (bits->list n . length)
      (validate-exact-integer n "bits->list")
      (let ((len (if (null? length)
                     (max 1 (+ (integer-length (abs n)) (if (negative? n) 1 0)))
                     (begin 
                       (validate-exact-integer (car length) "bits->list")
                       (if (not (>= (car length) 0))
                         (error "bits->list: length must be non-negative" (car length)))
                       (car length)))))
        (let loop ((i 0) (result '()))
          (if (= i len)
              (reverse result)
              (loop (+ i 1) 
                    (cons (if (bit-set? i n) 1 0) result))))))
    
    (define (list->bits bits)
      (if (not (list? bits))
        (error "list->bits: expected list" bits))
      (let loop ((bits (reverse bits)) (i 0) (result 0))
        (cond
          ((null? bits) result)
          (else
           (let ((bit (car bits)))
             (validate-bit-value bit "list->bits")
             (loop (cdr bits) 
                   (+ i 1)
                   (if (= bit 1)
                       (bitwise-ior result (arithmetic-shift 1 i))
                       result)))))))
    
    ;; Simplified bits->vector
    (define (bits->vector n . length)
      (list->vector (apply bits->list (cons n length))))
    
    (define (vector->bits bits)
      (if (not (vector? bits))
        (error "vector->bits: expected vector" bits))
      (list->bits (vector->list bits)))
    
    (define (bits . bits)
      (list->bits bits))
    
    ;; ============= HIGHER-ORDER OPERATIONS =============
    
    (define (bitwise-fold proc seed n)
      (validate-exact-integer n "bitwise-fold")
      (if (not (procedure? proc))
        (error "bitwise-fold: expected procedure" proc))
      (let ((len (max 1 (+ (integer-length (abs n)) (if (negative? n) 1 0)))))
        (let loop ((i 0) (acc seed))
          (if (= i len)
              acc
              (let ((bit (if (bit-set? i n) 1 0)))
                (loop (+ i 1) (proc bit acc)))))))
    
    (define (bitwise-for-each proc n)
      (validate-exact-integer n "bitwise-for-each")
      (if (not (procedure? proc))
        (error "bitwise-for-each: expected procedure" proc))
      (let ((len (max 1 (+ (integer-length (abs n)) (if (negative? n) 1 0)))))
        (let loop ((i 0))
          (if (not (= i len))
            (begin
              (let ((bit (if (bit-set? i n) 1 0)))
                (proc bit)
                (loop (+ i 1))))))))
    
    (define (bitwise-unfold stop? mapper successor seed)
      (if (not (procedure? stop?))
        (error "bitwise-unfold: stop? must be a procedure" stop?))
      (if (not (procedure? mapper))
        (error "bitwise-unfold: mapper must be a procedure" mapper))
      (if (not (procedure? successor))
        (error "bitwise-unfold: successor must be a procedure" successor))
      (let loop ((state seed) (i 0) (result 0))
        (if (stop? state)
            result
            (let ((bit (mapper state)))
              (if (not (exact-integer? bit))
                (error "bitwise-unfold: mapper must return exact integer" bit))
              (if (not (or (= bit 0) (= bit 1)))
                (error "bitwise-unfold: mapper must return 0 or 1" bit))
              (loop (successor state)
                    (+ i 1)
                    (if (= bit 1)
                        (bitwise-ior result (arithmetic-shift 1 i))
                        result))))))
    
    )) ; End of define-library