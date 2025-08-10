//! Runtime system for the Lambdust language.
//!
//! This module provides both single-threaded and multithreaded runtime systems
//! for evaluating Lambdust programs. The main entry points are:
//!
//! - `Runtime`: Legacy single-threaded runtime
//! - `LambdustRuntime`: New multithreaded runtime with parallel evaluation

mod evaluator;
mod thread_pool;
mod global_env;
mod io_coordinator;
mod error_propagation;
mod bootstrap;
mod bootstrap_integration;
mod library_path_resolver;
mod primitive_bridge; // Re-enabled for containers integration
// mod integration_example; // Temporarily disabled due to compilation issues

// Effect coordinator component modules
mod effect_coordinator_main;
mod thread_effect_state;
mod effect_event;
mod effect_policies;
mod concurrent_effect_system;
mod effect_transaction;
mod effect_dependency_graph;
mod effect_ordering_manager;
mod pending_effect;
mod ordering_constraint;
mod effect_channel;
mod effect_coordination_message;
mod effect_statistics;
mod effect_isolation_level;
mod effect_sandbox_config;
mod effect_isolation_statistics;
mod effect_context_extensions;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod bootstrap_integration_tests;

// Individual structure modules
mod runtime;
mod lambdust_runtime;
mod evaluator_handle;
mod evaluator_message;
mod parallel_result;

pub use evaluator::MultithreadedEvaluator;
pub use thread_pool::ThreadPool;
pub use global_env::GlobalEnvironmentManager;
pub use io_coordinator::IOCoordinator;
pub use error_propagation::ErrorPropagationCoordinator;
pub use bootstrap::{BootstrapSystem, BootstrapStatistics, MinimalPrimitivesRegistry};
pub use crate::module_system::BootstrapConfig;
pub use bootstrap_integration::{BootstrapIntegration, BootstrapIntegrationConfig, BootstrapMetrics, BootstrapMode};
pub use library_path_resolver::{LibraryPathResolver, LibraryPathConfig, LibraryValidationReport};
// Note: integration_example has compilation issues - temporarily disabled
pub use primitive_bridge::{MinimalPrimitiveRegistry, MinimalPrimitive, MinimalPrimitiveCategory};
// pub use integration_example::{IntegrationExample, IntegrationMetrics, run_integration_example};

// Effect coordinator component exports
pub use effect_coordinator_main::EffectCoordinator;
pub use thread_effect_state::ThreadEffectState;
pub use effect_event::{EffectEvent, EffectEventType};
pub use effect_policies::EffectPolicies;
pub use concurrent_effect_system::ConcurrentEffectSystem;
pub use effect_transaction::{EffectTransaction, TransactionState};
pub use effect_dependency_graph::EffectDependencyGraph;
pub use effect_ordering_manager::EffectOrderingManager;
pub use pending_effect::PendingEffect;
pub use ordering_constraint::{OrderingConstraint, ConstraintType};
pub use effect_channel::EffectChannel;
pub use effect_coordination_message::EffectCoordinationMessage;
pub use effect_statistics::EffectStatistics;
pub use effect_isolation_level::{EffectIsolationLevel, EffectIsolationRules, IsolationException};
pub use effect_sandbox_config::{EffectSandboxConfig, EffectSandboxHandle, SandboxResourceLimits, SandboxStatistics, ResourceUsage};
pub use effect_isolation_statistics::EffectIsolationStatistics;

// Re-export individual structures
pub use runtime::*;
pub use lambdust_runtime::*;
pub use evaluator_handle::*;
pub use evaluator_message::*;
pub use parallel_result::*;

