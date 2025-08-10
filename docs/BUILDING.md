# Building and Development Guide

This guide provides comprehensive instructions for building, developing, and contributing to the Lambdust project.

## Prerequisites

### System Requirements

- **Rust**: Version 1.75 or later (Rust 2024 edition)
- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 4GB RAM for compilation
- **Storage**: 2GB available space for full build

### Required Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add clippy rustfmt

# Optional: Install additional tools
cargo install cargo-watch cargo-expand cargo-tarpaulin
```

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/username/lambdust.git
cd lambdust

# Quick build
cargo build

# Optimized release build
cargo build --release
```

### 2. Run Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

### 3. Start Development

```bash
# Start interactive REPL
cargo run

# Run with development features
cargo run --features "dev,profiling"

# Run specific example
cargo run --example fibonacci
```

## Development Workflow

### Code Quality Standards

Lambdust maintains exceptional code quality through strict standards:

#### Compilation Standards

```bash
# Development phase (minimum requirement)
cargo check --lib                    # Must show 0 errors

# Commit phase (required before commits)
cargo clippy --lib                   # Must show 0 errors AND 0 warnings
cargo fmt                            # Code formatting
```

#### Incremental Development Protocol

**CRITICAL**: Follow the incremental development rules that have achieved 100% success across 226+ structure migrations:

1. **One Change at a Time**: Make exactly one focused change per step
2. **Immediate Verification**: Run `cargo check --lib` after each change
3. **Zero Error Tolerance**: Fix any errors immediately before proceeding
4. **Quality Gates**: Run `cargo clippy` before any commits

### Project Structure Understanding

Before contributing, understand the clean architecture achieved through refactoring:

```bash
# Explore the modular structure
find src/ -name "*.rs" | head -20

# Understand component organization
ls src/eval/monadic_architecture/     # 22 focused modules
ls src/concurrency/sync/              # 9 synchronization modules  
ls src/benchmarks/                    # Performance monitoring system
```

### Development Commands

#### Basic Development

```bash
# Watch for changes during development
cargo watch -x "check --lib"

# Continuous testing
cargo watch -x test

# Check specific module
cargo check --lib -p lambdust
```

#### Advanced Development

```bash
# Run with profiling
cargo run --features "profiling" --release

# Enable SIMD optimizations
cargo run --features "simd" --release  

# Debug mode with all features
cargo run --features "dev,debug,profiling"
```

#### Performance Analysis

```bash
# Run benchmarks
cargo bench

# Run with performance monitoring
cargo run --release --features "benchmark" -- --profile

# Memory usage analysis
cargo run --features "memory-profiling" --release
```

## Feature Flags

Lambdust supports various feature flags for different use cases:

### Core Features

```toml
[features]
default = ["r7rs-large", "stdlib"]

# Core language features
r7rs-large = []          # R7RS-large standard library
stdlib = []              # Built-in standard library
gradual-typing = []      # Gradual type system
effect-system = []       # Algebraic effects

# Performance features  
simd = ["dep:packed_simd"]           # SIMD optimizations
jit = ["dep:cranelift-jit"]          # JIT compilation
profiling = ["dep:pprof"]            # Performance profiling

# Development features
dev = ["debug", "testing"]           # Development mode
debug = []                           # Debug information
testing = []                         # Testing infrastructure
benchmark = []                       # Benchmarking system

# Concurrency features
actors = []                          # Actor model
parallel = ["dep:rayon"]             # Parallel evaluation
distributed = ["dep:tokio"]          # Distributed computing
```

### Usage Examples

```bash
# Minimal build
cargo build --no-default-features

# Performance-optimized build
cargo build --release --features "simd,jit,profiling"

# Development build with all tools
cargo build --features "dev,benchmark,actors"
```

## Testing

### Test Organization

```bash
# Unit tests (embedded in source files)
cargo test --lib

# Integration tests
cargo test --test integration

# Documentation tests
cargo test --doc

# Benchmark tests
cargo bench
```

### Test Categories

#### Core Language Tests
- R7RS compliance tests
- Type system tests  
- Effect system tests
- Macro expansion tests

#### Performance Tests
- Benchmarking suite tests
- Regression detection tests
- Statistical analysis tests
- Memory usage tests

#### System Tests
- Concurrency tests
- FFI integration tests
- Module system tests
- Error handling tests

### Running Specific Test Suites

```bash
# R7RS compliance
cargo test r7rs

# Type system
cargo test type_system

# Concurrency
cargo test concurrency

# Performance benchmarks  
cargo test --bench comprehensive_benchmark_suite
```

## Development Tools

### Code Analysis

```bash
# Comprehensive linting
cargo clippy --all-features --all-targets

# Security audit
cargo audit

# Dependency analysis
cargo tree
```

### Profiling and Debugging

```bash
# CPU profiling
cargo run --release --features "profiling" -- --cpu-profile

# Memory profiling
cargo run --release --features "profiling" -- --memory-profile

# Debug with GDB
cargo build && gdb target/debug/lambdust
```

### Documentation Generation

```bash
# Generate API documentation
cargo doc --all-features --open

# Check documentation coverage
cargo doc --all-features --document-private-items
```

## Contributing Guidelines

### Before Making Changes

1. **Understand the Architecture**: Read [ARCHITECTURE.md](ARCHITECTURE.md)
2. **Follow Code Standards**: Review [CONTRIBUTING.md](CONTRIBUTING.md)  
3. **Set up Development Environment**: Install required tools
4. **Run Initial Tests**: Ensure everything works on your system

### Development Process

1. **Create Feature Branch**: `git checkout -b feature/your-feature`
2. **Follow Incremental Development**: One change at a time with verification
3. **Maintain Quality Standards**: Zero errors and warnings
4. **Add Tests**: Cover new functionality with appropriate tests
5. **Update Documentation**: Keep documentation synchronized with changes

### Quality Checklist

Before submitting any changes:

- [ ] `cargo check --lib` returns 0 errors
- [ ] `cargo clippy --lib` returns 0 errors and 0 warnings  
- [ ] `cargo test` passes all tests
- [ ] `cargo fmt` has been applied
- [ ] Documentation updated where appropriate
- [ ] Following incremental development principles

## Troubleshooting

### Common Issues

#### Compilation Errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version  # Should be 1.75+
```

#### Performance Issues

```bash
# Profile the application
cargo run --release --features "profiling"

# Check memory usage
cargo run --features "memory-profiling"

# Run benchmarks
cargo bench --bench performance_suite
```

#### Test Failures

```bash
# Run tests with output
cargo test -- --nocapture --test-threads=1

# Run specific failing test
cargo test test_name -- --exact --nocapture

# Debug test with GDB
cargo test --no-run && gdb target/debug/deps/test_binary
```

### Getting Help

1. **Documentation**: Check comprehensive documentation in `docs/`
2. **API Reference**: Generate with `cargo doc --open`
3. **Architecture Guide**: Review system design in [ARCHITECTURE.md](ARCHITECTURE.md)
4. **Issues**: Report bugs or ask questions on GitHub Issues

## Performance Optimization

### Build Optimization

```bash
# Maximum performance build
cargo build --release \
  --features "simd,jit" \
  --target-cpu=native

# Link-time optimization
RUSTFLAGS="-C lto=fat" cargo build --release
```

### Runtime Optimization

```bash
# Enable all performance features
cargo run --release \
  --features "simd,jit,profiling" \
  -- --optimize-level=3

# Parallel evaluation
cargo run --release \
  --features "parallel,actors" \
  -- --threads=auto
```

This development guide reflects the current high-quality state of Lambdust after comprehensive structural refactoring and quality improvements. The development workflow emphasizes incremental development with continuous verification to maintain the exceptional quality standards achieved.