# SRFI-121: Generators Implementation Guide

## Overview

This document describes the complete implementation of SRFI-121 (Generators) in Lambdust, providing lazy sequence generation and processing capabilities following the R7RS specification.

## Architecture

The SRFI-121 implementation follows a two-layer architecture:

### Phase 1: Core Infrastructure (Rust)
- **Location**: `src/stdlib/generators.rs`, `src/containers/generator.rs`
- **Purpose**: Provides fundamental generator primitives and data structures
- **Integration**: Automatically loaded via `standard_library.rs`

### Phase 2-4: Complete Library (Scheme)
- **Location**: `stdlib/modules/srfi/121.scm`  
- **Purpose**: Full SRFI-121 specification implementation using Phase 1 primitives
- **Integration**: Available via `(import (srfi 121))`

## Phase 1: Core Primitives (Rust Implementation)

### Core Operations
```scheme
;; Type predicate  
(%generator? obj) → boolean

;; Core generator operations
(%make-generator thunk) → generator
(%generator-next gen) → value  
(%generator-exhausted? gen) → boolean
```

### Basic Constructors  
```scheme
;; Direct value constructors
(generator obj ...) → generator
(make-range-generator start [end [step]]) → generator
(make-iota-generator [count [start [step]]]) → generator

;; Collection converters
(list->generator lst) → generator
(vector->generator vec [start [end]]) → generator  
(string->generator str [start [end]]) → generator
```

### Utilities
```scheme
;; Consumer operations
(generator->list gen [n]) → list
(generator-fold kons knil gen) → value
```

## Phase 2-4: Complete SRFI-121 (Scheme Implementation)

### Generator Data Structure

The `Generator` struct in Rust supports multiple generator types:

```rust
pub enum GeneratorState {
    Procedure { thunk: Value, environment: Arc<ThreadSafeEnvironment> },
    Values { values: Vec<Value>, index: usize },
    Range { current: f64, step: f64, end: Option<f64> },
    Iota { count: i64, remaining: Option<usize>, step: i64 },
    List { current: Value },
    Vector { vector: Arc<RwLock<Vec<Value>>>, index: usize },
    String { string: String, index: usize },
    Exhausted,
}
```

### Additional Constructors (8 procedures)

```scheme
;; Collection generators with optional bounds
(vector->generator vec [start [end]]) → generator
(reverse-vector->generator vec [start [end]]) → generator  
(string->generator str [start [end]]) → generator
(bytevector->generator bv [start [end]]) → generator

;; Advanced constructors
(make-for-each-generator for-each obj) → generator
(make-unfold-generator stop? mapper successor seed) → generator
(make-coroutine-generator proc) → generator
```

### Generator Transformers (15 procedures)

#### Composition Operations
```scheme
(gcons* item ... gen) → generator          ; Cons elements onto generator
(gappend gen1 gen2 ...) → generator        ; Append generators
(gflatten gen) → generator                 ; Flatten generator of generators
```

#### Filtering Operations  
```scheme
(gfilter pred gen) → generator             ; Keep elements matching predicate
(gremove pred gen) → generator             ; Remove elements matching predicate
(gstate-filter proc gen) → generator       ; Stateful filtering
(gdelete item gen [equal?]) → generator    ; Delete specific elements
(gdelete-neighbor-dups gen [equal?]) → generator  ; Remove duplicate neighbors
```

#### Slicing Operations
```scheme
(gtake gen n) → generator                  ; Take first n elements
(gdrop gen n) → generator                  ; Skip first n elements  
(gtake-while pred gen) → generator         ; Take while predicate holds
(gdrop-while pred gen) → generator         ; Skip while predicate holds
```

#### Selection Operations
```scheme
(gindex value-gen index-gen) → generator   ; Index into value generator
(gselect gen index ...) → generator        ; Select specific indices
```

#### Grouping Operations
```scheme
(ggroup gen [equal?]) → generator          ; Group consecutive equal elements
```

### Generator Consumers (15 procedures)

#### Collection Converters
```scheme
(generator->list gen) → list
(generator->vector gen [n]) → vector
(generator->string gen) → string  
(generator->bytevector gen) → bytevector
```

#### Reduction Operations
```scheme
(generator-fold kons knil gen) → value     ; Left fold over generator
(generator-reduce gen proc) → value        ; Fold without initial value
```

#### Iteration Operations  
```scheme
(generator-for-each proc gen) → unspecified
(generator-map proc gen1 gen2 ...) → generator
```

#### Search Operations
```scheme
(generator-find pred gen) → value or #f
(generator-count pred gen) → integer
(generator-any pred gen) → boolean
(generator-every pred gen) → boolean  
```

#### Statistical Operations
```scheme
(generator-length gen) → integer           ; Count all elements
(generator-sum gen) → number              ; Sum numeric elements
```

#### Advanced Operations
```scheme
(generator-unfold gen unfolder [stop?]) → list
```

### Accumulator Procedures (8 procedures)

Accumulators provide stateful aggregation of values:

```scheme
;; Generic accumulator constructor
(make-accumulator proc init) → accumulator

;; Specialized accumulators
(count-accumulator) → accumulator          ; Count elements
(list-accumulator) → accumulator           ; Collect into list
(reverse-list-accumulator) → accumulator   ; Collect maintaining order
(vector-accumulator) → accumulator         ; Collect into vector
(string-accumulator) → accumulator         ; Collect chars into string
(bytevector-accumulator) → accumulator     ; Collect bytes into bytevector  
(sum-accumulator) → accumulator            ; Sum numeric values

;; Create generator from accumulator results
(make-accumulator-generator accumulator gen) → generator
```

## Key Implementation Features

### Lazy Evaluation
All generators support lazy evaluation - values are computed only when requested via `%generator-next`.

### Infinite Sequences  
Generators can represent infinite sequences:
```scheme
(define naturals (make-iota-generator))     ; Infinite counting from 0
(define evens (gfilter even? naturals))     ; Infinite even numbers
```

### Thread Safety
The Rust implementation uses `Arc<RwLock<T>>` for thread-safe generator state management.

### R7RS Compliance
- Full error handling with proper error messages
- Consistent EOF object handling (`*eof-object*` symbol)
- Integration with R7RS type system
- Compatible with existing Scheme procedures

### Memory Efficiency
- Stateful iteration without building intermediate collections
- Streaming processing for large datasets  
- Automatic cleanup of exhausted generators

## Usage Examples

### Basic Generator Usage
```scheme
(import (srfi 121))

;; Create and consume a simple generator
(define gen (generator 'a 'b 'c))
(generator->list gen)  ; ⇒ (a b c)

;; Infinite sequence processing
(define squares (generator-map (lambda (x) (* x x)) (make-iota-generator)))
(generator->list (gtake squares 5))  ; ⇒ (0 1 4 9 16)
```

### Advanced Generator Composition
```scheme
;; Chain multiple transformations
(define processed
  (gtake 
    (gfilter odd?
      (generator-map (lambda (x) (* x 3))
        (make-range-generator 1 20)))
    5))

(generator->list processed)  ; ⇒ (3 9 15 21 27)
```

### Accumulator Usage
```scheme
;; Custom data aggregation
(define acc (make-accumulator 
             (lambda (state item) (cons item state))
             '()))

;; Process generator through accumulator
(generator-for-each acc (generator 1 2 3))
(acc *eof-object*)  ; ⇒ (3 2 1)
```

## Integration Points

### With Evaluator
- Generator procedures are first-class values
- Compatible with call/cc for coroutine generators
- Exception handling for generator errors

### With Module System  
- Available via `(import (srfi 121))`
- All 53 procedures properly exported
- Compatible with R7RS library system

### With Other SRFIs
- Works with SRFI-1 (list procedures)
- Compatible with SRFI-43 (vector procedures)  
- Integrates with SRFI-13 (string procedures)

## Performance Characteristics

### Time Complexity
- Generator creation: O(1) for most types
- Next operation: O(1) amortized
- Transformation chaining: O(1) per transformation

### Space Complexity
- Constant space for stateful generators
- Linear space only for intermediate results when required
- Streaming processing avoids building large intermediate collections

### Optimization Opportunities
- Bytecode compilation for frequently used generator chains
- SIMD operations for numeric transformations  
- Fusion of adjacent transformations

## Testing

Comprehensive test suite available at `tests/srfi-121-test.scm`:
- Tests all 53 SRFI-121 procedures
- Verifies Phase 1 Rust primitives  
- Validates Phase 2-4 Scheme implementations
- Includes edge cases and error conditions

## Future Enhancements

### Planned Features
- Procedure-based generators with evaluator integration
- Parallel generator processing
- Generator debugging and introspection tools
- Performance profiling for generator chains

### Potential Optimizations
- Lazy fusion of generator transformations
- Specialized generators for common patterns
- Integration with streaming I/O
- Memory-mapped file generators

## Conclusion

The SRFI-121 implementation in Lambdust provides a complete, efficient, and R7RS-compliant generator system that supports both simple use cases and advanced lazy sequence processing patterns. The two-phase architecture balances performance (Rust primitives) with extensibility (Scheme combinators) while maintaining full specification compliance.