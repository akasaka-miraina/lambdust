# 🚀 Polynomial Universe Type System - Quick Start Guide

**"It's not Scheme, it's Lambdust!"** - Getting Started with Revolutionary Type System

## 🌟 What is Polynomial Universe Type System?

Lambdustに実装された**Polynomial Universe型システム**は、arXiv:2409.19176「Polynomial Universes in Homotopy Type Theory」の最新研究に基づく、世界最先端の型システムです。

### Key Features
- **Dependent Types**: Π-types (dependent functions), Σ-types (dependent products)
- **Universe Hierarchy**: Type₀ : Type₁ : Type₂ : ...
- **Polynomial Functors**: Category theory-based type constructors
- **Monad Algebra**: Distributive laws between monads
- **Type Inference**: Hindley-Milner with constraint solving

## 🎯 Basic Usage

### 1. Custom Type Predicates

```scheme
;; Define custom type predicate
(define-predicate "positive-integer" 
  (lambda (x) (and (integer? x) (> x 0))))

;; Use the predicate
(positive-integer? 42)    ; => #t
(positive-integer? -5)    ; => #f
(positive-integer? "hi")  ; => #f

;; List all defined predicates
(list-predicates)
; => ("positive-integer" ...)
```

### 2. Optional Type Annotations (R7RS Compatible)

```scheme
;; Traditional Scheme (works as before)
(define (factorial n)
  (if (= n 0) 1 (* n (factorial (- n 1)))))

;; With optional type annotations
(define (factorial [n : Natural]) : Natural
  (if (= n 0) 1 (* n (factorial (- n 1)))))

;; Function types
(define (map [f : (A → B)] [lst : List A]) : List B
  (if (null? lst)
      '()
      (cons (f (car lst)) (map f (cdr lst)))))
```

### 3. Advanced Type Features

```scheme
;; Dependent types with preconditions
(define (safe-divide [x : Real] [y : Real]) : Real
  (where (≠ y 0))  ; precondition
  (/ x y))

;; Vector types with length information
(define (vector-ref [v : Vector A n] [i : Natural]) : A
  (where (< i n))  ; bounds check
  (vector-ref v i))

;; Polynomial functors
(define (maybe-map [f : (A → B)] [m : Maybe A]) : Maybe B
  (case m
    [(Nothing) Nothing]
    [(Just x) (Just (f x))]))
```

## 🔧 Type System API

### Type Checking

```scheme
;; Enable type system (optional feature flag)
(use-type-system #t)

;; Check type of expression
(type-of 42)                    ; => Integer
(type-of "hello")               ; => String
(type-of '(1 2 3))             ; => List Integer

;; Type check value against expected type
(type-check 42 Integer)         ; => #t
(type-check "hi" Integer)       ; => #f

;; Infer most general type
(infer-type (lambda (x) (+ x 1)))  ; => (Integer → Integer)
```

### Monad Operations

```scheme
;; Apply distributive law
(define state-maybe (compose-monads State Maybe))

;; Pi-over-Sigma distributive law
(apply-distributive-law "pi-over-sigma" 
  '(lambda (x) (pair x (f x))))
```

## 📚 Type Hierarchy

### Base Types
- `Natural` (ℕ): 0, 1, 2, 3, ...
- `Integer` (ℤ): ..., -2, -1, 0, 1, 2, ...
- `Real` (ℝ): floating point numbers
- `Boolean` (𝔹): #t, #f
- `String`: "text"
- `Character`: #\a, #\b, ...
- `Symbol`: 'foo, 'bar, ...

### Type Constructors
- `A → B`: Function from A to B
- `A × B`: Product (pair) of A and B
- `A + B`: Sum (union) of A and B
- `List A`: List of elements of type A
- `Vector A n`: Vector of n elements of type A

### Dependent Types
- `Π(x:A).B(x)`: Dependent function type
- `Σ(x:A).B(x)`: Dependent product type

### Universe Hierarchy
- `Type₀`: Base types and their constructors
- `Type₁`: Types of Type₀
- `Type₂`: Types of Type₁
- ...

## 🧪 Examples

### 1. Safe List Operations

```scheme
;; Type-safe head function
(define (safe-head [lst : List A]) : Maybe A
  (if (null? lst)
      Nothing
      (Just (car lst))))

;; Length-indexed vectors
(define (vector-append [v1 : Vector A m] [v2 : Vector A n]) : Vector A (+ m n)
  ...)
```

### 2. Dependent Types Example

```scheme
;; Matrix with dimensions
(define (matrix-multiply 
  [m1 : Matrix A rows cols1] 
  [m2 : Matrix A cols1 cols2]) : Matrix A rows cols2
  ...)

;; Proof-carrying code
(define (sorted-insert 
  [x : A] 
  [lst : SortedList A]) : SortedList A
  ...)
```

### 3. Monad Transformers

```scheme
;; State monad transformer
(define (run-state-t [comp : StateT S M A] [initial : S]) : M (Pair A S)
  ...)

;; Combining effects
(do [x <- (get-state)]
    [y <- (lift (read-file "data.txt"))]
    [_ <- (put-state (+ x 1))]
    (return (string-append y (number->string x))))
```

## 🎯 Performance Benefits

### Compile-time Guarantees
- **Type Safety**: Catch type errors at compile time
- **Memory Safety**: Bounds checking for vectors/arrays
- **Effect Tracking**: Control side effects with types

### Runtime Optimizations
- **Specialized Code**: Generate optimized code for known types
- **Unboxing**: Remove boxing overhead for primitive types
- **Parallel Processing**: Type-safe parallelization

## 🔧 Configuration

### Enable Type System

```scheme
;; In your Scheme code
(use-type-system #t)

;; Or via feature flag
;; cargo run --features type-system
```

### Type Checking Modes

```scheme
;; Strict mode (all expressions must type check)
(set-type-checking-mode! 'strict)

;; Gradual mode (optional type annotations)
(set-type-checking-mode! 'gradual)

;; Inference mode (automatic type inference)
(set-type-checking-mode! 'inference)
```

## 🏆 Why Polynomial Universe?

### Academic Value
- **Latest Research**: Based on cutting-edge HoTT research
- **Mathematical Rigor**: Formal foundation with provable properties
- **Category Theory**: Clean mathematical abstractions

### Practical Benefits
- **Better Error Messages**: Precise type information
- **IDE Integration**: Rich type information for tooling
- **Performance**: Compile-time optimizations
- **Correctness**: Eliminate whole classes of bugs

### GHC Challenge
**"Same type safety, faster compilation than GHC"**
- **Parallel Type Checking**: Multi-threaded vs single-threaded
- **Incremental Inference**: Differential solving vs full recomputation
- **Zero-cost Abstractions**: Rust performance vs Haskell GC

## 🌟 What's Next?

### Phase 7: HoTT Type Classes + Monad Syntax
```scheme
;; Type classes with mathematical laws
(define-type-class (Functor F) 
  (fmap : (Π [A B : Type] (A → B) → F A → F B))
  (fmap-id : (Π [A : Type] (fmap id) ≡ id)))

;; Do notation
(do [x <- (return 42)]
    [y <- (Just 10)]
    (return (+ x y)))
```

### Performance Optimization
- Parallel type checking
- SIMD-optimized constraint solving
- CoW memory optimization (25-40% reduction)

---

**Welcome to the future of functional programming!** 🚀

*Lambdust: Where Theory Meets Practice*