# Lists Module

Comprehensive R7RS-compliant list operations, organized into specialized modules for optimal maintainability and performance.

## Module Structure

### Core Operations
- **`basic.rs`** - Fundamental list construction and deconstruction
  - `cons`, `car`, `cdr`, `list`, `make-list`
  - Basic list building operations following R7RS specification

- **`predicates.rs`** - List type testing and validation
  - `pair?`, `null?`, `list?`, `proper-list?`
  - Type predicates for list structure validation

- **`accessors.rs`** - List element access and traversal
  - `list-ref`, `length`, `list-tail`, `last-pair`
  - Safe and efficient list navigation operations

### Advanced Operations
- **`manipulation.rs`** - List transformation and mutation
  - `append`, `reverse`, `set-car!`, `set-cdr!`
  - In-place and functional list modification operations

- **`higher_order.rs`** - Functional programming primitives
  - `map`, `filter`, `fold-left`, `fold-right`, `for-each`
  - High-performance functional list processing

- **`utilities.rs`** - Common list algorithms
  - `member`, `assoc`, `sort`, `merge`
  - Utility functions for list searching and ordering

### Standards Compliance
- **`srfi1.rs`** - SRFI-1 extended list operations
  - Extended list processing functions beyond R7RS base
  - Full SRFI-1 compliance for interoperability

- **`common.rs`** - Shared utilities and constants
  - Helper functions, error types, and common patterns
  - Performance-optimized shared implementations

## Usage

```rust
// All operations are re-exported through the main module
use crate::stdlib::lists::{cons, car, cdr, map, filter};

// Or access specific modules directly
use crate::stdlib::lists::higher_order::{fold_left, fold_right};
```

## Performance Characteristics

- **Memory Efficient**: Optimized for minimal allocations
- **Type Safe**: Compile-time guarantee of list structure validity
- **R7RS Compliant**: Full adherence to Scheme R7RS specification
- **SIMD Optimized**: Vectorized operations where applicable

## Testing

Comprehensive test suite covering:
- R7RS compliance verification
- Performance benchmarking
- Edge case handling
- Memory safety validation