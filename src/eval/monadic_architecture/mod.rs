//! Clean Architecture implementation for monadic evaluator.
//!
//! This module implements a clean separation between:
//! 1. Domain Logic: Pure monadic computations and mathematical operations
//! 2. Application Services: Orchestration and business use cases
//! 3. Infrastructure: IO operations, persistence, external system integration
//!
//! The architecture follows the Dependency Inversion Principle:
//! Domain <- Application <- Infrastructure

// ================================
// DOMAIN LAYER - Core Business Logic
// ================================
pub mod monadic_computation;
pub mod monadic_transformation;
pub mod monad_type;
pub mod monad_service;
pub mod monad_configuration;
pub mod custom_monad_definition;

// ================================
// APPLICATION LAYER - Use Cases and Orchestration
// ================================
pub mod monadic_evaluation_orchestrator;
pub mod orchestrator_configuration;
pub mod evaluate_monadic_expression_use_case;
pub mod monadic_evaluation_input;
pub mod monadic_evaluation_result;
pub mod evaluation_metadata;
pub mod evaluation_metrics;

// ================================
// INFRASTRUCTURE LAYER - External Systems and Persistence
// ================================
pub mod continuation_repository;
pub mod effect_interpreter;
pub mod environment_manager;
pub mod in_memory_continuation_repository;
pub mod repository_configuration;
pub mod default_effect_interpreter;
pub mod interpreter_configuration;
pub mod default_environment_manager;
pub mod environment_manager_configuration;

// Re-exports for public API
pub use monadic_computation::MonadicComputation;
pub use monadic_transformation::MonadicTransformation;
pub use monad_type::MonadType;
pub use monad_service::MonadService;
pub use monad_configuration::MonadConfiguration;
pub use custom_monad_definition::{CustomMonadDefinition, MonadicContinuation, MonadicBindFunction};

pub use monadic_evaluation_orchestrator::MonadicEvaluationOrchestrator;
pub use orchestrator_configuration::OrchestratorConfiguration;
pub use evaluate_monadic_expression_use_case::EvaluateMonadicExpressionUseCase;
pub use monadic_evaluation_input::MonadicEvaluationInput;
pub use monadic_evaluation_result::MonadicEvaluationResult;
pub use evaluation_metadata::EvaluationMetadata;
pub use evaluation_metrics::EvaluationMetrics;

pub use continuation_repository::ContinuationRepository;
pub use effect_interpreter::EffectInterpreter;
pub use environment_manager::EnvironmentManager;
pub use in_memory_continuation_repository::InMemoryContinuationRepository;
pub use repository_configuration::RepositoryConfiguration;
pub use default_effect_interpreter::DefaultEffectInterpreter;
pub use interpreter_configuration::InterpreterConfiguration;
pub use default_environment_manager::DefaultEnvironmentManager;
pub use environment_manager_configuration::EnvironmentManagerConfiguration;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_monad_service_pure() {
        let service = MonadService::new();
        let computation = service.pure(42);
        
        match computation {
            MonadicComputation::Pure(value) => assert_eq!(value, 42),
            _ => panic!("Expected pure computation"),
        }
    }
    
    #[test]
    fn test_in_memory_continuation_repository() {
        let mut repo = InMemoryContinuationRepository {
            continuations: HashMap::new(),
            config: RepositoryConfiguration::default(),
        };
        
        // This would normally create a real continuation
        // For testing, we'll skip the actual creation
        assert_eq!(repo.list_all().len(), 0);
    }
}