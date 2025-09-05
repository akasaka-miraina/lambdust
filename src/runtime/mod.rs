//! Runtime system for the Lambdust language.
//!
//! This module provides both single-threaded and multithreaded runtime systems
//! for evaluating Lambdust programs. The main entry points are:
//!
//! - `Runtime`: Legacy single-threaded runtime
//! - `LambdustRuntime`: New multithreaded runtime with parallel evaluation

mod bootstrap;
mod bootstrap_integration;
mod error_propagation;
mod evaluator;
mod global_env;
mod io_coordinator;
mod library_path_resolver;
mod primitive_bridge;
mod thread_pool; // Re-enabled for containers integration
// mod integration_example; // Temporarily disabled due to compilation issues

// Parallel generational garbage collector
pub mod gc;

// Effect coordinator component modules
mod concurrent_effect_system;
mod effect_channel;
mod effect_context_extensions;
mod effect_coordination_message;
mod effect_coordinator_main;
mod effect_dependency_graph;
mod effect_event;
mod effect_isolation_level;
mod effect_isolation_statistics;
mod effect_ordering_manager;
mod effect_policies;
mod effect_sandbox_config;
mod effect_statistics;
mod effect_transaction;
mod ordering_constraint;
mod pending_effect;
mod thread_effect_state;

#[cfg(test)]
mod tests;

// #[cfg(test)]
// mod bootstrap_integration_tests;

// Individual structure modules
mod evaluator_handle;
mod evaluator_message;
mod lambdust_runtime;
mod parallel_result;
pub mod runtime;

pub use crate::module_system::BootstrapConfig;
pub use bootstrap::{BootstrapStatistics, BootstrapSystem, MinimalPrimitivesRegistry};
pub use bootstrap_integration::{
    BootstrapIntegration, BootstrapIntegrationConfig, BootstrapMetrics, BootstrapMode,
};
pub use error_propagation::ErrorPropagationCoordinator;
pub use evaluator::MultithreadedEvaluator;
pub use global_env::GlobalEnvironmentManager;
pub use io_coordinator::IOCoordinator;
pub use library_path_resolver::{LibraryPathConfig, LibraryPathResolver, LibraryValidationReport};
pub use thread_pool::ThreadPool;
// Note: integration_example has compilation issues - temporarily disabled
pub use primitive_bridge::{MinimalPrimitive, MinimalPrimitiveCategory, MinimalPrimitiveRegistry};
// pub use integration_example::{IntegrationExample, IntegrationMetrics, run_integration_example};

// Effect coordinator component exports
pub use concurrent_effect_system::ConcurrentEffectSystem;
pub use effect_channel::EffectChannel;
pub use effect_coordination_message::EffectCoordinationMessage;
pub use effect_coordinator_main::EffectCoordinator;
pub use effect_dependency_graph::EffectDependencyGraph;
pub use effect_event::{EffectEvent, EffectEventType};
pub use effect_isolation_level::{EffectIsolationLevel, EffectIsolationRules, IsolationException};
pub use effect_isolation_statistics::EffectIsolationStatistics;
pub use effect_ordering_manager::EffectOrderingManager;
pub use effect_policies::EffectPolicies;
pub use effect_sandbox_config::{
    EffectSandboxConfig, EffectSandboxHandle, ResourceUsage, SandboxResourceLimits,
    SandboxStatistics,
};
pub use effect_statistics::EffectStatistics;
pub use effect_transaction::{EffectTransaction, TransactionState};
pub use ordering_constraint::{ConstraintType, OrderingConstraint};
pub use pending_effect::PendingEffect;
pub use thread_effect_state::ThreadEffectState;

// Parallel generational garbage collector exports
pub use gc::{
    AllocationCoordinator, CollectionResult, GcConfigBuilder, GcSystem, GcSystemStatistics,
    GenerationId, GenerationManager, ObjectHeader, ParallelGc, ParallelGcConfig,
};

// Re-export individual structures
pub use evaluator_handle::*;
pub use evaluator_message::*;
pub use lambdust_runtime::*;
pub use parallel_result::*;
pub use runtime::*;
