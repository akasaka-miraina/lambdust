# Strings Module

R7RS-compliant string processing operations, modularized for maintainability and comprehensive Unicode support.

## Module Structure

### Core Operations
- **`basic.rs`** - String construction and fundamental operations
  - `make-string`, `string`, `string-length`, `string-ref`
  - Basic string creation and access operations
  - Unicode-aware character handling

- **`predicates.rs`** - String validation and testing
  - `string?`, `string-null?`, `string=?`, `string<?`
  - Comprehensive string comparison and validation
  - Locale-aware comparison operations

### Advanced Features
- **`common.rs`** - Shared utilities and character operations
  - Character set operations and utilities
  - String conversion and encoding support
  - Performance-optimized common patterns
  - Unicode normalization and validation

### Integration
- **`mod.rs`** - Module coordination and re-exports
  - Unified string API with transparent access
  - Integration with R7RS character system
  - Proper error handling and type safety

## Key Features

### Unicode Support
- Full UTF-8 string processing
- Character set operations (ASCII, Unicode categories)
- Proper handling of grapheme clusters
- Locale-aware string operations

### Performance Optimization
- SIMD-accelerated string operations where applicable
- Memory-efficient string representations
- Zero-copy operations when possible
- Optimized for common string patterns

### R7RS Compliance
- Complete implementation of R7RS string procedures
- Proper integration with character types
- Support for immutable string semantics
- Full compatibility with standard Scheme string operations

## Usage Examples

```rust
// Basic string operations
use crate::stdlib::strings::{make_string, string_length, string_ref};

// String predicates and comparison
use crate::stdlib::strings::{string_null_p, string_equal_p, string_less_p};

// Advanced character set operations
use crate::stdlib::strings::common::{CharacterSet, string_normalize};
```

## Performance Characteristics

- **Unicode Aware**: Proper handling of multi-byte characters
- **Memory Efficient**: Minimal string copying and allocation
- **Type Safe**: Compile-time string validity guarantees
- **Vectorized**: SIMD operations for bulk string processing

## Testing Coverage

- Unicode edge cases and corner cases
- Performance benchmarking against reference implementations
- R7RS compliance test suite
- Memory safety and correctness validation