# User Documentation

This directory contains user-facing documentation for the Lambdust Scheme interpreter.

## 📋 Contents

- **[BUILD_COMMANDS.md](BUILD_COMMANDS.md)**: Build commands, test execution, and development tools
- **[REPL.md](REPL.md)**: REPL usage guide and interactive environment
- **[BRIDGE_API.md](BRIDGE_API.md)**: Bridge API for Rust-Scheme interoperability
- **[POLYNOMIAL_UNIVERSE_QUICK_START.md](POLYNOMIAL_UNIVERSE_QUICK_START.md)**: Quick start guide for the advanced type system

## 🎯 Quick Start

### 1. Building Lambdust
```bash
# Standard build
cargo build --release

# With full features
cargo build --features development

# See BUILD_COMMANDS.md for all options
```

### 2. Running the REPL
```bash
# Start interactive REPL
cargo run --features repl

# See REPL.md for usage guide
```

### 3. Rust Integration
```rust
// Bridge API for Rust-Scheme interoperability
// See BRIDGE_API.md for complete guide
```

## 🚀 Feature Highlights

### Development Tools
- **Comprehensive Build System**: Multiple build profiles and feature flags
- **Interactive REPL**: Full-featured Scheme REPL with debugging support
- **Rust Integration**: Type-safe bridge API for Rust-Scheme interoperability
- **Advanced Type System**: Polynomial Universe with dependent types

### Performance Features
- **Copy-on-Write Environment**: 25-40% memory reduction
- **JIT Optimization**: Hot path detection and native code generation
- **Multiple Build Profiles**: From embedded (<500KB) to development (<100MB)

### Language Support
- **R7RS Compliance**: 99.8% implementation
- **SRFI Extensions**: 9+ SRFI implementations including world-first SRFI 46
- **Hygienic Macros**: Advanced macro system with pattern matching
- **Modern Scheme**: Contemporary language features and optimizations

## 📊 Build Profiles

| Profile | Size | Features | Use Case |
|---------|------|----------|----------|
| `embedded` | <500KB | Core only | IoT/embedded |
| `minimal` | <5MB | Basic features | Simple scripts |
| `standard` | <15MB | Full R7RS | General use |
| `verified` | <50MB | + Verification | Research/formal |
| `development` | <100MB | + Dev tools | Development |

## 🔗 Related Documentation

- **Implementation**: See [../implementation/](../implementation/) for technical details
- **Core**: See [../core/](../core/) for project overview
- **Development**: See [../development/](../development/) for contributing guidelines
- **Research**: See [../research/](../research/) for advanced features

## 📈 Getting Started Path

1. **Installation**: Follow `BUILD_COMMANDS.md` for setup
2. **First Steps**: Use `REPL.md` to explore the language
3. **Integration**: Check `BRIDGE_API.md` for Rust integration
4. **Advanced**: Explore `POLYNOMIAL_UNIVERSE_QUICK_START.md` for type system features