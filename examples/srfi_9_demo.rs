//! SRFI 9 (Define-record-type) demonstration
//!
//! This example shows how to use the define-record-type macro
//! to create structured data types in Scheme.

use lambdust::Interpreter;

fn main() {
    println!("=== SRFI 9 Define-record-type Demo ===\n");

    let mut interpreter = Interpreter::new();

    // Example 1: Simple point record
    println!("1. Creating a Point record type:");
    let point_definition = r#"
(define-record-type point
  (make-point x y)
  point?
  (x point-x set-point-x!)
  (y point-y set-point-y!))
"#;

    match interpreter.eval(point_definition) {
        Ok(_) => println!("✓ Point record type defined successfully"),
        Err(e) => println!("✗ Error defining point: {}", e),
    }

    // Example 2: Create a point instance
    println!("\n2. Creating a point instance:");
    match interpreter.eval("(make-point 3 4)") {
        Ok(result) => println!("✓ Created point: {}", result),
        Err(e) => println!("✗ Error creating point: {}", e),
    }

    // Example 3: Test predicate
    println!("\n3. Testing point predicate:");
    interpreter.eval("(define p (make-point 5 10))").ok();
    match interpreter.eval("(point? p)") {
        Ok(result) => println!("✓ (point? p) => {}", result),
        Err(e) => println!("✗ Error testing predicate: {}", e),
    }

    // Example 4: Access fields
    println!("\n4. Accessing point fields:");
    match interpreter.eval("(point-x p)") {
        Ok(result) => println!("✓ (point-x p) => {}", result),
        Err(e) => println!("✗ Error accessing x: {}", e),
    }

    match interpreter.eval("(point-y p)") {
        Ok(result) => println!("✓ (point-y p) => {}", result),
        Err(e) => println!("✗ Error accessing y: {}", e),
    }

    // Example 5: More complex record - Person
    println!("\n5. Creating a Person record type:");
    let person_definition = r#"
(define-record-type person
  (make-person name age email)
  person?
  (name person-name set-person-name!)
  (age person-age set-person-age!)
  (email person-email set-person-email!))
"#;

    match interpreter.eval(person_definition) {
        Ok(_) => println!("✓ Person record type defined successfully"),
        Err(e) => println!("✗ Error defining person: {}", e),
    }

    // Example 6: Create and use person
    println!("\n6. Creating and using person records:");
    interpreter
        .eval(r#"(define alice (make-person "Alice" 30 "alice@example.com"))"#)
        .ok();

    match interpreter.eval("(person? alice)") {
        Ok(result) => println!("✓ (person? alice) => {}", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    match interpreter.eval("(person-name alice)") {
        Ok(result) => println!("✓ (person-name alice) => {}", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    match interpreter.eval("(person-age alice)") {
        Ok(result) => println!("✓ (person-age alice) => {}", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Example 7: Type checking
    println!("\n7. Testing type safety:");
    match interpreter.eval("(person? p)") {
        Ok(result) => println!("✓ (person? p) => {} (point is not a person)", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    match interpreter.eval("(point? alice)") {
        Ok(result) => println!("✓ (point? alice) => {} (person is not a point)", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    println!("SRFI 9 provides:");
    println!("• Type-safe record creation");
    println!("• Automatic constructor generation");
    println!("• Type predicates for runtime checking");
    println!("• Field accessors and mutators");
    println!("• Distinct types (person ≠ point even with same structure)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambdust::Value;

    #[test]
    fn test_basic_record_operations() {
        let mut interpreter = Interpreter::new();

        // Define a simple record type
        let result = interpreter.eval(
            r#"
(define-record-type test-record
  (make-test-record field1 field2)
  test-record?
  (field1 test-record-field1)
  (field2 test-record-field2))
"#,
        );
        assert!(result.is_ok());

        // Create an instance
        let result = interpreter.eval("(make-test-record 42 \"hello\")");
        assert!(result.is_ok());

        // Store the instance
        interpreter
            .eval("(define r (make-test-record 42 \"hello\"))")
            .unwrap();

        // Test predicate
        let result = interpreter.eval("(test-record? r)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test field access
        let result = interpreter.eval("(test-record-field1 r)").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_record_type_safety() {
        let mut interpreter = Interpreter::new();

        // Define two different record types
        interpreter
            .eval(
                r#"
(define-record-type type-a
  (make-a field)
  a?
  (field a-field))
"#,
            )
            .unwrap();

        interpreter
            .eval(
                r#"
(define-record-type type-b
  (make-b field)
  b?
  (field b-field))
"#,
            )
            .unwrap();

        // Create instances
        interpreter.eval("(define a-instance (make-a 1))").unwrap();
        interpreter.eval("(define b-instance (make-b 2))").unwrap();

        // Test type predicates
        assert_eq!(
            interpreter.eval("(a? a-instance)").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            interpreter.eval("(b? a-instance)").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            interpreter.eval("(a? b-instance)").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            interpreter.eval("(b? b-instance)").unwrap(),
            Value::Boolean(true)
        );
    }
}
