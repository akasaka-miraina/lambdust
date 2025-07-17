# Monadic Effects System Design

## Overview

This document outlines the design for integrating a monadic effects system into Lambdust to maintain referential transparency while handling side effects like I/O operations and mutable state. The design is inspired by Haskell's approach to pure functional programming with controlled effects.

## Theoretical Foundation

Based on Moggi's seminal work "Monads and Effects", this system addresses the fundamental tension between:
- **Simple semantics**: Pure functional evaluation with referential transparency
- **Rich possibilities for side-effects**: I/O, state mutation, and other computational effects

### Key Principles
1. **Effect Isolation**: Side effects are explicitly tracked and contained within monadic contexts
2. **Referential Transparency**: Pure computations remain transparent, effects are made explicit
3. **Composability**: Effects can be combined and transformed systematically
4. **R7RS Compliance**: Integration respects Scheme's semantic model

## Current Implementation Analysis

### Existing Side Effect Infrastructure

Lambdust already has robust side effect analysis in `SemanticEvaluator`:

```rust
// src/evaluator/semantic.rs:1545-1559
fn is_side_effect_procedure(&self, name: &str) -> bool {
    matches!(name,
        // Assignment operations
        "set!" | "set-car!" | "set-cdr!" | "vector-set!" | "string-set!" |
        // I/O operations  
        "display" | "write" | "write-char" | "write-string" | "newline" |
        "read" | "read-char" | "read-string" | "read-line" |
        // File operations
        "call-with-output-file" | "call-with-input-file" |
        // System operations
        "load" | "eval" | "error" | "raise"
    )
}
```

### Side Effect Classification

**Mutation Effects**
- `set!` - Variable assignment
- `set-car!`, `set-cdr!` - Pair structure modification
- `vector-set!`, `string-set!` - Container modification

**I/O Effects** 
- `display`, `write`, `write-char` - Output operations
- `read`, `read-char`, `read-string` - Input operations
- `newline` - Line termination output

**System Effects**
- `load`, `eval` - Dynamic code evaluation
- `error`, `raise` - Exception handling
- File operations - Persistent storage access

## Monadic Effects System Design

### Core Types

```rust
/// Effect type classification
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Pure,
    IO(IOEffect),
    State(StateEffect),
    System(SystemEffect),
    Combined(Vec<Effect>),
}

/// I/O effect variants
#[derive(Debug, Clone, PartialEq)]
pub enum IOEffect {
    Read(String),                    // Input operation
    Write(String),                   // Output operation  
    Display(String),                 // Display operation
    FileOpen(String),                // File access
    FileClose(String),               // File closing
}

/// State mutation effects
#[derive(Debug, Clone, PartialEq)]
pub enum StateEffect {
    Mutation {
        variable: String,
        old_value: Option<Value>,
        new_value: Value,
    },
    ContainerUpdate {
        container: Value,
        index: usize,
        old_value: Option<Value>, 
        new_value: Value,
    },
}

/// System-level effects
#[derive(Debug, Clone, PartialEq)]
pub enum SystemEffect {
    Eval(String),                    // Dynamic evaluation
    Load(String),                    // Module loading
    Error(String),                   // Error raising
}

/// Monadic wrapper for effectful computations
#[derive(Debug, Clone)]
pub struct EffectM<T> {
    pub value: T,
    pub effects: Vec<Effect>,
}
```

### Monadic Operations

```rust
impl<T> EffectM<T> {
    /// Create pure monadic value
    pub fn pure(value: T) -> Self {
        Self {
            value,
            effects: vec![Effect::Pure],
        }
    }
    
    /// Create effectful monadic value
    pub fn effectful(value: T, effect: Effect) -> Self {
        Self {
            value,
            effects: vec![effect],
        }
    }
    
    /// Monadic bind operation
    pub fn bind<U, F>(self, f: F) -> EffectM<U>
    where
        F: FnOnce(T) -> EffectM<U>,
    {
        let result = f(self.value);
        let mut combined_effects = self.effects;
        combined_effects.extend(result.effects);
        
        EffectM {
            value: result.value,
            effects: combined_effects,
        }
    }
    
    /// Map over monadic value
    pub fn map<U, F>(self, f: F) -> EffectM<U>
    where
        F: FnOnce(T) -> U,
    {
        EffectM {
            value: f(self.value),
            effects: self.effects,
        }
    }
}
```

## Integration Strategy

### Phase 1: Effect Type System Foundation

**Extend SemanticEvaluator**
- Enhance `is_side_effect_procedure()` to return `Effect` classification
- Create effect tracking infrastructure
- Implement basic monadic wrappers

**Implementation Steps:**
1. Create `src/evaluator/effects/` module structure
2. Implement core effect types and monadic operations
3. Extend `SemanticEvaluator` with effect classification
4. Add effect tracking to pure evaluation methods

### Phase 2: Monadic Evaluation Layer

**MonadicEvaluator Implementation**
- Layer above `SemanticEvaluator` for effect-aware evaluation
- Transform side-effect operations into monadic computations
- Maintain backward compatibility with existing evaluation

**Key Components:**
```rust
pub struct MonadicEvaluator {
    semantic: SemanticEvaluator,
    effect_tracker: EffectTracker,
}

impl MonadicEvaluator {
    /// Evaluate expression with effect tracking
    pub fn eval_monadic(
        &mut self,
        expr: &Expr,
        env: &Environment,
    ) -> Result<EffectM<Value>, LambdustError> {
        // Delegate to semantic evaluator with effect wrapping
    }
    
    /// Execute side effects in controlled manner
    pub fn execute_effects(&mut self, effects: &[Effect]) -> Result<(), LambdustError> {
        // Safe effect execution with rollback capability
    }
}
```

### Phase 3: Runtime Integration

**RuntimeExecutor Enhancement**
- Integrate effect execution engine with optimization system
- Provide effect scheduling and batching
- Maintain optimization opportunities for pure computations

**Effect Optimization Strategies:**
- **Effect Batching**: Combine multiple I/O operations
- **Effect Reordering**: Optimize pure computations between effects
- **Effect Caching**: Memoize deterministic effectful operations

## Architecture Integration

### Relationship with Existing Systems

```
┌─────────────────────┐
│   EvaluatorInterface │ ← Future unified API
├─────────────────────┤
│   MonadicEvaluator  │ ← New effect-aware layer
├─────────────────────┤
│   SemanticEvaluator │ ← Existing pure reference
├─────────────────────┤  
│   RuntimeExecutor   │ ← Existing optimization layer
└─────────────────────┘
```

**Design Benefits:**
- **Incremental Adoption**: Can be introduced gradually
- **Backward Compatibility**: Existing code continues to work
- **Performance Preservation**: Pure computations remain optimized
- **Effect Safety**: Side effects become explicit and controllable

### SemanticEvaluator Integration Points

**Effect Detection Enhancement:**
```rust
// Current: src/evaluator/semantic.rs:505
"set!" => self.eval_set_pure(operands, env, cont),

// Enhanced monadic version:
"set!" => self.eval_set_monadic(operands, env, cont),
```

**Pure/Effect Separation:**
- Pure operations continue through existing `eval_pure()` methods
- Effectful operations route through new monadic evaluation
- Effect tracking maintains mathematical reference implementation

## Implementation Roadmap

### T0: Immediate Priority Task (Current Sprint)
**🎯 HIGHEST PRIORITY: Begin Monadic Effects System Implementation**
- [ ] Create `src/evaluator/effects/` module structure
- [ ] Implement basic `Effect` enum and classification
- [ ] Add `EffectM<T>` monadic wrapper with core operations
- [ ] Extend `SemanticEvaluator::is_side_effect_procedure()` to return `Effect` types
- [ ] Create initial integration tests for effect tracking
- [ ] Update project documentation to reflect T0 completion

### Milestone 1: Core Infrastructure (Week 1-2)  
- [ ] Complete remaining `Effect` type system features
- [ ] Enhance `EffectM` monadic wrapper with advanced operations
- [ ] Full effect classification integration in `SemanticEvaluator`
- [ ] Comprehensive test suite for effect tracking and monadic operations

### Milestone 2: Monadic Evaluator (Week 3-4)
- [ ] Implement `MonadicEvaluator` structure
- [ ] Integrate with existing `SemanticEvaluator`
- [ ] Transform I/O operations to monadic form
- [ ] Add state effect tracking for `set!` operations

### Milestone 3: Runtime Integration (Week 5-6)
- [ ] Enhance `RuntimeExecutor` with effect execution
- [ ] Implement effect optimization strategies
- [ ] Add effect batching and scheduling
- [ ] Performance benchmarking and optimization

### Milestone 4: API Unification (Week 7-8)
- [ ] Design `EvaluatorInterface` for transparent switching
- [ ] Implement monadic/pure evaluation selection
- [ ] Add verification system for effect correctness
- [ ] Complete documentation and examples

## Testing Strategy

### Effect Correctness Verification
```rust
#[test]
fn test_effect_tracking_completeness() {
    // Verify all side effects are captured
    let expr = parse("(begin (set! x 42) (display x))").unwrap();
    let result = monadic_eval.eval_monadic(&expr, &env).unwrap();
    
    assert_eq!(result.effects.len(), 2);
    assert!(matches!(result.effects[0], Effect::State(_)));
    assert!(matches!(result.effects[1], Effect::IO(IOEffect::Display(_))));
}

#[test] 
fn test_pure_computation_optimization() {
    // Ensure pure computations remain fast
    let expr = parse("(+ (* 2 3) (/ 12 4))").unwrap();
    let result = monadic_eval.eval_monadic(&expr, &env).unwrap();
    
    assert_eq!(result.effects, vec![Effect::Pure]);
    assert_eq!(result.value, Value::Number(SchemeNumber::Integer(9)));
}
```

### Effect Isolation Testing
- Verify effects don't leak between evaluations
- Test effect rollback capabilities  
- Validate referential transparency preservation

## Security and Safety Considerations

### Effect Sandboxing
- I/O effects can be restricted to safe operations
- File access effects can be limited to allowed directories
- System effects can be disabled in restricted environments

### Effect Auditing
- Complete effect trace for security analysis
- Effect logging for debugging and monitoring
- Effect replay for testing and verification

## Future Extensions

### Advanced Effect Systems
- **Effect Handlers**: Algebraic effects and handlers
- **Effect Polymorphism**: Generic effect-polymorphic functions
- **Effect Inference**: Automatic effect classification

### Performance Optimizations
- **Effect Fusion**: Combine related effects for efficiency
- **Lazy Effects**: Defer effect execution until needed
- **Parallel Effects**: Execute independent effects concurrently

### Language Extensions
- **Effect Annotations**: Scheme syntax for effect declarations
- **Effect Types**: Static effect typing for improved safety
- **Effect Contracts**: Runtime effect verification

## Conclusion

This monadic effects system provides a principled approach to managing side effects in Lambdust while preserving the benefits of pure functional programming. The design leverages existing infrastructure, maintains backward compatibility, and provides a foundation for advanced effect management techniques.

The implementation will enhance Lambdust's referential transparency guarantees while enabling powerful I/O and state manipulation capabilities, bringing Scheme closer to modern functional programming best practices.