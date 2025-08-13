# Lambdust Effect System Guide

This document provides comprehensive documentation of Lambdust's effect system, which enables pure functional programming while preserving Scheme's dynamic nature through transparent effect tracking and monadic abstractions.

## Table of Contents

1. [Effect System Overview](#effect-system-overview)
2. [Core Effect Types](#core-effect-types)
3. [Monadic Programming](#monadic-programming)
4. [Effect Handlers](#effect-handlers)
5. [Automatic Effect Lifting](#automatic-effect-lifting)
6. [Generational Environments](#generational-environments)
7. [Effect Integration](#effect-integration)
8. [Advanced Patterns](#advanced-patterns)

## Effect System Overview

Lambdust's effect system provides transparent tracking and management of computational effects while maintaining full Scheme compatibility:

### **Effect System Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Pure Functions │    │  Effect Context │    │ Effect Handlers │
│                 │    │                 │    │                 │
│ • Referential   │    │ • Active Effects│    │ • IO Handler    │
│   Transparency  │    │ • Effect Stack  │    │ • State Handler │
│ • Optimization  │    │ • Generation    │    │ • Error Handler │
│ • Reasoning     │    │ • Coordination  │    │ • Custom Effects│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                        ┌─────────────────┐
                        │  Effect Lifting │
                        │                 │
                        │ • Automatic     │
                        │ • Configurable  │
                        │ • Transparent   │
                        └─────────────────┘
                                  │
                        ┌─────────────────┐
                        │   Monads &      │
                        │  Transformers   │
                        │                 │
                        │ • IO Monad      │
                        │ • State Monad   │
                        │ • Error Monad   │
                        │ • Continuation  │
                        └─────────────────┘
```

### **Key Features**

- **Transparent Effect Tracking**: Effects are inferred and tracked automatically
- **Monadic Abstractions**: Full monadic programming support with do-notation
- **Effect Handlers**: Pluggable effect management with custom handlers
- **Generational Environments**: Functional handling of state mutations
- **Zero-Cost Abstractions**: Effects compile to efficient code
- **Scheme Compatibility**: Works seamlessly with existing Scheme code

## Core Effect Types

### **Built-in Effect Types** (`src/effects/mod.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Pure computation (no effects)
    Pure,
    /// IO effects (input/output operations)  
    IO,
    /// State effects (mutations that create new generations)
    State,
    /// Error effects (exceptions and error handling)
    Error,
    /// Custom effects with a name
    Custom(String),
}
```

### **Effect Classification in Practice**

#### **Pure Functions**
```scheme
;; Explicitly marked as pure
(define (square x)
  #:pure #t
  (* x x))

;; Pure by inference (no side effects)
(define (factorial n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

;; Higher-order pure functions
(define (compose f g)
  #:pure #t
  (lambda (x) (f (g x))))
```

#### **IO Effects**
```scheme
;; IO effects are automatically detected
(define (greet name)
  (display "Hello, ")  ; IO effect
  (display name)       ; IO effect
  (newline))          ; IO effect

;; File operations
(define (read-config filename)
  (call-with-input-file filename
    (lambda (port)
      (read port))))   ; IO effect

;; Network operations
(define (fetch-data url)
  (http-get url))      ; IO effect
```

#### **State Effects**
```scheme
;; Mutation creates state effects
(define (make-counter)
  (let ((count 0))
    (lambda ()
      (set! count (+ count 1))  ; State effect
      count)))

;; Vector mutations
(define (shuffle! vec)
  (do ((i (- (vector-length vec) 1) (- i 1)))
      ((< i 1) vec)
    (let ((j (random (+ i 1))))
      (vector-swap! vec i j))))   ; State effect
```

#### **Error Effects**
```scheme
;; Error handling creates error effects
(define (safe-divide x y)
  (if (= y 0)
      (error "Division by zero")  ; Error effect
      (/ x y)))

;; Exception handling
(define (safe-operation)
  (guard (ex [else (display "Error occurred")])  ; Error effect
    (risky-computation)))
```

## Monadic Programming

### **Core Monads** (`src/effects/builtin_monads.rs`)

#### **IO Monad**
```rust
pub struct IO<T> {
    computation: Box<dyn FnOnce() -> Result<T> + Send>,
}

impl<T> IO<T> {
    pub fn pure(value: T) -> IO<T> {
        IO {
            computation: Box::new(move || Ok(value)),
        }
    }
    
    pub fn bind<U, F>(self, f: F) -> IO<U>
    where
        F: FnOnce(T) -> IO<U> + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
    {
        IO {
            computation: Box::new(move || {
                let result = (self.computation)()?;
                (f(result).computation)()
            }),
        }
    }
    
    pub fn run(self) -> Result<T> {
        (self.computation)()
    }
}
```

#### **State Monad**
```rust
pub struct State<S, T> {
    computation: Box<dyn FnOnce(S) -> (T, S) + Send>,
}

impl<S, T> State<S, T> {
    pub fn pure(value: T) -> State<S, T> {
        State {
            computation: Box::new(move |state| (value, state)),
        }
    }
    
    pub fn get() -> State<S, S>
    where
        S: Clone + Send + 'static,
    {
        State {
            computation: Box::new(|state| {
                let result = state.clone();
                (result, state)
            }),
        }
    }
    
    pub fn put(new_state: S) -> State<S, ()> {
        State {
            computation: Box::new(move |_| ((), new_state)),
        }
    }
}
```

### **Monadic Programming in Scheme**

#### **Do-Notation for Monads**
```scheme
;; IO monad with do-notation
(define (interactive-greeting)
  (do-io
    [name <- (io-read-line "Enter your name: ")]
    [age <- (io-read-line "Enter your age: ")]
    (io-print (string-append "Hello " name ", you are " age " years old!"))))

;; State monad for stateful computations
(define (stateful-counter)
  (do-state
    [current <- state-get]
    [_ <- (state-put (+ current 1))]
    [new <- state-get]
    (state-return new)))

;; Error monad for safe computations
(define (safe-computation x y)
  (do-either
    [a <- (safe-sqrt x)]
    [b <- (safe-sqrt y)]  
    [result <- (safe-divide a b)]
    (either-return result)))
```

#### **Monadic Combinators**
```scheme
;; Sequence operations
(define (sequence-io actions)
  (fold-m io-bind (io-return '()) actions))

;; Map with effects
(define (map-io f lst)
  (if (null? lst)
      (io-return '())
      (do-io
        [x <- (f (car lst))]
        [xs <- (map-io f (cdr lst))]
        (io-return (cons x xs)))))

;; Filter with effects
(define (filter-io pred lst)
  (if (null? lst)
      (io-return '())
      (do-io
        [keep? <- (pred (car lst))]
        [rest <- (filter-io pred (cdr lst))]
        (io-return (if keep? 
                       (cons (car lst) rest)
                       rest)))))
```

### **Monad Transformers**

```scheme
;; IO + State transformer
(define-monad-transformer StateT
  (lambda (m)
    (lambda (s)
      (do m
        [(value . new-state) <- computation]
        (m-return (cons value new-state))))))

;; Error + IO transformer  
(define-monad-transformer ErrorT
  (lambda (m)
    (do m
      [result <- computation]
      (case result
        [(Left error) (m-return (Left error))]
        [(Right value) (m-return (Right value))]))))

;; Using transformer stack
(define (complex-computation)
  (run-state-t
    (run-error-t
      (do-state-error
        [x <- (lift-io (read-number "Enter x: "))]
        [y <- (lift-state (get-counter))]
        [result <- (safe-divide x y)]
        (return result)))
    initial-state))
```

## Effect Handlers

### **Effect Handler Interface** (`src/effects/handler.rs`)

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
    /// Effect was handled successfully with a value
    Value(Value),
    /// Effect was handled and should continue with another computation
    Continue(Value),
    /// Effect was not handled by this handler
    Unhandled,
    /// Effect handling resulted in an error
    Error(DiagnosticError),
}
```

### **Built-in Effect Handlers**

#### **IO Effect Handler**
```scheme
;; IO operations are handled by the IO handler
(define (file-operations)
  ;; These operations use the IO handler automatically
  (let ((content (call-with-input-file "config.txt" 
                   (lambda (port) (read-all port)))))
    (call-with-output-file "output.txt"
      (lambda (port) 
        (write (process-config content) port)))))
```

#### **State Effect Handler**
```scheme
;; State mutations are handled transparently
(define (counter-demo)
  (let ((count 0))
    (define (increment!)
      (set! count (+ count 1))  ; Creates new generation
      count)
    
    (define (get-count) count)
    
    ;; Both functions work in the new generation
    (values (increment!) (increment!) (get-count))))
```

### **Custom Effect Handlers**

#### **Logging Effect Handler**
```scheme
;; Define custom logging effect
(define-effect-handler logging-handler
  #:effect 'log
  #:handle (lambda (args)
             (let ((level (car args))
                   (message (cadr args)))
               (printf "[~a] ~a~n" level message)
               (unspecified))))

;; Use custom effect
(define (logged-computation x)
  (effect 'log 'info "Starting computation")
  (let ((result (* x x)))
    (effect 'log 'info (format "Result: ~a" result))
    result))
```

#### **Database Effect Handler**
```scheme
;; Database connection effect
(define-effect-handler db-handler
  #:effect 'database
  #:state (make-db-connection "localhost" 5432)
  #:handle (lambda (operation args state)
             (match operation
               ['query (db-query (car args) state)]
               ['insert (db-insert (car args) (cadr args) state)]
               ['update (db-update (car args) (cadr args) state)]
               ['delete (db-delete (car args) state)])))

;; Use database effect
(define (user-management)
  (effect 'database 'query "SELECT * FROM users")
  (effect 'database 'insert "users" '((name "Alice") (email "alice@example.com")))
  (effect 'database 'query "SELECT COUNT(*) FROM users"))
```

#### **Async Effect Handler**  
```scheme
;; Async computation effect
(define-effect-handler async-handler
  #:effect 'async
  #:handle (lambda (args)
             (let ((computation (car args)))
               (spawn-async
                 (lambda ()
                   (computation))))))

;; Parallel computations with async effect
(define (parallel-work)
  (let ((future1 (effect 'async (lambda () (expensive-computation-1))))
        (future2 (effect 'async (lambda () (expensive-computation-2))))
        (future3 (effect 'async (lambda () (expensive-computation-3)))))
    
    ;; Wait for all results
    (list (await future1) (await future2) (await future3))))
```

## Automatic Effect Lifting

### **Effect Lifting Configuration** (`src/effects/lifting_config.rs`)

```rust
pub struct LiftingConfig {
    pub auto_lift_io: bool,
    pub auto_lift_state: bool,
    pub auto_lift_error: bool,
    pub custom_rules: Vec<LiftingRule>,
}

pub struct LiftingRule {
    pub condition: LiftingCondition,
    pub transformation: LiftingTransformation,
    pub target_monad: String,
}

pub enum LiftingCondition {
    Always,
    OperationName(String),
    HasEffect(Vec<Effect>),
    Custom(fn(&str, &[Effect]) -> bool),
}
```

### **Automatic Effect Inference**

```scheme
;; Effects are automatically inferred and lifted
(define (process-file filename)
  ;; No explicit effect annotations needed
  (let ((content (read-file filename)))        ; IO effect inferred
    (let ((processed (string-uppercase content))) ; Pure
      (write-file "output.txt" processed))))    ; IO effect inferred

;; Function automatically has IO effect type
;; Inferred: filename -> IO ()

;; Mixed effect functions are lifted appropriately  
(define (stateful-io-operation)
  (let ((count 0))                    ; State
    (lambda ()
      (set! count (+ count 1))        ; State effect
      (display count)                 ; IO effect  
      (newline))))                    ; IO effect

;; Inferred type: () -> State (IO ())
```

### **Effect Lifting Rules**

```scheme
;; Configure automatic lifting
(configure-effect-lifting
  #:auto-lift-io #t
  #:auto-lift-state #t  
  #:rules
  (list
    ;; Lift arithmetic operations in IO context
    (lifting-rule
      #:condition (operation-name? '+ '- '* '/)
      #:in-context 'io
      #:lift-to 'io)
    
    ;; Lift list operations to preserve effects
    (lifting-rule  
      #:condition (operation-name? 'map 'filter 'fold)
      #:preserve-effects #t
      #:lift-to 'auto)))
```

## Generational Environments

### **Environment Generations** (`src/effects/generational.rs`)

The effect system handles mutations through generational environments:

```rust
pub struct GenerationalEnvironment {
    frames: Vec<Frame>,
    generation: Generation,
    parent: Option<Arc<GenerationalEnvironment>>,
    mutations: HashMap<String, Value>,
}

impl GenerationalEnvironment {
    /// Creates a new generation when mutations occur
    pub fn mutate(&mut self, name: String, value: Value) -> Generation {
        let new_generation = self.generation + 1;
        
        // Create new generation with mutation
        let mut new_env = self.clone();
        new_env.generation = new_generation;
        new_env.mutations.insert(name, value);
        
        new_generation
    }
    
    /// Lookup considers generation-specific mutations
    pub fn lookup(&self, name: &str) -> Option<Value> {
        // Check mutations in this generation first
        if let Some(value) = self.mutations.get(name) {
            Some(value.clone())
        } else {
            // Fall back to parent environments
            self.parent.as_ref().and_then(|parent| parent.lookup(name))
        }
    }
}
```

### **Generational Programming Patterns**

#### **Functional State Management**
```scheme
;; Each mutation creates a new generation
(define (demo-generations)
  (let ((x 10))
    (display x)        ; 10 (generation 0)
    
    (set! x 20)        ; Creates generation 1
    (display x)        ; 20 (generation 1)
    
    (let ((y 30))
      (set! x 40)      ; Creates generation 2
      (set! y 50)      ; Creates generation 3
      (display x)      ; 40 (generation 2+)
      (display y))     ; 50 (generation 3)
    
    (display x)))      ; 40 (mutations preserved)
```

#### **Transaction-like Behavior**
```scheme
;; Generations enable transaction-like rollback
(define (transactional-update data)
  (call-with-generation-checkpoint
    (lambda ()
      (set-car! data (process-first (car data)))
      (set-cdr! data (process-rest (cdr data)))
      ;; If any operation fails, entire generation is rolled back
      (validate-data data))
    
    ;; Rollback handler
    (lambda (error)
      (display "Transaction failed, rolling back")
      #f)))
```

#### **Time-Travel Debugging**
```scheme
;; Debug by examining different generations
(define (debug-generations)
  (let ((history '()))
    (define (checkpoint name)
      (set! history (cons (cons name (current-generation)) history)))
    
    (let ((x 1))
      (checkpoint "initial")
      
      (set! x (* x 2))
      (checkpoint "doubled")
      
      (set! x (+ x 5))  
      (checkpoint "added-5")
      
      ;; Examine history
      (for-each 
        (lambda (entry)
          (printf "~a: generation ~a~n" (car entry) (cdr entry)))
        (reverse history)))))
```

## Effect Integration

### **Effect System Coordination** (`src/runtime/effect_coordinator.rs`)

The effect coordinator manages the interaction between different effect systems:

```rust
pub struct EffectCoordinator {
    active_effects: Arc<RwLock<Vec<Effect>>>,
    effect_handlers: Arc<RwLock<HashMap<Effect, Box<dyn EffectHandler>>>>,
    effect_statistics: Arc<EffectStatistics>,
    isolation_manager: IsolationManager,
}

impl EffectCoordinator {
    /// Coordinates effect execution across the system
    pub async fn execute_with_effects<F, R>(&self, computation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
        R: Send,
    {
        let effect_context = EffectContext::new();
        
        // Set up effect tracking
        self.push_context(effect_context.clone()).await;
        
        // Execute computation with effect coordination
        let result = tokio::task::spawn_blocking(computation).await?;
        
        // Clean up effect tracking
        self.pop_context().await;
        
        result
    }
}
```

### **Multi-threaded Effect Coordination**

```scheme
;; Effects work across thread boundaries
(define (parallel-effects)
  (parallel-do
    ;; Each thread has its own effect context
    (thread-1
      (lambda ()
        (display "Thread 1: ")     ; IO effect
        (let ((x 10))
          (set! x 20)              ; State effect in thread 1
          (display x))))
    
    (thread-2  
      (lambda ()
        (display "Thread 2: ")     ; IO effect
        (let ((y 30))
          (set! y 40)              ; State effect in thread 2  
          (display y))))))

;; Effect isolation between threads
(define (isolated-effects)
  (let ((shared-state (make-shared-state 0)))
    (parallel-map
      (lambda (thread-id)
        ;; Each thread sees isolated view of effects
        (atomic-update! shared-state
          (lambda (current)
            (+ current thread-id))))  ; State effect is atomic
      '(1 2 3 4 5))))
```

## Advanced Patterns

### **Effect Polymorphism**

```scheme
;; Functions polymorphic in their effects
(define (generic-map f lst)
  #:type (∀ a b e. (a -e-> b) -> List a -e-> List b)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (generic-map f (cdr lst)))))

;; Works with any effect
(define pure-square (lambda (x) (* x x)))           ; Pure
(define io-display (lambda (x) (display x) x))      ; IO  
(define state-count (lambda (x) (increment-counter!) x)) ; State

;; All these work with generic-map
(generic-map pure-square '(1 2 3))      ; Pure computation
(generic-map io-display '(1 2 3))       ; IO computation
(generic-map state-count '(1 2 3))      ; Stateful computation
```

### **Effect Composition**

```scheme
;; Compose effects in function composition
(define (compose-effects f g)
  #:type (∀ a b c e1 e2. (b -e1-> c) -> (a -e2-> b) -> (a -e1,e2-> c))
  (lambda (x)
    (let ((intermediate (g x)))  ; Effect e2
      (f intermediate))))        ; Effect e1

;; Effect composition is automatic
(define read-and-display
  (compose-effects 
    (lambda (content) (display content) content)  ; IO
    (lambda (filename) (read-file filename))))    ; IO

;; Composed function has IO effect
```

### **Algebraic Effects**

```scheme
;; Define algebraic effects with handlers
(define-algebraic-effect Choice
  (choose : Boolean))

;; Handler for non-deterministic choice
(define choice-handler-all
  (effect-handler Choice
    [(choose) '(#t #f)]  ; Return all possibilities
    [(return x) (list x)]))

;; Handler for first choice
(define choice-handler-first  
  (effect-handler Choice
    [(choose) #t]        ; Always choose first option
    [(return x) x]))

;; Non-deterministic computation
(define (nondeterministic-calc)
  (let ((x (if (perform choose) 10 20))
        (y (if (perform choose) 5 15)))
    (+ x y)))

;; Different handlers give different results
(with-handler choice-handler-all nondeterministic-calc)    ; (15 25 25 35)
(with-handler choice-handler-first nondeterministic-calc)  ; 15
```

### **Effect-Aware Optimizations**

```scheme
;; Pure functions can be optimized aggressively
(define (pure-computation x y)
  #:pure #t
  #:memoize #t          ; Safe to memoize pure functions
  (expensive-calculation x y))

;; IO functions are not memoized
(define (io-computation x)
  (display "Computing...")  ; IO effect prevents memoization
  (expensive-calculation x))

;; Effect-directed parallelization
(define (auto-parallel lst)
  ;; Pure operations can be parallelized automatically
  (if (all-pure? (map operation lst))
      (parallel-map operation lst)    ; Safe parallelization
      (map operation lst)))           ; Sequential for effects
```

### **Effect Debugging and Monitoring**

```scheme
;; Debug effects with tracing
(define (traced-computation)
  (with-effect-tracing #t
    (lambda ()
      (let ((x (read-number)))        ; IO: traced
        (set! global-counter x)       ; State: traced
        (/ x 0)))))                   ; Error: traced

;; Effect profiling
(define (profiled-computation)
  (with-effect-profiling
    (lambda ()
      (parallel-map expensive-io-operation large-dataset))
    
    ;; Profile report shows:
    ;; IO operations: 1000 calls, 500ms total
    ;; State operations: 50 calls, 10ms total
    ;; Pure operations: 10000 calls, 200ms total
    ))
```

---

This effect system guide provides a comprehensive foundation for understanding and effectively using Lambdust's sophisticated effect system. The combination of automatic effect inference, powerful monadic abstractions, and transparent effect handling enables developers to write pure functional programs while maintaining the expressiveness and compatibility of Scheme.