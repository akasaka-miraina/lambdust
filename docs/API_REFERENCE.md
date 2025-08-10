# Lambdust API Reference

This document provides comprehensive API reference for Lambdust's core functionality, reflecting the current state after successful structural refactoring and clean architecture implementation.

## Core API Overview

Lambdust provides a rich API organized into logical modules with clear separation of concerns. All APIs follow consistent patterns and maintain R7RS compliance while providing advanced features.

## Value System

### Core Value Types

```rust
use lambdust::eval::Value;

// Creating values
let number = Value::number(42.0);
let string = Value::string("hello");
let boolean = Value::boolean(true);
let symbol = Value::symbol("my-symbol");
let nil = Value::Nil;

// Lists and pairs
let list = Value::list(vec![
    Value::number(1.0),
    Value::number(2.0),
    Value::number(3.0)
]);
let pair = Value::pair(Value::number(1.0), Value::number(2.0));

// Vectors
let vector = Value::vector(vec![
    Value::string("a"),
    Value::string("b"),
    Value::string("c")
]);
```

### Value Operations

```rust
// Type checking
assert!(value.is_number());
assert!(value.is_string());
assert!(value.is_list());

// Conversion
let as_number: Option<f64> = value.as_number();
let as_string: Option<&str> = value.as_string();
let as_list: Option<Vec<Value>> = value.as_list();

// Display and formatting
println!("{}", value);              // Display representation
println!("{:?}", value);            // Debug representation
```

## Evaluation Engine

### Basic Evaluation

```rust
use lambdust::eval::{Evaluator, Environment};

// Create evaluator
let mut evaluator = Evaluator::new();

// Simple expressions
let result = evaluator.eval("(+ 1 2 3)")?;
assert_eq!(result, Value::number(6.0));

// Variables
evaluator.eval("(define x 42)")?;
let result = evaluator.eval("x")?;
assert_eq!(result, Value::number(42.0));

// Functions
evaluator.eval("(define (square x) (* x x))")?;
let result = evaluator.eval("(square 5)")?;
assert_eq!(result, Value::number(25.0));
```

### Advanced Evaluation

```rust
use lambdust::eval::monadic_architecture::MonadicEvaluationOrchestrator;

// Monadic evaluation with effects
let orchestrator = MonadicEvaluationOrchestrator::new(config);
let input = MonadicEvaluationInput::new(expression, environment);
let result = orchestrator.evaluate_expression(input).await?;

// Effect-aware evaluation
let result = orchestrator.evaluate_with_effects(
    expression,
    vec![Effect::IO, Effect::State],
    handlers
).await?;
```

## Type System

### Gradual Typing

```rust
use lambdust::types::{TypeSystem, TypeLevel, TypeInference};

// Type system configuration
let mut type_system = TypeSystem::new();
type_system.set_level(TypeLevel::Gradual);

// Type inference
let inferred_type = type_system.infer_type(expression)?;
println!("Inferred type: {}", inferred_type);

// Type checking
let is_valid = type_system.check_type(value, expected_type)?;

// Gradual type annotations
evaluator.eval("(define (add x : Number y : Number) : Number (+ x y))")?;
```

### Type Definitions

```scheme
;; Algebraic data types
(define-type Color
  (Red)
  (Green) 
  (Blue)
  (RGB Number Number Number))

;; Pattern matching
(define (color-to-string color)
  (match color
    [(Red) "red"]
    [(Green) "green"]
    [(Blue) "blue"]
    [(RGB r g b) (format "rgb(~a,~a,~a)" r g b)]))

;; Type classes
(define-type-class (Eq a)
  (equal? : a a -> Boolean))

(define-instance (Eq Number)
  (define (equal? x y) (= x y)))
```

## Effect System

### Basic Effects

```rust
use lambdust::effects::{Effect, EffectHandler, EffectSystem};

// Define effect types
#[derive(Debug, Clone)]
pub enum ConsoleEffect {
    Print(String),
    ReadLine,
}

// Implement effect handler
struct ConsoleHandler;

impl EffectHandler<ConsoleEffect> for ConsoleHandler {
    type Result = Value;
    
    fn handle(&self, effect: ConsoleEffect) -> Result<Self::Result> {
        match effect {
            ConsoleEffect::Print(msg) => {
                println!("{}", msg);
                Ok(Value::Unspecified)
            }
            ConsoleEffect::ReadLine => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                Ok(Value::string(input.trim()))
            }
        }
    }
}
```

### Monadic Programming

```scheme
;; IO monad example
(define-monad IO
  (return : a -> (IO a))
  (bind : (IO a) (a -> (IO b)) -> (IO b)))

;; Stateful computation
(define-monad (State s)
  (return : a -> (State s a))
  (bind : (State s a) (a -> (State s b)) -> (State s b)))

;; Combining effects
(define (interactive-program)
  (do [name (read-line)]
      [_ (print (string-append "Hello, " name "!"))]
      [count (get-state)]
      [_ (put-state (+ count 1))]
      (return count)))
```

### Effect Coordination

```rust
use lambdust::runtime::effect_coordination::EffectCoordinator;

// Configure effect coordination
let coordinator = EffectCoordinator::new()
    .with_isolation_level(EffectIsolationLevel::Strict)
    .with_concurrent_effects(true)
    .with_resource_limits(limits);

// Execute with coordination
let result = coordinator.execute_with_coordination(
    computation,
    dependencies,
    handlers
).await?;
```

## Concurrency

### Actor Model

```rust
use lambdust::concurrency::actors::{Actor, ActorSystem, Message};

// Define actor
struct CounterActor {
    count: i32,
}

#[derive(Debug)]
enum CounterMessage {
    Increment,
    Decrement,
    GetCount(tokio::sync::oneshot::Sender<i32>),
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, message: Self::Message) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
            CounterMessage::GetCount(sender) => {
                let _ = sender.send(self.count);
            }
        }
        Ok(())
    }
}

// Use actor system
let system = ActorSystem::new();
let counter = system.spawn(CounterActor { count: 0 }).await?;

counter.send(CounterMessage::Increment).await?;
let (tx, rx) = tokio::sync::oneshot::channel();
counter.send(CounterMessage::GetCount(tx)).await?;
let count = rx.await?;
```

### Parallel Evaluation

```scheme
;; Parallel map
(define (parallel-map f lst)
  (parallel-eval
    (map (lambda (x) (spawn (lambda () (f x)))) lst)))

;; Futures and promises
(define future-result
  (future (expensive-computation input)))

;; Later...
(define result (force future-result))

;; Concurrent evaluation with synchronization
(define (concurrent-example)
  (let ([barrier (make-barrier 3)])
    (parallel-eval
      (spawn (lambda () (task-1) (barrier-wait barrier)))
      (spawn (lambda () (task-2) (barrier-wait barrier)))
      (spawn (lambda () (task-3) (barrier-wait barrier))))
    (final-task)))
```

### Synchronization Primitives

```rust
use lambdust::concurrency::sync::{Mutex, RwLock, Semaphore, Barrier};

// Mutex for exclusive access
let mutex = Mutex::new(shared_data);
{
    let guard = mutex.lock().await;
    // Critical section
    guard.modify();
}

// RwLock for multiple readers
let rwlock = RwLock::new(shared_data);
let read_guard = rwlock.read().await;
let value = read_guard.get();

// Semaphore for resource control
let semaphore = Semaphore::new(3); // Allow 3 concurrent operations
let permit = semaphore.acquire().await?;
// Use resource
drop(permit); // Release resource

// Barrier for synchronization
let barrier = Barrier::new(4); // Wait for 4 tasks
barrier.wait().await;
```

## Foreign Function Interface (FFI)

### C Integration

```rust
use lambdust::ffi::{FfiRegistry, CFunction, safe_call};

// Register C function
let registry = FfiRegistry::global();
registry.register_function(
    "c_sqrt",
    CFunction::new("libm.so", "sqrt")
        .with_signature("double sqrt(double)")
        .with_safety_checks(true)
)?;

// Call from Scheme
evaluator.eval("(define sqrt (ffi-import \"c_sqrt\"))")?;
let result = evaluator.eval("(sqrt 16.0)")?;
assert_eq!(result, Value::number(4.0));
```

### Safe FFI Patterns

```rust
use lambdust::ffi::{FfiCallback, MemoryManager};

// Callback from C to Scheme
let callback = FfiCallback::new(|args: &[Value]| -> Result<Value> {
    // Safe callback implementation
    let result = scheme_function(args)?;
    Ok(result)
});

// Memory management
let memory_manager = MemoryManager::new()
    .with_auto_cleanup(true)
    .with_leak_detection(cfg!(debug_assertions));

// Safe foreign memory access
let foreign_ptr = memory_manager.allocate_tracked(size)?;
// Automatic cleanup on drop
```

## Module System

### Module Definition

```scheme
;; Define a module
(define-library (my-library math)
  (import (scheme base)
          (scheme inexact))
  (export square cube factorial)
  
  (begin
    (define (square x) (* x x))
    (define (cube x) (* x x x))
    (define (factorial n)
      (if (<= n 1)
          1
          (* n (factorial (- n 1)))))))
```

### Module Loading

```rust
use lambdust::module_system::{ModuleSystem, LibraryId};

// Load module
let module_system = ModuleSystem::new();
let library_id = LibraryId::new(vec!["my-library".to_string(), "math".to_string()]);
let library = module_system.load_library(&library_id)?;

// Access exports
let square_fn = library.get_export("square")?;
let result = evaluator.call_function(square_fn, vec![Value::number(5.0)])?;
```

### Dynamic Module Loading

```scheme
;; Runtime module loading
(import-dynamically '(srfi 1) 
  (lambda (success?)
    (if success?
        (begin
          (display "SRFI-1 loaded successfully")
          (use-list-functions))
        (error "Failed to load SRFI-1"))))

;; Conditional imports
(cond-expand
  (srfi-1 (import (srfi 1)))
  (else   (import (my-library list-utils))))
```

## Performance and Benchmarking

### Built-in Benchmarking

```rust
use lambdust::benchmarks::{BenchmarkSuite, BenchmarkConfig};

// Create benchmark suite
let suite = BenchmarkSuite::new()
    .with_config(BenchmarkConfig {
        iterations: 1000,
        warmup_iterations: 100,
        timeout: Duration::from_secs(30),
        statistical_analysis: true,
    });

// Add benchmarks
suite.add_benchmark("fibonacci", || {
    evaluator.eval("(fibonacci 30)")
})?;

suite.add_benchmark("sort", || {
    evaluator.eval("(sort (generate-random-list 1000) <)")
})?;

// Run benchmarks
let results = suite.run().await?;
println!("Results: {}", results.summary());
```

### Performance Monitoring

```scheme
;; Built-in profiling
(with-profiler
  (complex-computation input))

;; Memory usage monitoring
(define memory-before (gc-stats))
(run-memory-intensive-task)
(define memory-after (gc-stats))
(display-memory-diff memory-before memory-after)

;; Performance assertions
(define-benchmark "fast-sort"
  (lambda () (sort random-data <))
  #:max-time 100ms
  #:min-ops-per-sec 1000)
```

## Error Handling

### Diagnostic Errors

```rust
use lambdust::diagnostics::{Error, DiagnosticError, SourceSpan};

// Create diagnostic errors
let error = DiagnosticError::new(
    "Type mismatch",
    SourceSpan::new(line, column, length),
    "Expected Number, found String"
);

// Error helpers
use lambdust::diagnostics::error::helpers;

let runtime_error = helpers::runtime_error_simple("Invalid operation");
let type_error = helpers::type_error("Expected function", Some(span));
let syntax_error = helpers::syntax_error("Unexpected token", span);
```

### Exception Handling

```scheme
;; Exception handling
(define (safe-division x y)
  (guard (condition
          [(division-by-zero? condition) 
           (display "Cannot divide by zero")
           #f])
    (/ x y)))

;; Custom exception types
(define-exception-type &custom-error
  &error
  make-custom-error
  custom-error?)

(define (raise-custom-error message)
  (raise (make-custom-error message)))

;; Error recovery
(with-exception-handler
  (lambda (condition)
    (log-error condition)
    (fallback-value))
  (lambda ()
    (risky-operation)))
```

## Advanced Features

### Metaprogramming

```scheme
;; Compile-time evaluation
(define-syntax compile-time-factorial
  (syntax-rules ()
    [(_ n) (quote ,(factorial n))]))

;; Code generation
(define-syntax define-getter-setter
  (syntax-rules ()
    [(_ field)
     (begin
       (define (,(symbol-append 'get- field) obj)
         (,(symbol-append field '-ref) obj))
       (define (,(symbol-append 'set- field '!) obj value)
         (,(symbol-append field '-set!) obj value)))]))

;; Program analysis
(define (analyze-performance expr)
  (let ([ast (parse expr)])
    (analyze-complexity ast)))
```

### Reflection

```rust
use lambdust::metaprogramming::reflection::{Reflector, ObjectMetadata};

// Runtime reflection
let reflector = Reflector::new();
let metadata = reflector.inspect_object(&value)?;

println!("Type: {}", metadata.type_info());
println!("Methods: {:?}", metadata.available_methods());
println!("Fields: {:?}", metadata.field_names());

// Dynamic method invocation
let result = reflector.invoke_method(&object, "method_name", args)?;
```

## Configuration

### Runtime Configuration

```rust
use lambdust::runtime::{LambdustRuntime, RuntimeConfig};

let config = RuntimeConfig {
    stack_size: 8 * 1024 * 1024,     // 8MB stack
    heap_size: 256 * 1024 * 1024,   // 256MB heap
    gc_threshold: 0.8,                // GC at 80% heap usage
    thread_pool_size: num_cpus::get(),
    enable_jit: true,
    enable_profiling: cfg!(debug_assertions),
    r7rs_strict_mode: false,
    allow_redefinition: true,
};

let runtime = LambdustRuntime::new(config)?;
```

### Feature Flags

```toml
[dependencies.lambdust]
version = "0.1.1"
features = [
    "r7rs-large",      # R7RS-large standard library
    "gradual-typing",  # Gradual type system
    "effect-system",   # Algebraic effects
    "simd",           # SIMD optimizations
    "profiling",      # Performance profiling
    "actors",         # Actor model
    "parallel",       # Parallel evaluation
]
```

## Examples

### Complete Program

```scheme
#!/usr/bin/env lambdust

;; File: fibonacci-server.scm
;; A concurrent Fibonacci server with type annotations and effects

(import (scheme base)
        (scheme write)
        (lambdust actors)
        (lambdust effects)
        (lambdust types))

;; Typed Fibonacci with memoization
(define (fibonacci n : Number) : Number
  (with-memoization
    (cond
      [(<= n 1) n]
      [else (+ (fibonacci (- n 1))
               (fibonacci (- n 2)))])))

;; Actor for handling Fibonacci requests
(define-actor fibonacci-actor
  (state (requests-handled : Number 0))
  
  (handle (compute n : Number)
    (do [result (fibonacci n)]
        [_ (set! requests-handled (+ requests-handled 1))]
        [_ (log-info (format "Computed fibonacci(~a) = ~a" n result))]
        (return result)))
  
  (handle (stats)
    (return requests-handled)))

;; Server with effect handling
(define (run-server port : Number)
  (with-effects [Console IO Network]
    (do [server (start-tcp-server port)]
        [actor (spawn-actor fibonacci-actor)]
        [_ (log-info (format "Fibonacci server started on port ~a" port))]
        (server-loop server actor))))

(define (server-loop server actor)
  (do [request (accept-connection server)]
      [n (parse-request request)]
      [result (send-message actor (compute n))]
      [_ (send-response request result)]
      (server-loop server actor)))

;; Main entry point
(when (script-file?)
  (run-server 8080))
```

This API reference reflects the current state of Lambdust with its clean architecture, comprehensive feature set, and professional implementation quality achieved through systematic refactoring and development.