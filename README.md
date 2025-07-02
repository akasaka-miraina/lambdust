# Lambdust (λust) - Rust Scheme Interpreter

[![Crates.io](https://img.shields.io/crates/v/lambdust)](https://crates.io/crates/lambdust)
[![Documentation](https://docs.rs/lambdust/badge.svg)](https://docs.rs/lambdust)
[![CI](https://github.com/akasaka-miraina/lambdust/workflows/Continuous%20Integration/badge.svg)](https://github.com/akasaka-miraina/lambdust/actions)
[![Coverage](https://codecov.io/gh/akasaka-miraina/lambdust/branch/main/graph/badge.svg)](https://codecov.io/gh/akasaka-miraina/lambdust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)

Lambdust is a complete R7RS Scheme interpreter implemented in Rust, designed for embedding in applications as a macro and scripting system. The name combines "lambda" (λ) with "Rust," reflecting Scheme's functional nature and the ability to add expressive power to existing applications.

## Features

- **R7RS Compliance**: Implements the R7RS Small language specification
- **Embedded Design**: Designed for integration into Rust applications
- **Macro System**: Full support for Scheme's powerful macro system
- **Type Safety**: Leverages Rust's type system for memory safety
- **Bridge API**: Seamless interoperability between Rust and Scheme
- **Performance**: Optimized for speed with tail-call optimization

## Quick Start

Add Lambdust to your `Cargo.toml`:

```toml
[dependencies]
lambdust = "0.1.0"
```

### Basic Usage

```rust
use lambdust::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("(+ 1 2 3)").unwrap();
    println!("Result: {}", result); // Prints: Result: 6
}
```

### Advanced Integration with Bridge API

```rust
use lambdust::{LambdustBridge, FromScheme, ToScheme, Value};

fn main() {
    let mut bridge = LambdustBridge::new();

    // Register external function
    bridge.register_function("square", Some(1), |args| {
        let n = f64::from_scheme(&args[0])?;
        (n * n).to_scheme()
    });

    // Define variables
    bridge.define("pi", Value::from(3.14159));

    // Execute Scheme code
    let result = bridge.eval("(* pi 2)").unwrap();
    println!("2π = {}", result);
}
```

## Architecture

The interpreter consists of several key components:

- **Lexer**: Tokenizes Scheme source code
- **Parser**: Builds Abstract Syntax Trees (AST) from tokens
- **Evaluator**: Executes Scheme expressions with proper semantics
- **Environment**: Manages variable bindings and lexical scoping
- **Macro System**: Handles macro expansion and transformation
- **Bridge**: Provides interoperability with external Rust code

## Supported Scheme Features

- All basic data types (numbers, strings, symbols, lists, etc.)
- Special forms (`define`, `lambda`, `if`, `cond`, `let`, etc.)
- First-class procedures and closures
- Tail-call optimization
- Macro system with built-in macros (`let`, `cond`, `case`, etc.)
- Proper lexical scoping
- Built-in procedures for list manipulation, arithmetic, etc.

### Built-in Procedures

#### Arithmetic Operations
- `+`, `-`, `*`, `/`: Basic arithmetic
- `=`, `<`, `>`, `<=`, `>=`: Numeric comparisons
- `abs`, `floor`, `ceiling`, `round`, `sqrt`: Math functions

#### List Operations
- `car`, `cdr`, `cons`: Basic list operations
- `list`, `length`, `append`, `reverse`: List construction and manipulation
- `map`, `filter`, `fold-left`, `fold-right`: Higher-order functions

#### Type Predicates
- `number?`, `string?`, `symbol?`, `list?`, `pair?`, `null?`
- `boolean?`, `procedure?`

#### String Operations
- `string-length`, `string-append`, `substring`
- `string=?`, `string<?`, `string>?`

#### I/O Operations
- `display`, `newline`, `read`

## Bridge API

The Bridge API enables seamless integration between Rust and Scheme code:

### Type Conversion

Implement `ToScheme` and `FromScheme` traits for automatic type conversion:

```rust
use lambdust::{ToScheme, FromScheme, Value, Result};

// Custom type conversion
impl ToScheme for MyStruct {
    fn to_scheme(&self) -> Result<Value> {
        // Convert to Scheme value
        Ok(Value::from(self.value))
    }
}

impl FromScheme for MyStruct {
    fn from_scheme(value: &Value) -> Result<Self> {
        // Convert from Scheme value
        let val = i64::from_scheme(value)?;
        Ok(MyStruct { value: val })
    }
}
```

### External Functions

Register Rust functions to be callable from Scheme:

```rust
bridge.register_function("my-func", Some(2), |args| {
    let a = i64::from_scheme(&args[0])?;
    let b = i64::from_scheme(&args[1])?;
    (a + b).to_scheme()
});
```

### Object Management

Register and manipulate Rust objects from Scheme:

```rust
#[derive(Debug)]
struct Counter { value: i32 }

let counter = Counter { value: 0 };
let counter_id = bridge.register_object(counter, "Counter");
bridge.define("my-counter", Value::from(counter_id));
```

## Examples

See the `examples/` directory for complete examples:

- `bridge_example.rs`: Demonstrates Bridge API usage
- Basic arithmetic and list operations
- Function definitions and macro usage
- External object integration

## Documentation

Complete API documentation is available at [docs.rs/lambdust](https://docs.rs/lambdust).

## Building from Source

```bash
git clone https://github.com/akasaka-miraina/lambdust.git
cd lambdust
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage report
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --open
```

### Test Coverage

The project maintains high test coverage across all modules:

- **Unit Tests**: 94 test cases covering core functionality
- **Documentation Tests**: 13 doctests ensuring example code works
- **Integration Tests**: End-to-end testing of the complete interpreter
- **Coverage Reports**: Automatically generated and uploaded to Codecov

Current test coverage (Overall: **52.94%**):
- ✅ Parser (83.26%)
- ✅ Environment (84.86%)  
- ✅ Lexer (72.14%)
- ✅ Interpreter (71.43%)
- ✅ Marshal (71.92%)
- ✅ Evaluator (65.75%)
- ✅ Macro System (62.15%)
- ⚠️ Built-in Functions (35% avg - needs improvement)
- ⚠️ Error Handling (21.32% - needs improvement)

### Generating Documentation

```bash
cargo doc --open --no-deps
```

## Development Plan

The development follows a systematic approach as documented in `CLAUDE.md`:

1. ✅ **Phase 1**: Core interpreter (lexer, parser, evaluator)
2. ✅ **Phase 2**: Built-in functions and macro system
3. ✅ **Phase 3**: Bridge API and external integration
4. 🔄 **Phase 4**: Advanced features and optimization
5. 🔄 **Phase 5**: Documentation and ecosystem

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built with R7RS Scheme specification compliance in mind
- Inspired by the elegance of Scheme and the power of Rust
- Special thanks to the Rust and Scheme communities

---

**Lambdust** - Adding λ-powered expressiveness to Rust applications.