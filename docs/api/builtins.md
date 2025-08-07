# Built-in Functions and Special Forms

This document describes the core built-in functions and special forms that are implemented directly in the Lambdust interpreter.

## Special Forms

Lambdust has exactly 10 special forms that provide the foundation for all other language constructs:

### 1. `quote`

Returns data without evaluation.

**Syntax**: `(quote datum)` or `'datum`

**Examples**:
```scheme
(quote hello)        ; => hello
'(1 2 3)            ; => (1 2 3)
'(+ 1 2)            ; => (+ 1 2)  ; not evaluated
```

### 2. `lambda`

Creates anonymous procedures.

**Syntax**: `(lambda formals body ...)`

**Examples**:
```scheme
(lambda (x) (* x x))              ; square function
(lambda (x y) (+ x y))            ; addition function
(lambda x x)                      ; variadic identity
(lambda (x . rest) (cons x rest)) ; mixed parameters
```

### 3. `if`

Conditional expression.

**Syntax**: `(if test consequent alternate)`

**Examples**:
```scheme
(if (> 5 3) 'greater 'not-greater)  ; => greater
(if #f 'true 'false)                 ; => false
(if 0 'truthy 'falsy)                ; => truthy (0 is truthy)
```

### 4. `define`

Variable and function definitions.

**Syntax**: 
- `(define variable expression)`
- `(define (name . formals) body ...)`

**Examples**:
```scheme
(define pi 3.14159)
(define (square x) (* x x))
(define (variadic . args) args)

;; With type annotations
(define (typed-add x y)
  #:type (-> Number Number Number)
  #:pure #t
  (+ x y))
```

### 5. `set!`

Assignment (transformed to state monad in pure contexts).

**Syntax**: `(set! variable expression)`

**Examples**:
```scheme
(define x 10)
(set! x 20)
x  ; => 20

;; In pure contexts, creates new environment generation
(define (counter)
  (let ([n 0])
    (lambda ()
      (set! n (+ n 1))  ; Creates new environment
      n)))
```

### 6. `define-syntax`

Macro definitions using syntax-rules.

**Syntax**: `(define-syntax name transformer)`

**Examples**:
```scheme
(define-syntax when
  (syntax-rules ()
    [(when test body ...)
     (if test (begin body ...))]))

(when (> 5 3)
  (display "5 is greater than 3")
  (newline))
```

### 7. `call-with-current-continuation` (call/cc)

Captures the current continuation.

**Syntax**: `(call-with-current-continuation proc)`

**Examples**:
```scheme
(call/cc (lambda (k) (k 42)))  ; => 42

;; Early return
(define (find-positive lst)
  (call/cc (lambda (return)
    (for-each (lambda (x)
                (when (positive? x)
                  (return x)))
              lst)
    #f)))
```

### 8. `primitive`

Direct access to built-in operations and FFI calls.

**Syntax**: `(primitive name arg ...)`

**Examples**:
```scheme
(primitive + 1 2 3)              ; => 6
(primitive cons 1 '(2 3))        ; => (1 2 3)
(primitive rust-function arg1 arg2)  ; FFI call
```

### 9. `::`

Type annotations.

**Syntax**: `(:: expression type)`

**Examples**:
```scheme
(:: 42 Number)
(:: "hello" String)
(:: (lambda (x) x) (-> a a))  ; polymorphic type
```

### 10. Keyword Literals

Keyword parameters and self-evaluating keywords.

**Syntax**: `#:keyword`

**Examples**:
```scheme
#:name                           ; keyword literal
(greet "Alice" #:greeting "Hi")  ; keyword parameter
```

## Core Primitive Functions

These functions are implemented directly in Rust and provide the foundation for all other operations:

### Arithmetic Operations

```scheme
(+ number ...)          ; addition
(- number number ...)   ; subtraction  
(* number ...)          ; multiplication
(/ number number ...)   ; division
(quotient n1 n2)       ; integer division
(remainder n1 n2)      ; remainder
(modulo n1 n2)         ; modulo
(abs number)           ; absolute value
(gcd n1 ...)           ; greatest common divisor
(lcm n1 ...)           ; least common multiple
```

### Comparison Operations

```scheme
(= number number ...)   ; numeric equality
(< number number ...)   ; numeric less than
(> number number ...)   ; numeric greater than
(<= number number ...)  ; numeric less than or equal
(>= number number ...)  ; numeric greater than or equal
```

### Type Predicates

```scheme
(boolean? obj)      ; boolean predicate
(number? obj)       ; number predicate
(integer? obj)      ; integer predicate
(real? obj)         ; real number predicate
(complex? obj)      ; complex number predicate
(string? obj)       ; string predicate
(symbol? obj)       ; symbol predicate
(char? obj)         ; character predicate
(vector? obj)       ; vector predicate
(procedure? obj)    ; procedure predicate
(pair? obj)         ; pair predicate
(null? obj)         ; null predicate
(list? obj)         ; proper list predicate
```

### Equivalence Operations

```scheme
(eq? obj1 obj2)      ; identity comparison
(eqv? obj1 obj2)     ; equivalence comparison
(equal? obj1 obj2)   ; structural equality
```

### List Operations

```scheme
(cons obj1 obj2)     ; construct pair
(car pair)           ; first element
(cdr pair)           ; rest elements
(list obj ...)       ; construct list
(length list)        ; list length
(append list ...)    ; append lists
(reverse list)       ; reverse list
(list-ref list k)    ; list element access
(list-tail list k)   ; list tail
```

### String Operations

```scheme
(string char ...)           ; construct string
(string-length string)      ; string length
(string-ref string k)       ; character access
(substring string start end) ; extract substring
(string-append string ...)  ; concatenate strings
(string=? string1 string2)  ; string equality
(string<? string1 string2)  ; string comparison
```

### Vector Operations

```scheme
(vector obj ...)            ; construct vector
(make-vector k obj)         ; create vector with fill
(vector-length vector)      ; vector length
(vector-ref vector k)       ; element access
(vector-set! vector k obj)  ; element assignment
(vector->list vector)       ; convert to list
(list->vector list)         ; convert from list
```

### I/O Operations

```scheme
(display obj port)          ; write human-readable
(write obj port)           ; write machine-readable
(newline port)             ; write newline
(read port)                ; read S-expression
(read-char port)           ; read character
(write-char char port)     ; write character
(current-input-port)       ; get current input port
(current-output-port)      ; get current output port
(current-error-port)       ; get current error port
```

### Control Flow

```scheme
(apply proc args)          ; apply procedure
(eval expr env)            ; evaluate expression
(error message obj ...)    ; signal error
(exit code)                ; exit program
```

## Type System Integration

### Type Checking Functions

```scheme
(type-of obj)              ; get type of object
(has-type? obj type)       ; type membership test  
(subtype? type1 type2)     ; subtype relationship
```

### Contract System

```scheme
(contract? obj)            ; contract predicate
(check-contract obj contract)  ; contract verification
(with-contract contract expr)   ; contract enforcement
```

## Effect System Integration

### Effect Operations

```scheme
(pure? proc)               ; pure procedure predicate
(effects-of expr)          ; get effects of expression
(with-effect effect expr)  ; execute with effect
(handle-effects handlers expr)  ; effect handling
```

### Monadic Operations

```scheme
(return value)             ; monadic return
(bind m f)                 ; monadic bind
(sequence m1 m2)           ; monadic sequence
```

## Memory Management

### Garbage Collection

```scheme
(gc)                       ; force garbage collection
(gc-stats)                 ; garbage collection statistics
(weak-ref obj)             ; create weak reference
(weak-ref-value weak)      ; dereference weak reference
```

## Debugging and Introspection

### Debugging Functions

```scheme
(trace proc)               ; enable procedure tracing
(untrace proc)             ; disable procedure tracing
(break)                    ; enter debugger
(backtrace)                ; show call stack
```

### Environment Introspection

```scheme
(environment-bindings env) ; get environment bindings
(environment-parent env)   ; get parent environment
(current-environment)      ; get current environment
```

## Performance and Optimization

### Optimization Hints

```scheme
(optimize-for-speed expr)  ; speed optimization hint
(optimize-for-size expr)   ; size optimization hint
(inline proc)              ; inlining hint
(no-inline proc)           ; prevent inlining
```

### Profiling

```scheme
(profile expr)             ; profile expression
(profile-report)           ; show profiling report
(clear-profile)            ; clear profiling data
```

## FFI Integration

### Foreign Function Interface

```scheme
(load-library path)        ; load dynamic library
(foreign-function lib name sig)  ; get foreign function
(foreign-data lib name type)     ; access foreign data
```

### Rust Integration

```scheme
(rust-call function args) ; call Rust function
(rust-struct fields)      ; create Rust struct
(rust-enum variant data)  ; create Rust enum
```

## Limitations and Implementation Notes

### Current Limitations

1. **Numeric Tower**: Full numeric tower is not yet implemented
2. **Unicode**: Limited Unicode support in string operations
3. **Modules**: Module system is still in development
4. **Macros**: Some advanced macro features are not yet available

### Implementation Details

1. **Tail Call Optimization**: All tail calls are optimized
2. **Garbage Collection**: Uses generational copying GC
3. **Type Inference**: Based on Hindley-Milner with extensions
4. **Effect Tracking**: Automatic effect inference and lifting

### Performance Characteristics

- **Function Calls**: O(1) for direct calls, O(log n) for dynamic dispatch
- **List Operations**: Standard time complexities (cons: O(1), length: O(n))
- **String Operations**: O(n) for most operations, with sharing optimizations
- **Vector Operations**: O(1) for access, O(n) for bulk operations

## See Also

- [Standard Library API](stdlib/) - Higher-level library functions
- [Type System Guide](../user-guide/type-system.md) - Detailed type system documentation
- [Effect System Guide](../user-guide/effect-system.md) - Effect system usage
- [Performance Guide](../developer/performance.md) - Optimization techniques