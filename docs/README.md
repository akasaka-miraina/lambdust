# Lambdust Documentation

Welcome to the comprehensive documentation for Lambdust (λust), a Scheme dialect that combines the simplicity and elegance of Scheme with modern type theory and functional programming concepts.

## Documentation Structure

### 📖 [User Guide](user-guide/)
- [Getting Started](user-guide/getting-started.md) - Installation, basic usage, and first steps
- [Language Overview](user-guide/language-overview.md) - Core language features and syntax
- [Type System](user-guide/type-system.md) - Gradual typing from dynamic to dependent types
- [Effect System](user-guide/effect-system.md) - Pure functional programming with effect tracking
- [REPL Guide](user-guide/repl-guide.md) - Interactive development environment

### 📚 [Tutorials](tutorials/)
- [Basic Programming](tutorials/basic-programming.md) - Fundamental programming concepts
- [Functional Programming](tutorials/functional-programming.md) - Higher-order functions and patterns
- [Type Annotations](tutorials/type-annotations.md) - Static typing and type inference
- [Effect Management](tutorials/effect-management.md) - Working with side effects
- [Module System](tutorials/module-system.md) - Organizing code with modules
- [Advanced Features](tutorials/advanced-features.md) - Macros, continuations, and more

### 📋 [API Reference](api/)
- [Built-in Functions](api/builtins.md) - Core language procedures
- [Standard Library](api/stdlib/) - R7RS-compliant standard library
  - [Arithmetic](api/stdlib/arithmetic.md) - Number operations and mathematics
  - [Strings](api/stdlib/strings.md) - String manipulation and processing
  - [Lists](api/stdlib/lists.md) - List operations and higher-order functions
  - [Vectors](api/stdlib/vectors.md) - Vector operations and utilities
  - [I/O](api/stdlib/io.md) - Input/output operations and ports
  - [Characters](api/stdlib/characters.md) - Character operations and predicates
  - [Control Flow](api/stdlib/control.md) - Control flow procedures
- [SRFI Support](api/srfi/) - Scheme Request for Implementation support
- [FFI](api/ffi.md) - Foreign Function Interface with Rust

### 🔄 [Migration Guides](migration/)
- [From Other Schemes](migration/from-other-schemes.md) - Migrating from R5RS, R6RS, R7RS implementations
- [Version Upgrades](migration/version-upgrades.md) - Upgrading between Lambdust versions
- [Breaking Changes](migration/breaking-changes.md) - Comprehensive list of breaking changes
- [Compatibility Matrix](migration/compatibility.md) - Feature compatibility across versions

### 🛠️ [Developer Documentation](developer/)
- [Contributing](developer/contributing.md) - How to contribute to Lambdust
- [Architecture](developer/architecture.md) - System design and implementation
- [Building from Source](developer/building.md) - Development environment setup
- [Testing](developer/testing.md) - Running tests and benchmarks
- [Performance](developer/performance.md) - Optimization and profiling
- [Internals](developer/internals/) - Deep dive into implementation details

### 📝 [Examples](examples/)
- [Basic Examples](examples/basic/) - Simple programs demonstrating core features
- [Advanced Examples](examples/advanced/) - Complex applications and patterns  
- [Performance Showcases](examples/performance/) - Optimized implementations
- [Integration Examples](examples/integration/) - Using Lambdust with other systems

## Quick Navigation

### For New Users
1. Start with [Getting Started](user-guide/getting-started.md)
2. Follow the [Basic Programming Tutorial](tutorials/basic-programming.md)
3. Explore [Language Overview](user-guide/language-overview.md)

### For Experienced Scheme Programmers
1. Check [Migration from Other Schemes](migration/from-other-schemes.md)
2. Review [Language Overview](user-guide/language-overview.md) for Lambdust-specific features
3. Explore [Type System](user-guide/type-system.md) and [Effect System](user-guide/effect-system.md)

### For Contributors
1. Read [Contributing Guidelines](developer/contributing.md)
2. Understand the [Architecture](developer/architecture.md)
3. Set up your [Development Environment](developer/building.md)

## Key Features

- **R7RS Compatible**: Existing Scheme code runs without modification
- **Gradually Typed**: Seamless progression from dynamic to static to dependent types
- **Pure Functional**: Transparent effect tracking and management
- **High Performance**: JIT compilation and advanced optimizations
- **Rust Integration**: Native FFI for seamless interoperability

## Community and Support

- **Issues**: Report bugs and request features on [GitHub Issues](https://github.com/username/lambdust/issues)
- **Discussions**: Join the community on [GitHub Discussions](https://github.com/username/lambdust/discussions)
- **Contributing**: See [Contributing Guidelines](developer/contributing.md)

## License

Lambdust is dual-licensed under MIT OR Apache-2.0. See [LICENSE-MIT](../LICENSE-MIT) and [LICENSE-APACHE](../LICENSE-APACHE) for details.