//! Tests for SRFI-9: Defining Record Types implementation

use lambdust::Lambdust;

/// Test if define-record-type is available as a function
#[test]
fn test_define_record_type_exists() {
    let source = r#"define-record-type"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            println!("define-record-type value: {value:?}");
            // It should be a procedure
            assert!(
                value.is_procedure(),
                "define-record-type should be a procedure"
            );
        }
        Err(e) => panic!("define-record-type is not bound: {e:?}"),
    }
}

/// Test minimal define-record-type call
#[test]
fn test_define_record_type_minimal_call() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            println!("Minimal define-record-type call succeeded: {value:?}");
        }
        Err(e) => {
            println!("Minimal define-record-type call failed: {e:?}");
            panic!("Minimal define-record-type call failed: {e:?}");
        }
    }
}

/// Test basic define-record-type with simple record
#[test]
fn test_define_record_type_basic() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(define p (make-point 10 20))
(point? p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(true));
        }
        Err(e) => panic!("Define-record-type basic test failed: {e:?}"),
    }
}

/// Test record accessors
#[test]
fn test_record_accessors() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(define p (make-point 42 24))
(point-x p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(42));
        }
        Err(e) => panic!("Record accessor test failed: {e:?}"),
    }
}

/// Test record mutators
#[test]
fn test_record_mutators() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x point-x-set!)
  (y point-y point-y-set!))

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
        Err(e) => panic!("Record mutator test failed: {e:?}"),
    }
}

/// Test record predicate with wrong type
#[test]
fn test_record_predicate_false() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(point? 42)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(false));
        }
        Err(e) => panic!("Record predicate false test failed: {e:?}"),
    }
}

/// Test multiple record types
#[test]
fn test_multiple_record_types() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(define-record-type person
  (make-person name age)
  person?
  (name person-name)
  (age person-age))

(define p (make-point 10 20))
(define person1 (make-person "Alice" 30))

(and (point? p) 
     (not (point? person1))
     (person? person1)
     (not (person? p)))
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(true));
        }
        Err(e) => panic!("Multiple record types test failed: {e:?}"),
    }
}

/// Test record with string fields
#[test]
fn test_record_string_fields() {
    let source = r#"
(define-record-type person
  (make-person name age)
  person?
  (name person-name)
  (age person-age))

(define alice (make-person "Alice" 25))
(person-name alice)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Alice"));
        }
        Err(e) => panic!("Record string fields test failed: {e:?}"),
    }
}

/// Test record with no mutators (immutable fields)
#[test]
fn test_immutable_record() {
    let source = r#"
(define-record-type circle
  (make-circle radius)
  circle?
  (radius circle-radius))

(define c (make-circle 5.0))
(circle-radius c)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            // The value should be a number (could be integer or real)
            if let Some(n) = value.as_integer() {
                assert_eq!(n, 5);
            } else {
                // For real numbers, check if it's close to 5.0
                let display = format!("{value}");
                assert!(display.contains("5") || display == "5.0");
            }
        }
        Err(e) => panic!("Immutable record test failed: {e:?}"),
    }
}

/// Test error handling for wrong constructor arity
#[test]
fn test_constructor_wrong_arity() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

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

/// Test nested record structures
#[test]
fn test_nested_records() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(define-record-type line
  (make-line start end)
  line?
  (start line-start)
  (end line-end))

(define p1 (make-point 0 0))
(define p2 (make-point 10 10))
(define l (make-line p1 p2))

(point-x (line-start l))
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(0));
        }
        Err(e) => panic!("Nested records test failed: {e:?}"),
    }
}

/// Test record equality (should be by identity, not by value)
#[test]
fn test_record_identity() {
    let source = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x)
  (y point-y))

(define p1 (make-point 10 20))
(define p2 (make-point 10 20))

(eq? p1 p2)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            // Records with same content but different identity should not be eq?
            assert_eq!(value.as_boolean(), Some(false));
        }
        Err(e) => panic!("Record identity test failed: {e:?}"),
    }
}
