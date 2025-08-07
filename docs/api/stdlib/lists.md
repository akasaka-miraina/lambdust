# List Operations

The list module provides comprehensive list processing operations following R7RS specifications with performance optimizations and extensions.

## Overview

Lists are fundamental data structures in Lambdust, implemented as linked lists with structure sharing for efficiency. The list module provides both basic operations and higher-order functions for functional programming.

### List Types

- **Proper List**: `(a b c)` - finite list ending with `()`
- **Improper List**: `(a b . c)` - finite list ending with non-null value
- **Circular List**: Lists with cycles (detected and handled safely)
- **Empty List**: `()` or `'()` - the null list

## Basic List Operations

### Construction

#### `cons`

```scheme
(cons obj1 obj2) -> pair
```

Creates a new pair with `obj1` as the car and `obj2` as the cdr.

**Type**: `(-> a b (Pair a b))`

**Examples**:
```scheme
(cons 1 2)           ; => (1 . 2)
(cons 1 '())         ; => (1)
(cons 1 '(2 3))      ; => (1 2 3)
(cons 'a (cons 'b '())) ; => (a b)
```

#### `list`

```scheme
(list obj ...) -> list
```

Creates a list from its arguments.

**Type**: `(-> a ... (List a))`

**Examples**:
```scheme
(list)               ; => ()
(list 1)             ; => (1)
(list 1 2 3)         ; => (1 2 3)
(list 'a 'b 'c)      ; => (a b c)
```

#### `make-list`

```scheme
(make-list k) -> list
(make-list k fill) -> list
```

Creates a list of length `k` with optional fill value.

**Type**: `(-> Integer (List a))` or `(-> Integer a (List a))`

**Examples**:
```scheme
(make-list 3)        ; => (() () ())
(make-list 3 'x)     ; => (x x x)
(make-list 0)        ; => ()
```

### Access

#### `car` and `cdr`

```scheme
(car pair) -> obj
(cdr pair) -> obj
```

Access the first element (car) or rest (cdr) of a pair.

**Type**: `(-> (Pair a b) a)` and `(-> (Pair a b) b)`

**Examples**:
```scheme
(car '(1 2 3))       ; => 1
(cdr '(1 2 3))       ; => (2 3)
(car '(a . b))       ; => a
(cdr '(a . b))       ; => b
```

**Errors**:
```scheme
(car '())            ; Error: car: contract violation
(car 42)             ; Error: car: contract violation
```

#### `cadr`, `caddr`, etc.

Convenient combinations of `car` and `cdr`:

```scheme
(cadr lst)           ; (car (cdr lst))
(caddr lst)          ; (car (cdr (cdr lst)))
(cadddr lst)         ; (car (cdr (cdr (cdr lst))))
```

**Examples**:
```scheme
(cadr '(1 2 3 4))    ; => 2
(caddr '(1 2 3 4))   ; => 3
(cadddr '(1 2 3 4))  ; => 4
```

#### `list-ref`

```scheme
(list-ref list k) -> obj
```

Returns the k-th element of the list (zero-indexed).

**Type**: `(-> (List a) Integer a)`

**Examples**:
```scheme
(list-ref '(a b c d) 0)  ; => a
(list-ref '(a b c d) 2)  ; => c
(list-ref '(a b c d) 3)  ; => d
```

**Errors**:
```scheme
(list-ref '(a b c) 5)    ; Error: list-ref: index out of bounds
(list-ref '(a b c) -1)   ; Error: list-ref: negative index
```

#### `list-tail`

```scheme
(list-tail list k) -> list
```

Returns the list after dropping the first `k` elements.

**Type**: `(-> (List a) Integer (List a))`

**Examples**:
```scheme
(list-tail '(1 2 3 4 5) 0)  ; => (1 2 3 4 5)
(list-tail '(1 2 3 4 5) 2)  ; => (3 4 5)
(list-tail '(1 2 3 4 5) 5)  ; => ()
```

### Predicates

#### `pair?`

```scheme
(pair? obj) -> boolean
```

Tests if object is a pair.

**Examples**:
```scheme
(pair? '(1 2))       ; => #t
(pair? '(1 . 2))     ; => #t
(pair? '())          ; => #f
(pair? 42)           ; => #f
```

#### `null?`

```scheme
(null? obj) -> boolean
```

Tests if object is the empty list.

**Examples**:
```scheme
(null? '())          ; => #t
(null? '(1))         ; => #f
(null? #f)           ; => #f
```

#### `list?`

```scheme
(list? obj) -> boolean
```

Tests if object is a proper list.

**Examples**:
```scheme
(list? '())          ; => #t
(list? '(1 2 3))     ; => #t
(list? '(1 . 2))     ; => #f (improper list)
```

## List Processing

### `length`

```scheme
(length list) -> integer
```

Returns the length of a proper list.

**Type**: `(-> (List a) Integer)`

**Examples**:
```scheme
(length '())         ; => 0
(length '(1 2 3))    ; => 3
(length '(a (b c) d)) ; => 3
```

**Errors**:
```scheme
(length '(1 . 2))    ; Error: length: improper list
```

### `append`

```scheme
(append list ...) -> list
```

Concatenates lists together.

**Type**: `(-> (List a) ... (List a))`

**Examples**:
```scheme
(append)             ; => ()
(append '(1 2))      ; => (1 2)
(append '(1 2) '(3 4)) ; => (1 2 3 4)
(append '(a) '(b c) '(d)) ; => (a b c d)
```

**Performance**: O(n) where n is the total length of all but the last list.

### `reverse`

```scheme
(reverse list) -> list
```

Returns a list with elements in reverse order.

**Type**: `(-> (List a) (List a))`

**Examples**:
```scheme
(reverse '())        ; => ()
(reverse '(1))       ; => (1)
(reverse '(1 2 3))   ; => (3 2 1)
(reverse '(a (b c) d)) ; => (d (b c) a)
```

**Performance**: O(n) with tail-call optimization.

### `list-copy`

```scheme
(list-copy obj) -> obj
```

Returns a shallow copy of the list structure.

**Type**: `(-> a a)`

**Examples**:
```scheme
(list-copy '(1 2 3)) ; => (1 2 3)
(list-copy 42)       ; => 42 (non-lists returned as-is)
```

## Searching and Membership

### `member`

```scheme
(member obj list) -> list | #f
(member obj list compare) -> list | #f
```

Searches for `obj` in `list` using `equal?` or custom comparison.

**Type**: `(-> a (List a) (List a) | Boolean)` or `(-> a (List a) (-> a a Boolean) (List a) | Boolean)`

**Examples**:
```scheme
(member 'a '(b c a d))   ; => (a d)
(member 'x '(a b c))     ; => #f
(member '(2) '((1) (2) (3))) ; => ((2) (3))

;; With custom comparison
(member "hello" '("HI" "HELLO" "world") string-ci=?)
; => ("HELLO" "world")
```

### `memq` and `memv`

```scheme
(memq obj list) -> list | #f
(memv obj list) -> list | #f
```

Like `member` but using `eq?` and `eqv?` respectively.

**Examples**:
```scheme
(memq 'a '(b c a d))     ; => (a d)
(memv 2 '(1 2 3))        ; => (2 3)

;; Difference with symbols
(memq 'a '(b c a d))     ; => (a d)
(member 'a '(b c a d))   ; => (a d) (same for symbols)

;; Difference with numbers
(memq 2.0 '(1.0 2.0 3.0)) ; => implementation-dependent
(memv 2.0 '(1.0 2.0 3.0)) ; => (2.0 3.0)
```

## Association Lists

### `assoc`

```scheme
(assoc obj alist) -> pair | #f
(assoc obj alist compare) -> pair | #f
```

Searches association list for a pair whose car equals `obj`.

**Type**: `(-> a (List (Pair a b)) (Pair a b) | Boolean)`

**Examples**:
```scheme
(assoc 'b '((a 1) (b 2) (c 3)))  ; => (b 2)
(assoc 'd '((a 1) (b 2) (c 3)))  ; => #f

;; With custom comparison
(assoc "hello" '(("HI" 1) ("HELLO" 2)) string-ci=?)
; => ("HELLO" 2)
```

### `assq` and `assv`

```scheme
(assq obj alist) -> pair | #f
(assv obj alist) -> pair | #f
```

Like `assoc` but using `eq?` and `eqv?` respectively.

**Examples**:
```scheme
(assq 'b '((a 1) (b 2) (c 3)))   ; => (b 2)
(assv 2 '((1 "one") (2 "two")))  ; => (2 "two")
```

## Higher-Order Functions

### `map`

```scheme
(map proc list1 list2 ...) -> list
```

Applies `proc` to corresponding elements of the lists.

**Type**: `(-> (-> a ... b) (List a) ... (List b))`

**Examples**:
```scheme
;; Single list
(map square '(1 2 3 4))      ; => (1 4 9 16)
(map char-upcase '(#\a #\b #\c)) ; => (#\A #\B #\C)

;; Multiple lists
(map + '(1 2 3) '(4 5 6))    ; => (5 7 9)
(map list '(a b) '(1 2) '(x y)) ; => ((a 1 x) (b 2 y))

;; With lambda
(map (lambda (x) (* x 2)) '(1 2 3 4)) ; => (2 4 6 8)
```

**Error Handling**:
```scheme
(map + '(1 2 3) '(4 5))      ; Error: map: lists must have same length
(map 42 '(1 2 3))            ; Error: map: first argument must be procedure
```

### `for-each`

```scheme
(for-each proc list1 list2 ...) -> unspecified
```

Applies `proc` to corresponding elements for side effects.

**Type**: `(-> (-> a ... Void) (List a) ... Void)`

**Examples**:
```scheme
(for-each display '(1 2 3))  ; prints: 123
(for-each (lambda (x) (display x) (newline)) '(a b c))
; prints:
; a
; b  
; c
```

### `filter`

```scheme
(filter pred list) -> list
```

Returns list of elements satisfying predicate.

**Type**: `(-> (-> a Boolean) (List a) (List a))`

**Examples**:
```scheme
(filter odd? '(1 2 3 4 5 6))     ; => (1 3 5)
(filter positive? '(-2 -1 0 1 2)) ; => (1 2)
(filter symbol? '(a 1 b 2 c))    ; => (a b c)

;; Custom predicate
(filter (lambda (x) (> x 10)) '(5 15 8 20 12))
; => (15 20 12)
```

### `fold-left` and `fold-right`

```scheme
(fold-left proc init list1 list2 ...) -> obj
(fold-right proc init list1 list2 ...) -> obj
```

Reduce lists using binary procedure.

**Type**: `(-> (-> a b ... a) a (List b) ... a)`

**Examples**:
```scheme
;; fold-left (left-associative)
(fold-left + 0 '(1 2 3 4))       ; => 10
(fold-left - 0 '(1 2 3))         ; => -6 (((0-1)-2)-3)
(fold-left cons '() '(1 2 3))    ; => ((((() . 1) . 2) . 3)

;; fold-right (right-associative)  
(fold-right + 0 '(1 2 3 4))      ; => 10
(fold-right - 0 '(1 2 3))        ; => 2 (1-(2-(3-0)))
(fold-right cons '() '(1 2 3))   ; => (1 2 3)

;; Finding maximum
(fold-left max 0 '(3 7 2 9 1))   ; => 9

;; String concatenation
(fold-right string-append "" '("hello" " " "world"))
; => "hello world"
```

## List Utilities (Extensions)

### `take` and `drop`

```scheme
(take list k) -> list
(drop list k) -> list
```

Take or drop first `k` elements.

**Examples**:
```scheme
(take '(1 2 3 4 5) 3)    ; => (1 2 3)
(drop '(1 2 3 4 5) 2)    ; => (3 4 5)
(take '(a b c) 0)        ; => ()
(drop '(a b c) 3)        ; => ()
```

### `split-at`

```scheme
(split-at list k) -> (list list)
```

Split list at position `k`.

**Examples**:
```scheme
(split-at '(1 2 3 4 5) 2)  ; => ((1 2) (3 4 5))
(split-at '(a b c) 0)      ; => (() (a b c))
```

### `zip` and `unzip`

```scheme
(zip list1 list2 ...) -> list
(unzip list) -> (list ...)
```

Zip lists together or unzip a list of lists.

**Examples**:
```scheme
(zip '(1 2 3) '(a b c))      ; => ((1 a) (2 b) (3 c))
(zip '(1 2) '(a b) '(x y))   ; => ((1 a x) (2 b x))

(unzip '((1 a) (2 b) (3 c))) ; => ((1 2 3) (a b c))
```

### `partition`

```scheme
(partition pred list) -> (list list)
```

Partition list into elements satisfying and not satisfying predicate.

**Examples**:
```scheme
(partition odd? '(1 2 3 4 5 6))  ; => ((1 3 5) (2 4 6))
(partition symbol? '(a 1 b 2 c)) ; => ((a b c) (1 2))
```

### `remove` and `remove-duplicates`

```scheme
(remove obj list) -> list
(remove-duplicates list) -> list
(remove-duplicates list equal-proc) -> list
```

Remove elements from list.

**Examples**:
```scheme
(remove 'a '(a b a c a))         ; => (b c)
(remove-duplicates '(1 2 1 3 2)) ; => (1 2 3)
(remove-duplicates '("a" "A" "a") string-ci=?)
; => ("a" "A")
```

### `sort`

```scheme
(sort list less-proc) -> list
```

Sort list using comparison procedure.

**Examples**:
```scheme
(sort '(3 1 4 1 5 9) <)          ; => (1 1 3 4 5 9)
(sort '("banana" "apple" "cherry") string<?)
; => ("apple" "banana" "cherry")

;; Custom comparison
(sort '((a 3) (b 1) (c 2)) 
      (lambda (x y) (< (cadr x) (cadr y))))
; => ((b 1) (c 2) (a 3))
```

## Performance Characteristics

### Time Complexity

| Operation | Best Case | Average Case | Worst Case |
|-----------|-----------|--------------|------------|
| `cons` | O(1) | O(1) | O(1) |
| `car`, `cdr` | O(1) | O(1) | O(1) |
| `length` | O(n) | O(n) | O(n) |
| `append` | O(m) | O(m) | O(m) |
| `reverse` | O(n) | O(n) | O(n) |
| `list-ref` | O(k) | O(k) | O(k) |
| `member` | O(1) | O(n/2) | O(n) |
| `map` | O(n) | O(n) | O(n) |
| `filter` | O(n) | O(n) | O(n) |
| `fold-left/right` | O(n) | O(n) | O(n) |
| `sort` | O(n log n) | O(n log n) | O(n log n) |

Where:
- n = length of list
- m = total length of all but last list (for `append`)
- k = index position

### Memory Usage

- **Structure sharing**: Lists share common tails when possible
- **Garbage collection**: Automatic memory management
- **Tail-call optimization**: Recursive functions don't grow the stack

## Error Handling

All list operations provide informative error messages:

```scheme
(car '())
; Error: car: contract violation
;   expected: pair?
;   given: ()
;   at: <stdin>:1:0

(list-ref '(a b c) 5)
; Error: list-ref: index out of bounds  
;   index: 5
;   list length: 3
;   at: <stdin>:1:0

(map + '(1 2 3) '(4 5))
; Error: map: all lists must have the same length
;   list 1 length: 3
;   list 2 length: 2
;   at: <stdin>:1:0
```

## Thread Safety

All list operations are thread-safe:
- **Immutable operations** can be called concurrently
- **Shared structure** is safe to read from multiple threads
- **No mutation** of existing list structure (except `list-set!`)

## See Also

- [Vectors](vectors.md) - Array-like data structures
- [Strings](strings.md) - String processing operations  
- [Higher-Order Functions Tutorial](../../tutorials/functional-programming.md)
- [R7RS Specification](https://small.r7rs.org/) - Official standard