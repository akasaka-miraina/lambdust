# Contributing to Lambdust

Thank you for your interest in contributing to Lambdust! This guide will help you get started with contributing code, documentation, tests, and other improvements.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Contribution Guidelines](#contribution-guidelines)
5. [Code Style and Standards](#code-style-and-standards)
6. [Testing](#testing)
7. [Documentation](#documentation)
8. [Submitting Changes](#submitting-changes)
9. [Review Process](#review-process)
10. [Community Guidelines](#community-guidelines)

## Getting Started

### Ways to Contribute

- **Bug Reports**: Help identify and document issues
- **Feature Requests**: Suggest new features and improvements
- **Code Contributions**: Implement bug fixes, features, and optimizations
- **Documentation**: Improve guides, API docs, and examples
- **Testing**: Add test cases and improve test coverage
- **Performance**: Profile and optimize critical code paths
- **Standard Library**: Implement R7RS procedures and SRFI extensions

### Before You Start

1. **Check existing issues** on [GitHub Issues](https://github.com/username/lambdust/issues)
2. **Search discussions** on [GitHub Discussions](https://github.com/username/lambdust/discussions)
3. **Read the architecture overview** in [architecture.md](architecture.md)
4. **Join our community** to discuss your ideas

## Development Setup

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Text Editor**: VS Code, Emacs, Vim, or your preferred editor

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/username/lambdust.git
cd lambdust

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Development Tools

```bash
# Install useful development tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-expand   # Expand macros
cargo install cargo-flamegraph  # Performance profiling

# Format code
cargo fmt

# Run linter
cargo clippy

# Check documentation
cargo doc --open
```

### IDE Setup

#### VS Code
Recommended extensions:
- `rust-analyzer`: Rust language server
- `CodeLLDB`: Debugging support
- `Scheme`: Scheme syntax highlighting
- `Better TOML`: TOML file support

#### Emacs
```elisp
;; Add to your .emacs configuration
(use-package rust-mode)
(use-package lsp-mode)
(use-package scheme-mode)
```

## Project Structure

Understanding the codebase organization:

```
lambdust/
├── src/                    # Main source code
│   ├── lib.rs             # Library entry point
│   ├── main.rs            # CLI application
│   ├── lexer/             # Tokenization
│   ├── parser/            # AST generation
│   ├── ast/               # Abstract syntax tree
│   ├── macro_system/      # Hygienic macros
│   ├── types/             # Type system
│   ├── effects/           # Effect system
│   ├── eval/              # Evaluation engine
│   ├── runtime/           # Runtime system
│   ├── stdlib/            # Standard library
│   ├── ffi/               # Rust FFI support
│   ├── diagnostics/       # Error handling
│   ├── repl/              # Interactive REPL
│   └── utils/             # Utilities
├── stdlib/                # Scheme standard library
├── tests/                 # Integration tests
├── benches/               # Benchmarks
├── examples/              # Example programs
└── docs/                  # Documentation
```

### Key Components

- **Lexer**: Converts source text into tokens
- **Parser**: Builds AST from tokens using recursive descent
- **Macro System**: Hygienic macro expansion with syntax-rules
- **Type System**: Gradual typing with Hindley-Milner inference
- **Effect System**: Tracks and manages side effects
- **Evaluator**: Interprets AST with tail-call optimization
- **Runtime**: Memory management and primitive operations
- **Standard Library**: R7RS-compliant procedures

## Contribution Guidelines

### Issue Types

**Bug Reports**
- Use the bug report template
- Include minimal reproduction case
- Specify Lambdust version and platform
- Provide error messages and stack traces

**Feature Requests**
- Use the feature request template
- Explain the use case and motivation
- Consider backward compatibility
- Suggest implementation approach if possible

**Performance Issues**
- Include benchmarks or profiling data
- Compare with other Scheme implementations when relevant
- Consider memory usage and time complexity

### Pull Request Guidelines

1. **Fork the repository** and create a feature branch
2. **Write clear commit messages** following conventional commits
3. **Add tests** for new functionality
4. **Update documentation** for user-facing changes
5. **Run the full test suite** before submitting
6. **Keep PRs focused** - one feature or fix per PR

### Commit Message Format

Use [Conventional Commits](https://conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Examples:
```
feat(stdlib): add SRFI-1 list processing procedures
fix(parser): handle nested quotes correctly
docs(api): update arithmetic module documentation
perf(eval): optimize tail-call elimination
test(lexer): add edge cases for string parsing
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

## Code Style and Standards

### Rust Code Style

Follow standard Rust conventions:

```rust
// Use rustfmt for formatting
cargo fmt

// Follow Rust naming conventions
struct MyStruct {
    field_name: String,
}

impl MyStruct {
    pub fn new() -> Self {
        Self {
            field_name: String::new(),
        }
    }
    
    pub fn method_name(&self) -> &str {
        &self.field_name
    }
}

// Document public APIs
/// Evaluates a Scheme expression in the given environment.
/// 
/// # Arguments
/// 
/// * `expr` - The expression to evaluate
/// * `env` - The environment for variable lookups
/// 
/// # Returns
/// 
/// The result of evaluation or an error
pub fn eval(expr: &Value, env: &Environment) -> Result<Value, Error> {
    // Implementation
}
```

### Scheme Code Style

For Scheme code in the standard library:

```scheme
;; Use consistent indentation (2 spaces)
(define (fibonacci n)
  "Calculate the nth Fibonacci number."
  (cond
    [(<= n 0) 0]
    [(= n 1) 1]
    [else (+ (fibonacci (- n 1))
             (fibonacci (- n 2)))]))

;; Use descriptive names
(define (list-has-duplicates? lst)
  "Check if list contains duplicate elements."
  ;; Implementation
  )

;; Document complex functions
(define (parse-s-expression input)
  "Parse an S-expression from input string.
   
   Returns either the parsed expression or #f if parsing fails.
   Handles nested lists, strings, numbers, and symbols."
  ;; Implementation
  )
```

### Error Handling

```rust
// Use Result types for fallible operations
pub fn parse_number(s: &str) -> Result<Value, ParseError> {
    // Implementation
}

// Provide detailed error messages
return Err(ParseError::InvalidNumber {
    input: s.to_string(),
    position: 0,
    expected: "valid number literal",
});

// Use the miette crate for user-facing errors
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Diagnostic, Error)]
#[error("Parse error")]
pub struct ParseError {
    #[source_code]
    src: String,
    
    #[label("Invalid syntax here")]
    err_span: SourceSpan,
}
```

## Testing

### Test Types

**Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = eval_string("(+ 1 2 3)").unwrap();
        assert_eq!(result, Value::Integer(6));
    }
    
    #[test]
    fn test_error_handling() {
        let result = eval_string("(/ 1 0)");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::DivisionByZero));
    }
}
```

**Integration Tests**
```rust
// tests/integration_test.rs
use lambdust::{Lambdust, Environment};

#[test]
fn test_factorial_program() {
    let mut lambdust = Lambdust::new();
    let result = lambdust.eval_file("examples/factorial.ldust").unwrap();
    // Test assertions
}
```

**Property-Based Tests**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_number_parsing_roundtrip(n in any::<i64>()) {
        let s = n.to_string();
        let parsed = parse_number(&s).unwrap();
        let back_to_string = parsed.to_string();
        prop_assert_eq!(s, back_to_string);
    }
}
```

**Scheme Test Files**
```scheme
;; tests/test_basic.ldust
(import (scheme base)
        (scheme write))

;; Test basic arithmetic
(assert (= (+ 1 2 3) 6))
(assert (= (* 2 3 4) 24))

;; Test list operations
(assert (equal? (reverse '(1 2 3)) '(3 2 1)))
(assert (= (length '(a b c d)) 4))

(display "All tests passed!")
(newline)
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test lexer

# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Run Scheme test files
./target/debug/lambdust tests/test_basic.ldust
```

### Coverage

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html --output-dir coverage/

# View coverage
open coverage/tarpaulin-report.html
```

## Documentation

### API Documentation

```rust
/// Evaluates a Scheme expression.
/// 
/// This function takes an expression and environment and returns
/// the result of evaluation. It handles all Scheme value types
/// and implements proper tail-call optimization.
/// 
/// # Arguments
/// 
/// * `expr` - The expression to evaluate
/// * `env` - The evaluation environment
/// 
/// # Returns
/// 
/// * `Ok(Value)` - The result of evaluation
/// * `Err(Error)` - An evaluation error
/// 
/// # Examples
/// 
/// ```
/// use lambdust::{Value, Environment, eval};
/// 
/// let env = Environment::new();
/// let expr = Value::List(vec![
///     Value::Symbol("+".into()),
///     Value::Integer(1),
///     Value::Integer(2),
/// ]);
/// 
/// let result = eval(&expr, &env).unwrap();
/// assert_eq!(result, Value::Integer(3));
/// ```
/// 
/// # Panics
/// 
/// This function does not panic. All errors are returned as `Result`.
/// 
/// # Safety
/// 
/// This function is safe to call from multiple threads concurrently.
pub fn eval(expr: &Value, env: &Environment) -> Result<Value, Error> {
    // Implementation
}
```

### User Documentation

Write clear, example-rich documentation:

```markdown
# List Processing

The `map` function applies a procedure to each element of a list.

## Syntax

```scheme
(map proc list1 list2 ...)
```

## Parameters

- `proc`: A procedure that accepts as many arguments as there are lists
- `list1`, `list2`, ...: One or more lists of the same length

## Returns

A new list containing the results of applying `proc` to corresponding elements.

## Examples

```scheme
;; Square each number
(map (lambda (x) (* x x)) '(1 2 3 4))
;; => (1 4 9 16)

;; Add corresponding elements
(map + '(1 2 3) '(4 5 6))
;; => (5 7 9)
```

## See Also

- [`for-each`](for-each.md) - Similar but doesn't collect results
- [`filter`](filter.md) - Select elements matching a predicate
```

## Submitting Changes

### Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes**
   - Write clean, well-documented code
   - Add appropriate tests
   - Update documentation

3. **Test thoroughly**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit with clear messages**
   ```bash
   git commit -m "feat(stdlib): implement SRFI-1 list procedures"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/my-new-feature
   ```

6. **Fill out PR template**
   - Describe what you changed and why
   - Link to related issues
   - Add test results
   - Update changelog if needed

### PR Template

```markdown
## Description

Brief description of changes and motivation.

## Type of Change

- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to not work as expected)
- [ ] Documentation update

## Testing

- [ ] Added/updated unit tests
- [ ] Added/updated integration tests
- [ ] All tests pass
- [ ] Benchmarks run (if performance-related)

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings from clippy
- [ ] Formatted with rustfmt
```

## Review Process

### What Reviewers Look For

1. **Correctness**: Does the code work as intended?
2. **Test Coverage**: Are there adequate tests?
3. **Documentation**: Is it properly documented?
4. **Style**: Does it follow project conventions?
5. **Performance**: Are there performance implications?
6. **Compatibility**: Does it maintain backward compatibility?

### Addressing Review Comments

- **Be responsive**: Reply to comments promptly
- **Ask questions**: If feedback is unclear, ask for clarification
- **Make changes**: Address valid concerns with code changes
- **Explain decisions**: Justify design choices when needed
- **Be patient**: Reviews take time and help improve code quality

### Review Timeline

- **Initial response**: Within 48 hours
- **Full review**: Within 1 week for most PRs
- **Large changes**: May take longer, consider breaking into smaller PRs

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment:

- **Be respectful**: Treat all contributors with respect
- **Be collaborative**: Work together to solve problems
- **Be patient**: Help newcomers learn and contribute
- **Be constructive**: Provide helpful feedback and suggestions

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Pull Requests**: Code review and technical discussion

### Getting Help

If you need help:

1. **Check existing documentation** first
2. **Search closed issues** for similar problems
3. **Ask on GitHub Discussions** for general questions
4. **Create an issue** for specific problems
5. **Join our chat** (link coming soon) for real-time help

## Recognition

Contributors are recognized in several ways:

- **Contributors file**: All contributors are listed
- **Release notes**: Significant contributions are highlighted
- **Hall of fame**: Outstanding contributors are featured
- **Commit attribution**: Your contributions are permanently recorded

## Special Areas Needing Help

### High-Priority Areas

- **R7RS Compliance**: Implementing missing R7RS procedures
- **SRFI Support**: Adding popular SRFI extensions
- **Performance**: Optimizing critical code paths
- **Documentation**: Improving user guides and API docs
- **Testing**: Increasing test coverage and edge cases

### Beginner-Friendly Tasks

Look for issues labeled `good first issue`:

- Documentation improvements
- Adding test cases  
- Implementing simple standard library functions
- Fixing typos and small bugs
- Adding examples

### Advanced Projects

For experienced contributors:

- Implementing the bytecode compiler
- Adding JIT compilation support
- Improving the type inference engine
- Implementing advanced SRFI extensions
- Performance profiling and optimization

Thank you for contributing to Lambdust! Your efforts help make functional programming more accessible and enjoyable for everyone. 🎉