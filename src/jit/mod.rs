//! Just-In-Time (JIT) compilation system for Lambdust Scheme
//!
//! This module provides comprehensive JIT compilation capabilities targeting
//! 5-15x performance improvements over pure interpretation while maintaining
//! R7RS-large compliance and seamless integration with existing systems.
//!
//! Key components:
//! - Hotspot detection and profiling
//! - Multi-tier compilation strategy  
//! - Native code generation with Cranelift
//! - Scheme-specific optimizations
//! - Profile-guided optimization
//! - Intelligent code caching

/// Hotspot detection and execution profiling
pub mod hotspot_detector;
/// Multi-tier compilation strategy management
pub mod compilation_tiers;
/// Native code generation using Cranelift backend
pub mod code_generator;
/// Scheme-specific optimization pipeline
pub mod optimization_pipeline;
/// Intelligent code cache with LRU eviction
pub mod code_cache;
/// Profile-guided optimization system
pub mod profile_guided_optimizer;
/// JIT configuration and settings
pub mod config;
/// Performance monitoring and metrics
pub mod metrics;

pub use hotspot_detector::{HotspotDetector, ExecutionProfile, CompilationCandidate};
pub use compilation_tiers::{TierManager, CompilationTier, TierTransition};
pub use code_generator::{CodeGenerator, NativeCode, TargetFeatures};
pub use optimization_pipeline::{OptimizationPipeline, SchemeOptimization};
pub use code_cache::{CodeCache, CacheEntry};
pub use config::EvictionPolicy;
pub use profile_guided_optimizer::{ProfileGuidedOptimizer, RuntimeProfile};
pub use config::{JitConfig, CompilationStrategy};
pub use metrics::{JitMetrics, PerformanceCounters};

use crate::ast::{Expr, Program};
use crate::eval::{Environment, Value};
use crate::diagnostics::{Error, Result};
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Main JIT compiler integrating all components
pub struct JitCompiler {
    /// Hotspot detection and profiling system
    hotspot_detector: Arc<Mutex<HotspotDetector>>,
    /// Multi-tier compilation management
    tier_manager: Arc<RwLock<TierManager>>,
    /// Native code generation backend
    code_generator: Arc<Mutex<CodeGenerator>>,
    /// Scheme-specific optimization pipeline
    optimization_pipeline: Arc<OptimizationPipeline>,
    /// Intelligent code cache
    code_cache: Arc<RwLock<CodeCache>>,
    /// Profile-guided optimization
    pgo: Arc<Mutex<ProfileGuidedOptimizer>>,
    /// JIT configuration
    config: JitConfig,
    /// Performance metrics
    metrics: Arc<RwLock<JitMetrics>>,
    /// Active compilations to prevent duplicate work
    active_compilations: Arc<Mutex<HashMap<String, Instant>>>,
}

/// Execution context for JIT compilation
#[derive(Debug, Clone)]
pub struct JitContext {
    /// Function or expression identifier
    pub identifier: String,
    /// Abstract syntax tree
    pub ast: Expr,
    /// Execution environment
    pub environment: Arc<Environment>,
    /// Execution count
    pub execution_count: u64,
    /// Total execution time
    pub total_time: Duration,
    /// Average execution time
    pub average_time: Duration,
    /// Type information from profiling
    pub type_info: HashMap<String, TypeProfile>,
}

/// Type profiling information for optimization
#[derive(Debug, Clone)]
pub struct TypeProfile {
    /// Most frequently observed type
    pub primary_type: String,
    /// Confidence percentage (0.0-1.0)
    pub confidence: f64,
    /// Total observations
    pub observations: u64,
}

/// Result of JIT compilation
pub enum CompilationResult {
    /// Successfully compiled native code
    Success {
        /// The compiled native code
        native_code: Arc<NativeCode>,
        /// Time spent compiling
        compilation_time: Duration,
        /// List of optimization passes applied
        optimizations_applied: Vec<String>,
    },
    /// Compilation deferred to later
    Deferred {
        /// Reason for deferring compilation
        reason: String,
        /// Duration to wait before retrying
        retry_after: Duration,
    },
    /// Compilation failed
    Failed {
        /// Error message describing the failure
        error: String,
        /// Fallback compilation tier to use
        fallback_tier: CompilationTier,
    },
}

impl JitCompiler {
    /// Creates a new JIT compiler with default configuration
    pub fn new() -> Result<Self> {
        let config = JitConfig::default();
        Self::with_config(config)
    }

    /// Creates a new JIT compiler with custom configuration
    pub fn with_config(config: JitConfig) -> Result<Self> {
        let code_generator = Arc::new(Mutex::new(CodeGenerator::new(config.to_codegen_config())?));
        
        Ok(JitCompiler {
            hotspot_detector: Arc::new(Mutex::new(HotspotDetector::new(config.hotspot_config.clone()))),
            tier_manager: Arc::new(RwLock::new(TierManager::new(config.tier_config.clone())?)),
            code_generator,
            optimization_pipeline: Arc::new(OptimizationPipeline::new({
                if config.optimization_config.enable_advanced_optimizations {
                    crate::jit::optimization_pipeline::OptimizationLevel::Aggressive
                } else if config.optimization_config.enable_basic_optimizations {
                    crate::jit::optimization_pipeline::OptimizationLevel::Balanced
                } else {
                    crate::jit::optimization_pipeline::OptimizationLevel::None
                }
            })?),
            code_cache: Arc::new(RwLock::new(CodeCache::new(config.cache_config.to_code_cache_config())?)),
            pgo: Arc::new(Mutex::new(ProfileGuidedOptimizer::new(config.pgo_config.clone().into())?)),
            config,
            metrics: Arc::new(RwLock::new(JitMetrics::new())),
            active_compilations: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Records execution of a function or expression for profiling
    pub fn record_execution(&self, context: JitContext, execution_time: Duration) -> Result<()> {
        // Update hotspot detector
        {
            let mut detector = self.hotspot_detector.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire hotspot detector lock".to_string(), None))?;
            
            detector.record_execution(
                context.identifier.clone(),
                context.ast.clone(),
                execution_time,
                context.environment.clone(),
            )?;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write()
                .map_err(|_| Error::runtime_error("Failed to acquire metrics lock".to_string(), None))?;
            
            metrics.record_execution(execution_time);
        }

        // Check if compilation should be triggered
        self.maybe_trigger_compilation(&context)?;

        Ok(())
    }

    /// Attempts to retrieve compiled native code for execution
    pub fn get_compiled_code(&self, ast: &Expr) -> Result<Option<NativeCode>> {
        let cache = self.code_cache.read()
            .map_err(|_| Error::runtime_error("Failed to acquire cache lock".to_string(), None))?;
        cache.get(ast)
    }

    /// Checks if compilation should be triggered and initiates it
    fn maybe_trigger_compilation(&self, context: &JitContext) -> Result<()> {
        let should_compile = {
            let detector = self.hotspot_detector.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire detector lock".to_string(), None))?;
            
            detector.should_compile(&context.identifier)?
        };

        if should_compile {
            self.trigger_compilation(context.clone())?;
        }

        Ok(())
    }

    /// Triggers asynchronous compilation of a function
    fn trigger_compilation(&self, context: JitContext) -> Result<()> {
        // Check if already compiling
        {
            let mut active = self.active_compilations.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire compilation lock".to_string(), None))?;
            
            if active.contains_key(&context.identifier) {
                return Ok(()); // Already compiling
            }
            
            active.insert(context.identifier.clone(), Instant::now());
        }

        // Determine compilation tier
        let tier = {
            let mut tier_manager = self.tier_manager.write()
                .map_err(|_| Error::runtime_error("Failed to acquire tier manager".to_string(), None))?;
            
            // Create execution profile for tier selection
            let mut profile = ExecutionProfile::new(context.identifier.clone(), context.ast.clone());
            profile.execution_count = context.execution_count;
            profile.total_time = context.total_time;
            profile.average_time = context.average_time;
            
            tier_manager.select_tier(&context.ast, &profile)?
        };

        // Start compilation process
        self.compile_function(context, tier)?;

        Ok(())
    }

    /// Compiles a function to the specified tier
    fn compile_function(&self, context: JitContext, tier: CompilationTier) -> Result<CompilationResult> {
        let start_time = Instant::now();

        // Generate native code first
        let native_code = {
            let mut generator = self.code_generator.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire code generator".to_string(), None))?;
            
            generator.compile_expression(&context.ast, tier)?
        };

        // Apply optimizations to native code
        let optimized_code = {
            let mut pipeline = self.optimization_pipeline.clone();
            let mut profile = ExecutionProfile::new(context.identifier.clone(), context.ast.clone());
            profile.execution_count = context.execution_count;
            profile.total_time = context.total_time;
            profile.average_time = context.average_time;
            
            // We need a mutable pipeline but we have Arc<> - for now, skip optimization
            native_code
        };

        let compilation_time = start_time.elapsed();

        // For now, always succeed with the generated code
        let native_code = optimized_code;
        
        // Store in code cache
        {
            let cache = self.code_cache.read()
                .map_err(|_| Error::runtime_error("Failed to acquire cache lock".to_string(), None))?;
            cache.store(context.ast.clone(), native_code.clone())?;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write()
                .map_err(|_| Error::runtime_error("Failed to acquire metrics".to_string(), None))?;
            
            metrics.record_compilation(compilation_time, tier);
        }

        // Remove from active compilations
        {
            let mut active = self.active_compilations.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire compilation lock".to_string(), None))?;
            
            active.remove(&context.identifier);
        }

        Ok(CompilationResult::Success {
            native_code: Arc::new(native_code),
            compilation_time,
            optimizations_applied: vec!["tier-specific".to_string()], // TODO: track actual optimizations
        })
    }

    /// Executes a function with JIT compilation if available
    pub fn execute_with_jit(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
    ) -> Result<Value> {
        let execution_start = Instant::now();

        // Try to get compiled code first
        if let Some(native_fn) = self.get_compiled_code(ast)? {
            // Record execution in cache
            {
                let cache = self.code_cache.read()
                    .map_err(|_| Error::runtime_error("Failed to acquire cache lock".to_string(), None))?;
                cache.record_execution(ast, execution_start.elapsed())?;
            }

            // Execute native code - TODO: implement proper execution interface
            // For now, return a placeholder value
            let result = Value::Nil;
            
            let execution_time = execution_start.elapsed();
            
            // Record execution for profiling
            self.record_execution(JitContext {
                identifier: identifier.to_string(),
                ast: ast.clone(),
                environment: env.clone(),
                execution_count: 1, // Will be aggregated by detector
                total_time: execution_time,
                average_time: execution_time,
                type_info: HashMap::new(), // TODO: collect actual type info
            }, execution_time)?;

            return Ok(result);
        }

        // Fallback to interpreter execution with profiling
        let result = self.execute_with_interpreter(ast, env)?;
        
        let execution_time = execution_start.elapsed();
        
        // Record execution for future compilation consideration
        self.record_execution(JitContext {
            identifier: identifier.to_string(),
            ast: ast.clone(),
            environment: env.clone(),
            execution_count: 1,
            total_time: execution_time,
            average_time: execution_time,
            type_info: HashMap::new(),
        }, execution_time)?;

        Ok(result)
    }

    /// Fallback interpreter execution (placeholder - integrate with existing evaluator)
    fn execute_with_interpreter(&self, ast: &Expr, env: &Arc<Environment>) -> Result<Value> {
        // TODO: Integrate with existing evaluator
        // For now, return a placeholder
        match ast {
            Expr::Literal(lit) => Ok(Value::from_literal(lit.clone())),
            _ => Ok(Value::Nil),
        }
    }

    /// Gets current JIT performance metrics
    pub fn get_metrics(&self) -> Result<JitMetrics> {
        let metrics = self.metrics.read()
            .map_err(|_| Error::runtime_error("Failed to acquire metrics".to_string(), None))?;
        
        Ok(metrics.clone())
    }

    /// Triggers code cache cleanup and optimization
    pub fn optimize_cache(&self) -> Result<()> {
        // The cache handles its own cleanup and optimization internally
        // We can clear it if needed
        let cache = self.code_cache.read()
            .map_err(|_| Error::runtime_error("Failed to acquire cache lock".to_string(), None))?;
        cache.clear()?;
        Ok(())
    }

    /// Enables or disables JIT compilation
    pub fn set_enabled(&mut self, enabled: bool) {
        // Update configuration
        // TODO: Implement configuration updates
    }

    /// Gets compilation statistics
    pub fn get_compilation_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        
        let cache_size = {
            let cache = self.code_cache.read()
                .map_err(|_| Error::runtime_error("Failed to acquire cache lock".to_string(), None))?;
            cache.size()?
        };
        stats.insert("cached_functions".to_string(), cache_size as u64);
        
        let metrics = self.metrics.read()
            .map_err(|_| Error::runtime_error("Failed to acquire metrics".to_string(), None))?;
        
        stats.insert("total_executions".to_string(), metrics.total_executions());
        stats.insert("compilation_time_ms".to_string(), metrics.total_compilation_time().as_millis() as u64);
        
        Ok(stats)
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default JIT compiler")
    }
}

/// Convenience functions for JIT compilation
pub mod utils {
    use super::*;

    /// Creates a JIT context from minimal information
    pub fn create_context(
        identifier: String,
        ast: Expr,
        environment: Arc<Environment>,
    ) -> JitContext {
        JitContext {
            identifier,
            ast,
            environment,
            execution_count: 0,
            total_time: Duration::ZERO,
            average_time: Duration::ZERO,
            type_info: HashMap::new(),
        }
    }

    /// Extracts function name from AST for profiling
    pub fn extract_function_name(ast: &Expr) -> String {
        match ast {
            Expr::Application { operator, .. } => {
                match &operator.inner {
                    Expr::Identifier(name) => name.clone(),
                    _ => "anonymous_application".to_string(),
                }
            }
            Expr::Lambda { .. } => "lambda".to_string(),
            Expr::Identifier(name) => name.clone(),
            _ => "expression".to_string(),
        }
    }

    /// Determines if an expression is suitable for JIT compilation
    pub fn is_jit_suitable(ast: &Expr) -> bool {
        matches!(ast, Expr::Lambda { .. } | Expr::Application { .. } | Expr::Let { .. } | Expr::LetRec { .. } | Expr::If { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_jit_compiler_creation() {
        let jit = JitCompiler::new();
        assert!(jit.is_ok());
    }

    #[test]
    fn test_jit_context_creation() {
        let ast = Expr::Literal(Literal::ExactInteger(42));
        let env = Arc::new(Environment::new(None, 0));
        
        let context = utils::create_context("test".to_string(), ast, env);
        assert_eq!(context.identifier, "test");
    }

    #[test]
    fn test_function_name_extraction() {
        let ast = Expr::Identifier("test_function".to_string());
        let name = utils::extract_function_name(&ast);
        assert_eq!(name, "test_function");
    }

    #[test]
    fn test_jit_suitability() {
        let suitable = Expr::Lambda {
            params: vec![],
            body: Box::new(Expr::Literal(Literal::ExactInteger(1))),
        };
        assert!(utils::is_jit_suitable(&suitable));

        let unsuitable = Expr::Literal(Literal::ExactInteger(42));
        assert!(!utils::is_jit_suitable(&unsuitable));
    }
}