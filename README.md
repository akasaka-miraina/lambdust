# Lambdust

A comprehensive R7RS-large compliant Scheme interpreter written in Rust, featuring advanced type systems, effect handling, and high-performance concurrent execution.

## Features

- **R7RS-large Compliance**: Full support for the R7RS-large standard with extensive SRFI implementations
- **Advanced Type System**: Gradual typing, algebraic data types, and type classes
- **Effect System**: Monadic programming with effect handlers for managing side effects
- **Concurrency**: Actor model, futures, Software Transactional Memory (STM)
- **FFI Support**: C interoperability with dynamic library loading
- **Performance Optimization**: Bytecode compilation and SIMD operations

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

## Documentation

For comprehensive documentation, see:
- [English Documentation](docs/DOCUMENTATION.md)
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

We welcome contributions! Please see our [Contributing Guidelines](docs/CONTRIBUTING.md) for details.

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