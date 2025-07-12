# R7RS-pico Ultra-Minimal Scheme Implementation

This document describes Lambdust's implementation of R7RS-pico, an ultra-minimal Scheme variant designed for embedded systems and educational purposes.

## Overview

R7RS-pico is a simplified subset of R7RS-small that removes complex features while maintaining the core functional programming capabilities of Scheme. The primary goal is to create an implementation that can be easily understood and deployed in resource-constrained environments.

## Key Characteristics

### Simplified Semantic Model

**R7RS-small**: `U -> P -> K -> C` (Environment -> Program -> Continuation -> Command)
**R7RS-pico**: `U -> E` (Environment -> Expressed value)

The R7RS-pico semantic model eliminates:
- Continuations and continuation-passing style
- Store modifications and side effects
- Complex control flow constructs

### Features Included

#### Data Types
- **Boolean**: `#t` and `#f`
- **Number**: Integers only (implementation-defined range)
- **Symbol**: Quoted identifiers
- **Pair**: Constructed with `cons`
- **Procedure**: Lambda expressions and built-ins
- **Null**: Empty list `()`

#### Special Forms
- **`lambda`**: Function definition
- **`if`**: Conditional expression (2 or 3 arguments)
- **`define`**: Variable and function definition
- **`quote`**: Literal data quotation

#### Built-in Procedures

**Arithmetic Operations**:
- `+` - Addition of two numbers
- `-` - Subtraction or unary negation
- `*` - Multiplication of two numbers
- `=` - Numeric equality
- `<` - Numeric less than
- `>` - Numeric greater than

**List Operations**:
- `cons` - Construct a pair
- `car` - First element of a pair
- `cdr` - Second element of a pair

**Type Predicates**:
- `null?` - Test for empty list
- `pair?` - Test for pair
- `number?` - Test for number
- `boolean?` - Test for boolean
- `symbol?` - Test for symbol
- `procedure?` - Test for procedure

**Equivalence**:
- `eqv?` - Test for equivalence

### Features Excluded

The following R7RS-small features are intentionally excluded from R7RS-pico:

- **Side Effects**: `set!`, `set-car!`, `set-cdr!`
- **Continuations**: `call/cc`, `call-with-values`
- **Complex Numbers**: Only integers supported
- **Vectors**: Array-like data structures
- **Strings**: Limited or no string operations
- **Input/Output**: File and port operations
- **Macros**: `syntax-rules`, `define-syntax`
- **Multiple Values**: `values`, `call-with-values`
- **Dynamic Environment**: `dynamic-wind`

## Implementation Details

### PicoEvaluator

The `PicoEvaluator` implements the simplified U -> E semantic model:

```rust
pub struct PicoEvaluator {
    max_recursion_depth: usize,
    current_depth: usize,
}
```

Key features:
- **Stack Overflow Protection**: Configurable recursion depth limit
- **Pure Functional Evaluation**: No side effects or mutations
- **Proper Tail Recursion**: Required by R7RS-pico specification
- **Minimal Memory Footprint**: Designed for embedded systems

### Initial Environment

The R7RS-pico initial environment contains only the essential built-in procedures:

```rust
pub fn create_pico_initial_environment() -> Rc<Environment>
```

This creates an environment with all 13 required built-in procedures, organized into:
- 6 arithmetic operations
- 3 list operations
- 6 type predicates
- 1 equivalence predicate

### Memory Model

Since R7RS-pico excludes side effects, implementations have flexibility in memory management:

- **No Mutation**: Values are immutable
- **No Store**: No global state modifications
- **Garbage Collection**: Optional (depends on implementation)
- **Stack Management**: Proper tail recursion prevents stack overflow

## Usage Examples

### Basic Arithmetic

```scheme
(+ 3 4)        ; => 7
(- 10 3)       ; => 7
(* 5 6)        ; => 30
(- 7)          ; => -7 (unary negation)
(= 5 5)        ; => #t
(< 3 7)        ; => #t
```

### List Operations

```scheme
(cons 1 2)                    ; => (1 . 2)
(cons 1 (cons 2 ()))         ; => (1 2)
(car (cons 1 2))             ; => 1
(cdr (cons 1 2))             ; => 2
```

### Conditional Expressions

```scheme
(if #t 1 2)                  ; => 1
(if #f 1 2)                  ; => 2
(if (< 3 5) 'yes 'no)        ; => yes
```

### Function Definition

```scheme
(define square (lambda (x) (* x x)))
(square 5)                   ; => 25

(define fact 
  (lambda (n) 
    (if (= n 0) 
        1 
        (* n (fact (- n 1))))))
(fact 5)                     ; => 120
```

### Type Predicates

```scheme
(number? 42)                 ; => #t
(boolean? #t)                ; => #t
(symbol? 'hello)             ; => #t
(pair? (cons 1 2))          ; => #t
(null? '())                  ; => #t
```

## Configuration

### Feature Flags

Enable R7RS-pico support in Cargo.toml:

```toml
[features]
pico = []
embedded = ["pico"]
```

### Binary Size

R7RS-pico configuration targets:
- **pico**: < 200KB binary size
- **embedded**: < 500KB binary size (includes pico)

### Recursion Limits

Configure recursion depth for embedded systems:

```rust
let evaluator = PicoEvaluator::with_recursion_limit(500); // Conservative for MCU
```

## Testing

Run R7RS-pico tests:

```bash
cargo test --features pico
```

Run the demonstration example:

```bash
cargo run --example r7rs_pico_demo --features pico
```

## Compliance

This implementation aims for full compliance with the R7RS-pico specification:

- ✅ **Semantic Model**: U -> E implemented correctly
- ✅ **Required Procedures**: All 13 built-ins implemented
- ✅ **Special Forms**: lambda, if, define, quote supported
- ✅ **Proper Tail Recursion**: Stack-safe recursive calls
- ✅ **Type System**: Correct type predicates and disjointness
- ✅ **Error Handling**: Appropriate error reporting

## Integration with Lambdust

R7RS-pico integrates seamlessly with the larger Lambdust ecosystem:

- **Shared Infrastructure**: Uses common AST, environment, and value types
- **Performance**: Benefits from Lambdust's optimization infrastructure
- **Testing**: Comprehensive test suite with existing framework
- **Documentation**: Fully documented with examples

## Future Enhancements

Potential improvements while maintaining R7RS-pico compliance:

1. **Memory Optimization**: Further reduce memory footprint
2. **Performance Tuning**: Optimize hot paths for embedded systems
3. **Cross-Compilation**: Support for various microcontroller architectures
4. **Educational Tools**: Enhanced error messages and debugging support
5. **Formal Verification**: Mathematical proofs of correctness

## References

- [R7RS-pico Specification](https://github.com/jrincayc/r7rs-pico-spec)
- [The Little Schemer](https://mitpress.mit.edu/books/little-schemer) (inspiration)
- [R7RS-small Standard](https://small.r7rs.org/)
- [Lambdust Architecture Documentation](ARCHITECTURE.md)