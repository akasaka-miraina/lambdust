//! Multi-tier compilation strategy for progressive optimization
//!
//! This module implements a sophisticated compilation pipeline that progressively
//! optimizes code through multiple tiers: interpreter → bytecode → native JIT.
//! Each tier provides increasingly aggressive optimizations while maintaining
//! fast compilation times for interactive development.

use crate::ast::Expr;
use crate::diagnostics::{Result, Error};
use crate::jit::code_generator::NativeCode;
use crate::jit::hotspot_detector::ExecutionProfile;
// Note: Bytecode integration will be added when bytecode module is available
use std::time::Duration;
use std::collections::HashMap;

/// Compilation tiers in ascending order of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilationTier {
    /// Direct AST interpretation - fastest to start, slowest execution
    Interpreter = 0,
    
    /// Stack-based bytecode - moderate startup cost, good execution speed
    Bytecode = 1,
    
    /// Basic JIT with simple optimizations - higher startup cost, fast execution
    JitBasic = 2,
    
    /// Optimized JIT with aggressive optimizations - highest startup cost, fastest execution
    JitOptimized = 3,
}

impl CompilationTier {
    /// Returns the name of this compilation tier
    pub fn name(&self) -> &'static str {
        match self {
            Self::Interpreter => "Interpreter",
            Self::Bytecode => "Bytecode",
            Self::JitBasic => "JIT Basic",
            Self::JitOptimized => "JIT Optimized",
        }
    }

    /// Returns the expected compilation time for this tier
    pub fn compilation_time(&self) -> Duration {
        match self {
            Self::Interpreter => Duration::ZERO, // No compilation needed
            Self::Bytecode => Duration::from_micros(100),
            Self::JitBasic => Duration::from_millis(5),
            Self::JitOptimized => Duration::from_millis(50),
        }
    }
    
    /// Returns the expected execution speedup compared to interpreter
    pub fn expected_speedup(&self) -> f64 {
        match self {
            Self::Interpreter => 1.0,
            Self::Bytecode => 3.0,
            Self::JitBasic => 8.0,
            Self::JitOptimized => 15.0,
        }
    }
    
    /// Returns the next higher tier, if any
    pub fn next_tier(&self) -> Option<Self> {
        match self {
            Self::Interpreter => Some(Self::Bytecode),
            Self::Bytecode => Some(Self::JitBasic),
            Self::JitBasic => Some(Self::JitOptimized),
            Self::JitOptimized => None,
        }
    }
}

/// Configuration for tier management
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// Enable automatic tier transitions based on execution profile
    pub auto_tier_up: bool,
    
    /// Minimum executions before considering tier-up
    pub tier_up_execution_threshold: u64,
    
    /// Minimum execution time before considering tier-up
    pub tier_up_time_threshold: Duration,
    
    /// Cost-benefit threshold for tier transitions
    pub tier_up_benefit_threshold: f64,
    
    /// Maximum compilation time allowed per tier
    pub max_compilation_time: HashMap<CompilationTier, Duration>,
    
    /// Enable speculative compilation (compile to higher tiers preemptively)
    pub enable_speculative_compilation: bool,
    
    /// Enable deoptimization (fall back to lower tiers if assumptions fail)
    pub enable_deoptimization: bool,
}

impl Default for TierConfig {
    fn default() -> Self {
        let mut max_compilation_time = HashMap::new();
        max_compilation_time.insert(CompilationTier::Bytecode, Duration::from_millis(1));
        max_compilation_time.insert(CompilationTier::JitBasic, Duration::from_millis(10));
        max_compilation_time.insert(CompilationTier::JitOptimized, Duration::from_millis(100));
        
        Self {
            auto_tier_up: true,
            tier_up_execution_threshold: 50,
            tier_up_time_threshold: Duration::from_millis(10),
            tier_up_benefit_threshold: 2.0,
            max_compilation_time,
            enable_speculative_compilation: true,
            enable_deoptimization: true,
        }
    }
}

/// Tier transition reason
#[derive(Debug, Clone)]
pub enum TierTransition {
    /// Automatic tier-up based on execution profile
    AutoTierUp {
        /// Source compilation tier
        from: CompilationTier,
        /// Target compilation tier
        to: CompilationTier,
        /// Reason for the transition
        reason: String,
    },
    
    /// Forced tier-up (e.g., user request)
    ForcedTierUp {
        /// Source compilation tier
        from: CompilationTier,
        /// Target compilation tier
        to: CompilationTier,
    },
    
    /// Deoptimization (tier-down due to failed assumptions)
    Deoptimization {
        /// Source compilation tier
        from: CompilationTier,
        /// Target compilation tier
        to: CompilationTier,
        /// Reason for deoptimization
        reason: String,
    },
    
    /// Speculative compilation
    SpeculativeCompilation {
        /// Target compilation tier
        to: CompilationTier,
        /// Reason for speculative compilation
        reason: String,
    },
}

/// Compiled code entry for different tiers
#[derive(Debug, Clone)]
pub struct TieredCode {
    /// Current compilation tier
    pub tier: CompilationTier,
    
    /// Native code representation (if compiled to JIT tiers)
    pub native_code: Option<NativeCode>,
    
    /// Compilation time for this tier
    pub compilation_time: Duration,
    
    /// Number of times this code has been executed
    pub execution_count: u64,
    
    /// Total execution time
    pub total_execution_time: Duration,
    
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Performance metrics since last tier change
    pub performance_since_tier_change: PerformanceMetrics,
}

impl TieredCode {
    /// Creates new tiered code entry
    pub fn new(tier: CompilationTier) -> Self {
        Self {
            tier,
            native_code: None,
            compilation_time: Duration::ZERO,
            execution_count: 0,
            total_execution_time: Duration::ZERO,
            avg_execution_time: Duration::ZERO,
            performance_since_tier_change: PerformanceMetrics::new(),
        }
    }
    
    /// Records execution of this code
    pub fn record_execution(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.total_execution_time += execution_time;
        self.avg_execution_time = self.total_execution_time / self.execution_count as u32;
        self.performance_since_tier_change.record_execution(execution_time);
    }
    
    /// Returns the current performance trend
    pub fn performance_trend(&self) -> PerformanceTrend {
        self.performance_since_tier_change.trend()
    }
}

/// Performance metrics for tracking execution performance
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Recent execution times (sliding window)
    execution_times: Vec<Duration>,
    
    /// Window size for trend analysis
    window_size: usize,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            execution_times: Vec::new(),
            window_size: 20,
        }
    }
    
    fn record_execution(&mut self, execution_time: Duration) {
        self.execution_times.push(execution_time);
        if self.execution_times.len() > self.window_size {
            self.execution_times.remove(0);
        }
    }
    
    fn trend(&self) -> PerformanceTrend {
        if self.execution_times.len() < 10 {
            return PerformanceTrend::Insufficient;
        }
        
        let mid = self.execution_times.len() / 2;
        let first_half: f64 = self.execution_times[..mid]
            .iter().map(|d| d.as_nanos() as f64).sum::<f64>() / mid as f64;
        let second_half: f64 = self.execution_times[mid..]
            .iter().map(|d| d.as_nanos() as f64).sum::<f64>() / (self.execution_times.len() - mid) as f64;
            
        let ratio = second_half / first_half;
        
        if ratio < 0.9 {
            PerformanceTrend::Improving
        } else if ratio > 1.1 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    /// Performance is improving over time
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is degrading over time
    Degrading,
    /// Insufficient data to determine trend
    Insufficient,
}

/// Tier manager for coordinating compilation across tiers
pub struct TierManager {
    /// Configuration
    config: TierConfig,
    
    /// Compiled code cache by expression
    code_cache: HashMap<String, TieredCode>,
    
    /// Tier transition history
    transition_history: Vec<TierTransition>,
    
    /// Statistics
    stats: TierStats,
}

impl TierManager {
    /// Creates a new tier manager
    pub fn new(config: TierConfig) -> Result<Self> {
        Ok(Self {
            config,
            code_cache: HashMap::new(),
            transition_history: Vec::new(),
            stats: TierStats::default(),
        })
    }
    
    /// Selects the appropriate compilation tier for an expression
    pub fn select_tier(&mut self, expr: &Expr, profile: &ExecutionProfile) -> Result<CompilationTier> {
        let expr_key = self.expression_key(expr);
        
        // Check if we have existing compiled code
        if let Some(existing_code) = self.code_cache.get(&expr_key) {
            // Consider tier-up if enabled
            if self.config.auto_tier_up && self.should_tier_up(existing_code, profile)? {
                if let Some(next_tier) = existing_code.tier.next_tier() {
                    self.record_transition(TierTransition::AutoTierUp {
                        from: existing_code.tier,
                        to: next_tier,
                        reason: "Execution profile indicates benefit from higher tier".to_string(),
                    });
                    return Ok(next_tier);
                }
            }
            
            return Ok(existing_code.tier);
        }
        
        // For new expressions, start with appropriate tier based on profile
        let initial_tier = if profile.execution_count == 0 {
            CompilationTier::Interpreter // Start with interpreter for new code
        } else if profile.execution_count < 10 {
            CompilationTier::Bytecode // Warm code gets bytecode
        } else if profile.average_time.as_micros() > 1000 {
            CompilationTier::JitBasic // Hot code gets basic JIT
        } else {
            CompilationTier::Bytecode
        };
        
        Ok(initial_tier)
    }
    
    /// Records compiled code for an expression at a specific tier
    pub fn record_compiled_code(&mut self, expr: &Expr, tier: CompilationTier, 
                               native_code: Option<NativeCode>,
                               compilation_time: Duration) -> Result<()> {
        let expr_key = self.expression_key(expr);
        
        let mut tiered_code = TieredCode::new(tier);
        tiered_code.native_code = native_code;
        tiered_code.compilation_time = compilation_time;
        
        self.code_cache.insert(expr_key, tiered_code);
        self.stats.compilations_by_tier.entry(tier)
            .and_modify(|count| *count += 1)
            .or_insert(1);
            
        Ok(())
    }
    
    /// Records execution of compiled code
    pub fn record_execution(&mut self, expr: &Expr, execution_time: Duration) -> Result<()> {
        let expr_key = self.expression_key(expr);
        
        if let Some(tiered_code) = self.code_cache.get_mut(&expr_key) {
            tiered_code.record_execution(execution_time);
        }
        
        Ok(())
    }
    
    /// Checks if code should be tier-up to next level
    fn should_tier_up(&self, code: &TieredCode, profile: &ExecutionProfile) -> Result<bool> {
        // Don't tier-up if already at maximum tier
        if code.tier.next_tier().is_none() {
            return Ok(false);
        }
        
        // Check execution threshold
        let execution_threshold_met = code.execution_count >= self.config.tier_up_execution_threshold;
        
        // Check time threshold
        let time_threshold_met = code.total_execution_time >= self.config.tier_up_time_threshold;
        
        // Check benefit threshold
        let next_tier = code.tier.next_tier().unwrap();
        let expected_benefit = next_tier.expected_speedup() / code.tier.expected_speedup();
        let benefit_threshold_met = expected_benefit >= self.config.tier_up_benefit_threshold;
        
        // Check performance trend (don't tier-up if performance is degrading)
        let performance_ok = matches!(code.performance_trend(), 
                                    PerformanceTrend::Stable | PerformanceTrend::Improving);
        
        Ok(execution_threshold_met && time_threshold_met && benefit_threshold_met && performance_ok)
    }
    
    /// Initiates speculative compilation to higher tiers
    pub fn speculative_compile(&mut self, expr: &Expr, profile: &ExecutionProfile) -> Result<Option<CompilationTier>> {
        if !self.config.enable_speculative_compilation {
            return Ok(None);
        }
        
        let expr_key = self.expression_key(expr);
        
        if let Some(existing_code) = self.code_cache.get(&expr_key) {
            // Consider speculative compilation to next tier
            if let Some(next_tier) = existing_code.tier.next_tier() {
                // Only speculate if the code shows promise
                let looks_promising = profile.compilation_benefit_score() > 5.0 
                                   && existing_code.execution_count > 20
                                   && matches!(existing_code.performance_trend(), PerformanceTrend::Stable);
                
                if looks_promising {
                    self.record_transition(TierTransition::SpeculativeCompilation {
                        to: next_tier,
                        reason: "High benefit score and stable performance".to_string(),
                    });
                    return Ok(Some(next_tier));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Handles deoptimization (falling back to lower tier)
    pub fn deoptimize(&mut self, expr: &Expr, reason: String) -> Result<CompilationTier> {
        if !self.config.enable_deoptimization {
            return Err(Box::new(Error::runtime_error("Deoptimization disabled".to_string(), None)));
        }
        
        let expr_key = self.expression_key(expr);
        
        if let Some(existing_code) = self.code_cache.get_mut(&expr_key) {
            let original_tier = existing_code.tier;
            
            // Fall back to previous tier
            let new_tier = match existing_code.tier {
                CompilationTier::JitOptimized => CompilationTier::JitBasic,
                CompilationTier::JitBasic => CompilationTier::Bytecode,
                CompilationTier::Bytecode => CompilationTier::Interpreter,
                CompilationTier::Interpreter => CompilationTier::Interpreter, // Can't go lower
            };
            
            existing_code.tier = new_tier;
            existing_code.performance_since_tier_change = PerformanceMetrics::new();
            
            self.record_transition(TierTransition::Deoptimization {
                from: original_tier,
                to: new_tier,
                reason,
            });
            
            return Ok(new_tier);
        }
        
        Ok(CompilationTier::Interpreter)
    }
    
    /// Records a tier transition
    fn record_transition(&mut self, transition: TierTransition) {
        self.transition_history.push(transition);
        if self.transition_history.len() > 1000 {
            self.transition_history.remove(0); // Keep bounded history
        }
    }
    
    /// Returns compilation statistics
    pub fn stats(&self) -> &TierStats {
        &self.stats
    }
    
    /// Generates expression key for caching
    fn expression_key(&self, expr: &Expr) -> String {
        format!("{expr:?}") // Simplified - would use proper AST hashing in practice
    }
}

/// Tier manager statistics
#[derive(Debug, Clone, Default)]
pub struct TierStats {
    /// Number of compilations by tier
    pub compilations_by_tier: HashMap<CompilationTier, u64>,
    
    /// Number of tier transitions
    pub tier_up_transitions: u64,
    
    /// Number of deoptimizations
    pub deoptimizations: u64,
    
    /// Number of speculative compilations
    pub speculative_compilations: u64,
    
    /// Average compilation time by tier
    pub avg_compilation_time: HashMap<CompilationTier, Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::jit::ExecutionProfile;
    
    #[test]
    fn test_compilation_tier_ordering() {
        assert!(CompilationTier::Interpreter < CompilationTier::Bytecode);
        assert!(CompilationTier::Bytecode < CompilationTier::JitBasic);
        assert!(CompilationTier::JitBasic < CompilationTier::JitOptimized);
    }
    
    #[test]
    fn test_tier_manager() {
        let mut manager = TierManager::new(TierConfig::default()).unwrap();
        let expr = Expr::Literal(Literal::ExactInteger(42));
        let profile = ExecutionProfile::new();
        
        let tier = manager.select_tier(&expr, &profile).unwrap();
        assert_eq!(tier, CompilationTier::Interpreter);
    }
    
    #[test]
    fn test_tiered_code() {
        let mut code = TieredCode::new(CompilationTier::Bytecode);
        assert_eq!(code.execution_count, 0);
        
        code.record_execution(Duration::from_micros(100));
        assert_eq!(code.execution_count, 1);
    }
}