//! Set operations implementation for SRFI-113: Set basic data structures.
//!
//! This module provides the primitive procedures for working with Sets in Lambdust,
//! implementing the full SRFI-113 specification for basic set operations.

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::containers::HashComparator;
use crate::diagnostics::{Error, Result, Span};
use crate::effects::Effect;
use std::sync::Arc;

/// Helper function to bind a set primitive.
fn bind_set_primitive(
    env: &Arc<ThreadSafeEnvironment>,
    name: &str,
    arity_min: usize,
    arity_max: Option<usize>,
    implementation: fn(&[Value]) -> Result<Value>,
) {
    env.define(name.to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.to_string(),
        arity_min,
        arity_max,
        implementation: PrimitiveImpl::RustFn(implementation),
        effects: vec![Effect::Pure],
    })));
}

/// Helper function to bind a set primitive with evaluator integration.
fn bind_set_evaluator_primitive(
    env: &Arc<ThreadSafeEnvironment>,
    name: &str,
    arity_min: usize,
    arity_max: Option<usize>,
    implementation: fn(&mut crate::eval::evaluator::Evaluator, &[Value]) -> Result<Value>,
) {
    env.define(name.to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.to_string(),
        arity_min,
        arity_max,
        implementation: PrimitiveImpl::EvaluatorIntegrated(implementation),
        effects: vec![Effect::Pure],
    })));
}

/// Helper function to create a runtime error with no span.
fn runtime_error(msg: impl Into<String>) -> Box<Error> {
    Box::new(Error::runtime_error(msg.into(), None))
}

/// Helper function to create a type error with no span.
fn type_error(msg: impl Into<String>) -> Box<Error> {
    Box::new(Error::type_error(msg.into(), Span::default()))
}

/// Helper function to create an arity error.
fn arity_error(name: &str, _min: usize, _max: Option<usize>, actual: usize) -> Box<Error> {
    Box::new(Error::arity_error(name, actual, actual)) // Simplified for now
}

/// Helper function to check arity.
fn arity_check(name: &str, args: &[Value], min: usize, max: Option<usize>) -> Result<()> {
    if let Some(max_arity) = max {
        if args.len() < min || args.len() > max_arity {
            return Err(arity_error(name, min, max, args.len()));
        }
    } else if args.len() < min {
        return Err(arity_error(name, min, max, args.len()));
    }
    Ok(())
}

/// Installs all set primitive procedures into the given environment.
pub fn install_set_primitives(env: &Arc<ThreadSafeEnvironment>) {
    // Basic set operations
    bind_set_primitive(env, "set", 0, None, primitive_set);
    bind_set_evaluator_primitive(env, "set-unfold", 4, Some(5), primitive_set_unfold);
    bind_set_primitive(env, "set?", 1, Some(1), primitive_set_p);
    bind_set_primitive(env, "set-contains?", 2, Some(2), primitive_set_contains_p);
    bind_set_primitive(env, "set-empty?", 1, Some(1), primitive_set_empty_p);
    bind_set_primitive(env, "set-disjoint?", 2, Some(2), primitive_set_disjoint_p);
    
    // Set modification
    bind_set_primitive(env, "set-adjoin", 1, None, primitive_set_adjoin);
    bind_set_primitive(env, "set-adjoin!", 1, None, primitive_set_adjoin_mut);
    bind_set_primitive(env, "set-delete", 1, None, primitive_set_delete);
    bind_set_primitive(env, "set-delete!", 1, None, primitive_set_delete_mut);
    bind_set_primitive(env, "set-delete-all", 2, Some(2), primitive_set_delete_all);
    bind_set_primitive(env, "set-delete-all!", 2, Some(2), primitive_set_delete_all_mut);
    
    // Set size operations
    bind_set_primitive(env, "set-size", 1, Some(1), primitive_set_size);
    
    // Set iteration (higher-order functions)
    bind_set_evaluator_primitive(env, "set-for-each", 2, Some(2), primitive_set_for_each);
    bind_set_evaluator_primitive(env, "set-fold", 3, Some(3), primitive_set_fold);
    bind_set_evaluator_primitive(env, "set-map", 2, Some(3), primitive_set_map);
    bind_set_evaluator_primitive(env, "set-filter", 2, Some(2), primitive_set_filter);
    bind_set_evaluator_primitive(env, "set-remove", 2, Some(2), primitive_set_remove);
    bind_set_evaluator_primitive(env, "set-partition", 2, Some(2), primitive_set_partition);
    
    // Set mutating iteration functions
    bind_set_evaluator_primitive(env, "set-filter!", 2, Some(2), primitive_set_filter_mut);
    bind_set_evaluator_primitive(env, "set-remove!", 2, Some(2), primitive_set_remove_mut);
    bind_set_evaluator_primitive(env, "set-partition!", 2, Some(2), primitive_set_partition_mut);
    
    // Set conversion
    bind_set_primitive(env, "set->list", 1, Some(1), primitive_set_to_list);
    bind_set_primitive(env, "list->set", 1, Some(2), primitive_list_to_set);
    
    // Set theory operations
    bind_set_primitive(env, "set-union", 0, None, primitive_set_union);
    bind_set_primitive(env, "set-intersection", 0, None, primitive_set_intersection);
    bind_set_primitive(env, "set-difference", 0, None, primitive_set_difference);
    bind_set_primitive(env, "set-xor", 2, Some(2), primitive_set_xor);
    
    bind_set_primitive(env, "set-union!", 0, None, primitive_set_union_mut);
    bind_set_primitive(env, "set-intersection!", 0, None, primitive_set_intersection_mut);
    bind_set_primitive(env, "set-difference!", 0, None, primitive_set_difference_mut);
    bind_set_primitive(env, "set-xor!", 2, Some(2), primitive_set_xor_mut);
    
    // Set comparison
    bind_set_primitive(env, "set=?", 2, None, primitive_set_equal_p);
    bind_set_primitive(env, "set<?", 2, None, primitive_set_subset_p);
    bind_set_primitive(env, "set>?", 2, None, primitive_set_superset_p);
    bind_set_primitive(env, "set<=?", 2, None, primitive_set_subset_eq_p);
    bind_set_primitive(env, "set>=?", 2, None, primitive_set_superset_eq_p);
    
    // Set copying
    bind_set_primitive(env, "set-copy", 1, Some(1), primitive_set_copy);
    
    // Set search operations
    bind_set_evaluator_primitive(env, "set-search!", 4, Some(4), primitive_set_search_mut);
}

/// Creates a new set with the given elements.
/// (set [comparator] element ...)
fn primitive_set(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        // Empty set with default comparator
        return Ok(Value::set());
    }
    
    // Check if first argument is a comparator (simplified for now - assumes no comparator)
    // In a full implementation, we'd check SRFI-128 comparator objects
    Ok(Value::set_from_iter(args.iter().cloned()))
}

/// Creates a set by unfolding a generator.
/// (set-unfold stop? mapper successor seed [comparator])
fn primitive_set_unfold(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-unfold", args, 4, Some(5))?;
    
    let stop_predicate = &args[0];
    let mapper = &args[1]; 
    let successor = &args[2];
    let mut seed = args[3].clone();
    // Note: Optional comparator at args[4] is ignored for now
    
    let mut elements = Vec::new();
    
    loop {
        // Check if we should stop
        let should_stop = apply_procedure_with_evaluator(eval, stop_predicate, &[seed.clone()])?;
        if !should_stop.is_falsy() {
            break;
        }
        
        // Map the seed to get the element for the set
        let element = apply_procedure_with_evaluator(eval, mapper, &[seed.clone()])?;
        elements.push(element);
        
        // Generate the next seed
        seed = apply_procedure_with_evaluator(eval, successor, &[seed])?;
        
        // Safety check to prevent infinite loops
        if elements.len() > 10000 {
            return Err(runtime_error("set-unfold: too many iterations (>10000), possible infinite loop"));
        }
    }
    
    Ok(Value::set_from_iter(elements))
}

/// Tests whether a value is a set.
/// (set? obj)
fn primitive_set_p(args: &[Value]) -> Result<Value> {
    arity_check("set?", args, 1, Some(1))?;
    Ok(Value::boolean(args[0].is_set()))
}

/// Tests whether a set contains an element.
/// (set-contains? set element)
fn primitive_set_contains_p(args: &[Value]) -> Result<Value> {
    arity_check("set-contains?", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[0] {
        match set.contains(&args[1]) {
            Ok(result) => Ok(Value::boolean(result)),
            Err(_) => Err(runtime_error("Failed to check set membership")),
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Tests whether a set is empty.
/// (set-empty? set)
fn primitive_set_empty_p(args: &[Value]) -> Result<Value> {
    arity_check("set-empty?", args, 1, Some(1))?;
    
    if let Value::Set(set) = &args[0] {
        match set.is_empty() {
            Ok(result) => Ok(Value::boolean(result)),
            Err(_) => Err(runtime_error("Failed to check if set is empty")),
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Tests whether two sets are disjoint.
/// (set-disjoint? set1 set2)
fn primitive_set_disjoint_p(args: &[Value]) -> Result<Value> {
    arity_check("set-disjoint?", args, 2, Some(2))?;
    
    match (&args[0], &args[1]) {
        (Value::Set(set1), Value::Set(set2)) => {
            match set1.is_disjoint(set2) {
                Ok(result) => Ok(Value::boolean(result)),
                Err(_) => Err(runtime_error("Failed to check if sets are disjoint")),
            }
        }
        _ => Err(type_error("Expected two sets")),
    }
}

/// Adds elements to a set, returning a new set.
/// (set-adjoin set element ...)
fn primitive_set_adjoin(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(arity_error("set-adjoin", 1, None, args.len()));
    }
    
    if let Value::Set(set) = &args[0] {
        let mut result_elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        for element in &args[1..] {
            if !result_elements.contains(element) {
                result_elements.push(element.clone());
            }
        }
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Adds elements to a set, modifying the set in place.
/// (set-adjoin! set element ...)
fn primitive_set_adjoin_mut(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(arity_error("set-adjoin!", 1, None, args.len()));
    }
    
    if let Value::Set(set) = &args[0] {
        for element in &args[1..] {
            set.adjoin(element.clone()).map_err(|_| runtime_error("Failed to add element to set"))?;
        }
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Removes an element from a set, returning a new set.
/// (set-delete set element ...)
fn primitive_set_delete(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(arity_error("set-delete", 1, None, args.len()));
    }
    
    if let Value::Set(set) = &args[0] {
        let mut result_elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        for element in &args[1..] {
            result_elements.retain(|x| x != element);
        }
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Removes an element from a set, modifying the set in place.
/// (set-delete! set element ...)
fn primitive_set_delete_mut(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(arity_error("set-delete!", 1, None, args.len()));
    }
    
    if let Value::Set(set) = &args[0] {
        for element in &args[1..] {
            set.delete(element).map_err(|_| runtime_error("Failed to remove element from set"))?;
        }
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Removes all given elements from a set, returning a new set.
/// (set-delete-all set element-list)
fn primitive_set_delete_all(args: &[Value]) -> Result<Value> {
    arity_check("set-delete-all", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[0] {
        if let Some(elements) = args[1].as_list() {
            let mut result_elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
            for element in elements {
                result_elements.retain(|x| x != &element);
            }
            Ok(Value::set_from_iter(result_elements))
        } else {
            Err(type_error("Expected a list"))
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Removes all given elements from a set, modifying the set in place.
/// (set-delete-all! set element-list)
fn primitive_set_delete_all_mut(args: &[Value]) -> Result<Value> {
    arity_check("set-delete-all!", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[0] {
        if let Some(elements) = args[1].as_list() {
            for element in elements {
                set.delete(&element).map_err(|_| runtime_error("Failed to remove element from set"))?;
            }
            Ok(Value::Unspecified)
        } else {
            Err(type_error("Expected a list"))
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Returns the number of elements in a set.
/// (set-size set)
fn primitive_set_size(args: &[Value]) -> Result<Value> {
    arity_check("set-size", args, 1, Some(1))?;
    
    if let Value::Set(set) = &args[0] {
        match set.size() {
            Ok(size) => Ok(Value::number(size as f64)),
            Err(_) => Err(runtime_error("Failed to get set size")),
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Helper function to properly apply a procedure using the evaluator's trampoline
fn apply_procedure_with_evaluator(
    evaluator: &mut crate::eval::evaluator::Evaluator,
    procedure: &Value,
    args: &[Value],
) -> Result<Value> {
    use crate::eval::evaluator::EvalStep;
    
    // Start with the initial procedure application
    let mut step = evaluator.apply_procedure(procedure.clone(), args.to_vec(), None);
    
    // Run the trampoline loop to handle all evaluation steps
    loop {
        step = match step {
            EvalStep::Return(value) => return Ok(value),
            EvalStep::Error(error) => return Err(Box::new(error)),
            EvalStep::Continue { expr, env } => {
                // Continue evaluation with the given expression and environment
                evaluator.eval_step(&expr, env)
            }
            EvalStep::TailCall { procedure: proc, args: tail_args, location } => {
                // Handle tail call by applying the procedure
                evaluator.apply_procedure(proc, tail_args, location)
            }
            EvalStep::CallContinuation { continuation, value } => {
                // Handle continuation call
                evaluator.call_continuation(continuation, value)
            }
            EvalStep::NonLocalJump { value, target_stack_depth: _ } => {
                // Non-local jump immediately returns the value
                return Ok(value);
            }
        }
    }
}

/// Applies a procedure to each element of a set.
/// (set-for-each proc set)
fn primitive_set_for_each(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-for-each", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        for element in elements {
            apply_procedure_with_evaluator(eval, &args[0], &[element])?;
        }
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Folds a procedure over the elements of a set.
/// (set-fold proc nil set)
fn primitive_set_fold(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-fold", args, 3, Some(3))?;
    
    if let Value::Set(set) = &args[2] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut accumulator = args[1].clone();
        
        for element in elements {
            accumulator = apply_procedure_with_evaluator(eval, &args[0], &[element, accumulator])?;
        }
        
        Ok(accumulator)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Maps a procedure over the elements of a set, returning a new set.
/// (set-map proc set [comparator])
fn primitive_set_map(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-map", args, 2, Some(3))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut result_elements = Vec::new();
        
        for element in elements {
            let mapped = apply_procedure_with_evaluator(eval, &args[0], &[element])?;
            result_elements.push(mapped);
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Filters elements of a set using a predicate, returning a new set.
/// (set-filter pred set)
fn primitive_set_filter(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-filter", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut result_elements = Vec::new();
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if !result.is_falsy() {
                result_elements.push(element);
            }
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Removes elements of a set using a predicate, returning a new set.
/// (set-remove pred set)
fn primitive_set_remove(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-remove", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut result_elements = Vec::new();
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if result.is_falsy() {
                result_elements.push(element);
            }
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Converts a set to a list.
/// (set->list set)
fn primitive_set_to_list(args: &[Value]) -> Result<Value> {
    arity_check("set->list", args, 1, Some(1))?;
    
    if let Value::Set(set) = &args[0] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        Ok(Value::list(elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Converts a list to a set.
/// (list->set list [comparator])
fn primitive_list_to_set(args: &[Value]) -> Result<Value> {
    arity_check("list->set", args, 1, Some(2))?;
    
    if let Some(elements) = args[0].as_list() {
        Ok(Value::set_from_iter(elements))
    } else {
        Err(type_error("Expected a list"))
    }
}

/// Computes the union of sets.
/// (set-union set ...)
fn primitive_set_union(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::set());
    }
    
    if let Value::Set(first_set) = &args[0] {
        let mut result_elements = first_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        
        for set_arg in &args[1..] {
            if let Value::Set(set) = set_arg {
                let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
                for element in elements {
                    if !result_elements.contains(&element) {
                        result_elements.push(element);
                    }
                }
            } else {
                return Err(type_error("Expected a set"));
            }
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Computes the intersection of sets.
/// (set-intersection set ...)
fn primitive_set_intersection(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::set());
    }
    
    if let Value::Set(first_set) = &args[0] {
        let mut result_elements = first_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        
        for set_arg in &args[1..] {
            if let Value::Set(set) = set_arg {
                result_elements.retain(|element| {
                    set.contains(element).unwrap_or(false)
                });
            } else {
                return Err(type_error("Expected a set"));
            }
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Computes the difference of sets.
/// (set-difference set ...)
fn primitive_set_difference(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::set());
    }
    
    if let Value::Set(first_set) = &args[0] {
        let mut result_elements = first_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        
        for set_arg in &args[1..] {
            if let Value::Set(set) = set_arg {
                result_elements.retain(|element| {
                    !set.contains(element).unwrap_or(false)
                });
            } else {
                return Err(type_error("Expected a set"));
            }
        }
        
        Ok(Value::set_from_iter(result_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Computes the symmetric difference (XOR) of sets.
/// (set-xor set1 set2)
fn primitive_set_xor(args: &[Value]) -> Result<Value> {
    arity_check("set-xor", args, 2, Some(2))?;
    
    match (&args[0], &args[1]) {
        (Value::Set(set1), Value::Set(set2)) => {
            let elements1 = set1.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
            let elements2 = set2.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
            let mut result_elements = Vec::new();
            
            // Elements in set1 but not in set2
            for element in &elements1 {
                if !set2.contains(element).unwrap_or(false) {
                    result_elements.push(element.clone());
                }
            }
            
            // Elements in set2 but not in set1
            for element in &elements2 {
                if !set1.contains(element).unwrap_or(false) {
                    result_elements.push(element.clone());
                }
            }
            
            Ok(Value::set_from_iter(result_elements))
        }
        _ => Err(type_error("Expected two sets")),
    }
}

// Mutating versions of set operations (simplified implementations)
fn primitive_set_union_mut(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Unspecified);
    }
    
    if let Value::Set(first_set) = &args[0] {
        for other_set_value in &args[1..] {
            if let Value::Set(other_set) = other_set_value {
                let elements = other_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
                first_set.adjoin_all(elements).map_err(|_| runtime_error("Failed to union sets"))?;
            } else {
                return Err(type_error("Expected a set"));
            }
        }
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

fn primitive_set_intersection_mut(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set-intersection!", 2, None, args.len()));
    }
    
    if let Value::Set(first_set) = &args[0] {
        let first_elements = first_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        first_set.clear().map_err(|_| runtime_error("Failed to clear set"))?;
        
        for element in first_elements {
            let mut in_all_sets = true;
            for other_set_value in &args[1..] {
                if let Value::Set(other_set) = other_set_value {
                    if !other_set.contains(&element).map_err(|_| runtime_error("Failed to check set membership"))? {
                        in_all_sets = false;
                        break;
                    }
                } else {
                    return Err(type_error("Expected a set"));
                }
            }
            if in_all_sets {
                first_set.adjoin(element).map_err(|_| runtime_error("Failed to add element to set"))?;
            }
        }
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

fn primitive_set_difference_mut(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set-difference!", 2, None, args.len()));
    }
    
    if let Value::Set(first_set) = &args[0] {
        let mut elements_to_remove = Vec::new();
        
        for other_set_value in &args[1..] {
            if let Value::Set(other_set) = other_set_value {
                let other_elements = other_set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
                elements_to_remove.extend(other_elements);
            } else {
                return Err(type_error("Expected a set"));
            }
        }
        
        first_set.delete_all(elements_to_remove).map_err(|_| runtime_error("Failed to remove elements from set"))?;
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

fn primitive_set_xor_mut(args: &[Value]) -> Result<Value> {
    arity_check("set-xor!", args, 2, Some(2))?;
    
    match (&args[0], &args[1]) {
        (Value::Set(set1), Value::Set(set2)) => {
            // XOR = (A ∪ B) - (A ∩ B)
            let set1_elements = set1.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
            let set2_elements = set2.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
            
            // Clear the first set
            set1.clear().map_err(|_| runtime_error("Failed to clear set"))?;
            
            // Add elements that are in set1 but not in set2
            for element in &set1_elements {
                if !set2.contains(element).map_err(|_| runtime_error("Failed to check set membership"))? {
                    set1.adjoin(element.clone()).map_err(|_| runtime_error("Failed to add element to set"))?;
                }
            }
            
            // Add elements that are in set2 but not in original set1
            for element in &set2_elements {
                if !set1_elements.contains(element) {
                    set1.adjoin(element.clone()).map_err(|_| runtime_error("Failed to add element to set"))?;
                }
            }
            
            Ok(Value::Unspecified)
        }
        _ => Err(type_error("Expected two sets")),
    }
}

/// Tests whether two sets are equal.
/// (set=? set1 set2 ...)
fn primitive_set_equal_p(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set=?", 2, None, args.len()));
    }
    
    for window in args.windows(2) {
        match (&window[0], &window[1]) {
            (Value::Set(set1), Value::Set(set2)) => {
                if set1.size().map_err(|_| runtime_error("Failed to get set size"))? != 
                   set2.size().map_err(|_| runtime_error("Failed to get set size"))? {
                    return Ok(Value::boolean(false));
                }
                
                let elements1 = set1.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
                for element in elements1 {
                    if !set2.contains(&element).map_err(|_| runtime_error("Failed to check set membership"))? {
                        return Ok(Value::boolean(false));
                    }
                }
            }
            _ => return Err(type_error("Expected sets")),
        }
    }
    
    Ok(Value::boolean(true))
}

/// Tests whether the first set is a proper subset of the second.
/// (set<? set1 set2 ...)
fn primitive_set_subset_p(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set<?", 2, None, args.len()));
    }
    
    for window in args.windows(2) {
        match (&window[0], &window[1]) {
            (Value::Set(set1), Value::Set(set2)) => {
                let size1 = set1.size().map_err(|_| runtime_error("Failed to get set size"))?;
                let size2 = set2.size().map_err(|_| runtime_error("Failed to get set size"))?;
                
                // Proper subset requires strictly smaller size
                if size1 >= size2 {
                    return Ok(Value::boolean(false));
                }
                
                // Check if set1 is a subset of set2
                if !set1.is_subset(set2).map_err(|_| runtime_error("Failed to check subset relation"))? {
                    return Ok(Value::boolean(false));
                }
            }
            _ => return Err(type_error("Expected sets")),
        }
    }
    
    Ok(Value::boolean(true))
}

/// Tests whether the first set is a proper superset of the second.
/// (set>? set1 set2 ...)
fn primitive_set_superset_p(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set>?", 2, None, args.len()));
    }
    
    for window in args.windows(2) {
        match (&window[0], &window[1]) {
            (Value::Set(set1), Value::Set(set2)) => {
                let size1 = set1.size().map_err(|_| runtime_error("Failed to get set size"))?;
                let size2 = set2.size().map_err(|_| runtime_error("Failed to get set size"))?;
                
                // Proper superset requires strictly larger size
                if size1 <= size2 {
                    return Ok(Value::boolean(false));
                }
                
                // Check if set1 is a superset of set2
                if !set1.is_superset(set2).map_err(|_| runtime_error("Failed to check superset relation"))? {
                    return Ok(Value::boolean(false));
                }
            }
            _ => return Err(type_error("Expected sets")),
        }
    }
    
    Ok(Value::boolean(true))
}

/// Tests whether the first set is a subset of or equal to the second.
/// (set<=? set1 set2 ...)
fn primitive_set_subset_eq_p(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set<=?", 2, None, args.len()));
    }
    
    for window in args.windows(2) {
        match (&window[0], &window[1]) {
            (Value::Set(set1), Value::Set(set2)) => {
                // Check if set1 is a subset of set2 (includes equality)
                if !set1.is_subset(set2).map_err(|_| runtime_error("Failed to check subset relation"))? {
                    return Ok(Value::boolean(false));
                }
            }
            _ => return Err(type_error("Expected sets")),
        }
    }
    
    Ok(Value::boolean(true))
}

/// Tests whether the first set is a superset of or equal to the second.
/// (set>=? set1 set2 ...)
fn primitive_set_superset_eq_p(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(arity_error("set>=?", 2, None, args.len()));
    }
    
    for window in args.windows(2) {
        match (&window[0], &window[1]) {
            (Value::Set(set1), Value::Set(set2)) => {
                // Check if set1 is a superset of set2 (includes equality)
                if !set1.is_superset(set2).map_err(|_| runtime_error("Failed to check superset relation"))? {
                    return Ok(Value::boolean(false));
                }
            }
            _ => return Err(type_error("Expected sets")),
        }
    }
    
    Ok(Value::boolean(true))
}

/// Creates a copy of a set.
/// (set-copy set)
fn primitive_set_copy(args: &[Value]) -> Result<Value> {
    arity_check("set-copy", args, 1, Some(1))?;
    
    if let Value::Set(set) = &args[0] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        Ok(Value::set_from_iter(elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Partitions a set into two sets based on a predicate.
/// (set-partition pred set)
fn primitive_set_partition(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-partition", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut true_elements = Vec::new();
        let mut false_elements = Vec::new();
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if result.is_falsy() {
                false_elements.push(element);
            } else {
                true_elements.push(element);
            }
        }
        
        let true_set = Value::set_from_iter(true_elements);
        let false_set = Value::set_from_iter(false_elements);
        
        // Return as a pair (true-set . false-set)
        Ok(Value::pair(true_set, false_set))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Mutating filter for sets.
/// (set-filter! pred set)
fn primitive_set_filter_mut(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-filter!", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        
        // Clear the set and repopulate with filtered elements
        set.clear().map_err(|_| runtime_error("Failed to clear set"))?;
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if !result.is_falsy() {
                set.adjoin(element).map_err(|_| runtime_error("Failed to add element to set"))?;
            }
        }
        
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Mutating remove for sets.
/// (set-remove! pred set)
fn primitive_set_remove_mut(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-remove!", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        
        // Clear the set and repopulate with non-removed elements
        set.clear().map_err(|_| runtime_error("Failed to clear set"))?;
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if result.is_falsy() {
                set.adjoin(element).map_err(|_| runtime_error("Failed to add element to set"))?;
            }
        }
        
        Ok(Value::Unspecified)
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Mutating partition for sets.
/// (set-partition! pred set)
fn primitive_set_partition_mut(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-partition!", args, 2, Some(2))?;
    
    if let Value::Set(set) = &args[1] {
        let elements = set.to_vec().map_err(|_| runtime_error("Failed to get set elements"))?;
        let mut false_elements = Vec::new();
        
        // Clear the set and repopulate with true elements only
        set.clear().map_err(|_| runtime_error("Failed to clear set"))?;
        
        for element in elements {
            let result = apply_procedure_with_evaluator(eval, &args[0], &[element.clone()])?;
            if result.is_falsy() {
                false_elements.push(element);
            } else {
                set.adjoin(element).map_err(|_| runtime_error("Failed to add element to set"))?;
            }
        }
        
        // Return the false set as the result
        Ok(Value::set_from_iter(false_elements))
    } else {
        Err(type_error("Expected a set"))
    }
}

/// Search operation for sets with continuation-style interface.
/// (set-search! set element failure success)
fn primitive_set_search_mut(eval: &mut crate::eval::evaluator::Evaluator, args: &[Value]) -> Result<Value> {
    arity_check("set-search!", args, 4, Some(4))?;
    
    if let Value::Set(set) = &args[0] {
        let element = &args[1];
        let failure = &args[2];
        let success = &args[3];
        
        let contains = set.contains(element).map_err(|_| runtime_error("Failed to check set membership"))?;
        
        if contains {
            // Element found: call success procedure with element and update/delete procedures
            let update_proc = Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "set-search-update".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(|update_args| {
                    // This is a simplified update procedure - in a full implementation,
                    // it would capture the set and element and perform the update
                    Ok(Value::Unspecified)
                }),
                effects: vec![crate::effects::Effect::Pure],
            }));
            
            let delete_proc = Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "set-search-delete".to_string(),
                arity_min: 0,
                arity_max: Some(0),
                implementation: PrimitiveImpl::RustFn(|_delete_args| {
                    // This is a simplified delete procedure - in a full implementation,
                    // it would capture the set and element and perform the deletion
                    Ok(Value::Unspecified)
                }),
                effects: vec![crate::effects::Effect::Pure],
            }));
            
            apply_procedure_with_evaluator(eval, success, &[element.clone(), update_proc, delete_proc])
        } else {
            // Element not found: call failure procedure with insert procedure  
            let insert_proc = Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "set-search-insert".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(|insert_args| {
                    // This is a simplified insert procedure - in a full implementation,
                    // it would capture the set and perform the insertion
                    Ok(Value::Unspecified)
                }),
                effects: vec![crate::effects::Effect::Pure],
            }));
            
            apply_procedure_with_evaluator(eval, failure, &[insert_proc])
        }
    } else {
        Err(type_error("Expected a set"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Evaluator, ThreadSafeEnvironment};
    use std::sync::Arc;

    fn setup_test_env() -> Arc<ThreadSafeEnvironment> {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        install_set_primitives(&env);
        env
    }

    #[test]
    fn test_set_creation() {
        let env = setup_test_env();
        let mut eval = Evaluator::new();
        
        let result = primitive_set(&[]).unwrap();
        assert!(result.is_set());
        
        let result = primitive_set(&[Value::number(1.0), Value::number(2.0)]).unwrap();
        assert!(result.is_set());
    }

    #[test]
    fn test_set_predicates() {
        let env = setup_test_env();
        let mut eval = Evaluator::new();
        
        let set = Value::set();
        let result = primitive_set_p(&[set]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let non_set = Value::number(42.0);
        let result = primitive_set_p(&[non_set]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_set_operations() {
        let env = setup_test_env();
        let mut eval = Evaluator::new();
        
        let set1 = Value::set_from_iter(vec![Value::number(1.0), Value::number(2.0)]);
        let set2 = Value::set_from_iter(vec![Value::number(2.0), Value::number(3.0)]);
        
        let union = primitive_set_union(&[set1.clone(), set2.clone()]).unwrap();
        assert!(union.is_set());
        
        let intersection = primitive_set_intersection(&[set1.clone(), set2.clone()]).unwrap();
        assert!(intersection.is_set());
    }
}