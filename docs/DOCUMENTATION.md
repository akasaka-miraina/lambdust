# Lambdust Documentation

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Building and Installation](#building-and-installation)
4. [API Reference](#api-reference)
5. [Type System](#type-system)
6. [Effect System](#effect-system)
7. [Concurrency](#concurrency)
8. [Performance](#performance)
9. [Contributing](#contributing)
10. [Changelog](#changelog)

## Overview

Lambdust is a comprehensive R7RS-large compliant Scheme interpreter written in Rust, featuring advanced type systems, effect handling, concurrency primitives, and performance optimization capabilities.

### Key Features

- **R7RS-large Compliance**: Extensive support for the R7RS-large Scheme standard with SRFI implementations
- **Advanced Type System**: Gradual typing, algebraic data types, and type classes
- **Effect System**: Monadic programming with effect handlers for managing side effects
- **Concurrency System**: Actor model, futures, STM (Software Transactional Memory), and parallel computation
- **FFI System**: C interoperability with dynamic library loading
- **Metaprogramming**: Hygienic macros, reflection, and code generation
- **Performance Optimization**: Bytecode compilation and primitive specialization

### Core Components

Lambdust is built around 42 core primitives that bootstrap the entire system, providing a solid foundation for Scheme language features while maintaining memory safety through Rust's ownership system.

## Architecture

### System Architecture

The interpreter consists of several interconnected subsystems:

- **Parser**: Converts Scheme source code into Abstract Syntax Trees (AST)
- **Evaluator**: Executes Scheme expressions using the core primitive operations
- **Type System**: Provides gradual typing with inference and checking
- **Effect System**: Manages side effects through monadic composition
- **Runtime**: Coordinates all subsystems and provides the execution environment
- **Memory Management**: Rust-based ownership with garbage collection for Scheme values

### Value System

All Scheme values are represented through a unified `Value` enum that supports:
- Basic types: numbers, strings, symbols, characters
- Compound types: pairs, vectors, procedures
- Advanced types: algebraic data types, type classes
- Special values: continuations, ports, promises

### Environment System

Variable binding and scope management is handled through:
- Lexical scoping with proper tail recursion
- Dynamic environment support for special forms
- Module system with import/export capabilities

## Building and Installation

### Prerequisites

- Rust 1.75.0 or later
- Cargo package manager
- Platform-specific development tools (for FFI)

### Build Commands

```bash
# Basic build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with specific features
cargo build --features "enhanced-repl,async-runtime"

# Performance monitoring
cargo run --bin performance-monitor
```

### Feature Flags

Available feature flags for customizing the build:

- `minimal-repl`: Lightweight REPL with basic functionality
- `enhanced-repl`: Full-featured REPL with syntax highlighting
- `async-runtime`: Asynchronous runtime support
- `network-io`: Network I/O capabilities
- `ffi`: Foreign Function Interface support
- `benchmarks`: Performance benchmarking tools

## API Reference

### Core Primitives

The 42 core primitives form the foundation of all Scheme operations:

#### Arithmetic Operations
- `+`, `-`, `*`, `/`: Basic arithmetic
- `=`, `<`, `>`, `<=`, `>=`: Numeric comparisons
- `number?`, `integer?`, `real?`: Type predicates

#### List Operations
- `cons`, `car`, `cdr`: Fundamental pair operations
- `list`, `length`, `append`: List manipulation
- `null?`, `pair?`: List predicates

#### Control Flow
- `if`, `cond`, `case`: Conditional expressions
- `and`, `or`, `not`: Boolean operations
- `call/cc`: Continuations

#### I/O Operations
- `read`, `write`, `display`: Input/output
- `open-input-port`, `close-port`: Port management

### Advanced Features

#### Type System Integration

```scheme
;; Gradual typing example
(define (factorial (n : Integer)) : Integer
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))
```

#### Effect System Usage

```scheme
;; Effect handling
(with-effects
  (IO State)
  (do-io-operation)
  (modify-state value))
```

## Type System

### Gradual Typing

Lambdust supports gradual typing, allowing code to transition smoothly between dynamic and static typing:

- **Dynamic typing**: Traditional Scheme behavior with runtime type checking
- **Static typing**: Compile-time type verification with type annotations
- **Mixed mode**: Combine typed and untyped code seamlessly

### Type Classes

Support for Haskell-style type classes enables generic programming:

```scheme
(define-type-class Eq a
  (== : a -> a -> Boolean))

(define-instance (Eq Integer)
  (== = integer-equal?))
```

### Algebraic Data Types

Define custom data types with pattern matching:

```scheme
(define-data Maybe a
  Nothing
  (Just a))

(define (maybe-map f m)
  (match m
    (Nothing Nothing)
    ((Just x) (Just (f x)))))
```

## Effect System

### Monadic Architecture

The effect system is built on monadic principles, allowing pure functional management of side effects:

- **IO Effects**: File system, network, console operations
- **State Effects**: Mutable state management
- **Error Effects**: Exception handling and error propagation
- **Async Effects**: Asynchronous computation coordination

### Effect Handlers

Custom effect handlers can be defined for domain-specific effect management:

```scheme
(define-effect Logger
  (log-info : String -> Unit)
  (log-error : String -> Unit))

(define console-logger
  (handler Logger
    (log-info msg (display msg))
    (log-error msg (display msg stderr))))
```

## Concurrency

### Actor System

Lambdust provides an actor-based concurrency model:

- **Actors**: Isolated units of computation with private state
- **Message Passing**: Asynchronous communication between actors
- **Supervision**: Hierarchical error handling and recovery

### Futures and Promises

Asynchronous computation support through futures:

```scheme
(define future-result
  (future (expensive-computation)))

(define combined
  (await (map-future process-result future-result)))
```

### Software Transactional Memory (STM)

Lock-free concurrency through STM:

```scheme
(define account-balance (stm-var 1000))

(stm-atomic
  (lambda ()
    (let ((balance (stm-read account-balance)))
      (stm-write account-balance (- balance 100)))))
```

## Performance

### Optimization Features

- **Bytecode Compilation**: JIT compilation for frequently executed code
- **Primitive Specialization**: Type-specific optimizations
- **SIMD Operations**: Vectorized numeric computations
- **Memory Pooling**: Reduced allocation overhead

### Benchmarking

Performance monitoring is available through the dedicated performance monitor:

```bash
cargo run --bin performance-monitor --features benchmarks
```

### Memory Management

- **Zero-copy Operations**: Minimize memory allocation where possible
- **Reference Counting**: Automatic memory management for Scheme values
- **Memory Safety**: Rust's ownership system prevents memory leaks and unsafe access

## Contributing

### Development Guidelines

1. **Code Quality**: All code must pass `cargo clippy` with zero warnings
2. **Testing**: Comprehensive test coverage is required for new features
3. **Documentation**: All public APIs must be documented
4. **Performance**: Consider performance implications of changes

### Development Workflow

1. Create feature branch from main
2. Implement changes with tests
3. Ensure all tests pass: `cargo test`
4. Check code quality: `cargo clippy`
5. Submit pull request with detailed description

### Code Organization

- One structure per file
- Structure name matches file name
- No structures in `mod.rs` files
- Clear separation of concerns

## Changelog

### Version 0.1.1 (Current)

#### Features
- Complete R7RS-large compliance implementation
- Advanced type system with gradual typing
- Effect system with monadic architecture
- Concurrency primitives (actors, futures, STM)
- FFI system for C interoperability
- Performance monitoring and benchmarking tools

#### Improvements
- Replaced external dependencies with internal implementations
- Comprehensive test suite and quality assurance
- Optimized memory usage and allocation patterns
- Enhanced error reporting and diagnostics

#### Bug Fixes
- Resolved all compilation warnings and errors
- Fixed memory safety issues in FFI layer
- Corrected type inference edge cases

### Future Releases

#### Version 0.1.2 (Planned)
- Enhanced documentation and examples
- Performance optimizations and SIMD improvements
- Extended SRFI library implementations
- IDE integration support

#### Version 0.2.0 (Roadmap)
- JIT compilation improvements
- Advanced debugging tools
- Cross-platform binary distributions
- Community contribution guidelines