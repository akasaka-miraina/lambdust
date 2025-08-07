//! Testing architecture with dependency injection and mocking support.
//!
//! This module provides a comprehensive testing framework for the monadic evaluator,
//! including dependency injection, mock implementations, and test utilities
//! that enable thorough unit and integration testing.

use crate::eval::{
    Value, Environment,
    monadic_architecture::{
        MonadicComputation, ContinuationRepository, EffectInterpreter, EnvironmentManager,
        MonadicEvaluationInput, MonadicEvaluationResult,
    },
    continuation_domain::{
        CapturedContinuation, ContinuationId,
        ContinuationCaptureService, ContinuationApplicationService, ContinuationCompositionService,
    },
    effect_integration::{
        UnifiedEffectRegistry, UnifiedEffectHandler, EffectHandlerMetadata,
    },
};
use crate::effects::{
    Effect, EffectContext, EffectfulComputation,
};
use crate::diagnostics::{Result, Error};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// Dependency injection container for the monadic evaluator.
///
/// This container manages all dependencies and allows for easy substitution
/// of mock implementations during testing.
pub struct DIContainer {
    /// Registered dependencies by type name
    dependencies: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    
    /// Singleton instances
    singletons: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
    
    /// Factory functions for creating instances
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn std::any::Any + Send + Sync> + Send + Sync>>,
    
    /// Configuration for the container
    config: DIConfiguration,
}

impl std::fmt::Debug for DIContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DIContainer {{ dependencies: <{}>, singletons: <{}>, factories: <{}>, config: {:?} }}",
               self.dependencies.len(),
               self.singletons.len(), 
               self.factories.len(),
               self.config)
    }
}

/// Configuration for dependency injection
#[derive(Debug, Clone)]
pub struct DIConfiguration {
    /// Whether to enable automatic dependency resolution
    pub auto_resolve: bool,
    
    /// Whether to cache resolved instances
    pub enable_caching: bool,
    
    /// Whether to enable circular dependency detection
    pub detect_circular_deps: bool,
    
    /// Maximum resolution depth
    pub max_resolution_depth: usize,
}

/// Mock continuation repository for testing
#[derive(Debug)]
pub struct MockContinuationRepository {
    /// Mock storage
    storage: Arc<Mutex<HashMap<ContinuationId, CapturedContinuation>>>,
    
    /// Mock behavior configuration
    behavior: MockRepositoryBehavior,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<RepositoryCall>>>,
}

/// Mock behavior configuration for repository
#[derive(Debug, Clone)]
pub struct MockRepositoryBehavior {
    /// Whether store operations should fail
    pub store_should_fail: bool,
    
    /// Whether find operations should fail
    pub find_should_fail: bool,
    
    /// Simulated storage capacity
    pub max_capacity: Option<usize>,
    
    /// Simulated latency (in milliseconds)
    pub simulated_latency_ms: u64,
}

/// Repository call tracking for assertions
#[derive(Debug, Clone)]
pub enum RepositoryCall {
    Store(ContinuationId),
    Find(ContinuationId),
    Remove(ContinuationId),
    List,
    GarbageCollect(u64),
}

/// Mock effect interpreter for testing
#[derive(Debug)]
pub struct MockEffectInterpreter {
    /// Mock responses for different effects
    responses: Arc<Mutex<HashMap<Effect, MockEffectResponse>>>,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EffectCall>>>,
    
    /// Behavior configuration
    behavior: MockEffectBehavior,
}

/// Mock response for effect interpretation
#[derive(Debug, Clone)]
pub enum MockEffectResponse {
    /// Return a specific value
    Value(Value),
    
    /// Return an error
    Error(String),
    
    /// Return a monadic computation
    Computation(MockMonadicComputation),
    
    /// Delay and then return a value
    DelayedValue(Value, u64), // value, delay in ms
}

/// Simplified mock monadic computation
#[derive(Debug, Clone)]
pub enum MockMonadicComputation {
    Pure(Value),
    IO(Value),
    Maybe(Option<Value>),
    Either(std::result::Result<Value, String>),
}

/// Effect interpretation call tracking
#[derive(Debug, Clone)]
pub struct EffectCall {
    pub effect: Effect,
    pub args: Vec<Value>,
    pub timestamp: std::time::SystemTime,
}

/// Mock behavior for effect interpreter
#[derive(Debug, Clone)]
pub struct MockEffectBehavior {
    /// Whether to fail on unknown effects
    pub fail_on_unknown: bool,
    
    /// Simulated processing time
    pub processing_time_ms: u64,
    
    /// Whether to enable async simulation
    pub simulate_async: bool,
}

/// Mock environment manager for testing
#[derive(Debug)]
pub struct MockEnvironmentManager {
    /// Environment storage
    environments: Arc<Mutex<HashMap<u64, Rc<Environment>>>>,
    
    /// Environment ID counter
    next_id: Arc<Mutex<u64>>,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EnvironmentCall>>>,
    
    /// Behavior configuration
    behavior: MockEnvironmentBehavior,
}

/// Environment operation call tracking
#[derive(Debug, Clone)]
pub enum EnvironmentCall {
    Create(Option<u64>), // parent ID
    Clone(u64),          // environment ID
    Extend(u64),         // environment ID
    Lookup(u64, String), // environment ID, variable name
    Update(u64, String), // environment ID, variable name
}

/// Mock behavior for environment manager
#[derive(Debug, Clone)]
pub struct MockEnvironmentBehavior {
    /// Whether lookups should fail for specific variables
    pub failing_lookups: Vec<String>,
    
    /// Whether updates should fail
    pub updates_should_fail: bool,
    
    /// Maximum number of environments to track
    pub max_environments: Option<usize>,
}

/// Mock effect handler for testing specific effects
#[derive(Debug)]
pub struct MockEffectHandler {
    /// Name of the effect this handler manages
    name: String,
    
    /// Effects this handler can process
    supported_effects: Vec<Effect>,
    
    /// Mock responses for different effect-argument combinations
    responses: Arc<Mutex<HashMap<(Effect, Vec<Value>), MockEffectResponse>>>,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EffectHandlerCall>>>,
    
    /// Behavior configuration
    behavior: MockHandlerBehavior,
}

/// Effect handler call tracking
#[derive(Debug, Clone)]
pub struct EffectHandlerCall {
    pub effect: Effect,
    pub args: Vec<Value>,
    pub context: EffectContext,
    pub timestamp: std::time::SystemTime,
}

/// Mock behavior for effect handler
#[derive(Debug, Clone)]
pub struct MockHandlerBehavior {
    /// Whether to simulate processing delays
    pub simulate_delays: bool,
    
    /// Base processing time
    pub base_processing_time_ms: u64,
    
    /// Whether to enable failure simulation
    pub enable_failures: bool,
    
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
}

/// Test fixture builder for setting up test scenarios
#[derive(Debug)]
pub struct TestFixtureBuilder {
    /// DI container being built
    container: DIContainer,
    
    /// Mock configurations
    mock_configs: HashMap<String, MockConfiguration>,
}

/// Configuration for mock components
#[derive(Debug, Clone)]
pub enum MockConfiguration {
    Repository(MockRepositoryBehavior),
    EffectInterpreter(MockEffectBehavior),
    EnvironmentManager(MockEnvironmentBehavior),
    EffectHandler(MockHandlerBehavior),
    Custom(HashMap<String, String>),
}

/// Test scenario definition for integration tests
#[derive(Debug, Clone)]
pub struct TestScenario {
    /// Name of the scenario
    pub name: String,
    
    /// Description of what's being tested
    pub description: String,
    
    /// Input for the test
    pub input: MonadicEvaluationInput,
    
    /// Expected outcome
    pub expected_outcome: ExpectedOutcome,
    
    /// Mock configurations for this scenario
    pub mock_configs: HashMap<String, MockConfiguration>,
    
    /// Additional assertions
    pub assertions: Vec<TestAssertion>,
}

/// Expected outcome of a test
#[derive(Debug, Clone)]
pub enum ExpectedOutcome {
    /// Expect successful evaluation with specific value
    Success(Value),
    
    /// Expect successful evaluation with any value
    SuccessAny,
    
    /// Expect error with specific message
    Error(String),
    
    /// Expect error of any kind
    ErrorAny,
    
    /// Expect specific monadic computation
    MonadicComputation(MockMonadicComputation),
    
    /// Custom validation function
    Custom(fn(&MonadicEvaluationResult) -> bool),
}

/// Test assertion for verifying behavior
#[derive(Debug, Clone)]
pub enum TestAssertion {
    /// Assert that a specific method was called
    MethodCalled(String, usize), // method name, call count
    
    /// Assert performance characteristics
    PerformanceWithin {
        max_time_ms: u64,
        max_memory_bytes: usize,
    },
    
    /// Assert effect usage
    EffectsUsed(Vec<Effect>),
    
    /// Assert continuation capture
    ContinuationsCaptured(usize),
    
    /// Custom assertion function
    Custom(fn(&TestExecutionContext) -> bool),
}

/// Context available during test assertion evaluation
#[derive(Debug)]
pub struct TestExecutionContext {
    /// The test result
    pub result: MonadicEvaluationResult,
    
    /// Mock call logs
    pub call_logs: HashMap<String, Vec<String>>,
    
    /// Performance metrics
    pub metrics: TestMetrics,
}

/// Test-specific metrics
#[derive(Debug, Clone)]
pub struct TestMetrics {
    /// Total execution time
    pub execution_time_ms: u64,
    
    /// Memory usage
    pub memory_used_bytes: usize,
    
    /// Number of dependency resolutions
    pub dependency_resolutions: usize,
    
    /// Number of mock calls
    pub total_mock_calls: usize,
}

// ================================
// IMPLEMENTATION
// ================================

impl DIContainer {
    /// Create a new DI container
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            singletons: HashMap::new(),
            factories: HashMap::new(),
            config: DIConfiguration::default(),
        }
    }
    
    /// Register a dependency instance
    pub fn register<T: Send + Sync + 'static>(&mut self, name: &str, instance: T) {
        self.dependencies.insert(
            name.to_string(),
            Box::new(instance)
        );
    }
    
    /// Register a singleton dependency
    pub fn register_singleton<T: Send + Sync + 'static>(&mut self, name: &str, instance: T) {
        self.singletons.insert(
            name.to_string(),
            Arc::new(instance)
        );
    }
    
    /// Register a factory function
    pub fn register_factory<T: Send + Sync + 'static, F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.factories.insert(
            name.to_string(),
            Box::new(move || Box::new(factory()))
        );
    }
    
    /// Resolve a dependency by name
    pub fn resolve<T: Send + Sync + 'static>(&self, name: &str) -> Option<&T> {
        // Try singleton first
        if let Some(singleton) = self.singletons.get(name) {
            return singleton.downcast_ref::<T>();
        }
        
        // Try regular dependency
        if let Some(dependency) = self.dependencies.get(name) {
            return dependency.downcast_ref::<T>();
        }
        
        None
    }
    
    /// Create instance using factory
    pub fn create<T: Send + Sync + 'static>(&self, name: &str) -> Option<T> {
        if let Some(factory) = self.factories.get(name) {
            let instance = factory();
            return instance.downcast::<T>().ok().map(|boxed| *boxed);
        }
        None
    }
    
    /// Check if a dependency is registered
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.contains_key(name) ||
        self.singletons.contains_key(name) ||
        self.factories.contains_key(name)
    }
    
    /// List all registered dependencies
    pub fn list_dependencies(&self) -> Vec<String> {
        let mut deps: Vec<String> = self.dependencies.keys().clone())().collect();
        deps.extend(self.singletons.keys().clone())());
        deps.extend(self.factories.keys().clone())());
        deps.sort();
        deps.dedup();
        deps
    }
}

// Mock implementations

impl MockContinuationRepository {
    /// Create a new mock repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            behavior: MockRepositoryBehavior::default(),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create with custom behavior
    pub fn with_behavior(behavior: MockRepositoryBehavior) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            behavior,
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get call log for assertions
    pub fn call_log(&self) -> Vec<RepositoryCall> {
        self.call_log.lock().unwrap().clone())
    }
    
    /// Clear call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
    
    /// Get number of stored continuations
    pub fn storage_size(&self) -> usize {
        self.storage.lock().unwrap().len()
    }
}

impl ContinuationRepository for MockContinuationRepository {
    fn store(&mut self, continuation: CapturedContinuation) -> Result<ContinuationId> {
        // Log the call
        self.call_log.lock().unwrap().push(RepositoryCall::Store(continuation.id));
        
        // Simulate failure if configured
        if self.behavior.store_should_fail {
            return Err(Box::new(Error::runtime_error(
                "Mock repository configured to fail".to_string(),
                None,
            ));
        }
        
        // Check capacity
        if let Some(max_cap) = self.behavior.max_capacity {
            if self.storage.lock().unwrap().len() >= max_cap {
                return Err(Box::new(Error::runtime_error(
                    "Mock repository at capacity".to_string(),
                    None,
                ));
            }
        }
        
        // Simulate latency
        if self.behavior.simulated_latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.behavior.simulated_latency_ms));
        }
        
        let id = continuation.id;
        self.storage.lock().unwrap().insert(id, continuation);
        Ok(id)
    }
    
    fn find_by_id(&self, id: ContinuationId) -> Option<CapturedContinuation> {
        // Log the call
        self.call_log.lock().unwrap().push(RepositoryCall::Find(id));
        
        // Simulate failure if configured
        if self.behavior.find_should_fail {
            return None;
        }
        
        // Simulate latency
        if self.behavior.simulated_latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.behavior.simulated_latency_ms));
        }
        
        self.storage.lock().unwrap().get(&id).clone())()
    }
    
    fn remove(&mut self, id: ContinuationId) -> Result<()> {
        self.call_log.lock().unwrap().push(RepositoryCall::Remove(id));
        self.storage.lock().unwrap().remove(&id);
        Ok(())
    }
    
    fn list_all(&self) -> Vec<ContinuationId> {
        self.call_log.lock().unwrap().push(RepositoryCall::List);
        self.storage.lock().unwrap().keys().copied().collect()
    }
    
    fn garbage_collect(&mut self, current_generation: u64) -> Result<usize> {
        self.call_log.lock().unwrap().push(RepositoryCall::GarbageCollect(current_generation));
        
        let initial_size = self.storage.lock().unwrap().len();
        
        // Simple GC simulation - remove old continuations
        self.storage.lock().unwrap().retain(|_id, cont| {
            cont.metadata.generation + 5 > current_generation // Keep recent ones
        });
        
        let final_size = self.storage.lock().unwrap().len();
        Ok(initial_size - final_size)
    }
}

impl MockEffectInterpreter {
    /// Create a new mock effect interpreter
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
            behavior: MockEffectBehavior::default(),
        }
    }
    
    /// Add a mock response for a specific effect
    pub fn add_response(&self, effect: Effect, response: MockEffectResponse) {
        self.responses.lock().unwrap().insert(effect, response);
    }
    
    /// Get call log for assertions
    pub fn call_log(&self) -> Vec<EffectCall> {
        self.call_log.lock().unwrap().clone())
    }
    
    /// Clear call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
}

#[async_trait]
impl EffectInterpreter for MockEffectInterpreter {
    async fn interpret(&self, effect: EffectfulComputation) -> Result<Value> {
        // Log the call (simplified)
        let effect_call = EffectCall {
            effect: Effect::IO, // Simplified - would extract from EffectfulComputation
            args: vec![],       // Simplified
            timestamp: std::time::SystemTime::now(),
        };
        
        self.call_log.lock().unwrap().push(effect_call);
        
        // Simulate processing time
        if self.behavior.processing_time_ms > 0 {
            tokio::time::sleep(
                tokio::time::Duration::from_millis(self.behavior.processing_time_ms)
            ).await;
        }
        
        // Return mock response or default
        Ok(Value::string("mock effect result".to_string()))
    }
    
    fn can_interpret(&self, effect: &Effect) -> bool {
        !self.behavior.fail_on_unknown ||
        self.responses.lock().unwrap().contains_key(effect)
    }
    
    fn available_effects(&self) -> Vec<Effect> {
        self.responses.lock().unwrap().keys().clone())().collect()
    }
}

impl TestFixtureBuilder {
    /// Create a new test fixture builder
    pub fn new() -> Self {
        Self {
            container: DIContainer::new(),
            mock_configs: HashMap::new(),
        }
    }
    
    /// Add a mock continuation repository
    pub fn with_mock_continuation_repository(mut self, behavior: MockRepositoryBehavior) -> Self {
        let mock_repo = MockContinuationRepository::with_behavior(behavior.clone());
        // Note: Mock repository disabled due to Send trait requirements
        // In a production system, we would use Arc<ThreadSafeEnvironment> throughout
        // self.container.register("continuation_repository", mock_repo);
        self.mock_configs.insert("continuation_repository".to_string(), MockConfiguration::Repository(behavior));
        self
    }
    
    /// Add a mock effect interpreter
    pub fn with_mock_effect_interpreter(mut self, behavior: MockEffectBehavior) -> Self {
        let mock_interpreter = MockEffectInterpreter::new();
        self.container.register("effect_interpreter", mock_interpreter);
        self.mock_configs.insert("effect_interpreter".to_string(), MockConfiguration::EffectInterpreter(behavior));
        self
    }
    
    /// Build the test fixture
    pub fn build(self) -> TestFixture {
        TestFixture {
            container: self.container,
            mock_configs: self.mock_configs,
        }
    }
}

/// Complete test fixture with all mocks configured
#[derive(Debug)]
pub struct TestFixture {
    /// DI container with mock dependencies
    pub container: DIContainer,
    
    /// Mock configurations used
    pub mock_configs: HashMap<String, MockConfiguration>,
}

impl TestFixture {
    /// Execute a test scenario
    pub async fn execute_scenario(&self, scenario: TestScenario) -> TestResult {
        let start_time = std::time::Instant::now();
        
        // TODO: Execute the actual evaluation using the mock dependencies
        // For now, return a mock result
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let mock_result = MonadicEvaluationResult {
            computation: MonadicComputation::Pure(Value::Unspecified),
            metadata: crate::eval::monadic_architecture::EvaluationMetadata {
                steps_taken: 1,
                max_stack_depth: 1,
                monads_used: vec![],
                tail_call_optimized: false,
            },
            effects: vec![],
            metrics: crate::eval::monadic_architecture::EvaluationMetrics {
                evaluation_time_ns: execution_time * 1_000_000,
                memory_allocated: 1024,
                continuations_captured: 0,
                io_operations: 0,
            },
        };
        
        TestResult {
            scenario: scenario.clone()),
            execution_result: Ok(mock_result.clone()),
            execution_time_ms: execution_time,
            assertions_passed: Vec::new(),
            assertions_failed: Vec::new(),
            mock_call_counts: HashMap::new(),
        }
    }
    
    /// Get a specific mock from the container
    pub fn get_mock<T: Send + Sync + 'static>(&self, name: &str) -> Option<&T> {
        self.container.resolve::<T>(name)
    }
}

/// Result of executing a test scenario
#[derive(Debug)]
pub struct TestResult {
    /// The scenario that was executed
    pub scenario: TestScenario,
    
    /// Result of the evaluation
    pub execution_result: Result<MonadicEvaluationResult>,
    
    /// Time taken to execute
    pub execution_time_ms: u64,
    
    /// Assertions that passed
    pub assertions_passed: Vec<TestAssertion>,
    
    /// Assertions that failed
    pub assertions_failed: Vec<TestAssertion>,
    
    /// Mock call counts for verification
    pub mock_call_counts: HashMap<String, usize>,
}

impl TestResult {
    /// Check if the test passed
    pub fn passed(&self) -> bool {
        self.assertions_failed.is_empty() && self.execution_result.is_ok()
    }
    
    /// Get a summary of the test result
    pub fn summary(&self) -> String {
        format!(
            "Test '{}': {} ({}ms, {}/{} assertions passed)",
            self.scenario.name,
            if self.passed() { "PASSED" } else { "FAILED" },
            self.execution_time_ms,
            self.assertions_passed.len(),
            self.assertions_passed.len() + self.assertions_failed.len()
        )
    }
}

// Default implementations

impl Default for DIConfiguration {
    fn default() -> Self {
        Self {
            auto_resolve: true,
            enable_caching: true,
            detect_circular_deps: true,
            max_resolution_depth: 10,
        }
    }
}

impl Default for MockRepositoryBehavior {
    fn default() -> Self {
        Self {
            store_should_fail: false,
            find_should_fail: false,
            max_capacity: None,
            simulated_latency_ms: 0,
        }
    }
}

impl Default for MockEffectBehavior {
    fn default() -> Self {
        Self {
            fail_on_unknown: false,
            processing_time_ms: 0,
            simulate_async: true,
        }
    }
}

impl Default for MockEnvironmentBehavior {
    fn default() -> Self {
        Self {
            failing_lookups: Vec::new(),
            updates_should_fail: false,
            max_environments: None,
        }
    }
}

impl Default for MockHandlerBehavior {
    fn default() -> Self {
        Self {
            simulate_delays: false,
            base_processing_time_ms: 0,
            enable_failures: false,
            failure_rate: 0.0,
        }
    }
}

// Test utilities and helper functions

/// Create a basic test scenario
pub fn create_basic_scenario(name: &str, input: MonadicEvaluationInput) -> TestScenario {
    TestScenario {
        name: name.to_string(),
        description: format!("Basic test scenario: {}", name),
        input,
        expected_outcome: ExpectedOutcome::SuccessAny,
        mock_configs: HashMap::new(),
        assertions: Vec::new(),
    }
}

/// Create a test scenario for call/cc testing
pub fn create_call_cc_scenario(
    name: &str,
    input: MonadicEvaluationInput,
    expected_continuations: usize,
) -> TestScenario {
    let mut scenario = create_basic_scenario(name, input);
    scenario.description = format!("Call/cc test scenario: {}", name);
    scenario.assertions.push(TestAssertion::ContinuationsCaptured(expected_continuations));
    scenario
}

/// Create a test scenario for IO testing
pub fn create_io_scenario(
    name: &str,
    input: MonadicEvaluationInput,
    expected_io_ops: usize,
) -> TestScenario {
    let mut scenario = create_basic_scenario(name, input);
    scenario.description = format!("IO test scenario: {}", name);
    scenario.assertions.push(TestAssertion::MethodCalled("io_operation".to_string(), expected_io_ops));
    scenario
}

/// Create a performance test scenario
pub fn create_performance_scenario(
    name: &str,
    input: MonadicEvaluationInput,
    max_time_ms: u64,
    max_memory_bytes: usize,
) -> TestScenario {
    let mut scenario = create_basic_scenario(name, input);
    scenario.description = format!("Performance test scenario: {}", name);
    scenario.assertions.push(TestAssertion::PerformanceWithin {
        max_time_ms,
        max_memory_bytes,
    });
    scenario
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::diagnostics::Span;
    use crate::eval::operational_semantics::EvaluationContext;
    
    #[test]
    fn test_di_container() {
        let mut container = DIContainer::new();
        
        // Register a mock repository
        let mock_repo = MockContinuationRepository::new();
        container.register("repository", mock_repo);
        
        // Verify registration
        assert!(container.has_dependency("repository"));
        
        // Try to resolve
        let resolved: Option<&MockContinuationRepository> = container.resolve("repository");
        assert!(resolved.is_some());
    }
    
    #[test]
    fn test_mock_continuation_repository() {
        let mut mock_repo = MockContinuationRepository::new();
        
        // Create a dummy continuation for testing
        let env = Rc::new(Environment::new(None, 0));
        let context = EvaluationContext::empty(env.clone());
        let capture_service = crate::eval::continuation_domain::ContinuationCaptureService::new();
        
        let continuation = capture_service.capture_continuation(
            &context,
            Span::default(),
            0,
        ).unwrap();
        
        // Test store operation
        let id = mock_repo.store(continuation.clone()).unwrap();
        assert_eq!(mock_repo.storage_size(), 1);
        
        // Test find operation
        let found = mock_repo.find_by_id(id);
        assert!(found.is_some());
        
        // Verify call log
        let calls = mock_repo.call_log();
        assert_eq!(calls.len(), 2); // store + find
    }
    
    #[test]
    fn test_test_fixture_builder() {
        let fixture = TestFixtureBuilder::new()
            .with_mock_continuation_repository(MockRepositoryBehavior::default())
            .with_mock_effect_interpreter(MockEffectBehavior::default())
            .build();
        
        assert!(fixture.container.has_dependency("continuation_repository"));
        assert!(fixture.container.has_dependency("effect_interpreter"));
    }
    
    #[tokio::test]
    async fn test_mock_effect_interpreter() {
        let interpreter = MockEffectInterpreter::new();
        
        // Add a mock response
        interpreter.add_response(Effect::IO, MockEffectResponse::Value(Value::string("test".to_string())));
        
        // Test interpretation
        let effect_computation = EffectfulComputation::IO {
            action: crate::effects::continuation_monad::ContIOAction::Write(Value::string("test".to_string())),
        };
        
        let result = interpreter.interpret(effect_computation).await;
        assert!(result.is_ok());
        
        // Verify call log
        let calls = interpreter.call_log();
        assert_eq!(calls.len(), 1);
    }
    
    #[test]
    fn test_scenario_creation() {
        let env = Rc::new(Environment::new(None, 0));
        let context = EvaluationContext::empty(env.clone());
        
        let input = MonadicEvaluationInput {
            expression: crate::ast::Spanned {
                inner: Expr::Literal(Literal::Number(42.0)),
                span: Span::default(),
            },
            environment: env,
            expected_monad: None,
            context,
        };
        
        let scenario = create_basic_scenario("test_number", input);
        assert_eq!(scenario.name, "test_number");
        assert!(scenario.description.contains("Basic test scenario"));
        
        let performance_scenario = create_performance_scenario(
            "test_performance",
            scenario.input.clone()),
            1000,
            1024,
        );
        assert_eq!(performance_scenario.assertions.len(), 1);
    }
}
