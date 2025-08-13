# Contributing to Lambdust

Thank you for your interest in contributing to Lambdust! This document provides comprehensive guidelines for contributing to the project while maintaining our high standards for code quality, architecture, and development workflow.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Standards](#development-standards)
3. [Code Organization Rules](#code-organization-rules)
4. [Development Workflow](#development-workflow)
5. [Testing Guidelines](#testing-guidelines)
6. [Documentation Standards](#documentation-standards)
7. [Submission Process](#submission-process)
8. [Code Review Process](#code-review-process)
9. [Community Guidelines](#community-guidelines)

## Getting Started

### Prerequisites

1. **Rust 2024 Edition** or later
2. **Git** for version control
3. **GitHub account** for pull requests
4. **Development environment** set up according to [BUILDING.md](BUILDING.md)

### First-Time Setup

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/yourusername/lambdust.git
cd lambdust

# Add upstream remote
git remote add upstream https://github.com/username/lambdust.git

# Create development branch
git checkout -b feature/your-feature-name

# Verify everything builds
cargo check --lib
cargo clippy --lib
cargo test
```

### Quick Contribution Checklist

Before submitting any contribution, ensure:

- ✅ `cargo check --lib` shows **0 errors**
- ✅ `cargo clippy --lib` shows **0 errors and 0 warnings**
- ✅ `cargo test` passes **all tests**
- ✅ **Documentation** updated for new features
- ✅ **One focused change** per pull request
- ✅ **Commit messages** follow conventional format
- ✅ **Code organization** follows project standards

## Development Standards

### Quality Gates

Lambdust maintains **zero-error development standards**:

1. **Development Phase**: `cargo check --lib` must show 0 errors
2. **Commit Phase**: `cargo clippy --lib` must show 0 errors AND 0 warnings
3. **Integration Phase**: All tests must pass

### Incremental Development Rules

**MANDATORY**: Follow these rules for all development work:

1. **One Change at a Time**: Make only ONE focused change per iteration
2. **Error Check After Each Step**: Run `cargo check --lib` after every modification
3. **Zero Error Requirement**: Each step must maintain or achieve zero compilation errors
4. **No Batch Operations**: Never perform large-scale refactoring simultaneously

#### Example: Correct Incremental Approach

```bash
# Step 1: Move ONE structure to new file + update imports
git add src/effects/new_structure.rs src/effects/mod.rs
cargo check --lib  # MUST show 0 errors
git commit -m "Move NewStructure to dedicated file"

# Step 2: Move SECOND structure to new file + update imports  
git add src/effects/another_structure.rs src/effects/mod.rs
cargo check --lib  # MUST show 0 errors
git commit -m "Move AnotherStructure to dedicated file"

# Continue one structure at a time...
```

#### Recovery Protocol

If errors are introduced during development:

1. **STOP immediately** - do not continue with additional changes
2. **Fix introduced errors first** - before proceeding with next step
3. **Verify zero errors** - before moving to next change
4. **If unfixable** - revert the problematic change and try different approach

## Code Organization Rules

### File and Structure Rules

**CRITICAL**: All contributors must follow these mandatory rules:

1. **One Structure Per File**: Each `.rs` file must contain exactly ONE primary structure
2. **Structure Name = File Name**: Structure name must match file name
3. **No Structures in mod.rs**: `mod.rs` files must NEVER contain structure definitions
4. **Clean Separation**: Related types should be in separate files with proper re-exports

#### ✅ Correct Organization:

```rust
// src/effects/effect_context.rs - CORRECT
pub struct EffectContext {
    // fields...
}

impl EffectContext {
    // methods...
}

// src/effects/mod.rs - CORRECT
pub mod effect_context;
pub mod effect_system;

pub use effect_context::EffectContext;
pub use effect_system::EffectSystem;

// Helper functions are OK in mod.rs
pub fn helper_function() -> bool {
    true
}
```

#### ❌ Incorrect Organization:

```rust
// src/effects/mod.rs - WRONG
pub struct EffectContext { /* ... */ }  // Structure definitions not allowed
pub struct EffectSystem { /* ... */ }   // Multiple structures not allowed

impl EffectContext { /* ... */ }        // Implementations not allowed
```

### Import Guidelines

Organize imports with clear hierarchies:

```rust
// Standard library imports first
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Internal imports (grouped by module)
use crate::diagnostics::{Error, Result};
use crate::eval::Value;
```

## Development Workflow

### Branch Strategy

1. **Main Branch**: Always deployable, protected
2. **Feature Branches**: `feature/descriptive-name` for new features
3. **Bug Fix Branches**: `fix/issue-description` for bug fixes
4. **Documentation Branches**: `docs/topic-name` for documentation

### Commit Message Format

Use conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no logic changes)
- `refactor`: Code refactoring
- `test`: Test additions or modifications
- `perf`: Performance improvements
- `build`: Build system changes

Examples:
```
feat(types): Add algebraic data type support

Implement pattern matching and constructor functions for
user-defined algebraic data types with proper type checking.

Closes #42
```

### Development Commands

```bash
# Primary development check (use frequently)
cargo check --lib

# Quality check (use before commits)
cargo clippy --lib

# Format code
cargo fmt

# Test specific areas
cargo test eval::tests
cargo test --doc

# Full quality verification
cargo clippy --lib -- -D warnings
cargo test --release
```

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **Documentation Tests**: Test code examples in docs
4. **Property-Based Tests**: Random testing with `proptest`
5. **Performance Tests**: Benchmark and regression tests

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        let value = create_test_value();
        assert_eq!(value.expected_property(), expected_result);
    }
    
    #[test]
    #[should_panic(expected = "specific error message")]
    fn test_error_conditions() {
        create_invalid_input();
    }
}
```

#### Integration Tests
```rust
// tests/integration_test.rs
use lambdust::{Lambdust, Value};

#[test]
fn test_end_to_end_evaluation() {
    let mut interpreter = Lambdust::new();
    let result = interpreter.eval("(+ 1 2 3)", None).unwrap();
    assert_eq!(result, Value::Number(6.into()));
}
```

#### Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_arithmetic_properties(a in any::<i64>(), b in any::<i64>()) {
        let result = add_numbers(a, b);
        prop_assert_eq!(result, a + b);
    }
}
```

### Test Requirements

- **Coverage**: Aim for 90%+ code coverage for new code
- **Edge Cases**: Test boundary conditions and error paths
- **Documentation**: All tests should be self-documenting
- **Performance**: Include performance tests for critical paths
- **R7RS Compliance**: New language features need R7RS compatibility tests

## Documentation Standards

### Documentation Requirements

All public interfaces require comprehensive documentation:

```rust
/// Brief description of the structure/function.
///
/// Longer description explaining the purpose, behavior, and usage patterns.
/// Include information about thread safety, performance characteristics,
/// and any important implementation details.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value and any special cases.
///
/// # Errors
///
/// Description of possible error conditions and when they occur.
///
/// # Examples
///
/// ```
/// use lambdust::YourType;
/// 
/// let instance = YourType::new();
/// let result = instance.method(42)?;
/// assert_eq!(result, expected_value);
/// ```
///
/// # Panics
///
/// Description of conditions that cause panics (if any).
///
/// # Safety
///
/// For unsafe functions, describe safety requirements.
pub struct YourType {
    // ...
}
```

### Documentation Types

1. **API Documentation**: Public interfaces with examples
2. **Architecture Documentation**: High-level design decisions
3. **Tutorial Documentation**: Step-by-step usage guides
4. **Reference Documentation**: Comprehensive feature coverage

### Documentation Testing

```bash
# Test documentation examples
cargo test --doc

# Generate documentation
cargo doc --open

# Check for missing documentation
cargo doc --document-private-items
```

## Submission Process

### Pre-Submission Checklist

Before creating a pull request:

```bash
# 1. Ensure your branch is up to date
git fetch upstream
git rebase upstream/main

# 2. Run full quality check
cargo check --lib
cargo clippy --lib
cargo test
cargo test --doc

# 3. Check formatting
cargo fmt --check

# 4. Run benchmarks (if applicable)
cargo bench --features benchmarks

# 5. Update documentation
cargo doc --open
```

### Pull Request Guidelines

1. **Title**: Clear, descriptive title following conventional commits
2. **Description**: Explain what changes were made and why
3. **Testing**: Describe how the changes were tested
4. **Breaking Changes**: Clearly mark any breaking changes
5. **Dependencies**: Note any new dependencies or version updates
6. **Performance**: Include benchmark results for performance-related changes

#### Pull Request Template

```markdown
## Description
Brief description of changes made.

## Changes Made
- [ ] Feature addition
- [ ] Bug fix  
- [ ] Refactoring
- [ ] Documentation
- [ ] Performance improvement

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] Performance tests included

## Quality Checklist
- [ ] `cargo check --lib` shows 0 errors
- [ ] `cargo clippy --lib` shows 0 errors and warnings
- [ ] `cargo test` passes all tests
- [ ] Documentation updated
- [ ] Commit messages follow conventional format

## Breaking Changes
None / Describe breaking changes

## Additional Notes
Any additional context or considerations.
```

### Review Process

1. **Automated Checks**: CI pipeline must pass
2. **Code Review**: At least one maintainer approval required
3. **Architecture Review**: For significant architectural changes
4. **Performance Review**: For performance-critical changes
5. **Documentation Review**: For public API changes

## Code Review Process

### As a Contributor

- **Respond Promptly**: Address review feedback quickly
- **Be Receptive**: Consider reviewer suggestions seriously  
- **Ask Questions**: Clarify unclear feedback
- **Update Documentation**: Keep docs in sync with changes
- **Test Thoroughly**: Verify all suggested changes work

### Review Criteria

Reviewers will evaluate:

1. **Code Quality**: Adherence to project standards
2. **Architecture**: Consistency with system design
3. **Testing**: Adequate test coverage and quality
4. **Documentation**: Clarity and completeness
5. **Performance**: Impact on system performance
6. **Maintainability**: Long-term code maintainability

### Common Review Feedback

- **Missing Tests**: Add unit/integration tests
- **Missing Documentation**: Add or improve documentation
- **Code Organization**: Follow one-structure-per-file rule
- **Performance Concerns**: Optimize critical paths
- **Breaking Changes**: Minimize breaking changes

## Community Guidelines

### Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct):

- **Be Respectful**: Treat all community members with respect
- **Be Inclusive**: Welcome newcomers and diverse perspectives
- **Be Constructive**: Provide helpful, actionable feedback
- **Be Patient**: Allow time for learning and improvement

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions  
- **Pull Requests**: Code contributions and reviews
- **Documentation**: Comprehensive guides and references

### Getting Help

1. **Check Documentation**: Read existing docs first
2. **Search Issues**: Look for existing discussions
3. **Ask Questions**: Create discussion topics for questions
4. **Provide Context**: Include relevant system information

### Recognition

Contributors are recognized through:

- **Commit Attribution**: Git history preserves authorship
- **Release Notes**: Significant contributions mentioned
- **Documentation**: Contributor acknowledgment sections

## Advanced Contribution Topics

### Performance Contributions

For performance-related contributions:

1. **Benchmark First**: Establish baseline performance
2. **Profile Changes**: Use profiling tools to verify improvements
3. **Include Benchmarks**: Add benchmark tests for new code paths
4. **Document Trade-offs**: Explain performance vs. other considerations

### Architecture Contributions

For architectural changes:

1. **Propose First**: Discuss major changes before implementation
2. **Document Design**: Update architecture documentation
3. **Consider Compatibility**: Maintain R7RS compatibility
4. **Plan Migration**: Provide migration path for breaking changes

### FFI Contributions

For Foreign Function Interface contributions:

1. **Memory Safety**: Ensure proper memory management
2. **Type Safety**: Provide type-safe Rust wrappers
3. **Documentation**: Document C interoperability requirements
4. **Testing**: Include FFI-specific tests

---

Thank you for contributing to Lambdust! Your contributions help make Scheme programming more powerful, efficient, and enjoyable for everyone. Together, we're building a modern Lisp that combines the elegance of Scheme with the performance and safety of Rust.