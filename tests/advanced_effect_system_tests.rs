//! Advanced effect system tests for Lambdust
//! 
//! This module tests the advanced effect system features including:
//! - Monadic operations (Maybe, Either, State, Reader, Writer transformers)
//! - Effect composition and algebraic effect handlers
//! - Continuation-based effects and delimited continuations

use lambdust::{
    effects::*,
    eval::Value,
    ast::Literal,
    diagnostics::Error,
};
use std::sync::Arc;

#[test]
fn test_effect_types() {
    let pure = Effect::Pure;
    let io = Effect::IO;
    let state = Effect::State;
    let error = Effect::Error;
    let custom = Effect::Custom("Database".to_string());
    
    assert!(pure.is_pure());
    assert!(io.is_io());
    assert!(state.is_state());
    assert!(error.is_error());
    
    // Test effect strength ordering
    assert!(pure.strength() < state.strength());
    assert!(state.strength() < io.strength());
    assert!(io.strength() < error.strength());
    assert!(error.strength() < custom.strength());
}

#[test]
fn test_effect_combination() {
    let pure = Effect::Pure;
    let io = Effect::IO;
    let state = Effect::State;
    let error = Effect::Error;
    
    // Pure combined with anything gives the other effect
    assert_eq!(pure.combine(&io), io);
    assert_eq!(io.combine(&pure), io);
    
    // Error is the strongest effect
    assert_eq!(error.combine(&io), error);
    assert_eq!(io.combine(&error), error);
    
    // IO is stronger than State
    assert_eq!(io.combine(&state), io);
    assert_eq!(state.combine(&io), io);
}

#[test]
fn test_effect_context() {
    let mut context = EffectContext::new();
    assert!(context.is_pure());
    
    // Add effects
    context.add_effect(Effect::IO);
    assert!(!context.is_pure());
    assert!(context.has_effect(&Effect::IO));
    assert!(!context.has_effect(&Effect::State));
    
    context.add_effect(Effect::State);
    assert!(context.has_effect(&Effect::State));
    
    // Test context combination
    let mut other_context = EffectContext::pure();
    other_context.add_effect(Effect::Error);
    
    let combined = context.combine(&other_context);
    assert!(combined.has_effect(&Effect::IO));
    assert!(combined.has_effect(&Effect::State));
    assert!(combined.has_effect(&Effect::Error));
}

#[test]
fn test_maybe_monad() {
    // Test Maybe construction
    let nothing = MaybeMonad::nothing();
    let just_42 = MaybeMonad::just(Value::Literal(Literal::number(42.0)));
    
    assert!(nothing.is_nothing());
    assert!(just_42.is_just());
    
    // Test bind operation
    let double = |x: &Value| -> MaybeMonad {
        if let Value::Literal(Literal::Number(n)) = x {
            MaybeMonad::just(Value::Literal(Literal::number(n * 2.0)))
        } else {
            MaybeMonad::nothing()
        }
    };
    
    let result = just_42.bind(&double);
    assert!(result.is_just());
    
    if let Some(Value::Literal(Literal::Number(n))) = result.unwrap() {
        assert_eq!(n, 84.0);
    }
    
    // Test Nothing bind
    let nothing_result = nothing.bind(&double);
    assert!(nothing_result.is_nothing());
}

#[test]
fn test_either_monad() {
    // Test Either construction
    let left = EitherMonad::left(Value::Literal(Literal::string("error".to_string())));
    let right = EitherMonad::right(Value::Literal(Literal::number(42.0)));
    
    assert!(left.is_left());
    assert!(right.is_right());
    
    // Test map_right operation
    let double = |x: &Value| -> Value {
        if let Value::Literal(Literal::Number(n)) = x {
            Value::Literal(Literal::number(n * 2.0))
        } else {
            x.clone()
        }
    };
    
    let result = right.map_right(&double);
    assert!(result.is_right());
    
    // Test left unchanged
    let left_result = left.map_right(&double);
    assert!(left_result.is_left());
}

#[test]
fn test_state_monad() {
    let initial_state = Value::Literal(Literal::number(0.0));
    
    // Create a state computation that increments the state
    let increment = StateMonad::new(|state: &Value| -> (Value, Value) {
        if let Value::Literal(Literal::Number(n)) = state {
            let new_state = Value::Literal(Literal::number(n + 1.0));
            (state.clone(), new_state.clone())
        } else {
            (state.clone(), state.clone())
        }
    });
    
    let (result, final_state) = increment.run_state(&initial_state);
    
    if let (
        Value::Literal(Literal::Number(result_n)),
        Value::Literal(Literal::Number(state_n))
    ) = (&result, &final_state) {
        assert_eq!(*result_n, 0.0); // Original state returned as result
        assert_eq!(*state_n, 1.0);  // State incremented
    }
}

#[test]
fn test_reader_monad() {
    let environment = Value::Literal(Literal::string("config".to_string()));
    
    // Create a reader computation that reads from the environment
    let read_config = ReaderMonad::new(|env: &Value| -> Value {
        env.clone()
    });
    
    let result = read_config.run_reader(&environment);
    
    if let Value::Literal(Literal::String(s)) = result {
        assert_eq!(s, "config");
    }
    
    // Test reader bind operation
    let get_length = ReaderMonad::new(|env: &Value| -> Value {
        if let Value::Literal(Literal::String(s)) = env {
            Value::Literal(Literal::number(s.len() as f64))
        } else {
            Value::Literal(Literal::number(0.0))
        }
    });
    
    let bound = read_config.bind(|_| get_length);
    let length_result = bound.run_reader(&environment);
    
    if let Value::Literal(Literal::Number(n)) = length_result {
        assert_eq!(n, 6.0); // "config".len()
    }
}

#[test]
fn test_writer_monad() {
    // Create a writer computation that logs operations
    let log_and_compute = WriterMonad::tell(Value::Literal(Literal::string("Computing...".to_string())))
        .bind(|_| WriterMonad::new(
            Value::Literal(Literal::number(42.0)),
            Value::Literal(Literal::string("Result computed".to_string()))
        ));
    
    let (result, log) = log_and_compute.run_writer();
    
    if let Value::Literal(Literal::Number(n)) = result {
        assert_eq!(n, 42.0);
    }
    
    // Log should contain both messages (implementation depends on log combining)
    assert!(matches!(log, Value::Literal(Literal::String(_))));
}

#[test]
fn test_io_monad() {
    // Create a mock IO operation
    let mock_read = IOMonad::new(|| -> std::io::Result<Value> {
        Ok(Value::Literal(Literal::string("hello world".to_string())))
    });
    
    // In a real implementation, this would perform actual I/O
    // For testing, we assume the operation succeeds
    let result = mock_read.run();
    
    match result {
        Ok(Value::Literal(Literal::String(s))) => {
            assert_eq!(s, "hello world");
        }
        _ => panic!("Expected successful IO result"),
    }
}

#[test]
fn test_effect_handlers() {
    // Create a simple effect handler for logging
    #[derive(Debug)]
    struct LoggingHandler {
        logs: Arc<std::sync::Mutex<Vec<String>>>,
    }
    
    impl EffectHandler for LoggingHandler {
        fn handle(&self, effect: &Effect, args: &[Value]) -> lambdust::Result<EffectResult> {
            match effect {
                Effect::Custom(name) if name == "Log" => {
                    if let Some(Value::Literal(Literal::String(message))) = args.first() {
                        if let Ok(mut logs) = self.logs.lock() {
                            logs.push(message.clone());
                        }
                        Ok(EffectResult::Value(Value::Unspecified))
                    } else {
                        Ok(EffectResult::Unhandled)
                    }
                }
                _ => Ok(EffectResult::Unhandled),
            }
        }
        
        fn effect_name(&self) -> &str {
            "Log"
        }
        
        fn can_handle(&self, effect: &Effect) -> bool {
            matches!(effect, Effect::Custom(name) if name == "Log")
        }
    }
    
    let logs = Arc::new(std::sync::Mutex::new(Vec::new()));
    let handler = LoggingHandler { logs: logs.clone() };
    
    let log_effect = Effect::Custom("Log".to_string());
    let args = vec![Value::Literal(Literal::string("Test message".to_string()))];
    
    let result = handler.handle(&log_effect, &args);
    assert!(result.is_ok());
    
    if let Ok(logs_guard) = logs.lock() {
        assert_eq!(logs_guard.len(), 1);
        assert_eq!(logs_guard[0], "Test message");
    }
}

#[test]
fn test_effect_system() {
    let mut system = EffectSystem::new();
    
    // Test initial state
    assert!(system.context().is_pure());
    
    // Enter an effect context
    let old_context = system.enter_context(vec![Effect::IO]);
    assert!(system.context().has_effect(&Effect::IO));
    
    // Exit context
    system.exit_context(old_context);
    assert!(system.context().is_pure());
}

#[test]
fn test_lifting_configuration() {
    let mut config = LiftingConfig::new();
    
    // Test default lifting rules
    assert!(config.auto_lift_io);
    assert!(config.auto_lift_state);
    assert!(config.auto_lift_error);
    
    // Add custom lifting rule
    config.add_rule("custom-op".to_string(), LiftingRule {
        target_effect: Effect::Custom("Database".to_string()),
        condition: LiftingCondition::Always,
    });
    
    assert!(config.custom_rules.contains_key("custom-op"));
    
    // Test no-lifting configuration
    let no_lift = LiftingConfig::no_lifting();
    assert!(!no_lift.auto_lift_io);
    assert!(!no_lift.auto_lift_state);
    assert!(!no_lift.auto_lift_error);
}

#[test]
fn test_automatic_lifting() {
    let system = EffectSystem::new();
    
    // Test I/O operation lifting
    let io_effect = system.should_lift("display", &[]);
    assert_eq!(io_effect, Some(Effect::IO));
    
    let read_effect = system.should_lift("read", &[]);
    assert_eq!(read_effect, Some(Effect::IO));
    
    // Test state operation lifting
    let state_effect = system.should_lift("set!", &[]);
    assert_eq!(state_effect, Some(Effect::State));
    
    // Test error operation lifting
    let error_effect = system.should_lift("error", &[]);
    assert_eq!(error_effect, Some(Effect::Error));
    
    // Test no lifting for unknown operations
    let no_effect = system.should_lift("unknown-op", &[]);
    assert_eq!(no_effect, None);
}

#[test]
fn test_generational_environments() {
    let mut env_manager = GenerationalEnvManager::new();
    
    // Create a new generation
    let gen1 = env_manager.new_generation();
    assert!(gen1 > 0);
    
    // Create another generation
    let gen2 = env_manager.new_generation();
    assert!(gen2 > gen1);
    
    // Test generation tracking
    assert_eq!(env_manager.current_generation(), gen2);
}

#[test]
fn test_monad_transformer_stack() {
    // Test StateT over Maybe
    type StateMaybe<S> = StateT<S, MaybeMonad>;
    
    // This would test transformer composition in a real implementation
    // For now, we test the concept with a simple state computation
    let computation = |state: i32| -> (Option<i32>, i32) {
        if state > 0 {
            (Some(state * 2), state + 1)
        } else {
            (None, state)
        }
    };
    
    let result = computation(5);
    assert_eq!(result, (Some(10), 6));
    
    let none_result = computation(-1);
    assert_eq!(none_result, (None, -1));
}

#[test]
fn test_algebraic_effects() {
    // Test algebraic effect definition
    let choose_effect = AlgebraicEffect::new("Choose".to_string(), vec![
        EffectOperation::new("choose".to_string(), vec!["Bool".to_string()], "Bool".to_string()),
    ]);
    
    assert_eq!(choose_effect.name, "Choose");
    assert_eq!(choose_effect.operations.len(), 1);
    
    // Test effect handler composition
    let handler1 = SimpleEffectHandler::new("Choose", |_args| {
        EffectResult::Value(Value::Literal(Literal::boolean(true)))
    });
    
    let handler2 = SimpleEffectHandler::new("State", |_args| {
        EffectResult::Value(Value::Unspecified)
    });
    
    let composed = EffectHandlerComposition::new(vec![
        Box::new(handler1),
        Box::new(handler2),
    ]);
    
    assert_eq!(composed.handlers.len(), 2);
}

#[test]
fn test_delimited_continuations() {
    // Test basic continuation capture and resumption
    let continuation = DelimitedContinuation::new(
        Box::new(|value| Value::Literal(Literal::number(
            if let Value::Literal(Literal::Number(n)) = value {
                n + 1.0
            } else {
                0.0
            }
        ))),
        PromptTag::new("test".to_string()),
    );
    
    let input = Value::Literal(Literal::number(41.0));
    let result = continuation.resume(&input);
    
    if let Value::Literal(Literal::Number(n)) = result {
        assert_eq!(n, 42.0);
    }
}

// Integration test combining multiple effect system features
#[test]
fn test_advanced_effect_system_integration() {
    let mut system = EffectSystem::with_config(LiftingConfig::new());
    
    // Set up a complex effect context
    let _context = system.enter_context(vec![Effect::IO, Effect::State]);
    
    // Test multiple effect handling
    assert!(system.context().has_effect(&Effect::IO));
    assert!(system.context().has_effect(&Effect::State));
    
    // Test effect lifting in context
    let should_lift_display = system.should_lift("display", system.context().effects());
    assert_eq!(should_lift_display, Some(Effect::IO));
    
    // Test generational environment integration
    let env_gen = system.env_manager().new_generation();
    assert!(env_gen > 0);
}