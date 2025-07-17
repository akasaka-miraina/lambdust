# Lambdust Bridge API Documentation

## Overview

The Lambdust Bridge API provides a seamless interface for integrating the Lambdust Scheme interpreter with external Rust applications. This allows applications to:

- Expose Rust functions to Scheme code
- Register external objects for manipulation from Scheme
- Convert between Rust types and Scheme values
- Execute Scheme code with access to application data and functions

## Core Components

### LambdustBridge

The main entry point for integrating Lambdust with your application.

```rust
use lambdust::LambdustBridge;

let mut bridge = LambdustBridge::new();
```

### Key Methods

#### Function Registration

```rust
// Register a simple function
bridge.register_function("add-one", Some(1), |args| {
    let n = i64::from_scheme(&args[0])?;
    (n + 1).to_scheme()
});

// Register a variadic function
bridge.register_function("sum", None, |args| {
    let mut total = 0i64;
    for arg in args {
        total += i64::from_scheme(arg)?;
    }
    total.to_scheme()
});
```

#### Object Registration

```rust
// Register an external object
let my_object = MyStruct::new();
let object_id = bridge.register_object(my_object, "MyStruct");
```

#### Variable Definition

```rust
// Define Scheme variables
bridge.define("pi", Value::from(3.14159));
bridge.define("app-name", Value::from("My Application"));
```

#### Code Evaluation

```rust
// Evaluate Scheme expressions
let result = bridge.eval("(+ 1 2 3)")?;
let result = bridge.eval("(call-external \"my-function\" 42)")?;

// Load and evaluate Scheme files
let result = bridge.load_file("script.scm")?;
```

## Type Conversion

### ToScheme and FromScheme Traits

These traits handle conversion between Rust types and Scheme values.

#### Built-in Implementations

```rust
// Rust to Scheme
let scheme_int = 42i64.to_scheme()?;        // -> Value::Number(Integer(42))
let scheme_float = 3.14f64.to_scheme()?;    // -> Value::Number(Real(3.14))
let scheme_bool = true.to_scheme()?;        // -> Value::Boolean(true)
let scheme_string = "hello".to_scheme()?;   // -> Value::String("hello")

// Scheme to Rust
let rust_int = i64::from_scheme(&scheme_value)?;
let rust_float = f64::from_scheme(&scheme_value)?;
let rust_bool = bool::from_scheme(&scheme_value)?;
let rust_string = String::from_scheme(&scheme_value)?;
```

#### Custom Type Conversion

```rust
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl ToScheme for Point {
    fn to_scheme(&self) -> Result<Value> {
        // Convert to a list: (x y)
        Ok(Value::from_vector(vec![
            self.x.to_scheme()?,
            self.y.to_scheme()?,
        ]))
    }
}

impl FromScheme for Point {
    fn from_scheme(value: &Value) -> Result<Self> {
        if let Some(vec) = value.to_vector() {
            if vec.len() == 2 {
                return Ok(Point {
                    x: f64::from_scheme(&vec[0])?,
                    y: f64::from_scheme(&vec[1])?,
                });
            }
        }
        Err(LambdustError::TypeError("Expected point as (x y)".to_string()))
    }
}
```

## Scheme Interface

### Built-in Bridge Functions

From Scheme code, you can access external functionality through these functions:

#### call-external

Call registered external functions:

```scheme
;; Call a function with arguments
(call-external "function-name" arg1 arg2 ...)

;; Examples
(call-external "square" 5)                    ; -> 25
(call-external "string-upper" "hello")        ; -> "HELLO"
(call-external "make-user" "Alice" 30 "alice@example.com")
```

#### get-property

Access properties of external objects:

```scheme
;; Get object property
(get-property object-id "property-name")

;; Example
(get-property user-id "name")                 ; -> "Alice"
```

#### set-property!

Modify properties of external objects:

```scheme
;; Set object property
(set-property! object-id "property-name" new-value)

;; Example
(set-property! user-id "age" 31)
```

## Application Integration Protocol

### 1. Basic Setup

```rust
use lambdust::{LambdustBridge, Value, Result};

fn main() -> Result<()> {
    let mut bridge = LambdustBridge::new();
    
    // Register your functions and objects here
    setup_bridge(&mut bridge);
    
    // Execute Scheme code
    let result = bridge.eval("(main-script)")?;
    
    Ok(())
}

fn setup_bridge(bridge: &mut LambdustBridge) {
    // Register functions
    bridge.register_function("app-log", Some(1), |args| {
        let message = String::from_scheme(&args[0])?;
        println!("App Log: {}", message);
        Ok(Value::Undefined)
    });
    
    // Define constants
    bridge.define("app-version", Value::from("1.0.0"));
}
```

### 2. Error Handling

```rust
// External functions should return Result<Value>
bridge.register_function("divide", Some(2), |args| {
    let a = f64::from_scheme(&args[0])?;
    let b = f64::from_scheme(&args[1])?;
    
    if b == 0.0 {
        return Err(LambdustError::DivisionByZero);
    }
    
    (a / b).to_scheme()
});
```

### 3. State Management

```rust
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct AppState {
    counter: i32,
    users: Vec<String>,
}

fn setup_stateful_bridge(bridge: &mut LambdustBridge) {
    let state = Arc::new(Mutex::new(AppState {
        counter: 0,
        users: Vec::new(),
    }));
    
    // Increment counter
    let state_clone = state.clone();
    bridge.register_function("increment", Some(0), move |_args| {
        let mut state = state_clone.lock().unwrap();
        state.counter += 1;
        state.counter.to_scheme()
    });
    
    // Add user
    let state_clone = state.clone();
    bridge.register_function("add-user", Some(1), move |args| {
        let name = String::from_scheme(&args[0])?;
        let mut state = state_clone.lock().unwrap();
        state.users.push(name);
        (state.users.len() as i64).to_scheme()
    });
}
```

### 4. Complex Object Interaction

```rust
#[derive(Debug)]
struct Database {
    connections: HashMap<String, String>,
}

impl Database {
    fn connect(&mut self, name: &str, url: &str) -> Result<()> {
        self.connections.insert(name.to_string(), url.to_string());
        Ok(())
    }
    
    fn query(&self, conn_name: &str, sql: &str) -> Result<Vec<String>> {
        // Simulate database query
        Ok(vec![format!("Result for: {}", sql)])
    }
}

// Register database operations
fn setup_database_bridge(bridge: &mut LambdustBridge, db: Arc<Mutex<Database>>) {
    let db_clone = db.clone();
    bridge.register_function("db-connect", Some(2), move |args| {
        let name = String::from_scheme(&args[0])?;
        let url = String::from_scheme(&args[1])?;
        
        let mut db = db_clone.lock().unwrap();
        db.connect(&name, &url)?;
        
        Ok(Value::Boolean(true))
    });
    
    let db_clone = db.clone();
    bridge.register_function("db-query", Some(2), move |args| {
        let conn_name = String::from_scheme(&args[0])?;
        let sql = String::from_scheme(&args[1])?;
        
        let db = db_clone.lock().unwrap();
        let results = db.query(&conn_name, &sql)?;
        
        // Convert results to Scheme list
        let scheme_results: Result<Vec<Value>> = results
            .into_iter()
            .map(|s| s.to_scheme())
            .collect();
            
        Ok(Value::from_vector(scheme_results?))
    });
}
```

## Example Scheme Scripts

### Basic Usage

```scheme
;; Define helper functions
(define (square x) (* x x))
(define (cube x) (* x x x))

;; Use external functions
(define result (call-external "factorial" 5))
(call-external "app-log" (string-append "Factorial result: " (number->string result)))

;; Work with application state
(call-external "increment")
(call-external "add-user" "Alice")
(call-external "add-user" "Bob")
```

### Database Operations

```scheme
;; Connect to database
(call-external "db-connect" "main" "sqlite:///app.db")

;; Define query helper
(define (query-users)
  (call-external "db-query" "main" "SELECT * FROM users"))

;; Process results
(define users (query-users))
(define user-count (length users))
(call-external "app-log" 
  (string-append "Found " (number->string user-count) " users"))
```

### Configuration and Control

```scheme
;; Application configuration
(define config 
  '((debug-mode #t)
    (max-connections 100)
    (timeout 30)))

;; Apply configuration
(define (apply-config cfg)
  (cond 
    ((null? cfg) #t)
    (else 
      (let ((setting (car cfg)))
        (call-external "set-config" 
          (symbol->string (car setting))
          (cadr setting))
        (apply-config (cdr cfg))))))

(apply-config config)
```

## Best Practices

### 1. Error Handling
- Always return `Result<Value>` from external functions
- Use appropriate `LambdustError` types for different error conditions
- Provide meaningful error messages

### 2. Type Safety
- Implement `ToScheme` and `FromScheme` for custom types
- Validate argument types in external functions
- Use type predicates in Scheme code when needed

### 3. Performance
- Minimize data copying between Rust and Scheme
- Use object IDs for large or complex objects
- Cache frequently accessed data

### 4. Memory Management
- External objects are reference-counted automatically
- Be careful with circular references
- Clean up resources in object destructors

### 5. Threading
- The bridge is not thread-safe by default
- Use `Arc<Mutex<>>` for shared state
- Consider using channels for async operations

## Security Considerations

- Validate all inputs from Scheme code
- Limit access to sensitive functions
- Sandbox Scheme execution if needed
- Be careful with file system and network access

## Examples

See the `examples/` directory for complete working examples:
- `bridge_example.rs` - Basic bridge usage
- `database_example.rs` - Database integration
- `gui_example.rs` - GUI application integration
- `plugin_system.rs` - Plugin architecture example