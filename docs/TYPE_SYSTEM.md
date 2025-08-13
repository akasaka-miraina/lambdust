# Lambdust Type System Guide

This document provides comprehensive documentation of Lambdust's four-level gradual type system, which seamlessly combines dynamic typing with static analysis and dependent types.

## Table of Contents

1. [Type System Overview](#type-system-overview)
2. [Four Typing Levels](#four-typing-levels)
3. [Type Inference Engine](#type-inference-engine)
4. [Type Classes and Constraints](#type-classes-and-constraints)
5. [Algebraic Data Types](#algebraic-data-types)
6. [Gradual Typing Integration](#gradual-typing-integration)
7. [Advanced Features](#advanced-features)
8. [Best Practices](#best-practices)

## Type System Overview

Lambdust's type system is designed to provide maximum flexibility while enabling powerful static analysis and optimization:

### **Type System Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Dynamic      │    │   Contracts     │    │    Static       │
│    Types        │────┤                 ├────┤   Inference     │
│                 │    │ • Runtime Check │    │                 │
│ • R7RS Compat   │    │ • Gradual Opt   │    │ • HM Inference  │
│ • Zero Overhead │    │ • Soft Typing   │    │ • Unification   │
│ • Full Scheme   │    │ • Type Guards   │    │ • Polymorphism  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                        ┌─────────────────┐
                        │   Dependent     │
                        │    Types        │
                        │                 │
                        │ • Proof Objects │
                        │ • Refinements   │
                        │ • Verification  │
                        └─────────────────┘
```

### **Key Features**

- **Gradual Progression**: Smooth transition between typing levels
- **Backward Compatibility**: Full R7RS compatibility in dynamic mode
- **Zero Runtime Overhead**: Types are erased unless specifically checked
- **Inference Engine**: Powerful Hindley-Milner type inference
- **Type Classes**: Haskell-style constrained polymorphism
- **Row Polymorphism**: Extensible records and variants
- **Effect Integration**: Types integrated with effect system

## Four Typing Levels

### **1. Dynamic Typing (Level 0)**

The default mode providing full R7RS Scheme compatibility:

```scheme
;; Pure dynamic typing - no type annotations needed
(define (factorial n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

;; All Scheme operations work as expected
(factorial 5)     ; => 120
(factorial 5.0)   ; => 120.0
(factorial "5")   ; => Runtime error with helpful message
```

**Characteristics:**
- Zero type annotations required
- Full R7RS compatibility
- Runtime type checking only
- Maximum flexibility

### **2. Contract Typing (Level 1)**

Runtime type checking with gradual optimization:

```scheme
;; Contract-based typing with runtime verification
(define (safe-divide x y)
  #:contract (-> (and Number (not zero?)) 
                 (and Number (not zero?)) 
                 Number)
  (/ x y))

;; Contracts are checked at runtime
(safe-divide 10 2)  ; => 5 (contract satisfied)
(safe-divide 10 0)  ; => Contract violation: y must be non-zero

;; Contracts can be complex predicates
(define (process-list lst)
  #:contract (-> (List-of Number) (List-of Number))
  (map square lst))

;; Contract checking with custom predicates
(define positive-integer?
  (lambda (x)
    (and (integer? x) (positive? x))))

(define (safe-nth lst n)
  #:contract (-> (List-of Any) positive-integer? Any)
  (list-ref lst (- n 1)))
```

**Contract Features:**
- Runtime type checking
- Custom predicate support  
- Gradual optimization opportunities
- Clear error messages with blame assignment

### **3. Static Typing (Level 2)**

Compile-time type checking with Hindley-Milner inference:

```scheme
;; Type annotations enable static checking
(define (typed-map f lst)
  #:type (∀ (a b) (-> (-> a b) (List a) (List b)))
  (if (null? lst)
      '()
      (cons (f (car lst))
            (typed-map f (cdr lst)))))

;; Type inference works across function boundaries
(define numbers '(1 2 3 4 5))
(define squares (typed-map square numbers))
; Inferred type: (List Number)

;; Higher-order functions with type inference
(define (compose f g)
  #:type (∀ (a b c) (-> (-> b c) (-> a b) (-> a c)))
  (lambda (x) (f (g x))))

(define add1 (lambda (x) (+ x 1)))
(define double (lambda (x) (* x 2)))
(define add1-then-double (compose double add1))
; Inferred type: (-> Number Number)
```

**Static Typing Features:**
- Hindley-Milner type inference
- Parametric polymorphism
- Type error detection at compile time
- Optimization opportunities

### **4. Dependent Typing (Level 3)**

Types that depend on values, enabling verification:

```scheme
;; Dependent types with refinement predicates
(define-type Nat Number (lambda (n) (>= n 0)))
(define-type Vec (n : Nat) (vector-of-length n Any))

;; Length-indexed vectors
(define (safe-head v)
  #:type (∀ (n : Nat) (-> (Vec (+ n 1)) Any))
  (vector-ref v 0))

;; Proof-carrying code
(define (binary-search arr key)
  #:type (-> (Sorted-Vector Number) Number (Maybe Nat))
  #:requires (sorted? arr)
  #:ensures (lambda (result)
              (match result
                [(Just idx) (= (vector-ref arr idx) key)]
                [Nothing (not (member key (vector->list arr)))]))
  (binary-search-impl arr key 0 (- (vector-length arr) 1)))

;; Liquid types for precise specifications
(define (divide x y)
  #:type (-> x:Number {y:Number | y ≠ 0} {r:Number | r = x/y})
  (/ x y))
```

**Dependent Features:**
- Value-dependent types
- Refinement predicates
- Proof obligations
- Verification conditions

## Type Inference Engine

### **Hindley-Milner Algorithm** (`src/types/inference.rs`)

The inference engine implements Algorithm W with extensions:

```rust
pub struct TypeInferer {
    substitution: Substitution,
    constraints: Vec<Constraint>,
    type_env: TypeEnv,
    fresh_var_generator: FreshVarGen,
}

impl TypeInferer {
    /// Infer the type of an expression
    pub fn infer(&mut self, expr: &Spanned<Expr>) -> InferResult<Type> {
        match &expr.inner {
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),
            Expr::Symbol(name) => self.infer_variable(name),
            Expr::Lambda { formals, body } => self.infer_lambda(formals, body),
            Expr::Application { operator, operands } => {
                self.infer_application(operator, operands)
            }
            // ... other expression types
        }
    }
    
    /// Generate constraints and solve them
    fn infer_application(&mut self, op: &Spanned<Expr>, args: &[Spanned<Expr>]) 
                        -> InferResult<Type> {
        let op_type = self.infer(op)?;
        let arg_types: Result<Vec<_>, _> = args.iter().map(|arg| self.infer(arg)).collect();
        let arg_types = arg_types?;
        
        let result_type = self.fresh_type_var();
        let function_type = Type::function(arg_types, result_type.clone());
        
        self.add_constraint(Constraint::Equal(op_type, function_type))?;
        self.solve_constraints()?;
        
        Ok(self.apply_substitution(&result_type))
    }
}
```

### **Type Inference Examples**

#### **Automatic Type Inference**
```scheme
;; No type annotations needed - types are inferred
(define id (lambda (x) x))
; Inferred: ∀a. a -> a

(define const (lambda (x y) x))  
; Inferred: ∀a b. a -> b -> a

(define flip (lambda (f x y) (f y x)))
; Inferred: ∀a b c. (a -> b -> c) -> b -> a -> c

;; Complex inference across multiple functions
(define apply-twice (lambda (f x) (f (f x))))
; Inferred: ∀a. (a -> a) -> a -> a

(define increment (lambda (n) (+ n 1)))
; Inferred: Number -> Number

(define result (apply-twice increment 5))
; Inferred: Number, evaluates to 7
```

#### **Polymorphic Type Inference**
```scheme
;; Generic list operations
(define length
  (lambda (lst)
    (if (null? lst)
        0
        (+ 1 (length (cdr lst))))))
; Inferred: ∀a. List a -> Number

(define map
  (lambda (f lst)
    (if (null? lst)
        '()
        (cons (f (car lst)) (map f (cdr lst))))))
; Inferred: ∀a b. (a -> b) -> List a -> List b

;; Higher-order function inference
(define fold-right
  (lambda (f init lst)
    (if (null? lst)
        init
        (f (car lst) (fold-right f init (cdr lst))))))
; Inferred: ∀a b. (a -> b -> b) -> b -> List a -> b
```

### **Error Messages and Debugging**

The type system provides detailed error messages:

```scheme
;; Type mismatch example
(define bad-function
  (lambda (x)
    (+ x "hello")))

;; Error message:
;; Type Error in bad-function at line 3:
;;   Cannot unify Number with String
;;   In expression: (+ x "hello")  
;;   Expected: Number
;;   Actual: String
;;   
;; Type inference trace:
;;   x : Number (from + constraint)
;;   "hello" : String
;;   + : Number -> Number -> Number
;;   Cannot satisfy: String <: Number
```

## Type Classes and Constraints

### **Type Class System** (`src/types/type_classes.rs`)

Lambdust supports Haskell-style type classes for constrained polymorphism:

```rust
/// Type class definition
pub struct TypeClass {
    pub name: String,
    pub parameters: Vec<TypeVar>,
    pub superclasses: Vec<Constraint>,
    pub methods: Vec<Method>,
}

/// Type class instance
pub struct Instance {
    pub class: String,
    pub types: Vec<Type>,
    pub constraints: Vec<Constraint>,
    pub implementations: HashMap<String, Value>,
}
```

### **Built-in Type Classes**

#### **Eq - Equality Testing**
```scheme
;; Type class for equality
(define-class (Eq a)
  (= : a -> a -> Boolean)
  (≠ : a -> a -> Boolean))

;; Default implementation for ≠
(define-default ≠ (lambda (x y) (not (= x y))))

;; Instances for built-in types
(define-instance (Eq Number)
  (= number=?))

(define-instance (Eq String)  
  (= string=?))

;; Derived instance for lists
(define-instance (Eq a) => (Eq (List a))
  (= (lambda (xs ys)
       (cond
         [(and (null? xs) (null? ys)) #t]
         [(or (null? xs) (null? ys)) #f]
         [else (and (= (car xs) (car ys))
                    (= (cdr xs) (cdr ys)))]))))
```

#### **Ord - Ordering**
```scheme
;; Ordering type class
(define-class (Eq a) => (Ord a)
  (compare : a -> a -> Ordering)
  (<  : a -> a -> Boolean)
  (<= : a -> a -> Boolean)
  (>  : a -> a -> Boolean)  
  (>= : a -> a -> Boolean))

;; Ordering enumeration
(define-type Ordering
  LT  ; Less than
  EQ  ; Equal
  GT) ; Greater than

;; Minimal complete definition
(define-default < (lambda (x y) (eq? (compare x y) 'LT)))
(define-default <= (lambda (x y) (not (> x y))))
(define-default > (lambda (x y) (eq? (compare x y) 'GT)))
(define-default >= (lambda (x y) (not (< x y))))
```

#### **Show - String Representation**
```scheme
;; String representation type class
(define-class (Show a)
  (show : a -> String))

;; Instances
(define-instance (Show Number)
  (show number->string))

(define-instance (Show String)
  (show (lambda (s) (string-append "\"" s "\""))))

(define-instance (Show Boolean)
  (show (lambda (b) (if b "true" "false"))))

;; Generic show for lists
(define-instance (Show a) => (Show (List a))
  (show (lambda (lst)
          (string-append "["
                         (string-join (map show lst) ", ")
                         "]"))))
```

### **Custom Type Classes**

```scheme
;; Define a numeric type class
(define-class (Numeric a)
  (+ : a -> a -> a)
  (* : a -> a -> a)
  (negate : a -> a)
  (abs : a -> a)
  (signum : a -> a))

;; Vector space type class
(define-class (Numeric a) => (VectorSpace a)
  (scalar-multiply : Number -> a -> a)
  (vector-add : a -> a -> a)
  (zero : a))

;; 2D point instance
(define-type Point2D Number Number)

(define-instance (VectorSpace Point2D)
  (scalar-multiply 
    (lambda (scalar point)
      (match point
        [(Point2D x y) (Point2D (* scalar x) (* scalar y))])))
  (vector-add
    (lambda (p1 p2)
      (match (list p1 p2)
        [(list (Point2D x1 y1) (Point2D x2 y2))
         (Point2D (+ x1 x2) (+ y1 y2))])))
  (zero (Point2D 0 0)))
```

## Algebraic Data Types

### **ADT Definition and Usage**

```scheme
;; Simple enumeration
(define-type Color
  Red
  Green  
  Blue)

;; Parametric data type
(define-type Maybe (a)
  Nothing
  (Just a))

;; Recursive data type
(define-type Tree (a)
  Leaf
  (Node a (Tree a) (Tree a)))

;; Multiple type parameters
(define-type Either (a b)
  (Left a)
  (Right b))
```

### **Pattern Matching**

```scheme
;; Pattern matching with match
(define (maybe-map f maybe-val)
  (match maybe-val
    [Nothing Nothing]
    [(Just x) (Just (f x))]))

;; Pattern matching with multiple cases
(define (tree-size tree)
  (match tree
    [Leaf 0]
    [(Node _ left right) 
     (+ 1 (tree-size left) (tree-size right))]))

;; Pattern matching with guards
(define (classify-number n)
  (match n
    [x #:when (= x 0) 'zero]
    [x #:when (> x 0) 'positive]
    [x #:when (< x 0) 'negative]))

;; Nested pattern matching
(define (unwrap-nested maybe-maybe)
  (match maybe-maybe
    [Nothing Nothing]
    [(Just Nothing) Nothing]
    [(Just (Just x)) (Just x)]))
```

### **Pattern Guards and Complex Patterns**

```scheme
;; Pattern guards for conditional matching
(define (safe-head lst)
  (match lst
    [(cons x _) #:when (not (null? lst)) (Just x)]
    [_ Nothing]))

;; Variable patterns and wildcards
(define (second lst)
  (match lst
    [(cons _ (cons x _)) (Just x)]
    [_ Nothing]))

;; As-patterns (binding whole and parts)
(define (duplicate-head lst)
  (match lst
    [(cons x tail) #:as original-list
     (cons x original-list)]
    [empty-list empty-list]))
```

## Gradual Typing Integration

### **Type Level Transitions** (`src/types/gradual.rs`)

The gradual typing system enables smooth transitions between typing levels:

```rust
pub struct GradualChecker {
    dynamic_checker: DynamicChecker,
    contract_checker: ContractChecker,  
    static_checker: StaticChecker,
    dependent_checker: DependentChecker,
}

impl GradualChecker {
    pub fn check_gradual(&mut self, expr: &Spanned<Expr>, level: TypeLevel) 
                        -> GradualResult {
        match level {
            TypeLevel::Dynamic => self.check_dynamic(expr),
            TypeLevel::Contracts => self.check_contracts(expr),
            TypeLevel::Static => self.check_static(expr),
            TypeLevel::Dependent => self.check_dependent(expr),
        }
    }
    
    /// Insert runtime checks at type boundaries
    pub fn insert_checks(&mut self, expr: &Spanned<Expr>) -> Spanned<Expr> {
        // Insert checks where typed code calls untyped code
        // and vice versa
    }
}
```

### **Mixed Typing in Practice**

```scheme
;; File 1: Dynamic code (legacy)
(define (legacy-function x y)
  ;; No type information
  (+ x y))

;; File 2: Gradually typed code  
#:type-level contracts

(define (safe-wrapper a b)
  #:contract (-> Number Number Number)
  ;; Runtime check inserted automatically
  (legacy-function a b))

;; File 3: Statically typed code
#:type-level static

(define (fully-typed x y)
  #:type (-> Number Number Number)  
  ;; Static verification, runtime check at boundary
  (safe-wrapper x y))
```

### **Blame Assignment**

When runtime checks fail, the gradual typing system assigns blame:

```scheme
;; Typed function
(define (typed-double x)
  #:type (-> Number Number)
  (* 2 x))

;; Dynamic caller passes wrong type
(define (bad-caller)
  (typed-double "not a number"))

;; Error message with blame:
;; Contract violation in typed-double
;; Expected: Number
;; Actual: String  
;; Blame: bad-caller (dynamic code)
;; The dynamic caller bad-caller failed to provide 
;; a Number as required by typed-double's contract
```

## Advanced Features

### **Row Polymorphism** (`src/types/row.rs`)

Extensible records using row polymorphism:

```scheme
;; Row types for extensible records
(define-type Person (r : Row)
  (record (name : String) 
          (age : Number) 
          | r))

;; Extension with additional fields
(define-type Employee (r : Row)  
  (record (salary : Number)
          (department : String)
          | (Person r)))

;; Polymorphic functions over records
(define (greet person)
  #:type (∀ (r : Row) (-> (Person r) String))
  (string-append "Hello, " (person.name)))

;; Works with any extension of Person
(define employee (Employee "Alice" 30 50000 "Engineering"))
(greet employee) ; => "Hello, Alice"
```

### **Higher-Kinded Types**

```scheme
;; Higher-kinded type variables
(define-class (Functor (f : * -> *))
  (map : ∀ a b. (a -> b) -> f a -> f b))

;; Instance for Maybe
(define-instance (Functor Maybe)
  (map maybe-map))

;; Instance for List
(define-instance (Functor List)
  (map list-map))

;; Generic functions using functors
(define (void f-val)
  #:type (∀ (f : * -> *) a. (Functor f) => f a -> f ())
  (map (lambda (_) ()) f-val))
```

### **Type Families**

```scheme
;; Associated types via type families
(define-class (Collection (c : * -> *))
  (type Element c : *)
  (type Index c : *)
  (empty : ∀ a. c a)
  (insert : ∀ a. Index c -> a -> c a -> c a)
  (lookup : ∀ a. Index c -> c a -> Maybe a))

;; Instance for vectors
(define-instance (Collection Vector)
  (type Element Vector = Any)
  (type Index Vector = Number)
  (empty (vector))
  (insert vector-set!)
  (lookup vector-ref-safe))
```

## Best Practices

### **1. Progressive Typing Strategy**

#### **Start Dynamic, Add Types Incrementally**
```scheme
;; Phase 1: Dynamic prototype
(define (process-data data)
  (map transform (filter valid? data)))

;; Phase 2: Add contracts
(define (process-data data)
  #:contract (-> (List Any) (List ProcessedData))
  (map transform (filter valid? data)))

;; Phase 3: Full static typing
(define (process-data data)
  #:type (-> (List InputData) (List ProcessedData))
  (map transform (filter valid? data)))
```

#### **Use Type-Driven Development**
```scheme
;; Start with type signatures
(define (merge-sorted xs ys)
  #:type (∀ a. (Ord a) => List a -> List a -> List a)
  ;; Implementation guided by types
  (cond
    [(null? xs) ys]
    [(null? ys) xs]
    [(<= (car xs) (car ys))
     (cons (car xs) (merge-sorted (cdr xs) ys))]
    [else
     (cons (car ys) (merge-sorted xs (cdr ys)))]))
```

### **2. Effective Use of Type Classes**

#### **Design Minimal Type Classes**
```scheme
;; Good: Minimal complete definition
(define-class (Eq a)
  (= : a -> a -> Boolean))

;; Derived methods
(define ≠ (lambda (x y) (not (= x y))))

;; Bad: Redundant methods in class
(define-class (BadEq a)
  (= : a -> a -> Boolean)
  (≠ : a -> a -> Boolean)  ; Could be derived
  (eq? : a -> a -> Boolean) ; Redundant with =
  (neq? : a -> a -> Boolean)) ; Redundant with ≠
```

#### **Use Superclass Constraints Appropriately**
```scheme
;; Good: Logical hierarchy
(define-class (Eq a) => (Ord a)
  (compare : a -> a -> Ordering))

;; Good: Multiple constraints when needed
(define-class (Eq a) (Show a) => (Debug a)
  (debug : a -> String))
```

### **3. Pattern Matching Best Practices**

#### **Exhaustive Pattern Matching**
```scheme
;; Good: Exhaustive patterns
(define (maybe-to-string maybe-val)
  (match maybe-val
    [Nothing "nothing"]
    [(Just x) (show x)]))

;; Compiler warns about non-exhaustive patterns
(define (incomplete-match color)
  (match color
    [Red "red"]
    [Green "green"]
    ;; Missing Blue case - warning issued
    ))
```

#### **Use Guards Judiciously**
```scheme
;; Good: Simple guards for additional constraints
(define (classify-temperature temp)
  (match temp
    [t #:when (<= t 32) 'freezing]
    [t #:when (<= t 70) 'cool] 
    [t #:when (<= t 90) 'warm]
    [_ 'hot]))

;; Consider helper functions for complex guards
(define hot-day? (lambda (temp humidity) (and (> temp 85) (> humidity 60))))

(define (comfort-level temp humidity)
  (match (list temp humidity)
    [(list t h) #:when (hot-day? t h) 'uncomfortable]
    [_ 'comfortable]))
```

### **4. Performance Considerations**

#### **Type Specialization**
```scheme
;; Generic function
(define (generic-add x y)
  #:type (∀ a. (Numeric a) => a -> a -> a)
  (+ x y))

;; Specialized versions for better performance  
(define (number-add x y)
  #:type (-> Number Number Number)
  (+ x y))

;; Use specialized versions in hot paths
(define (sum-numbers nums)
  (fold-left number-add 0 nums))
```

#### **Avoid Unnecessary Boxing**
```scheme
;; Good: Direct numeric operations
(define (fast-computation x)
  #:type (-> Number Number)
  (* (+ x 1) (- x 1)))

;; Bad: Boxing through generic operations  
(define (slow-computation x)
  (let ((boxed-x (box x)))
    (* (+ (unbox boxed-x) 1) 
       (- (unbox boxed-x) 1))))
```

---

This type system guide provides a comprehensive foundation for understanding and effectively using Lambdust's sophisticated type system. The four-level gradual typing approach enables developers to choose the appropriate level of type safety and performance optimization for their specific needs while maintaining the flexibility and expressiveness of Scheme.