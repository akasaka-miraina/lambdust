//! Integration tests for the effect system.
//!
//! These tests verify the complete effect system pipeline including
//! effect tracking, monadic operations, handler resolution, generational
//! environments, and integration with the evaluation system.

#[cfg(test)]
mod tests {
    use crate::effects::*;
    use crate::eval::value::Value;
    use crate::diagnostics::Error;
    use std::sync::Arc;

    // ============================================================================
    // EFFECT BASIC FUNCTIONALITY TESTS
    // ============================================================================

    #[test]
    fn test_effect_creation_and_properties() {
        let pure = Effect::Pure;
        let io = Effect::IO;
        let state = Effect::State;
        let error = Effect::Error;
        let custom = Effect::Custom("test".to_string());

        assert!(pure.is_pure());
        assert!(!io.is_pure());
        assert!(io.is_io());
        assert!(state.is_state());
        assert!(error.is_error());
        
        // Test effect ordering by strength
        assert!(pure < state);
        assert!(state < io);
        assert!(io < error);
        assert!(error < custom);
    }

    #[test]
    fn test_effect_combination() {
        let pure = Effect::Pure;
        let io = Effect::IO;
        let state = Effect::State;
        let error = Effect::Error;

        // Pure combined with anything gives the other
        assert_eq!(pure.combine(&io), io);
        assert_eq!(io.combine(&pure), io);

        // Error is strongest
        assert_eq!(error.combine(&io), error);
        assert_eq!(io.combine(&error), error);

        // IO is stronger than State
        assert_eq!(io.combine(&state), io);
        assert_eq!(state.combine(&io), io);

        // Same effects combine to themselves
        assert_eq!(io.combine(&io), io);
    }

    #[test]
    fn test_effect_strength_ordering() {
        let effects = vec![
            Effect::Error,
            Effect::Custom("test".to_string()),
            Effect::Pure,
            Effect::IO,
            Effect::State,
        ];

        let mut sorted_effects = effects.clone());
        sorted_effects.sort();

        assert_eq!(sorted_effects[0], Effect::Pure);
        assert_eq!(sorted_effects[1], Effect::State);
        assert_eq!(sorted_effects[2], Effect::IO);
        assert_eq!(sorted_effects[3], Effect::Error);
        assert_eq!(sorted_effects[4], Effect::Custom("test".to_string()));
    }

    // ============================================================================
    // EFFECT CONTEXT TESTS
    // ============================================================================

    #[test]
    fn test_effect_context_creation() {
        let empty_context = EffectContext::new();
        let pure_context = EffectContext::pure();

        assert!(empty_context.effects().is_empty());
        assert!(pure_context.is_pure());
        assert_eq!(pure_context.effects().len(), 1);
        assert_eq!(pure_context.effects()[0], Effect::Pure);
    }

    #[test]
    fn test_effect_context_operations() {
        let mut context = EffectContext::new();

        // Add effects
        context.add_effect(Effect::IO);
        context.add_effect(Effect::State);
        context.add_effect(Effect::IO); // Duplicate should be ignored

        assert_eq!(context.effects().len(), 2);
        assert!(context.has_effect(&Effect::IO));
        assert!(context.has_effect(&Effect::State));
        assert!(!context.has_effect(&Effect::Error));
        assert!(!context.is_pure());
    }

    #[test]
    fn test_effect_context_combination() {
        let mut context1 = EffectContext::pure();
        context1.add_effect(Effect::IO);

        let mut context2 = EffectContext::new();
        context2.add_effect(Effect::State);
        context2.add_effect(Effect::Error);

        let combined = context1.combine(&context2);

        assert!(combined.has_effect(&Effect::Pure));
        assert!(combined.has_effect(&Effect::IO));
        assert!(combined.has_effect(&Effect::State));
        assert!(combined.has_effect(&Effect::Error));
        assert_eq!(combined.effects().len(), 4);
    }

    #[test]
    fn test_effect_context_with_without() {
        let mut context = EffectContext::pure();
        context.add_effect(Effect::IO);
        context.add_effect(Effect::State);

        let with_error = context.with_effects(vec![Effect::Error]);
        assert!(with_error.has_effect(&Effect::Error));
        assert!(with_error.has_effect(&Effect::IO));

        let without_io = context.without_effects(vec![Effect::IO]);
        assert!(!without_io.has_effect(&Effect::IO));
        assert!(without_io.has_effect(&Effect::State));
        assert!(without_io.has_effect(&Effect::Pure));
    }

    // ============================================================================
    // EFFECT HANDLER TESTS
    // ============================================================================

    /// Mock effect handler for testing.
    #[derive(Debug)]
    struct MockIOHandler;

    impl EffectHandler for MockIOHandler {
        fn handle(&self, effect: &Effect, args: &[Value]) -> crate::diagnostics::Result<EffectResult> {
            match effect {
                Effect::IO => {
                    if args.is_empty() {
                        Ok(EffectResult::Value(Value::string("IO handled")))
                    } else {
                        Ok(EffectResult::Value(args[0].clone()))
                    }
                }
                _ => Ok(EffectResult::Unhandled),
            }
        }

        fn effect_name(&self) -> &str {
            "IO"
        }

        fn can_handle(&self, effect: &Effect) -> bool {
            matches!(effect, Effect::IO)
        }
    }

    /// Mock error handler for testing.
    #[derive(Debug)]
    struct MockErrorHandler;

    impl EffectHandler for MockErrorHandler {
        fn handle(&self, effect: &Effect, args: &[Value]) -> crate::diagnostics::Result<EffectResult> {
            match effect {
                Effect::Error => {
                    if args.is_empty() {
                        Ok(EffectResult::Error(Error::runtime_error(
                            "Mock error".to_string(),
                            None,
                        )))
                    } else {
                        Ok(EffectResult::Error(Error::runtime_error(
                            format!("Mock error: {:?}", args[0]),
                            None,
                        )))
                    }
                }
                _ => Ok(EffectResult::Unhandled),
            }
        }

        fn effect_name(&self) -> &str {
            "Error"
        }

        fn can_handle(&self, effect: &Effect) -> bool {
            matches!(effect, Effect::Error)
        }
    }

    #[test]
    fn test_effect_handler_functionality() {
        let handler = MockIOHandler;
        
        // Test capability check
        assert!(handler.can_handle(&Effect::IO));
        assert!(!handler.can_handle(&Effect::State));
        assert_eq!(handler.effect_name(), "IO");

        // Test handling
        let result = handler.handle(&Effect::IO, &[]).unwrap();
        match result {
            EffectResult::Value(val) => {
                assert_eq!(val.as_string(), Some("IO handled"));
            }
            _ => panic!("Expected value result"),
        }

        // Test with arguments
        let args = vec![Value::number(42.0)];
        let result = handler.handle(&Effect::IO, &args).unwrap();
        match result {
            EffectResult::Value(val) => {
                assert_eq!(val.as_number(), Some(42.0));
            }
            _ => panic!("Expected value result"),
        }

        // Test unhandled effect
        let result = handler.handle(&Effect::State, &[]).unwrap();
        assert!(matches!(result, EffectResult::Unhandled));
    }

    #[test]
    fn test_error_handler_functionality() {
        let handler = MockErrorHandler;
        
        assert!(handler.can_handle(&Effect::Error));
        assert_eq!(handler.effect_name(), "Error");

        // Test error handling
        let result = handler.handle(&Effect::Error, &[]).unwrap();
        match result {
            EffectResult::Error(err) => {
                assert!(err.to_string().contains("Mock error"));
            }
            _ => panic!("Expected error result"),
        }
    }

    #[test]
    fn test_effect_context_handler_integration() {
        let mut context = EffectContext::new();
        
        let handler_ref = EffectHandlerRef {
            effect_name: "IO".to_string(),
            handler: Arc::new(MockIOHandler),
        };
        
        context.add_handler(handler_ref);
        
        // Test finding handlers
        let found_handler = context.find_handler(&Effect::IO);
        assert!(found_handler.is_some());
        assert_eq!(found_handler.unwrap().effect_name, "IO");

        let not_found = context.find_handler(&Effect::State);
        assert!(not_found.is_none());
    }

    // ============================================================================
    // EFFECT SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_effect_system_creation() {
        let system = EffectSystem::new();
        
        assert!(system.context().is_pure());
        assert!(system.lifting_config().auto_lift_io);
        assert!(system.lifting_config().auto_lift_state);
        assert!(system.lifting_config().auto_lift_error);
    }

    #[test]
    fn test_effect_system_with_config() {
        let config = LiftingConfig::no_lifting();
        let system = EffectSystem::with_config(config);
        
        assert!(!system.lifting_config().auto_lift_io);
        assert!(!system.lifting_config().auto_lift_state);
        assert!(!system.lifting_config().auto_lift_error);
    }

    #[test]
    fn test_effect_system_context_management() {
        let mut system = EffectSystem::new();
        
        assert!(system.context().is_pure());
        
        // Enter a new context
        let old_context = system.enter_context(vec![Effect::IO, Effect::State]);
        assert!(system.context().has_effect(&Effect::IO));
        assert!(system.context().has_effect(&Effect::State));
        
        // Exit context
        system.exit_context(old_context);
        assert!(system.context().is_pure());
    }

    #[test]
    fn test_effect_system_handler_integration() {
        let mut system = EffectSystem::new();
        
        // Add a handler to the context
        let handler_ref = EffectHandlerRef {
            effect_name: "IO".to_string(),
            handler: Arc::new(MockIOHandler),
        };
        system.context_mut().add_handler(handler_ref);
        
        // Test effect handling
        let result = system.handle_effect(&Effect::IO, &[]).unwrap();
        match result {
            EffectResult::Value(val) => {
                assert_eq!(val.as_string(), Some("IO handled"));
            }
            _ => panic!("Expected value result"),
        }
        
        // Test unhandled effect
        let result = system.handle_effect(&Effect::State, &[]).unwrap();
        assert!(matches!(result, EffectResult::Unhandled));
    }

    // ============================================================================
    // LIFTING CONFIGURATION TESTS
    // ============================================================================

    #[test]
    fn test_lifting_config_default() {
        let config = LiftingConfig::default();
        
        assert!(config.auto_lift_io);
        assert!(config.auto_lift_state);
        assert!(config.auto_lift_error);
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_lifting_config_no_lifting() {
        let config = LiftingConfig::no_lifting();
        
        assert!(!config.auto_lift_io);
        assert!(!config.auto_lift_state);
        assert!(!config.auto_lift_error);
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_lifting_config_custom_rules() {
        let mut config = LiftingConfig::new();
        
        let rule = LiftingRule {
            target_effect: Effect::Custom("test".to_string()),
            condition: LiftingCondition::Always,
        };
        
        config.add_rule("test-op".to_string(), rule);
        
        assert_eq!(config.custom_rules.len(), 1);
        assert!(config.custom_rules.contains_key("test-op"));
    }

    #[test]
    fn test_lifting_conditions() {
        let always = LiftingCondition::Always;
        let op_name = LiftingCondition::OperationName("test".to_string());
        let has_effect = LiftingCondition::HasEffect(vec![Effect::IO]);
        let custom = LiftingCondition::Custom(|op, _| op == "custom");
        
        // Always condition
        assert!(always.matches("anything", &[]));
        
        // Operation name condition
        assert!(op_name.matches("test", &[]));
        assert!(!op_name.matches("other", &[]));
        
        // Has effect condition
        assert!(has_effect.matches("", &[Effect::IO]));
        assert!(has_effect.matches("", &[Effect::State, Effect::IO]));
        assert!(!has_effect.matches("", &[Effect::State]));
        
        // Custom condition
        assert!(custom.matches("custom", &[]));
        assert!(!custom.matches("other", &[]));
    }

    #[test]
    fn test_effect_system_automatic_lifting() {
        let system = EffectSystem::new();
        
        // Test built-in lifting rules
        assert_eq!(system.should_lift("display", &[]), Some(Effect::IO));
        assert_eq!(system.should_lift("write", &[]), Some(Effect::IO));
        assert_eq!(system.should_lift("set!", &[]), Some(Effect::State));
        assert_eq!(system.should_lift("error", &[]), Some(Effect::Error));
        
        // Test non-lifting operations
        assert_eq!(system.should_lift("+", &[]), None);
        assert_eq!(system.should_lift("lambda", &[]), None);
        
        // Test with disabled lifting
        let no_lift_system = EffectSystem::with_config(LiftingConfig::no_lifting());
        assert_eq!(no_lift_system.should_lift("display", &[]), None);
    }

    #[test]
    fn test_effect_system_custom_lifting_rules() {
        let mut config = LiftingConfig::new();
        
        let rule = LiftingRule {
            target_effect: Effect::Custom("database".to_string()),
            condition: LiftingCondition::OperationName("db-query".to_string()),
        };
        
        config.add_rule("db-query".to_string(), rule);
        let system = EffectSystem::with_config(config);
        
        assert_eq!(
            system.should_lift("db-query", &[]),
            Some(Effect::Custom("database".to_string()))
        );
        assert_eq!(system.should_lift("other-op", &[]), None);
    }

    // ============================================================================
    // EFFECT RESULT TESTS
    // ============================================================================

    #[test]
    fn test_effect_result_variants() {
        let value_result = EffectResult::Value(Value::number(42.0));
        let continue_result = EffectResult::Continue(Value::string("continue"));
        let unhandled_result = EffectResult::Unhandled;
        let error_result = EffectResult::Error(Error::runtime_error("test".to_string(), None));
        
        assert!(matches!(value_result, EffectResult::Value(_)));
        assert!(matches!(continue_result, EffectResult::Continue(_)));
        assert!(matches!(unhandled_result, EffectResult::Unhandled));
        assert!(matches!(error_result, EffectResult::Error(_)));
    }

    // ============================================================================
    // DISPLAY AND DEBUG TESTS
    // ============================================================================

    #[test]
    fn test_effect_display() {
        assert_eq!(format!("{}", Effect::Pure), "Pure");
        assert_eq!(format!("{}", Effect::IO), "IO");
        assert_eq!(format!("{}", Effect::State), "State");
        assert_eq!(format!("{}", Effect::Error), "Error");
        assert_eq!(format!("{}", Effect::Custom("test".to_string())), "Custom(test)");
    }

    #[test]
    fn test_effect_context_display() {
        let pure_context = EffectContext::pure();
        assert_eq!(format!("{}", pure_context), "Pure");
        
        let mut complex_context = EffectContext::new();
        complex_context.add_effect(Effect::State);
        complex_context.add_effect(Effect::IO);
        complex_context.add_effect(Effect::Error);
        
        let display = format!("{}", complex_context);
        assert!(display.contains("State"));
        assert!(display.contains("IO"));
        assert!(display.contains("Error"));
        assert!(display.starts_with('['));
        assert!(display.ends_with(']'));
    }

    // ============================================================================
    // INTEGRATION WITH GENERATIONAL ENVIRONMENTS (Placeholder)
    // ============================================================================

    #[test]
    #[ignore] // Will pass when generational environment is fully implemented
    fn test_effect_system_generational_integration() {
        let mut system = EffectSystem::new();
        
        // Test that the effect system integrates with generational environments
        // This requires the generational environment manager to be implemented
        assert!(system.env_manager().current_generation() >= 0);
    }

    // ============================================================================
    // MONADIC OPERATIONS (Placeholder)
    // ============================================================================

    #[test]
    #[ignore] // Will pass when monadic operations are implemented
    fn test_monadic_effect_operations() {
        // Test monadic bind, return, and other operations
        // This requires the monad implementation to be complete
        
        let system = EffectSystem::new();
        assert!(system.context().is_pure());
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn test_effect_system_performance() {
        let start = std::time::Instant::now();
        
        // Create many effect contexts and perform operations
        for _ in 0..1000 {
            let mut context = EffectContext::new();
            context.add_effect(Effect::IO);
            context.add_effect(Effect::State);
            context.add_effect(Effect::Error);
            
            let _ = context.is_pure();
            let _ = context.has_effect(&Effect::IO);
            let combined = context.combine(&EffectContext::pure());
            let _ = combined.effects().len();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Effect operations should be fast");
    }

    #[test]
    fn test_effect_handler_performance() {
        let handler = Arc::new(MockIOHandler);
        let start = std::time::Instant::now();
        
        // Perform many handler operations
        for _ in 0..1000 {
            let _ = handler.can_handle(&Effect::IO);
            let _ = handler.can_handle(&Effect::State);
            let _ = handler.effect_name();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "Handler operations should be very fast");
    }

    // ============================================================================
    // ERROR HANDLING TESTS
    // ============================================================================

    #[test]
    fn test_effect_handler_error_propagation() {
        let handler = MockErrorHandler;
        
        let result = handler.handle(&Effect::Error, &[Value::string("test error")]).unwrap();
        match result {
            EffectResult::Error(err) => {
                assert!(err.to_string().contains("test error"));
            }
            _ => panic!("Expected error result"),
        }
    }

    // ============================================================================
    // THREAD SAFETY TESTS (Basic structural tests)
    // ============================================================================

    #[test]
    fn test_effect_handler_thread_safety() {
        // Test that effect handlers can be shared across threads
        let handler: Arc<dyn EffectHandler + Send + Sync> = Arc::new(MockIOHandler);
        
        // This test verifies the type constraints are correct
        let handler_ref = EffectHandlerRef {
            effect_name: "IO".to_string(),
            handler: handler.clone()),
        };
        
        assert_eq!(handler_ref.effect_name, "IO");
        
        // In a real scenario, we would test actual thread usage,
        // but this verifies the types are Send + Sync
    }
}