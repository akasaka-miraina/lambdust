# Lambdust Architecture

This document provides a comprehensive overview of Lambdust's architecture, reflecting the current state after successful structural refactoring and clean architecture implementation.

## Overview

Lambdust is built with a clean, modular architecture that follows domain-driven design principles. The system is organized into distinct layers and components, each with well-defined responsibilities and clear boundaries.

## Project Structure

```
lambdust/
├── src/                         # Core implementation
│   ├── ast/                    # Abstract syntax tree definitions
│   ├── bytecode/               # Bytecode compilation and virtual machine
│   ├── concurrency/            # Concurrency primitives and actor system
│   │   ├── sync/               # Synchronization primitives (9 modules)
│   │   ├── actors.rs           # Actor model implementation
│   │   ├── channels.rs         # Communication channels
│   │   ├── futures.rs          # Async/await support
│   │   └── parallel.rs         # Parallel evaluation
│   ├── containers/             # Advanced data structures
│   ├── diagnostics/            # Error handling and reporting
│   ├── effects/                # Effect system implementation
│   ├── eval/                   # Core evaluation engine
│   │   ├── monadic_architecture/ # Clean architecture (22 modules)
│   │   ├── testing_architecture/ # Testing infrastructure (25 modules)
│   │   ├── evaluator.rs        # Main evaluator
│   │   └── value.rs           # Value representation
│   ├── ffi/                    # Foreign function interface
│   ├── lexer/                  # Tokenization
│   ├── macro_system/           # Macro expansion
│   ├── metaprogramming/        # Advanced metaprogramming
│   │   ├── environment_management/ # Environment system (7 modules)
│   │   ├── program_analysis/   # Program analysis (9 modules)
│   │   └── ...
│   ├── module_system/          # Module loading and resolution
│   ├── numeric/                # Numeric tower implementation
│   ├── parser/                 # Expression parsing
│   ├── repl/                   # Interactive environment
│   ├── runtime/                # Runtime coordination
│   │   ├── effect_coordination/ # Effect coordination (15 modules)
│   │   └── ...
│   ├── stdlib/                 # Built-in procedures
│   ├── types/                  # Type system
│   └── utils/                  # Utility functions
├── docs/                       # Documentation
└── benches/                   # Performance benchmarks
```

## Core Architectural Principles

### 1. Clean Architecture

Lambdust follows clean architecture principles with clear separation of layers:

- **Domain Layer**: Core business logic and language semantics
- **Application Layer**: Use cases and orchestration logic
- **Infrastructure Layer**: External interfaces and implementations

### 2. Modular Design

Each module has a single responsibility with minimal coupling:

```rust
// Example: Effect coordination system
pub mod effect_coordinator_main;    // Main coordinator
pub mod thread_effect_state;       // Thread-local state
pub mod effect_policies;           // Policy management
pub mod concurrent_effect_system;  // Concurrent operations
```

### 3. One Structure Per File

All structures follow the "one primary structure per file" principle:

```rust
// src/eval/monadic_architecture/monadic_computation.rs
pub struct MonadicComputation<M, A> {
    computation: Box<dyn FnOnce() -> M::Container<A>>,
}

// Implementation in same file
impl<M: Monad, A> MonadicComputation<M, A> { ... }
```

## System Components

### 1. Evaluation Engine

The evaluation engine uses a monadic architecture with clean separation:

```
eval/monadic_architecture/
├── domain/                 # Core monadic computation logic
├── application/           # Evaluation orchestration 
└── infrastructure/       # External system interfaces
```

**Key Components:**
- `MonadicEvaluationOrchestrator`: Coordinates evaluation workflow
- `MonadicComputation`: Represents computations in monadic context
- `EffectInterpreter`: Handles effect interpretation

### 2. Type System

Four-level gradual typing system:

```rust
pub enum TypeLevel {
    Dynamic,        // No static typing
    Optional,       // Optional type annotations
    Gradual,        // Mixed static/dynamic
    Static,         // Full static typing
}
```

**Components:**
- Type inference engine with Hindley-Milner extensions
- Constraint solver for type unification  
- Integration bridge between dynamic and static typing
- Algebraic data types and type classes

### 3. Effect System

Algebraic effect system with handler-based composition:

```rust
pub trait Effect {
    type Operation;
    type Result;
}

pub struct EffectHandler<E: Effect> {
    handle: Box<dyn Fn(E::Operation) -> E::Result>,
}
```

**Architecture:**
- Effect definitions with operation signatures
- Handler implementations for effect interpretation
- Effect coordination across concurrent contexts
- Monadic integration for pure functional programming

### 4. Concurrency System

Actor model with message passing:

```
concurrency/
├── sync/                  # Synchronization primitives
│   ├── mutex.rs          # Thread-safe mutual exclusion
│   ├── rwlock.rs         # Reader-writer locks
│   ├── semaphore.rs      # Resource control
│   └── atomic_ref.rs     # Lock-free programming
├── actors.rs             # Actor implementation
├── channels.rs           # Communication channels
└── parallel.rs           # Parallel evaluation
```

### 5. Runtime Coordination

Advanced effect coordination and resource management:

```
runtime/effect_coordination/
├── effect_coordinator_main.rs    # Main coordinator
├── concurrent_effect_system.rs   # Concurrent operations
├── effect_isolation_level.rs     # Isolation management
└── effect_sandbox_config.rs      # Sandboxing
```

## Data Flow

### 1. Evaluation Pipeline

```
Input → Lexer → Parser → AST → Macro Expansion → 
Type Checking → Effect Analysis → Evaluation → Output
```

### 2. Effect Handling

```
Effect Request → Effect Coordinator → Handler Lookup → 
Effect Interpretation → Result Propagation
```

### 3. Concurrent Evaluation

```
Expression → Parallelization Analysis → Task Distribution → 
Worker Threads → Result Aggregation → Final Value
```

## Memory Management

### 1. Garbage Collection

- Reference counting for Scheme values
- Cycle detection for circular references
- Generational collection for performance
- Memory pressure monitoring

### 2. Resource Management

```rust
pub struct ResourceManager {
    memory_tracker: MemoryTracker,
    pressure_monitor: MemoryPressureMonitor,
    gc_policy: GCPolicy,
}
```

## Performance Architecture

### 1. Benchmarking System

Comprehensive performance monitoring:

```
benchmarks/
├── comprehensive_benchmark_suite/  # Main benchmarking (7 modules)
├── statistical_analysis/          # Statistical analysis (5 modules)  
├── regression_detection/          # Performance regression (7 modules)
└── performance_monitoring/       # Real-time monitoring
```

### 2. Optimization Strategies

- SIMD operations for numeric computations
- Bytecode compilation for frequently executed code
- Primitive specialization based on type information
- Memory pooling for allocation-heavy operations

## Testing Architecture

Sophisticated testing infrastructure:

```
eval/testing_architecture/
├── di_container.rs           # Dependency injection
├── mock_environment_manager.rs  # Environment mocking
├── test_fixture_builder.rs  # Test fixture construction
└── test_execution_context.rs   # Execution context
```

## Foreign Function Interface

Safe C interoperability:

```
ffi/
├── builtin_ffi_module.rs    # Module registration
├── io_functions.rs          # I/O function bindings
├── type_checking_functions.rs  # Type checking
└── libffi_integration.rs   # Low-level FFI
```

## Module System

Dynamic module loading with dependency resolution:

```rust
pub struct ModuleSystem {
    loader: SchemeLibraryLoader,
    cache: ModuleCache,
    resolver: DependencyResolver,
}
```

## Quality Assurance

### 1. Code Organization Standards

- One primary structure per file
- Clean module boundaries
- Consistent naming conventions
- Comprehensive documentation

### 2. Testing Standards

- Unit tests for each module
- Integration tests for system interactions
- Property-based testing for critical algorithms
- Performance regression testing

### 3. Documentation Standards

- Module-level documentation
- Comprehensive API documentation  
- Usage examples and tutorials
- Architecture decision records

## Extension Points

The architecture provides clear extension points for:

1. **New Effect Types**: Implement `Effect` trait
2. **Custom Data Types**: Extend value system
3. **Alternative Backends**: Implement evaluation traits
4. **Additional Languages**: Extend parser and AST
5. **Performance Analyzers**: Extend benchmarking system

## Development Workflow

The architecture supports efficient development through:

1. **Incremental Compilation**: Modular design enables fast rebuilds
2. **Isolated Testing**: Each component can be tested independently  
3. **Clear Interfaces**: Well-defined boundaries between components
4. **Documentation Integration**: Architecture docs stay synchronized with code

This architecture reflects the current state of Lambdust after comprehensive structural refactoring, providing a solid foundation for continued development and advanced language features.