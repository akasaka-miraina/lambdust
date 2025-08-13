# Lambdust Development Guide for Claude

This document provides essential information for Claude to work effectively with the Lambdust project.

## Project Overview

Lambdust is a comprehensive R7RS-large compliant Scheme interpreter written in Rust, featuring:

- **R7RS-large compliance** with extensive SRFI support
- **Advanced type system** with gradual typing, algebraic data types, and type classes
- **Effect system** with monadic programming and effect handlers
- **Concurrency system** supporting actors, futures, STM, and parallel computation
- **FFI system** for C interoperability with dynamic library loading
- **Metaprogramming** with hygienic macros, reflection, and code generation
- **Performance optimization** with bytecode compilation and primitive specialization

## Sub agents

- **language-processer-architect** : Well versed in language processing systems, responsible for the core design of Lambdust. **MUST verify system integrity after each step.**
- **cs-architect** : Architect with expertise in computer science and strong knowledge of type theory and formal semantics. **MUST ensure mathematical correctness is maintained incrementally.**
- **rust-expert-programmer** : Programmers who are proficient in Rust and capable of implementing solutions that maximize performance while maintaining readability. **MUST run `cargo check --lib` after every file modification and maintain 0 errors. Before any commit recommendations, MUST ensure `cargo clippy` shows 0 errors and 0 warnings.**
- **lambdust-r7rs-programmer** : Lambdust's 42 primitives, the theory behind them, and the requirements for a Rust programmer. An excellent Scheme (or Common LISP) programmer. **MUST verify R7RS compliance after each change.**

They recognize their own roles and work together with other agents—or drive multiple agents in parallel—to solve development tasks. **ALL subagents must follow incremental development rules and maintain zero compilation errors at each step.** In addition, related documents must be updated synchronously, and functions that were previously working must be maintained in the same way. These are maintained in accordance with the principle of continuous development.

### **Universal Subagent Requirements:**

1. **Development-Phase Error-Free**: ALL subagents must maintain `cargo check --lib` showing 0 errors during development
2. **Commit-Phase Quality**: Before recommending commits, ensure `cargo clippy` shows 0 errors AND 0 warnings
3. **Incremental Progress**: Make only one focused change per step
4. **Immediate Verification**: Check compilation status after every modification
5. **Stop-and-Fix Protocol**: If errors appear, MUST stop and fix before proceeding
6. **Quality Level Awareness**: Understand difference between development (cargo check) and commit (clippy) standards
7. **Collaboration Responsibility**: When working together, each agent verifies their portion meets appropriate quality level

## Project Structure

```
lambdust/
├── src/                    # Main source code
│   ├── ast/               # Abstract syntax tree definitions
│   ├── bytecode/          # Bytecode compiler and VM
│   ├── concurrency/       # Actor system, futures, channels
│   ├── containers/        # Advanced data structures
│   ├── diagnostics/       # Error handling and reporting
│   ├── effects/           # Effect system and monads
│   ├── eval/              # Core evaluator and values
│   ├── ffi/               # Foreign function interface
│   ├── lexer/             # Tokenization and parsing
│   ├── macro_system/      # Hygienic macro expansion
│   ├── metaprogramming/   # Reflection and code generation
│   ├── module_system/     # Module loading and resolution
│   ├── numeric/           # Numeric tower implementation
│   ├── parser/            # Expression and literal parsing
│   ├── repl/              # Interactive environment
│   ├── runtime/           # Runtime system coordination
│   ├── stdlib/            # Built-in procedures and libraries
│   ├── types/             # Type system and inference
│   └── utils/             # Utilities and helper functions
├── stdlib/                # Scheme standard library modules
├── docs/                  # Documentation
│   └── FormalSemantics.md # Complete formal semantics
├── tests/                 # Comprehensive test suites
├── examples/              # Example programs and demos
├── benches/               # Performance benchmarks
└── scripts/               # Development utilities
```

## Key Implementation Details

### Core Architecture

The interpreter is built around these central components:

1. **42 Core Primitives** (`src/eval/value.rs`): Fundamental operations that bootstrap the entire system
2. **Value System** (`src/eval/value.rs`): Unified value representation supporting all Scheme types
3. **Environment System** (`src/eval/environment.rs`): Variable binding and scope management
4. **Effect Coordination** (`src/runtime/effect_coordinator.rs`): Managing side effects and I/O

### Type System Integration

The advanced type system is integrated through:
- **Type Bridge** (`src/types/integration_bridge.rs`): Connects dynamic and static typing
- **Gradual Typing** (`src/types/gradual.rs`): Smooth transition between type levels
- **Type Classes** (`src/types/type_classes.rs`): Haskell-style type constraints

### Build and Development Commands

```bash
# Basic build and test
cargo build                    # Build the project
cargo test                     # Run all tests
cargo check                    # Quick syntax/type check (DEVELOPMENT STANDARD)
cargo check --lib             # Library-only check (PRIMARY DEVELOPMENT TOOL)

# Performance testing
cargo bench                    # Run benchmarks
cargo test --release          # Run tests in release mode

# REPL and examples
cargo run                      # Start interactive REPL
cargo run --example <name>     # Run specific example

# Code quality tools (COMMIT REQUIREMENTS)
cargo clippy                   # Linting - must be clean before commit
cargo clippy --lib            # Library-only linting
cargo fmt                      # Code formatting (if available)
```

### **Quality Levels:**

- **Development Phase**: `cargo check --lib` with 0 errors is sufficient
- **Commit Phase**: `cargo clippy` with 0 errors AND 0 warnings is required
- **Note**: clippy is much more strict than cargo check and will show many additional issues

### Key Files for Development

- **`src/lib.rs`**: Main library entry point
- **`src/main.rs`**: REPL and CLI interface
- **`src/eval/value.rs`**: Core value types and primitives
- **`src/eval/evaluator.rs`**: Main evaluation engine
- **`src/runtime/bootstrap.rs`**: System initialization
- **`docs/FormalSemantics.md`**: Complete mathematical semantics

### Testing Strategy

The project has comprehensive test coverage:

- **Unit tests**: In each module (`mod tests`)
- **Integration tests**: In `tests/` directory
- **R7RS compliance**: Extensive compliance test suites
- **SRFI tests**: Individual SRFI implementation tests
- **Performance tests**: Benchmarks and performance regression tests

### Common Development Patterns

1. **Adding New Primitives**: Add to `src/eval/value.rs` primitive list and implement in appropriate stdlib module
2. **Adding SRFI Support**: Create module in `src/stdlib/` and add tests in `tests/srfi_*`
3. **Type System Extensions**: Extend `src/types/` modules and update integration bridge
4. **FFI Functions**: Add to `src/ffi/` and ensure proper memory management

### Memory Management

- Uses Rust's ownership system for memory safety
- Garbage collection for Scheme values via reference counting
- Careful handling of FFI pointers and C memory
- Thread-safe shared data structures with `Arc<RwLock<T>>`

### Error Handling

- Comprehensive error types in `src/diagnostics/error.rs`
- Span-based error reporting with source location
- Stack trace preservation for debugging
- Result-based error propagation throughout

### Performance Considerations

- Bytecode compilation for frequently executed code
- Primitive specialization based on type information
- SIMD optimizations for numeric operations
- Memory pooling for allocation-heavy operations

## Development Workflow

### **MANDATORY Incremental Development Process:**

1. **Before Making Changes**: 
   - Run `cargo check --lib` to record current error count
   - Verify project compiles successfully (0 errors preferred)

2. **During Development** (REPEAT for each individual change):
   - Make ONE focused change (single file, single structure, single feature)
   - Run `cargo check --lib` immediately after the change
   - **REQUIREMENT**: Error count must NOT increase
   - If errors appear: STOP and fix them before proceeding

3. **Change Completion Verification**:
   - Run `cargo check --lib 2>&1 | grep -c error` 
   - **MUST return 0** before proceeding to next change

4. **After All Changes**: 
   - Run relevant tests (`cargo test`)
   - Verify all functionality is preserved

5. **Development Phase vs Commit Phase**:
   - **During Development**: `cargo check --lib` level verification is sufficient
   - **Before Committing**: `cargo clippy` must show 0 errors and 0 warnings
   - **Note**: clippy produces many more warnings/errors than cargo check
   - **Important**: Development proceeds with cargo check, commits require clippy clean

6. **Documentation**: 
   - Update relevant docs when adding features
   - Update CLAUDE.md if development processes are refined

### **Key Principles:**

- **Error-Free Development**: Never leave errors "to fix later"
- **One Change at a Time**: Incremental, verifiable progress
- **Immediate Feedback**: Check compilation after every significant change
- **Fail Fast**: Stop and fix issues immediately when they appear

## Code Organization Rules

**CRITICAL: All subagents must follow these mandatory rules for code organization:**

### File and Structure Rules

1. **One Structure Per File**: Each `.rs` file must contain exactly ONE primary structure and its implementation
2. **Structure Name = File Name**: The structure name must match the file name (e.g., `Parser` struct in `parser.rs`)
3. **No Structures in mod.rs**: The `mod.rs` files must NEVER contain structure definitions or implementations
   - Exception: Helper functions and utilities are allowed
   - Exception: Module-level configuration constants
4. **Clean Separation**: Related types should be in separate files with proper `mod.rs` re-exports

### Examples of Correct Organization:

```rust
// src/effects/effect_context.rs - ✅ CORRECT
pub struct EffectContext {
    // fields...
}

impl EffectContext {
    // methods...
}

// src/effects/mod.rs - ✅ CORRECT
pub mod effect_context;
pub mod effect_system;
pub mod lifting_config;

pub use effect_context::EffectContext;
pub use effect_system::EffectSystem;
pub use lifting_config::LiftingConfig;

// Helper functions are OK in mod.rs
pub fn helper_function() -> bool {
    true
}
```

### Examples of Incorrect Organization:

```rust
// src/effects/mod.rs - ❌ WRONG
pub struct EffectContext { /* ... */ }  // Structure definitions not allowed
pub struct EffectSystem { /* ... */ }   // Multiple structures not allowed

impl EffectContext { /* ... */ }        // Implementations not allowed
```

### Migration Priority:

When refactoring existing violations:
1. **High Priority**: Files with multiple structures
2. **Medium Priority**: Structures in mod.rs files
3. **Low Priority**: Minor organization improvements

### Enforcement:

- All subagents (rust-expert-programmer, language-processor-architect, etc.) must enforce these rules
- Code reviews must check for compliance
- New code that violates these rules must be rejected
- Existing violations should be fixed during related work

## Incremental Development Rules

**CRITICAL: All subagents must follow these mandatory incremental development rules:**

### Step-by-Step Development Process

1. **One Step at a Time**: Make only ONE focused change per iteration
2. **Error Check After Each Step**: Run `cargo check --lib` after every modification
3. **Zero Error Requirement**: Each step must maintain or achieve zero compilation errors
4. **No Batch Operations**: Never perform large-scale refactoring or multiple file moves simultaneously

### Examples of Correct Incremental Approach:

```
Step 1: Move ONE structure to new file + update imports
        → cargo check --lib (must show 0 errors)
Step 2: Move SECOND structure to new file + update imports  
        → cargo check --lib (must show 0 errors)
Step 3: Move THIRD structure to new file + update imports
        → cargo check --lib (must show 0 errors)
...continue until complete
```

### Examples of WRONG Batch Approach:

```
❌ WRONG: Move all 39 structures at once
❌ WRONG: Assume imports will work after bulk changes
❌ WRONG: "Fix errors later" mentality
```

### Mandatory Error Checking:

- **Before any change**: `cargo check --lib` must show current error count
- **After each change**: `cargo check --lib` must show same or fewer errors
- **End condition**: `cargo check --lib 2>&1 | grep -c error` must return **0**

### Recovery Protocol:

If errors are introduced during development:
1. **STOP immediately** - do not continue with additional changes
2. **Fix introduced errors first** - before proceeding
3. **Verify zero errors** - before moving to next step
4. **If unfixable** - revert the problematic change and try different approach

### Subagent Responsibilities:

- **rust-expert-programmer**: Must check compilation after every file modification
- **language-processor-architect**: Must verify system integrity at each step
- **All subagents**: Must refuse to proceed if errors are present

## Important Notes

- The project maintains strict R7RS compliance while extending functionality
- All new features should include comprehensive tests
- Performance implications should be considered for core operations
- FFI code requires extra attention to memory safety
- The formal semantics document should be updated for language changes

## Troubleshooting

- **Compilation Issues**: Check feature flags and conditional compilation
- **Test Failures**: Many tests depend on proper environment setup
- **Performance Issues**: Use benchmarks to identify bottlenecks
- **FFI Problems**: Verify C library compatibility and memory management

This guide provides the foundation for effective development work on Lambdust. The project represents a sophisticated implementation of modern Scheme with advanced programming language features.