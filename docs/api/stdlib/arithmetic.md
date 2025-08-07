# Arithmetic Operations

The arithmetic module provides comprehensive number operations and mathematical functions following R7RS specifications with Lambdust extensions.

## Overview

Lambdust supports multiple numeric types and provides efficient arithmetic operations with automatic type promotion and overflow handling.

### Numeric Types

- **Integer**: Arbitrary precision integers
- **Rational**: Exact rational numbers (fractions)
- **Real**: Floating-point numbers (IEEE 754 double precision)
- **Complex**: Complex numbers with real and imaginary parts

## Basic Arithmetic

### Addition

```scheme
(+ number ...)
```

Returns the sum of its arguments. With no arguments, returns 0.

**Type**: `(-> Number ... Number)`

**Examples**:
```scheme
(+)              ; => 0
(+ 5)            ; => 5
(+ 1 2 3)        ; => 6
(+ 1.5 2.5)      ; => 4.0
(+ 1/2 1/3)      ; => 5/6
```

**Errors**:
- Type error if any argument is not a number

### Subtraction

```scheme
(- number)
(- number number ...)
```

With one argument, returns the negation. With multiple arguments, subtracts subsequent arguments from the first.

**Type**: `(-> Number Number ... Number)`

**Examples**:
```scheme
(- 5)            ; => -5
(- 10 3)         ; => 7
(- 10 3 2)       ; => 5
(- 1/2)          ; => -1/2
```

**Errors**:
- Arity error if called with no arguments
- Type error if any argument is not a number

### Multiplication

```scheme
(* number ...)
```

Returns the product of its arguments. With no arguments, returns 1.

**Type**: `(-> Number ... Number)`

**Examples**:
```scheme
(*)              ; => 1
(* 5)            ; => 5
(* 2 3 4)        ; => 24
(* 1.5 2)        ; => 3.0
(* 2/3 3/4)      ; => 1/2
```

### Division

```scheme
(/ number)
(/ number number ...)
```

With one argument, returns the reciprocal. With multiple arguments, divides the first by the product of the rest.

**Type**: `(-> Number Number ... Number)`

**Examples**:
```scheme
(/ 4)            ; => 1/4
(/ 10 2)         ; => 5
(/ 12 2 3)       ; => 2
(/ 1 3)          ; => 1/3
```

**Errors**:
- Division by zero error
- Arity error if called with no arguments

## Integer Operations

### Quotient and Remainder

```scheme
(quotient n1 n2)
(remainder n1 n2)
(modulo n1 n2)
```

Integer division operations.

**Type**: `(-> Integer Integer Integer)`

**Examples**:
```scheme
(quotient 13 4)   ; => 3
(remainder 13 4)  ; => 1
(modulo 13 4)     ; => 1
(modulo -13 4)    ; => 3  ; always positive with positive divisor
```

### GCD and LCM

```scheme
(gcd n1 ...)
(lcm n1 ...)
```

Greatest common divisor and least common multiple.

**Type**: `(-> Integer ... Integer)`

**Examples**:
```scheme
(gcd 12 18)       ; => 6
(gcd 12 18 24)    ; => 6
(lcm 12 18)       ; => 36
(lcm 12 18 24)    ; => 72
```

## Comparison Operations

### Numeric Equality

```scheme
(= number number ...)
```

Tests numeric equality.

**Type**: `(-> Number Number ... Boolean)`

**Examples**:
```scheme
(= 5 5)           ; => #t
(= 5 5.0)         ; => #t (numeric equality)
(= 1/2 0.5)       ; => #t
(= 1 2 3)         ; => #f
```

### Ordering Relations

```scheme
(< number number ...)
(> number number ...)
(<= number number ...)
(>= number number ...)
```

Numeric ordering predicates.

**Type**: `(-> Number Number ... Boolean)`

**Examples**:
```scheme
(< 1 2 3)         ; => #t
(> 5 4 3)         ; => #t
(<= 1 1 2)        ; => #t
(>= 3 2 2)        ; => #t
```

## Mathematical Functions

### Absolute Value

```scheme
(abs number)
```

Returns the absolute value.

**Type**: `(-> Number Number)`

**Examples**:
```scheme
(abs -5)          ; => 5
(abs 3.14)        ; => 3.14
(abs -1/2)        ; => 1/2
```

### Min and Max

```scheme
(min number number ...)
(max number number ...)
```

Returns the minimum or maximum of the arguments.

**Type**: `(-> Number Number ... Number)`

**Examples**:
```scheme
(min 3 1 4 1 5)   ; => 1
(max 3 1 4 1 5)   ; => 5
```

### Rounding Functions

```scheme
(floor number)
(ceiling number)
(truncate number)
(round number)
```

Rounding operations.

**Type**: `(-> Number Number)`

**Examples**:
```scheme
(floor 3.7)       ; => 3.0
(ceiling 3.2)     ; => 4.0
(truncate -3.7)   ; => -3.0
(round 3.5)       ; => 4.0  ; rounds to even
```

### Exponential and Logarithmic

```scheme
(exp number)
(log number)
(log number base)
(sqrt number)
(expt number number)
```

Exponential and logarithmic functions.

**Type**: `(-> Number Number)` or `(-> Number Number Number)`

**Examples**:
```scheme
(exp 1)           ; => 2.718281828459045
(log 10)          ; => 2.302585092994046
(log 100 10)      ; => 2.0
(sqrt 9)          ; => 3.0
(expt 2 8)        ; => 256
```

### Trigonometric Functions

```scheme
(sin number)
(cos number)
(tan number)
(asin number)
(acos number)
(atan number)
(atan number number)
```

Trigonometric functions (angles in radians).

**Type**: `(-> Number Number)` or `(-> Number Number Number)`

**Examples**:
```scheme
(sin 0)           ; => 0.0
(cos 0)           ; => 1.0
(tan 0)           ; => 0.0
(atan 1)          ; => 0.7853981633974483
(atan 1 1)        ; => 0.7853981633974483
```

## Type Conversion

### Exact and Inexact

```scheme
(exact number)
(inexact number)
(exact? number)
(inexact? number)
```

Exact/inexact conversion and predicates.

**Examples**:
```scheme
(exact 3.14)      ; => 7853981633974483/2500000000000000
(inexact 22/7)    ; => 3.142857142857143
(exact? 1/2)      ; => #t
(inexact? 3.14)   ; => #t
```

### Number to String

```scheme
(number->string number)
(number->string number radix)
```

Convert number to string representation.

**Type**: `(-> Number String)` or `(-> Number Integer String)`

**Examples**:
```scheme
(number->string 123)     ; => "123"
(number->string 255 16)  ; => "ff"
(number->string 1/2)     ; => "1/2"
```

### String to Number

```scheme
(string->number string)
(string->number string radix)
```

Parse string as number.

**Type**: `(-> String Number)` or `(-> String Integer Number)`

**Examples**:
```scheme
(string->number "123")     ; => 123
(string->number "ff" 16)   ; => 255
(string->number "1/2")     ; => 1/2
(string->number "invalid") ; => #f
```

## Complex Numbers

### Complex Construction

```scheme
(make-rectangular real imag)
(make-polar magnitude angle)
```

Create complex numbers.

**Type**: `(-> Number Number Complex)`

**Examples**:
```scheme
(make-rectangular 3 4)    ; => 3+4i
(make-polar 5 0)          ; => 5+0i
```

### Complex Accessors

```scheme
(real-part complex)
(imag-part complex)
(magnitude complex)
(angle complex)
```

Access complex number components.

**Type**: `(-> Complex Number)`

**Examples**:
```scheme
(real-part 3+4i)      ; => 3
(imag-part 3+4i)      ; => 4
(magnitude 3+4i)      ; => 5.0
(angle 1+1i)          ; => 0.7853981633974483
```

## Numeric Predicates

### Type Predicates

```scheme
(number? obj)
(integer? obj)
(rational? obj)
(real? obj)
(complex? obj)
```

Test numeric types.

**Examples**:
```scheme
(number? 42)      ; => #t
(integer? 42)     ; => #t
(rational? 1/2)   ; => #t
(real? 3.14)      ; => #t
(complex? 3+4i)   ; => #t
```

### Property Predicates

```scheme
(zero? number)
(positive? number)
(negative? number)
(odd? integer)
(even? integer)
```

Test numeric properties.

**Examples**:
```scheme
(zero? 0)         ; => #t
(positive? 5)     ; => #t
(negative? -3)    ; => #t
(odd? 7)          ; => #t
(even? 4)         ; => #t
```

## Extended Mathematical Functions

### Hyperbolic Functions

```scheme
(sinh number)
(cosh number)
(tanh number)
(asinh number)
(acosh number)
(atanh number)
```

Hyperbolic trigonometric functions.

### Special Functions

```scheme
(gamma number)          ; Gamma function
(factorial integer)     ; Factorial (extension)
(binomial n k)         ; Binomial coefficient (extension)
```

## Random Numbers

```scheme
(random)               ; Random float [0,1)
(random integer)       ; Random integer [0,n)
(random-seed integer)  ; Set random seed
```

**Examples**:
```scheme
(random)          ; => 0.7853981633974483
(random 10)       ; => 7
(random-seed 42)  ; => unspecified
```

## Performance Considerations

### Optimization Notes

- Integer arithmetic uses arbitrary precision with overflow to rationals
- Common operations are optimized for small integers
- Complex operations are optimized for rectangular form
- Trigonometric functions use hardware acceleration when available

### Memory Usage

- Small integers are unboxed for efficiency
- Rationals automatically reduce to lowest terms
- Complex numbers with zero imaginary part become real

## Error Handling

All arithmetic functions provide detailed error messages:

```scheme
(+ 1 "hello")
; Error: +: contract violation
;   expected: number
;   given: "hello"

(/ 0)
; Error: /: division by zero

(sqrt -1)
; Result: 0+1i (returns complex result)
```

## Threading and Concurrency

All arithmetic operations are thread-safe and can be called concurrently without synchronization.