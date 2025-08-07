# List Module Rust Builtin Functions Specification

This document specifies the Rust builtin functions required to support the R7RS-compliant list module in Lambdust.

## Core R7RS List Operations

### Basic List Operations

```rust
// Core predicates and constructors
builtin:pair?(obj) -> Boolean
builtin:cons(obj1, obj2) -> Pair
builtin:car(pair) -> Value
builtin:cdr(pair) -> Value
builtin:null?(obj) -> Boolean
builtin:list?(obj) -> Boolean

// List construction and manipulation
builtin:list-from-values(values: Vec<Value>) -> List
builtin:length(list) -> Integer
builtin:append-lists(lists: Vec<List>) -> List
builtin:reverse(list) -> List
```

### List Accessors

```rust
// List access and mutation
builtin:list-ref(list, k: Integer) -> Value
builtin:list-set!(list, k: Integer, obj) -> Unspecified
builtin:list-tail(list, k: Integer) -> List
```

### List Construction

```rust
// List creation utilities
builtin:make-list(k: Integer, fill: Value) -> List
builtin:list-copy(obj) -> Value
```

### List Searching and Association

```rust
// Membership testing
builtin:memq(obj, list) -> List | Boolean
builtin:memv(obj, list) -> List | Boolean
builtin:member(obj, list, compare: Procedure) -> List | Boolean

// Association list operations
builtin:assq(obj, alist) -> Pair | Boolean
builtin:assv(obj, alist) -> Pair | Boolean
builtin:assoc(obj, alist, compare: Procedure) -> Pair | Boolean
```

### Higher-Order Functions

```rust
// Functional programming primitives
builtin:map(proc: Procedure, list) -> List
builtin:map-multi(proc: Procedure, lists: Vec<List>) -> List
builtin:for-each(proc: Procedure, list) -> Unspecified
builtin:for-each-multi(proc: Procedure, lists: Vec<List>) -> Unspecified
```

## Extension Functions

### List Generation

```rust
// Arithmetic sequence generation
builtin:iota(count: Integer, start: Integer, step: Integer) -> List
```

### Extended Predicates

```rust
// Advanced list type checking
builtin:proper-list?(obj) -> Boolean
builtin:circular-list?(obj) -> Boolean
```

### Sorting and Merging

```rust
// Sorting operations
builtin:sort(less?: Procedure, list) -> List
builtin:merge(less?: Procedure, list1, list2) -> List
```

### Duplicate Removal

```rust
// Uniqueness operations
builtin:remove-duplicates(list, equal?: Procedure) -> List
```

## Implementation Requirements

### Error Handling

All builtin functions should provide proper error handling:

- Type checking for arguments
- Bounds checking for list operations
- Proper error messages with source location information
- Graceful handling of edge cases (empty lists, circular lists, etc.)

### Performance Considerations

#### Memory Management
- Efficient allocation and deallocation of list structures
- Sharing of immutable list segments where possible
- Tail-call optimization for recursive operations

#### Algorithmic Efficiency
- `length`: O(n) with memoization where beneficial
- `append`: O(n) for the total length of all but the last list
- `reverse`: O(n) with tail-call optimization
- `list-ref`: O(n) with bounds checking
- `sort`: O(n log n) stable sort (preferably merge sort)
- `member`/`assoc` family: O(n) with early termination

### Thread Safety

All list operations should be thread-safe:
- Immutable operations can be safely called concurrently
- Mutable operations (`list-set!`) require proper synchronization
- Reference counting or garbage collection integration

### Integration Points

#### Value System Integration

```rust
// Required Value enum variants
enum Value {
    Nil,
    Pair(Box<Value>, Box<Value>),
    // ... other variants
}

impl Value {
    fn is_nil(&self) -> bool;
    fn is_pair(&self) -> bool;
    fn as_list(&self) -> Option<Vec<Value>>;
    fn list(elements: Vec<Value>) -> Value;
    fn pair(car: Value, cdr: Value) -> Value;
}
```

#### Procedure Call Integration

```rust
// For higher-order functions
trait Callable {
    fn call(&self, args: &[Value], env: &Environment) -> Result<Value>;
}

// Integration with evaluator for procedure calls
fn call_procedure(proc: &Value, args: &[Value], env: &Environment) -> Result<Value>;
```

#### Error Reporting Integration

```rust
// Error types for list operations
#[derive(Debug)]
enum ListError {
    IndexOutOfBounds { index: i64, length: usize },
    ImproperList { operation: String },
    TypeError { expected: String, actual: String },
    EmptyList { operation: String },
}

impl From<ListError> for DiagnosticError {
    fn from(err: ListError) -> Self;
}
```

## Testing Requirements

### Unit Tests

Each builtin function should have comprehensive unit tests covering:
- Normal operation cases
- Edge cases (empty lists, single elements)
- Error conditions
- Performance characteristics for large lists

### Integration Tests

- Interaction between different list operations
- Memory usage patterns
- Thread safety verification
- Performance benchmarks

### R7RS Compliance Tests

- All R7RS Section 6.4 procedures must pass compliance tests
- Behavior must match R7RS specification exactly
- Error conditions must be handled as specified

## Example Implementation Signatures

```rust
// In src/stdlib/lists.rs

pub fn builtin_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(ListError::ArityError { 
            expected: 1, 
            actual: args.len() 
        }.into());
    }
    Ok(Value::boolean(args[0].is_pair()))
}

pub fn builtin_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(ListError::ArityError { 
            expected: 2, 
            actual: args.len() 
        }.into());
    }
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

pub fn builtin_list_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(ListError::ArityError { 
            expected: 2, 
            actual: args.len() 
        }.into());
    }
    
    let list = &args[0];
    let index = args[1].as_integer()
        .ok_or(ListError::TypeError { 
            expected: "integer".to_string(),
            actual: args[1].type_name().to_string()
        })?;
    
    if index < 0 {
        return Err(ListError::IndexOutOfBounds { 
            index, 
            length: 0 
        }.into());
    }
    
    // Implementation continues...
}
```

## Performance Benchmarks

The implementation should meet these performance targets:

- `cons`: O(1) - constant time
- `car`/`cdr`: O(1) - constant time  
- `length`: O(n) - linear with list length
- `append`: O(m) where m is total length of all but last argument
- `reverse`: O(n) - linear with tail-call optimization
- `list-ref`: O(k) where k is the index
- `member`: O(n) - linear search with early termination
- `sort`: O(n log n) - efficient stable sort
- `map`: O(n) - linear with procedure call overhead

Memory usage should be proportional to the size of the data structures created, with minimal overhead for bookkeeping.