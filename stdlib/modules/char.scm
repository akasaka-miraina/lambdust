;; Lambdust Built-in Character Module
;; Provides comprehensive R7RS-compliant character manipulation functions

(define-module (:: char)
  (metadata
    (version "1.0.0")
    (description "Comprehensive character manipulation operations")
    (author "Lambdust Core Team"))
  
  (export 
    ;; Core character operations
    char? char=? char<? char>? char<=? char>=?
    char-ci=? char-ci<? char-ci>? char-ci<=? char-ci>=?
    
    ;; Character classification
    char-alphabetic? char-numeric? char-whitespace?
    char-upper-case? char-lower-case? char-title-case?
    char-general-category
    
    ;; Character conversion
    char-upcase char-downcase char-titlecase char-foldcase
    
    ;; Character/integer conversion
    char->integer integer->char
    
    ;; Character/string conversion
    char->string string->char
    
    ;; Additional character utilities
    char-ascii? char-control? char-graphic? char-print?
    char-punctuation? char-symbol? char-hex-digit?
    char-blank? char-iso-control?)

  ;; ============= Core Character Operations =============

  (define (char? obj)
    "Returns #t if obj is a character, #f otherwise."
    (builtin:char? obj))

  ;; Character comparison
  (define (char=? char1 char2 . chars)
    "Returns #t if all characters are equal."
    (if (null? chars)
        (builtin:char=? char1 char2)
        (and (builtin:char=? char1 char2)
             (apply char=? char2 chars))))

  (define (char<? char1 char2 . chars)
    "Returns #t if characters are in strictly increasing order."
    (if (null? chars)
        (builtin:char<? char1 char2)
        (and (builtin:char<? char1 char2)
             (apply char<? char2 chars))))

  (define (char>? char1 char2 . chars)
    "Returns #t if characters are in strictly decreasing order."
    (if (null? chars)
        (builtin:char>? char1 char2)
        (and (builtin:char>? char1 char2)
             (apply char>? char2 chars))))

  (define (char<=? char1 char2 . chars)
    "Returns #t if characters are in non-decreasing order."
    (if (null? chars)
        (builtin:char<=? char1 char2)
        (and (builtin:char<=? char1 char2)
             (apply char<=? char2 chars))))

  (define (char>=? char1 char2 . chars)
    "Returns #t if characters are in non-increasing order."
    (if (null? chars)
        (builtin:char>=? char1 char2)
        (and (builtin:char>=? char1 char2)
             (apply char>=? char2 chars))))

  ;; Case-insensitive character comparison
  (define (char-ci=? char1 char2 . chars)
    "Case-insensitive character equality."
    (apply char=? (char-foldcase char1) (char-foldcase char2)
           (map char-foldcase chars)))

  (define (char-ci<? char1 char2 . chars)
    "Case-insensitive character less-than."
    (apply char<? (char-foldcase char1) (char-foldcase char2)
           (map char-foldcase chars)))

  (define (char-ci>? char1 char2 . chars)
    "Case-insensitive character greater-than."
    (apply char>? (char-foldcase char1) (char-foldcase char2)
           (map char-foldcase chars)))

  (define (char-ci<=? char1 char2 . chars)
    "Case-insensitive character less-than-or-equal."
    (apply char<=? (char-foldcase char1) (char-foldcase char2)
           (map char-foldcase chars)))

  (define (char-ci>=? char1 char2 . chars)
    "Case-insensitive character greater-than-or-equal."
    (apply char>=? (char-foldcase char1) (char-foldcase char2)
           (map char-foldcase chars)))

  ;; ============= Character Classification =============

  (define (char-alphabetic? char)
    "Returns #t if char is alphabetic."
    (builtin:char-alphabetic? char))

  (define (char-numeric? char)
    "Returns #t if char is numeric."
    (builtin:char-numeric? char))

  (define (char-whitespace? char)
    "Returns #t if char is whitespace."
    (builtin:char-whitespace? char))

  (define (char-upper-case? char)
    "Returns #t if char is uppercase."
    (builtin:char-upper-case? char))

  (define (char-lower-case? char)
    "Returns #t if char is lowercase."
    (builtin:char-lower-case? char))

  (define (char-title-case? char)
    "Returns #t if char is titlecase."
    (builtin:char-title-case? char))

  (define (char-general-category char)
    "Returns the general category of char."
    (builtin:char-general-category char))

  ;; ============= Character Conversion =============

  (define (char-upcase char)
    "Returns the uppercase version of char."
    (builtin:char-upcase char))

  (define (char-downcase char)
    "Returns the lowercase version of char."
    (builtin:char-downcase char))

  (define (char-titlecase char)
    "Returns the titlecase version of char."
    (builtin:char-titlecase char))

  (define (char-foldcase char)
    "Returns the case-folded version of char."
    (builtin:char-foldcase char))

  ;; ============= Character/Integer Conversion =============

  (define (char->integer char)
    "Returns the integer code for char."
    (builtin:char->integer char))

  (define (integer->char n)
    "Returns the character with integer code n."
    (builtin:integer->char n))

  ;; ============= Character/String Conversion =============

  (define (char->string char)
    "Returns a string containing only char."
    (string char))

  (define (string->char str)
    "Returns the first character of str if str has length 1."
    (if (= (string-length str) 1)
        (string-ref str 0)
        (error "string->char: string must have length 1" str)))

  ;; ============= Additional Character Utilities =============

  (define (char-ascii? char)
    "Returns #t if char is an ASCII character."
    (let ((code (char->integer char)))
      (and (>= code 0) (<= code 127))))

  (define (char-control? char)
    "Returns #t if char is a control character."
    (let ((code (char->integer char)))
      (or (and (>= code 0) (<= code 31))
          (= code 127))))

  (define (char-graphic? char)
    "Returns #t if char is a graphic character."
    (and (not (char-control? char))
         (not (char-whitespace? char))))

  (define (char-print? char)
    "Returns #t if char is a printing character."
    (or (char-graphic? char)
        (char=? char #\space)))

  (define (char-punctuation? char)
    "Returns #t if char is a punctuation character."
    (and (char-graphic? char)
         (not (char-alphabetic? char))
         (not (char-numeric? char))))

  (define (char-symbol? char)
    "Returns #t if char is a symbol character."
    (and (char-graphic? char)
         (not (char-alphabetic? char))
         (not (char-numeric? char))
         (not (char-punctuation? char))))

  (define (char-hex-digit? char)
    "Returns #t if char is a hexadecimal digit."
    (or (char-numeric? char)
        (and (char-ci>=? char #\a) (char-ci<=? char #\f))))

  (define (char-blank? char)
    "Returns #t if char is a blank character (space or tab)."
    (or (char=? char #\space)
        (char=? char #\tab)))

  (define (char-iso-control? char)
    "Returns #t if char is an ISO control character."
    (let ((code (char->integer char)))
      (or (and (>= code 0) (<= code 31))
          (and (>= code 127) (<= code 159)))))

  ;; ============= Character Constants =============

  ;; Common character constants
  (define char:space #\space)
  (define char:tab #\tab)
  (define char:newline #\newline)
  (define char:return #\return)
  (define char:null #\null)
  (define char:alarm #\alarm)
  (define char:backspace #\backspace)
  (define char:escape #\escape)
  (define char:delete #\delete)
  (define char:vtab #\vtab)
  (define char:page #\page)

  ;; ASCII digit characters
  (define char:0 #\0)
  (define char:1 #\1)
  (define char:2 #\2)
  (define char:3 #\3)
  (define char:4 #\4)
  (define char:5 #\5)
  (define char:6 #\6)
  (define char:7 #\7)
  (define char:8 #\8)
  (define char:9 #\9)

  ;; ASCII letter characters (uppercase)
  (define char:A #\A)
  (define char:B #\B)
  (define char:C #\C)
  (define char:D #\D)
  (define char:E #\E)
  (define char:F #\F)
  (define char:G #\G)
  (define char:H #\H)
  (define char:I #\I)
  (define char:J #\J)
  (define char:K #\K)
  (define char:L #\L)
  (define char:M #\M)
  (define char:N #\N)
  (define char:O #\O)
  (define char:P #\P)
  (define char:Q #\Q)
  (define char:R #\R)
  (define char:S #\S)
  (define char:T #\T)
  (define char:U #\U)
  (define char:V #\V)
  (define char:W #\W)
  (define char:X #\X)
  (define char:Y #\Y)
  (define char:Z #\Z)

  ;; ASCII letter characters (lowercase)
  (define char:a #\a)
  (define char:b #\b)
  (define char:c #\c)
  (define char:d #\d)
  (define char:e #\e)
  (define char:f #\f)
  (define char:g #\g)
  (define char:h #\h)
  (define char:i #\i)
  (define char:j #\j)
  (define char:k #\k)
  (define char:l #\l)
  (define char:m #\m)
  (define char:n #\n)
  (define char:o #\o)
  (define char:p #\p)
  (define char:q #\q)
  (define char:r #\r)
  (define char:s #\s)
  (define char:t #\t)
  (define char:u #\u)
  (define char:v #\v)
  (define char:w #\w)
  (define char:x #\x)
  (define char:y #\y)
  (define char:z #\z))