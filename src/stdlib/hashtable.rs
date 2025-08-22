//! SRFI-125: Hash Tables Implementation
//!
//! This module provides a SRFI-125 compliant implementation of hash tables.
//! It builds upon the existing high-performance hash table implementation
//! in the containers module.
//!
//! SRFI-125 Specification includes:
//! - Hash table constructors and predicates
//! - Accessors and mutators
//! - The whole hash table (searching, folding, mapping)
//! - Hash table conversion
//! - Hash table copying

use crate::containers::{HashComparator, ThreadSafeHashTable};
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Creates SRFI-125 hash table bindings for the standard library.
pub fn create_hashtable_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Constructors
    bind_constructors(env);

    // Predicates
    bind_predicates(env);

    // Accessors
    bind_accessors(env);

    // Mutators
    bind_mutators(env);

    // The whole hash table
    bind_whole_table_operations(env);

    // Conversion
    bind_conversion_operations(env);

    // Copying
    bind_copying_operations(env);
}

/// Binds hash table constructor procedures
fn bind_constructors(env: &Arc<ThreadSafeEnvironment>) {
    // make-hash-table
    env.define(
        "make-hash-table".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-hash-table".to_string(),
            arity_min: 0,
            arity_max: Some(3), // comparator?, capacity?, rest
            implementation: PrimitiveImpl::RustFn(primitive_make_hash_table),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table
    env.define(
        "hash-table".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table".to_string(),
            arity_min: 0,
            arity_max: None, // Variadic (comparator key1 value1 key2 value2 ...)
            implementation: PrimitiveImpl::RustFn(primitive_hash_table),
            effects: vec![Effect::Pure],
        })),
    );

    // alist->hash-table
    env.define(
        "alist->hash-table".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "alist->hash-table".to_string(),
            arity_min: 1,
            arity_max: Some(2), // alist, comparator?
            implementation: PrimitiveImpl::RustFn(primitive_alist_to_hash_table),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Binds hash table predicate procedures
fn bind_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table?
    env.define(
        "hash-table?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_p),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-contains?
    env.define(
        "hash-table-contains?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-contains?".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_contains),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-empty?
    env.define(
        "hash-table-empty?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-empty?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_empty),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table=?
    env.define(
        "hash-table=?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table=?".to_string(),
            arity_min: 2,
            arity_max: None, // Variadic
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_equal),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-mutable?
    env.define(
        "hash-table-mutable?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-mutable?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_mutable),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Binds hash table accessor procedures
fn bind_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table-ref
    env.define(
        "hash-table-ref".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-ref".to_string(),
            arity_min: 2,
            arity_max: Some(3), // hash-table, key, failure?
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_ref),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-ref/default
    env.define(
        "hash-table-ref/default".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-ref/default".to_string(),
            arity_min: 3,
            arity_max: Some(3), // hash-table, key, default
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_ref_default),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Binds hash table mutator procedures
fn bind_mutators(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table-set!
    env.define(
        "hash-table-set!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-set!".to_string(),
            arity_min: 1,
            arity_max: None, // hash-table key1 value1 key2 value2 ...
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_set),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-delete!
    env.define(
        "hash-table-delete!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-delete!".to_string(),
            arity_min: 2,
            arity_max: None, // hash-table key1 key2 ...
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_delete),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-intern!
    env.define(
        "hash-table-intern!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-intern!".to_string(),
            arity_min: 3,
            arity_max: Some(3), // hash-table, key, failure
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_intern),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-update!
    env.define(
        "hash-table-update!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-update!".to_string(),
            arity_min: 3,
            arity_max: Some(4), // hash-table, key, updater, failure?
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_update),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-update!/default
    env.define(
        "hash-table-update!/default".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-update!/default".to_string(),
            arity_min: 4,
            arity_max: Some(4), // hash-table, key, updater, default
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_update_default),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-pop!
    env.define(
        "hash-table-pop!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-pop!".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_pop),
            effects: vec![Effect::Mutation],
        })),
    );

    // hash-table-clear!
    env.define(
        "hash-table-clear!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-clear!".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_clear),
            effects: vec![Effect::Mutation],
        })),
    );
}

/// Binds whole hash table operation procedures
fn bind_whole_table_operations(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table-size
    env.define(
        "hash-table-size".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-size".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_size),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-keys
    env.define(
        "hash-table-keys".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-keys".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_keys),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-values
    env.define(
        "hash-table-values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-values".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_values),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-entries
    env.define(
        "hash-table-entries".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-entries".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_entries),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-find
    env.define(
        "hash-table-find".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-find".to_string(),
            arity_min: 3,
            arity_max: Some(3), // hash-table, predicate, failure
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_find),
            effects: vec![Effect::IO], // Predicate may have side effects
        })),
    );

    // hash-table-count
    env.define(
        "hash-table-count".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-count".to_string(),
            arity_min: 2,
            arity_max: Some(2), // hash-table, predicate
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_count),
            effects: vec![Effect::IO], // Predicate may have side effects
        })),
    );
}

/// Binds hash table conversion procedures
fn bind_conversion_operations(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table->alist
    env.define(
        "hash-table->alist".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table->alist".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_to_alist),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Binds hash table copying procedures
fn bind_copying_operations(env: &Arc<ThreadSafeEnvironment>) {
    // hash-table-copy
    env.define(
        "hash-table-copy".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-copy".to_string(),
            arity_min: 1,
            arity_max: Some(2), // hash-table, mutable?
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_copy),
            effects: vec![Effect::Pure],
        })),
    );

    // hash-table-empty-copy
    env.define(
        "hash-table-empty-copy".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "hash-table-empty-copy".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_hash_table_empty_copy),
            effects: vec![Effect::Pure],
        })),
    );
}

// ============= CONSTRUCTOR IMPLEMENTATIONS =============

/// `make-hash-table` primitive
fn primitive_make_hash_table(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => {
            // Default hash table with default comparator
            let hash_table = ThreadSafeHashTable::new();
            Ok(Value::AdvancedHashTable(Arc::new(hash_table)))
        }
        1 => {
            // Hash table with custom comparator
            // TODO: Extract comparator from args[0]
            let hash_table = ThreadSafeHashTable::new();
            Ok(Value::AdvancedHashTable(Arc::new(hash_table)))
        }
        2 => {
            // Hash table with comparator and capacity
            // TODO: Extract comparator and capacity
            let hash_table = ThreadSafeHashTable::new();
            Ok(Value::AdvancedHashTable(Arc::new(hash_table)))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "make-hash-table: expected 0-3 arguments, got {}",
                args.len()
            ),
            None,
        ))),
    }
}

/// `hash-table` primitive
fn primitive_hash_table(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "hash-table: expected at least 1 argument (comparator)".to_string(),
            None,
        )));
    }

    // First argument should be comparator
    // Remaining arguments are key-value pairs
    if (args.len() - 1) % 2 != 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "hash-table: key-value pairs must be even in number".to_string(),
            None,
        )));
    }

    let hash_table = ThreadSafeHashTable::new();

    // Insert key-value pairs
    for chunk in args[1..].chunks(2) {
        if chunk.len() == 2 {
            hash_table.insert(chunk[0].clone(), chunk[1].clone());
        }
    }

    Ok(Value::AdvancedHashTable(Arc::new(hash_table)))
}

/// `alist->hash-table` primitive
fn primitive_alist_to_hash_table(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "alist->hash-table: expected 1-2 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    let alist = &args[0];
    let hash_table = ThreadSafeHashTable::new();

    // Convert alist to hash table
    let mut current = alist;
    while !matches!(current, Value::Nil) {
        match current {
            Value::Pair(car, cdr) => {
                // Each element should be a pair (key . value)
                match car.as_ref() {
                    Value::Pair(key, value) => {
                        hash_table.insert(key.as_ref().clone(), value.as_ref().clone());
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "alist->hash-table: alist elements must be pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr.as_ref();
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "alist->hash-table: argument must be a proper list".to_string(),
                    None,
                )));
            }
        }
    }

    Ok(Value::AdvancedHashTable(Arc::new(hash_table)))
}

// ============= PREDICATE IMPLEMENTATIONS =============

/// `hash-table?` primitive
fn primitive_hash_table_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(matches!(
        args[0],
        Value::AdvancedHashTable(_)
    )))
}

/// `hash-table-contains?` primitive
fn primitive_hash_table_contains(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-contains?: expected 2 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let contains = hash_table.contains_key(&args[1]);
            Ok(Value::boolean(contains))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-contains?: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-empty?` primitive
fn primitive_hash_table_empty(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-empty?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => Ok(Value::boolean(hash_table.is_empty())),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-empty?: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table=?` primitive
fn primitive_hash_table_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "hash-table=?: expected at least 2 arguments".to_string(),
            None,
        )));
    }

    // For simplicity, use reference equality
    // In a full implementation, this would compare contents
    for i in 1..args.len() {
        if !matches!((&args[0], &args[i]),
                    (Value::AdvancedHashTable(a), Value::AdvancedHashTable(b)) if Arc::ptr_eq(a, b))
        {
            return Ok(Value::boolean(false));
        }
    }

    Ok(Value::boolean(true))
}

/// `hash-table-mutable?` primitive
fn primitive_hash_table_mutable(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-mutable?: expected 1 argument, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(_) => {
            // All our hash tables are mutable
            Ok(Value::boolean(true))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-mutable?: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

// ============= ACCESSOR IMPLEMENTATIONS =============

/// `hash-table-ref` primitive
fn primitive_hash_table_ref(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-ref: expected 2-3 arguments, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            match hash_table.get(&args[1]) {
                Some(value) => Ok(value),
                None => {
                    if args.len() == 3 {
                        // Return failure value
                        Ok(args[2].clone())
                    } else {
                        Err(Box::new(DiagnosticError::runtime_error(
                            "hash-table-ref: key not found".to_string(),
                            None,
                        )))
                    }
                }
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-ref: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-ref/default` primitive
fn primitive_hash_table_ref_default(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-ref/default: expected 3 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => match hash_table.get(&args[1]) {
            Some(value) => Ok(value),
            None => Ok(args[2].clone()),
        },
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-ref/default: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

// ============= MUTATOR IMPLEMENTATIONS =============

/// `hash-table-set!` primitive
fn primitive_hash_table_set(args: &[Value]) -> Result<Value> {
    if args.is_empty() || (args.len() - 1) % 2 != 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-set!: requires hash table followed by even number of key-value pairs"
                .to_string(),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            // Set each key-value pair
            for chunk in args[1..].chunks(2) {
                if chunk.len() == 2 {
                    hash_table.insert(chunk[0].clone(), chunk[1].clone());
                }
            }
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-set!: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-delete!` primitive
fn primitive_hash_table_delete(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-delete!: requires hash table and at least one key".to_string(),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let mut count = 0;
            for key in &args[1..] {
                if hash_table.remove(key).is_some() {
                    count += 1;
                }
            }
            Ok(Value::integer(count))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-delete!: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-intern!` primitive
fn primitive_hash_table_intern(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-intern!: expected 3 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let key = &args[1];
            match hash_table.get(key) {
                Some(value) => Ok(value),
                None => {
                    let new_value = args[2].clone();
                    hash_table.insert(key.clone(), new_value.clone());
                    Ok(new_value)
                }
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-intern!: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-update!` primitive - simplified implementation
fn primitive_hash_table_update(_args: &[Value]) -> Result<Value> {
    // This requires calling user-provided updater function
    // For now, return an error indicating it needs evaluator integration
    Err(Box::new(DiagnosticError::runtime_error(
        "hash-table-update!: not yet implemented (requires evaluator integration)".to_string(),
        None,
    )))
}

/// `hash-table-update!/default` primitive - simplified implementation
fn primitive_hash_table_update_default(_args: &[Value]) -> Result<Value> {
    // This requires calling user-provided updater function
    // For now, return an error indicating it needs evaluator integration
    Err(Box::new(DiagnosticError::runtime_error(
        "hash-table-update!/default: not yet implemented (requires evaluator integration)"
            .to_string(),
        None,
    )))
}

/// `hash-table-pop!` primitive
fn primitive_hash_table_pop(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-pop!: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            // Get arbitrary key-value pair and remove it
            let entries = hash_table.entries();
            if let Some((key, value)) = entries.first() {
                let key = key.clone();
                let value = value.clone();
                hash_table.remove(&key);
                // Return multiple values (key, value)
                Ok(Value::list(vec![key, value]))
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "hash-table-pop!: hash table is empty".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-pop!: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-clear!` primitive
fn primitive_hash_table_clear(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-clear!: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            hash_table.clear();
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-clear!: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

// ============= WHOLE TABLE OPERATION IMPLEMENTATIONS =============

/// `hash-table-size` primitive
fn primitive_hash_table_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-size: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => Ok(Value::integer(hash_table.len() as i64)),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-size: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-keys` primitive
fn primitive_hash_table_keys(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-keys: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let keys = hash_table.keys();
            Ok(Value::list(keys))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-keys: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-values` primitive
fn primitive_hash_table_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table-values: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let values = hash_table.values();
            Ok(Value::list(values))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-values: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-entries` primitive
fn primitive_hash_table_entries(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-entries: expected 1 argument, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let entries = hash_table.entries();
            let keys: Vec<Value> = entries.iter().map(|(k, _)| k.clone()).collect();
            let values: Vec<Value> = entries.iter().map(|(_, v)| v.clone()).collect();

            // Return two values: keys and values
            Ok(Value::list(vec![Value::list(keys), Value::list(values)]))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-entries: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-find` primitive - simplified implementation
fn primitive_hash_table_find(_args: &[Value]) -> Result<Value> {
    // This requires calling user-provided predicate function
    // For now, return an error indicating it needs evaluator integration
    Err(Box::new(DiagnosticError::runtime_error(
        "hash-table-find: not yet implemented (requires evaluator integration)".to_string(),
        None,
    )))
}

/// `hash-table-count` primitive - simplified implementation
fn primitive_hash_table_count(_args: &[Value]) -> Result<Value> {
    // This requires calling user-provided predicate function
    // For now, return an error indicating it needs evaluator integration
    Err(Box::new(DiagnosticError::runtime_error(
        "hash-table-count: not yet implemented (requires evaluator integration)".to_string(),
        None,
    )))
}

// ============= CONVERSION IMPLEMENTATIONS =============

/// `hash-table->alist` primitive
fn primitive_hash_table_to_alist(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("hash-table->alist: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let entries = hash_table.entries();
            let pairs: Vec<Value> = entries
                .into_iter()
                .map(|(k, v)| Value::Pair(Box::new(k), Box::new(v)))
                .collect();
            Ok(Value::list(pairs))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table->alist: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

// ============= COPYING IMPLEMENTATIONS =============

/// `hash-table-copy` primitive
fn primitive_hash_table_copy(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-copy: expected 1-2 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(hash_table) => {
            let new_table = ThreadSafeHashTable::new();
            // Copy all entries
            for (key, value) in hash_table.entries() {
                new_table.insert(key, value);
            }
            Ok(Value::AdvancedHashTable(Arc::new(new_table)))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-copy: first argument must be a hash table".to_string(),
            None,
        ))),
    }
}

/// `hash-table-empty-copy` primitive
fn primitive_hash_table_empty_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "hash-table-empty-copy: expected 1 argument, got {}",
                args.len()
            ),
            None,
        )));
    }

    match &args[0] {
        Value::AdvancedHashTable(_) => {
            // Create new empty hash table with same properties
            let new_table = ThreadSafeHashTable::new();
            Ok(Value::AdvancedHashTable(Arc::new(new_table)))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "hash-table-empty-copy: argument must be a hash table".to_string(),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_table_predicate() {
        let hash_table = ThreadSafeHashTable::new();
        let hash_table_value = Value::AdvancedHashTable(Arc::new(hash_table));

        let result = primitive_hash_table_p(&[hash_table_value]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let non_hash_table = Value::integer(42);
        let result = primitive_hash_table_p(&[non_hash_table]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_make_hash_table() {
        let result = primitive_make_hash_table(&[]).unwrap();
        assert!(matches!(result, Value::AdvancedHashTable(_)));
    }

    #[test]
    fn test_hash_table_operations() {
        let hash_table = ThreadSafeHashTable::new();
        let hash_table_value = Value::AdvancedHashTable(Arc::new(hash_table));

        // Test empty
        let result = primitive_hash_table_empty(&[hash_table_value.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test set
        let set_args = vec![
            hash_table_value.clone(),
            Value::string("key"),
            Value::integer(42),
        ];
        primitive_hash_table_set(&set_args).unwrap();

        // Test not empty
        let result = primitive_hash_table_empty(&[hash_table_value.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test contains
        let contains_args = vec![hash_table_value.clone(), Value::string("key")];
        let result = primitive_hash_table_contains(&contains_args).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test ref
        let ref_args = vec![hash_table_value.clone(), Value::string("key")];
        let result = primitive_hash_table_ref(&ref_args).unwrap();
        assert_eq!(result, Value::integer(42));

        // Test size
        let result = primitive_hash_table_size(&[hash_table_value.clone()]).unwrap();
        assert_eq!(result, Value::integer(1));
    }
}
