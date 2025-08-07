# Lambdust Documentation Index

This is a comprehensive index of all Lambdust documentation, organized by topic and audience.

## Quick Start Paths

### 🚀 New to Lambdust
1. [Getting Started](user-guide/getting-started.md) - Installation and first steps
2. [Basic Programming Tutorial](tutorials/basic-programming.md) - Learn the fundamentals
3. [Language Overview](user-guide/language-overview.md) - Core concepts and features

### 🔄 Migrating from Other Schemes
1. [Migration Guide](migration/from-other-schemes.md) - Compatibility and differences
2. [Breaking Changes](migration/breaking-changes.md) - What might need updating
3. [API Reference](api/builtins.md) - Function compatibility

### 👨‍💻 Contributing to Lambdust
1. [Contributing Guidelines](developer/contributing.md) - How to get involved
2. [Architecture Overview](developer/architecture.md) - System design
3. [Building from Source](developer/building.md) - Development setup

## Documentation by Type

### 📖 User Documentation

#### Getting Started
- **[Getting Started](user-guide/getting-started.md)** - Installation, basic usage, first programs
- **[Language Overview](user-guide/language-overview.md)** - Core language features and syntax
- **[REPL Guide](user-guide/repl-guide.md)** - Interactive development environment

#### Advanced Topics
- **[Type System](user-guide/type-system.md)** - Gradual typing from dynamic to dependent types
- **[Effect System](user-guide/effect-system.md)** - Pure functional programming with effect tracking
- **[Module System](user-guide/module-system.md)** - Organizing code with R7RS libraries

### 📚 Tutorials

#### Beginner Tutorials  
- **[Basic Programming](tutorials/basic-programming.md)** - Variables, functions, control flow
- **[Working with Lists](tutorials/list-processing.md)** - List operations and patterns
- **[String Manipulation](tutorials/string-processing.md)** - Text processing techniques

#### Intermediate Tutorials
- **[Functional Programming](tutorials/functional-programming.md)** - Higher-order functions, map, filter, fold
- **[Type Annotations](tutorials/type-annotations.md)** - Adding types to your programs
- **[Error Handling](tutorials/error-handling.md)** - Exception handling and recovery

#### Advanced Tutorials
- **[Effect Management](tutorials/effect-management.md)** - Working with computational effects
- **[Macro Programming](tutorials/macro-programming.md)** - Writing powerful macros
- **[Performance Optimization](tutorials/performance.md)** - Making your code faster

### 📋 API Reference

#### Core Language
- **[Built-in Functions](api/builtins.md)** - Special forms and primitive procedures
- **[Special Forms](api/special-forms.md)** - Core language constructs (if, lambda, define, etc.)
- **[Primitive Operations](api/primitives.md)** - Low-level operations and FFI

#### Standard Library
- **[Arithmetic](api/stdlib/arithmetic.md)** - Number operations and mathematical functions  
- **[Strings](api/stdlib/strings.md)** - String manipulation and processing
- **[Lists](api/stdlib/lists.md)** - List operations and higher-order functions
- **[Vectors](api/stdlib/vectors.md)** - Vector operations and utilities
- **[Characters](api/stdlib/characters.md)** - Character operations and predicates
- **[I/O](api/stdlib/io.md)** - Input/output operations and ports
- **[Control Flow](api/stdlib/control.md)** - Control flow procedures

#### Extensions
- **[SRFI Support](api/srfi/)** - Scheme Request for Implementation extensions
- **[FFI](api/ffi.md)** - Foreign Function Interface with Rust
- **[Type System API](api/types.md)** - Type checking and inference functions
- **[Effect System API](api/effects.md)** - Effect handling and management

### 🔄 Migration Documentation

#### From Other Schemes
- **[From R5RS/R7RS](migration/from-r7rs.md)** - Standard Scheme compatibility
- **[From R6RS](migration/from-r6rs.md)** - Library system differences
- **[From Racket](migration/from-racket.md)** - Racket-specific features
- **[From Guile](migration/from-guile.md)** - GNU Scheme extensions
- **[From Chicken](migration/from-chicken.md)** - Compilation model differences

#### Version Management
- **[Version Upgrades](migration/version-upgrades.md)** - Upgrading between Lambdust versions
- **[Breaking Changes](migration/breaking-changes.md)** - Comprehensive list of breaking changes
- **[Compatibility Matrix](migration/compatibility.md)** - Feature compatibility across versions
- **[Migration Tools](migration/tools.md)** - Automated migration assistance

### 🛠️ Developer Documentation

#### Getting Started
- **[Contributing](developer/contributing.md)** - How to contribute to Lambdust
- **[Building](developer/building.md)** - Development environment setup
- **[Testing](developer/testing.md)** - Running tests and benchmarks

#### Architecture & Design
- **[Architecture](developer/architecture.md)** - High-level system design
- **[Compiler Pipeline](developer/compiler-pipeline.md)** - Lexing, parsing, evaluation
- **[Type System Implementation](developer/type-system-impl.md)** - Type checker internals
- **[Effect System Implementation](developer/effect-system-impl.md)** - Effect tracking internals
- **[Memory Management](developer/memory-management.md)** - Garbage collection and optimization

#### Implementation Details
- **[Standard Library Implementation](developer/stdlib-impl.md)** - How the stdlib is built
- **[FFI Implementation](developer/ffi-impl.md)** - Foreign function interface details
- **[Performance](developer/performance.md)** - Optimization techniques and profiling
- **[Debugging](developer/debugging.md)** - Debugging tools and techniques

### 📝 Examples

#### Basic Examples
- **[Hello World](examples/basic/hello-world.md)** - Your first Lambdust program
- **[Calculator](examples/basic/calculator.md)** - Interactive calculator with REPL
- **[Text Processing](examples/basic/text-processing.md)** - File and string manipulation
- **[Data Structures](examples/basic/data-structures.md)** - Working with lists, vectors, etc.

#### Intermediate Examples
- **[Web Server](examples/intermediate/web-server.md)** - Simple HTTP server
- **[JSON Parser](examples/intermediate/json-parser.md)** - Parsing and generating JSON
- **[Database Interface](examples/intermediate/database.md)** - Database operations
- **[Configuration Manager](examples/intermediate/config-manager.md)** - Application configuration

#### Advanced Examples
- **[Compiler](examples/advanced/compiler.md)** - Simple language compiler
- **[Web Framework](examples/advanced/web-framework.md)** - Full web application framework
- **[Game Engine](examples/advanced/game-engine.md)** - 2D game engine
- **[Scientific Computing](examples/advanced/scientific.md)** - Numerical computations

#### Integration Examples
- **[Rust Integration](examples/integration/rust-ffi.md)** - Using Rust libraries
- **[C Integration](examples/integration/c-ffi.md)** - Interfacing with C code
- **[System Integration](examples/integration/system.md)** - Operating system interfaces
- **[Network Programming](examples/integration/networking.md)** - Network clients and servers

## Documentation by Feature

### 🔤 Language Features

#### Core Syntax
- [S-expressions and syntax](user-guide/language-overview.md#syntax)
- [Special forms](api/builtins.md#special-forms) - quote, lambda, if, define, etc.
- [Literals](api/builtins.md#literals) - numbers, strings, characters, booleans

#### Data Types
- [Numbers](api/stdlib/arithmetic.md) - integers, rationals, reals, complex
- [Strings](api/stdlib/strings.md) - text processing and manipulation
- [Characters](api/stdlib/characters.md) - character operations
- [Lists](api/stdlib/lists.md) - linked lists and operations
- [Vectors](api/stdlib/vectors.md) - arrays and vector operations
- [Records](api/stdlib/records.md) - structured data types

#### Control Flow
- [Conditionals](tutorials/basic-programming.md#conditional-logic) - if, cond, case
- [Loops](tutorials/basic-programming.md#recursion) - recursion and iteration
- [Exceptions](api/stdlib/exceptions.md) - error handling
- [Continuations](api/stdlib/continuations.md) - call/cc and control

#### Functions and Procedures
- [Function definition](tutorials/basic-programming.md#functions) - define, lambda
- [Higher-order functions](tutorials/functional-programming.md) - map, filter, fold
- [Closures](tutorials/functional-programming.md#closures) - lexical scoping
- [Tail recursion](developer/performance.md#tail-calls) - optimization

### 🎯 Type System

#### Gradual Typing
- [Type annotations](tutorials/type-annotations.md) - adding types gradually
- [Type inference](user-guide/type-system.md#inference) - automatic type deduction
- [Contracts](user-guide/type-system.md#contracts) - runtime type checking
- [Dependent types](user-guide/type-system.md#dependent) - advanced type features

#### Type Categories
- [Basic types](api/types.md#basic-types) - Number, String, Boolean, etc.
- [Composite types](api/types.md#composite-types) - List, Vector, Procedure
- [Polymorphic types](api/types.md#polymorphism) - generic programming
- [Effect types](api/effects.md#effect-types) - computational effects

### ⚡ Effect System

#### Effect Categories
- [Pure computations](user-guide/effect-system.md#pure) - no side effects
- [I/O effects](user-guide/effect-system.md#io) - input/output operations
- [State effects](user-guide/effect-system.md#state) - mutable state
- [Control effects](user-guide/effect-system.md#control) - exceptions, continuations

#### Effect Management
- [Effect inference](tutorials/effect-management.md#inference) - automatic effect tracking
- [Effect handlers](tutorials/effect-management.md#handlers) - managing effects
- [Monadic programming](tutorials/effect-management.md#monads) - effect composition

### 📦 Module System

#### Library Definition
- [R7RS libraries](user-guide/module-system.md#r7rs) - standard library format
- [Imports and exports](user-guide/module-system.md#imports) - module interfaces
- [Library paths](user-guide/module-system.md#paths) - module resolution

#### Standard Libraries
- [R7RS base](api/stdlib/) - core standard library
- [SRFI extensions](api/srfi/) - community extensions
- [Lambdust extensions](api/extensions/) - implementation-specific features

### 🔧 Development Tools

#### Interactive Development
- [REPL](user-guide/repl-guide.md) - read-eval-print loop
- [Debugging](developer/debugging.md) - debugging tools and techniques
- [Profiling](developer/performance.md#profiling) - performance analysis

#### Build Tools
- [Compilation](developer/building.md#compilation) - building Lambdust programs
- [Testing](developer/testing.md) - unit and integration testing
- [Benchmarking](developer/performance.md#benchmarking) - performance measurement

## Standards Compliance

### R7RS Compliance
- **[R7RS Base Library](api/stdlib/r7rs-base.md)** - Complete implementation status
- **[R7RS Compliance Report](compliance/r7rs-report.md)** - Detailed compliance analysis
- **[R7RS Test Suite](tests/r7rs-tests.md)** - Standard test suite results

### SRFI Support
- **[Implemented SRFIs](api/srfi/implemented.md)** - Currently supported SRFIs
- **[SRFI Roadmap](api/srfi/roadmap.md)** - Planned SRFI implementations
- **[SRFI Compatibility](api/srfi/compatibility.md)** - Cross-implementation compatibility

## Performance Documentation

### Optimization Guides
- **[Performance Tips](developer/performance.md#tips)** - Writing efficient code
- **[Memory Management](developer/memory-management.md)** - Understanding GC behavior
- **[Profiling Guide](developer/performance.md#profiling)** - Finding bottlenecks

### Benchmarks
- **[Standard Benchmarks](benchmarks/standard.md)** - Common performance tests
- **[Comparison Benchmarks](benchmarks/comparison.md)** - vs other Scheme implementations
- **[Regression Tests](benchmarks/regression.md)** - Performance regression tracking

## Community Resources

### Getting Help
- **[FAQ](community/faq.md)** - Frequently asked questions
- **[Troubleshooting](community/troubleshooting.md)** - Common issues and solutions
- **[GitHub Discussions](https://github.com/username/lambdust/discussions)** - Community discussion
- **[GitHub Issues](https://github.com/username/lambdust/issues)** - Bug reports and feature requests

### Contributing
- **[Code of Conduct](community/code-of-conduct.md)** - Community guidelines
- **[Contributor Guide](developer/contributing.md)** - How to contribute
- **[Documentation Guide](developer/documentation.md)** - Writing documentation
- **[Release Process](developer/release-process.md)** - How releases are made

## Reference Materials

### Language References
- **[Syntax Reference](reference/syntax.md)** - Complete syntax specification
- **[Semantics Reference](reference/semantics.md)** - Evaluation rules and semantics
- **[Standard Library Reference](reference/stdlib.md)** - Complete procedure listing
- **[Error Reference](reference/errors.md)** - Error types and messages

### Implementation References
- **[Implementation Notes](reference/implementation.md)** - Implementation-specific details
- **[Extensions Reference](reference/extensions.md)** - Lambdust-specific extensions
- **[Compatibility Reference](reference/compatibility.md)** - Cross-platform compatibility

---

## Document Status

This index covers the comprehensive documentation system for Lambdust. Each document provides:

- ✅ **Complete coverage** of its topic area
- ✅ **Practical examples** and code samples  
- ✅ **Cross-references** to related topics
- ✅ **Up-to-date information** reflecting current implementation
- ✅ **Multiple skill levels** from beginner to expert

The documentation is designed to serve multiple audiences:

- **New users** learning functional programming
- **Experienced Scheme programmers** exploring Lambdust features  
- **Migrating developers** coming from other languages
- **Contributors** wanting to improve Lambdust
- **Researchers** interested in language design

All documentation follows consistent formatting, includes working code examples, and provides clear navigation paths for different learning objectives.