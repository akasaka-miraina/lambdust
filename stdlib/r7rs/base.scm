;; R7RS Standard Library - Base Module
;; Provides the core R7RS-small language functionality

(define-library (scheme base)
  (export 
    ;; Boolean operations
    not boolean? boolean=?
    
    ;; Equivalence predicates
    eqv? eq? equal?
    
    ;; Numbers
    number? complex? real? rational? integer? exact? inexact?
    exact-integer? finite? infinite? nan?
    = < > <= >= zero? positive? negative? odd? even?
    max min + * - / abs quotient remainder modulo
    gcd lcm numerator denominator floor ceiling truncate round
    rationalize square sqrt exact-integer-sqrt expt
    make-rectangular make-polar real-part imag-part magnitude angle
    exact inexact exact->inexact inexact->exact
    number->string string->number
    
    ;; Characters
    char? char=? char<? char>? char<=? char>=?
    char-ci=? char-ci<? char-ci>? char-ci<=? char-ci>=?
    char-alphabetic? char-numeric? char-whitespace?
    char-upper-case? char-lower-case?
    char->integer integer->char char-upcase char-downcase
    char-foldcase
    
    ;; Strings
    string? make-string string string-length string-ref string-set!
    string=? string<? string>? string<=? string>=?
    string-ci=? string-ci<? string-ci>? string-ci<=? string-ci>=?
    substring string-append string-copy string-copy! string-fill!
    string->list list->string string-for-each string-map
    
    ;; Vectors
    vector? make-vector vector vector-length vector-ref vector-set!
    vector-copy vector-copy! vector-append vector-fill!
    vector->list list->vector vector-for-each vector-map
    
    ;; Bytevectors  
    bytevector? make-bytevector bytevector bytevector-length
    bytevector-u8-ref bytevector-u8-set! bytevector-copy
    bytevector-copy! bytevector-append utf8->string string->utf8
    
    ;; Lists and pairs
    pair? cons car cdr set-car! set-cdr!
    caar cadr cdar cddr null? list? make-list list length append reverse
    list-tail list-ref list-set! list-copy
    memq memv member assq assv assoc
    
    ;; Symbols
    symbol? symbol=? symbol->string string->symbol
    
    ;; Control features
    procedure? apply map string-map vector-map for-each string-for-each vector-for-each
    call-with-current-continuation call/cc values call-with-values dynamic-wind
    
    ;; Exceptions
    error error-object? error-object-message error-object-irritants
    raise raise-continuable with-exception-handler guard
    
    ;; Environments and evaluation
    environment scheme-report-environment null-environment
    eval
    
    ;; Input and output
    input-port? output-port? textual-port? binary-port? port?
    current-input-port current-output-port current-error-port
    close-port close-input-port close-output-port
    open-input-string open-output-string get-output-string
    open-input-bytevector open-output-bytevector get-output-bytevector
    read read-char peek-char read-line char-ready?
    read-string read-u8 peek-u8 u8-ready?
    read-bytevector read-bytevector!
    write display newline write-char write-string write-u8 write-bytevector
    flush-output-port
    
    ;; System interface
    load file-exists? delete-file command-line exit emergency-exit
    get-environment-variable get-environment-variables
    current-second current-jiffy jiffies-per-second
    features
    
    ;; Syntax keywords
    define define-values define-syntax define-record-type
    let let* letrec letrec* let-values let*-values
    lambda case-lambda
    if when unless cond case else =>
    and or
    begin do
    make-parameter parameterize
    guard let-syntax letrec-syntax
    syntax-rules syntax-error
    include include-ci
    quote quasiquote unquote unquote-splicing
    set!)

  (begin
    ;; Most R7RS base procedures are implemented as Rust primitives
    ;; and are automatically available in the global environment.

  ;; ============= Boolean Operations =============
  ;; These are implemented as Rust builtins and re-exported

  ;; not, boolean?, and boolean=? are already bound by Rust stdlib

  ;; ============= Equivalence Predicates =============
  ;; These are implemented as Rust builtins and re-exported

  ;; eq?, eqv?, and equal? are already bound by Rust stdlib

  ;; ============= Numbers =============
  ;; Numeric operations are implemented as Rust builtins and automatically bound
  ;; All arithmetic and numeric procedures are available from the Rust stdlib

  ;; ============= Characters =============
  ;; Character operations are re-exported from (:: char) module
  ;; All procedures are automatically available through import

  ;; ============= Strings =============
  ;; String operations are re-exported from (:: string) module
  ;; All procedures are automatically available through import

  ;; ============= Vectors =============
  ;; Vector operations are re-exported from (:: vector) module
  ;; All procedures are automatically available through import

  ;; ============= Bytevectors =============
  ;; Note: Bytevector support is limited in current implementation
  ;; These procedures will need to be implemented in Rust stdlib

  (define (bytevector? obj)
    "Returns #t if obj is a bytevector."
    ;; TODO: Implement bytevector support in Rust
    #f)

  (define (make-bytevector k . byte)
    "Returns a newly allocated bytevector of k elements."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector . bytes)
    "Returns a bytevector whose elements are the given bytes."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-length bv)
    "Returns the number of bytes in bv."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-u8-ref bv k)
    "Returns the kth byte of bv."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-u8-set! bv k byte)
    "Sets the kth byte of bv to byte."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-copy bv . start-end)
    "Returns a copy of bv."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-copy! to at from . start-end)
    "Copies bytes from from to to."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (bytevector-append . bytevectors)
    "Returns a bytevector formed by concatenating the bytevectors."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (utf8->string bv . start-end)
    "Converts UTF-8 bytevector to string."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  (define (string->utf8 str . start-end)
    "Converts string to UTF-8 bytevector."
    ;; TODO: Implement in Rust stdlib
    (error "bytevectors not yet implemented"))

  ;; ============= Lists and Pairs =============
  ;; List operations are re-exported from (:: list) module
  ;; All procedures are automatically available through import
  ;; Note: set-car! and set-cdr! need to be implemented in Rust

  (define (set-car! pair obj)
    "Sets the car of pair to obj."
    ;; TODO: Implement in Rust stdlib
    (error "set-car! not yet implemented"))

  (define (set-cdr! pair obj)
    "Sets the cdr of pair to obj."
    ;; TODO: Implement in Rust stdlib
    (error "set-cdr! not yet implemented"))

  ;; ============= Symbols =============
  ;; Symbol operations are implemented as Rust builtins

  (define (symbol=? sym1 sym2 . syms)
    "Returns #t if all symbols are equal."
    (and (symbol? sym1)
         (symbol? sym2)
         (eq? sym1 sym2)
         (or (null? syms)
             (apply symbol=? sym2 syms))))

  ;; ============= Control Features =============
  ;; Most control features are implemented as Rust builtins

  (define (apply proc args . more-args)
    "Applies proc to the given arguments."
    ;; TODO: Implement proper apply in Rust stdlib
    (if (null? more-args)
        (apply-primitive proc args)
        (apply-primitive proc (append (cons args (butlast more-args)) 
                                   (last more-args)))))

  ;; Helper functions that may not exist yet
  (define (butlast lst)
    "Returns all but the last element of lst."
    (if (null? (cdr lst))
        '()
        (cons (car lst) (butlast (cdr lst)))))

  (define (last lst)
    "Returns the last element of lst."
    (if (null? (cdr lst))
        (car lst)
        (last (cdr lst))))

  ;; Map and for-each should be available from (:: list) module
  ;; call/cc, values, call-with-values, dynamic-wind need Rust implementation
  
  (define call/cc call-with-current-continuation)

  (define (values . things)
    "Returns multiple values."
    ;; TODO: Implement multiple values in Rust
    (if (null? things)
        (if #f #f)  ; unspecified
        (if (null? (cdr things))
            (car things)
            things)))

  (define (call-with-values producer consumer)
    "Calls consumer with values from producer."
    ;; TODO: Implement in Rust stdlib
    (let ((vals (producer)))
      (if (pair? vals)
          (apply consumer vals)
          (consumer vals))))

  (define (dynamic-wind before thunk after)
    "Calls thunk with before/after guards."
    ;; TODO: Implement in Rust stdlib
    (before)
    (let ((result (thunk)))
      (after)
      result))

  ;; ============= Exceptions =============
  ;; Basic exception system implementation

  (define (error message . irritants)
    "Raises an error with message and irritants."
    ;; Basic error implementation - create error object and raise it
    (raise (make-error-object message irritants)))

  (define (make-error-object message irritants)
    "Creates an error object (internal helper)."
    ;; Simple error object representation as a list
    (list 'error-object message irritants))

  (define (error-object? obj)
    "Returns #t if obj is an error object."
    (and (pair? obj) 
         (eq? (car obj) 'error-object)
         (= (length obj) 3)))

  (define (error-object-message error-obj)
    "Returns the message of an error object."
    (if (error-object? error-obj)
        (cadr error-obj)
        (error "not an error object" error-obj)))

  (define (error-object-irritants error-obj)
    "Returns the irritants of an error object."
    (if (error-object? error-obj)
        (caddr error-obj)
        (error "not an error object" error-obj)))

  (define (raise obj)
    "Raises an exception."
    ;; TODO: Implement proper exception handling in Rust
    (display "Exception raised: " (current-error-port))
    (display obj (current-error-port))
    (newline (current-error-port))
    (exit 1))

  (define (raise-continuable obj)
    "Raises a continuable exception."
    ;; TODO: Implement continuable exceptions
    (raise obj))

  (define (with-exception-handler handler thunk)
    "Calls thunk with exception handler."
    ;; TODO: Implement proper exception handling
    ;; For now, just call thunk directly
    (thunk))

  ;; ============= Input and Output =============
  ;; I/O operations are re-exported from (:: io) module
  ;; All procedures are automatically available through import
  ;; Note: Some I/O procedures may need implementation in Rust

  ;; ============= System Interface =============
  ;; System procedures that need Rust implementation

  (define (load filename . environment)
    "Loads a Scheme file."
    ;; TODO: Implement file loading in Rust
    (error "load not yet implemented"))

  (define (file-exists? filename)
    "Returns #t if file exists."
    ;; TODO: Implement in Rust stdlib
    (error "file-exists? not yet implemented"))

  (define (delete-file filename)
    "Deletes a file."
    ;; TODO: Implement in Rust stdlib
    (error "delete-file not yet implemented"))

  (define (command-line)
    "Returns the command line arguments."
    ;; TODO: Implement in Rust stdlib
    '())

  (define (exit . code)
    "Exits the program."
    ;; TODO: Implement proper exit in Rust
    (let ((exit-code (if (null? code) 0 (car code))))
      (error "exit not yet implemented")))

  (define (emergency-exit . code)
    "Emergency exit."
    ;; TODO: Implement in Rust stdlib
    (if (null? code)
        (exit)
        (exit (car code))))

  (define (get-environment-variable name)
    "Gets an environment variable."
    ;; TODO: Implement in Rust stdlib
    #f)

  (define (get-environment-variables)
    "Gets all environment variables."
    ;; TODO: Implement in Rust stdlib
    '())

  (define (current-second)
    "Returns current time in seconds since epoch."
    ;; TODO: Implement in Rust stdlib
    0)

  (define (current-jiffy)
    "Returns current time in jiffies."
    ;; TODO: Implement in Rust stdlib
    0)

  (define (jiffies-per-second)
    "Returns jiffies per second."
    ;; TODO: Implement in Rust stdlib
    1000000)

  (define (features)
    "Returns list of feature identifiers."
    ;; Basic feature list for Lambdust
    '(lambdust r7rs exact-closed exact-complex ieee-float full-unicode ratios))

  ;; ============= Environments =============
  ;; Environment procedures that need Rust implementation

  (define (environment . import-sets)
    "Creates a new environment."
    ;; TODO: Implement environment creation in Rust
    (error "environment not yet implemented"))

  (define (scheme-report-environment version)
    "Returns R5RS environment."
    ;; TODO: Implement in Rust stdlib
    (if (= version 5)
        (interaction-environment)  ; fallback
        (error "unsupported scheme version" version)))

  (define (null-environment version)
    "Returns null environment."
    ;; TODO: Implement in Rust stdlib
    (if (= version 5)
        (interaction-environment)  ; fallback
        (error "unsupported scheme version" version)))

  (define (interaction-environment)
    "Returns the interaction environment."
    ;; TODO: Implement in Rust stdlib - return current global environment
    'current-environment)

  (define (eval expr environment)
    "Evaluates expr in environment."
    ;; TODO: Implement proper eval in Rust
    (error "eval not yet implemented"))

  ;; ============= Parameters =============
  ;; Parameter objects for dynamic scoping

  (define (make-parameter init . converter)
    "Creates a parameter object."
    ;; TODO: Implement parameters in Rust stdlib
    ;; For now, return a simple closure-based implementation
    (let ((conv (if (null? converter) (lambda (x) x) (car converter)))
          (value (if (null? converter) init ((car converter) init))))
      (lambda args
        (cond
          ((null? args) value)
          ((= (length args) 1)
           (set! value (conv (car args)))
           (if #f #f))  ; unspecified
          (else (error "parameter: wrong number of arguments" args))))))

  ;; Note: parameterize syntax form needs macro system support

  ;; ============= Additional Helper Functions =============
  ;; These are moved from earlier section where they were defined
  
  (define (apply-primitive proc args)
    "Apply a procedure to a list of arguments (primitive version)."
    ;; TODO: This should be implemented as a real Rust builtin
    (error "apply not yet fully implemented"))

  ) ;; End of (begin ...)
) ;; End of (define-library (scheme base) ...)