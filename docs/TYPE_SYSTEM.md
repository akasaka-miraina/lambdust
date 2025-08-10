# Type System Guide

Lambdust implements a sophisticated four-level gradual typing system that allows seamless progression from dynamic to fully static typing. This guide covers the complete type system implementation and usage patterns.

## Overview

The Lambdust type system provides:

- **Gradual Typing**: Smooth transition between dynamic and static typing
- **Type Inference**: Advanced Hindley-Milner with extensions
- **Algebraic Data Types**: Sum types and product types with pattern matching
- **Type Classes**: Haskell-style type constraints and polymorphism
- **Dependent Types**: Limited dependent type features for advanced users
- **Effect Types**: Integration with the algebraic effect system

## Type System Levels

### Level 1: Dynamic Typing

Pure dynamic typing with runtime type checking:

```scheme
;; No type annotations - fully dynamic
(define (add x y)
  (+ x y))

(add 1 2)        ;; => 3
(add 1.5 2.3)    ;; => 3.8
(add "hello" " world")  ;; Runtime error: + expects numbers
```

### Level 2: Optional Typing

Optional type annotations for documentation and basic checking:

```scheme
;; Optional type hints
(define (add x : Number y : Number) : Number
  (+ x y))

(define (greet name : String) : String
  (string-append "Hello, " name "!"))

;; Type hints are checked but not enforced
(add 1 2)        ;; => 3
(add 1 "2")      ;; Warning: type mismatch, but continues execution
```

### Level 3: Gradual Typing

Mixed static and dynamic typing with gradual enforcement:

```scheme
;; Statically typed function
(define (safe-add x : Number y : Number) : Number
  (+ x y))

;; Dynamically typed function
(define (flexible-add x y)
  (+ x y))

;; Gradual interaction
(define (mixed-computation data)
  (let ([typed-result : Number (safe-add 1 2)]
        [dynamic-result (flexible-add 3 4)])
    (+ typed-result dynamic-result)))

;; Type boundaries are enforced
(safe-add 1 "2")  ;; Type error: argument 2 must be Number
```

### Level 4: Static Typing

Full static typing with compile-time verification:

```scheme
#:type-level static

;; All functions must be fully typed
(define (factorial n : Natural) : Natural
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

;; Type checking prevents runtime errors
(define (process-data data : (List Number)) : Number
  (fold + 0 data))

;; Compile-time error prevention
(process-data '(1 2 "3"))  ;; Compile error: "3" is not Number
```

## Type Syntax

### Basic Types

```scheme
;; Primitive types
Number          ;; Floating-point numbers
Integer         ;; Integer numbers  
String          ;; Text strings
Boolean         ;; #t or #f
Character       ;; Single characters
Symbol          ;; Scheme symbols

;; Container types
(List Number)           ;; List of numbers
(Vector String)         ;; Vector of strings
(Pair Number String)    ;; Pair with typed components

;; Function types
(Number Number -> Number)       ;; Two numbers to number
(String -> Boolean)             ;; String to boolean
((List a) -> Number)            ;; Generic list to number
```

### Generic Types

```scheme
;; Type variables
(define (identity x : a) : a
  x)

(define (first lst : (List a)) : a
  (car lst))

;; Multiple type variables
(define (map f : (a -> b) lst : (List a)) : (List b)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; Type constraints
(define (sort lst : (List a)) : (List a)
  (where (Ord a)
    (quick-sort lst)))
```

## Algebraic Data Types

### Sum Types (Variants)

```scheme
;; Define a sum type
(define-type Color
  (Red)
  (Green)
  (Blue)
  (RGB Number Number Number)
  (HSV Number Number Number))

;; Constructor functions are automatically created
(define red-color (Red))
(define custom-color (RGB 255 128 0))

;; Pattern matching
(define (color-to-hex color : Color) : String
  (match color
    [(Red) "#FF0000"]
    [(Green) "#00FF00"] 
    [(Blue) "#0000FF"]
    [(RGB r g b) (format "#~2,'0X~2,'0X~2,'0X" 
                        (inexact->exact r)
                        (inexact->exact g)
                        (inexact->exact b))]
    [(HSV h s v) (rgb-to-hex (hsv->rgb h s v))]))
```

### Product Types (Records)

```scheme
;; Define a product type
(define-type Person
  (make-person name : String
               age : Number
               email : String))

;; Usage
(define john (make-person "John Doe" 30 "john@example.com"))

;; Field accessors (automatically generated)
(person-name john)    ;; => "John Doe"
(person-age john)     ;; => 30
(person-email john)   ;; => "john@example.com"

;; Pattern matching with records
(define (adult? person : Person) : Boolean
  (match person
    [(make-person _ age _) (>= age 18)]))
```

### Recursive Types

```scheme
;; Binary tree
(define-type (Tree a)
  (Empty)
  (Node (Tree a) a (Tree a)))

;; List definition
(define-type (MyList a)
  (Nil)
  (Cons a (MyList a)))

;; Usage
(define int-tree : (Tree Number)
  (Node (Node (Empty) 1 (Empty))
        2
        (Node (Empty) 3 (Empty))))

(define (tree-sum tree : (Tree Number)) : Number
  (match tree
    [(Empty) 0]
    [(Node left value right)
     (+ value (tree-sum left) (tree-sum right))]))
```

## Type Classes

Type classes provide structured polymorphism similar to Haskell's type classes:

### Defining Type Classes

```scheme
;; Basic type class
(define-type-class (Eq a)
  (equal? : a a -> Boolean)
  (not-equal? : a a -> Boolean))

;; Default implementations
(define-type-class (Eq a) 
  (equal? : a a -> Boolean)
  (not-equal? : a a -> Boolean)
  
  ;; Default implementation for not-equal?
  (default not-equal? (lambda (x y) (not (equal? x y)))))

;; Type class with dependencies
(define-type-class (Ord a)
  (super (Eq a))  ;; Ord requires Eq
  (compare : a a -> Ordering)
  (< : a a -> Boolean)
  (<= : a a -> Boolean)
  (> : a a -> Boolean)
  (>= : a a -> Boolean))
```

### Type Class Instances

```scheme
;; Implement Eq for Number
(define-instance (Eq Number)
  (define (equal? x y) (= x y)))

;; Implement Ord for Number
(define-instance (Ord Number)
  (define (compare x y)
    (cond [(< x y) 'LT]
          [(> x y) 'GT]
          [else 'EQ]))
  (define (< x y) (< x y))
  (define (<= x y) (<= x y))
  (define (> x y) (> x y))
  (define (>= x y) (>= x y)))

;; Generic functions using type classes
(define (sort lst : (List a)) : (List a)
  (where (Ord a)
    (merge-sort lst)))

;; Usage
(sort '(3 1 4 1 5 9))  ;; Works because Number implements Ord
```

### Advanced Type Classes

```scheme
;; Functor type class
(define-type-class (Functor f)
  (fmap : (a -> b) (f a) -> (f b)))

;; Monad type class
(define-type-class (Monad m)
  (super (Functor m))
  (return : a -> (m a))
  (bind : (m a) (a -> (m b)) -> (m b)))

;; Maybe type with Functor and Monad instances
(define-type (Maybe a)
  (Nothing)
  (Just a))

(define-instance (Functor Maybe)
  (define (fmap f maybe)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (Just (f x))])))

(define-instance (Monad Maybe)
  (define (return x) (Just x))
  (define (bind maybe f)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (f x)])))
```

## Type Inference

### Hindley-Milner Inference

The type system includes sophisticated type inference:

```scheme
;; No type annotations needed - types are inferred
(define (compose f g)
  (lambda (x) (f (g x))))
;; Inferred type: (b -> c) (a -> b) -> (a -> c)

(define (map f lst)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))
;; Inferred type: (a -> b) (List a) -> (List b)

;; Complex inference
(define (fold f init lst)
  (if (null? lst)
      init
      (fold f (f init (car lst)) (cdr lst))))
;; Inferred type: (b a -> b) b (List a) -> b
```

### Type Constraints

```scheme
;; Constrained type inference
(define (sort-by f lst)
  (sort (map f lst)))
;; Inferred type: (a -> b) (List a) -> (List b) where (Ord b)

(define (unique lst)
  (remove-duplicates lst))
;; Inferred type: (List a) -> (List a) where (Eq a)
```

## Integration with Effect System

Types can be combined with effects for precise tracking:

```scheme
;; Effect-typed functions
(define (read-file filename : String) : (IO String)
  (with-file-input filename
    (lambda (port)
      (read-string #f port))))

(define (write-log message : String) : (IO Unit)
  (with-file-output "app.log"
    (lambda (port)
      (write-line message port))))

;; Combined effects and types
(define (process-data filename : String) : (IO (Either Error (List Number)))
  (do [content (read-file filename)]
      [parsed (parse-numbers content)]
      [processed (map square parsed)]
      [_ (write-log (format "Processed ~a numbers" (length processed)))]
      (return (Right processed))))
```

## Error Handling with Types

### Result Types

```scheme
(define-type (Result a e)
  (Ok a)
  (Error e))

(define (safe-divide x : Number y : Number) : (Result Number String)
  (if (= y 0)
      (Error "Division by zero")
      (Ok (/ x y))))

;; Monadic error handling
(define-instance (Monad (Result e))
  (define (return x) (Ok x))
  (define (bind result f)
    (match result
      [(Error e) (Error e)]
      [(Ok x) (f x)])))

;; Usage
(define computation
  (do [x (safe-divide 10 2)]    ;; Ok 5
      [y (safe-divide x 0)]     ;; Error "Division by zero"
      [z (safe-divide y 3)]     ;; Skipped due to error
      (return z)))
```

## Performance Considerations

### Type Specialization

The type system enables performance optimizations:

```scheme
;; Generic function
(define (sum lst : (List a)) : a
  (where (Num a)
    (fold + (zero) lst)))

;; Specialized versions are generated automatically
;; sum-number : (List Number) -> Number    ; Optimized for numbers
;; sum-complex : (List Complex) -> Complex ; Optimized for complex numbers
```

### Compile-time Optimizations

```scheme
#:optimize-types #t

;; Type information enables:
;; - Inlining of type-specific operations  
;; - Elimination of runtime type checks
;; - SIMD optimizations for numeric types
;; - Memory layout optimizations

(define (vector-add v1 : (Vector Number) v2 : (Vector Number)) : (Vector Number)
  ;; Compiled to optimized SIMD operations
  (vector-map + v1 v2))
```

## Advanced Features

### Dependent Types (Limited)

```scheme
;; Length-indexed vectors
(define-type (Vec n a)
  (make-vec (vector a) (= (vector-length vector) n)))

(define (safe-head vec : (Vec (> n 0) a)) : a
  (vector-ref (vec-data vec) 0))

;; Refinement types
(define-type Positive (and Number (> x 0)))
(define-type NonEmptyString (and String (> (string-length x) 0)))

(define (sqrt-positive x : Positive) : Positive
  (sqrt x))  ;; Type system guarantees x > 0 and result > 0
```

### Type-level Programming

```scheme
;; Type-level computations
(define-type-function (Replicate n a)
  (if (= n 0)
      '()
      (cons a (Replicate (- n 1) a))))

;; Usage
(define tuple : (Replicate 3 Number)
  '(1 2 3))  ;; Type: (Number Number Number)
```

## Configuration

### Type System Settings

```scheme
;; Global type system configuration
(set-type-level! 'gradual)          ;; Set default type level
(set-type-inference! #t)            ;; Enable type inference
(set-type-optimization! #t)         ;; Enable type-based optimizations
(set-type-warnings! 'strict)        ;; Warning level for type mismatches

;; Module-specific settings
#:type-level static                 ;; This module uses static typing
#:type-inference aggressive         ;; Use aggressive inference
#:type-checking strict             ;; Strict type checking
```

## Examples

### Complete Type System Usage

```scheme
#!/usr/bin/env lambdust
#:type-level gradual

(import (scheme base)
        (lambdust types)
        (lambdust effects))

;; Define a complete data processing pipeline with types

;; Custom data types
(define-type (Employee)
  (make-employee name : String
                 id : Integer
                 salary : Number
                 department : String))

(define-type Department
  (Engineering)
  (Sales)  
  (Marketing)
  (HR))

;; Type classes for our domain
(define-type-class (Payroll a)
  (calculate-pay : a -> Number)
  (tax-rate : a -> Number))

(define-instance (Payroll Employee)
  (define (calculate-pay emp)
    (* (employee-salary emp) 0.8))  ;; After deductions
  (define (tax-rate emp)
    (cond [(> (employee-salary emp) 100000) 0.3]
          [(> (employee-salary emp) 50000) 0.25]
          [else 0.2])))

;; Effectful computation with types
(define (process-payroll employees : (List Employee)) : (IO (List Number))
  (do [_ (log-info "Starting payroll processing")]
      [payments (map calculate-pay employees)]
      [total (sum payments)]
      [_ (log-info (format "Total payroll: $~a" total))]
      [_ (write-payroll-report employees payments)]
      (return payments)))

;; Safe computation with error handling
(define (load-employee-data filename : String) : (IO (Result (List Employee) String))
  (guard (condition
          [(file-not-found? condition)
           (return (Error "Employee data file not found"))]
          [(parse-error? condition) 
           (return (Error "Invalid employee data format"))])
    (do [content (read-file filename)]
        [employees (parse-employees content)]
        (return (Ok employees)))))

;; Main program
(define (main)
  (do [result (load-employee-data "employees.json")]
      (match result
        [(Error msg) 
         (log-error msg)
         (exit 1)]
        [(Ok employees)
         (do [payments (process-payroll employees)]
             [_ (log-info "Payroll processing complete")]
             (return payments))])))

(when (script-file?)
  (main))
```

This type system provides a solid foundation for building reliable, efficient, and maintainable Scheme programs while preserving the flexibility that makes Lisp languages powerful.