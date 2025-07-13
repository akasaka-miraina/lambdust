//! Frequency Tracking Module
//!
//! このモジュールは多次元実行頻度追跡システムを実装します。
//! 基本的な実行カウント、時間ベース解析、コンテキスト追跡を含みます。

use crate::error::Result;
use crate::value::Value;
use super::core_types::{
    ExecutionRecord, MemoryAllocation, AllocationType, CallStackContext,
    BindingContext, ModuleContext, PeakDetector, PeriodicityAnalyzer,
    FrequencyTrendAnalyzer,
};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};

/// Multi-dimensional frequency tracking system
#[derive(Debug)]
pub struct FrequencyTracker {
    /// Basic execution counts per expression
    pub execution_counts: HashMap<String, ExecutionRecord>,
    
    /// Time-based frequency analysis
    pub temporal_analysis: TemporalFrequencyAnalysis,
    
    /// Context-sensitive frequency tracking
    pub context_tracker: ContextualFrequencyTracker,
    
    /// Frequency trend analysis
    pub trend_analyzer: FrequencyTrendAnalyzer,
}

/// Temporal frequency analysis for time-based patterns
#[derive(Debug)]
pub struct TemporalFrequencyAnalysis {
    /// Time windows for analysis (1s, 10s, 1m, 10m, 1h)
    pub time_windows: Vec<Duration>,
    
    /// Execution counts per time window
    pub windowed_counts: HashMap<Duration, HashMap<String, VecDeque<u64>>>,
    
    /// Peak detection system
    pub peak_detector: PeakDetector,
    
    /// Periodicity analysis
    pub periodicity_analyzer: PeriodicityAnalyzer,
}

/// Context-sensitive frequency tracking
#[derive(Debug)]
pub struct ContextualFrequencyTracker {
    /// Call stack contexts
    pub call_contexts: HashMap<String, CallStackContext>,
    
    /// Variable binding contexts
    pub binding_contexts: HashMap<String, BindingContext>,
    
    /// Module/namespace contexts
    pub module_contexts: HashMap<String, ModuleContext>,
}

impl FrequencyTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_counts: HashMap::new(),
            temporal_analysis: TemporalFrequencyAnalysis::new(),
            context_tracker: ContextualFrequencyTracker::new(),
            trend_analyzer: FrequencyTrendAnalyzer,
        }
    }
    
    pub fn record_execution(
        &mut self,
        expr_hash: &str,
        execution_time: Duration,
        memory_usage: usize,
        return_value: &Value,
    ) -> Result<()> {
        let now = SystemTime::now();
        
        // Classify types first
        let allocation_type = self.classify_allocation_type(memory_usage);
        let value_type = self.classify_value_type(return_value);
        
        let record = self.execution_counts.entry(expr_hash.to_string()).or_insert_with(|| {
            ExecutionRecord {
                total_executions: 0,
                execution_times: VecDeque::new(),
                memory_allocations: Vec::new(),
                return_value_types: HashMap::new(),
                error_count: 0,
                average_execution_time: Duration::ZERO,
                execution_time_stddev: Duration::ZERO,
                first_seen: now,
                last_seen: now,
                peak_frequency: 0.0,
            }
        });
        
        // Update basic counts
        record.total_executions += 1;
        record.last_seen = now;
        
        // Update execution times (keep last 100)
        record.execution_times.push_back(execution_time);
        if record.execution_times.len() > 100 {
            record.execution_times.pop_front();
        }
        
        // Update average execution time
        let total_time: Duration = record.execution_times.iter().sum();
        record.average_execution_time = total_time / record.execution_times.len() as u32;
        
        // Record memory allocation
        record.memory_allocations.push(MemoryAllocation {
            size: memory_usage,
            timestamp: Instant::now(),
            allocation_type,
            short_lived: false, // Will be updated later
        });
        
        // Update return value type tracking
        *record.return_value_types.entry(value_type).or_insert(0) += 1;
        
        Ok(())
    }
    
    fn classify_allocation_type(&self, size: usize) -> AllocationType {
        match size {
            0..=64 => AllocationType::StackFrame,
            65..=1024 => AllocationType::String,
            1025..=8192 => AllocationType::Collection,
            _ => AllocationType::HeapObject,
        }
    }
    
    fn classify_value_type(&self, value: &Value) -> String {
        match value {
            Value::Number(_) => "Number".to_string(),
            Value::Boolean(_) => "Boolean".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Symbol(_) => "Symbol".to_string(),
            Value::Pair(_) => "List".to_string(),
            Value::Vector(_) => "Vector".to_string(),
            Value::Procedure(_) => "Procedure".to_string(),
            Value::Nil => "Nil".to_string(),
            _ => "Other".to_string(),
        }
    }
    
    /// Get execution record for an expression
    pub fn get_execution_record(&self, expr_hash: &str) -> Option<&ExecutionRecord> {
        self.execution_counts.get(expr_hash)
    }
    
    /// Get all tracked expressions
    pub fn get_tracked_expressions(&self) -> Vec<String> {
        self.execution_counts.keys().cloned().collect()
    }
    
    /// Calculate overall execution statistics
    pub fn calculate_statistics(&self) -> FrequencyStatistics {
        let total_expressions = self.execution_counts.len();
        let total_executions: u64 = self.execution_counts.values()
            .map(|record| record.total_executions)
            .sum();
        
        let average_executions = if total_expressions > 0 {
            total_executions as f64 / total_expressions as f64
        } else {
            0.0
        };
        
        let total_memory: usize = self.execution_counts.values()
            .flat_map(|record| &record.memory_allocations)
            .map(|alloc| alloc.size)
            .sum();
        
        FrequencyStatistics {
            total_expressions,
            total_executions,
            average_executions_per_expression: average_executions,
            total_memory_allocated: total_memory,
        }
    }
}

/// Frequency tracking statistics
#[derive(Debug, Clone)]
pub struct FrequencyStatistics {
    /// Total number of tracked expressions
    pub total_expressions: usize,
    
    /// Total number of executions across all expressions
    pub total_executions: u64,
    
    /// Average executions per expression
    pub average_executions_per_expression: f64,
    
    /// Total memory allocated across all expressions
    pub total_memory_allocated: usize,
}

impl TemporalFrequencyAnalysis {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            time_windows: vec![
                Duration::from_secs(1), 
                Duration::from_secs(10),
                Duration::from_secs(60),
                Duration::from_secs(600),
                Duration::from_secs(3600),
            ], 
            windowed_counts: HashMap::new(), 
            peak_detector: PeakDetector, 
            periodicity_analyzer: PeriodicityAnalyzer,
        } 
    }
    
    /// Record execution for temporal analysis
    pub fn record_execution(&mut self, expr_hash: &str, _execution_time: Duration) {
        for &window in &self.time_windows {
            let window_map = self.windowed_counts.entry(window).or_insert_with(HashMap::new);
            let expr_counts = window_map.entry(expr_hash.to_string()).or_insert_with(VecDeque::new);
            
            // Add current execution
            expr_counts.push_back(1);
            
            // Keep only recent executions within the time window
            let cutoff_time = std::time::Instant::now() - window;
            while let Some(&first_time) = expr_counts.front() {
                if std::time::Instant::now().duration_since(cutoff_time) > Duration::from_nanos(first_time) {
                    expr_counts.pop_front();
                } else {
                    break;
                }
            }
        }
    }
    
    /// Get execution frequency within a time window
    pub fn get_frequency_in_window(&self, expr_hash: &str, window: Duration) -> u64 {
        self.windowed_counts
            .get(&window)
            .and_then(|window_map| window_map.get(expr_hash))
            .map(|counts| counts.iter().sum())
            .unwrap_or(0)
    }
}

impl ContextualFrequencyTracker {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            call_contexts: HashMap::new(), 
            binding_contexts: HashMap::new(), 
            module_contexts: HashMap::new(),
        } 
    }
    
    /// Record execution with call context
    pub fn record_call_context(&mut self, expr_hash: &str, call_stack: &[String], execution_time: Duration) {
        if !call_stack.is_empty() {
            let context_key = format!("{}@{}", expr_hash, call_stack.join("->"));
            let context = self.call_contexts.entry(context_key).or_insert_with(|| {
                CallStackContext {
                    depth: call_stack.len(),
                    caller_chain: call_stack.to_vec(),
                    context_frequency: 0,
                    context_avg_time: Duration::ZERO,
                }
            });
            
            context.context_frequency += 1;
            // Update moving average
            context.context_avg_time = Duration::from_nanos(
                ((context.context_avg_time.as_nanos() * (context.context_frequency - 1) as u128
                 + execution_time.as_nanos()) / context.context_frequency as u128) as u64
            );
        }
    }
    
    /// Record execution with binding context
    pub fn record_binding_context(&mut self, expr_hash: &str, bindings: &HashMap<String, String>) {
        if !bindings.is_empty() {
            let context_key = format!("{}@bindings", expr_hash);
            let context = self.binding_contexts.entry(context_key).or_insert_with(|| {
                BindingContext {
                    bindings: bindings.clone(),
                    created_at: Instant::now(),
                    frequency: 0,
                }
            });
            
            context.frequency += 1;
        }
    }
    
    /// Get call context for expression
    pub fn get_call_context(&self, expr_hash: &str) -> Option<&CallStackContext> {
        // Find the most recent call context for this expression
        self.call_contexts.iter()
            .filter(|(key, _)| key.starts_with(expr_hash))
            .max_by_key(|(_, context)| context.context_frequency)
            .map(|(_, context)| context)
    }
}

impl Default for FrequencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TemporalFrequencyAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ContextualFrequencyTracker {
    fn default() -> Self {
        Self::new()
    }
}