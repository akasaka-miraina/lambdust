# Lambdust API Reference

This document provides comprehensive API documentation for the Lambdust Scheme interpreter library. The API is designed to be both powerful for advanced users and accessible for newcomers.

## Table of Contents

1. [Library Interface](#library-interface)
2. [Core Types](#core-types)
3. [Evaluation System](#evaluation-system)
4. [Type System](#type-system)
5. [Effect System](#effect-system)
6. [Runtime System](#runtime-system)
7. [Examples](#examples)
8. [Error Handling](#error-handling)

## Library Interface

### Primary Interfaces

Lambdust provides two main interfaces for different use cases:

#### `Lambdust` - Single-threaded Interface

```rust
use lambdust::{Lambdust, Value, Result};

pub struct Lambdust {
    runtime: Runtime,
}

impl Lambdust {
    /// Creates a new Lambdust instance with default configuration.
    pub fn new() -> Self;
    
    /// Creates a new Lambdust instance with custom runtime configuration.
    pub fn with_runtime(runtime: Runtime) -> Self;
    
    /// Evaluates a Lambdust program from source code.
    pub fn eval(&mut self, source: &str, filename: Option<&str>) -> Result<Value>;
    
    /// Tokenizes source code into a stream of tokens.
    pub fn tokenize(&self, source: &str, filename: Option<&str>) -> Result<Vec<Token>>;
    
    /// Parses tokens into an Abstract Syntax Tree.
    pub fn parse(&self, tokens: Vec<Token>) -> Result<Program>;
    
    /// Expands macros in the AST.
    pub fn expand_macros(&self, program: Program) -> Result<Program>;
    
    /// Performs type checking on the AST.
    pub fn type_check(&self, program: Program) -> Result<Program>;
}
```

**Example Usage:**
```rust
use lambdust::Lambdust;

let mut interpreter = Lambdust::new();
let result = interpreter.eval("(+ 1 2 3)", Some("example.scm"))?;
println!("Result: {}", result); // Result: 6
```

#### `MultithreadedLambdust` - Parallel Interface

```rust
use lambdust::{MultithreadedLambdust, ParallelResult, EvaluatorHandle};

pub struct MultithreadedLambdust {
    runtime: LambdustRuntime,
}

impl MultithreadedLambdust {
    /// Creates a new multithreaded Lambdust instance.
    pub fn new(num_threads: Option<usize>) -> Result<Self>;
    
    /// Evaluates a program using parallel evaluation.
    pub async fn eval(&self, source: &str, filename: Option<&str>) -> Result<Value>;
    
    /// Evaluates multiple expressions in parallel.
    pub async fn eval_parallel(&self, sources: Vec<(&str, Option<&str>)>) -> Result<ParallelResult>;
    
    /// Spawns a new evaluator and returns a handle to it.
    pub fn spawn_evaluator(&self) -> Result<EvaluatorHandle>;
    
    /// Gets the number of evaluator threads.
    pub fn thread_count(&self) -> usize;
    
    /// Shuts down the multithreaded runtime.
    pub async fn shutdown(self) -> Result<()>;
}
```

**Example Usage:**
```rust
use lambdust::MultithreadedLambdust;

let interpreter = MultithreadedLambdust::new(Some(4)).await?;
let sources = vec![
    ("(fibonacci 10)", Some("fib.scm")),
    ("(factorial 5)", Some("fact.scm")),
    ("(prime? 97)", Some("prime.scm")),
];
let results = interpreter.eval_parallel(sources).await?;
interpreter.shutdown().await?;
```

## Core Types

### Value System

The `Value` enum represents all possible Scheme values:

```rust
pub enum Value {
    // Primitive values
    Literal(Literal),           // Numbers, strings, characters, booleans
    Symbol(SymbolId),          // Interned symbols
    Keyword(String),           // Keywords (#:key)
    Nil,                       // Empty list ()
    Unspecified,              // Result of side effects
    
    // Compound values
    Pair(Arc<Value>, Arc<Value>),                    // Cons pairs
    Vector(Arc<RwLock<Vec<Value>>>),                // Vectors
    Hashtable(Arc<RwLock<HashMap<Value, Value>>>),  // Hash tables
    
    // Advanced containers (Thread-safe)
    AdvancedHashTable(Arc<ThreadSafeHashTable>),    // SRFI-125
    Ideque(Arc<PersistentIdeque>),                  // SRFI-134
    PriorityQueue(Arc<ThreadSafePriorityQueue>),    // Priority queues
    OrderedSet(Arc<ThreadSafeOrderedSet>),          // Red-black trees
    ListQueue(Arc<ThreadSafeListQueue>),            // SRFI-117
    RandomAccessList(Arc<ThreadSafeRandomAccessList>), // SRFI-101
    
    // Procedures
    Procedure(Arc<Procedure>),                       // User-defined procedures
    CaseLambda(Arc<CaseLambdaProcedure>),           // Variable arity procedures
    Primitive(Arc<PrimitiveProcedure>),             // Built-in primitives
    Continuation(Arc<Continuation>),                 // Continuations
    Syntax(Arc<SyntaxTransformer>),                 // Macros
    
    // Advanced features
    Port(Arc<Port>),                                // I/O ports
    Promise(Arc<RwLock<Promise>>),                  // Lazy evaluation
    Type(Arc<TypeValue>),                           // Type values
    ForeignObject(Arc<ForeignObject>),              // FFI objects
    Parameter(Arc<Parameter>),                      // Parameters
}
```

#### Value Creation

```rust
use lambdust::Value;
use std::sync::Arc;

// Creating primitive values
let num = Value::from(42);
let str_val = Value::from("hello");
let bool_val = Value::from(true);

// Creating compound values
let pair = Value::cons(Value::from(1), Value::from(2));
let list = Value::list(vec![Value::from(1), Value::from(2), Value::from(3)]);
let vector = Value::vector(vec![Value::from("a"), Value::from("b")]);
```

#### Value Operations

```rust
impl Value {
    // Type predicates
    pub fn is_number(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_symbol(&self) -> bool;
    pub fn is_pair(&self) -> bool;
    pub fn is_list(&self) -> bool;
    pub fn is_procedure(&self) -> bool;
    
    // Constructors
    pub fn cons(car: Value, cdr: Value) -> Value;
    pub fn list(elements: Vec<Value>) -> Value;
    pub fn vector(elements: Vec<Value>) -> Value;
    
    // Accessors (with error handling)
    pub fn car(&self) -> Result<&Value>;
    pub fn cdr(&self) -> Result<&Value>;
    pub fn vector_ref(&self, index: usize) -> Result<Value>;
    pub fn vector_set(&self, index: usize, value: Value) -> Result<()>;
    
    // Conversions
    pub fn to_string(&self) -> Result<String>;
    pub fn to_number(&self) -> Result<f64>;
    pub fn to_bool(&self) -> bool; // Scheme truthiness
}
```

### AST Types

#### Program Structure

```rust
pub struct Program {
    pub expressions: Vec<Spanned<Expr>>,
    pub imports: Vec<ImportSpec>,
    pub exports: Vec<ExportSpec>,
}

pub enum Expr {
    // Literals and identifiers
    Literal(Literal),
    Symbol(String),
    Keyword(String),
    
    // Special forms
    Quote(Box<Spanned<Expr>>),
    Lambda {
        formals: Formals,
        body: Vec<Spanned<Expr>>,
    },
    If {
        test: Box<Spanned<Expr>>,
        consequent: Box<Spanned<Expr>>,
        alternate: Option<Box<Spanned<Expr>>>,
    },
    Define {
        name: String,
        value: Box<Spanned<Expr>>,
    },
    Set {
        name: String,
        value: Box<Spanned<Expr>>,
    },
    
    // Applications and other expressions
    Application {
        operator: Box<Spanned<Expr>>,
        operands: Vec<Spanned<Expr>>,
    },
}
```

#### Literals

```rust
pub enum Literal {
    Number(Number),
    String(String),
    Character(char),
    Boolean(bool),
    Bytevector(Vec<u8>),
}

pub enum Number {
    Integer(i64),
    BigInteger(num_bigint::BigInt),
    Rational(num_rational::Rational64),
    BigRational(num_rational::BigRational),
    Real(f64),
    Complex {
        real: f64,
        imag: f64,
    },
}
```

## Evaluation System

### Evaluator Interface

```rust
pub struct Evaluator {
    environment: Environment,
    call_stack: Vec<Frame>,
}

impl Evaluator {
    pub fn new() -> Self;
    pub fn with_environment(env: Environment) -> Self;
    
    /// Evaluates an expression in the current environment.
    pub fn eval(&mut self, expr: &Spanned<Expr>) -> Result<Value>;
    
    /// Evaluates a sequence of expressions.
    pub fn eval_sequence(&mut self, exprs: &[Spanned<Expr>]) -> Result<Value>;
    
    /// Applies a procedure to arguments.
    pub fn apply(&mut self, proc: &Value, args: &[Value]) -> Result<Value>;
    
    /// Enters a new lexical scope.
    pub fn push_frame(&mut self, bindings: HashMap<String, Value>);
    
    /// Exits the current lexical scope.
    pub fn pop_frame(&mut self);
}
```

### Environment Management

```rust
pub struct Environment {
    frames: Vec<Frame>,
    global: Arc<RwLock<HashMap<String, Value>>>,
    generation: Generation,
}

impl Environment {
    pub fn new() -> Self;
    pub fn global() -> Self; // Pre-populated with primitives
    
    /// Defines a new variable in the current scope.
    pub fn define(&mut self, name: String, value: Value) -> Result<()>;
    
    /// Sets an existing variable's value.
    pub fn set(&mut self, name: &str, value: Value) -> Result<()>;
    
    /// Looks up a variable's value.
    pub fn lookup(&self, name: &str) -> Result<Value>;
    
    /// Creates a new generation (for mutation tracking).
    pub fn new_generation(&mut self) -> Generation;
    
    /// Extends environment with new bindings.
    pub fn extend(&self, bindings: HashMap<String, Value>) -> Environment;
}
```

### Procedures

```rust
pub struct Procedure {
    pub formals: Formals,
    pub body: Vec<Spanned<Expr>>,
    pub environment: Environment,
    pub name: Option<String>,
}

pub enum Formals {
    Fixed(Vec<String>),                    // (lambda (a b c) ...)
    Variadic(Vec<String>, String),         // (lambda (a b . rest) ...)  
    Single(String),                        // (lambda args ...)
}

pub struct PrimitiveProcedure {
    pub name: String,
    pub arity: Arity,
    pub implementation: PrimitiveImpl,
    pub pure: bool,
    pub effects: Vec<Effect>,
}

pub enum Arity {
    Exact(usize),
    AtLeast(usize),
    Between(usize, usize),
    Any,
}
```

## Type System

### Type Representation

```rust
pub enum Type {
    // Basic types
    Number,
    String,
    Symbol,
    Boolean,
    Char,
    Unit,
    
    // Compound types
    Pair(Box<Type>, Box<Type>),
    List(Box<Type>),
    Vector(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    // Type variables and polymorphism
    Variable(TypeVar),
    Forall {
        vars: Vec<TypeVar>,
        body: Box<Type>,
    },
    
    // Advanced types
    Record(Row),
    Variant(Row),
    Dynamic,
    Unknown,
}

pub struct TypeVar {
    pub id: u32,
    pub name: Option<String>,
}

pub struct Row {
    pub fields: HashMap<String, Type>,
    pub rest: Option<TypeVar>,
}
```

### Type Checking

```rust
pub struct TypeChecker {
    env: TypeEnv,
    constraints: Vec<Constraint>,
    substitution: Substitution,
}

impl TypeChecker {
    pub fn new() -> Self;
    
    /// Infers the type of an expression.
    pub fn infer(&mut self, expr: &Spanned<Expr>) -> Result<Type>;
    
    /// Checks that an expression has the expected type.
    pub fn check(&mut self, expr: &Spanned<Expr>, expected: &Type) -> Result<()>;
    
    /// Unifies two types, updating the substitution.
    pub fn unify(&mut self, a: &Type, b: &Type) -> Result<()>;
    
    /// Applies the current substitution to a type.
    pub fn apply_substitution(&self, ty: &Type) -> Type;
}
```

### Gradual Typing

```rust
pub enum TypeLevel {
    Dynamic,    // Default R7RS behavior
    Contracts,  // Runtime type checking
    Static,     // Compile-time inference  
    Dependent,  // Proof-carrying code
}

pub struct GradualTypeChecker {
    level: TypeLevel,
    static_checker: TypeChecker,
    contract_checker: ContractChecker,
}

impl GradualTypeChecker {
    pub fn new(level: TypeLevel) -> Self;
    
    /// Checks types according to the current level.
    pub fn check_gradual(&mut self, expr: &Spanned<Expr>) -> Result<Type>;
    
    /// Inserts runtime checks for contract violations.
    pub fn insert_checks(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>>;
}
```

## Effect System

### Effect Types

```rust
pub enum Effect {
    Pure,                    // No effects
    IO,                      // Input/output operations
    State,                   // Mutations (state changes)
    Error,                   // Error handling
    Custom(String),          // User-defined effects
}

pub struct EffectContext {
    active_effects: Vec<Effect>,
    handlers: Vec<Arc<dyn EffectHandler>>,
    generation: Generation,
}
```

### Effect Handlers

```rust
pub trait EffectHandler: std::fmt::Debug {
    /// Handles an effect with the given arguments.
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult>;
    
    /// Returns the name of the effect this handler manages.
    fn effect_name(&self) -> &str;
    
    /// Returns true if this handler can handle the given effect.
    fn can_handle(&self, effect: &Effect) -> bool;
}

pub enum EffectResult {
    Value(Value),            // Effect handled successfully
    Continue(Value),         // Continue with another computation
    Unhandled,              // Effect not handled
    Error(Error),           // Error during handling
}
```

### Monadic Interface

```rust
pub trait Monad<T> {
    type Output<U>;
    
    fn pure(value: T) -> Self::Output<T>;
    fn bind<U, F>(self, f: F) -> Self::Output<U>
    where
        F: FnOnce(T) -> Self::Output<U>;
}

// IO Monad
pub struct IO<T> {
    computation: Box<dyn FnOnce() -> Result<T>>,
}

impl<T> IO<T> {
    pub fn run(self) -> Result<T> {
        (self.computation)()
    }
    
    pub fn lift<F>(f: F) -> IO<T> 
    where
        F: FnOnce() -> Result<T> + 'static,
    {
        IO { computation: Box::new(f) }
    }
}
```

## Runtime System

### Runtime Configuration

```rust
pub struct Runtime {
    evaluator: Evaluator,
    type_checker: Option<TypeChecker>,
    effect_system: EffectSystem,
    module_system: ModuleSystem,
}

impl Runtime {
    pub fn new() -> Self;
    
    pub fn with_config(config: RuntimeConfig) -> Self;
    
    /// Evaluates a program in this runtime.
    pub fn eval(&mut self, program: Program) -> Result<Value>;
    
    /// Loads a module into the runtime.
    pub fn load_module(&mut self, name: &str) -> Result<()>;
    
    /// Registers a new primitive procedure.
    pub fn register_primitive(&mut self, name: String, proc: PrimitiveProcedure);
}

pub struct RuntimeConfig {
    pub type_level: TypeLevel,
    pub max_call_stack: usize,
    pub gc_threshold: usize,
    pub enable_tail_calls: bool,
    pub enable_debugging: bool,
}
```

### Parallel Runtime

```rust
pub struct LambdustRuntime {
    evaluators: Vec<EvaluatorHandle>,
    scheduler: Scheduler,
    effect_coordinator: EffectCoordinator,
}

impl LambdustRuntime {
    pub fn new() -> Result<Self>;
    pub fn with_threads(count: usize) -> Result<Self>;
    
    /// Evaluates a program using parallel evaluation.
    pub async fn eval_program(&self, program: &Program) -> Result<Value>;
    
    /// Evaluates multiple expressions in parallel.
    pub async fn eval_parallel(&self, exprs: Vec<(Expr, Option<Span>)>) -> ParallelResult;
    
    /// Spawns a new evaluator thread.
    pub fn spawn_evaluator(&self) -> Result<EvaluatorHandle>;
}

pub struct ParallelResult {
    pub results: Vec<Result<Value>>,
    pub total_time: std::time::Duration,
    pub individual_times: Vec<std::time::Duration>,
    pub thread_usage: Vec<f64>,
}
```

## Examples

### Basic Evaluation

```rust
use lambdust::{Lambdust, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Lambdust::new();
    
    // Arithmetic
    let result = interpreter.eval("(+ 1 2 3)", None)?;
    assert_eq!(result, Value::from(6));
    
    // Define and use a function
    interpreter.eval("(define (square x) (* x x))", None)?;
    let result = interpreter.eval("(square 5)", None)?;
    assert_eq!(result, Value::from(25));
    
    // List processing
    interpreter.eval("(define lst '(1 2 3 4 5))", None)?;
    let result = interpreter.eval("(map square lst)", None)?;
    // result is (1 4 9 16 25)
    
    Ok(())
}
```

### Type Annotations

```rust
use lambdust::{Lambdust, TypeLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Lambdust::new();
    
    // Enable static type checking
    interpreter.runtime_mut().set_type_level(TypeLevel::Static);
    
    // Define a typed function
    interpreter.eval(r#"
        (define (typed-add x y)
          #:type (-> Number Number Number)
          (+ x y))
    "#, None)?;
    
    // This will pass type checking
    let result = interpreter.eval("(typed-add 3 4)", None)?;
    assert_eq!(result, Value::from(7));
    
    // This would cause a type error
    // interpreter.eval("(typed-add \"hello\" 4)", None)?; // Error!
    
    Ok(())
}
```

### Effect Handling

```rust
use lambdust::{Lambdust, Value, Effect, EffectHandler, EffectResult};

struct LoggingHandler;

impl EffectHandler for LoggingHandler {
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if let Effect::Custom(name) = effect {
            if name == "log" {
                if let Some(Value::String(msg)) = args.first() {
                    println!("LOG: {}", msg);
                    return Ok(EffectResult::Value(Value::Unspecified));
                }
            }
        }
        Ok(EffectResult::Unhandled)
    }
    
    fn effect_name(&self) -> &str { "log" }
    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Custom(name) if name == "log")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Lambdust::new();
    
    // Register custom effect handler
    interpreter.runtime_mut().register_effect_handler(Box::new(LoggingHandler));
    
    // Use custom effect
    interpreter.eval(r#"
        (define (greet name)
          (effect 'log (string-append "Greeting " name))
          (display "Hello, ")
          (display name)
          (newline))
    "#, None)?;
    
    interpreter.eval("(greet \"World\")", None)?;
    // Output: LOG: Greeting World
    //         Hello, World
    
    Ok(())
}
```

### Parallel Evaluation

```rust
use lambdust::MultithreadedLambdust;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interpreter = MultithreadedLambdust::new(Some(4))?;
    
    // Define computation-heavy functions
    interpreter.eval(r#"
        (define (fibonacci n)
          (if (<= n 1)
              n
              (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
        
        (define (factorial n)
          (if (<= n 1)
              1
              (* n (factorial (- n 1)))))
    "#, None).await?;
    
    // Evaluate multiple expressions in parallel
    let sources = vec![
        ("(fibonacci 35)", Some("fib.scm")),
        ("(factorial 20)", Some("fact.scm")), 
        ("(map fibonacci '(10 15 20 25))", Some("map.scm")),
    ];
    
    let results = interpreter.eval_parallel(sources).await?;
    
    println!("Total time: {:?}", results.total_time);
    for (i, result) in results.results.iter().enumerate() {
        println!("Result {}: {:?}", i, result);
        println!("Time {}: {:?}", i, results.individual_times[i]);
    }
    
    interpreter.shutdown().await?;
    Ok(())
}
```

## Error Handling

### Error Types

```rust
pub enum Error {
    // Parse errors
    ParseError {
        message: String,
        span: Span,
    },
    
    // Runtime errors
    TypeError {
        expected: String,
        actual: String,
        span: Option<Span>,
    },
    
    NameError {
        name: String,
        span: Option<Span>,
    },
    
    ArityError {
        expected: String,
        actual: usize,
        span: Option<Span>,
    },
    
    // Effect errors
    EffectError {
        effect: Effect,
        message: String,
        span: Option<Span>,
    },
    
    // System errors
    IOError {
        kind: std::io::ErrorKind,
        message: String,
    },
}
```

### Error Recovery

```rust
use lambdust::{Lambdust, Error};

fn main() {
    let mut interpreter = Lambdust::new();
    
    // Handle parse errors
    match interpreter.eval("(+ 1 2", None) {
        Ok(value) => println!("Result: {}", value),
        Err(Error::ParseError { message, span }) => {
            eprintln!("Parse error at {:?}: {}", span, message);
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
    
    // Handle runtime errors with stack traces
    match interpreter.eval("(undefined-function 42)", None) {
        Err(Error::NameError { name, span }) => {
            eprintln!("Undefined variable: {} at {:?}", name, span);
            if let Some(trace) = interpreter.runtime().stack_trace() {
                eprintln!("Stack trace:");
                for frame in trace.frames() {
                    eprintln!("  at {}", frame);
                }
            }
        }
        _ => {}
    }
}
```

---

This API reference provides comprehensive coverage of the Lambdust library interface. For more detailed examples and tutorials, see the documentation in the `docs/` directory and the example programs in the `examples/` directory.