# Contributing to Lambdust

Thank you for your interest in contributing to Lambdust! This document provides comprehensive guidelines for contributing to our high-quality Scheme interpreter project.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

1. **Read the Documentation**: Familiarize yourself with the project through:
   - [README.md](README.md) - Project overview and features
   - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and design
   - [BUILDING.md](BUILDING.md) - Development setup and workflow

2. **Development Environment**: Set up your development environment following the [Building Guide](BUILDING.md)

3. **Understand the Quality Standards**: Lambdust maintains exceptional quality through strict standards established during our successful structural refactoring of 226+ structures with 100% success rate.

## Quality Standards

### Mandatory Quality Requirements

All contributions must meet these non-negotiable standards:

#### Compilation Standards
```bash
# Development Phase (Required for ALL changes)
cargo check --lib                    # MUST return 0 errors

# Commit Phase (Required before ANY commits)  
cargo clippy --lib                   # MUST return 0 errors AND 0 warnings
cargo test                           # MUST pass all tests
cargo fmt                            # MUST be applied
```

#### Incremental Development Protocol

**CRITICAL**: Follow the proven incremental development methodology that achieved 100% success across our major refactoring:

1. **One Focused Change Per Step**: Make exactly one logical change at a time
2. **Immediate Verification**: Run `cargo check --lib` after each change
3. **Zero Error Tolerance**: Fix any errors immediately before proceeding  
4. **Quality Gate Enforcement**: Ensure zero warnings before commits

### Code Organization Standards

#### One Structure Per File Rule
All code must follow the "one primary structure per file" principle:

```rust
// ✅ CORRECT: single_structure.rs
pub struct MyStructure {
    field: String,
}

impl MyStructure {
    // Implementation here
}

// ❌ WRONG: multiple_structures.rs  
pub struct FirstStructure { ... }
pub struct SecondStructure { ... }  // Multiple structures not allowed
```

#### Module Organization
- Each module should have a single, clear responsibility
- Use `mod.rs` for re-exports and module coordination only
- Keep structure definitions in dedicated files
- Maintain clean dependency boundaries

### Documentation Standards

All public interfaces require comprehensive documentation:

```rust
/// Comprehensive description of the structure's purpose and usage.
///
/// # Examples
/// 
/// ```rust
/// let example = MyStructure::new("example");
/// assert_eq!(example.field(), "example");
/// ```
pub struct MyStructure {
    /// Field documentation explaining purpose and constraints
    field: String,
}

impl MyStructure {
    /// Creates a new instance with the specified value.
    ///
    /// # Arguments
    /// * `value` - The initial value for the field
    ///
    /// # Returns
    /// A new `MyStructure` instance
    pub fn new(value: impl Into<String>) -> Self {
        Self { field: value.into() }
    }
}
```

## Contribution Workflow

### 1. Issue Discussion

Before starting work:
- Check existing issues for similar work
- Create an issue for new features or significant changes
- Discuss the approach and get feedback from maintainers

### 2. Development Process

#### Branch Management
```bash
# Create feature branch
git checkout -b feature/descriptive-name

# Or for bug fixes
git checkout -b fix/bug-description
```

#### Incremental Development
```bash
# Make one focused change
# Run immediate verification
cargo check --lib    # Must be 0 errors

# Continue with next change only if check passes
# Repeat for each logical change
```

#### Before Committing
```bash
# Comprehensive quality check
cargo clippy --lib    # Must be 0 errors, 0 warnings  
cargo test            # All tests must pass
cargo fmt             # Apply formatting
```

### 3. Commit Guidelines

#### Commit Message Format
```
type(scope): brief description

Longer explanation if needed, focusing on what and why rather than how.

- List any breaking changes
- Reference related issues
```

#### Commit Types
- `feat`: New feature
- `fix`: Bug fix  
- `docs`: Documentation changes
- `style`: Formatting, no logic changes
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `perf`: Performance improvements

#### Example Commits
```bash
feat(eval): add monadic computation evaluation

Implements monadic evaluation architecture with clean separation of
domain, application, and infrastructure layers.

- Adds MonadicComputation trait for composable computations
- Implements evaluation orchestrator for coordinating computations
- Provides effect interpreter integration

Closes #123

fix(concurrency): resolve thread safety in effect coordination

Fixes data race condition in concurrent effect handling by properly
synchronizing access to shared effect state.

- Updates EffectCoordinator to use Arc<RwLock<T>> for thread safety
- Adds comprehensive test coverage for concurrent scenarios
- Maintains backward compatibility with existing APIs

Fixes #456
```

### 4. Pull Request Process

#### PR Title and Description
```markdown
# Pull Request Title
Brief, descriptive title following commit message conventions

## Summary
Clear description of what this PR accomplishes

## Changes
- List of specific changes made
- Any new features or capabilities added
- Bug fixes or improvements implemented

## Testing
- How the changes were tested
- Any new tests added
- Performance impact analysis if relevant

## Documentation
- Documentation updates included
- API changes documented
- Examples updated if needed

## Quality Checklist
- [ ] `cargo check --lib` returns 0 errors
- [ ] `cargo clippy --lib` returns 0 errors and 0 warnings
- [ ] `cargo test` passes all tests
- [ ] Code follows one-structure-per-file principle
- [ ] All public APIs documented
- [ ] Architecture principles maintained
```

#### Review Process
1. **Automated Checks**: All CI checks must pass
2. **Code Review**: At least one maintainer review required
3. **Quality Verification**: Manual verification of quality standards
4. **Integration Testing**: Comprehensive testing in integration environment

## Types of Contributions

### Code Contributions

#### Language Features
- R7RS-large compliance improvements
- Type system enhancements  
- Effect system extensions
- Macro system improvements

#### Performance Improvements
- Optimization implementations
- Benchmarking enhancements
- Memory management improvements
- Concurrent evaluation optimizations

#### Infrastructure
- Build system improvements
- Testing infrastructure
- Development tooling
- CI/CD pipeline enhancements

### Documentation Contributions

#### API Documentation
- Function and method documentation
- Usage examples and tutorials
- Architecture explanations
- Performance guides

#### User Documentation
- Getting started guides
- Feature explanations
- Best practices documentation
- Migration guides

### Testing Contributions

#### Test Coverage
- Unit tests for new functionality
- Integration tests for system interactions
- Property-based tests for critical algorithms
- Performance regression tests

#### Quality Assurance
- Code review and feedback
- Bug reporting and verification
- Performance analysis and optimization
- Security analysis and improvements

## Architecture Guidelines

### Design Principles

1. **Clean Architecture**: Maintain clear separation between domain, application, and infrastructure layers
2. **Single Responsibility**: Each module should have one clear purpose
3. **Dependency Inversion**: Depend on abstractions, not concrete implementations
4. **Composition over Inheritance**: Favor composition and traits over complex inheritance hierarchies

### Module Design

```rust
// ✅ GOOD: Clear, focused module
pub mod evaluation_orchestrator {
    /// Coordinates monadic evaluation workflow
    pub struct EvaluationOrchestrator {
        // Single responsibility: orchestration
    }
}

// ❌ AVOID: Mixed responsibilities
pub mod evaluation_system {
    pub struct Evaluator { ... }         // Evaluation logic
    pub struct Parser { ... }            // Parsing logic - should be separate
    pub struct TypeChecker { ... }       // Type checking - should be separate
}
```

### Error Handling

Use the established diagnostic error system:

```rust
use crate::diagnostics::{Result, Error};

pub fn example_function() -> Result<Value> {
    // Use error helpers for consistent error handling
    match computation {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::runtime_error("Clear error message", span).boxed()),
    }
}
```

## Testing Guidelines

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_functionality() {
        // Arrange
        let input = create_test_input();
        
        // Act  
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_error_conditions() {
        // Test error scenarios
        let result = function_with_invalid_input();
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// tests/integration_example.rs
use lambdust::prelude::*;

#[test]
fn test_end_to_end_evaluation() {
    let interpreter = Interpreter::new();
    let result = interpreter.eval("(+ 1 2 3)").unwrap();
    assert_eq!(result, Value::number(6.0));
}
```

## Performance Guidelines

### Benchmarking

Use the integrated benchmarking system for performance-related changes:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_evaluation(c: &mut Criterion) {
    c.bench_function("fibonacci_30", |b| {
        b.iter(|| fibonacci(30))
    });
}

criterion_group!(benches, benchmark_evaluation);
criterion_main!(benches);
```

### Memory Management

- Follow Rust ownership principles
- Use `Arc<RwLock<T>>` for shared mutable state
- Prefer `Rc<RefCell<T>>` for single-threaded scenarios
- Consider memory pools for frequent allocations

## Release Process

### Version Management

We follow Semantic Versioning (SemVer):
- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible

### Release Checklist

1. **Quality Verification**
   - [ ] All tests passing
   - [ ] Zero clippy warnings
   - [ ] Comprehensive documentation review
   - [ ] Performance regression testing

2. **Documentation Updates**
   - [ ] CHANGELOG.md updated
   - [ ] API documentation current
   - [ ] README.md reflects new features
   - [ ] Migration guide if needed

3. **Release Preparation**
   - [ ] Version number updated
   - [ ] Git tags created
   - [ ] Release notes prepared
   - [ ] Crates.io publication ready

## Recognition

Contributors are recognized through:
- Contributor list in README.md
- Release notes acknowledgments  
- GitHub contributor statistics
- Optional blog posts for significant contributions

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Code Reviews**: Feedback on pull requests

### Resources

- [Architecture Documentation](ARCHITECTURE.md)
- [API Reference](API_REFERENCE.md)
- [Performance Guide](PERFORMANCE.md)
- [Type System Guide](TYPE_SYSTEM.md)
- [Effect System Guide](EFFECT_SYSTEM.md)

## Thank You

Your contributions help make Lambdust a world-class Scheme implementation. Every contribution, whether code, documentation, testing, or feedback, is valuable and appreciated.

Welcome to the Lambdust community!