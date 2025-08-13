;; SRFI-135: Immutable Texts
;; 
;; This library provides immutable text strings with efficient operations,
;; supporting Unicode and internationalization features. Texts are immutable
;; sequences of characters with efficient substring operations and concatenation.
;;
;; Reference: https://srfi.schemers.org/srfi-135/srfi-135.html

(define-library (srfi 135)
  (import (scheme base)
          (scheme char))
  
  (export
    ;; === Text Constructors ===
    text text-tabulate
    
    ;; === Text Predicates ===
    text? text-null? text-every text-any
    
    ;; === Text Selectors ===
    text-length text-ref text-start text-end
    
    ;; === Text Comparison ===
    text=? text<? text>? text<=? text>=?
    text-ci=? text-ci<? text-ci>? text-ci<=? text-ci>=?
    
    ;; === Text Prefix/Suffix ===
    text-prefix-length text-suffix-length
    text-prefix? text-suffix?
    
    ;; === Text Searching ===
    text-index text-index-right text-skip text-skip-right
    text-contains text-contains-right
    
    ;; === Text Case ===
    text-upcase text-downcase text-foldcase text-titlecase
    
    ;; === Text Reversal ===
    text-reverse
    
    ;; === Text Concatenation ===
    text-append text-concatenate text-concatenate-reverse
    text-join
    
    ;; === Text Mapping ===
    text-map text-map-index text-for-each text-for-each-index
    text-count
    
    ;; === Text Filtering ===
    text-filter text-remove
    
    ;; === Text Splitting ===
    text-segment text-split
    
    ;; === Text Padding/Trimming ===
    text-pad text-pad-right text-trim text-trim-right text-trim-both
    
    ;; === Text Replacement ===
    text-replace
    
    ;; === Subtexts ===
    subtext text-copy text-take text-drop
    text-take-right text-drop-right
    
    ;; === Conversions ===
    text->string string->text
    text->vector vector->text
    text->list list->text
    text->utf8 utf8->text text->utf16 utf16->text)

  (begin
    ;; ============= INTERNAL TEXT REPRESENTATION =============
    
    ;; Text is represented as an immutable string with metadata
    ;; for efficient operations. In this simplified implementation,
    ;; we use strings as the underlying representation.
    
    (define text-type-tag 'text)
    
    ;; Internal text constructor
    (define (%make-text string start end)
      "Create internal text representation."
      (vector text-type-tag string start end))
    
    ;; Internal text accessors
    (define (%text-string t) (vector-ref t 1))
    (define (%text-start t) (vector-ref t 2))
    (define (%text-end t) (vector-ref t 3))
    
    ;; ============= TEXT CONSTRUCTORS =============
    
    ;; Create text from characters
    (define (text . chars)
      "Create a text from zero or more characters."
      (if (null? chars)
          (%make-text "" 0 0)
          (let ((str (list->string chars)))
            (%make-text str 0 (string-length str)))))
    
    ;; Create text by tabulating function over indices
    (define (text-tabulate proc len)
      "Create text by calling proc for indices 0 to len-1."
      (unless (procedure? proc)
        (error "text-tabulate: first argument must be a procedure" proc))
      (unless (and (integer? len) (>= len 0))
        (error "text-tabulate: second argument must be non-negative integer" len))
      (if (= len 0)
          (%make-text "" 0 0)
          (let ((chars (make-vector len)))
            (do ((i 0 (+ i 1)))
                ((>= i len)
                 (let ((str (list->string (vector->list chars))))
                   (%make-text str 0 len)))
              (let ((char (proc i)))
                (unless (char? char)
                  (error "text-tabulate: procedure must return character" char))
                (vector-set! chars i char))))))
    
    ;; ============= TEXT PREDICATES =============
    
    ;; Test if object is a text
    (define (text? obj)
      "Test whether obj is a text."
      (and (vector? obj)
           (= (vector-length obj) 4)
           (eq? (vector-ref obj 0) text-type-tag)))
    
    ;; Test if text is empty
    (define (text-null? txt)
      "Test whether text is empty."
      (unless (text? txt)
        (error "text-null?: not a text" txt))
      (= (%text-start txt) (%text-end txt)))
    
    ;; Test if predicate holds for all characters
    (define (text-every pred txt . args)
      "Test whether predicate holds for every character in text."
      (unless (text? txt)
        (error "text-every: not a text" txt))
      (let* ((start (if (pair? args) (car args) (%text-start txt)))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (%text-end txt)))
             (str (%text-string txt)))
        (let loop ((i start))
          (cond
            ((>= i end) #t)
            ((pred (string-ref str i)) (loop (+ i 1)))
            (else #f)))))
    
    ;; Test if predicate holds for any character
    (define (text-any pred txt . args)
      "Test whether predicate holds for any character in text."
      (unless (text? txt)
        (error "text-any: not a text" txt))
      (let* ((start (if (pair? args) (car args) (%text-start txt)))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (%text-end txt)))
             (str (%text-string txt)))
        (let loop ((i start))
          (cond
            ((>= i end) #f)
            ((pred (string-ref str i)) #t)
            (else (loop (+ i 1)))))))
    
    ;; ============= TEXT SELECTORS =============
    
    ;; Get text length
    (define (text-length txt)
      "Return the length of text."
      (unless (text? txt)
        (error "text-length: not a text" txt))
      (- (%text-end txt) (%text-start txt)))
    
    ;; Get character at index
    (define (text-ref txt k)
      "Return the k-th character of text."
      (unless (text? txt)
        (error "text-ref: not a text" txt))
      (let ((start (%text-start txt))
            (end (%text-end txt)))
        (unless (and (>= k 0) (< k (- end start)))
          (error "text-ref: index out of bounds" k))
        (string-ref (%text-string txt) (+ start k))))
    
    ;; Get effective start index
    (define (text-start txt)
      "Return the start index of text (always 0 for this implementation)."
      (unless (text? txt)
        (error "text-start: not a text" txt))
      0)
    
    ;; Get effective end index
    (define (text-end txt)
      "Return the end index of text."
      (unless (text? txt)
        (error "text-end: not a text" txt))
      (text-length txt))
    
    ;; ============= TEXT COMPARISON =============
    
    ;; Text equality
    (define (text=? txt1 txt2 . args)
      "Test whether texts are equal."
      (unless (text? txt1)
        (error "text=?: first argument not a text" txt1))
      (unless (text? txt2)
        (error "text=?: second argument not a text" txt2))
      (let* ((start1 (if (pair? args) (car args) 0))
             (end1 (if (and (pair? args) (pair? (cdr args))) 
                       (cadr args) (text-length txt1)))
             (start2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args)))
                        (caddr args) 0))
             (end2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args))
                           (pair? (cdddr args)))
                      (cadddr args) (text-length txt2))))
        (and (= (- end1 start1) (- end2 start2))
             (string=? (substring (%text-string txt1) 
                                  (+ (%text-start txt1) start1)
                                  (+ (%text-start txt1) end1))
                       (substring (%text-string txt2)
                                  (+ (%text-start txt2) start2)
                                  (+ (%text-start txt2) end2))))))
    
    ;; Text lexicographic ordering
    (define (text<? txt1 txt2 . args)
      "Test whether txt1 is lexicographically less than txt2."
      (unless (text? txt1)
        (error "text<?: first argument not a text" txt1))
      (unless (text? txt2)
        (error "text<?: second argument not a text" txt2))
      (let* ((start1 (if (pair? args) (car args) 0))
             (end1 (if (and (pair? args) (pair? (cdr args))) 
                       (cadr args) (text-length txt1)))
             (start2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args)))
                        (caddr args) 0))
             (end2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args))
                           (pair? (cdddr args)))
                      (cadddr args) (text-length txt2))))
        (string<? (substring (%text-string txt1) 
                             (+ (%text-start txt1) start1)
                             (+ (%text-start txt1) end1))
                  (substring (%text-string txt2)
                             (+ (%text-start txt2) start2)
                             (+ (%text-start txt2) end2)))))
    
    ;; Additional comparison predicates
    (define (text>? txt1 txt2 . args)
      "Test whether txt1 is lexicographically greater than txt2."
      (apply text<? txt2 txt1 args))
    
    (define (text<=? txt1 txt2 . args)
      "Test whether txt1 is lexicographically less than or equal to txt2."
      (not (apply text>? txt1 txt2 args)))
    
    (define (text>=? txt1 txt2 . args)
      "Test whether txt1 is lexicographically greater than or equal to txt2."
      (not (apply text<? txt1 txt2 args)))
    
    ;; Case-insensitive comparisons
    (define (text-ci=? txt1 txt2 . args)
      "Test whether texts are equal ignoring case."
      (apply text=? (text-foldcase txt1) (text-foldcase txt2) args))
    
    (define (text-ci<? txt1 txt2 . args)
      "Test whether txt1 < txt2 ignoring case."
      (apply text<? (text-foldcase txt1) (text-foldcase txt2) args))
    
    (define (text-ci>? txt1 txt2 . args)
      "Test whether txt1 > txt2 ignoring case."
      (apply text>? (text-foldcase txt1) (text-foldcase txt2) args))
    
    (define (text-ci<=? txt1 txt2 . args)
      "Test whether txt1 <= txt2 ignoring case."
      (apply text<=? (text-foldcase txt1) (text-foldcase txt2) args))
    
    (define (text-ci>=? txt1 txt2 . args)
      "Test whether txt1 >= txt2 ignoring case."
      (apply text>=? (text-foldcase txt1) (text-foldcase txt2) args))
    
    ;; ============= TEXT PREFIX/SUFFIX =============
    
    ;; Length of common prefix
    (define (text-prefix-length txt1 txt2 . args)
      "Return length of common prefix of two texts."
      (unless (text? txt1)
        (error "text-prefix-length: first argument not a text" txt1))
      (unless (text? txt2)
        (error "text-prefix-length: second argument not a text" txt2))
      (let* ((start1 (if (pair? args) (car args) 0))
             (end1 (if (and (pair? args) (pair? (cdr args))) 
                       (cadr args) (text-length txt1)))
             (start2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args)))
                        (caddr args) 0))
             (end2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args))
                           (pair? (cdddr args)))
                      (cadddr args) (text-length txt2)))
             (len1 (- end1 start1))
             (len2 (- end2 start2))
             (min-len (min len1 len2)))
        (let loop ((i 0))
          (cond
            ((>= i min-len) i)
            ((char=? (text-ref txt1 (+ start1 i))
                     (text-ref txt2 (+ start2 i)))
             (loop (+ i 1)))
            (else i)))))
    
    ;; Length of common suffix
    (define (text-suffix-length txt1 txt2 . args)
      "Return length of common suffix of two texts."
      (unless (text? txt1)
        (error "text-suffix-length: first argument not a text" txt1))
      (unless (text? txt2)
        (error "text-suffix-length: second argument not a text" txt2))
      (let* ((start1 (if (pair? args) (car args) 0))
             (end1 (if (and (pair? args) (pair? (cdr args))) 
                       (cadr args) (text-length txt1)))
             (start2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args)))
                        (caddr args) 0))
             (end2 (if (and (pair? args) (pair? (cdr args)) (pair? (cddr args))
                           (pair? (cdddr args)))
                      (cadddr args) (text-length txt2)))
             (len1 (- end1 start1))
             (len2 (- end2 start2))
             (min-len (min len1 len2)))
        (let loop ((i 0))
          (cond
            ((>= i min-len) i)
            ((char=? (text-ref txt1 (- end1 1 i))
                     (text-ref txt2 (- end2 1 i)))
             (loop (+ i 1)))
            (else i)))))
    
    ;; Test for prefix
    (define (text-prefix? txt1 txt2 . args)
      "Test whether txt1 is a prefix of txt2."
      (let ((prefix-len (apply text-prefix-length txt1 txt2 args)))
        (= prefix-len (text-length txt1))))
    
    ;; Test for suffix
    (define (text-suffix? txt1 txt2 . args)
      "Test whether txt1 is a suffix of txt2."
      (let ((suffix-len (apply text-suffix-length txt1 txt2 args)))
        (= suffix-len (text-length txt1))))
    
    ;; ============= TEXT CASE CONVERSION =============
    
    ;; Convert to uppercase
    (define (text-upcase txt)
      "Convert text to uppercase."
      (unless (text? txt)
        (error "text-upcase: not a text" txt))
      (let ((str (substring (%text-string txt) (%text-start txt) (%text-end txt))))
        (string->text (string-upcase str))))
    
    ;; Convert to lowercase
    (define (text-downcase txt)
      "Convert text to lowercase."
      (unless (text? txt)
        (error "text-downcase: not a text" txt))
      (let ((str (substring (%text-string txt) (%text-start txt) (%text-end txt))))
        (string->text (string-downcase str))))
    
    ;; Convert to foldcase
    (define (text-foldcase txt)
      "Convert text to foldcase for case-insensitive comparison."
      (unless (text? txt)
        (error "text-foldcase: not a text" txt))
      (let ((str (substring (%text-string txt) (%text-start txt) (%text-end txt))))
        (string->text (string-foldcase str))))
    
    ;; Convert to titlecase
    (define (text-titlecase txt)
      "Convert text to titlecase."
      (unless (text? txt)
        (error "text-titlecase: not a text" txt))
      (let ((str (substring (%text-string txt) (%text-start txt) (%text-end txt))))
        (string->text (string-titlecase str))))
    
    ;; ============= TEXT CONCATENATION =============
    
    ;; Append texts
    (define (text-append . texts)
      "Append zero or more texts."
      (if (null? texts)
          (%make-text "" 0 0)
          (let ((strings (map (lambda (txt)
                                (unless (text? txt)
                                  (error "text-append: not a text" txt))
                                (substring (%text-string txt) 
                                           (%text-start txt) 
                                           (%text-end txt)))
                              texts)))
            (let ((result-str (apply string-append strings)))
              (%make-text result-str 0 (string-length result-str))))))
    
    ;; ============= TEXT REVERSAL =============
    
    ;; Reverse text
    (define (text-reverse txt)
      "Return the reverse of text."
      (unless (text? txt)
        (error "text-reverse: not a text" txt))
      (let* ((str (substring (%text-string txt) (%text-start txt) (%text-end txt)))
             (chars (string->list str))
             (reversed-chars (reverse chars))
             (reversed-str (list->string reversed-chars)))
        (%make-text reversed-str 0 (string-length reversed-str))))
    
    ;; ============= SUBTEXT OPERATIONS =============
    
    ;; Extract subtext
    (define (subtext txt start end)
      "Extract subtext from start to end."
      (unless (text? txt)
        (error "subtext: not a text" txt))
      (let ((len (text-length txt)))
        (unless (and (>= start 0) (<= start len))
          (error "subtext: start index out of bounds" start))
        (unless (and (>= end start) (<= end len))
          (error "subtext: end index out of bounds" end))
        (if (= start end)
            (%make-text "" 0 0)
            (let ((substr (substring (%text-string txt)
                                     (+ (%text-start txt) start)
                                     (+ (%text-start txt) end))))
              (%make-text substr 0 (- end start))))))
    
    ;; Copy text
    (define (text-copy txt . args)
      "Copy text, optionally with start/end bounds."
      (unless (text? txt)
        (error "text-copy: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (subtext txt start end)))
    
    ;; Take first n characters
    (define (text-take txt n)
      "Take first n characters of text."
      (unless (text? txt)
        (error "text-take: not a text" txt))
      (unless (and (integer? n) (>= n 0))
        (error "text-take: n must be non-negative integer" n))
      (let ((len (text-length txt)))
        (subtext txt 0 (min n len))))
    
    ;; Drop first n characters
    (define (text-drop txt n)
      "Drop first n characters of text."
      (unless (text? txt)
        (error "text-drop: not a text" txt))
      (unless (and (integer? n) (>= n 0))
        (error "text-drop: n must be non-negative integer" n))
      (let ((len (text-length txt)))
        (subtext txt (min n len) len)))
    
    ;; Take last n characters
    (define (text-take-right txt n)
      "Take last n characters of text."
      (unless (text? txt)
        (error "text-take-right: not a text" txt))
      (unless (and (integer? n) (>= n 0))
        (error "text-take-right: n must be non-negative integer" n))
      (let ((len (text-length txt)))
        (subtext txt (max 0 (- len n)) len)))
    
    ;; Drop last n characters
    (define (text-drop-right txt n)
      "Drop last n characters of text."
      (unless (text? txt)
        (error "text-drop-right: not a text" txt))
      (unless (and (integer? n) (>= n 0))
        (error "text-drop-right: n must be non-negative integer" n))
      (let ((len (text-length txt)))
        (subtext txt 0 (max 0 (- len n)))))
    
    ;; ============= CONVERSIONS =============
    
    ;; Convert text to string
    (define (text->string txt)
      "Convert text to string."
      (unless (text? txt)
        (error "text->string: not a text" txt))
      (substring (%text-string txt) (%text-start txt) (%text-end txt)))
    
    ;; Convert string to text
    (define (string->text str)
      "Convert string to text."
      (unless (string? str)
        (error "string->text: not a string" str))
      (%make-text str 0 (string-length str)))
    
    ;; Convert text to list
    (define (text->list txt . args)
      "Convert text to list of characters."
      (unless (text? txt)
        (error "text->list: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (string->list (text->string (subtext txt start end)))))
    
    ;; Convert list to text
    (define (list->text chars)
      "Convert list of characters to text."
      (unless (list? chars)
        (error "list->text: not a list" chars))
      (for-each (lambda (c)
                  (unless (char? c)
                    (error "list->text: not a character" c)))
                chars)
      (string->text (list->string chars)))
    
    ;; Convert text to vector
    (define (text->vector txt . args)
      "Convert text to vector of characters."
      (unless (text? txt)
        (error "text->vector: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (list->vector (text->list txt start end))))
    
    ;; Convert vector to text
    (define (vector->text vec . args)
      "Convert vector of characters to text."
      (unless (vector? vec)
        (error "vector->text: not a vector" vec))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (vector-length vec))))
        (list->text (vector->list (vector-copy vec start end)))))
    
    ;; Simple UTF-8 conversion (placeholder)
    (define (text->utf8 txt)
      "Convert text to UTF-8 bytevector."
      (unless (text? txt)
        (error "text->utf8: not a text" txt))
      (string->utf8 (text->string txt)))
    
    (define (utf8->text bv)
      "Convert UTF-8 bytevector to text."
      (unless (bytevector? bv)
        (error "utf8->text: not a bytevector" bv))
      (string->text (utf8->string bv)))
    
    ;; Simple UTF-16 conversion (placeholder)
    (define (text->utf16 txt . args)
      "Convert text to UTF-16 bytevector."
      (unless (text? txt)
        (error "text->utf16: not a text" txt))
      (let ((endianness (if (pair? args) (car args) 'big)))
        (string->utf16 (text->string txt) endianness)))
    
    (define (utf16->text bv . args)
      "Convert UTF-16 bytevector to text."
      (unless (bytevector? bv)
        (error "utf16->text: not a bytevector" bv))
      (let ((endianness (if (pair? args) (car args) 'big)))
        (string->text (utf16->string bv endianness))))
    
    ;; ============= PLACEHOLDER IMPLEMENTATIONS =============
    ;; These procedures require more complex implementations for full SRFI-135 compliance
    
    ;; Text searching (simplified implementations)
    (define (text-index txt pred . args)
      "Find index of first character satisfying predicate."
      (unless (text? txt)
        (error "text-index: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (let loop ((i start))
          (cond
            ((>= i end) #f)
            ((pred (text-ref txt i)) i)
            (else (loop (+ i 1)))))))
    
    (define (text-index-right txt pred . args)
      "Find index of last character satisfying predicate."
      (unless (text? txt)
        (error "text-index-right: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (let loop ((i (- end 1)))
          (cond
            ((< i start) #f)
            ((pred (text-ref txt i)) i)
            (else (loop (- i 1)))))))
    
    (define (text-skip txt pred . args)
      "Find index of first character not satisfying predicate."
      (apply text-index txt (lambda (c) (not (pred c))) args))
    
    (define (text-skip-right txt pred . args)
      "Find index of last character not satisfying predicate."
      (apply text-index-right txt (lambda (c) (not (pred c))) args))
    
    (define (text-contains txt pattern . args)
      "Test if text contains pattern."
      (unless (text? txt)
        (error "text-contains: first argument not a text" txt))
      (unless (text? pattern)
        (error "text-contains: second argument not a text" pattern))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt)))
             (txt-str (text->string txt))
             (pat-str (text->string pattern)))
        (let ((index (string-contains txt-str pat-str start end)))
          (and index (>= index start) (<= index end) index))))
    
    (define (text-contains-right txt pattern . args)
      "Test if text contains pattern (search from right)."
      (unless (text? txt)
        (error "text-contains-right: first argument not a text" txt))
      (unless (text? pattern)
        (error "text-contains-right: second argument not a text" pattern))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt)))
             (txt-str (text->string txt))
             (pat-str (text->string pattern)))
        ;; Simple implementation - find all occurrences and return last
        (let loop ((pos start) (last-match #f))
          (let ((next (string-contains txt-str pat-str pos end)))
            (if next
                (loop (+ next 1) next)
                last-match)))))
    
    ;; Additional procedures would be implemented here for full SRFI-135 compliance:
    ;; - text-concatenate, text-concatenate-reverse, text-join
    ;; - text-map, text-map-index, text-for-each, text-for-each-index
    ;; - text-count, text-filter, text-remove
    ;; - text-segment, text-split
    ;; - text-pad, text-pad-right, text-trim, text-trim-right, text-trim-both
    ;; - text-replace
    
    ;; For now, we provide basic placeholder implementations
    (define (text-concatenate texts)
      "Concatenate a list of texts."
      (apply text-append texts))
    
    (define (text-concatenate-reverse texts . args)
      "Concatenate texts in reverse order."
      (text-concatenate (reverse texts)))
    
    (define (text-join texts delimiter . args)
      "Join texts with delimiter."
      (if (null? texts)
          (string->text "")
          (let loop ((result (car texts))
                     (rest (cdr texts)))
            (if (null? rest)
                result
                (loop (text-append result delimiter (car rest))
                      (cdr rest))))))
    
    (define (text-map proc txt . args)
      "Map procedure over characters in text."
      (unless (text? txt)
        (error "text-map: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt)))
             (chars (text->list txt start end))
             (mapped-chars (map proc chars)))
        (list->text mapped-chars)))
    
    (define (text-for-each proc txt . args)
      "Apply procedure to each character in text."
      (unless (text? txt)
        (error "text-for-each: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (do ((i start (+ i 1)))
            ((>= i end))
          (proc (text-ref txt i)))))
    
    (define (text-count pred txt . args)
      "Count characters satisfying predicate."
      (unless (text? txt)
        (error "text-count: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt))))
        (let loop ((i start) (count 0))
          (if (>= i end)
              count
              (loop (+ i 1) 
                    (if (pred (text-ref txt i)) (+ count 1) count))))))
    
    (define (text-filter pred txt . args)
      "Filter characters satisfying predicate."
      (unless (text? txt)
        (error "text-filter: not a text" txt))
      (let* ((start (if (pair? args) (car args) 0))
             (end (if (and (pair? args) (pair? (cdr args))) 
                      (cadr args) (text-length txt)))
             (chars (text->list txt start end))
             (filtered-chars (filter pred chars)))
        (list->text filtered-chars)))
    
    (define (text-remove pred txt . args)
      "Remove characters satisfying predicate."
      (apply text-filter (lambda (c) (not (pred c))) txt args))
    
    ;; ============= USAGE EXAMPLES =============
    
    ;; Example 1: Basic text creation and manipulation
    ;; (define t1 (text #\h #\e #\l #\l #\o))
    ;; (define t2 (string->text " world"))
    ;; (define greeting (text-append t1 t2))
    ;; (text->string greeting)  ; => "hello world"
    
    ;; Example 2: Text comparison and searching
    ;; (text=? (string->text "hello") (string->text "hello"))  ; => #t
    ;; (text-prefix? (string->text "hell") (string->text "hello"))  ; => #t
    ;; (text-contains (string->text "hello world") (string->text "world"))  ; => 6
    
    ;; Example 3: Case conversion
    ;; (text->string (text-upcase (string->text "hello")))  ; => "HELLO"
    ;; (text->string (text-titlecase (string->text "hello world")))  ; => "Hello World"
    
    ;; Example 4: Subtext operations
    ;; (define txt (string->text "hello world"))
    ;; (text->string (text-take txt 5))  ; => "hello"
    ;; (text->string (text-drop txt 6))  ; => "world"
    ;; (text->string (subtext txt 2 7))  ; => "llo w"
    
    ;; Example 5: Text transformation
    ;; (define txt (string->text "hello"))
    ;; (text->string (text-reverse txt))  ; => "olleh"
    ;; (text->string (text-map char-upcase txt))  ; => "HELLO"
    
    ;; ============= PERFORMANCE NOTES =============
    
    ;; This implementation provides basic SRFI-135 functionality using
    ;; underlying string operations. For production use, a more sophisticated
    ;; implementation would use:
    ;; 
    ;; - Rope data structures for efficient concatenation
    ;; - Copy-on-write semantics for memory efficiency  
    ;; - Unicode normalization and proper locale support
    ;; - Efficient search algorithms (Boyer-Moore, KMP)
    ;; - Lazy evaluation for large texts
    
    ;; Current complexity characteristics:
    ;; - text-append: O(n) where n is total length
    ;; - subtext: O(k) where k is substring length
    ;; - text-length: O(1)
    ;; - text-ref: O(1)
    ;; - text=?: O(min(n,m)) where n,m are text lengths
    ;; - text-contains: O(n*m) naive search
    
    ))