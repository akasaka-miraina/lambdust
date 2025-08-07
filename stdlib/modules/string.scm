;; Lambdust R7RS-Compliant String Module
;; Provides complete R7RS Section 6.7 string manipulation functions

(define-module (:: string)
  (metadata
    (version "2.0.0")
    (description "R7RS-compliant string manipulation operations")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.7"))
  
  (export 
    ;; === R7RS Section 6.7 String Procedures ===
    
    ;; Core string operations
    string? string-length string-ref string-set!
    
    ;; String comparison (case-sensitive)
    string=? string<? string>? string<=? string>=?
    
    ;; String comparison (case-insensitive)
    string-ci=? string-ci<? string-ci>? string-ci<=? string-ci>=?
    
    ;; String construction and manipulation
    make-string string substring string-append
    string-copy string-copy! string-fill!
    
    ;; String/list conversions
    string->list list->string
    
    ;; String iteration (higher-order functions)
    string-for-each string-map
    
    ;; Case conversion
    string-upcase string-downcase string-foldcase
    
    ;; === Additional Lambdust Extensions ===
    ;; (Beyond R7RS but commonly useful)
    
    ;; String trimming
    string-trim string-trim-left string-trim-right
    
    ;; String splitting and joining
    string-split string-join
    
    ;; String searching
    string-contains? string-prefix? string-suffix?
    
    ;; String replacement and reversal
    string-replace string-reverse)

  ;; ============= R7RS Core String Operations =============

  (define (string? obj)
    "Returns #t if obj is a string, #f otherwise.
     
     R7RS: (string? obj) procedure
     The string? predicate returns #t if obj is a string, and otherwise returns #f."
    (builtin:string? obj))

  (define (string-length str)
    "Returns the number of characters in str.
     
     R7RS: (string-length string) procedure
     Returns the number of characters in the given string."
    (builtin:string-length str))

  (define (string-ref str k)
    "Returns character k of str using zero-origin indexing.
     
     R7RS: (string-ref string k) procedure
     The string-ref procedure returns character k of string using zero-origin indexing.
     It is an error if k is not a valid index of string."
    (builtin:string-ref str k))

  (define (string-set! str k char)
    "Stores char in element k of str.
     
     R7RS: (string-set! string k char) procedure
     The string-set! procedure stores char in element k of string.
     It is an error if k is not a valid index of string."
    (builtin:string-set! str k char))

  ;; ============= R7RS String Comparison =============

  (define (string=? str1 str2 . strings)
    "Returns #t if all strings are equal (lexicographic).
     
     R7RS: (string=? string1 string2 string3 ...) procedure
     These procedures return #t if their arguments are (respectively):
     equal strings."
    (if (null? strings)
        (builtin:string=? str1 str2)
        (and (builtin:string=? str1 str2)
             (apply string=? str2 strings))))

  (define (string<? str1 str2 . strings)
    "Returns #t if strings are in strictly increasing lexicographic order.
     
     R7RS: (string<? string1 string2 string3 ...) procedure"
    (if (null? strings)
        (builtin:string<? str1 str2)
        (and (builtin:string<? str1 str2)
             (apply string<? str2 strings))))

  (define (string>? str1 str2 . strings)
    "Returns #t if strings are in strictly decreasing lexicographic order.
     
     R7RS: (string>? string1 string2 string3 ...) procedure"
    (if (null? strings)
        (builtin:string>? str1 str2)
        (and (builtin:string>? str1 str2)
             (apply string>? str2 strings))))

  (define (string<=? str1 str2 . strings)
    "Returns #t if strings are in non-decreasing lexicographic order.
     
     R7RS: (string<=? string1 string2 string3 ...) procedure"
    (if (null? strings)
        (builtin:string<=? str1 str2)
        (and (builtin:string<=? str1 str2)
             (apply string<=? str2 strings))))

  (define (string>=? str1 str2 . strings)
    "Returns #t if strings are in non-increasing lexicographic order.
     
     R7RS: (string>=? string1 string2 string3 ...) procedure"
    (if (null? strings)
        (builtin:string>=? str1 str2)
        (and (builtin:string>=? str1 str2)
             (apply string>=? str2 strings))))

  ;; === R7RS Case-Insensitive String Comparison ===
  
  (define (string-ci=? str1 str2 . strings)
    "Case-insensitive string equality comparison.
     
     R7RS: (string-ci=? string1 string2 string3 ...) procedure
     These procedures are similar to string=?, string<?, etc., but they treat
     upper and lower case letters as the same."
    (apply string=? (string-foldcase str1) (string-foldcase str2)
           (map string-foldcase strings)))

  (define (string-ci<? str1 str2 . strings)
    "Case-insensitive string less-than comparison.
     
     R7RS: (string-ci<? string1 string2 string3 ...) procedure"
    (apply string<? (string-foldcase str1) (string-foldcase str2)
           (map string-foldcase strings)))

  (define (string-ci>? str1 str2 . strings)
    "Case-insensitive string greater-than comparison.
     
     R7RS: (string-ci>? string1 string2 string3 ...) procedure"
    (apply string>? (string-foldcase str1) (string-foldcase str2)
           (map string-foldcase strings)))

  (define (string-ci<=? str1 str2 . strings)
    "Case-insensitive string less-than-or-equal comparison.
     
     R7RS: (string-ci<=? string1 string2 string3 ...) procedure"
    (apply string<=? (string-foldcase str1) (string-foldcase str2)
           (map string-foldcase strings)))

  (define (string-ci>=? str1 str2 . strings)
    "Case-insensitive string greater-than-or-equal comparison.
     
     R7RS: (string-ci>=? string1 string2 string3 ...) procedure"
    (apply string>=? (string-foldcase str1) (string-foldcase str2)
           (map string-foldcase strings)))

  ;; ============= R7RS String Construction =============

  (define (make-string k . char-opt)
    "Returns a newly allocated string of length k.
     
     R7RS: (make-string k) procedure
           (make-string k char) procedure
     The make-string procedure returns a newly allocated string of length k.
     If char is given, then all elements of the string are initialized to char,
     otherwise the contents of the string are unspecified."
    (let ((char (if (null? char-opt) #\space (car char-opt))))
      (builtin:make-string k char)))

  (define (string . chars)
    "Returns a string composed of the given character arguments.
     
     R7RS: (string char ...) procedure
     The string procedure returns a newly allocated string composed of the arguments.
     It is analogous to list."
    (list->string chars))

  (define (substring str start end)
    "Returns a substring of str from start to end.
     
     R7RS: (substring string start end) procedure
     The substring procedure returns a newly allocated string formed from the
     characters of string beginning with index start and ending with index end.
     This is equivalent to calling string-copy with the same arguments."
    (builtin:substring str start end))

  (define (string-append . strings)
    "Returns a string formed by concatenating the given strings.
     
     R7RS: (string-append string ...) procedure
     Returns a newly allocated string whose characters are the concatenation
     of the characters in the given strings."
    (if (null? strings)
        ""
        (builtin:string-append-list strings)))

  (define (string-copy str . start-end)
    "Returns a copy of the given string.
     
     R7RS: (string-copy string) procedure
           (string-copy string start) procedure  
           (string-copy string start end) procedure
     Returns a newly allocated copy of the part of the given string between
     start and end."
    (if (null? start-end)
        (builtin:string-copy str)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (string-length str))))
          (substring str start end))))

  (define (string-copy! to at from . start-end)
    "Copies characters from source string to destination string.
     
     R7RS: (string-copy! to at from) procedure
           (string-copy! to at from start) procedure
           (string-copy! to at from start end) procedure
     Copies the characters of string from between start and end to string to,
     starting at at."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (string-length from)
                   (cadr start-end))))
      (builtin:string-copy! to at from start end)))

  (define (string-fill! str char . start-end)
    "Fills a string with the given character.
     
     R7RS: (string-fill! string fill) procedure
           (string-fill! string fill start) procedure
           (string-fill! string fill start end) procedure
     Stores char in every element of the string between start and end."
    (let ((start (if (null? start-end) 0 (car start-end)))
          (end (if (or (null? start-end) (< (length start-end) 2))
                   (string-length str)
                   (cadr start-end))))
      (builtin:string-fill! str char start end)))

  ;; ============= R7RS String/List Conversions =============

  (define (string->list str . start-end)
    "Returns a list of the characters of the given string.
     
     R7RS: (string->list string) procedure
           (string->list string start) procedure
           (string->list string start end) procedure
     The string->list procedure returns a newly allocated list of the
     characters of string between start and end."
    (if (null? start-end)
        (builtin:string->list str)
        (let ((start (car start-end))
              (end (if (> (length start-end) 1) 
                       (cadr start-end) 
                       (string-length str))))
          (builtin:string->list (substring str start end)))))

  (define (list->string chars)
    "Returns a string formed from the given list of characters.
     
     R7RS: (list->string list) procedure
     The list->string procedure returns a newly allocated string formed
     from the elements in the list list. It is an error if any element of list
     is not a character."
    (builtin:list->string chars))

  ;; ============= R7RS String Iteration (Higher-Order Functions) =============

  (define (string-for-each proc str . strings)
    "Applies procedure to the characters of the given strings.
     
     R7RS: (string-for-each proc string1 string2 ...) procedure
     The string-for-each procedure applies proc element-wise to the characters
     of the strings for its side effects, in order from the first character(s)
     to the last. The proc procedure is always called with the same number
     of arguments as there are strings."
    (define (string-for-each-helper strings index)
      (when (< index (apply min (map string-length strings)))
        (apply proc (map (lambda (s) (string-ref s index)) strings))
        (string-for-each-helper strings (+ index 1))))
    (string-for-each-helper (cons str strings) 0))

  (define (string-map proc str . strings)
    "Returns a string formed by applying procedure to the characters of the given strings.
     
     R7RS: (string-map proc string1 string2 ...) procedure  
     The string-map procedure applies proc element-wise to the characters of the
     strings and returns a string of the results, in order. The proc procedure
     is always called with the same number of arguments as there are strings."
    (define (string-map-helper strings index acc)
      (if (< index (apply min (map string-length strings)))
          (let ((result (apply proc (map (lambda (s) (string-ref s index)) strings))))
            (string-map-helper strings (+ index 1) (cons result acc)))
          (reverse acc)))
    (list->string (string-map-helper (cons str strings) 0 '())))

  ;; ============= R7RS Case Conversion =============

  (define (string-upcase str)
    "Returns a string with all characters converted to uppercase.
     
     R7RS: (string-upcase string) procedure
     The string-upcase procedure returns a newly allocated string containing
     the characters of string, but with all lowercase characters replaced by
     their uppercase equivalents."
    (builtin:string-upcase str))

  (define (string-downcase str)
    "Returns a string with all characters converted to lowercase.
     
     R7RS: (string-downcase string) procedure  
     The string-downcase procedure returns a newly allocated string containing
     the characters of string, but with all uppercase characters replaced by
     their lowercase equivalents."
    (builtin:string-downcase str))

  (define (string-foldcase str)
    "Returns a string with case-folded characters for case-insensitive comparison.
     
     R7RS: (string-foldcase string) procedure
     The string-foldcase procedure applies the Unicode simple case-folding
     algorithm to the characters of string and returns the result."
    (builtin:string-foldcase str))

  ;; ============= Additional Lambdust String Utilities =============
  ;; (Extensions beyond R7RS for practical use)

  (define (string-trim str . char-set)
    "Removes whitespace or specified characters from both ends.
     
     Extension: Trims both leading and trailing whitespace characters."
    (builtin:string-trim str))

  (define (string-trim-left str . char-set)
    "Removes whitespace or specified characters from the left end.
     
     Extension: Trims leading whitespace characters."
    (builtin:string-trim-left str))

  (define (string-trim-right str . char-set)
    "Removes whitespace or specified characters from the right end.
     
     Extension: Trims trailing whitespace characters."
    (builtin:string-trim-right str))

  (define (string-split str delimiter)
    "Splits string by delimiter, returning a list of strings.
     
     Extension: Returns a list of strings split by the delimiter."
    (builtin:string-split str delimiter))

  (define (string-join strings . delimiter)
    "Joins list of strings with optional delimiter.
     
     Extension: Concatenates strings with the given delimiter (default empty)."
    (let ((delim (if (null? delimiter) "" (car delimiter))))
      (builtin:string-join strings delim)))

  (define (string-contains? str substring)
    "Returns #t if string contains the given substring.
     
     Extension: Substring search operation."
    (builtin:string-contains? str substring))

  (define (string-prefix? prefix str)
    "Returns #t if string starts with the given prefix.
     
     Extension: Prefix matching operation."
    (and (>= (string-length str) (string-length prefix))
         (string=? prefix (substring str 0 (string-length prefix)))))

  (define (string-suffix? suffix str)
    "Returns #t if string ends with the given suffix.
     
     Extension: Suffix matching operation."
    (let ((str-len (string-length str))
          (suffix-len (string-length suffix)))
      (and (>= str-len suffix-len)
           (string=? suffix (substring str (- str-len suffix-len) str-len)))))

  (define (string-replace str old new)
    "Replaces all occurrences of old substring with new substring.
     
     Extension: Global string replacement operation."
    (builtin:string-replace str old new))

  (define (string-reverse str)
    "Returns a string with characters in reverse order.
     
     Extension: String reversal operation."
    (list->string (reverse (string->list str)))))