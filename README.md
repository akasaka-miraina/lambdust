# Lambdust (λust) - Modern Scheme with Gradual Typing and Effects

[![Rust](https://img.shields.io/badge/rust-2024%20edition-blue)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#building)

A modern Scheme interpreter that combines the simplicity and elegance of Scheme with advanced programming language features including gradual typing, effect systems, and high-performance implementation.

## ✨ Features

### 🎯 **Core Language**
- **R7RS-large compliance** with extensive SRFI support
- **Fully hygienic macro system** with R7RS-compatible syntax-rules
- **Tail call optimization** and proper lexical scoping
- **42 core primitives** bootstrapping the entire system

### 🏗️ **Advanced Type System**
- **Four-level gradual typing**: Dynamic → Contracts → Static → Dependent
- **Hindley-Milner type inference** with rank-n polymorphism
- **Algebraic data types** and pattern matching
- **Type classes** with Haskell-style constraints
- **Row polymorphism** for extensible records

### 🎭 **Effect System**  
- **Transparent effect tracking** preserving Scheme semantics
- **Monadic programming** with IO, State, Error, and custom effects
- **Effect handlers** for custom effect management
- **Generational environments** handling mutations functionally

### ⚡ **Performance & Concurrency**
- **Bytecode compilation** with optimization passes
- **Multi-threaded parallel evaluation** with actor model
- **Lock-free data structures** and STM support
- **SIMD optimization** for numeric operations
- **Profile-guided optimization** and JIT compilation support

### 🔗 **Interoperability**
- **Foreign Function Interface** for C/Rust interoperability
- **Dynamic library loading** with type-safe bindings
- **Comprehensive I/O system** including async/await and network operations
- **Module system** with R7RS-compatible libraries

## Quick Start

### Installation

```bash
cargo build --release
```

### Running the REPL

```bash
cargo run --features repl
```

### Executing a file

```bash
cargo run examples/fibonacci.scm
```

### Evaluating an expression

```bash
cargo run -- --eval "(+ 1 2 3)"
```

### Hello, World!

```scheme
;; Basic R7RS Scheme
(display "Hello, World!")
(newline)

;; With type annotations and effects
(define (greet name)
  #:type (-> String (IO Unit))
  #:pure #f
  (display "Hello, ")
  (display name)
  (newline))

(greet "Lambdust")
```

## Language Overview

### Core Language

Lambdust has exactly 42 core primitives that bootstrap the entire system, implementing:

- **Special forms**: `quote`, `lambda`, `if`, `define`, `set!`, `define-syntax`
- **Control flow**: `call-with-current-continuation`, conditionals
- **Type annotations**: `::` for gradual typing
- **Effect tracking**: Automatic effect inference and monadic lifting
- **FFI primitives**: `primitive` for built-in operations

### Advanced Examples

```scheme
;; Gradual typing - start dynamic, add types incrementally
(define (factorial n)
  #:type (-> Number Number)
  #:contract (-> (and Number (>= 0)) Number)
  #:pure #t
  (if (zero? n) 1 (* n (factorial (- n 1)))))

;; Effect system with monadic programming  
(define (safe-divide x y)
  #:type (-> Number Number (Either String Number))
  (if (zero? y)
      (Left "Division by zero")
      (Right (/ x y))))

;; Concurrent programming with actors
(define counter-actor
  (actor
    [(initial-state 0)]
    [(increment n) (+ state n)]
    [(get) state]))

;; Pattern matching with algebraic data types
(define-type Maybe (a)
  Nothing
  (Just a))

(define (maybe-map f maybe-val)
  (match maybe-val
    [Nothing Nothing]
    [(Just x) (Just (f x))]))

;; FFI with Rust/C functions
(define-ffi "libmath.so"
  [fast-sqrt (-> Number Number) "sqrt"])

(fast-sqrt 16) ; => 4.0
```

## 📁 Project Structure

```
lambdust/
├── 🎯 Core Implementation
│   ├── src/lexer/              # Tokenization and lexical analysis
│   ├── src/parser/             # Expression parsing and AST generation
│   ├── src/ast/                # Abstract syntax tree definitions
│   ├── src/eval/               # Core evaluation engine and environments
│   └── src/diagnostics/        # Error handling and reporting
│
├── 🏗️ Language Systems  
│   ├── src/types/              # Four-level gradual type system
│   ├── src/effects/            # Effect system with monadic programming
│   ├── src/macro_system/       # Hygienic macro expansion
│   ├── src/module_system/      # R7RS module system and library loading
│   └── src/metaprogramming/    # Reflection and code generation
│
├── ⚡ Runtime & Performance
│   ├── src/runtime/            # Runtime system coordination  
│   ├── src/bytecode/           # Bytecode compiler and virtual machine
│   ├── src/concurrency/        # Actor system and parallel evaluation
│   ├── src/containers/         # High-performance data structures
│   └── src/benchmarks/         # Performance analysis and regression detection
│
├── 🔗 Interoperability
│   ├── src/ffi/                # Foreign function interface
│   ├── src/stdlib/             # R7RS standard library + extensions
│   ├── src/numeric/            # Advanced numeric tower
│   └── src/utils/              # Memory management and profiling
│
├── 🎮 User Interface
│   ├── src/repl/               # Enhanced REPL with debugging
│   ├── src/main.rs             # CLI application entry point
│   └── src/lib.rs              # Library interface
│
└── 📚 Resources
    ├── stdlib/                 # Scheme standard library modules
    ├── examples/               # Example programs and demos
    ├── docs/                   # Comprehensive documentation
    └── tests/                  # Test suites and benchmarks
```

## 🏛️ Architecture Highlights

### 🧠 **Clean Modular Design** 
- **226+ structures** organized with one-structure-per-file principle
- **Zero compilation errors** and zero clippy warnings maintained
- **Professional documentation** across all public interfaces
- **Incremental development** with continuous quality assurance

### 🎯 **Type System Integration**
- **Integration Bridge** connecting dynamic and static typing seamlessly  
- **Gradual Typing** allowing smooth transitions between type levels
- **R7RS Integration** maintaining backward compatibility

### 🎭 **Effect System Architecture**
- **Effect Coordination** managing side effects and I/O transparently
- **Monadic Architecture** with comprehensive effect handlers
- **Generational Environments** handling state changes functionally

## 🏃‍♂️ Getting Started

### Prerequisites

- Rust 2024 edition or later
- System dependencies for optional features (see [BUILDING.md](BUILDING.md))

### Basic Usage

```bash
# Interactive REPL
cargo run --features enhanced-repl

# Execute a file
cargo run examples/fibonacci.scm

# Evaluate expression  
cargo run -- --eval "(map (lambda (x) (* x x)) '(1 2 3 4 5))"

# Enable type checking
cargo run -- --type-level static examples/typed-program.scm

# Parallel evaluation
cargo run --bin native-benchmark-runner -- --parallel 4
```

### Development

```bash
# Development build with error checking
cargo check --lib

# Run all tests
cargo test

# Performance benchmarks  
cargo bench --features benchmarks

# Code quality (required for contributions)
cargo clippy
cargo fmt
```

## 📊 Performance

Lambdust achieves excellent performance through multiple optimization strategies:

- **Bytecode compilation** with multi-pass optimization
- **Primitive specialization** based on type information  
- **SIMD vectorization** for numeric operations
- **Memory pooling** for allocation-heavy workloads
- **Lock-free concurrency** with work-stealing schedulers

See [PERFORMANCE.md](PERFORMANCE.md) for detailed benchmarks and optimization guides.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Code organization standards
- Development workflow and quality requirements
- Testing guidelines and documentation standards
- Submission process and review criteria

### Quick Contribution Checklist

1. ✅ `cargo check --lib` shows 0 errors
2. ✅ `cargo clippy` shows 0 errors and warnings  
3. ✅ `cargo test` passes all tests
4. ✅ Documentation updated for new features
5. ✅ One focused change per pull request

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture and component design |
| [BUILDING.md](BUILDING.md) | Build instructions and development setup |  
| [API_REFERENCE.md](API_REFERENCE.md) | Core API documentation and examples |
| [TYPE_SYSTEM.md](TYPE_SYSTEM.md) | Gradual typing and type inference |
| [EFFECT_SYSTEM.md](EFFECT_SYSTEM.md) | Effect system and monadic programming |
| [CONCURRENCY.md](CONCURRENCY.md) | Concurrency model and parallel evaluation |
| [FFI.md](FFI.md) | Foreign function interface guide |
| [PERFORMANCE.md](PERFORMANCE.md) | Performance optimization and benchmarking |
| [docs/ja/](docs/ja/) | Japanese documentation |

## 🧪 Testing

Comprehensive testing strategy ensuring reliability:

- **Unit tests** in each module with 95%+ coverage
- **Integration tests** for end-to-end functionality
- **R7RS compliance tests** ensuring standard conformance  
- **SRFI implementation tests** for extended functionality
- **Performance regression tests** maintaining optimization gains
- **Property-based testing** with randomized inputs

```bash
cargo test                    # All tests
cargo test --release          # Release mode testing  
cargo test integration::      # Integration tests only
cargo bench                   # Performance benchmarks
```

## 🔖 Version Information

- **Language Version**: 0.1.0
- **R7RS Compliance**: Full R7RS-large support
- **Rust Edition**: 2024
- **SRFI Support**: 50+ SRFIs implemented

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🌟 Acknowledgments

- R7RS Working Group for the Scheme specification
- Rust community for excellent tooling and libraries
- Academic research in type theory and effect systems
- Open source Scheme implementations inspiring this work

## 🎯 Future Roadmap

- **Dependent types** with proof assistant capabilities
- **JIT compilation** for performance-critical code paths  
- **Distributed computing** with transparent remote evaluation
- **IDE integration** with Language Server Protocol support
- **WebAssembly target** for browser-based execution

---

*Lambdust: Where Lisp elegance meets modern type theory* ✨