//! Module system integration tests for macro export/import functionality

use lambdust::module_system::ModuleSystem;
use lambdust::macros::{Macro, SyntaxRulesTransformer, SyntaxRule, Pattern, Template};
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_macro() -> Macro {
        let literals = vec![];
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("test-macro".to_string()),
                Pattern::Variable("x".to_string()),
            ]),
            template: Template::List(vec![
                Template::Literal("result".to_string()),
                Template::Variable("x".to_string()),
            ]),
        }];
        
        let transformer = SyntaxRulesTransformer::new(literals, rules);
        
        Macro::SyntaxRules {
            name: "test-macro".to_string(),
            transformer,
        }
    }

    #[test]
    fn test_module_system_creation() {
        let module_system = ModuleSystem::new();
        
        // Should have empty macro collections initially
        assert_eq!(module_system.get_imported_macros().len(), 0);
        assert_eq!(module_system.get_exported_macros().len(), 0);
        assert_eq!(module_system.get_exported_bindings().len(), 0);
    }

    #[test]
    fn test_macro_export() {
        let mut module_system = ModuleSystem::new();
        let test_macro = create_test_macro();
        
        // Export macro
        let result = module_system.export_macro("test-macro".to_string(), test_macro);
        assert!(result.is_ok());
        
        // Check if macro is available
        assert!(module_system.has_macro("test-macro"));
        assert!(module_system.get_macro("test-macro").is_some());
        
        // Check exported macros collection
        assert_eq!(module_system.get_exported_macros().len(), 1);
        assert!(module_system.get_exported_macros().contains_key("test-macro"));
    }

    #[test]
    fn test_macro_import() {
        let mut module_system = ModuleSystem::new();
        let test_macro = create_test_macro();
        
        // Import macro
        let result = module_system.import_macro("test-macro".to_string(), test_macro);
        assert!(result.is_ok());
        
        // Check if macro is available
        assert!(module_system.has_macro("test-macro"));
        assert!(module_system.get_macro("test-macro").is_some());
        
        // Check imported macros collection
        assert_eq!(module_system.get_imported_macros().len(), 1);
        assert!(module_system.get_imported_macros().contains_key("test-macro"));
    }

    #[test]
    fn test_macro_export_conflict() {
        let mut module_system = ModuleSystem::new();
        let test_macro1 = create_test_macro();
        let test_macro2 = create_test_macro();
        
        // Export first macro
        let result1 = module_system.export_macro("test-macro".to_string(), test_macro1);
        assert!(result1.is_ok());
        
        // Try to export second macro with same name - should fail
        let result2 = module_system.export_macro("test-macro".to_string(), test_macro2);
        assert!(result2.is_err());
        
        // Should still have only one macro
        assert_eq!(module_system.get_exported_macros().len(), 1);
    }

    #[test]
    fn test_macro_import_same_macro() {
        let mut module_system = ModuleSystem::new();
        let test_macro1 = create_test_macro();
        let test_macro2 = create_test_macro();
        
        // Import first macro
        let result1 = module_system.import_macro("test-macro".to_string(), test_macro1);
        assert!(result1.is_ok());
        
        // Import same macro again - should succeed (equivalent macros)
        let result2 = module_system.import_macro("test-macro".to_string(), test_macro2);
        assert!(result2.is_ok());
        
        // Should still have only one macro
        assert_eq!(module_system.get_imported_macros().len(), 1);
    }

    #[test]
    fn test_binding_export() {
        let mut module_system = ModuleSystem::new();
        let test_value = Value::Number(SchemeNumber::Integer(42));
        
        // Export binding
        let result = module_system.export_binding("test-binding".to_string(), test_value);
        assert!(result.is_ok());
        
        // Check exported bindings collection
        assert_eq!(module_system.get_exported_bindings().len(), 1);
        assert!(module_system.get_exported_bindings().contains_key("test-binding"));
        
        // Check value
        if let Some(Value::Number(SchemeNumber::Integer(val))) = module_system.get_exported_bindings().get("test-binding") {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected integer value");
        }
    }

    #[test]
    fn test_binding_export_conflict() {
        let mut module_system = ModuleSystem::new();
        let test_value1 = Value::Number(SchemeNumber::Integer(42));
        let test_value2 = Value::Number(SchemeNumber::Integer(100));
        
        // Export first binding
        let result1 = module_system.export_binding("test-binding".to_string(), test_value1);
        assert!(result1.is_ok());
        
        // Try to export second binding with same name - should fail
        let result2 = module_system.export_binding("test-binding".to_string(), test_value2);
        assert!(result2.is_err());
        
        // Should still have only one binding
        assert_eq!(module_system.get_exported_bindings().len(), 1);
        
        // Original value should be preserved
        if let Some(Value::Number(SchemeNumber::Integer(val))) = module_system.get_exported_bindings().get("test-binding") {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected original integer value");
        }
    }

    #[test]
    fn test_macro_lookup_precedence() {
        let mut module_system = ModuleSystem::new();
        let imported_macro = create_test_macro();
        let exported_macro = create_test_macro();
        
        // Import macro
        module_system.import_macro("test-macro".to_string(), imported_macro).unwrap();
        
        // Export macro with same name
        module_system.export_macro("test-macro".to_string(), exported_macro).unwrap();
        
        // Both should be available
        assert!(module_system.has_macro("test-macro"));
        assert!(module_system.get_macro("test-macro").is_some());
        
        // Should have both in their respective collections
        assert_eq!(module_system.get_imported_macros().len(), 1);
        assert_eq!(module_system.get_exported_macros().len(), 1);
    }

    #[test]
    fn test_macro_not_found() {
        let module_system = ModuleSystem::new();
        
        // Check for non-existent macro
        assert!(!module_system.has_macro("non-existent"));
        assert!(module_system.get_macro("non-existent").is_none());
    }

    #[test]
    fn test_multiple_macros() {
        let mut module_system = ModuleSystem::new();
        let macro1 = create_test_macro();
        let macro2 = create_test_macro();
        
        // Export multiple macros
        module_system.export_macro("macro1".to_string(), macro1).unwrap();
        module_system.import_macro("macro2".to_string(), macro2).unwrap();
        
        // Both should be available
        assert!(module_system.has_macro("macro1"));
        assert!(module_system.has_macro("macro2"));
        
        // Check collections
        assert_eq!(module_system.get_exported_macros().len(), 1);
        assert_eq!(module_system.get_imported_macros().len(), 1);
    }

    #[test]
    fn test_integration_with_existing_functionality() {
        let module_system = ModuleSystem::new();
        
        // Test that existing SRFI functionality still works
        let available_srfis = module_system.available_srfis();
        assert!(!available_srfis.is_empty());
        
        // Test that bindings functionality still works
        assert!(!module_system.has_binding("non-existent"));
        assert!(module_system.get_binding("non-existent").is_none());
        
        // Test that imported bindings collection is still accessible
        assert_eq!(module_system.imported_bindings().len(), 0);
    }
}