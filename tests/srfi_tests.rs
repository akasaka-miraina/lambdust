//! SRFI (Scheme Request for Implementation) integration tests
//! 
//! Tests for the SRFI module system and individual SRFI implementations.

use lambdust::{Interpreter, Value, SrfiRegistry, ModuleSystem};

#[cfg(test)]
mod srfi_module_system_tests {
    use super::*;

    #[test]
    fn test_srfi_registry() {
        let registry = SrfiRegistry::with_standard_srfis();
        let available = registry.available_srfis();
        
        // Standard SRFIs should be available
        assert!(available.contains(&9));  // SRFI 9 (define-record-type)
        assert!(available.contains(&45)); // SRFI 45 (lazy evaluation)
        assert!(available.contains(&46)); // SRFI 46 (syntax-rules extensions)
    }

    #[test]
    fn test_module_system() {
        let module_system = ModuleSystem::new();
        let available = module_system.available_srfis();
        
        assert!(available.len() >= 3);
        assert!(available.contains(&9));
        assert!(available.contains(&45));
        assert!(available.contains(&46));
    }

    #[test]
    fn test_srfi_info() {
        let registry = SrfiRegistry::with_standard_srfis();
        
        if let Some((id, name, parts)) = registry.get_srfi_info(9) {
            assert_eq!(id, 9);
            assert_eq!(name, "Defining Record Types");
            assert!(parts.contains(&"records"));
            assert!(parts.contains(&"types"));
        } else {
            panic!("SRFI 9 should be available");
        }
        
        if let Some((id, name, parts)) = registry.get_srfi_info(45) {
            assert_eq!(id, 45);
            assert_eq!(name, "Primitives for Expressing Iterative Lazy Algorithms");
            assert!(parts.contains(&"lazy"));
            assert!(parts.contains(&"promises"));
        } else {
            panic!("SRFI 45 should be available");
        }
    }
}

#[cfg(test)]
mod srfi_9_tests {
    use super::*;

    #[test]
    fn test_srfi_9_record_operations() {
        let mut interpreter = Interpreter::new();
        
        // Test record creation and access
        let result = interpreter.eval("(make-record 'person '(\"Alice\" 30))").unwrap();
        assert!(matches!(result, Value::Record(_)));
        
        // Test record type checking
        let result = interpreter.eval("(record-of-type? (make-record 'person '()) 'person)").unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        let result = interpreter.eval("(record-of-type? 42 'person)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[cfg(test)]
mod srfi_45_tests {
    use super::*;

    #[test]
    fn test_srfi_45_lazy_evaluation() {
        let mut interpreter = Interpreter::new();
        
        // Test delay
        let result = interpreter.eval("(delay (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));
        
        // Test lazy
        let result = interpreter.eval("(lazy (+ 1 2))").unwrap();
        assert!(matches!(result, Value::Promise(_)));
        
        // Test promise?
        let result = interpreter.eval("(promise? (delay 42))").unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        let result = interpreter.eval("(promise? 42)").unwrap();
        assert_eq!(result, Value::Boolean(false));
        
        // Test force (basic)
        let result = interpreter.eval("(force 42)").unwrap();
        assert_eq!(result, Value::from(42i64));
    }
}

#[cfg(test)]
mod srfi_46_tests {
    use super::*;

    #[test]
    fn test_srfi_46_syntax_extensions() {
        // SRFI 46 provides macro syntax extensions
        // Since it doesn't export runtime functions, we test its presence
        let registry = SrfiRegistry::with_standard_srfis();
        assert!(registry.has_srfi(46));
        
        if let Some((id, name, parts)) = registry.get_srfi_info(46) {
            assert_eq!(id, 46);
            assert_eq!(name, "Basic Syntax-rules Extensions");
            assert!(parts.contains(&"syntax"));
            assert!(parts.contains(&"ellipsis"));
        }
    }
}

#[cfg(test)]
mod srfi_import_syntax_tests {
    use super::*;
    
    // Note: These tests would require implementing the actual import syntax parser
    // in the main interpreter. For now, we test the module system components.
    
    #[test]
    fn test_import_parsing_readiness() {
        // Test that the module system can handle import specifications
        let module_system = ModuleSystem::new();
        
        // Verify we have the infrastructure for import handling
        assert!(!module_system.available_srfis().is_empty());
        
        // Test SRFI availability checking
        assert!(module_system.srfi_info(9).is_some());
        assert!(module_system.srfi_info(45).is_some());
        assert!(module_system.srfi_info(46).is_some());
        assert!(module_system.srfi_info(999).is_none());
    }
}