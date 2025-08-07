# SRFI Implementation Documentation

This directory contains Lambdust's implementation of Scheme Requests for Implementation (SRFIs), providing comprehensive libraries that extend R7RS-small toward R7RS-large compliance.

## Overview

Our SRFI implementations follow these design principles:

- **Pure Scheme Implementation**: Built using existing Lambdust primitives and R7RS-small features
- **R7RS Compliance**: Full compatibility with R7RS-small as foundation
- **Performance Oriented**: Optimized for common use cases while maintaining correctness
- **Integration Focused**: Seamless integration with existing Lambdust stdlib
- **Documentation Rich**: Comprehensive documentation and usage examples

## Currently Implemented SRFIs

### Phase 1: Core Data Structures (Completed)

#### SRFI-1: List Library
**Status**: ✅ Complete  
**File**: `1.scm`  
**Description**: Comprehensive list processing procedures extending R7RS basic list operations.

**Key Features**:
- Constructors: `make-list`, `list-tabulate`, `iota`, `circular-list`
- Predicates: `proper-list?`, `circular-list?`, `null-list?`
- Selectors: `first` through `tenth`, `take`, `drop`, `split-at`
- Fold/unfold: `fold`, `fold-right`, `unfold`, `reduce`
- Filtering: `filter`, `partition`, `remove`
- Searching: `find`, `any`, `every`, `list-index`
- Set operations: `lset-union`, `lset-intersection`, `lset-difference`

#### SRFI-2: and-let*
**Status**: ✅ Complete  
**File**: `2.scm`  
**Description**: Sequential binding and testing with short-circuit evaluation.

**Key Features**:
- Macro for safe nested operations
- Multiple clause types: bindings, tests, variables
- Extensive documentation with patterns

#### SRFI-6: Basic String Ports
**Status**: ✅ Complete  
**File**: `6.scm`  
**Description**: String port operations (now part of R7RS).

**Key Features**:
- `open-input-string`, `open-output-string`, `get-output-string`
- R7RS compatibility layer
- Usage examples and patterns

#### SRFI-13: String Library
**Status**: ✅ Complete  
**File**: `13.scm`  
**Description**: Comprehensive string processing procedures.

**Key Features**:
- Predicates: `string-null?`, `string-every`, `string-any`
- Constructors: `string-tabulate`, `string-join`
- Selection: `string-take`, `string-drop`, `string-pad`, `string-trim`
- Comparison: `string-compare`, `string-hash`
- Case conversion: `string-upcase`, `string-downcase`, `string-titlecase`
- Searching: `string-contains`, `string-index`, `string-count`

#### SRFI-26: Notation for Specializing Parameters (cut/cute)
**Status**: ✅ Complete  
**File**: `26.scm`  
**Description**: Create specialized procedures by "cutting" slots in procedure calls.

**Key Features**:
- `cut` macro for lazy slot filling
- `cute` macro for eager expression evaluation
- `<>` and `<...>` placeholder symbols
- Comprehensive pattern matching

#### SRFI-41: Streams (NEW)
**Status**: ✅ Complete - Phase 1 Priority  
**File**: `41.scm`  
**Description**: Lazy sequences for efficient processing of potentially infinite data.

**Key Features**:
- Stream constructors: `stream-cons`, `stream-null`
- Stream primitives: `stream-car`, `stream-cdr`, `stream-lambda`
- Conversion: `list->stream`, `stream->list`, `port->stream`
- Operations: `stream-map`, `stream-filter`, `stream-fold`, `stream-take`
- Generators: `stream-from`, `stream-range`, `stream-iterate`, `stream-constant`
- Advanced: `stream-unfold`, `stream-zip`, pattern matching

**Example Usage**:
```scheme
(import (srfi 41))

;; Infinite stream of natural numbers
(define nats (stream-from 0))

;; Fibonacci sequence
(define fibs
  (stream-cons 0
    (stream-cons 1
      (stream-map + fibs (stream-cdr fibs)))))

;; Take first 10 Fibonacci numbers
(stream->list (stream-take 10 fibs))
;; => (0 1 1 2 3 5 8 13 21 34)
```

#### SRFI-111: Boxes (NEW)
**Status**: ✅ Complete - Phase 1 Priority  
**File**: `111.scm`  
**Description**: Single-cell mutable containers for controlled mutability.

**Key Features**:
- Basic operations: `box`, `box?`, `unbox`, `set-box!`
- Atomic operations: `box-cas!` (compare-and-swap), `box-swap!`
- Thread-safe semantics preparation
- Extensive usage patterns and examples

**Example Usage**:
```scheme
(import (srfi 111))

;; Create and use a counter
(define counter (box 0))
(define (increment!) 
  (box-swap! counter (lambda (n) (+ n 1))))

(increment!) ; returns 0, counter now 1
(increment!) ; returns 1, counter now 2
(unbox counter) ; => 2
```

#### SRFI-125: Intermediate Hash Tables (NEW)
**Status**: ✅ Complete - Phase 1 Priority  
**File**: `125.scm`  
**Description**: Comprehensive hash table operations with customizable hash and equality functions.

**Key Features**:
- Constructors: `make-hash-table`, `hash-table`, `alist->hash-table`
- Access: `hash-table-ref`, `hash-table-contains?`, `hash-table-set!`
- Bulk operations: `hash-table-keys`, `hash-table-values`, `hash-table-entries`
- Mapping: `hash-table-map`, `hash-table-fold`, `hash-table-for-each`
- Merging: `hash-table-union!`, `hash-table-intersection!`
- Copying: `hash-table-copy`, immutable hash tables

**Example Usage**:
```scheme
(import (srfi 125))

;; Create hash table with word counts
(define word-counts (make-hash-table string=? string-hash))

(define (count-word! word)
  (hash-table-update!/default word-counts word 
                              (lambda (n) (+ n 1)) 0))

(count-word! "hello")
(count-word! "world") 
(count-word! "hello")

(hash-table-ref word-counts "hello") ; => 2
```

### Previously Implemented SRFIs

#### SRFI-8: receive
**Status**: ✅ Complete  
**File**: `8.scm`

#### SRFI-9: Defining Record Types
**Status**: ✅ Complete  
**File**: `9.scm`

#### SRFI-14: Character Sets
**Status**: ✅ Complete  
**File**: `14.scm`

#### SRFI-16: case-lambda
**Status**: ✅ Complete  
**File**: `16.scm`

#### SRFI-23: Error Reporting Mechanism
**Status**: ✅ Complete  
**File**: `23.scm`

#### SRFI-39: Parameter Objects
**Status**: ✅ Complete  
**File**: `39.scm`

#### SRFI-43: Vector Library
**Status**: ✅ Complete  
**File**: `43.scm`

## Integration and Interoperability

The new SRFI implementations are designed to work seamlessly together:

### Streams + Boxes
```scheme
;; Stream of mutable counters
(define counters (stream-map box (stream-range 1 6)))
(stream-for-each (lambda (b) (set-box! b (* 2 (unbox b))))
                 (stream-take 3 counters))
```

### Streams + Hash Tables
```scheme
;; Build hash table from stream
(define squares (make-hash-table))
(stream-for-each (lambda (n) (hash-table-set! squares n (* n n)))
                 (stream-range 1 11))
```

### Boxes + Hash Tables
```scheme
;; Hash table of mutable state
(define state (make-hash-table))
(hash-table-set! state 'counter (box 0))
(hash-table-set! state 'total (box 0))

(define (increment-and-add! value)
  (box-swap! (hash-table-ref state 'counter) (lambda (c) (+ c 1)))
  (box-swap! (hash-table-ref state 'total) (lambda (t) (+ t value))))
```

## Testing

Comprehensive test suites are provided:

- `/tests/test-srfi-41.ldust` - Streams test suite
- `/tests/test-srfi-111.ldust` - Boxes test suite  
- `/tests/test-srfi-125.ldust` - Hash tables test suite
- `/tests/test-srfi-suite.ldust` - Integration test suite

Run tests with:
```bash
lambdust tests/test-srfi-suite.ldust
```

## R7RS-Large Roadmap

### Phase 2: Advanced Algorithms and Collections (Q2 2025)
- **SRFI-132**: Sort Libraries - Comprehensive sorting algorithms
- **SRFI-113**: Sets and Bags - Collection data structures
- **SRFI-151**: Bitwise Operations - Low-level programming support

### Phase 3: Extended I/O and System (Q3 2025)  
- **SRFI-158**: Generators and Accumulators - Iterator patterns
- **SRFI-170**: POSIX API - System interface compatibility

### Phase 4: Mathematical Extensions (Q4 2025)
- **SRFI-143**: Fixnums - Efficient integer arithmetic
- **SRFI-144**: Flonums - Floating-point operations

## Design Notes

### Performance Considerations

- **Streams**: Use Scheme's `delay`/`force` for lazy evaluation with memoization
- **Boxes**: Implemented using vectors for constant-time operations
- **Hash Tables**: Use vector of buckets with automatic resizing (load factor 0.75)

### Memory Management

- **Streams**: Only computed elements are stored; proper tail recursion optimized
- **Boxes**: Minimal overhead (2 vector slots + type tag)
- **Hash Tables**: Efficient rehashing and garbage collection friendly

### Error Handling

All implementations include:
- Proper type checking with informative error messages
- Consistent error reporting across operations
- Guard against common programming errors

### Thread Safety

Current implementations are designed for single-threaded use but prepare for multi-threading:
- Box CAS operations provide atomic semantics foundation
- Hash table operations are designed to be thread-safe with external synchronization
- Streams are naturally immutable after creation

## Usage Examples

### Functional Programming Patterns

```scheme
;; Pipeline processing with streams
(define (process-numbers n)
  (stream->list
    (stream-take n
      (stream-filter prime?
        (stream-map square
          (stream-from 1))))))

;; Memoized computation with hash tables
(define (make-memoized-func f)
  (let ((cache (make-hash-table equal?)))
    (lambda (x)
      (hash-table-intern! cache x (lambda () (f x))))))

;; State management with boxes
(define (make-counter initial)
  (let ((count (box initial)))
    (lambda (op)
      (case op
        ((get) (unbox count))
        ((inc) (box-swap! count (lambda (n) (+ n 1))))
        ((dec) (box-swap! count (lambda (n) (- n 1))))))))
```

### Data Processing Pipelines

```scheme
;; ETL pipeline using all three SRFIs
(define (process-data-stream input-stream)
  (let ((cache (make-hash-table string=?))
        (stats (box (make-hash-table))))
    
    ;; Process stream with caching
    (stream-map
      (lambda (item)
        (let ((processed (hash-table-intern! cache 
                                           (item-key item)
                                           (lambda () (expensive-transform item)))))
          ;; Update statistics
          (let ((current-stats (unbox stats)))
            (hash-table-update!/default current-stats 'count 
                                       (lambda (n) (+ n 1)) 0))
          processed))
      input-stream)))
```

## Contributing

When implementing new SRFIs:

1. Follow the existing patterns and naming conventions
2. Include comprehensive documentation with examples
3. Provide complete test coverage
4. Consider integration with existing SRFIs
5. Optimize for common use cases
6. Include error handling and edge cases

## References

- [R7RS-small Specification](https://small.r7rs.org/)
- [SRFI Home Page](https://srfi.schemers.org/)
- [Gauche R7RS-large Implementation](http://practical-scheme.net/gauche/man/gauche-refe/R7RS-large.html)
- [R7RS Working Group 2](https://github.com/johnwcowan/r7rs-work)

## License

These SRFI implementations are part of Lambdust and follow the same license terms. Individual SRFIs may have specific license requirements as noted in their specifications.