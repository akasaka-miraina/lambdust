# Lambdust Architecture

This document provides a comprehensive overview of Lambdust's architecture, design decisions, and implementation details.

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Core Components](#core-components)
3. [Compilation Pipeline](#compilation-pipeline)
4. [Type System](#type-system)
5. [Effect System](#effect-system)
6. [Memory Management](#memory-management)
7. [Module System](#module-system)
8. [FFI Integration](#ffi-integration)
9. [Performance Considerations](#performance-considerations)
10. [Future Architecture](#future-architecture)

## High-Level Architecture

Lambdust follows a traditional compiler/interpreter architecture with modern functional programming enhancements:

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Source Code   │───▶│    Lexer     │───▶│     Tokens      │
└─────────────────┘    └──────────────┘    └─────────────────┘
                                                    │
┌─────────────────┐    ┌──────────────┐            ▼
│   Typed AST     │◀───│ Type Checker │    ┌─────────────────┐
└─────────────────┘    └──────────────┘    │     Parser      │
         │                      ▲          └─────────────────┘
         ▼                      │                   │
┌─────────────────┐    ┌──────────────┐            ▼
│ Effect Analysis │───▶│     AST      │◀───┌─────────────────┐
└─────────────────┘    └──────────────┘    │ Macro Expander  │
         │                      │          └─────────────────┘
         ▼                      ▼
┌─────────────────┐    ┌──────────────┐
│   Optimized     │───▶│   Evaluator  │
│     Code        │    │  /Compiler   │
└─────────────────┘    └──────────────┘
                               │
                               ▼
                       ┌──────────────┐
                       │   Runtime    │
                       │   System     │
                       └──────────────┘
```

### Design Principles

1. **Correctness First**: R7RS compliance and mathematical correctness
2. **Gradual Enhancement**: Optional typing and effect annotations
3. **Performance**: Zero-cost abstractions where possible
4. **Composability**: Clean separation of concerns
5. **Extensibility**: Plugin architecture for features and optimizations

## Core Components

### 1. Lexer (`src/lexer/`)

The lexer converts source text into a stream of tokens using the `logos` crate for performance.

```rust
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
    Boolean(bool),
    
    // Symbols and identifiers
    Symbol(String),
    Keyword(String),
    
    // Structural
    LeftParen,
    RightParen,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    
    // Special
    Dot,
    EOF,
}
```

**Key Features**:
- Fast tokenization with `logos`
- Unicode support for identifiers
- Proper handling of Scheme numeric literals
- Source location tracking for error reporting

### 2. Parser (`src/parser/`)

Recursive descent parser that builds an Abstract Syntax Tree (AST):

```rust
pub enum Expr {
    // Literals
    Literal(Literal),
    
    // Variables and symbols
    Symbol(String),
    
    // Lists and function calls
    List(Vec<Expr>),
    
    // Special forms
    Quote(Box<Expr>),
    If {
        test: Box<Expr>,
        consequent: Box<Expr>,
        alternate: Option<Box<Expr>>,
    },
    Lambda {
        params: Parameters,
        body: Vec<Expr>,
    },
    Define {
        name: String,
        value: Box<Expr>,
    },
    // ... other forms
}
```

**Key Features**:
- Handles all R7RS syntax forms
- Proper error recovery and reporting
- Source span tracking for debugging
- Extensible for new syntax forms

### 3. Macro System (`src/macro_system/`)

Hygienic macro expansion using `syntax-rules`:

```rust
pub struct MacroExpander {
    environment: MacroEnvironment,
    hygiene_context: HygieneContext,
}

impl MacroExpander {
    pub fn expand(&mut self, expr: &Expr) -> Result<Expr, MacroError> {
        match expr {
            Expr::List(exprs) if self.is_macro_call(exprs) => {
                self.expand_macro(exprs)
            }
            _ => self.expand_subexpressions(expr),
        }
    }
}
```

**Key Features**:
- Full `syntax-rules` support
- Hygienic variable capture prevention
- Recursive macro expansion
- Error reporting with macro context

### 4. Type System (`src/types/`)

Gradual type system with Hindley-Milner inference:

```rust
pub enum Type {
    // Base types
    Number,
    String,
    Boolean,
    Character,
    
    // Compound types
    List(Box<Type>),
    Vector(Box<Type>),
    Function {
        params: Vec<Type>,
        result: Box<Type>,
        effects: EffectSet,
    },
    
    // Type variables for inference
    Variable(TypeVar),
    
    // Contracts and refinements
    Contract(ContractExpr),
}
```

**Type Inference Engine**:
```rust
pub struct TypeInferencer {
    constraints: Vec<Constraint>,
    substitutions: Substitution,
    type_var_counter: usize,
}

impl TypeInferencer {
    pub fn infer(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_type(lit)),
            Expr::Symbol(name) => self.lookup_type(name, env),
            Expr::List(exprs) => self.infer_application(exprs, env),
            // ... other cases
        }
    }
}
```

### 5. Effect System (`src/effects/`)

Tracks and manages computational effects:

```rust
pub enum Effect {
    // I/O effects
    Read,
    Write,
    FileSystem,
    Network,
    
    // State effects
    Mutation,
    Reference,
    
    // Control effects
    Exception,
    Continuation,
    
    // Resource effects
    Memory,
    Time,
}

pub struct EffectAnalyzer {
    effect_stack: Vec<EffectSet>,
    handlers: HashMap<Effect, Handler>,
}
```

**Effect Handling**:
```rust
pub fn handle_effects<T>(
    effects: &[Effect],
    handlers: &[Handler],
    computation: impl FnOnce() -> T,
) -> Result<T, EffectError> {
    // Effect handling implementation
}
```

### 6. Evaluator (`src/eval/`)

Tree-walking interpreter with optimizations:

```rust
pub struct Evaluator {
    environment: Environment,
    call_stack: CallStack,
    gc: GarbageCollector,
}

impl Evaluator {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Literal(lit) => Ok(Value::from_literal(lit)),
            Expr::Symbol(name) => self.lookup_variable(name),
            Expr::List(exprs) => self.eval_application(exprs),
            Expr::If { test, consequent, alternate } => {
                self.eval_conditional(test, consequent, alternate)
            }
            // ... other cases
        }
    }
    
    fn eval_application(&mut self, exprs: &[Expr]) -> Result<Value, EvalError> {
        let proc = self.eval(&exprs[0])?;
        let args: Result<Vec<_>, _> = exprs[1..].iter()
            .map(|arg| self.eval(arg))
            .collect();
        
        self.apply_procedure(proc, args?)
    }
}
```

## Compilation Pipeline

### Phase 1: Lexical Analysis

```rust
// Input: "(define (square x) (* x x))"
// Output: Tokens
[
    LeftParen,
    Symbol("define"),
    LeftParen,
    Symbol("square"),
    Symbol("x"),
    RightParen,
    LeftParen,
    Symbol("*"),
    Symbol("x"),
    Symbol("x"),
    RightParen,
    RightParen,
]
```

### Phase 2: Parsing

```rust
// Output: AST
Define {
    name: "square",
    value: Lambda {
        params: Parameters::Fixed(vec!["x"]),
        body: vec![
            List(vec![
                Symbol("*"),
                Symbol("x"),
                Symbol("x"),
            ])
        ],
    },
}
```

### Phase 3: Macro Expansion

```rust
// Expands user-defined and built-in macros
// (when test body ...) => (if test (begin body ...))
```

### Phase 4: Type Checking (Optional)

```rust
// Infers types and checks annotations
// (define (square x) (* x x))
// Inferred type: (-> Number Number)
```

### Phase 5: Effect Analysis

```rust
// Tracks computational effects
// Pure function: no effects
// I/O function: (Read Write) effects
```

### Phase 6: Optimization

```rust
// Various optimizations:
// - Constant folding
// - Dead code elimination  
// - Tail call optimization
// - Inline expansion
```

### Phase 7: Evaluation/Compilation

```rust
// Either interpret directly or compile to bytecode/native code
```

## Type System

### Type Hierarchy

```
Value
├── Number
│   ├── Integer
│   ├── Rational  
│   ├── Real
│   └── Complex
├── Boolean
├── Character
├── String
├── Symbol
├── Pair
├── List
├── Vector
├── Procedure
├── Port
└── Record
```

### Type Inference

Lambdust uses a modified Hindley-Milner type system:

1. **Constraint Generation**: Generate type constraints from expressions
2. **Unification**: Solve constraints to find most general types
3. **Generalization**: Abstract over type variables to create polymorphic types
4. **Instantiation**: Create fresh type variables for polymorphic types

```rust
// Type constraint examples
// (+ x y) generates:
//   x : Number
//   y : Number  
//   result : Number

// (cons x xs) generates:
//   xs : List[α]
//   result : List[α]
//   (no constraint on x, it can be any type α)
```

### Gradual Typing

Types can be gradually added:

```scheme
;; Stage 1: Dynamic typing
(define (map f lst)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; Stage 2: Add type annotations
(define (map f lst)
  #:type (-> (-> a b) (List a) (List b))
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; Stage 3: Add contracts for runtime checking
(define (map f lst)
  #:type (-> (-> a b) (List a) (List b))
  #:contract (-> procedure? list? list?)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))
```

## Effect System

### Effect Types

Effects are categorized into several types:

```rust
pub enum EffectCategory {
    // Pure computations (no effects)
    Pure,
    
    // I/O operations  
    IO(IOEffect),
    
    // State manipulation
    State(StateEffect),
    
    // Control flow
    Control(ControlEffect),
    
    // Resource management
    Resource(ResourceEffect),
}
```

### Effect Inference

Effects are inferred automatically:

```scheme
;; Pure function - no effects inferred
(define (add x y) (+ x y))

;; I/O effects inferred from display
(define (greet name)
  (display "Hello, ")
  (display name))

;; State effects inferred from set!
(define (counter)
  (let ([n 0])
    (lambda ()
      (set! n (+ n 1))
      n)))
```

### Effect Handlers

Effects can be handled using monadic patterns:

```scheme
;; Handle I/O effects
(with-io-handler
  (lambda (io-op)
    ;; Custom I/O handling
    )
  (lambda ()
    ;; Code with I/O effects
    ))

;; Handle state effects  
(with-state initial-state
  (lambda ()
    ;; Stateful computation
    ))
```

## Memory Management

### Garbage Collection

Lambdust uses a generational copying garbage collector:

```rust
pub struct GarbageCollector {
    young_generation: Heap,
    old_generation: Heap,
    remembered_set: HashSet<ObjectRef>,
}

impl GarbageCollector {
    pub fn collect(&mut self) -> CollectionStats {
        // Minor collection first
        if self.young_generation.needs_collection() {
            self.minor_collection()
        }
        
        // Major collection if needed
        if self.old_generation.needs_collection() {
            self.major_collection()
        }
    }
}
```

### Value Representation

Values use tagged unions for efficient representation:

```rust
pub enum Value {
    // Immediate values (no heap allocation)
    Integer(i64),
    Boolean(bool),
    Character(char),
    
    // Heap-allocated values
    String(Gc<String>),
    List(Gc<List>),
    Vector(Gc<Vector>),
    Procedure(Gc<Procedure>),
}
```

### Memory Pool

For performance, small objects use memory pools:

```rust
pub struct MemoryPool<T> {
    free_list: Vec<*mut T>,
    chunks: Vec<Box<[T]>>,
    chunk_size: usize,
}
```

## Module System

### Library Definition

Libraries are defined using R7RS syntax:

```scheme
(define-library (math utilities)
  (export square cube factorial)
  (import (scheme base))
  
  (begin
    (define (square x) (* x x))
    (define (cube x) (* x x x))
    (define (factorial n)
      (if (<= n 1)
          1
          (* n (factorial (- n 1)))))))
```

### Module Resolution

```rust
pub struct ModuleResolver {
    search_paths: Vec<PathBuf>,
    loaded_modules: HashMap<ModuleName, Module>,
    dependency_graph: Graph<ModuleName>,
}

impl ModuleResolver {
    pub fn resolve(&mut self, name: &ModuleName) -> Result<Module, ModuleError> {
        // Check if already loaded
        if let Some(module) = self.loaded_modules.get(name) {
            return Ok(module.clone());
        }
        
        // Find module file
        let path = self.find_module_file(name)?;
        
        // Load and compile module
        let module = self.load_module(&path)?;
        
        // Cache for future use
        self.loaded_modules.insert(name.clone(), module.clone());
        
        Ok(module)
    }
}
```

## FFI Integration

### Rust FFI

Lambdust provides seamless Rust integration:

```rust
// Define Rust function
#[no_mangle]
pub extern "C" fn rust_add(x: i64, y: i64) -> i64 {
    x + y
}

// Register with Lambdust
pub fn register_rust_functions(env: &Environment) {
    env.define("rust-add", Value::ForeignFunction {
        name: "rust_add",
        arity: 2,
        function: rust_add as *const (),
    });
}
```

```scheme
;; Use from Scheme
(define (fast-add x y)
  (rust-add x y))
```

### C FFI

Support for calling C libraries:

```scheme
;; Load C library
(load-library "libm.so")

;; Define foreign function
(define c-sin
  (foreign-function "libm" "sin"
    (-> double double)))

;; Use it
(c-sin 3.14159)
```

## Performance Considerations

### Optimization Strategies

1. **Tail Call Optimization**: Convert tail calls to jumps
2. **Constant Folding**: Evaluate constant expressions at compile time
3. **Inline Expansion**: Inline small functions to reduce call overhead
4. **Type Specialization**: Generate specialized code for known types
5. **Effect Optimization**: Optimize pure computations

### Benchmarking

Performance is tracked across multiple dimensions:

```rust
pub struct BenchmarkSuite {
    arithmetic_benchmarks: Vec<Benchmark>,
    list_processing_benchmarks: Vec<Benchmark>,
    macro_expansion_benchmarks: Vec<Benchmark>,
    memory_benchmarks: Vec<Benchmark>,
}
```

### Memory Usage

- **Small integers**: Unboxed for efficiency
- **Strings**: Copy-on-write sharing
- **Lists**: Structure sharing where possible
- **Procedures**: Closure capture optimization

## Future Architecture

### Planned Enhancements

1. **Bytecode Compiler**: Compile to efficient bytecode
2. **JIT Compilation**: Just-in-time compilation for hot code
3. **Parallel Evaluation**: Thread-safe parallel execution
4. **Advanced Type Features**: Dependent types and refinement types
5. **Native Compilation**: Compile to native machine code

### Extensibility Points

The architecture is designed for extensibility:

- **Pluggable optimizations**: Add new optimization passes
- **Custom effects**: Define domain-specific effects
- **Language extensions**: Add new syntax forms
- **Backend targets**: Support multiple compilation targets

### Modular Design

Each component is designed to be replaceable:

```rust
pub trait Lexer {
    fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, LexError>;
}

pub trait Parser {
    fn parse(&mut self, tokens: &[Token]) -> Result<Expr, ParseError>;
}

pub trait TypeChecker {
    fn check(&mut self, expr: &Expr) -> Result<Type, TypeError>;
}

pub trait Evaluator {
    fn eval(&mut self, expr: &Expr) -> Result<Value, EvalError>;
}
```

This modular design allows:
- **A/B testing** of different implementations
- **Performance experiments** with different algorithms
- **Feature development** without breaking existing code
- **Third-party extensions** and customizations

The architecture balances correctness, performance, and maintainability while providing a solid foundation for future enhancements and extensions.