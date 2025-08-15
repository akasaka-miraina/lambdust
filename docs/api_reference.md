# Dependent Types API Reference

## Overview

This document provides a comprehensive API reference for Lambdust's dependent type system. The API is designed for developers who want to integrate dependent types into their Scheme programs or extend the type system itself.

## Core Types and Structures

### `DependentType` Enum

The fundamental type representation in Lambdust's dependent type system.

```rust
pub enum DependentType {
    Universe(usize),
    Pi { parameter_name: String, parameter_type: Box<DependentType>, return_type: Box<DependentType> },
    Sigma { first_name: String, first_type: Box<DependentType>, second_type: Box<DependentType> },
    Identity { type_expr: Box<DependentType>, left: Box<DependentType>, right: Box<DependentType> },
    Inductive { name: String, parameters: Vec<DependentType>, universe_level: usize, constructors: Vec<(String, DependentType)>, induction_principle: Option<Box<DependentType>> },
    Variable { name: String, de_bruijn_index: usize },
    Application { function: Box<DependentType>, argument: Box<DependentType> },
    Lambda { parameter_name: String, parameter_type: Box<DependentType>, body: Box<DependentType> },
}
```

#### Constructors

##### `DependentType::universe(level: usize) -> DependentType`

Creates a universe type at the specified level.

```rust
let type_0 = DependentType::universe(0);  // Type₀
let type_1 = DependentType::universe(1);  // Type₁
```

##### `DependentType::pi(param_name: String, param_type: DependentType, return_type: DependentType) -> DependentType`

Creates a Π-type (dependent function type).

```rust
let nat_type = DependentType::universe(0);
let vec_type = DependentType::pi(
    "n".to_string(),
    nat_type.clone(),
    DependentType::application(
        DependentType::variable("Vector"),
        vec![DependentType::variable("Integer"), DependentType::variable("n")]
    )
);
```

##### `DependentType::sigma(first_name: String, first_type: DependentType, second_type: DependentType) -> DependentType`

Creates a Σ-type (dependent pair type).

```rust
let dependent_pair = DependentType::sigma(
    "x".to_string(),
    DependentType::universe(0),
    DependentType::application(
        DependentType::variable("Vector"), 
        vec![DependentType::variable("Integer"), DependentType::variable("x")]
    )
);
```

##### `DependentType::identity(type_expr: DependentType, left: DependentType, right: DependentType) -> DependentType`

Creates an identity type expressing equality between two terms.

```rust
let equality = DependentType::identity(
    DependentType::universe(0),
    DependentType::variable("x"),
    DependentType::variable("y")
);
```

#### Methods

##### `free_variables(&self) -> HashSet<String>`

Returns the set of free variables in the type expression.

```rust
let pi_type = DependentType::pi("x".to_string(), nat_type, body_with_x_and_y);
let free_vars = pi_type.free_variables();  // Returns {"y"}
```

##### `substitute(&self, var_name: &str, replacement: &DependentType) -> DependentType`

Performs capture-avoiding substitution with automatic α-conversion.

```rust
let original = DependentType::pi("x".to_string(), DependentType::variable("A"), DependentType::variable("x"));
let substituted = original.substitute("A", &DependentType::universe(0));
```

##### `normalize(&self, context: &TypeContext) -> Result<DependentType>`

Normalizes the type to weak head normal form.

```rust
let normalized = complex_type.normalize(&context)?;
```

##### `pretty_print(&self) -> String`

Returns a human-readable representation of the type.

```rust
println!("Type: {}", my_type.pretty_print());
// Output: (Pi (x : Nat) (Vector Integer x))
```

---

## Type Checking

### `TypeChecker` Struct

The main interface for type checking and inference.

```rust
pub struct TypeChecker {
    context: TypeContext,
    equality_checker: DefinitionalEquality,
    constraint_solver: ConstraintSolver,
}
```

#### Constructor

##### `TypeChecker::new() -> TypeChecker`

Creates a new type checker with an empty context.

```rust
let mut checker = TypeChecker::new();
```

#### Methods

##### `check_type(&mut self, expr: &DependentType, expected_type: Option<&DependentType>, context: &TypeContext) -> Result<DependentType>`

Main entry point for type checking. Supports both inference and checking modes.

```rust
// Type inference
let inferred_type = checker.check_type(&expression, None, &context)?;

// Type checking  
let checked_type = checker.check_type(&expression, Some(&expected), &context)?;
```

##### `infer_type(&mut self, expr: &DependentType, context: &TypeContext) -> Result<DependentType>`

Infers the type of an expression.

```rust
let lambda_expr = DependentType::lambda("x", nat_type, body);
let function_type = checker.infer_type(&lambda_expr, &context)?;
```

##### `check_against_type(&mut self, expr: &DependentType, expected: &DependentType, context: &TypeContext) -> Result<DependentType>`

Checks that an expression has the expected type.

```rust
checker.check_against_type(&expression, &pi_type, &context)?;
```

##### `check_definitional_equality(&mut self, type1: &DependentType, type2: &DependentType, context: &TypeContext) -> Result<()>`

Checks if two types are definitionally equal.

```rust
checker.check_definitional_equality(&inferred, &expected, &context)?;
```

---

## Type Context

### `TypeContext` Struct

Manages variable bindings and type information.

```rust
pub struct TypeContext {
    bindings: HashMap<String, DependentType>,
    definitions: HashMap<String, DependentType>,
    universe_constraints: Vec<UniverseConstraint>,
}
```

#### Constructor

##### `TypeContext::new() -> TypeContext`

Creates an empty type context.

```rust
let context = TypeContext::new();
```

#### Methods

##### `extend(&self, name: String, type_expr: DependentType) -> TypeContext`

Creates a new context with an additional binding.

```rust
let extended = context.extend("x".to_string(), nat_type);
```

##### `lookup_type(&self, name: &str) -> Option<DependentType>`

Looks up the type of a variable.

```rust
if let Some(var_type) = context.lookup_type("x") {
    println!("Variable x has type: {}", var_type.pretty_print());
}
```

##### `add_definition(&mut self, name: String, definition: DependentType)`

Adds a type definition to the context.

```rust
context.add_definition("Vector".to_string(), vector_type_constructor);
```

##### `lookup_definition(&self, name: &str) -> Option<&DependentType>`

Looks up a type definition.

```rust
if let Some(def) = context.lookup_definition("Vector") {
    // Use the definition
}
```

---

## Scheme Integration

### `SchemeIntegration` Struct

Provides integration between R7RS Scheme values and dependent types.

```rust
pub struct SchemeIntegration {
    value_type_mapping: HashMap<String, DependentType>,
    type_cache: HashMap<String, DependentType>,
    primitive_registry: Arc<MinimalPrimitiveRegistry>,
    universe_levels: HashMap<String, usize>,
}
```

#### Constructor

##### `SchemeIntegration::new() -> Result<SchemeIntegration>`

Creates a new Scheme integration instance.

```rust
let integration = SchemeIntegration::new()?;
```

#### Methods

##### `value_to_type(&mut self, value: &Value) -> Result<DependentType>`

Converts an R7RS Scheme value to a dependent type.

```rust
let scheme_value = Value::exact_integer(42);
let dependent_type = integration.value_to_type(&scheme_value)?;
```

##### `type_to_value(&self, dep_type: &DependentType) -> Result<Value>`

Converts a dependent type back to an R7RS value (type erasure).

```rust
let scheme_value = integration.type_to_value(&dependent_type)?;
```

##### `infer_expression_type(&mut self, expr: &Expr, env: &Environment) -> Result<DependentType>`

Infers the dependent type of a Scheme expression.

```rust
let expr = Expr::literal(Literal::ExactInteger(42));
let dep_type = integration.infer_expression_type(&expr, &env)?;
```

##### `is_value_compatible(&mut self, value: &Value, expected_type: &DependentType) -> Result<bool>`

Checks if a Scheme value is compatible with a dependent type.

```rust
let compatible = integration.is_value_compatible(&value, &expected_type)?;
```

---

## Gradual Typing

### `GradualType` Enum

Represents the four levels of typing in Lambdust.

```rust
pub enum GradualType {
    Dynamic,
    Contract(ContractSpec),
    Static(StaticType),
    Dependent(DependentType),
}
```

#### Methods

##### `migrate_to(&self, target_level: TypingLevel) -> Result<GradualType>`

Migrates from one typing level to another.

```rust
let dynamic_type = GradualType::Dynamic;
let contract_type = dynamic_type.migrate_to(TypingLevel::Contract)?;
let static_type = contract_type.migrate_to(TypingLevel::Static)?;
let dependent_type = static_type.migrate_to(TypingLevel::Dependent)?;
```

##### `is_compatible_with(&self, other: &GradualType) -> bool`

Checks compatibility across typing levels.

```rust
let compatible = type1.is_compatible_with(&type2);
```

### `TypingLevel` Enum

Represents the four typing levels.

```rust
pub enum TypingLevel {
    Dynamic,
    Contract,
    Static,
    Dependent,
}
```

---

## Memory Management

### `DependentTypeArena` Struct

Provides efficient memory allocation for dependent types.

```rust
pub struct DependentTypeArena {
    arena: bumpalo::Bump,
    allocation_stats: AllocationStats,
}
```

#### Constructor

##### `DependentTypeArena::new() -> DependentTypeArena`

Creates a new arena allocator.

```rust
let arena = DependentTypeArena::new();
```

#### Methods

##### `allocate_type<'a>(&'a self, type_expr: DependentType) -> &'a DependentType`

Allocates a type in the arena.

```rust
let allocated_type = arena.allocate_type(my_type);
```

##### `bulk_allocate<'a>(&'a self, types: Vec<DependentType>) -> Vec<&'a DependentType>`

Efficiently allocates multiple types.

```rust
let allocated_types = arena.bulk_allocate(type_vec);
```

##### `reset(&mut self)`

Resets the arena, deallocating all memory.

```rust
arena.reset();  // All previously allocated types become invalid
```

---

## Error Types

### `TypeError` Struct

Represents type checking errors with detailed information.

```rust
pub struct TypeError {
    pub message: String,
    pub span: Span,
    pub error_code: &'static str,
    pub help: Option<&'static str>,
    pub labels: Vec<ErrorLabel>,
}
```

#### Common Error Constructors

##### `TypeError::universe_inconsistency(expected: usize, actual: usize, span: Span) -> TypeError`

Universe level mismatch error.

##### `TypeError::substitution_failure(var_name: &str, expected_type: &DependentType, span: Span) -> TypeError`

Variable substitution error.

##### `TypeError::type_mismatch(expected: &DependentType, actual: &DependentType, span: Span) -> TypeError`

General type mismatch error.

---

## Constraint Solving

### `ConstraintSolver` Struct

Solves type constraints for dependent type inference.

```rust
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    solutions: HashMap<String, DependentType>,
}
```

#### Methods

##### `add_constraint(&mut self, constraint: Constraint)`

Adds a constraint to the solver.

```rust
solver.add_constraint(Constraint::equality(type1, type2));
```

##### `solve(&mut self) -> Result<Solution>`

Solves all constraints and returns a solution.

```rust
let solution = solver.solve()?;
```

##### `solve_parallel(&mut self) -> Result<Solution>`

Solves constraints using parallel processing.

```rust
let solution = solver.solve_parallel()?;  // Uses SIMD and threading
```

---

## Identity Types and J-Eliminator

### `JEliminator` Struct

Implements the J-eliminator for identity types.

```rust
pub struct JEliminator {
    motive_cache: HashMap<DependentType, DependentType>,
}
```

#### Methods

##### `eliminate(&mut self, motive: &DependentType, base_case: &DependentType, target_type: &DependentType, left: &DependentType, right: &DependentType, proof: &DependentType) -> Result<DependentType>`

Performs J-elimination (path induction).

```rust
let result = j_eliminator.eliminate(
    &motive,
    &base_case, 
    &target_type,
    &left_term,
    &right_term,
    &equality_proof
)?;
```

---

## Universe Hierarchy

### `UniverseHierarchy` Struct

Manages the universe hierarchy and prevents paradoxes.

```rust
pub struct UniverseHierarchy {
    levels: HashMap<DependentType, usize>,
    constraints: Vec<UniverseConstraint>,
}
```

#### Methods

##### `assign_level(&mut self, type_expr: &DependentType) -> Result<usize>`

Assigns a universe level to a type.

```rust
let level = hierarchy.assign_level(&my_type)?;
```

##### `check_consistency(&self) -> Result<()>`

Checks that the universe hierarchy is consistent.

```rust
hierarchy.check_consistency()?;
```

---

## Performance Optimization

### Caching

Most type operations support caching for improved performance:

```rust
// Type checking with caching
let cached_result = checker.check_type_cached(&expr, &expected, &context)?;

// Definitional equality with memoization
let equal = equality_checker.are_equal_memoized(&type1, &type2, &context)?;
```

### Parallel Processing

Several operations support parallel execution:

```rust
// Parallel constraint solving
let solution = solver.solve_constraints_parallel(&constraints)?;

// Parallel normalization
let normalized_types = normalizer.normalize_batch_parallel(&types, &context)?;
```

### SIMD Operations

SIMD instructions are used for vector operations:

```rust
// SIMD constraint evaluation
let results = solver.evaluate_constraints_simd(&constraint_batch)?;
```

---

## Usage Examples

### Basic Type Checking

```rust
use lambdust::types::dependent::*;

// Create a type checker
let mut checker = TypeChecker::new();
let mut context = TypeContext::new();

// Add basic types to context
context.add_definition("Nat".to_string(), DependentType::universe(0));
context.add_definition("Vector".to_string(), 
    DependentType::pi("A".to_string(), DependentType::universe(0),
        DependentType::pi("n".to_string(), DependentType::variable("Nat"),
            DependentType::universe(0))));

// Create a dependent function type
let vec_length_fn = DependentType::pi("n".to_string(), DependentType::variable("Nat"),
    DependentType::pi("v".to_string(), 
        DependentType::application(
            DependentType::variable("Vector"),
            vec![DependentType::variable("Integer"), DependentType::variable("n")]
        ),
        DependentType::variable("Nat")
    ));

// Type check the function
let checked_type = checker.check_type(&vec_length_fn, None, &context)?;
println!("Type: {}", checked_type.pretty_print());
```

### Scheme Integration

```rust
use lambdust::eval::Value;
use lambdust::types::dependent::SchemeIntegration;

// Create integration bridge
let mut integration = SchemeIntegration::new()?;

// Convert Scheme values to dependent types
let scheme_number = Value::exact_integer(42);
let dependent_type = integration.value_to_type(&scheme_number)?;

println!("Scheme value {} has dependent type {}", 
    scheme_number, dependent_type.pretty_print());
```

### Gradual Typing Migration

```rust
use lambdust::types::dependent::{GradualType, TypingLevel};

// Start with dynamic typing
let dynamic = GradualType::Dynamic;

// Gradually add more type information
let with_contracts = dynamic.migrate_to(TypingLevel::Contract)?;
let static_typed = with_contracts.migrate_to(TypingLevel::Static)?;
let dependent_typed = static_typed.migrate_to(TypingLevel::Dependent)?;

// Check compatibility
assert!(dynamic.is_compatible_with(&dependent_typed));
```

---

## Error Handling

All API functions return `Result<T, Error>` where `Error` provides detailed diagnostic information:

```rust
match checker.check_type(&expr, Some(&expected), &context) {
    Ok(type_result) => println!("Type check succeeded: {}", type_result.pretty_print()),
    Err(error) => {
        eprintln!("Type error: {}", error);
        for label in error.labels() {
            eprintln!("  {}: {}", label.span(), label.message());
        }
        if let Some(help) = error.help() {
            eprintln!("Help: {}", help);
        }
    }
}
```

---

## Thread Safety

The API is designed to be thread-safe where appropriate:

- `TypeContext` is immutable and can be shared across threads
- `TypeChecker` must be used within a single thread but can be cheaply cloned
- `SchemeIntegration` uses interior mutability for caching but is thread-safe
- `DependentTypeArena` is not thread-safe and should be used within a single thread

For multi-threaded usage:

```rust
use std::sync::Arc;

let context = Arc::new(TypeContext::new());
let context_clone = Arc::clone(&context);

// Use in different threads
std::thread::spawn(move || {
    let mut checker = TypeChecker::new();
    checker.check_type(&expr, None, &context_clone);
});
```