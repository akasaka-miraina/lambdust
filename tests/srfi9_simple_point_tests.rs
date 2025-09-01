//! Tests for SRFI-9 simple point implementation

use lambdust::Lambdust;

/// Test basic point creation and predicate
#[test]
fn test_point_creation_basic() {
    let source = r#"
(define p (make-point 10 20))
(point? p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(true));
        }
        Err(e) => panic!("Point creation test failed: {e:?}"),
    }
}

/// Test point accessors
#[test]
fn test_point_accessors() {
    let source = r#"
(define p (make-point 42 24))
(point-x p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(42));
        }
        Err(e) => panic!("Point accessor test failed: {e:?}"),
    }
}

/// Test point y accessor
#[test]
fn test_point_y_accessor() {
    let source = r#"
(define p (make-point 10 30))
(point-y p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(30));
        }
        Err(e) => panic!("Point y accessor test failed: {e:?}"),
    }
}

/// Test point mutators
#[test]
fn test_point_mutators() {
    let source = r#"
(define p (make-point 10 20))
(point-x-set! p 100)
(point-x p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(100));
        }
        Err(e) => panic!("Point mutator test failed: {e:?}"),
    }
}

/// Test point y mutator
#[test]
fn test_point_y_mutator() {
    let source = r#"
(define p (make-point 10 20))
(point-y-set! p 200)
(point-y p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(200));
        }
        Err(e) => panic!("Point y mutator test failed: {e:?}"),
    }
}

/// Test predicate with non-point
#[test]
fn test_point_predicate_false() {
    let source = r#"
(point? 42)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(false));
        }
        Err(e) => panic!("Point predicate false test failed: {e:?}"),
    }
}

/// Test constructor with wrong arity
#[test]
fn test_point_constructor_wrong_arity() {
    let source = r#"
(make-point 10)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - wrong number of arguments
        }
        Ok(value) => {
            panic!("Should have failed with wrong arity, got: {value:?}");
        }
    }
}

/// Test accessor with non-record
#[test]
fn test_point_accessor_non_record() {
    let source = r#"
(point-x "not a point")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - not a record
        }
        Ok(value) => {
            panic!("Should have failed with non-record argument, got: {value:?}");
        }
    }
}

/// Test mutator with non-record
#[test]
fn test_point_mutator_non_record() {
    let source = r#"
(point-x-set! "not a point" 42)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - not a record
        }
        Ok(value) => {
            panic!("Should have failed with non-record argument, got: {value:?}");
        }
    }
}

/// Test multiple mutations
#[test]
fn test_multiple_mutations() {
    let source = r#"
(define p (make-point 1 2))
(point-x-set! p 10)
(point-y-set! p 20)
(+ (point-x p) (point-y p))
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(30));
        }
        Err(e) => panic!("Multiple mutations test failed: {e:?}"),
    }
}
