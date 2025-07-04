//! Dynamic points tests for R7RS dynamic-wind semantics

use lambdust::evaluator::{continuation::DynamicPoint, types::Evaluator};
use lambdust::value::{Procedure, Value};

#[test]
fn test_dynamic_point_creation() {
    let before = Some(Value::Procedure(Procedure::Builtin {
        name: "before-thunk".to_string(),
        arity: Some(0),
        func: |_| Ok(Value::Undefined),
    }));

    let after = Some(Value::Procedure(Procedure::Builtin {
        name: "after-thunk".to_string(),
        arity: Some(0),
        func: |_| Ok(Value::Undefined),
    }));

    let point = DynamicPoint::new(before, after, None, 0);

    assert_eq!(point.id, 0);
    assert!(point.is_active());
    assert_eq!(point.depth(), 0);
    assert!(point.before.is_some());
    assert!(point.after.is_some());
}

#[test]
fn test_dynamic_point_hierarchy() {
    let parent = DynamicPoint::new(None, None, None, 0);
    let child = DynamicPoint::new(None, None, Some(Box::new(parent)), 1);

    assert_eq!(child.depth(), 1);
    assert_eq!(child.id, 1);
    assert!(child.parent.is_some());
}

#[test]
fn test_dynamic_point_path_to_root() {
    let root = DynamicPoint::new(None, None, None, 0);
    let middle = DynamicPoint::new(None, None, Some(Box::new(root)), 1);
    let leaf = DynamicPoint::new(None, None, Some(Box::new(middle)), 2);

    let path = leaf.path_to_root();
    assert_eq!(path, vec![2, 1, 0]);
}

#[test]
fn test_dynamic_point_common_ancestor() {
    let root = DynamicPoint::new(None, None, None, 0);
    let left_child = DynamicPoint::new(None, None, Some(Box::new(root.clone())), 1);
    let right_child = DynamicPoint::new(None, None, Some(Box::new(root)), 2);

    let common = left_child.common_ancestor(&right_child);
    assert_eq!(common, Some(0)); // Root should be common ancestor
}

#[test]
fn test_evaluator_dynamic_point_management() {
    let mut evaluator = Evaluator::new();

    // Initially no dynamic points
    assert_eq!(evaluator.dynamic_point_depth(), 0);
    assert!(evaluator.current_dynamic_point().is_none());

    // Push a dynamic point
    let before_thunk = Some(Value::Procedure(Procedure::Builtin {
        name: "before".to_string(),
        arity: Some(0),
        func: |_| Ok(Value::Undefined),
    }));

    let after_thunk = Some(Value::Procedure(Procedure::Builtin {
        name: "after".to_string(),
        arity: Some(0),
        func: |_| Ok(Value::Undefined),
    }));

    let id = evaluator.push_dynamic_point(before_thunk, after_thunk);

    assert_eq!(id, 0);
    assert_eq!(evaluator.dynamic_point_depth(), 1);
    assert!(evaluator.current_dynamic_point().is_some());

    // The current dynamic point should have the correct ID
    let current = evaluator.current_dynamic_point().unwrap();
    assert_eq!(current.id, 0);
    assert!(current.is_active());
}

#[test]
fn test_evaluator_dynamic_point_stack() {
    let mut evaluator = Evaluator::new();

    // Push multiple dynamic points
    let id1 = evaluator.push_dynamic_point(None, None);
    let id2 = evaluator.push_dynamic_point(None, None);
    let id3 = evaluator.push_dynamic_point(None, None);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
    assert_eq!(evaluator.dynamic_point_depth(), 3);

    // Current should be the last pushed
    let current = evaluator.current_dynamic_point().unwrap();
    assert_eq!(current.id, 2);

    // Pop dynamic points
    let popped = evaluator.pop_dynamic_point();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().id, 2);
    assert_eq!(evaluator.dynamic_point_depth(), 2);

    // Current should now be the previous one
    let current = evaluator.current_dynamic_point().unwrap();
    assert_eq!(current.id, 1);
}

#[test]
fn test_evaluator_find_dynamic_point() {
    let mut evaluator = Evaluator::new();

    let id1 = evaluator.push_dynamic_point(None, None);
    let id2 = evaluator.push_dynamic_point(None, None);

    // Find by ID
    let found = evaluator.find_dynamic_point(id1);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, id1);

    let found = evaluator.find_dynamic_point(id2);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, id2);

    // Non-existent ID
    let not_found = evaluator.find_dynamic_point(999);
    assert!(not_found.is_none());
}

#[test]
fn test_evaluator_clear_dynamic_points() {
    let mut evaluator = Evaluator::new();

    // Push some dynamic points
    evaluator.push_dynamic_point(None, None);
    evaluator.push_dynamic_point(None, None);

    assert_eq!(evaluator.dynamic_point_depth(), 2);

    // Clear all
    evaluator.clear_dynamic_points();

    assert_eq!(evaluator.dynamic_point_depth(), 0);
    assert!(evaluator.current_dynamic_point().is_none());

    // Next pushed point should start with ID 0 again
    let id = evaluator.push_dynamic_point(None, None);
    assert_eq!(id, 0);
}

#[test]
fn test_dynamic_point_deactivation() {
    let mut point = DynamicPoint::new(None, None, None, 0);

    assert!(point.is_active());

    point.deactivate();

    assert!(!point.is_active());
}

#[test]
fn test_evaluator_dynamic_point_modification() {
    let mut evaluator = Evaluator::new();

    let id = evaluator.push_dynamic_point(None, None);

    // Modify the dynamic point
    {
        let point = evaluator.find_dynamic_point_mut(id).unwrap();
        point.deactivate();
    }

    // Check the modification
    let point = evaluator.find_dynamic_point(id).unwrap();
    assert!(!point.is_active());
}
