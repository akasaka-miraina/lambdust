# Lambdust (λust)

A Scheme dialect that combines the simplicity and elegance of Scheme with modern type theory and functional programming concepts.

## Features

- **R7RS-compatible**: Existing Scheme code runs without modification
- **Gradually typed**: From dynamic to dependent types
- **Purely functional**: With transparent handling of effects through monads
- **Efficient**: JIT compilation and profile-guided optimization
- **Rust FFI**: Seamless interoperability with Rust code

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
cargo run examples/basic.ldust
```

### Evaluating an expression

```bash
cargo run -- --eval "(+ 1 2 3)"
```

## Language Overview

### Core Language

Lambdust has exactly 10 special forms:

1. `quote` - Returns data without evaluation
2. `lambda` - Creates procedures
3. `if` - Conditional expressions
4. `define` - Variable and function definitions
5. `set!` - Assignment (transformed to state monad)
6. `define-syntax` - Macro definitions
7. `call-with-current-continuation` - Continuation capture
8. `primitive` - Built-in operations and FFI calls
9. `::` - Type annotations
10. Keyword literals (`#:key`)

### Type System

Lambdust supports four levels of typing:

1. **Dynamic** (default) - R7RS compatible dynamic typing
2. **Contracts** - Runtime type checking with contracts
3. **Static** - Compile-time type checking with inference
4. **Dependent** - Experimental dependent types

```scheme
;; Dynamic typing (default)
(define (add x y) (+ x y))

;; Static typing with annotations
(define (typed-add x y)
  #:type (-> Number Number Number)
  (+ x y))

;; Contract-based typing
(define (safe-divide x y)
  #:contract (-> Number (and Number (not zero?)) Number)
  (/ x y))
```

### Effect System

Side effects are tracked and handled through monads:

```scheme
;; Pure function
(define (square x)
  #:pure #t
  (* x x))

;; IO effects automatically tracked
(define (greet name)
  (display "Hello, ")  ; Lifted to IO monad
  (display name)
  (newline))

;; State effects
(define (counter)
  (let ([n 0])
    (lambda ()
      (set! n (+ n 1))  ; Creates new environment generation
      n)))
```

## Project Structure

```
lambdust/
├── Cargo.toml                 # Project configuration and dependencies
├── src/
│   ├── lib.rs                 # Main library entry point
│   ├── main.rs                # CLI application
│   ├── lexer/                 # Tokenization
│   │   ├── mod.rs
│   │   └── token.rs
│   ├── parser/                # AST generation
│   │   ├── mod.rs
│   │   ├── expression.rs
│   │   ├── literals.rs
│   │   └── special_forms.rs
│   ├── ast/                   # Abstract syntax tree
│   │   ├── mod.rs
│   │   ├── literal.rs
│   │   └── visitor.rs
│   ├── macro_system/          # Hygienic macros
│   ├── types/                 # Type system
│   ├── eval/                  # Evaluation engine
│   │   ├── mod.rs
│   │   ├── value.rs
│   │   ├── environment.rs
│   │   └── evaluator.rs
│   ├── effects/               # Effect system
│   ├── runtime/               # Runtime system
│   ├── stdlib/                # Standard library
│   ├── ffi/                   # Rust FFI support
│   ├── diagnostics/           # Error handling
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   └── span.rs
│   └── utils/                 # Utilities
│       ├── mod.rs
│       └── symbol.rs
├── tests/                     # Integration tests
├── examples/                  # Example programs
├── benches/                   # Benchmarks
└── docs/                      # Documentation
```

## Architecture

The implementation follows a traditional compiler pipeline:

1. **Lexer** - Tokenizes source code using the `logos` crate
2. **Parser** - Builds AST using recursive descent parsing
3. **Macro Expansion** - Expands user-defined and built-in macros
4. **Type Checking** - Optional static type analysis with Hindley-Milner inference
5. **Effect Analysis** - Tracks and transforms side effects
6. **Evaluation** - Interprets or compiles the code with tail call optimization

## Dependencies

- **Parsing**: `nom`, `logos` for lexical analysis
- **Type System**: `petgraph`, `ena` for type graphs and unification
- **Runtime**: `gc`, `im` for garbage collection and immutable data structures
- **Diagnostics**: `miette`, `ariadne` for beautiful error reporting
- **CLI**: `clap`, `rustyline` for command-line interface and REPL

## Testing Strategy

The project includes comprehensive testing at multiple levels:

### Unit Tests
- Each module has dedicated unit tests
- Tests cover lexing, parsing, type checking, and evaluation
- Property-based testing with `proptest` for edge cases

### Integration Tests
- End-to-end testing of complete programs
- R7RS compatibility tests
- Performance and memory usage tests

### Snapshot Testing
- AST structure verification with `insta`
- Error message consistency testing

## Contributing

1. Ensure all tests pass: `cargo test`
2. Check formatting: `cargo fmt`
3. Run clippy: `cargo clippy`
4. Add tests for new features
5. Update documentation as needed

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## References

- [R7RS Scheme Standard](https://small.r7rs.org/)
- [Hindley-Milner Type System](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [Gradual Typing](https://en.wikipedia.org/wiki/Gradual_typing)
- [Effect Systems](https://en.wikipedia.org/wiki/Effect_system)