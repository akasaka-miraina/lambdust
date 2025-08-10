# Building Lambdust

This document provides comprehensive instructions for building, testing, and developing Lambdust from source.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites) 
3. [Building](#building)
4. [Development Workflow](#development-workflow)
5. [Testing](#testing)
6. [Benchmarking](#benchmarking)
7. [Feature Flags](#feature-flags)
8. [Platform-Specific Notes](#platform-specific-notes)
9. [Troubleshooting](#troubleshooting)

## Quick Start

```bash
# Clone and build
git clone https://github.com/username/lambdust.git
cd lambdust
cargo build --release

# Run the REPL
cargo run --features repl

# Run tests
cargo test
```

## Prerequisites

### Required Dependencies

- **Rust 2024 Edition** or later
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Update: `rustup update`

### Optional System Dependencies

#### For FFI Support (`--features ffi`)
- **pkg-config** (Linux/macOS)
- **libffi-dev** (Linux) / **libffi** (macOS via brew)
- **C compiler** (gcc, clang, or MSVC)

```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libffi-dev build-essential

# macOS (Homebrew)  
brew install pkg-config libffi

# Windows (MSYS2)
pacman -S mingw-w64-x86_64-pkg-config mingw-w64-x86_64-libffi
```

#### For Enhanced REPL (`--features enhanced-repl`)
- No additional system dependencies required

#### For Advanced I/O (`--features advanced-io`)
- **OpenSSL development libraries** (for TLS support)

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# macOS
brew install openssl

# Windows
# OpenSSL is handled automatically by the `openssl-sys` crate
```

## Building

### Development Build

```bash
# Fast development build with debug symbols
cargo build

# Library-only build (faster for development)
cargo build --lib

# Check compilation without building
cargo check --lib
```

### Release Build

```bash
# Optimized release build
cargo build --release

# Release build with specific features
cargo build --release --features "repl,ffi,advanced-io"
```

### Build Profiles

The project uses optimized build profiles defined in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit for better optimization
panic = "abort"         # Smaller binary size

[profile.dev]
opt-level = 0           # Fast compilation
debug = true            # Debug symbols
overflow-checks = true  # Runtime overflow checking
```

### Binary Targets

Lambdust includes several binary targets:

```bash
# Main REPL and CLI
cargo build --bin lambdust

# Scheme comparison benchmarks
cargo build --bin scheme-comparison

# Native benchmark runner
cargo build --bin native-benchmark-runner

# Performance monitoring
cargo build --bin performance-monitor
```

## Development Workflow

### Code Quality Standards

Lambdust maintains **zero compilation errors** and **zero clippy warnings** throughout development:

```bash
# Primary development check (REQUIRED)
cargo check --lib

# Code quality check (REQUIRED for contributions)
cargo clippy --lib

# Format code
cargo fmt

# Full quality check
cargo clippy --lib -- -D warnings
```

### Incremental Development Process

Follow the **incremental development rules** documented in `CLAUDE.md`:

1. **Before Changes**: Run `cargo check --lib` to establish baseline
2. **During Development**: Check compilation after every modification
3. **Error Requirement**: Error count must never increase
4. **Quality Gates**: `cargo clippy` must show 0 errors and warnings before commits

### Development Commands

```bash
# Quick syntax/type check (use during development)
cargo check --lib

# Full compilation check
cargo build --lib

# Test specific module
cargo test eval::tests

# Test with backtrace on failure
RUST_BACKTRACE=1 cargo test

# Watch for changes and rebuild
cargo install cargo-watch
cargo watch -x "check --lib"
```

## Testing

### Test Categories

Lambdust has comprehensive test coverage across multiple categories:

#### Unit Tests
```bash
# All unit tests
cargo test --lib

# Specific module tests
cargo test ast::tests
cargo test eval::tests::test_evaluation
cargo test types::tests::test_inference
```

#### Integration Tests
```bash
# All integration tests
cargo test --test integration

# R7RS compliance tests
cargo test r7rs_compliance

# SRFI implementation tests
cargo test srfi_tests
```

#### Documentation Tests
```bash
# Test code examples in documentation
cargo test --doc
```

### Test Configuration

#### Release Mode Testing
```bash
# Test in release mode for performance-sensitive tests
cargo test --release
```

#### Parallel Testing
```bash
# Control test parallelism
cargo test -- --test-threads=4

# Run tests sequentially (for debugging)
cargo test -- --test-threads=1
```

#### Test Output Control
```bash
# Show all test output
cargo test -- --nocapture

# Show only failed test output
cargo test -- --show-output
```

### Property-Based Testing

Some tests use `proptest` for randomized testing:

```bash
# Run with custom iteration count
PROPTEST_CASES=10000 cargo test prop_

# Generate failure reproduction cases
PROPTEST_VERBOSE=1 cargo test prop_failing_test
```

## Benchmarking

### Running Benchmarks

```bash
# All benchmarks (requires --features benchmarks)
cargo bench --features benchmarks

# Specific benchmark suite
cargo bench --bench core_performance_benchmarks

# Scheme comparison benchmarks
cargo bench --bench scheme_operation_benchmarks

# Container performance
cargo bench --bench containers
```

### Benchmark Categories

- **Core Performance**: Basic operations and primitive functions
- **Memory Usage**: Memory allocation and garbage collection
- **Latency**: Response time measurements
- **Parallel Evaluation**: Multi-threaded performance
- **Regression Testing**: Performance regression detection

### Benchmark Output

Benchmarks produce HTML reports when run:
```bash
cargo bench --features benchmarks
# Reports generated in target/criterion/
```

## Feature Flags

Lambdust uses feature flags to control compilation and dependencies:

### Default Features
```bash
# Default feature set
cargo build --features "repl,async,advanced-io"
```

### Available Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `repl` | Basic REPL support | `rustyline`, `colored`, `dirs` |
| `enhanced-repl` | Advanced REPL with syntax highlighting | `reedline`, `nu-ansi-term`, `crossterm`, `syntect` |
| `async` | Async I/O and runtime | `tokio`, `tokio-util` |
| `advanced-io` | Network I/O and TLS | `rustls`, `webpki-roots` |
| `ffi` | Foreign Function Interface | `libffi`, `cc` |
| `benchmarks` | Performance benchmarking | `criterion`, `flame` |
| `property-testing` | Property-based tests | `proptest` |
| `compression` | Data compression | `flate2`, `zstd`, `lz4_flex` |
| `tls` | TLS/SSL support | `rustls`, `webpki-roots` |

### Feature Combinations

```bash
# Minimal build
cargo build --no-default-features

# Full-featured build
cargo build --features "enhanced-repl,ffi,benchmarks,compression,tls"

# Development build with testing
cargo build --features "property-testing,benchmarks"
```

## Platform-Specific Notes

### Linux

- **Dependencies**: Most dependencies available via package manager
- **Performance**: Best overall performance platform
- **FFI**: Full support for dynamic library loading

```bash
# Ubuntu/Debian full setup
sudo apt-get update
sudo apt-get install build-essential pkg-config libffi-dev libssl-dev
```

### macOS

- **Requirements**: Xcode command line tools or Xcode
- **Dependencies**: Install via Homebrew
- **Architecture**: Native support for both Intel and Apple Silicon

```bash
# Install Xcode command line tools
xcode-select --install

# Install dependencies via Homebrew
brew install pkg-config libffi openssl
```

### Windows

- **Toolchain**: MSVC toolchain recommended (via Visual Studio)
- **Alternative**: MSYS2/MinGW-w64 supported
- **Dependencies**: Most handled automatically by crates

```bash
# Using MSVC (recommended)
rustup toolchain install stable-x86_64-pc-windows-msvc

# Using MSYS2
rustup toolchain install stable-x86_64-pc-windows-gnu
```

## Performance Optimization

### Compiler Optimizations

```bash
# Maximum optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Profile-guided optimization (PGO)
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" cargo build --release
# Run benchmarks to generate profile data
RUSTFLAGS="-C profile-use=/tmp/pgo-data" cargo build --release
```

### Link-Time Optimization

```bash
# Full LTO (already enabled in release profile)
cargo build --release

# Thin LTO for faster builds
CARGO_PROFILE_RELEASE_LTO=thin cargo build --release
```

## Development Tools

### Recommended Tools

```bash
# Install useful development tools
cargo install cargo-watch    # Auto-rebuild on changes  
cargo install cargo-expand   # Macro expansion
cargo install cargo-audit    # Security audit
cargo install cargo-bloat    # Binary size analysis
cargo install flamegraph     # Profiling
```

### IDE Integration

#### VS Code
- Install Rust extension (`rust-lang.rust-analyzer`)
- Configure settings for Lambdust project structure

#### IntelliJ/CLion
- Install Rust plugin
- Import as Cargo project

### Debugging

```bash
# Debug build with full symbols
cargo build --profile dev

# Run with debugger
rust-gdb target/debug/lambdust
# or
lldb target/debug/lambdust
```

## Troubleshooting

### Common Issues

#### Compilation Errors

**Issue**: `cargo check --lib` shows errors
**Solution**: Follow incremental development workflow, fix errors immediately

**Issue**: FFI compilation fails
**Solution**: Ensure system dependencies are installed, check feature flags

**Issue**: OpenSSL linking errors on Linux
**Solution**: 
```bash
sudo apt-get install libssl-dev pkg-config
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
```

#### Performance Issues

**Issue**: Slow compilation
**Solution**: Use `cargo check --lib` for development, enable `dev` profile

**Issue**: Slow test execution
**Solution**: Use `cargo test --release` for performance-sensitive tests

#### Platform-Specific Problems

**macOS**: Library not found errors
```bash
# Fix library paths
export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
```

**Windows**: MSVC linker issues
```bash
# Ensure Visual Studio Build Tools are installed
# Or switch to MinGW toolchain
rustup default stable-x86_64-pc-windows-gnu
```

### Getting Help

1. Check this documentation first
2. Search existing issues on GitHub
3. Run with `RUST_BACKTRACE=1` for detailed error information
4. Include your platform information and Rust version when reporting issues

```bash
# Gather system information
rustc --version
cargo --version
uname -a  # Linux/macOS
# or
systeminfo | findstr /B /C:"OS Name" /C:"OS Version"  # Windows
```

## Continuous Integration

The project uses GitHub Actions for automated testing across platforms:

- **Linux**: Ubuntu latest with full feature testing
- **macOS**: macOS latest with FFI testing
- **Windows**: Windows latest with MSVC toolchain
- **Coverage**: Code coverage reporting with `tarpaulin`
- **Quality**: Clippy linting and formatting checks

All pull requests must pass the CI pipeline before merging.

---

This build system is designed to support the high-quality, zero-error development workflow that enabled the successful refactoring of 226+ structures with a 100% success rate.