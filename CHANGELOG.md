# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Tail call optimization (planned)

## [0.1.0] - 2025-01-01

### Added
- Initial release of Lambdust Scheme interpreter
- R7RS Small language specification compliance
- Complete lexer and parser implementation
- AST-based evaluator with proper Scheme semantics
- Environment management with lexical scoping
- Comprehensive macro system with built-in macros
- Bridge API for external application integration
- Type conversion traits (`ToScheme`, `FromScheme`)
- External function registration capabilities
- Object management system
- Built-in procedures:
  - Arithmetic operations (`+`, `-`, `*`, `/`, etc.)
  - List operations (`car`, `cdr`, `cons`, `list`, etc.)
  - Type predicates (`number?`, `string?`, etc.)
  - String operations (`string-length`, `string-append`, etc.)
  - Higher-order functions (`map`, `filter`, `fold-left`, `fold-right`)
  - I/O operations (`display`, `newline`, `read`)
- Special forms:
  - `define`, `lambda`, `if`, `begin`, `set!`
  - `quote`, `quasiquote`, `unquote`, `unquote-splicing`
- Built-in macros:
  - `let`, `let*`, `letrec`
  - `cond`, `case`, `when`, `unless`
- Comprehensive test suite (50+ unit tests)
- Complete Rustdoc documentation with examples
- Bridge API examples and documentation

### Technical Features
- Memory-safe implementation using Rust's type system
- Proper error handling with detailed error messages
- Support for multiple numeric types (integers, rationals, reals, complex)
- Thread-safe object management
- Modular architecture for easy extension

[Unreleased]: https://github.com/akasaka-miraina/lambdust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/akasaka-miraina/lambdust/releases/tag/v0.1.0