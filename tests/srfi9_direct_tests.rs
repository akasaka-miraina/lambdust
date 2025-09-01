//! Tests for SRFI-9 direct implementation (non-macro approach)

use lambdust::Lambdust;

/// Test basic record type creation using direct approach
#[test]
fn test_record_type_create_basic() {
    let source = r#"
(define procs 
  (record-type-create 
    "point"                    ; type name
    (make-point x y)          ; constructor spec
    point?                    ; predicate name
    (x point-x point-x-set!)  ; field with mutator  
    (y point-y)))             ; field without mutator

; Extract procedures
(define make-point (car procs))
(define point? (cadr procs))

; Test basic functionality
(define p (make-point 10 20))
(point? p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(true));
        }
        Err(e) => panic!("Direct record type creation test failed: {e:?}"),
    }
}

/// Test record accessors
#[test]
fn test_record_direct_accessors() {
    let source = r#"
(define procs 
  (record-type-create 
    "point"                    
    (make-point x y)          
    point?                    
    (x point-x point-x-set!)   
    (y point-y)))

; Extract procedures  
(define make-point (car procs))
(define point? (cadr procs))
(define point-x (caddr procs))

; Test accessor
(define p (make-point 42 24))
(point-x p)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(42));
        }
        Err(e) => panic!("Direct record accessor test failed: {e:?}"),
    }
}

/// Test record mutators
#[test]
fn test_record_direct_mutators() {
    let source = r#"
(define procs 
  (record-type-create 
    "point"                    
    (make-point x y)          
    point?                    
    (x point-x point-x-set!)   
    (y point-y)))

; Extract procedures
(define make-point (car procs))
(define point-x (caddr procs))
(define point-x-set! (cadddr procs))

; Test mutator
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
        Err(e) => panic!("Direct record mutator test failed: {e:?}"),
    }
}

/// Test predicate with wrong type
#[test]
fn test_record_direct_predicate_false() {
    let source = r#"
(define procs 
  (record-type-create 
    "point"                    
    (make-point x y)          
    point?                    
    (x point-x)
    (y point-y)))

(define point? (cadr procs))
(point? 42)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(false));
        }
        Err(e) => panic!("Direct record predicate false test failed: {e:?}"),
    }
}

/// Test multiple record types don't interfere
#[test]
fn test_multiple_direct_record_types() {
    let source = r#"
(define point-procs 
  (record-type-create 
    "point"
    (make-point x y)
    point?
    (x point-x)
    (y point-y)))

(define person-procs
  (record-type-create
    "person" 
    (make-person name age)
    person?
    (name person-name)
    (age person-age)))

; Extract constructors and predicates
(define make-point (car point-procs))
(define point? (cadr point-procs))
(define make-person (car person-procs))
(define person? (cadr person-procs))

; Test type isolation
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
        Err(e) => panic!("Multiple direct record types test failed: {e:?}"),
    }
}

/// Test constructor with wrong arity
#[test]
fn test_direct_constructor_wrong_arity() {
    let source = r#"
(define procs 
  (record-type-create 
    "point"
    (make-point x y)
    point?
    (x point-x)
    (y point-y)))

(define make-point (car procs))
(make-point 10)  ; Wrong arity - should have 2 args
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

/// Test using string fields
#[test]
fn test_direct_record_string_fields() {
    let source = r#"
(define procs
  (record-type-create
    "person"
    (make-person name age)
    person?
    (name person-name)
    (age person-age)))

(define make-person (car procs))
(define person-name (caddr procs))

(define alice (make-person "Alice" 25))
(person-name alice)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Alice"));
        }
        Err(e) => panic!("Direct record string fields test failed: {e:?}"),
    }
}

/// Test immutable fields (no mutator)
#[test]
fn test_direct_immutable_record() {
    let source = r#"
(define procs
  (record-type-create
    "circle"
    (make-circle radius)
    circle?
    (radius circle-radius)))  ; No mutator specified

(define make-circle (car procs))
(define circle-radius (caddr procs))

(define c (make-circle 5))
(circle-radius c)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(5));
        }
        Err(e) => panic!("Direct immutable record test failed: {e:?}"),
    }
}
