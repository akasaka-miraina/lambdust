//! Comprehensive unit tests for Record types (SRFI 9)
//!
//! Tests the Record and RecordType value types that provide structured data
//! according to SRFI 9 specifications.

use lambdust::lexer::SchemeNumber;
use lambdust::value::{Record, RecordType, Value};

/// Helper to create test record type
fn create_test_record_type() -> RecordType {
    RecordType {
        name: "person".to_string(),
        field_names: vec!["name".to_string(), "age".to_string(), "email".to_string()],
        constructor_name: "make-person".to_string(),
        predicate_name: "person?".to_string(),
    }
}

/// Helper to create test record
fn create_test_record() -> Record {
    Record {
        record_type: create_test_record_type(),
        fields: vec![
            Value::String("Alice".to_string()),
            Value::Number(SchemeNumber::Integer(30)),
            Value::String("alice@example.com".to_string()),
        ],
    }
}

/// Helper to create various record types for testing
fn create_various_record_types() -> Vec<RecordType> {
    vec![
        RecordType {
            name: "point".to_string(),
            field_names: vec!["x".to_string(), "y".to_string()],
            constructor_name: "make-point".to_string(),
            predicate_name: "point?".to_string(),
        },
        RecordType {
            name: "book".to_string(),
            field_names: vec!["title".to_string(), "author".to_string(), "isbn".to_string()],
            constructor_name: "make-book".to_string(),
            predicate_name: "book?".to_string(),
        },
        RecordType {
            name: "empty-record".to_string(),
            field_names: vec![],
            constructor_name: "make-empty".to_string(),
            predicate_name: "empty?".to_string(),
        },
    ]
}

#[test]
fn test_record_type_creation() {
    let record_type = create_test_record_type();
    
    assert_eq!(record_type.name, "person");
    assert_eq!(record_type.field_names.len(), 3);
    assert_eq!(record_type.field_names[0], "name");
    assert_eq!(record_type.field_names[1], "age");
    assert_eq!(record_type.field_names[2], "email");
    assert_eq!(record_type.constructor_name, "make-person");
    assert_eq!(record_type.predicate_name, "person?");
}

#[test]
fn test_record_type_empty_fields() {
    let empty_record_type = RecordType {
        name: "empty".to_string(),
        field_names: vec![],
        constructor_name: "make-empty".to_string(),
        predicate_name: "empty?".to_string(),
    };
    
    assert_eq!(empty_record_type.name, "empty");
    assert!(empty_record_type.field_names.is_empty());
    assert_eq!(empty_record_type.constructor_name, "make-empty");
    assert_eq!(empty_record_type.predicate_name, "empty?");
}

#[test]
fn test_record_type_single_field() {
    let single_field_type = RecordType {
        name: "wrapper".to_string(),
        field_names: vec!["value".to_string()],
        constructor_name: "make-wrapper".to_string(),
        predicate_name: "wrapper?".to_string(),
    };
    
    assert_eq!(single_field_type.field_names.len(), 1);
    assert_eq!(single_field_type.field_names[0], "value");
}

#[test]
fn test_record_creation() {
    let record = create_test_record();
    
    assert_eq!(record.record_type.name, "person");
    assert_eq!(record.fields.len(), 3);
    
    // Check field values
    assert_eq!(record.fields[0], Value::String("Alice".to_string()));
    assert_eq!(record.fields[1], Value::Number(SchemeNumber::Integer(30)));
    assert_eq!(record.fields[2], Value::String("alice@example.com".to_string()));
}

#[test]
fn test_record_with_empty_fields() {
    let empty_record_type = RecordType {
        name: "empty".to_string(),
        field_names: vec![],
        constructor_name: "make-empty".to_string(),
        predicate_name: "empty?".to_string(),
    };
    
    let empty_record = Record {
        record_type: empty_record_type,
        fields: vec![],
    };
    
    assert_eq!(empty_record.record_type.name, "empty");
    assert!(empty_record.fields.is_empty());
}

#[test]
fn test_record_with_various_value_types() {
    let mixed_record_type = RecordType {
        name: "mixed".to_string(),
        field_names: vec![
            "number".to_string(),
            "string".to_string(),
            "boolean".to_string(),
            "character".to_string(),
            "nil".to_string(),
        ],
        constructor_name: "make-mixed".to_string(),
        predicate_name: "mixed?".to_string(),
    };
    
    let mixed_record = Record {
        record_type: mixed_record_type,
        fields: vec![
            Value::Number(SchemeNumber::Integer(42)),
            Value::String("hello".to_string()),
            Value::Boolean(true),
            Value::Character('x'),
            Value::Nil,
        ],
    };
    
    assert_eq!(mixed_record.fields.len(), 5);
    assert_eq!(mixed_record.fields[0], Value::Number(SchemeNumber::Integer(42)));
    assert_eq!(mixed_record.fields[1], Value::String("hello".to_string()));
    assert_eq!(mixed_record.fields[2], Value::Boolean(true));
    assert_eq!(mixed_record.fields[3], Value::Character('x'));
    assert_eq!(mixed_record.fields[4], Value::Nil);
}

#[test]
fn test_value_is_record() {
    let record = create_test_record();
    let record_value = Value::Record(record);
    let non_record_value = Value::Number(SchemeNumber::Integer(42));
    
    assert!(record_value.is_record());
    assert!(!non_record_value.is_record());
}

#[test]
fn test_value_as_record() {
    let record = create_test_record();
    let record_value = Value::Record(record.clone());
    let non_record_value = Value::String("not a record".to_string());
    
    // Should extract record successfully
    let extracted_record = record_value.as_record();
    assert!(extracted_record.is_some());
    assert_eq!(extracted_record.unwrap().record_type.name, "person");
    
    // Should return None for non-record
    assert!(non_record_value.as_record().is_none());
}

#[test]
fn test_value_is_record_of_type() {
    let person_record = create_test_record();
    let person_value = Value::Record(person_record);
    
    let point_record = Record {
        record_type: RecordType {
            name: "point".to_string(),
            field_names: vec!["x".to_string(), "y".to_string()],
            constructor_name: "make-point".to_string(),
            predicate_name: "point?".to_string(),
        },
        fields: vec![
            Value::Number(SchemeNumber::Integer(10)),
            Value::Number(SchemeNumber::Integer(20)),
        ],
    };
    let point_value = Value::Record(point_record);
    
    let non_record_value = Value::Number(SchemeNumber::Integer(42));
    
    // Test correct type matching
    assert!(person_value.is_record_of_type("person"));
    assert!(!person_value.is_record_of_type("point"));
    
    assert!(point_value.is_record_of_type("point"));
    assert!(!point_value.is_record_of_type("person"));
    
    // Non-record should always return false
    assert!(!non_record_value.is_record_of_type("person"));
    assert!(!non_record_value.is_record_of_type("point"));
}

#[test]
fn test_record_type_equality() {
    let type1 = create_test_record_type();
    let type2 = create_test_record_type();
    let different_type = RecordType {
        name: "different".to_string(),
        field_names: vec!["field".to_string()],
        constructor_name: "make-different".to_string(),
        predicate_name: "different?".to_string(),
    };
    
    assert_eq!(type1, type2);
    assert_ne!(type1, different_type);
}

#[test]
fn test_record_equality() {
    let record1 = create_test_record();
    let record2 = create_test_record();
    
    let different_record = Record {
        record_type: create_test_record_type(),
        fields: vec![
            Value::String("Bob".to_string()),
            Value::Number(SchemeNumber::Integer(25)),
            Value::String("bob@example.com".to_string()),
        ],
    };
    
    assert_eq!(record1, record2);
    assert_ne!(record1, different_record);
}

#[test]
fn test_record_with_nested_records() {
    let address_type = RecordType {
        name: "address".to_string(),
        field_names: vec!["street".to_string(), "city".to_string()],
        constructor_name: "make-address".to_string(),
        predicate_name: "address?".to_string(),
    };
    
    let address_record = Record {
        record_type: address_type.clone(),
        fields: vec![
            Value::String("123 Main St".to_string()),
            Value::String("Anytown".to_string()),
        ],
    };
    
    let person_with_address_type = RecordType {
        name: "person-with-address".to_string(),
        field_names: vec!["name".to_string(), "address".to_string()],
        constructor_name: "make-person-with-address".to_string(),
        predicate_name: "person-with-address?".to_string(),
    };
    
    let person_with_address = Record {
        record_type: person_with_address_type,
        fields: vec![
            Value::String("Charlie".to_string()),
            Value::Record(address_record),
        ],
    };
    
    assert_eq!(person_with_address.fields.len(), 2);
    assert!(person_with_address.fields[1].is_record());
    assert!(person_with_address.fields[1].is_record_of_type("address"));
}

#[test]
fn test_record_debug_formatting() {
    let record = create_test_record();
    let record_value = Value::Record(record);
    
    // Should format without panicking
    let debug_string = format!("{:?}", record_value);
    assert!(!debug_string.is_empty());
    assert!(debug_string.contains("Record"));
}

#[test]
fn test_record_clone_behavior() {
    let original_record = create_test_record();
    let cloned_record = original_record.clone();
    
    // Should be equal but separate instances
    assert_eq!(original_record, cloned_record);
    
    // Verify deep cloning of fields
    assert_eq!(original_record.fields.len(), cloned_record.fields.len());
    for (orig, clone) in original_record.fields.iter().zip(cloned_record.fields.iter()) {
        assert_eq!(orig, clone);
    }
}

#[test]
fn test_various_record_types() {
    let record_types = create_various_record_types();
    
    assert_eq!(record_types.len(), 3);
    
    // Point record
    assert_eq!(record_types[0].name, "point");
    assert_eq!(record_types[0].field_names.len(), 2);
    
    // Book record
    assert_eq!(record_types[1].name, "book");
    assert_eq!(record_types[1].field_names.len(), 3);
    
    // Empty record
    assert_eq!(record_types[2].name, "empty-record");
    assert!(record_types[2].field_names.is_empty());
}

#[test]
fn test_record_field_access_patterns() {
    let record = create_test_record();
    
    // Test field access by index
    assert_eq!(record.fields[0], Value::String("Alice".to_string()));
    assert_eq!(record.fields[1], Value::Number(SchemeNumber::Integer(30)));
    assert_eq!(record.fields[2], Value::String("alice@example.com".to_string()));
    
    // Test field name correspondence
    assert_eq!(record.record_type.field_names[0], "name");
    assert_eq!(record.record_type.field_names[1], "age");
    assert_eq!(record.record_type.field_names[2], "email");
}

#[test]
fn test_record_large_field_count() {
    let large_field_names: Vec<String> = (0..100)
        .map(|i| format!("field{}", i))
        .collect();
    
    let large_record_type = RecordType {
        name: "large-record".to_string(),
        field_names: large_field_names.clone(),
        constructor_name: "make-large".to_string(),
        predicate_name: "large?".to_string(),
    };
    
    let large_fields: Vec<Value> = (0..100)
        .map(|i| Value::Number(SchemeNumber::Integer(i)))
        .collect();
    
    let large_record = Record {
        record_type: large_record_type,
        fields: large_fields,
    };
    
    assert_eq!(large_record.record_type.field_names.len(), 100);
    assert_eq!(large_record.fields.len(), 100);
    assert_eq!(large_record.fields[99], Value::Number(SchemeNumber::Integer(99)));
}

#[test]
fn test_record_constructor_predicate_names() {
    let record_types = create_various_record_types();
    
    // Verify constructor and predicate naming patterns
    assert_eq!(record_types[0].constructor_name, "make-point");
    assert_eq!(record_types[0].predicate_name, "point?");
    
    assert_eq!(record_types[1].constructor_name, "make-book");
    assert_eq!(record_types[1].predicate_name, "book?");
    
    assert_eq!(record_types[2].constructor_name, "make-empty");
    assert_eq!(record_types[2].predicate_name, "empty?");
}

#[test]
fn test_record_field_value_consistency() {
    let record = create_test_record();
    
    // Number of fields should match number of field names
    assert_eq!(record.record_type.field_names.len(), record.fields.len());
    
    // Field access should be consistent
    for i in 0..record.fields.len() {
        assert!(i < record.record_type.field_names.len());
        assert!(i < record.fields.len());
    }
}