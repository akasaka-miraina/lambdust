;; R7RS Standard Library - Character Module
;; Provides extended character functionality

(define-library (scheme char)
  (export 
    ;; Character classification
    char-alphabetic? char-numeric? char-whitespace?
    char-upper-case? char-lower-case? char-title-case?
    char-general-category
    
    ;; Character conversion
    char-upcase char-downcase char-titlecase char-foldcase
    
    ;; String case conversion
    string-upcase string-downcase string-titlecase string-foldcase
    
    ;; Character sets
    char-set? char-set-contains? char-set-size char-set-count
    char-set-adjoin char-set-delete char-set-union char-set-intersection
    char-set-difference char-set-complement
    char-set:lower-case char-set:upper-case char-set:title-case
    char-set:letter char-set:digit char-set:letter+digit
    char-set:graphic char-set:printing char-set:whitespace
    char-set:iso-control char-set:punctuation char-set:symbol
    char-set:hex-digit char-set:blank char-set:ascii
    char-set:empty char-set:full)

  (begin

  ;; ============= Extended Character Classification =============

  (define (char-title-case? char)
    "Returns #t if char is a titlecase character."
    (char-title-case? char))

  (define (char-general-category char)
    "Returns the Unicode general category of char."
    (char-general-category char))

  ;; ============= Character Case Conversion =============

  (define (char-titlecase char)
    "Returns the titlecase version of char."
    (char-titlecase char))

  ;; ============= String Case Conversion =============

  (define (string-upcase str)
    "Returns str with all characters in uppercase."
    (string-map char-upcase str))

  (define (string-downcase str)
    "Returns str with all characters in lowercase."
    (string-map char-downcase str))

  (define (string-titlecase str)
    "Returns str with titlecase capitalization."
    (if (string=? str "")
        ""
        (string-append 
          (string (char-titlecase (string-ref str 0)))
          (string-downcase (substring str 1)))))

  (define (string-foldcase str)
    "Returns str with case-folded characters."
    (string-map char-foldcase str))

  ;; ============= Character Sets =============
  ;; Basic character set implementation

  (define (char-set? obj)
    "Returns #t if obj is a character set."
    (builtin:char-set? obj))

  (define (char-set-contains? char-set char)
    "Returns #t if char-set contains char."
    (builtin:char-set-contains? char-set char))

  (define (char-set-size char-set)
    "Returns the number of characters in char-set."
    (builtin:char-set-size char-set))

  (define (char-set-count pred char-set)
    "Returns the number of characters in char-set satisfying pred."
    (builtin:char-set-count pred char-set))

  (define (char-set-adjoin char-set . chars)
    "Returns char-set with chars added."
    (builtin:char-set-adjoin char-set chars))

  (define (char-set-delete char-set . chars)
    "Returns char-set with chars removed."
    (builtin:char-set-delete char-set chars))

  (define (char-set-union . char-sets)
    "Returns the union of char-sets."
    (builtin:char-set-union char-sets))

  (define (char-set-intersection . char-sets)
    "Returns the intersection of char-sets."
    (builtin:char-set-intersection char-sets))

  (define (char-set-difference char-set1 . char-sets)
    "Returns char-set1 with elements of char-sets removed."
    (builtin:char-set-difference char-set1 char-sets))

  (define (char-set-complement char-set)
    "Returns the complement of char-set."
    (builtin:char-set-complement char-set))

  ;; ============= Predefined Character Sets =============

  (define char-set:lower-case
    "Character set of lowercase letters."
    (builtin:make-char-set-from-predicate char-lower-case?))

  (define char-set:upper-case
    "Character set of uppercase letters."
    (builtin:make-char-set-from-predicate char-upper-case?))

  (define char-set:title-case
    "Character set of titlecase letters."
    (builtin:make-char-set-from-predicate char-title-case?))

  (define char-set:letter
    "Character set of alphabetic characters."
    (builtin:make-char-set-from-predicate char-alphabetic?))

  (define char-set:digit
    "Character set of numeric digits."
    (builtin:make-char-set-from-predicate char-numeric?))

  (define char-set:letter+digit
    "Character set of letters and digits."
    (char-set-union char-set:letter char-set:digit))

  (define char-set:graphic
    "Character set of graphic characters."
    (builtin:make-char-set-from-predicate 
      (lambda (c) (and (not (char-whitespace? c)) (not (char-iso-control? c))))))

  (define char-set:printing
    "Character set of printing characters."
    (char-set-union char-set:graphic 
                    (builtin:make-char-set-from-chars (list #\space))))

  (define char-set:whitespace
    "Character set of whitespace characters."
    (builtin:make-char-set-from-predicate char-whitespace?))

  (define char-set:iso-control
    "Character set of ISO control characters."
    (builtin:make-char-set-from-predicate char-iso-control?))

  (define char-set:punctuation
    "Character set of punctuation characters."
    (builtin:make-char-set-from-predicate char-punctuation?))

  (define char-set:symbol
    "Character set of symbol characters."
    (builtin:make-char-set-from-predicate char-symbol?))

  (define char-set:hex-digit
    "Character set of hexadecimal digits."
    (builtin:make-char-set-from-predicate char-hex-digit?))

  (define char-set:blank
    "Character set of blank characters."
    (builtin:make-char-set-from-predicate char-blank?))

  (define char-set:ascii
    "Character set of ASCII characters."
    (builtin:make-char-set-from-predicate char-ascii?))

  (define char-set:empty
    "Empty character set."
    (builtin:make-empty-char-set))

  (define char-set:full
    "Character set containing all characters."
    (char-set-complement char-set:empty))

  ;; ============= Additional Character Set Operations =============

  (define (char-set . chars)
    "Creates a character set from the given characters."
    (builtin:make-char-set-from-chars chars))

  (define (list->char-set chars . base-cs)
    "Creates a character set from a list of characters."
    (let ((base (if (null? base-cs) char-set:empty (car base-cs))))
      (char-set-union base (apply char-set chars))))

  (define (string->char-set str . base-cs)
    "Creates a character set from a string."
    (apply list->char-set (string->list str) base-cs))

  (define (char-set-filter pred char-set)
    "Returns characters in char-set satisfying pred."
    (builtin:char-set-filter pred char-set))

  (define (char-set->list char-set)
    "Returns a list of characters in char-set."
    (builtin:char-set->list char-set))

  (define (char-set->string char-set)
    "Returns a string of characters in char-set."
    (list->string (char-set->list char-set)))

  (define (char-set-every pred char-set)
    "Returns #t if pred is true for every character in char-set."
    (builtin:char-set-every pred char-set))

  (define (char-set-any pred char-set)
    "Returns #t if pred is true for any character in char-set."
    (builtin:char-set-any pred char-set))

  ;; ============= Character Set Cursors =============
  ;; Simple cursor implementation for character set iteration

  (define (char-set-cursor char-set)
    "Returns a cursor for char-set."
    (char-set->list char-set))

  (define (char-set-ref char-set cursor)
    "Returns the character at cursor position."
    (if (null? cursor)
        (error "char-set-ref: cursor at end")
        (car cursor)))

  (define (char-set-cursor-next char-set cursor)
    "Advances cursor to next position."
    (if (null? cursor)
        cursor
        (cdr cursor)))

  (define (end-of-char-set? cursor)
    "Returns #t if cursor is at end."
    (null? cursor))

  (define (char-set-fold kons knil char-set)
    "Folds over character set."
    (let ((cursor (char-set-cursor char-set)))
      (char-set-fold-helper kons knil char-set cursor)))

  (define (char-set-fold-helper kons knil char-set cursor)
    "Helper for char-set-fold."
    (if (end-of-char-set? cursor)
        knil
        (kons (char-set-ref char-set cursor)
              (char-set-fold-helper kons knil char-set 
                                   (char-set-cursor-next char-set cursor)))))

  (define (char-set-unfold p f g seed . base-cs)
    "Unfolds a character set."
    (let ((base (if (null? base-cs) char-set:empty (car base-cs))))
      (char-set-unfold-helper p f g seed base)))

  (define (char-set-unfold-helper p f g seed base)
    "Helper for char-set-unfold."
    (if (p seed)
        base
        (char-set-adjoin (char-set-unfold-helper p f g (g seed) base) (f seed))))

  (define (char-set-for-each proc char-set)
    "Applies proc to each character in char-set."
    (char-set-fold (lambda (char acc) (proc char) acc) #f char-set))

  (define (char-set-map proc char-set)
    "Returns a new char-set with proc applied to each character."
    (char-set-fold (lambda (char acc) 
                     (char-set-adjoin acc (proc char)))
                   char-set:empty 
                   char-set)))))