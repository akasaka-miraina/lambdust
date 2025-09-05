# Lambdust Contract System

A comprehensive contract system for the Lambdust Scheme interpreter with runtime checking and precise blame tracking.

## Overview

The contract system provides:

- **Function contracts** with domain/codomain checking
- **Data contracts** with predicates and constraints  
- **Higher-order contracts** for functions that accept/return functions
- **Contract combinators** (and/c, or/c, not/c, ->, etc.)
- **Blame tracking** for precise error attribution
- **Performance optimization** through contract compilation
- **Integration** with gradual typing

## Architecture

The contract system consists of several key components:

### Core Components

1. **AST (`ast.rs`)** - Contract expression abstract syntax tree
2. **Predicates (`predicates.rs`)** - Built-in and custom predicate functions
3. **Combinators (`combinators.rs`)** - Contract combination and evaluation
4. **Compiler (`compiler.rs`)** - Contract compilation and optimization
5. **Enforcement (`enforcement.rs`)** - Runtime contract checking
6. **Blame (`blame.rs`)** - Blame tracking and error attribution
7. **Runtime (`runtime.rs`)** - Runtime integration and coordination
8. **Parser (`parser.rs`)** - Contract syntax parsing
9. **Evaluator (`evaluator.rs`)** - Integration with main evaluator

### Data Flow

```
Contract Source Code
        ↓
    Parser (syntax)
        ↓
    AST (contract expressions)
        ↓
    Compiler (optimization)
        ↓
    CompiledContract (runtime checker)
        ↓
    Enforcement (runtime checking)
        ↓
    Blame Tracking (error attribution)
```

## Contract Language

### Basic Predicates

```scheme
;; Type predicates
number?     ; tests for numbers
string?     ; tests for strings
boolean?    ; tests for booleans
list?       ; tests for proper lists
vector?     ; tests for vectors

;; Numeric predicates
positive?   ; tests for positive numbers
negative?   ; tests for negative numbers
zero?       ; tests for zero
even?       ; tests for even integers
odd?        ; tests for odd integers

;; Collection predicates
null?       ; tests for empty list
empty?      ; tests for empty collections
non-empty?  ; tests for non-empty collections
```

### Contract Combinators

```scheme
;; Logical combinators
(and/c contract ...)        ; all contracts must hold
(or/c contract ...)         ; at least one contract must hold
(not/c contract)            ; contract must not hold

;; Function contracts
(-> domain ... codomain)    ; function contract
(->i ([var contract] ...) codomain)  ; dependent function contract
(->* (domain ...) codomain) ; case function contract

;; Structural contracts
(listof contract)           ; homogeneous list contract
(vectorof contract)         ; homogeneous vector contract
(hash/c key-contract value-contract) ; hash table contract
(tuple/c contract ...)      ; fixed-length tuple contract

;; Value contracts
(one-of/c value ...)        ; value must be one of the given values
(between/c min max)         ; numeric range contract
(</c value)                 ; less than contract
(>/c value)                 ; greater than contract
(=/c value)                 ; equality contract

;; Higher-order contracts
(listof (-> number? string?)) ; list of functions
(-> (-> number? boolean?) (listof number?) (listof number?)) ; function taking function

;; Special contracts
any/c                       ; matches any value
none/c                      ; matches no value
flat/c                      ; flat contract with custom predicate
```

### Define/Contract Syntax

```scheme
;; Basic function contract
(define/contract (safe-divide x y)
  (-> number? (and/c number? (not/c zero?)) number?)
  (/ x y))

;; Higher-order function contract
(define/contract (map f lst)
  (-> (-> any/c any/c) list? list?)
  (if (null? lst)
      '()
      (cons (f (car lst)) (map f (cdr lst)))))

;; Dependent contract
(define/contract (vector-ref v i)
  (->i ([v vector?] [i (and/c exact-integer? (</c (vector-length v)))])
       any/c)
  (vector-ref v i))

;; Variable contract
(define/contract max-items
  (and/c exact-integer? positive?)
  100)
```

## Implementation Details

### Contract Compilation

Contracts are compiled into efficient runtime checkers:

1. **Parsing** - Contract syntax is parsed into AST nodes
2. **Optimization** - Contracts are optimized using various strategies:
   - Constant folding (e.g., `(and/c any/c number?)` → `number?`)
   - Dead code elimination
   - Predicate inlining
   - Check combining
3. **Code Generation** - Optimized contracts are compiled to runtime checkers
4. **Caching** - Compiled contracts are cached for reuse

### Blame Tracking

The blame system tracks contract violations with precise attribution:

- **Positive blame** - The party that provided the value
- **Negative blame** - The party that required the contract  
- **Call stack** - Complete call context at contract establishment
- **Boundary information** - Details about the contract boundary
- **Violation history** - Record of all contract violations

### Performance Optimization

The system includes several performance optimizations:

- **Contract compilation** - Contracts are compiled to efficient checkers
- **Predicate inlining** - Simple predicates are inlined
- **Constant folding** - Compile-time contract simplification
- **Caching** - Compiled contracts and results are cached
- **Lazy checking** - Some checks are deferred until necessary

### Error Handling

Contract violations produce detailed error messages:

```
Contract violation: expected (and/c number? positive?), got -5
  Blame: caller at test.scm:10:5
  Contract: safe-divide at test.scm:5:1
  Stack trace:
    safe-divide (test.scm:5:1)
    main (test.scm:10:5)
```

## Usage Examples

### Basic Usage

```scheme
;; Load the contract system
(import (lambdust contracts))

;; Define a simple contracted function
(define/contract (square x)
  (-> number? number?)
  (* x x))

;; Use the function
(square 5)    ; returns 25
(square "hi") ; Contract violation: expected number?, got "hi"
```

### Advanced Usage

```scheme
;; Complex data structure contract
(define/contract database-record
  (hash/c symbol? (or/c string? number? boolean?))
  (make-hash))

;; Higher-order function with contracts
(define/contract (fold-left f init lst)
  (-> (-> any/c any/c any/c) any/c list? any/c)
  (if (null? lst)
      init
      (fold-left f (f init (car lst)) (cdr lst))))

;; Parametric contract
(define/contract (make-list-of-contract element-contract)
  (-> contract? contract?)
  (listof element-contract))
```

### Integration with Type System

```scheme
;; Contracts can work alongside type annotations
(define/contract (typed-function x : Number) : String
  (-> number? string?)
  (number->string x))

;; Gradual migration from dynamic to static
(define/contract (gradually-typed x)
  (-> any/c string?)  ; Start with loose contract
  (cond
    [(number? x) (number->string x)]
    [(string? x) x]
    [else (error "Unexpected type")]))
```

## API Reference

### Main Contract System

```rust
// Create a new contract system
let system = ContractSystem::new();

// Compile a contract
let compiled = system.compile_contract(&contract, &context)?;

// Check a value against a contract
system.check_contract(&value, &compiled, &blame)?;

// Wrap a value with contract checking
let wrapped = system.wrap_with_contract(value, compiled, blame)?;
```

### Contract Runtime

```rust
// Create runtime coordinator
let runtime = ContractRuntime::new();

// Register named contracts
runtime.register_contract("my-contract", contract, blame)?;

// Check against named contract
runtime.check_named_contract(&value, "my-contract", &blame)?;

// Performance statistics
let stats = runtime.performance_stats();
```

### Predicate Registry

```rust
// Create predicate registry
let registry = PredicateRegistry::new();

// Register custom predicate
registry.register("even?", even_predicate());

// Look up predicate
let predicate = registry.lookup("number?")?;
let result = predicate.test(&value);
```

## Configuration

The contract system can be configured with various options:

```rust
let config = ContractConfig {
    enable_checking: true,           // Enable runtime checking
    enable_compilation: true,        // Enable contract compilation
    enable_blame_tracking: true,     // Enable blame tracking
    max_recursion_depth: 100,        // Maximum recursion depth
    enable_contract_caching: true,   // Enable contract caching
    enable_gradual_integration: true, // Enable gradual typing integration
};

let system = ContractSystem::with_config(config);
```

## Testing

The contract system includes comprehensive tests:

```bash
# Run all contract tests
cargo test contracts

# Run specific test modules
cargo test contracts::predicates
cargo test contracts::combinators
cargo test contracts::enforcement

# Run integration tests
cargo test contracts::tests::integration_tests
```

## Future Enhancements

Planned improvements include:

1. **Dependent contracts** - Full support for dependent contract types
2. **Contract inference** - Automatic contract inference from usage
3. **Blame simplification** - Better blame chain simplification
4. **Performance monitoring** - Real-time performance analysis
5. **Contract debugging** - Interactive contract debugging tools
6. **IDE integration** - Language server protocol support
7. **Symbolic execution** - Symbolic contract verification

## Related Documentation

- [Type System Documentation](../types/README.md)
- [Effect System Documentation](../effects/README.md)
- [Macro System Documentation](../macro_system/README.md)
- [Implementation Roadmap](../../IMPLEMENTATION_ROADMAP.md)