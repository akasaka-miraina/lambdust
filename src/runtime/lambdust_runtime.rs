use super::{
    ThreadPool, GlobalEnvironmentManager, EffectCoordinator, 
    IOCoordinator, ErrorPropagationCoordinator, EvaluatorHandle,
    EvaluatorMessage, ParallelResult, BootstrapIntegration, BootstrapIntegrationConfig, BootstrapMode
};
use crate::ast::{Expr, Program};
use crate::diagnostics::{Result, Span};
use crate::eval::Value;
use crate::module_system::{ModuleSystem, ImportSpec};
use crossbeam::channel;
use std::sync::Arc;
use std::collections::HashMap;

/// The main multithreaded runtime for the Lambdust language.
///
/// This runtime provides parallel evaluation capabilities while maintaining
/// proper Scheme semantics. It uses an actor-based evaluator architecture
/// with message passing for inter-evaluator communication.
#[derive(Debug)]
pub struct LambdustRuntime {
    /// Thread pool for parallel evaluation
    pub(crate) thread_pool: Arc<ThreadPool>,
    /// Global environment manager
    global_env: Arc<GlobalEnvironmentManager>,
    /// Effect coordination system
    effect_coordinator: Arc<EffectCoordinator>,
    /// IO coordination system
    io_coordinator: Arc<IOCoordinator>,
    /// Error propagation system
    error_propagation: Arc<ErrorPropagationCoordinator>,
    /// Module system for imports and exports
    module_system: Arc<std::sync::RwLock<ModuleSystem>>,
    /// Handle counter for tracking evaluators
    handle_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl LambdustRuntime {
    /// Creates a new multithreaded runtime.
    pub fn new() -> crate::diagnostics::Result<Self> {
        Self::with_threads(4)
    }
    
    /// Creates a new runtime with the specified number of threads.
    pub fn with_threads(thread_count: usize) -> crate::diagnostics::Result<Self> {
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let effect_coordinator = Arc::new(EffectCoordinator::new());
        let io_coordinator = Arc::new(IOCoordinator::new());
        let error_propagation = Arc::new(ErrorPropagationCoordinator::new());
        #[allow(clippy::arc_with_non_send_sync)]
        let module_system = Arc::new(std::sync::RwLock::new(ModuleSystem::new()?));
        let handle_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        // Create the thread pool with the required dependencies
        let thread_pool = Arc::new(ThreadPool::new(
            thread_count, 
            global_env.clone(), 
            effect_coordinator.clone()
        )?);
        
        Ok(Self {
            thread_pool,
            global_env,
            effect_coordinator,
            io_coordinator,
            error_propagation,
            module_system,
            handle_counter,
        })
    }
    
    /// Gets a reference to the thread pool.
    pub fn thread_pool(&self) -> &Arc<ThreadPool> {
        &self.thread_pool
    }
    
    /// Gets a reference to the global environment manager.
    pub fn global_env(&self) -> &Arc<GlobalEnvironmentManager> {
        &self.global_env
    }
    
    /// Gets a reference to the effect coordinator.
    pub fn effect_coordinator(&self) -> &Arc<EffectCoordinator> {
        &self.effect_coordinator
    }
    
    /// Gets a reference to the IO coordinator.
    pub fn io_coordinator(&self) -> &Arc<IOCoordinator> {
        &self.io_coordinator
    }
    
    /// Gets a reference to the error propagation coordinator.
    pub fn error_propagation(&self) -> &Arc<ErrorPropagationCoordinator> {
        &self.error_propagation
    }
    
    /// Gets a reference to the module system.
    pub fn module_system(&self) -> &Arc<std::sync::RwLock<ModuleSystem>> {
        &self.module_system
    }
    
    /// Gets the next handle ID.
    pub fn next_handle_id(&self) -> u64 {
        self.handle_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Creates a new multithreaded runtime with bootstrap configuration.
    pub fn with_bootstrap_config(num_threads: Option<usize>, bootstrap_config: BootstrapIntegrationConfig) -> Result<Self> {
        let num_threads = num_threads.unwrap_or_else(num_cpus::get);
        
        // Run bootstrap process
        let mut bootstrap = BootstrapIntegration::with_config(bootstrap_config)?;
        let global_env = bootstrap.bootstrap()?;
        
        let effect_coordinator = Arc::new(EffectCoordinator::new());
        let io_coordinator = Arc::new(IOCoordinator::new());
        let error_propagation = Arc::new(ErrorPropagationCoordinator::new());
        #[allow(clippy::arc_with_non_send_sync)]
        let module_system = Arc::new(std::sync::RwLock::new(
            ModuleSystem::new().map_err(|e| crate::diagnostics::Error::runtime_error(
                format!("Failed to create module system: {e}"),
                None,
            ))?
        ));
        let thread_pool = Arc::new(ThreadPool::new(num_threads, global_env.clone(), effect_coordinator.clone())?);
        
        Ok(Self {
            thread_pool,
            global_env,
            effect_coordinator,
            io_coordinator,
            error_propagation,
            module_system,
            handle_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Creates a new multithreaded runtime in fallback mode.
    pub fn with_fallback(num_threads: Option<usize>) -> Result<Self> {
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Fallback,
            verbose: false,
            ..Default::default()
        };
        Self::with_bootstrap_config(num_threads, config)
    }

    /// Evaluates a single expression using the thread pool.
    pub async fn eval_expr(&self, expr: Expr, span: Option<Span>) -> Result<Value> {
        let (sender, receiver) = channel::bounded(1);
        
        let message = EvaluatorMessage::Evaluate {
            expr,
            span,
            sender,
        };
        
        self.thread_pool.submit_work(message)?;
        
        receiver.recv().map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to receive evaluation result: {e}"),
                span,
            )
        })?
    }

    /// Evaluates multiple expressions in parallel.
    pub async fn eval_parallel(&self, expressions: Vec<(Expr, Option<Span>)>) -> ParallelResult {
        let start_time = std::time::Instant::now();
        let num_expressions = expressions.len();
        
        // Create channels for each expression
        let mut receivers = Vec::new();
        
        for (expr, span) in expressions {
            let (sender, receiver) = channel::bounded(1);
            receivers.push(receiver);
            
            let message = EvaluatorMessage::Evaluate {
                expr,
                span,
                sender,
            };
            
            // Submit to thread pool (fire and forget for parallel execution)
            if let Err(e) = self.thread_pool.submit_work(message) {
                // If submission fails, create an error result
                receivers.pop(); // Remove the receiver we just added
                receivers.push({
                    let (error_sender, error_receiver) = channel::bounded(1);
                    let _ = error_sender.send(Err(crate::diagnostics::Error::runtime_error(
                        format!("Failed to submit work: {e}"),
                        span,
                    ).into()));
                    error_receiver
                });
            }
        }
        
        // Collect results in order
        let mut results = Vec::new();
        for receiver in receivers {
            match receiver.recv() {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(crate::diagnostics::Error::runtime_error(
                    format!("Failed to receive result: {e}"),
                    None,
                ).into()))
            }
        }
        
        let elapsed = start_time.elapsed();
        let threads_used = std::cmp::min(num_expressions, self.thread_pool.size());
        
        ParallelResult {
            results,
            elapsed,
            threads_used,
        }
    }

    /// Evaluates a program using parallel evaluation where possible.
    pub async fn eval_program(&self, program: &Program) -> Result<Value> {
        if program.expressions.is_empty() {
            return Ok(Value::Unspecified);
        }
        
        // For now, evaluate expressions sequentially to maintain ordering
        // TODO: Analyze dependencies to enable more parallelism
        let mut result = Value::Unspecified;
        
        for expr in &program.expressions {
            result = self.eval_expr(expr.inner.clone(), Some(expr.span)).await?;
        }
        
        Ok(result)
    }

    /// Spawns a new evaluator and returns a handle to it.
    pub fn spawn_evaluator(&self) -> Result<EvaluatorHandle> {
        let id = self.handle_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.thread_pool.spawn_evaluator(id)
    }

    /// Shuts down the runtime, stopping all evaluator threads.
    pub async fn shutdown(self) -> Result<()> {
        Arc::try_unwrap(self.thread_pool)
            .map_err(|_| crate::diagnostics::Error::runtime_error(
                "Failed to shutdown runtime: thread pool still has references".to_string(),
                None,
            ))?
            .shutdown()
            .await
    }

    /// Gets the number of evaluator threads.
    pub fn thread_count(&self) -> usize {
        self.thread_pool.size()
    }

    /// Imports a module into the runtime.
    pub async fn import_module(&self, import_spec: ImportSpec) -> Result<HashMap<String, Value>> {
        let mut module_system = self.module_system.write().map_err(|_| {
            crate::diagnostics::Error::runtime_error(
                "Failed to acquire module system lock".to_string(),
                None,
            )
        })?;
        
        module_system.resolve_import(&import_spec)
    }

    /// Gets access to the module system.
    pub fn with_module_system<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&ModuleSystem) -> R,
    {
        let module_system = self.module_system.read().map_err(|_| {
            crate::diagnostics::Error::runtime_error(
                "Failed to acquire module system read lock".to_string(),
                None,
            )
        })?;
        
        Ok(f(&module_system))
    }

    /// Gets mutable access to the module system.
    pub fn with_module_system_mut<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut ModuleSystem) -> Result<R>,
    {
        let mut module_system = self.module_system.write().map_err(|_| {
            crate::diagnostics::Error::runtime_error(
                "Failed to acquire module system write lock".to_string(),
                None,
            )
        })?;
        
        f(&mut module_system)
    }
}

impl Default for LambdustRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default runtime")
    }
}