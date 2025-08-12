//! Hotspot detection and execution profiling system
//! 
//! This module implements sophisticated hotspot detection for JIT compilation,
//! tracking execution frequency, timing, and complexity to make intelligent
//! compilation decisions.

use crate::ast::Expr;
use crate::eval::Environment;
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for hotspot detection
#[derive(Debug, Clone)]
pub struct HotspotConfig {
    /// Minimum execution frequency (calls per second) to consider for compilation
    pub min_frequency: f64,
    /// Minimum total execution time before compilation
    pub min_total_time: Duration,
    /// Minimum execution count before compilation
    pub min_execution_count: u64,
    /// Complexity threshold for immediate compilation
    pub complexity_threshold: f64,
    /// Stability window - how long to observe before compiling
    pub stability_window: Duration,
    /// Maximum functions to track simultaneously
    pub max_tracked_functions: usize,
}

impl Default for HotspotConfig {
    fn default() -> Self {
        HotspotConfig {
            min_frequency: 10.0,              // 10 calls per second
            min_total_time: Duration::from_millis(50),  // 50ms total
            min_execution_count: 100,         // 100 executions
            complexity_threshold: 5.0,        // Complex functions compile sooner
            stability_window: Duration::from_secs(2),   // 2 second observation
            max_tracked_functions: 1000,      // Track up to 1000 functions
        }
    }
}

/// Execution profile for a function
#[derive(Debug, Clone)]
pub struct ExecutionProfile {
    /// Function identifier
    pub identifier: String,
    /// AST representation
    pub ast: Expr,
    /// Total execution count
    pub execution_count: u64,
    /// Total execution time
    pub total_time: Duration,
    /// Average execution time
    pub average_time: Duration,
    /// Minimum execution time observed
    pub min_time: Duration,
    /// Maximum execution time observed
    pub max_time: Duration,
    /// Time of first execution
    pub first_execution: Instant,
    /// Time of last execution
    pub last_execution: Instant,
    /// Complexity score (higher = more complex)
    pub complexity_score: f64,
    /// Performance variance (lower = more stable)
    pub variance: f64,
    /// Whether this function has been compiled
    pub is_compiled: bool,
    /// Compilation attempts (for retry logic)
    pub compilation_attempts: u32,
}

impl ExecutionProfile {
    /// Creates a new execution profile
    pub fn new(identifier: String, ast: Expr) -> Self {
        let now = Instant::now();
        let complexity_score = Self::calculate_complexity(&ast);
        
        ExecutionProfile {
            identifier,
            ast,
            execution_count: 0,
            total_time: Duration::ZERO,
            average_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            first_execution: now,
            last_execution: now,
            complexity_score,
            variance: 0.0,
            is_compiled: false,
            compilation_attempts: 0,
        }
    }

    /// Updates the profile with a new execution
    pub fn record_execution(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.total_time += execution_time;
        self.last_execution = Instant::now();
        
        // Update min/max times
        if execution_time < self.min_time {
            self.min_time = execution_time;
        }
        if execution_time > self.max_time {
            self.max_time = execution_time;
        }
        
        // Calculate new average
        self.average_time = self.total_time / self.execution_count as u32;
        
        // Update variance (simplified calculation)
        self.update_variance(execution_time);
    }

    /// Calculates execution frequency (calls per second)
    pub fn execution_frequency(&self) -> f64 {
        let elapsed = self.last_execution.duration_since(self.first_execution);
        if elapsed.as_secs_f64() > 0.0 {
            self.execution_count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculates time stability (lower variance = more stable)
    pub fn time_stability(&self) -> f64 {
        if self.variance > 0.0 {
            1.0 / (1.0 + self.variance)
        } else {
            1.0
        }
    }

    /// Calculates compilation benefit score
    pub fn compilation_benefit_score(&self) -> f64 {
        let frequency_factor = (self.execution_frequency() / 10.0).min(2.0);
        let time_factor = (self.total_time.as_millis() as f64 / 100.0).min(3.0);
        let complexity_factor = (self.complexity_score / 5.0).min(2.0);
        let stability_factor = self.time_stability();
        
        frequency_factor * time_factor * complexity_factor * stability_factor
    }

    /// Updates performance variance
    fn update_variance(&mut self, new_time: Duration) {
        if self.execution_count <= 1 {
            self.variance = 0.0;
            return;
        }
        
        let avg_ms = self.average_time.as_millis() as f64;
        let new_ms = new_time.as_millis() as f64;
        let diff = new_ms - avg_ms;
        
        // Exponential moving variance
        let alpha = 0.1; // Smoothing factor
        self.variance = (1.0 - alpha) * self.variance + alpha * (diff * diff);
    }

    /// Calculates AST complexity score
    fn calculate_complexity(ast: &Expr) -> f64 {
        match ast {
            Expr::Literal(_) => 0.1,
            Expr::Identifier(_) => 0.2,
            Expr::Lambda { formals, body, .. } => {
                let param_count = match formals {
                    crate::ast::Formals::Fixed(params) => params.len(),
                    crate::ast::Formals::Variable(_) => 1,
                    crate::ast::Formals::Mixed { fixed, .. } => fixed.len() + 1,
                    crate::ast::Formals::Keyword { fixed, .. } => fixed.len(),
                };
                2.0 + param_count as f64 * 0.5 + body.iter().map(|e| Self::calculate_complexity(&e.inner)).sum::<f64>()
            }
            Expr::Application { operator, operands } => {
                1.0 + Self::calculate_complexity(&operator.inner) +
                operands.iter().map(|e| Self::calculate_complexity(&e.inner)).sum::<f64>()
            }
            Expr::If { test, consequent, alternative } => {
                1.5 + Self::calculate_complexity(&test.inner) +
                Self::calculate_complexity(&consequent.inner) +
                alternative.as_ref().map_or(0.0, |e| Self::calculate_complexity(&e.inner))
            }
            Expr::Let { bindings, body } => {
                1.0 + bindings.len() as f64 * 0.3 +
                bindings.iter().map(|binding| Self::calculate_complexity(&binding.value.inner)).sum::<f64>() +
                body.iter().map(|e| Self::calculate_complexity(&e.inner)).sum::<f64>()
            }
            Expr::LetRec { bindings, body } => {
                1.5 + bindings.len() as f64 * 0.4 +
                bindings.iter().map(|binding| Self::calculate_complexity(&binding.value.inner)).sum::<f64>() +
                body.iter().map(|e| Self::calculate_complexity(&e.inner)).sum::<f64>()
            }
            Expr::Begin(body) => {
                0.5 + body.iter().map(|e| Self::calculate_complexity(&e.inner)).sum::<f64>()
            }
            Expr::Quote { .. } => 0.1,
            Expr::Set { .. } => 0.5,
            // Handle other expression types with default complexity
            _ => 1.0,
        }
    }
}

/// Compilation candidate with scoring
#[derive(Debug, Clone)]
pub struct CompilationCandidate {
    /// Function identifier
    pub identifier: String,
    /// Compilation benefit score
    pub score: f64,
    /// Execution profile
    pub profile: ExecutionProfile,
    /// Recommended compilation tier
    pub recommended_tier: CompilationTier,
}

/// Compilation tier recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationTier {
    /// Stay with interpreter
    Interpreter,
    /// Compile to bytecode
    Bytecode,
    /// Basic JIT compilation
    JitBasic,
    /// Optimized JIT compilation
    JitOptimized,
}

/// Hotspot detection system
pub struct HotspotDetector {
    /// Configuration
    config: HotspotConfig,
    /// Execution profiles by function identifier
    profiles: HashMap<String, ExecutionProfile>,
    /// Functions currently being compiled
    compiling: HashMap<String, Instant>,
    /// Compilation history
    compilation_history: Vec<(String, Instant, bool)>, // (identifier, time, success)
}

impl HotspotDetector {
    /// Creates a new hotspot detector
    pub fn new(config: HotspotConfig) -> Self {
        HotspotDetector {
            config,
            profiles: HashMap::new(),
            compiling: HashMap::new(),
            compilation_history: Vec::new(),
        }
    }

    /// Records execution of a function
    pub fn record_execution(
        &mut self,
        identifier: String,
        ast: Expr,
        execution_time: Duration,
        _environment: Arc<Environment>,
    ) -> Result<()> {
        // Get or create profile
        let profile = self.profiles
            .entry(identifier.clone())
            .or_insert_with(|| ExecutionProfile::new(identifier.clone(), ast));

        profile.record_execution(execution_time);

        // Limit number of tracked functions
        if self.profiles.len() > self.config.max_tracked_functions {
            self.cleanup_stale_profiles();
        }

        Ok(())
    }

    /// Determines if a function should be compiled
    pub fn should_compile(&self, identifier: &str) -> Result<bool> {
        let profile = match self.profiles.get(identifier) {
            Some(profile) => profile,
            None => return Ok(false),
        };

        // Don't compile if already compiled or currently compiling
        if profile.is_compiled || self.compiling.contains_key(identifier) {
            return Ok(false);
        }

        // Check basic thresholds
        let meets_frequency = profile.execution_frequency() >= self.config.min_frequency;
        let meets_total_time = profile.total_time >= self.config.min_total_time;
        let meets_count = profile.execution_count >= self.config.min_execution_count;
        let meets_complexity = profile.complexity_score >= self.config.complexity_threshold;
        
        // Check stability window
        let elapsed_since_first = profile.last_execution.duration_since(profile.first_execution);
        let stable = elapsed_since_first >= self.config.stability_window;

        // Complex functions can be compiled with relaxed requirements
        if meets_complexity && profile.execution_count >= 10 {
            return Ok(true);
        }

        // Otherwise, all standard criteria must be met
        Ok(meets_frequency && meets_total_time && meets_count && stable)
    }

    /// Gets compilation candidates sorted by benefit score
    pub fn get_compilation_candidates(&self) -> Vec<CompilationCandidate> {
        let mut candidates = Vec::new();

        for profile in self.profiles.values() {
            if !profile.is_compiled && !self.compiling.contains_key(&profile.identifier) {
                let score = profile.compilation_benefit_score();
                if score > 1.0 {  // Minimum benefit threshold
                    let tier = self.recommend_tier(profile);
                    candidates.push(CompilationCandidate {
                        identifier: profile.identifier.clone(),
                        score,
                        profile: profile.clone(),
                        recommended_tier: tier,
                    });
                }
            }
        }

        // Sort by benefit score (highest first)
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    /// Marks a function as being compiled
    pub fn mark_compiling(&mut self, identifier: &str) {
        self.compiling.insert(identifier.to_string(), Instant::now());
        
        if let Some(profile) = self.profiles.get_mut(identifier) {
            profile.compilation_attempts += 1;
        }
    }

    /// Marks a function compilation as completed
    pub fn mark_compiled(&mut self, identifier: &str, success: bool) {
        self.compiling.remove(identifier);
        self.compilation_history.push((identifier.to_string(), Instant::now(), success));
        
        if success {
            if let Some(profile) = self.profiles.get_mut(identifier) {
                profile.is_compiled = true;
            }
        }
    }

    /// Recommends compilation tier based on profile
    fn recommend_tier(&self, profile: &ExecutionProfile) -> CompilationTier {
        let score = profile.compilation_benefit_score();
        let frequency = profile.execution_frequency();
        let complexity = profile.complexity_score;

        // Very high benefit or very hot functions get optimized JIT
        if score > 10.0 || frequency > 100.0 {
            CompilationTier::JitOptimized
        }
        // Good benefit or hot functions get basic JIT
        else if score > 5.0 || frequency > 25.0 || complexity > 8.0 {
            CompilationTier::JitBasic
        }
        // Moderate benefit gets bytecode
        else if score > 2.0 || frequency > 5.0 {
            CompilationTier::Bytecode
        }
        // Everything else stays interpreted
        else {
            CompilationTier::Interpreter
        }
    }

    /// Cleans up stale profiles to maintain memory usage
    fn cleanup_stale_profiles(&mut self) {
        let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes
        
        // Remove profiles that haven't been executed recently and aren't compiled
        self.profiles.retain(|_, profile| {
            profile.last_execution > cutoff_time || profile.is_compiled
        });
    }

    /// Gets performance statistics
    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("tracked_functions".to_string(), self.profiles.len() as f64);
        stats.insert("compiled_functions".to_string(), 
                    self.profiles.values().filter(|p| p.is_compiled).count() as f64);
        stats.insert("compiling_functions".to_string(), self.compiling.len() as f64);
        
        if !self.profiles.is_empty() {
            let total_executions: u64 = self.profiles.values().map(|p| p.execution_count).sum();
            let avg_frequency: f64 = self.profiles.values()
                .map(|p| p.execution_frequency())
                .sum::<f64>() / self.profiles.len() as f64;
            let avg_complexity: f64 = self.profiles.values()
                .map(|p| p.complexity_score)
                .sum::<f64>() / self.profiles.len() as f64;
            
            stats.insert("total_executions".to_string(), total_executions as f64);
            stats.insert("average_frequency".to_string(), avg_frequency);
            stats.insert("average_complexity".to_string(), avg_complexity);
        }

        stats
    }

    /// Gets the top hotspots by benefit score
    pub fn get_top_hotspots(&self, limit: usize) -> Vec<&ExecutionProfile> {
        let mut profiles: Vec<&ExecutionProfile> = self.profiles.values()
            .filter(|p| !p.is_compiled)
            .collect();
        
        profiles.sort_by(|a, b| b.compilation_benefit_score()
                               .partial_cmp(&a.compilation_benefit_score())
                               .unwrap_or(std::cmp::Ordering::Equal));
        
        profiles.into_iter().take(limit).collect()
    }

    /// Resets compilation status for a function (for recompilation)
    pub fn reset_compilation_status(&mut self, identifier: &str) {
        if let Some(profile) = self.profiles.get_mut(identifier) {
            profile.is_compiled = false;
        }
        self.compiling.remove(identifier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_execution_profile_creation() {
        let ast = Expr::Literal(Literal::ExactInteger(42));
        let profile = ExecutionProfile::new("test".to_string(), ast);
        
        assert_eq!(profile.identifier, "test");
        assert_eq!(profile.execution_count, 0);
        assert!(profile.complexity_score > 0.0);
    }

    #[test]
    fn test_complexity_calculation() {
        // Simple literal should have low complexity
        let simple = Expr::Literal(Literal::ExactInteger(1));
        let simple_score = ExecutionProfile::calculate_complexity(&simple);
        assert!(simple_score < 1.0);

        // Lambda should have higher complexity
        let lambda = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Identifier("x".to_string())),
        };
        let lambda_score = ExecutionProfile::calculate_complexity(&lambda);
        assert!(lambda_score > simple_score);
    }

    #[test]
    fn test_hotspot_detection() {
        let config = HotspotConfig::default();
        let mut detector = HotspotDetector::new(config);
        
        let ast = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Identifier("x".to_string())),
        };
        let env = Arc::new(Environment::new(None, 0));
        
        // Record multiple executions
        for _ in 0..150 {
            detector.record_execution(
                "test_function".to_string(),
                ast.clone(),
                Duration::from_micros(100),
                env.clone(),
            ).unwrap();
        }
        
        // Should now be considered for compilation
        let should_compile = detector.should_compile("test_function").unwrap();
        assert!(should_compile);
    }

    #[test]
    fn test_compilation_candidates() {
        let config = HotspotConfig::default();
        let mut detector = HotspotDetector::new(config);
        
        let ast = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Identifier("x".to_string())),
        };
        let env = Arc::new(Environment::new(None, 0));
        
        // Record executions for multiple functions
        for i in 0..3 {
            for _ in 0..100 {
                detector.record_execution(
                    format!("function_{}", i),
                    ast.clone(),
                    Duration::from_micros(100 * (i + 1)),
                    env.clone(),
                ).unwrap();
            }
        }
        
        let candidates = detector.get_compilation_candidates();
        assert!(!candidates.is_empty());
        
        // Should be sorted by benefit score
        if candidates.len() > 1 {
            assert!(candidates[0].score >= candidates[1].score);
        }
    }

    #[test]
    fn test_tier_recommendation() {
        let config = HotspotConfig::default();
        let detector = HotspotDetector::new(config);
        
        // Create a high-benefit profile
        let mut profile = ExecutionProfile::new(
            "hot_function".to_string(),
            Expr::Lambda {
                params: vec!["x".to_string()],
                body: Box::new(Expr::Identifier("x".to_string())),
            },
        );
        
        // Simulate many fast executions
        for _ in 0..1000 {
            profile.record_execution(Duration::from_nanos(100));
        }
        
        let tier = detector.recommend_tier(&profile);
        
        // Should recommend optimized JIT for very hot functions
        assert!(matches!(tier, CompilationTier::JitOptimized | CompilationTier::JitBasic));
    }
}