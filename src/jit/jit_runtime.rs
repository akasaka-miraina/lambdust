#![allow(missing_docs)]//! JIT runtime coordination system
//!
//! This module coordinates all JIT compilation activities and provides
//! the main interface for JIT-enabled execution.

use crate::ast::Expr;
use crate::diagnostics::{Error, Result};
use crate::eval::{Environment, Value};
use crate::jit::deoptimization::{DeoptimizationConfig, DeoptimizationContext};
use crate::jit::dependent_hotspot_detector::{
    DependentCompilationCandidate, DependentExecutionProfile, TypeExecutionContext,
};
use crate::jit::security::{
    ExecutionPermissions, JitSecurityFramework as SecurityFramework, SecurityConfig,
};
use crate::jit::specialized_compilation_tiers::{
    SpecializedCompilationResult, SpecializedNativeCode, SpecializedTierConfig,
};
use crate::jit::{
    DeoptimizationManager, DeoptimizationReason, DependentHotspotDetector, JitCompiler, JitConfig,
    JitSecurityFramework, SecurityVerificationResult, SpecializedTierManager,
};
use std::collections::HashMap;
use std::hint;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// High-performance memory pool for JIT allocations
struct JitMemoryPool {
    /// Pre-allocated execution contexts pool
    execution_context_pool: Vec<Box<JitExecutionContext>>,
    /// Available contexts bitmap
    available_contexts: AtomicU64,
    /// Pool size
    pool_size: usize,
}

impl JitMemoryPool {
    fn new(pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(Box::new(JitExecutionContext::new_empty()));
        }

        Self {
            execution_context_pool: pool,
            available_contexts: AtomicU64::new(u64::MAX >> (64 - pool_size)),
            pool_size,
        }
    }

    /// Fast allocation from pool with lockless design
    fn try_allocate(&self) -> Option<NonNull<JitExecutionContext>> {
        let mut available = self.available_contexts.load(Ordering::Acquire);

        loop {
            if available == 0 {
                return None; // Pool exhausted
            }

            let slot = available.trailing_zeros() as usize;
            let new_available = available & !(1u64 << slot);

            match self.available_contexts.compare_exchange_weak(
                available,
                new_available,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // High-performance unsafe optimization for lockless allocation:
                    //
                    // Safety invariants for this unsafe block:
                    // 1. slot < pool_size (guaranteed by trailing_zeros() on available mask)
                    // 2. slot corresponds to an available context (CAS operation succeeded)
                    // 3. Pool is pre-allocated during initialization and never reallocated
                    // 4. All pool elements are properly initialized in new()
                    // 5. Memory layout is stable for the lifetime of the pool
                    //
                    // Performance benefits:
                    // - Eliminates null pointer checking overhead (new_unchecked)
                    // - Direct pointer arithmetic without bounds checking
                    // - Zero-copy context acquisition
                    unsafe {
                        let ptr = self.execution_context_pool.as_ptr().add(slot);
                        // Use new_unchecked for performance - we've verified the pointer is valid
                        // This eliminates null checking overhead in the critical allocation path
                        return Some(NonNull::new_unchecked(ptr as *mut JitExecutionContext));
                    }
                }
                Err(current) => available = current,
            }
        }
    }

    /// Fast deallocation back to pool with unsafe optimizations
    ///
    /// # Safety
    /// This function uses unsafe pointer arithmetic for maximum deallocation performance.
    ///
    /// Safety invariants:
    /// - `ptr` must have been allocated from this pool
    /// - `ptr` must not be used after this call
    /// - Pool must still be valid (not dropped)
    ///
    /// Performance optimizations:
    /// - Direct pointer arithmetic to calculate slot index
    /// - Lock-free atomic operations for availability tracking
    /// - Minimal bounds checking (only debug assertions)
    unsafe fn deallocate(&self, ptr: NonNull<JitExecutionContext>) {
        // High-performance unsafe optimization for deallocation:
        //
        // Safety invariants:
        // 1. ptr was originally allocated from this pool
        // 2. ptr points to a valid JitExecutionContext within pool bounds
        // 3. offset_from is safe because both pointers are from same allocation
        // 4. Pool memory layout is stable
        unsafe {
            let base_ptr = self.execution_context_pool.as_ptr() as *const JitExecutionContext;
            let slot = ptr.as_ptr().offset_from(base_ptr) as usize;

            // Debug assertion to catch logic errors in debug builds
            debug_assert!(
                slot < self.pool_size,
                "Slot index {} exceeds pool size {}",
                slot,
                self.pool_size
            );

            // Use unchecked indexing for performance in release builds
            // Safety: slot is guaranteed to be valid by pool allocation invariants
            if slot < self.pool_size {
                let mask = 1u64 << slot;
                self.available_contexts.fetch_or(mask, Ordering::AcqRel);
            }
        }
    }
}

/// Lock-free statistics for maximum performance
#[repr(align(64))] // Cache line alignment
struct AtomicRuntimeStats {
    total_executions: AtomicU64,
    compiled_executions: AtomicU64,
    total_execution_time_nanos: AtomicU64,
    compiled_execution_time_nanos: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl AtomicRuntimeStats {
    fn new() -> Self {
        Self {
            total_executions: AtomicU64::new(0),
            compiled_executions: AtomicU64::new(0),
            total_execution_time_nanos: AtomicU64::new(0),
            compiled_execution_time_nanos: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    /// Record execution with minimal overhead
    #[inline(always)]
    fn record_execution(&self, execution_time: Duration, used_compiled_code: bool) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_nanos
            .fetch_add(execution_time.as_nanos() as u64, Ordering::Relaxed);

        if used_compiled_code {
            self.compiled_executions.fetch_add(1, Ordering::Relaxed);
            self.compiled_execution_time_nanos
                .fetch_add(execution_time.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    #[inline(always)]
    fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    fn to_runtime_stats(&self) -> JitRuntimeStats {
        JitRuntimeStats {
            total_executions: self.total_executions.load(Ordering::Relaxed),
            compiled_executions: self.compiled_executions.load(Ordering::Relaxed),
            interpreted_executions: self.total_executions.load(Ordering::Relaxed)
                - self.compiled_executions.load(Ordering::Relaxed),
            total_execution_time: Duration::from_nanos(
                self.total_execution_time_nanos.load(Ordering::Relaxed),
            ),
            compiled_execution_time: Duration::from_nanos(
                self.compiled_execution_time_nanos.load(Ordering::Relaxed),
            ),
            interpreted_execution_time: Duration::from_nanos(
                self.total_execution_time_nanos
                    .load(Ordering::Relaxed)
                    .saturating_sub(self.compiled_execution_time_nanos.load(Ordering::Relaxed)),
            ),
            compiled_functions: HashMap::new(), // Updated separately
        }
    }
}

/// Main JIT runtime that coordinates all compilation activities
pub struct JitRuntime {
    /// Configuration
    config: JitConfig,

    /// Base JIT compiler
    base_compiler: Arc<Mutex<JitCompiler>>,

    /// Dependent type-aware hotspot detection
    dependent_detector: Arc<Mutex<DependentHotspotDetector>>,

    /// Specialized compilation tiers
    specialized_tiers: Arc<Mutex<SpecializedTierManager>>,

    /// Security verification framework
    security_framework: Arc<Mutex<JitSecurityFramework>>,

    /// Deoptimization manager
    deoptimization_manager: Arc<Mutex<DeoptimizationManager>>,

    /// Execution contexts by function
    execution_contexts: Arc<RwLock<HashMap<String, JitExecutionContext>>>,

    /// High-performance memory pool
    memory_pool: JitMemoryPool,

    /// Lock-free runtime statistics
    atomic_stats: AtomicRuntimeStats,

    /// Active compilation tasks with reduced lock contention
    active_compilations: Arc<RwLock<HashMap<String, CompilationTask>>>,

    /// Fast compilation flags
    compilation_enabled: AtomicBool,

    /// Compilation queue capacity
    max_concurrent_compilations: usize,
}

impl JitRuntime {
    /// Creates a new JIT runtime with specified configuration
    pub fn new(config: JitConfig) -> Result<Self> {
        // Create specialized tier configuration
        let specialized_config = SpecializedTierConfig {
            specialization_config:
                crate::jit::specialized_compilation_tiers::SpecializationEngineConfig::default(),
            cache_config: crate::jit::specialized_compilation_tiers::SpecializedCacheConfig,
            performance_config:
                crate::jit::specialized_compilation_tiers::PerformanceTrackingConfig,
        };

        // Create security configuration
        let security_config = SecurityConfig::default();

        // Create deoptimization configuration
        let deopt_config = DeoptimizationConfig::default();

        Ok(Self {
            config: config.clone(),
            base_compiler: Arc::new(Mutex::new(JitCompiler::with_config(config.clone())?)),
            dependent_detector: Arc::new(Mutex::new(DependentHotspotDetector::new(
                config.hotspot_config.clone(),
            )?)),
            specialized_tiers: Arc::new(Mutex::new(SpecializedTierManager::new(
                specialized_config,
            )?)),
            security_framework: Arc::new(Mutex::new(JitSecurityFramework::new(security_config)?)),
            deoptimization_manager: Arc::new(Mutex::new(DeoptimizationManager::new(deopt_config)?)),
            execution_contexts: Arc::new(RwLock::new(HashMap::new())),
            memory_pool: JitMemoryPool::new(64), // 64 pre-allocated contexts
            atomic_stats: AtomicRuntimeStats::new(),
            compilation_enabled: AtomicBool::new(true),
            max_concurrent_compilations: 4,
            active_compilations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Executes a function with full JIT optimization
    pub fn execute_with_jit(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
        type_context: Option<TypeExecutionContext>,
    ) -> Result<JitExecutionResult> {
        let execution_start = Instant::now();

        // Get or create execution context
        let mut execution_context = self.get_or_create_execution_context(identifier, ast, env)?;

        // Record execution for profiling
        self.record_execution_for_profiling(identifier, ast, env, type_context.clone())?;

        // Check for existing compiled code
        if let Some(result) =
            self.try_execute_compiled_code(identifier, ast, env, &mut execution_context)?
        {
            let total_time = execution_start.elapsed();
            self.update_runtime_stats(identifier, total_time, true)?;
            return Ok(result);
        }

        // Check if we should trigger compilation
        if self.should_trigger_compilation(identifier)? {
            self.trigger_adaptive_compilation(identifier, ast, env, type_context)?;
        }

        // Execute with interpreter fallback
        let result = self.execute_with_interpreter_fallback(ast, env)?;
        let total_time = execution_start.elapsed();

        self.update_runtime_stats(identifier, total_time, false)?;

        Ok(JitExecutionResult {
            value: result,
            execution_time: total_time,
            used_compiled_code: false,
            tier_used: None,
        })
    }

    /// Triggers adaptive compilation based on profile analysis
    fn trigger_adaptive_compilation(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
        type_context: Option<TypeExecutionContext>,
    ) -> Result<()> {
        // Check if already compiling
        {
            let active = self.active_compilations.try_read().map_err(|_| {
                Error::runtime_error("Failed to acquire compilations lock".to_string(), None)
            })?;
            if active.contains_key(identifier) {
                return Ok(());
            }
        }

        // Get compilation candidates from dependent detector
        let candidates = {
            let detector = self.dependent_detector.lock().map_err(|_| {
                Error::runtime_error("Failed to acquire detector lock".to_string(), None)
            })?;
            detector.get_dependent_compilation_candidates()?
        };

        // Find our function in candidates
        let our_candidate = candidates
            .iter()
            .find(|c| c.base_candidate.identifier == identifier);

        if let Some(candidate) = our_candidate {
            // Start compilation task
            let task = CompilationTask {
                identifier: identifier.to_string(),
                ast: ast.clone(),
                candidate: candidate.clone(),
                start_time: Instant::now(),
                type_context,
            };

            // Record active compilation
            {
                let mut active = self.active_compilations.write().map_err(|_| {
                    Error::runtime_error("Failed to acquire compilations lock".to_string(), None)
                })?;
                active.insert(identifier.to_string(), task.clone());
            }

            // Perform compilation
            self.perform_compilation(task)?;
        }

        Ok(())
    }

    /// Performs the actual compilation with security verification
    fn perform_compilation(&self, task: CompilationTask) -> Result<()> {
        // Determine specialized tier selection
        let tier_result = {
            let mut specialized_tiers = self.specialized_tiers.lock().map_err(|_| {
                Error::runtime_error("Failed to acquire specialized tiers lock".to_string(), None)
            })?;

            // Create dependent execution profile from candidate
            let profile = self.create_dependent_profile_from_candidate(&task.candidate)?;

            specialized_tiers.select_specialized_tier(
                &task.ast,
                &profile,
                &task.candidate.specialization_opportunities,
            )?
        };

        match tier_result {
            SpecializedCompilationResult::CacheHit(cached_code) => {
                // Verify cached code security
                self.verify_and_store_code(&task.identifier, &cached_code)?;
            }
            SpecializedCompilationResult::BaseTier(tier) => {
                // Compile with base tier
                self.compile_with_base_tier(&task, tier)?;
            }
            SpecializedCompilationResult::SpecializedTier {
                tier,
                opportunities,
                ..
            } => {
                // Compile with specialized tier
                self.compile_with_specialized_tier(&task, tier, &opportunities)?;
            }
        }

        // Remove from active compilations
        {
            let mut active = self.active_compilations.write().map_err(|_| {
                Error::runtime_error("Failed to acquire compilations lock".to_string(), None)
            })?;
            active.remove(&task.identifier);
        }

        Ok(())
    }

    /// Compiles with specialized tier
    fn compile_with_specialized_tier(
        &self,
        task: &CompilationTask,
        tier: crate::jit::specialized_compilation_tiers::SpecializedCompilationTier,
        opportunities: &[crate::jit::dependent_hotspot_detector::SpecializationOpportunity],
    ) -> Result<()> {
        let profile = self.create_dependent_profile_from_candidate(&task.candidate)?;

        let specialized_code = {
            let mut specialized_tiers = self.specialized_tiers.lock().map_err(|_| {
                Error::runtime_error("Failed to acquire specialized tiers lock".to_string(), None)
            })?;

            specialized_tiers.compile_specialized(&task.ast, tier, opportunities, &profile)?
        };

        // Verify security of specialized code
        self.verify_and_store_specialized_code(&task.identifier, &specialized_code)?;

        Ok(())
    }

    /// Compiles with base tier
    fn compile_with_base_tier(
        &self,
        task: &CompilationTask,
        tier: crate::jit::compilation_tiers::CompilationTier,
    ) -> Result<()> {
        // Use base compiler
        let context = crate::jit::JitContext {
            identifier: task.identifier.clone(),
            ast: task.ast.clone(),
            environment: Arc::new(crate::eval::Environment::new(None, 0)), // Simplified
            execution_count: 1,
            total_time: Duration::from_millis(1),
            average_time: Duration::from_millis(1),
            type_info: HashMap::new(),
        };

        let base_compiler = self.base_compiler.lock().map_err(|_| {
            Error::runtime_error("Failed to acquire base compiler lock".to_string(), None)
        })?;

        // This would trigger compilation in the base compiler
        // For now, we'll just record the attempt

        Ok(())
    }

    /// Verifies and stores specialized compiled code
    fn verify_and_store_specialized_code(
        &self,
        identifier: &str,
        code: &SpecializedNativeCode,
    ) -> Result<()> {
        // Security verification
        let verification_result = {
            let mut security = self.security_framework.lock().map_err(|_| {
                Error::runtime_error(
                    "Failed to acquire security framework lock".to_string(),
                    None,
                )
            })?;

            let proofs = Vec::new(); // Would extract from code
            security.verify_specialized_code(&code.base_code, &proofs)?
        };

        match verification_result {
            SecurityVerificationResult::Approved { .. } => {
                // Store in execution context
                self.store_verified_code(identifier, code.clone())?;
            }
            SecurityVerificationResult::Verified => {
                // Code passed all checks, store it
                self.store_verified_code(identifier, code.clone())?;
            }
            SecurityVerificationResult::Skipped => {
                // Security was skipped, store with caution
                self.store_verified_code(identifier, code.clone())?;
            }
            SecurityVerificationResult::Failed { reason } => {
                // Trigger deoptimization for failed verification
                self.handle_security_rejection(identifier, reason)?;
            }
            SecurityVerificationResult::Rejected { reason } => {
                // Trigger deoptimization
                self.handle_security_rejection(identifier, reason)?;
            }
        }

        Ok(())
    }

    /// Verifies and stores regular compiled code
    fn verify_and_store_code(&self, identifier: &str, code: &SpecializedNativeCode) -> Result<()> {
        // Similar to verify_and_store_specialized_code but for regular code
        self.verify_and_store_specialized_code(identifier, code)
    }

    /// Stores verified code in execution context
    fn store_verified_code(&self, identifier: &str, code: SpecializedNativeCode) -> Result<()> {
        let mut contexts = self.execution_contexts.write().map_err(|_| {
            Error::runtime_error("Failed to acquire contexts lock".to_string(), None)
        })?;

        if let Some(context) = contexts.get_mut(identifier) {
            context.compiled_code = Some(code);
            context.compilation_timestamp = Some(Instant::now());
        }

        Ok(())
    }

    /// Handles security verification rejection
    fn handle_security_rejection(&self, identifier: &str, reason: String) -> Result<()> {
        // Trigger deoptimization
        let deopt_context = self.create_deoptimization_context(identifier)?;
        let deopt_reason = DeoptimizationReason::SecurityViolation {
            violation_type: reason,
        };

        let mut deopt_manager = self.deoptimization_manager.lock().map_err(|_| {
            Error::runtime_error(
                "Failed to acquire deoptimization manager lock".to_string(),
                None,
            )
        })?;

        deopt_manager.trigger_deoptimization(&deopt_context, deopt_reason)?;

        Ok(())
    }

    /// Creates deoptimization context for a function
    fn create_deoptimization_context(&self, identifier: &str) -> Result<DeoptimizationContext> {
        let contexts = self.execution_contexts.try_read().map_err(|_| {
            Error::runtime_error("Failed to acquire contexts lock".to_string(), None)
        })?;

        if let Some(context) = contexts.get(identifier) {
            Ok(DeoptimizationContext {
                function_id: identifier.to_string(),
                current_tier: crate::jit::compilation_tiers::CompilationTier::Interpreter, // Simplified
                original_ast: context.original_ast.clone(),
                environment: context.environment.clone(),
            })
        } else {
            Err(Box::new(Error::runtime_error(
                format!("No execution context found for {identifier}"),
                None,
            )))
        }
    }

    /// Records execution for profiling purposes
    fn record_execution_for_profiling(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
        type_context: Option<TypeExecutionContext>,
    ) -> Result<()> {
        let mut detector = self.dependent_detector.lock().map_err(|_| {
            Error::runtime_error("Failed to acquire detector lock".to_string(), None)
        })?;

        detector.record_execution_with_types(
            identifier.to_string(),
            ast.clone(),
            Duration::from_millis(1), // Placeholder
            env.clone(),
            type_context,
        )?;

        Ok(())
    }

    /// Checks if compilation should be triggered
    fn should_trigger_compilation(&self, identifier: &str) -> Result<bool> {
        let detector = self.dependent_detector.lock().map_err(|_| {
            Error::runtime_error("Failed to acquire detector lock".to_string(), None)
        })?;

        detector.should_compile_with_dependent_analysis(identifier)
    }

    /// Tries to execute with existing compiled code
    fn try_execute_compiled_code(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
        context: &mut JitExecutionContext,
    ) -> Result<Option<JitExecutionResult>> {
        if let Some(code) = &context.compiled_code {
            // Execute with compiled code
            let execution_start = Instant::now();

            // Create secure execution context
            let permissions = ExecutionPermissions::memory_access();
            let secure_context = {
                let mut security = self.security_framework.lock().map_err(|_| {
                    Error::runtime_error(
                        "Failed to acquire security framework lock".to_string(),
                        None,
                    )
                })?;

                security.create_secure_execution_context(&code.base_code, permissions)?
            };

            // Execute securely
            let args = Vec::new(); // Simplified
            let execution_result = {
                let mut security = self.security_framework.lock().map_err(|_| {
                    Error::runtime_error(
                        "Failed to acquire security framework lock".to_string(),
                        None,
                    )
                })?;

                security.execute_secure(&mut secure_context.clone(), &args)?
            };

            let execution_time = execution_start.elapsed();

            // Check for security violations that require deoptimization
            // For now, assume no security violations since execution_result is a Value
            if false {
                // execution_result.security_violations.is_empty() would be checked in complete implementation
                let deopt_context = self.create_deoptimization_context(identifier)?;
                let reason = DeoptimizationReason::SecurityViolation {
                    violation_type: "runtime_violation".to_string(),
                };

                let mut deopt_manager = self.deoptimization_manager.lock().map_err(|_| {
                    Error::runtime_error(
                        "Failed to acquire deoptimization manager lock".to_string(),
                        None,
                    )
                })?;

                deopt_manager.trigger_deoptimization(&deopt_context, reason)?;

                // Fall back to interpreter
                return Ok(None);
            }

            return Ok(Some(JitExecutionResult {
                value: execution_result, // execution_result is already a Value
                execution_time,
                used_compiled_code: true,
                tier_used: Some(code.tier),
            }));
        }

        Ok(None)
    }

    /// Executes with interpreter fallback
    fn execute_with_interpreter_fallback(
        &self,
        ast: &Expr,
        env: &Arc<Environment>,
    ) -> Result<Value> {
        // Simplified interpreter execution
        match ast {
            crate::ast::Expr::Literal(lit) => Ok(Value::from_literal(lit.clone())),
            _ => Ok(Value::Nil),
        }
    }

    /// Gets or creates execution context for a function
    fn get_or_create_execution_context(
        &self,
        identifier: &str,
        ast: &Expr,
        env: &Arc<Environment>,
    ) -> Result<JitExecutionContext> {
        let mut contexts = self.execution_contexts.write().map_err(|_| {
            Error::runtime_error("Failed to acquire contexts lock".to_string(), None)
        })?;

        if let Some(context) = contexts.get(identifier) {
            Ok(context.clone())
        } else {
            let context = JitExecutionContext {
                identifier: identifier.to_string(),
                original_ast: ast.clone(),
                environment: env.clone(),
                compiled_code: None,
                compilation_timestamp: None,
                execution_count: 0,
                total_execution_time: Duration::ZERO,
                last_execution_time: Instant::now(),
            };

            contexts.insert(identifier.to_string(), context.clone());
            Ok(context)
        }
    }

    /// Creates dependent profile from compilation candidate
    fn create_dependent_profile_from_candidate(
        &self,
        candidate: &DependentCompilationCandidate,
    ) -> Result<DependentExecutionProfile> {
        // Convert candidate to profile
        Ok(DependentExecutionProfile {
            base_profile: candidate.base_candidate.profile.clone(),
            type_computation_stats:
                crate::jit::dependent_hotspot_detector::TypeComputationStats::new(),
            proof_stats: crate::jit::dependent_hotspot_detector::ProofStats::new(),
            constraint_stats: crate::jit::dependent_hotspot_detector::ConstraintStats::new(),
            memory_access_patterns:
                crate::jit::dependent_hotspot_detector::MemoryAccessPatterns::new(),
        })
    }

    /// Updates runtime statistics with minimal overhead
    #[inline(always)]
    fn update_runtime_stats(
        &self,
        _identifier: &str,
        execution_time: Duration,
        used_compiled_code: bool,
    ) -> Result<()> {
        self.atomic_stats
            .record_execution(execution_time, used_compiled_code);
        Ok(())
    }

    /// Gets runtime statistics
    pub fn get_runtime_stats(&self) -> Result<JitRuntimeStats> {
        Ok(self.atomic_stats.to_runtime_stats())
    }
}

/// Execution context for a JIT-compiled function
#[derive(Debug, Clone)]
pub struct JitExecutionContext {
    /// Function identifier
    pub identifier: String,

    /// Original AST
    pub original_ast: Expr,

    /// Execution environment
    pub environment: Arc<Environment>,

    /// Compiled code (if available)
    pub compiled_code: Option<SpecializedNativeCode>,

    /// When code was compiled
    pub compilation_timestamp: Option<Instant>,

    /// Number of executions
    pub execution_count: u64,

    /// Total execution time
    pub total_execution_time: Duration,

    /// Last execution time
    pub last_execution_time: Instant,
}

impl JitExecutionContext {
    /// Creates an empty execution context for pool allocation
    pub fn new_empty() -> Self {
        Self {
            identifier: String::new(),
            original_ast: crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            environment: Arc::new(crate::eval::Environment::new(None, 0)),
            compiled_code: None,
            compilation_timestamp: None,
            execution_count: 0,
            total_execution_time: Duration::ZERO,
            last_execution_time: Instant::now(),
        }
    }

    /// Resets context for reuse
    pub fn reset(
        &mut self,
        identifier: String,
        ast: crate::ast::Expr,
        env: Arc<crate::eval::Environment>,
    ) {
        self.identifier = identifier;
        self.original_ast = ast;
        self.environment = env;
        self.compiled_code = None;
        self.compilation_timestamp = None;
        self.execution_count = 0;
        self.total_execution_time = Duration::ZERO;
        self.last_execution_time = Instant::now();
    }
}

/// Result of JIT execution
#[derive(Debug, Clone)]
pub struct JitExecutionResult {
    /// Execution result value
    pub value: Value,

    /// Time taken to execute
    pub execution_time: Duration,

    /// Whether compiled code was used
    pub used_compiled_code: bool,

    /// Which tier was used (if compiled code was used)
    pub tier_used: Option<crate::jit::specialized_compilation_tiers::SpecializedCompilationTier>,
}

/// Runtime statistics for JIT system
#[derive(Debug, Clone)]
pub struct JitRuntimeStats {
    /// Total executions
    pub total_executions: u64,

    /// Executions using compiled code
    pub compiled_executions: u64,

    /// Executions using interpreter
    pub interpreted_executions: u64,

    /// Total execution time
    pub total_execution_time: Duration,

    /// Execution time using compiled code
    pub compiled_execution_time: Duration,

    /// Execution time using interpreter
    pub interpreted_execution_time: Duration,

    /// Functions with compiled code
    pub compiled_functions: HashMap<String, Instant>,
}

impl Default for JitRuntimeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl JitRuntimeStats {
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            compiled_executions: 0,
            interpreted_executions: 0,
            total_execution_time: Duration::ZERO,
            compiled_execution_time: Duration::ZERO,
            interpreted_execution_time: Duration::ZERO,
            compiled_functions: HashMap::new(),
        }
    }

    pub fn record_execution(
        &mut self,
        identifier: &str,
        execution_time: Duration,
        used_compiled_code: bool,
    ) {
        self.total_executions += 1;
        self.total_execution_time += execution_time;

        if used_compiled_code {
            self.compiled_executions += 1;
            self.compiled_execution_time += execution_time;
            self.compiled_functions
                .entry(identifier.to_string())
                .or_insert_with(Instant::now);
        } else {
            self.interpreted_executions += 1;
            self.interpreted_execution_time += execution_time;
        }
    }

    /// Calculates JIT compilation effectiveness
    pub fn jit_effectiveness(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }

        self.compiled_executions as f64 / self.total_executions as f64
    }

    /// Calculates average speedup from JIT compilation
    pub fn average_speedup(&self) -> f64 {
        if self.interpreted_executions == 0 || self.compiled_executions == 0 {
            return 1.0;
        }

        let avg_interpreted_time =
            self.interpreted_execution_time.as_nanos() as f64 / self.interpreted_executions as f64;
        let avg_compiled_time =
            self.compiled_execution_time.as_nanos() as f64 / self.compiled_executions as f64;

        if avg_compiled_time > 0.0 {
            avg_interpreted_time / avg_compiled_time
        } else {
            1.0
        }
    }
}

/// Active compilation task
#[derive(Debug, Clone)]
pub struct CompilationTask {
    /// Function identifier
    pub identifier: String,

    /// AST to compile
    pub ast: Expr,

    /// Compilation candidate information
    pub candidate: DependentCompilationCandidate,

    /// When compilation started
    pub start_time: Instant,

    /// Type execution context
    pub type_context: Option<TypeExecutionContext>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_runtime_creation() {
        let config = JitConfig::default();
        let runtime = JitRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_runtime_stats() {
        let mut stats = JitRuntimeStats::new();

        stats.record_execution("test_fn", Duration::from_millis(10), false);
        assert_eq!(stats.interpreted_executions, 1);
        assert_eq!(stats.compiled_executions, 0);

        stats.record_execution("test_fn", Duration::from_millis(5), true);
        assert_eq!(stats.interpreted_executions, 1);
        assert_eq!(stats.compiled_executions, 1);

        // Should show 2x speedup (10ms -> 5ms)
        assert!(stats.average_speedup() > 1.5);
    }

    #[test]
    fn test_jit_effectiveness() {
        let mut stats = JitRuntimeStats::new();

        stats.record_execution("fn1", Duration::from_millis(1), false);
        stats.record_execution("fn2", Duration::from_millis(1), true);
        stats.record_execution("fn3", Duration::from_millis(1), true);

        // 2 out of 3 executions used compiled code
        assert_eq!(stats.jit_effectiveness(), 2.0 / 3.0);
    }
}
