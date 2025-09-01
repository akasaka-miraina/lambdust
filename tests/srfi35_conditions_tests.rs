// SRFI-35 Conditions compliance tests
//
// This test suite verifies complete SRFI-35 compliance including:
// - Condition type definition and hierarchy
// - Condition object creation and manipulation
// - Standard condition types and their behavior
// - Condition composition (compound conditions)
// - Integration with existing exception handling
// - define-condition-type macro functionality

use lambdust::ast::Literal;
use lambdust::eval::value::Value;
use lambdust::stdlib::srfi35_conditions::{
    initialize_condition_system, condition_system, ConditionTypeRegistry, StandardConditionTypes,
    ConditionValue, SimpleCondition, ConditionObject,
};
use lambdust::stdlib::srfi35_procedures::*;
use lambdust::utils::string_interner::StringInterner;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod condition_system_tests {
    use super::*;

    fn setup_condition_system() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).expect("Failed to initialize condition system");
    }

    #[test]
    fn test_condition_type_registration() {
        setup_condition_system();
        let system = condition_system();
        
        // Test standard types are registered
        let condition_type = system.registry.get_type_by_name("&condition");
        assert!(condition_type.is_some(), "Standard &condition type should be registered");
        
        let message_type = system.registry.get_type_by_name("&message");
        assert!(message_type.is_some(), "Standard &message type should be registered");
        
        let error_type = system.registry.get_type_by_name("&error");
        assert!(error_type.is_some(), "Standard &error type should be registered");
    }

    #[test]
    fn test_condition_type_hierarchy() {
        setup_condition_system();
        let system = condition_system();
        
        // Test inheritance relationships
        assert!(system.registry.is_subtype(system.standard_types.message, system.standard_types.condition));
        assert!(system.registry.is_subtype(system.standard_types.error, system.standard_types.serious));
        assert!(system.registry.is_subtype(system.standard_types.serious, system.standard_types.condition));
        assert!(system.registry.is_subtype(system.standard_types.violation, system.standard_types.condition));
        
        // Test non-inheritance relationships
        assert!(!system.registry.is_subtype(system.standard_types.condition, system.standard_types.message));
        assert!(!system.registry.is_subtype(system.standard_types.violation, system.standard_types.error));
    }

    #[test]
    fn test_simple_condition_creation() {
        setup_condition_system();
        let system = condition_system();
        let message_field = system.registry.interner.intern("message");
        
        let mut fields = HashMap::new();
        fields.insert(message_field, Value::string("Test message".to_string()));
        
        let condition = SimpleCondition::new(system.standard_types.message, fields);
        
        assert_eq!(condition.condition_type, system.standard_types.message);
        assert!(condition.get_field(message_field).is_some());
        
        match condition.get_field(message_field).unwrap() {
            Value::Literal(Literal::String(s)) => assert_eq!(s, "Test message"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_compound_condition_creation() {
        setup_condition_system();
        let system = condition_system();
        let message_field = system.registry.interner.intern("message");
        
        // Create message condition
        let mut message_fields = HashMap::new();
        message_fields.insert(message_field, Value::string("Error occurred".to_string()));
        let message_condition = SimpleCondition::new(system.standard_types.message, message_fields);
        
        // Create error condition
        let error_condition = SimpleCondition::new(system.standard_types.error, HashMap::new());
        
        // Create compound condition
        let compound = ConditionObject::compound(&[message_condition.clone(), error_condition.clone()]);
        
        match compound {
            ConditionObject::Compound(conditions) => {
                assert_eq!(conditions.len(), 2);
                assert_eq!(conditions[0].condition_type, system.standard_types.message);
                assert_eq!(conditions[1].condition_type, system.standard_types.error);
            },
            _ => panic!("Expected compound condition"),
        }
    }

    #[test]
    fn test_condition_type_checking() {
        setup_condition_system();
        let system = condition_system();
        
        // Create error condition (which is also a serious condition)
        let error_condition = SimpleCondition::new(system.standard_types.error, HashMap::new());
        let condition_obj = ConditionObject::simple(error_condition);
        
        // Test type hierarchy checking
        assert!(condition_obj.has_type(system.standard_types.error, &system.registry));
        assert!(condition_obj.has_type(system.standard_types.serious, &system.registry));
        assert!(condition_obj.has_type(system.standard_types.condition, &system.registry));
        
        // Test negative cases
        assert!(!condition_obj.has_type(system.standard_types.message, &system.registry));
        assert!(!condition_obj.has_type(system.standard_types.violation, &system.registry));
    }

    #[test]
    fn test_condition_field_access() {
        setup_condition_system();
        let system = condition_system();
        let message_field = system.registry.interner.intern("message");
        
        let mut fields = HashMap::new();
        fields.insert(message_field, Value::string("Test message".to_string()));
        
        let message_condition = SimpleCondition::new(system.standard_types.message, fields);
        let condition_obj = ConditionObject::simple(message_condition);
        
        // Test field access
        let message_value = condition_obj.condition_ref(message_field, &system.registry);
        assert!(message_value.is_some());
        
        match message_value.unwrap() {
            Value::Literal(Literal::String(s)) => assert_eq!(s, "Test message"),
            _ => panic!("Expected string value"),
        }
        
        // Test non-existent field
        let nonexistent_field = system.registry.interner.intern("nonexistent");
        let nonexistent_value = condition_obj.condition_ref(nonexistent_field, &system.registry);
        assert!(nonexistent_value.is_none());
    }

    #[test]
    fn test_condition_extraction() {
        setup_condition_system();
        let system = condition_system();
        
        // Create compound condition with message and error
        let message_condition = SimpleCondition::new(system.standard_types.message, HashMap::new());
        let error_condition = SimpleCondition::new(system.standard_types.error, HashMap::new());
        let compound = ConditionObject::compound(&[message_condition, error_condition]);
        
        // Extract error conditions
        let error_conditions = compound.extract_conditions(system.standard_types.error, &system.registry);
        assert_eq!(error_conditions.len(), 1);
        assert_eq!(error_conditions[0].condition_type, system.standard_types.error);
        
        // Extract serious conditions (error is a subtype of serious)
        let serious_conditions = compound.extract_conditions(system.standard_types.serious, &system.registry);
        assert_eq!(serious_conditions.len(), 1);
        assert_eq!(serious_conditions[0].condition_type, system.standard_types.error);
        
        // Extract message conditions
        let message_conditions = compound.extract_conditions(system.standard_types.message, &system.registry);
        assert_eq!(message_conditions.len(), 1);
        assert_eq!(message_conditions[0].condition_type, system.standard_types.message);
        
        // Extract non-existent type
        let violation_conditions = compound.extract_conditions(system.standard_types.violation, &system.registry);
        assert_eq!(violation_conditions.len(), 0);
    }
}

#[cfg(test)]
mod condition_procedures_tests {
    use super::*;

    fn setup_condition_system() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).expect("Failed to initialize condition system");
    }

    #[test]
    fn test_condition_predicate() {
        setup_condition_system();
        
        // Create a condition
        let message_args = &[Value::string("Test message".to_string())];
        let condition_value = primitive_make_message_condition(&message_args).unwrap();
        
        // Test condition? predicate
        let predicate_result = primitive_condition_p(&[condition_value.clone()]).unwrap();
        assert_eq!(predicate_result, Value::boolean(true));
        
        // Test with non-condition
        let non_condition_result = primitive_condition_p(&[Value::integer(42)]).unwrap();
        assert_eq!(non_condition_result, Value::boolean(false));
    }

    #[test]
    fn test_message_condition_creation_and_access() {
        setup_condition_system();
        
        let message = "Test error message";
        let args = &[Value::string(message.to_string())];
        
        // Create message condition
        let condition = primitive_make_message_condition(&args).unwrap();
        
        // Test message condition predicate
        let predicate_result = primitive_message_condition_p(&[condition.clone()]).unwrap();
        assert_eq!(predicate_result, Value::boolean(true));
        
        // Test message accessor
        let message_result = primitive_condition_message(&[condition]).unwrap();
        match message_result {
            Value::Literal(Literal::String(s)) => assert_eq!(s, message),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_error_condition_creation() {
        setup_condition_system();
        
        let condition = primitive_make_error(&[]).unwrap();
        
        // Test error condition predicate
        let predicate_result = primitive_error_p(&[condition.clone()]).unwrap();
        assert_eq!(predicate_result, Value::boolean(true));
        
        // Test that error is also a serious condition
        let serious_result = primitive_serious_condition_p(&[condition]).unwrap();
        assert_eq!(serious_result, Value::boolean(true));
    }

    #[test]
    fn test_compound_condition_creation() {
        setup_condition_system();
        
        // Create individual conditions
        let message_condition = primitive_make_message_condition(&[Value::string("Error".to_string())]).unwrap();
        let error_condition = primitive_make_error(&[]).unwrap();
        
        // Create compound condition
        let compound_args = &[message_condition.clone(), error_condition.clone()];
        let compound = primitive_condition(&compound_args).unwrap();
        
        // Test that it's still a condition
        let predicate_result = primitive_condition_p(&[compound.clone()]).unwrap();
        assert_eq!(predicate_result, Value::boolean(true));
        
        // Test simple-conditions extraction
        let simple_conditions_result = primitive_simple_conditions(&[compound]).unwrap();
        match simple_conditions_result {
            Value::List(conditions) => {
                assert_eq!(conditions.len(), 2);
                // Both should be condition objects
                for condition in conditions {
                    let is_condition = primitive_condition_p(&[condition]).unwrap();
                    assert_eq!(is_condition, Value::boolean(true));
                }
            },
            _ => panic!("Expected list of conditions"),
        }
    }

    #[test]
    fn test_condition_has_type() {
        setup_condition_system();
        let system = condition_system();
        
        // Create error condition
        let error_condition = primitive_make_error(&[]).unwrap();
        
        // Test has-type with error type
        let error_type = Value::ConditionType(system.standard_types.error);
        let has_error_type = primitive_condition_has_type_p(&[error_condition.clone(), error_type]).unwrap();
        assert_eq!(has_error_type, Value::boolean(true));
        
        // Test has-type with serious type (error is a subtype of serious)
        let serious_type = Value::ConditionType(system.standard_types.serious);
        let has_serious_type = primitive_condition_has_type_p(&[error_condition.clone(), serious_type]).unwrap();
        assert_eq!(has_serious_type, Value::boolean(true));
        
        // Test has-type with message type (error is not a message)
        let message_type = Value::ConditionType(system.standard_types.message);
        let has_message_type = primitive_condition_has_type_p(&[error_condition, message_type]).unwrap();
        assert_eq!(has_message_type, Value::boolean(false));
    }

    #[test]
    fn test_extract_condition() {
        setup_condition_system();
        let system = condition_system();
        
        // Create compound condition
        let message_condition = primitive_make_message_condition(&[Value::string("Error".to_string())]).unwrap();
        let error_condition = primitive_make_error(&[]).unwrap();
        let compound = primitive_condition(&[message_condition, error_condition]).unwrap();
        
        // Extract error conditions
        let error_type = Value::ConditionType(system.standard_types.error);
        let extracted = primitive_extract_condition(&[compound.clone(), error_type]).unwrap();
        
        // Should get back a condition containing only error conditions
        match extracted {
            Value::Condition(_) => {
                let is_error = primitive_error_p(&[extracted]).unwrap();
                assert_eq!(is_error, Value::boolean(true));
            },
            _ => panic!("Expected condition object"),
        }
        
        // Extract non-existent type should return false
        let violation_type = Value::ConditionType(system.standard_types.violation);
        let no_extract = primitive_extract_condition(&[compound, violation_type]).unwrap();
        assert_eq!(no_extract, Value::boolean(false));
    }

    #[test]
    fn test_condition_ref() {
        setup_condition_system();
        let system = condition_system();
        
        let message = "Test message";
        let condition = primitive_make_message_condition(&[Value::string(message.to_string())]).unwrap();
        
        // Access message field
        let message_symbol = system.registry.interner.intern("message");
        let field_value = primitive_condition_ref(&[condition, Value::symbol(message_symbol)]).unwrap();
        
        match field_value {
            Value::Literal(Literal::String(s)) => assert_eq!(s, message),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_register_condition_type() {
        setup_condition_system();
        
        let args = vec![
            Value::string("&my-error".to_string()),
            Value::string("&error".to_string()),
            Value::string("make-my-error".to_string()),
            Value::string("my-error?".to_string()),
            Value::list_from_vec(&[]), // No fields for simplicity
        ];
        
        let result = primitive_register_condition_type(&args);
        assert!(result.is_ok());
        
        // Verify the type was registered
        let type_result = primitive_get_condition_type(&[Value::string("&my-error".to_string())]);
        assert!(type_result.is_ok());
        
        match type_result.unwrap() {
            Value::ConditionType(_) => {}, // Success
            _ => panic!("Expected condition type"),
        }
    }

    #[test]
    fn test_make_condition_with_fields() {
        setup_condition_system();
        
        // Create field bindings: list of (field-name . value) pairs
        let field_pair = Value::cons(
            Value::symbol("message"),
            Value::string("Custom message".to_string())
        );
        let field_bindings = Value::list_from_vec(&[field_pair]);
        
        let args = vec![
            Value::string("&message".to_string()),
            field_bindings,
        ];
        
        let result = primitive_make_condition(&args);
        assert!(result.is_ok());
        
        let condition = result.unwrap();
        
        // Verify it's a condition
        let is_condition = primitive_condition_p(&[condition.clone()]).unwrap();
        assert_eq!(is_condition, Value::boolean(true));
        
        // Verify it's a message condition
        let is_message = primitive_message_condition_p(&[condition]).unwrap();
        assert_eq!(is_message, Value::boolean(true));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn setup_condition_system() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).expect("Failed to initialize condition system");
    }

    #[test]
    fn test_exception_condition_integration() {
        setup_condition_system();
        
        // Create a condition
        let condition_value = primitive_make_message_condition(&[Value::string("Error".to_string())]).unwrap();
        
        // Convert to exception object
        if let Value::Condition(condition) = condition_value {
            let exception = lambdust::stdlib::exceptions::ExceptionObject::from_condition(condition.clone(), false);
            
            // Verify exception properties
            assert_eq!(exception.exception_type, "error");
            assert_eq!(exception.message, Some("Error".to_string()));
            assert!(!exception.continuable);
            assert!(exception.has_condition());
            
            // Convert back to condition
            let recovered_condition = exception.to_condition().unwrap();
            assert!(Arc::ptr_eq(&condition, &recovered_condition));
        } else {
            panic!("Expected condition value");
        }
    }

    #[test]
    fn test_comprehensive_condition_workflow() {
        setup_condition_system();
        let system = condition_system();
        
        // 1. Create individual conditions
        let message_condition = primitive_make_message_condition(&[Value::string("File not found".to_string())]).unwrap();
        let error_condition = primitive_make_error(&[]).unwrap();
        
        // 2. Create compound condition
        let compound = primitive_condition(&[message_condition, error_condition]).unwrap();
        
        // 3. Verify compound condition properties
        assert_eq!(primitive_condition_p(&[compound.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(primitive_error_p(&[compound.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(primitive_message_condition_p(&[compound.clone()]).unwrap(), Value::boolean(true));
        
        // 4. Extract simple conditions
        let simple_conditions = primitive_simple_conditions(&[compound.clone()]).unwrap();
        if let Value::List(conditions) = simple_conditions {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected list of conditions");
        }
        
        // 5. Test type checking
        let error_type = Value::ConditionType(system.standard_types.error);
        let has_error_type = primitive_condition_has_type_p(&[compound.clone(), error_type]).unwrap();
        assert_eq!(has_error_type, Value::boolean(true));
        
        // 6. Extract specific conditions
        let error_type = Value::ConditionType(system.standard_types.error);
        let extracted_error = primitive_extract_condition(&[compound.clone(), error_type]).unwrap();
        assert_ne!(extracted_error, Value::boolean(false));
        
        // 7. Access condition fields
        let message_symbol = system.registry.interner.intern("message");
        let message_value = primitive_condition_ref(&[compound, Value::symbol(message_symbol)]).unwrap();
        match message_value {
            Value::Literal(Literal::String(s)) => assert_eq!(s, "File not found"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_standard_condition_hierarchy_completeness() {
        setup_condition_system();
        let system = condition_system();
        
        // Test that all standard types exist and have correct relationships
        let types = [
            ("&condition", system.standard_types.condition),
            ("&message", system.standard_types.message),
            ("&serious", system.standard_types.serious),
            ("&error", system.standard_types.error),
            ("&violation", system.standard_types.violation),
            ("&assertion", system.standard_types.assertion),
            ("&non-continuable", system.standard_types.non_continuable),
            ("&implementation-restriction", system.standard_types.implementation_restriction),
        ];
        
        for (name, id) in types {
            let registered_type = system.registry.get_type_by_name(name);
            assert!(registered_type.is_some(), "Type {} should be registered", name);
            assert_eq!(registered_type.unwrap().id, id, "Type {} should have correct ID", name);
        }
        
        // Test inheritance relationships
        assert!(system.registry.is_subtype(system.standard_types.error, system.standard_types.serious));
        assert!(system.registry.is_subtype(system.standard_types.serious, system.standard_types.condition));
        assert!(system.registry.is_subtype(system.standard_types.assertion, system.standard_types.violation));
        assert!(system.registry.is_subtype(system.standard_types.violation, system.standard_types.condition));
    }
}

// Helper function to create a Value::Symbol for testing
impl Value {
    fn symbol(name: &str) -> Self {
        // This is a simplified version for testing
        // In real implementation, this would use the string interner
        let interner = StringInterner::new();
        let symbol_id = interner.intern(name);
        Value::symbol(symbol_id)
    }
}

#[cfg(test)]
mod error_handling_integration_tests {
    use super::*;
    use lambdust::stdlib::exceptions::ExceptionObject;

    fn setup_condition_system() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).expect("Failed to initialize condition system");
    }

    #[test]
    fn test_raise_with_condition() {
        setup_condition_system();
        
        // Create a condition
        let condition = primitive_make_message_condition(&[Value::string("Test error".to_string())]).unwrap();
        
        // Convert to exception for raising
        if let Value::Condition(condition_value) = condition {
            let exception = ExceptionObject::from_condition(condition_value, false);
            
            // Verify the exception has the condition
            assert!(exception.has_condition());
            assert_eq!(exception.exception_type, "error");
            assert_eq!(exception.message, Some("Test error".to_string()));
            
            // Test that we can get the condition back
            let recovered_condition = exception.get_condition();
            assert!(recovered_condition.is_some());
        } else {
            panic!("Expected condition value");
        }
    }

    #[test]
    fn test_legacy_exception_to_condition_conversion() {
        setup_condition_system();
        
        // Create legacy exception
        let exception = ExceptionObject::error("Legacy error".to_string(), &[Value::integer(123)]);
        
        // Convert to condition
        let condition = exception.to_condition().unwrap();
        
        // Verify condition properties
        let system = condition_system();
        assert!(condition.condition.has_type(system.standard_types.error, &system.registry));
        
        let message_field = system.registry.interner.intern("message");
        let message_value = condition.condition.condition_ref(message_field, &system.registry);
        assert!(message_value.is_some());
        
        match message_value.unwrap() {
            Value::Literal(lambdust::ast::Literal::String(s)) => assert_eq!(s, "Legacy error"),
            _ => panic!("Expected string message"),
        }
    }
}