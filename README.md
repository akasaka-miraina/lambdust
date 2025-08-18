# Lambdust

[![CI](https://github.com/username/lambdust/workflows/Continuous%20Integration/badge.svg)](https://github.com/akasaka-miraina/lambdust/actions/workflows/ci.yml)
[![Performance](https://github.com/username/lambdust/workflows/Performance%20Testing/badge.svg)](https://github.com/akasaka-miraina/lambdust/actions/workflows/performance.yml)
[![R7RS Compliance](https://github.com/username/lambdust/workflows/R7RS%20Compliance%20Testing/badge.svg)](https://github.com/akasaka-miraina/lambdust/actions/workflows/r7rs-compliance.yml)
[![Security](https://github.com/username/lambdust/workflows/Security%20Audit/badge.svg)](https://github.com/username/akasaka-miraina/actions/workflows/security.yml)
[![Documentation](https://github.com/username/lambdust/workflows/Documentation/badge.svg)](https://github.com/username/akasaka-miraina/actions/workflows/docs.yml)
[![Code Quality](https://img.shields.io/badge/clippy-warnings%200-green.svg)](https://github.com/rust-lang/rust-clippy)

A high-performance R7RS-large compliant Scheme interpreter written in Rust, featuring modular architecture, advanced type systems, and enterprise-grade development practices.

## Features

### 🏗️ **Modular Architecture**
- **Optimized File Structure**: Modularized codebase with <20,000 tokens per file for enhanced maintainability
- **stdlib/lists/**: 9 specialized modules (basic, predicates, accessors, higher-order functions, etc.)
- **stdlib/strings/**: 4 focused modules for comprehensive string operations
- **Zero Compilation Warnings**: Enterprise-grade code quality with complete clippy compliance

### 🚀 **R7RS Compliance**
- **R7RS-large Standard**: Full support with extensive SRFI implementations
- **Contract System**: Built-in design-by-contract programming with blame tracking
- **Macro System**: Advanced hygiene-preserving macros with syntax-case support

### 🔬 **Advanced Type Systems**
- **Gradual Typing**: Seamless integration of static and dynamic typing
- **Dependent Types**: Pi-types and Sigma-types for expressive specifications  
- **Algebraic Data Types**: Pattern matching and type classes support

### ⚡ **Performance & Concurrency**
- **JIT Compilation**: Hotspot detection and optimization
- **Effect System**: Monadic programming with algebraic effects
- **Actor Model**: High-performance concurrent execution
- **SIMD Operations**: Vectorized numeric computations

## Quick Start

```bash
# Clone the repository
git clone https://github.com/username/lambdust.git
cd lambdust

# Build the project
cargo build --release

# Run the REPL
cargo run

# Run performance monitor
cargo run --bin performance-monitor
```

## Example

```scheme
;; Factorial with gradual typing
(define (factorial (n : Integer)) : Integer
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

;; Actor-based concurrency
(define counter-actor
  (spawn-actor
    (lambda (msg)
      (match msg
        ((increment) (update-state (+ (get-state) 1)))
        ((get) (reply (get-state)))))))

;; Effect handling
(with-effects
  (IO State)
  (log-info "Starting computation")
  (let ((result (expensive-computation)))
    (save-state result)))
```

## Architecture

### 📁 **Modular Codebase Structure**

Lambdust follows a domain-driven modular architecture optimized for maintainability and performance:

```
src/
├── stdlib/                    # Standard Library Implementation
│   ├── lists/                # List operations (9 modules)
│   │   ├── basic.rs          # cons, car, cdr, list construction
│   │   ├── predicates.rs     # pair?, null?, list? predicates
│   │   ├── accessors.rs      # list-ref, length, list-tail
│   │   ├── manipulation.rs   # append, reverse, set-car!/cdr!
│   │   ├── higher_order.rs   # map, filter, fold-left/right
│   │   ├── utilities.rs      # member, assoc, sort operations
│   │   └── srfi1.rs          # SRFI-1 compliance extensions
│   ├── strings/              # String operations (4 modules)
│   │   ├── basic.rs          # string construction and basic ops
│   │   ├── predicates.rs     # string predicates and tests
│   │   └── common.rs         # shared utilities and constants
│   └── arithmetic.rs         # Comprehensive numeric tower
├── eval/                     # Evaluation Engine
│   ├── evaluator.rs          # Core evaluation logic
│   ├── value.rs              # Value representation and operations
│   └── environment.rs        # Environment management
├── types/                    # Type System Implementation
│   ├── gradual_system.rs     # Gradual typing infrastructure
│   ├── dependent/            # Dependent type theory
│   └── inference.rs          # Type inference engine
├── macro_system/             # Advanced Macro Processing
├── contracts/                # Design-by-Contract System
└── jit/                      # Just-In-Time Compilation
```

### 🔧 **Development Quality Standards**

- **Zero Compilation Warnings**: Enforced clippy compliance across entire codebase
- **Token-Optimized Files**: All source files <20,000 tokens for optimal AI-assisted development
- **Subagent Collaboration**: Systematic use of specialized AI agents for different domains
- **Phase-Gate Quality**: Error/warning-free completion required for each development phase

## Documentation

### For Users
- [User Guide](docs/user_guide.md) - Getting started and usage examples
- [Features Overview](FEATURES.md) - Complete feature list
- [Roadmap](NEXT_STEPS_ROADMAP.md) - Development roadmap

### For Developers  
- [Development Documentation](docs/development/README.md) - API references and implementation guides
- [Architecture Documentation](docs/architecture/README.md) - System architecture and design

### Language Support
- [Japanese Documentation](docs/ja/DOCUMENTATION.md)

## Building

### Prerequisites
- Rust 1.75.0 or later
- Cargo package manager

### Available Features
- `minimal-repl`: Lightweight REPL
- `enhanced-repl`: Full-featured REPL with syntax highlighting
- `async-runtime`: Asynchronous runtime support
- `network-io`: Network I/O capabilities
- `ffi`: Foreign Function Interface support

## Testing

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features "enhanced-repl,async-runtime"

# Check code quality
cargo clippy
```

## Performance

Lambdust is designed for high performance with:
- Zero-copy operations where possible
- SIMD-optimized numeric computations
- JIT compilation for hot paths
- Efficient memory management

## Contributing

We welcome contributions! Please see our [Development Documentation](docs/development/README.md) for contribution guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure `cargo clippy` passes with zero warnings
5. Submit a pull request

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

This project builds upon decades of Scheme language development and the Rust ecosystem. Special thanks to the R7RS working group and the Rust community.