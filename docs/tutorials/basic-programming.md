# Basic Programming in Lambdust

This tutorial introduces the fundamental concepts of programming in Lambdust, covering syntax, data types, functions, and control structures.

## Table of Contents

1. [Expressions and Evaluation](#expressions-and-evaluation)
2. [Data Types](#data-types)
3. [Variables and Constants](#variables-and-constants)
4. [Functions](#functions)
5. [Conditional Logic](#conditional-logic)
6. [Lists and Data Structures](#lists-and-data-structures)
7. [Recursion](#recursion)
8. [Higher-Order Functions](#higher-order-functions)
9. [Error Handling](#error-handling)
10. [Practice Exercises](#practice-exercises)

## Expressions and Evaluation

Lambdust uses **S-expressions** (symbolic expressions) for all code. Everything is an expression that evaluates to a value.

### Basic Syntax

```scheme
;; Comments start with semicolons
;; S-expressions use parentheses: (operator operand1 operand2 ...)

;; Numbers evaluate to themselves
42          ; => 42
3.14        ; => 3.14
1/2         ; => 1/2 (exact rational)

;; Strings evaluate to themselves
"Hello"     ; => "Hello"

;; Symbols need to be quoted to prevent evaluation
'hello      ; => hello
'(1 2 3)    ; => (1 2 3)
```

### Arithmetic Expressions

```scheme
;; Prefix notation: operator comes first
(+ 1 2)         ; => 3
(* 3 4)         ; => 12
(- 10 3)        ; => 7
(/ 15 3)        ; => 5

;; Nested expressions
(+ (* 2 3) (/ 8 4))     ; => 8
(* (+ 1 2) (- 5 3))     ; => 6

;; Multiple arguments
(+ 1 2 3 4 5)           ; => 15
(* 2 3 4)               ; => 24
```

## Data Types

Lambdust provides several built-in data types:

### Numbers

```scheme
;; Integers
42
-17
0

;; Rationals (exact fractions)
1/2
22/7
-3/4

;; Floating-point
3.14159
-2.5
1.0e10

;; Complex numbers
3+4i
-2-5i
```

### Booleans

```scheme
#t          ; true
#f          ; false

;; Boolean operations
(not #t)    ; => #f
(not #f)    ; => #t
(not 0)     ; => #f (0 is truthy)
(not '())   ; => #f (empty list is truthy)
```

### Characters

```scheme
#\a         ; character 'a'
#\A         ; character 'A'
#\0         ; character '0'
#\space     ; space character
#\newline   ; newline character

;; Character predicates
(char? #\a)         ; => #t
(char-alphabetic? #\a)  ; => #t
(char-numeric? #\5)     ; => #t
```

### Strings

```scheme
"Hello, World!"
"Scheme is fun"
""              ; empty string

;; String operations
(string-length "hello")     ; => 5
(string-ref "hello" 1)      ; => #\e
(substring "hello" 1 4)     ; => "ell"
(string-append "hello" " " "world")  ; => "hello world"
```

### Symbols

```scheme
'hello      ; symbol
'lambda     ; symbol
'+          ; symbol (even operators are symbols when quoted)

;; Symbol predicates and operations
(symbol? 'hello)        ; => #t
(symbol->string 'hello) ; => "hello"
(string->symbol "world"); => world
```

## Variables and Constants

### Defining Variables

```scheme
;; Define a variable
(define x 42)
x                       ; => 42

;; Define a string
(define greeting "Hello, Lambdust!")
greeting                ; => "Hello, Lambdust!"

;; Define a list
(define numbers '(1 2 3 4 5))
numbers                 ; => (1 2 3 4 5)
```

### Modifying Variables

```scheme
;; Define a variable
(define counter 0)
counter                 ; => 0

;; Modify it
(set! counter 10)
counter                 ; => 10

;; Increment
(set! counter (+ counter 1))
counter                 ; => 11
```

### Local Variables with `let`

```scheme
;; let creates local bindings
(let ([x 10]
      [y 20])
  (+ x y))              ; => 30

;; Variables are only available inside the let
; x                     ; Error: x is not defined

;; Nested let expressions
(let ([x 5])
  (let ([y (+ x 3)])
    (* x y)))           ; => 40
```

## Functions

### Defining Functions

```scheme
;; Simple function definition
(define (square x)
  (* x x))

(square 5)              ; => 25
(square 3.5)            ; => 12.25

;; Function with multiple parameters
(define (add-three x y z)
  (+ x y z))

(add-three 1 2 3)       ; => 6

;; Function with documentation
(define (circle-area radius)
  "Calculate the area of a circle given the radius."
  (* 3.14159 radius radius))

(circle-area 5)         ; => 78.53975
```

### Anonymous Functions (Lambda)

```scheme
;; Lambda creates anonymous functions
(lambda (x) (* x x))    ; square function

;; Using lambda directly
((lambda (x) (* x x)) 7)    ; => 49

;; Assigning lambda to variable
(define square (lambda (x) (* x x)))
(square 4)              ; => 16

;; Multi-parameter lambda
(define add (lambda (x y) (+ x y)))
(add 3 4)               ; => 7
```

### Functions with Variable Arguments

```scheme
;; Rest parameters with dot notation
(define (sum . numbers)
  (if (null? numbers)
      0
      (+ (car numbers) (apply sum (cdr numbers)))))

(sum)                   ; => 0
(sum 1)                 ; => 1
(sum 1 2 3 4)          ; => 10

;; Mixed parameters
(define (greet name . titles)
  (string-append "Hello, " 
                 (string-join titles " ")
                 " " 
                 name))

(greet "Smith" "Dr." "Professor")  ; => "Hello, Dr. Professor Smith"
```

## Conditional Logic

### Basic `if` Expressions

```scheme
;; if expression: (if test consequent alternate)
(define (abs x)
  (if (< x 0)
      (- x)         ; consequent: negate if negative
      x))           ; alternate: return as-is if positive

(abs -5)            ; => 5
(abs 7)             ; => 7

;; if without alternate (returns unspecified if test fails)
(if (> 5 3)
    (display "5 is greater than 3"))  ; prints: 5 is greater than 3
```

### Multi-way Conditionals with `cond`

```scheme
;; cond for multiple conditions
(define (grade score)
  (cond
    [(>= score 90) "A"]
    [(>= score 80) "B"]
    [(>= score 70) "C"]
    [(>= score 60) "D"]
    [else "F"]))

(grade 85)          ; => "B"
(grade 55)          ; => "F"

;; cond with actions
(define (describe-number x)
  (cond
    [(< x 0) 
     (display "Negative number: ")
     (display x)]
    [(= x 0)
     (display "Zero")]
    [(> x 0)
     (display "Positive number: ")
     (display x)]
    [else
     (display "Not a number")]))
```

### Boolean Operators

```scheme
;; and: returns first false value or last value
(and #t #t #t)      ; => #t
(and #t #f #t)      ; => #f
(and 1 2 3)         ; => 3

;; or: returns first true value or last value  
(or #f #f #t)       ; => #t
(or #f #f #f)       ; => #f
(or 1 2 3)          ; => 1

;; not: logical negation
(not #t)            ; => #f
(not #f)            ; => #t
(not 0)             ; => #f (0 is truthy)
```

## Lists and Data Structures

### Creating Lists

```scheme
;; Quoted lists
'(1 2 3 4)          ; => (1 2 3 4)
'()                 ; => () (empty list)

;; Constructed lists
(list 1 2 3 4)      ; => (1 2 3 4)
(cons 1 '(2 3))     ; => (1 2 3)
(cons 'a (cons 'b '())) ; => (a b)
```

### List Operations

```scheme
;; Accessing list elements
(define lst '(a b c d e))

(car lst)           ; => a (first element)
(cdr lst)           ; => (b c d e) (rest of list)
(cadr lst)          ; => b (second element)
(caddr lst)         ; => c (third element)

;; List predicates
(null? '())         ; => #t
(null? '(1 2))      ; => #f
(list? '(1 2 3))    ; => #t
(pair? '(1 . 2))    ; => #t

;; List length and access
(length '(1 2 3 4)) ; => 4
(list-ref '(a b c d) 2) ; => c (zero-indexed)
```

### List Processing

```scheme
;; Append lists
(append '(1 2) '(3 4) '(5 6))   ; => (1 2 3 4 5 6)

;; Reverse list
(reverse '(1 2 3 4))            ; => (4 3 2 1)

;; Membership testing
(member 'b '(a b c d))          ; => (b c d)
(member 'x '(a b c d))          ; => #f

;; List searching
(memq 'b '(a b c))              ; => (b c) (identity comparison)
(memv 2 '(1 2 3))               ; => (2 3) (value comparison)
```

## Recursion

Recursion is fundamental in Lambdust. Many operations are naturally recursive.

### Simple Recursion

```scheme
;; Recursive factorial
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(factorial 5)       ; => 120
(factorial 0)       ; => 1

;; Recursive list length
(define (my-length lst)
  (if (null? lst)
      0
      (+ 1 (my-length (cdr lst)))))

(my-length '(a b c d))  ; => 4
```

### Tail Recursion

Tail recursion is optimized to prevent stack overflow:

```scheme
;; Tail-recursive factorial with accumulator
(define (factorial-iter n acc)
  (if (<= n 1)
      acc
      (factorial-iter (- n 1) (* n acc))))

(define (factorial n)
  (factorial-iter n 1))

;; Tail-recursive list reversal
(define (reverse-iter lst acc)
  (if (null? lst)
      acc
      (reverse-iter (cdr lst) (cons (car lst) acc))))

(define (my-reverse lst)
  (reverse-iter lst '()))
```

### Tree Recursion

```scheme
;; Fibonacci sequence (inefficient but illustrative)
(define (fibonacci n)
  (cond
    [(<= n 0) 0]
    [(= n 1) 1]
    [else (+ (fibonacci (- n 1))
             (fibonacci (- n 2)))]))

(fibonacci 10)      ; => 55

;; Tree traversal
(define (count-leaves tree)
  (cond
    [(null? tree) 0]
    [(not (pair? tree)) 1]
    [else (+ (count-leaves (car tree))
             (count-leaves (cdr tree)))]))

(count-leaves '((1 2) (3 4) 5))  ; => 5
```

## Higher-Order Functions

Functions that operate on other functions:

### Map

```scheme
;; Apply function to each element
(map square '(1 2 3 4))         ; => (1 4 9 16)
(map + '(1 2 3) '(4 5 6))      ; => (5 7 9)
(map (lambda (x) (* x 2)) '(1 2 3 4))  ; => (2 4 6 8)

;; String operations with map
(map char-upcase (string->list "hello"))  ; => (#\H #\E #\L #\L #\O)
```

### Filter

```scheme
;; Filter elements matching predicate
(filter odd? '(1 2 3 4 5 6))           ; => (1 3 5)
(filter (lambda (x) (> x 0)) '(-2 -1 0 1 2))  ; => (1 2)

;; Custom predicate
(define (positive-even? x)
  (and (even? x) (positive? x)))

(filter positive-even? '(-4 -2 0 2 4 6))  ; => (2 4 6)
```

### Fold (Reduce)

```scheme
;; Left fold (fold-left)
(fold-left + 0 '(1 2 3 4))      ; => 10
(fold-left * 1 '(1 2 3 4))      ; => 24
(fold-left cons '() '(1 2 3))   ; => ((((() . 1) . 2) . 3)

;; Right fold (fold-right)
(fold-right + 0 '(1 2 3 4))     ; => 10
(fold-right cons '() '(1 2 3))  ; => (1 2 3)

;; Finding maximum
(fold-left max 0 '(3 7 2 9 1))  ; => 9
```

### Apply

```scheme
;; Apply function to list of arguments
(apply + '(1 2 3 4))            ; => 10
(apply max '(3 7 2 9 1))        ; => 9
(apply string-append '("Hello" " " "World"))  ; => "Hello World"

;; Combining with other functions
(apply + (map square '(1 2 3 4)))  ; => 30 (sum of squares)
```

## Error Handling

### Basic Error Reporting

```scheme
;; Signal errors with descriptive messages
(define (safe-divide x y)
  (if (= y 0)
      (error "Division by zero" x y)
      (/ x y)))

(safe-divide 10 2)      ; => 5
; (safe-divide 10 0)    ; Error: Division by zero 10 0
```

### Input Validation

```scheme
;; Validate function arguments
(define (factorial n)
  (cond
    [(not (integer? n))
     (error "factorial: argument must be integer" n)]
    [(negative? n)
     (error "factorial: argument must be non-negative" n)]
    [else
     (if (<= n 1) 1 (* n (factorial (- n 1))))]))

(factorial 5)           ; => 120
; (factorial -3)        ; Error: factorial: argument must be non-negative -3
; (factorial 3.5)       ; Error: factorial: argument must be integer 3.5
```

## Practice Exercises

### Exercise 1: Basic Functions

Write functions to solve these problems:

```scheme
;; 1. Convert temperature from Celsius to Fahrenheit
(define (celsius-to-fahrenheit c)
  ;; Your code here
  )

;; Test: (celsius-to-fahrenheit 0) should return 32
;; Test: (celsius-to-fahrenheit 100) should return 212

;; 2. Check if a number is even
(define (even? n)
  ;; Your code here
  )

;; Test: (even? 4) should return #t
;; Test: (even? 7) should return #f
```

### Exercise 2: List Processing

```scheme
;; 1. Find the sum of all numbers in a list
(define (sum-list lst)
  ;; Your code here
  )

;; Test: (sum-list '(1 2 3 4 5)) should return 15

;; 2. Count how many times an element appears in a list
(define (count-occurrences item lst)
  ;; Your code here
  )

;; Test: (count-occurrences 'a '(a b a c a)) should return 3
```

### Exercise 3: Recursion

```scheme
;; 1. Calculate the nth Fibonacci number efficiently
(define (fibonacci n)
  ;; Your code here (try both recursive and iterative approaches)
  )

;; 2. Flatten a nested list
(define (flatten lst)
  ;; Your code here
  )

;; Test: (flatten '((1 2) (3 (4 5)) 6)) should return (1 2 3 4 5 6)
```

### Solutions

<details>
<summary>Click to reveal solutions</summary>

```scheme
;; Exercise 1 Solutions
(define (celsius-to-fahrenheit c)
  (+ (* c 9/5) 32))

(define (my-even? n)
  (= (remainder n 2) 0))

;; Exercise 2 Solutions
(define (sum-list lst)
  (if (null? lst)
      0
      (+ (car lst) (sum-list (cdr lst)))))

(define (count-occurrences item lst)
  (cond
    [(null? lst) 0]
    [(equal? item (car lst)) 
     (+ 1 (count-occurrences item (cdr lst)))]
    [else 
     (count-occurrences item (cdr lst))]))

;; Exercise 3 Solutions
(define (fibonacci n)
  (define (fib-iter a b count)
    (if (= count 0)
        a
        (fib-iter b (+ a b) (- count 1))))
  (fib-iter 0 1 n))

(define (flatten lst)
  (cond
    [(null? lst) '()]
    [(pair? (car lst))
     (append (flatten (car lst)) (flatten (cdr lst)))]
    [else
     (cons (car lst) (flatten (cdr lst)))]))
```
</details>

## Next Steps

Congratulations! You've learned the basics of Lambdust programming. Next, explore:

1. **[Functional Programming](functional-programming.md)** - Advanced functional programming techniques
2. **[Type System](type-annotations.md)** - Adding type annotations to your code
3. **[Effect System](effect-management.md)** - Managing side effects
4. **[Standard Library](../api/stdlib/)** - Exploring built-in functions

Keep practicing with the REPL and try building small programs to solidify your understanding!