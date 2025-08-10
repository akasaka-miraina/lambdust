# Lambdust Architecture Documentation

This document provides a comprehensive overview of the Lambdust interpreter architecture, focusing on the clean modular design achieved through systematic structural refactoring.

## Table of Contents

1. [Overview](#overview)
2. [Core Architecture Principles](#core-architecture-principles)
3. [System Components](#system-components)
4. [Module Organization](#module-organization)
5. [Data Flow](#data-flow)
6. [Key Design Decisions](#key-design-decisions)
7. [Performance Considerations](#performance-considerations)
8. [Future Architecture](#future-architecture)

## Overview

Lambdust is architected as a modern, modular Scheme interpreter that successfully combines:

- **Clean separation of concerns** with 226+ individually structured modules
- **Gradual typing integration** bridging dynamic and static type systems
- **Effect system coordination** managing computational contexts transparently
- **High-performance runtime** with bytecode compilation and parallel evaluation

The architecture has been systematically refactored to achieve:
- ✅ **Zero compilation errors** maintained throughout development
- ✅ **Zero clippy warnings** ensuring code quality standards
- ✅ **Professional documentation** across all public interfaces
- ✅ **One-structure-per-file principle** for maximum modularity

## Core Architecture Principles

### 1. **Modular Design**
- Every significant structure lives in its own dedicated file
- Clear public API boundaries with comprehensive documentation
- Minimal coupling between components through well-defined interfaces

### 2. **Incremental Quality**
- Continuous compilation checking during development
- Quality gates enforced at every development step
- Error-free development with immediate feedback loops

### 3. **Functional Architecture**
- Immutable data structures where possible using `im` crate
- Pure functional evaluation with transparent effect handling
- Monadic effect management preserving referential transparency

### 4. **Performance-First Design**
- Zero-copy operations where feasible
- Memory pooling for allocation-heavy operations
- SIMD optimizations for numeric computations
- Bytecode compilation for frequently executed code

## System Components

The architecture is organized into eight major subsystems:

### 1. 🎯 **Core Implementation** (`src/`)

#### **Lexical Analysis** (`src/lexer/`)
- `lexer.rs` - Main tokenization engine using `logos` crate
- `token.rs` - Token definitions and token stream management
- `numeric.rs` - Specialized numeric literal parsing
- `optimized.rs` - Performance-optimized tokenization paths
- `string_utils.rs` - String processing utilities

#### **Parsing** (`src/parser/`)
- `parser.rs` - Main recursive descent parser
- `expression.rs` - Expression parsing with precedence handling
- `literals.rs` - Literal value parsing (numbers, strings, symbols)
- `special_forms.rs` - Special form recognition and parsing
- `parser_builder.rs` - Configurable parser construction

#### **Abstract Syntax Tree** (`src/ast/`)
- `program.rs` - Top-level program representation
- `literal.rs` - Literal value nodes
- `binding.rs` - Variable binding structures
- `visitor.rs` - AST traversal patterns
- `case_clause.rs`, `cond_clause.rs` - Conditional structures

#### **Evaluation Engine** (`src/eval/`)
- `evaluator.rs` - Main evaluation engine with tail call optimization
- `value.rs` - Unified value representation (42 core primitives)
- `environment.rs` - Lexical scoping and variable binding
- `parameter.rs` - Parameter binding and frame management
- `fast_path.rs` - Optimized evaluation paths for common operations

### 2. 🏗️ **Language Systems**

#### **Type System** (`src/types/`)
- `type_checker.rs` - Four-level gradual type checking
- `inference.rs` - Hindley-Milner type inference engine
- `unification.rs` - Type unification with occurs check
- `type_classes.rs` - Type class system with constraint solving
- `algebraic.rs` - Algebraic data types and pattern matching
- `gradual.rs` - Dynamic-static type integration
- `integration_bridge.rs` - Bridge between type systems

#### **Effect System** (`src/effects/`)
- `effect_system.rs` - Central effect coordination
- `effect_context.rs` - Computational context tracking
- `handler.rs` - Effect handler implementation
- `monad.rs` - Core monadic abstractions
- `lifting.rs` - Automatic effect lifting rules
- `generational.rs` - Generational environment management

#### **Macro System** (`src/macro_system/`)
- `macro_expander.rs` - Hygienic macro expansion
- `syntax_rules.rs` - R7RS-compatible syntax-rules
- `pattern.rs` - Pattern matching for macros
- `template.rs` - Template instantiation
- `hygiene.rs` - Hygiene preservation mechanisms

#### **Module System** (`src/module_system/`)
- `module_system.rs` - Module loading and resolution
- `loader.rs` - Dynamic module loading
- `resolver.rs` - Module dependency resolution
- `import.rs`, `export.rs` - Import/export specifications

### 3. ⚡ **Runtime & Performance**

#### **Runtime Coordination** (`src/runtime/`)
- `lambdust_runtime.rs` - Multi-threaded runtime coordination
- `effect_coordinator.rs` - Effect system integration
- `bootstrap.rs` - System initialization and primitive loading
- `evaluator.rs` - Runtime evaluator interface
- `thread_pool.rs` - Thread pool management for parallel evaluation

#### **Bytecode System** (`src/bytecode/`)
- `compiler.rs` - AST to bytecode compilation
- `vm.rs` - Virtual machine execution engine
- `optimizer.rs` - Bytecode optimization passes
- `instruction.rs` - Bytecode instruction definitions
- `bytecode_engine.rs` - High-level bytecode interface

#### **Concurrency** (`src/concurrency/`)
- `actors.rs` - Actor model implementation
- `channels.rs` - Communication channels
- `parallel.rs` - Parallel evaluation strategies
- `futures.rs` - Async/await integration
- `sync.rs` - Synchronization primitives
- `scheduler.rs` - Work-stealing scheduler

#### **High-Performance Containers** (`src/containers/`)
- `hash_table.rs` - Optimized hash table implementation
- `ideque.rs` - Immutable double-ended queue
- `priority_queue.rs` - Priority queue with heap structure
- `random_access_list.rs` - Functional random access list
- `ordered_set.rs` - Ordered set with tree structure

### 4. 🔗 **Interoperability**

#### **Foreign Function Interface** (`src/ffi/`)
- `ffi_registry.rs` - Dynamic function registration
- `library.rs` - Dynamic library loading
- `c_types.rs` - C type mapping and conversion
- `safety.rs` - Memory safety guarantees
- `callback.rs` - Callback function management
- `scheme_api.rs` - Scheme-accessible FFI interface

#### **Standard Library** (`src/stdlib/`)
- `arithmetic.rs` - Numeric operations and mathematical functions
- `lists.rs` - List processing functions
- `strings.rs` - String manipulation
- `io.rs` - Input/output operations
- `vectors.rs` - Vector operations
- `concurrency.rs` - Concurrency primitives for Scheme

#### **Numeric Tower** (`src/numeric/`)
- `bigint.rs` - Arbitrary precision integers
- `rational.rs` - Rational number arithmetic
- `complex.rs` - Complex number support
- `tower.rs` - Numeric tower coordination
- `simd_optimization.rs` - SIMD-accelerated operations

### 5. 🎮 **User Interface**

#### **Enhanced REPL** (`src/repl/`)
- `session.rs` - REPL session management
- `completion.rs` - Intelligent code completion
- `editor.rs` - Line editing with history
- `debugger.rs` - Interactive debugging capabilities
- `inspector.rs` - Value inspection and exploration

### 6. 📊 **Quality Assurance**

#### **Diagnostics** (`src/diagnostics/`)
- `error.rs` - Comprehensive error types with context
- `span.rs` - Source location tracking
- `stack_trace.rs` - Stack trace generation
- `suggestions.rs` - Error recovery and suggestions
- `source_map.rs` - Source mapping for compiled code

#### **Benchmarking** (`src/benchmarks/`)
- `performance_tester.rs` - Performance measurement framework
- `regression_detection.rs` - Performance regression detection
- `scheme_comparison.rs` - Comparison with other Scheme implementations
- `statistical_analysis.rs` - Statistical analysis of performance data

## Module Organization

### File Structure Principle

Every module follows the **one-structure-per-file** principle:

```rust
// ✅ CORRECT: src/effects/effect_context.rs
pub struct EffectContext {
    // Single primary structure per file
}

impl EffectContext {
    // All implementations in the same file
}

// ✅ CORRECT: src/effects/mod.rs  
pub mod effect_context;
pub mod effect_system;

pub use effect_context::EffectContext;
pub use effect_system::EffectSystem;

// Helper functions are allowed in mod.rs
pub fn helper_function() -> bool {
    true
}
```

### Documentation Standards

Every public interface includes comprehensive documentation:

```rust
/// Effect context for tracking computational contexts and effects.
///
/// This structure maintains the current computational context during evaluation,
/// tracking active effects and providing the foundation for effect handling
/// and monadic lifting.
///
/// # Examples
///
/// ```rust
/// let mut context = EffectContext::new();
/// context.push_effect(Effect::IO);
/// assert!(context.has_effect(&Effect::IO));
/// ```
pub struct EffectContext {
    // ...
}
```

### Import Organization

Modules are organized with clear import hierarchies:

```rust
// Core re-exports for convenience
pub use ast::{Expr, Literal, Program};
pub use diagnostics::{Error, Result, Span};
pub use eval::{Evaluator, Value};
pub use runtime::{Runtime, LambdustRuntime};

// Specialized re-exports with namespace preservation
pub use metaprogramming::{
    MetaprogrammingSystem, ReflectionSystem, CodeGenerator
};
```

## Data Flow

### 1. **Compilation Pipeline**

```
Source Code
    ↓
Lexer (tokenization)
    ↓
Parser (AST generation)
    ↓
Macro Expander (hygiene)
    ↓
Type Checker (inference)
    ↓
Effect Analyzer (tracking)
    ↓
Bytecode Compiler (optimization)
    ↓
Virtual Machine (execution)
```

### 2. **Effect Integration**

```
Pure Computation
    ↓
Effect Detection
    ↓
Effect Context Update
    ↓
Handler Resolution
    ↓
Monadic Lifting
    ↓
Effectful Computation
```

### 3. **Type System Integration**

```
Dynamic Expression
    ↓
Type Annotation Detection
    ↓
Inference Engine
    ↓
Constraint Generation
    ↓
Unification
    ↓
Type-Directed Compilation
```

## Key Design Decisions

### 1. **Unified Value Representation**

The `Value` enum in `src/eval/value.rs` serves as the single point of truth for all Scheme values:

```rust
pub enum Value {
    // Basic types
    Number(Number),
    String(String), 
    Symbol(Symbol),
    Boolean(bool),
    Char(char),
    
    // Compound types
    Pair(Box<Value>, Box<Value>),
    Vector(Vec<Value>),
    
    // Functions and control
    Procedure(Procedure),
    Continuation(Continuation),
    
    // Advanced features
    TypeValue(TypeValue),
    ForeignObject(ForeignObject),
    // ...
}
```

**Rationale**: Single representation simplifies type checking, effect tracking, and FFI integration while maintaining performance.

### 2. **Effect System Architecture**

Effects are tracked through a combination of:
- **Effect Context** - Current computational context
- **Effect Handlers** - Pluggable effect management
- **Monadic Lifting** - Automatic effect integration

**Rationale**: This preserves Scheme's dynamic nature while enabling pure functional programming patterns.

### 3. **Gradual Typing Integration**

The type system operates at four levels with seamless transitions:

```rust
pub enum TypeLevel {
    Dynamic,    // Default R7RS behavior
    Contracts,  // Runtime checking
    Static,     // Compile-time inference
    Dependent,  // Proof-carrying code
}
```

**Rationale**: Allows progressive adoption of typing without breaking existing Scheme code.

### 4. **42 Core Primitives**

The system is bootstrapped by exactly 42 core primitives that implement all fundamental operations:

```rust
// Core primitive operations
fn primitive_add(args: &[Value]) -> Result<Value>;
fn primitive_apply(args: &[Value]) -> Result<Value>;
fn primitive_car(args: &[Value]) -> Result<Value>;
// ... 39 more primitives
```

**Rationale**: Minimal kernel approach enables reasoning about correctness while maximizing extensibility.

## Performance Considerations

### 1. **Memory Management**

- **Reference counting** with `Rc<RefCell<T>>` for Scheme values
- **Memory pools** for frequent allocations (`src/utils/memory_pool.rs`)
- **String interning** for symbols (`src/utils/string_interner.rs`)

### 2. **Optimization Strategies**

- **Fast path execution** for common operations (`src/eval/fast_path.rs`)
- **Primitive specialization** based on type information
- **Bytecode compilation** with multi-pass optimization
- **SIMD vectorization** for numeric arrays

### 3. **Concurrency Design**

- **Lock-free data structures** where possible
- **Work-stealing scheduler** for parallel evaluation
- **Actor model** for high-level concurrency
- **STM (Software Transactional Memory)** for coordinated updates

## Integration Points

### 1. **Type System Bridge**

`src/types/integration_bridge.rs` provides seamless integration between:
- Dynamic value system (`Value` enum)
- Static type system (`Type` enum)
- Effect tracking (`Effect` enum)

### 2. **Runtime Coordination**

`src/runtime/effect_coordinator.rs` coordinates:
- Effect system integration
- Multi-threaded evaluation
- I/O operation management
- Resource lifecycle management

### 3. **FFI Integration**

`src/ffi/ffi_registry.rs` enables:
- Dynamic library loading
- Type-safe function binding
- Memory safety guarantees
- Callback management

## Future Architecture

### Planned Improvements

1. **JIT Compilation**
   - LLVM backend integration
   - Profile-guided optimization
   - Adaptive compilation strategies

2. **Distributed Computing**
   - Network-transparent evaluation
   - Automatic data distribution
   - Fault-tolerant execution

3. **Dependent Types**
   - Proof-carrying code
   - Theorem proving integration
   - Advanced verification capabilities

4. **IDE Integration**
   - Language Server Protocol support
   - Real-time error checking
   - Intelligent code completion

### Architectural Challenges

1. **Maintaining R7RS Compatibility**
   - Backward compatibility constraints
   - Performance vs. compliance tradeoffs
   - Extension mechanism design

2. **Effect System Complexity**
   - Effect inference scalability
   - Handler composition patterns
   - Performance overhead minimization

3. **Type System Integration**
   - Gradual typing soundness
   - Error message quality
   - Compilation time optimization

---

This architecture represents the culmination of systematic refactoring efforts that achieved a 100% success rate in structural migration while maintaining zero compilation errors throughout the development process. The modular design enables continued evolution while preserving system integrity and performance characteristics.