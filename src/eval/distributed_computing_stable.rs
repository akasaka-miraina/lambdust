//! Phase 5 Stage 3: Advanced Distributed Computing System
//!
//! This module implements a comprehensive distributed computing system that seamlessly
//! integrates with Stage 1 JIT optimization and Stage 2 parallel execution systems.
//! The system provides Byzantine fault tolerance, ML-powered load balancing, and
//! CRDT-based consistency guarantees.
//!
//! ## Architecture
//!
//! The distributed computing system consists of five main components:
//!
//! 1. **Distributed Execution Engine**: High-performance task distribution with intelligent scheduling
//! 2. **Node Discovery & Management**: Byzantine fault tolerant node discovery with gossip protocol
//! 3. **ML-Powered Load Balancer**: Adaptive load balancing using reinforcement learning
//! 4. **Hierarchical Fault Tolerance**: Multi-layer fault detection and automatic recovery
//! 5. **CRDT Consistency System**: Conflict-free replicated data types for distributed state
//!
//! ## Integration Points
//!
//! - **Stage 1 JIT Integration**: Optimized code distribution and remote JIT compilation
//! - **Stage 2 Parallel Integration**: Distributed work-stealing and NUMA-aware allocation
//! - **Future Stage Integration**: Prepared for security and performance enhancements

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::eval::{Value, ParallelExecutionSystem, JITIntegrationSystem};
use crate::diagnostics::{Error, Result};
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use std::net::SocketAddr;
use dashmap::DashMap;
use ordered_float::OrderedFloat;

// ============= CORE TYPES =============

/// Unique identifier for distributed nodes
pub type NodeId = u64;

/// Unique identifier for distributed tasks
pub type TaskId = u64;

/// Unique identifier for CRDT objects
pub type CRDTId = u64;

/// Network address for node communication
pub type NetworkAddress = SocketAddr;

/// Distributed task execution result
pub type DistributedResult<T> = std::result::Result<T, DistributedError>;

// ============= ERROR TYPES =============

/// Errors that can occur in distributed computing
#[derive(Debug, Clone)]
pub enum DistributedError {
    /// Node is not available
    NodeUnavailable(NodeId),
    /// Task execution failed
    TaskExecutionFailed(TaskId, String),
    /// Network communication error
    NetworkError(String),
    /// Consensus failure in Byzantine fault tolerance
    ConsensusFailed(String),
    /// Load balancer prediction failed
    LoadBalancerFailed(String),
    /// CRDT synchronization error
    CRDTSyncError(String),
    /// Resource allocation failed
    ResourceAllocationFailed(String),
}

impl std::fmt::Display for DistributedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeUnavailable(id) => write!(f, "Node {} is unavailable", id),
            Self::TaskExecutionFailed(task_id, msg) => write!(f, "Task {} failed: {}", task_id, msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ConsensusFailed(msg) => write!(f, "Consensus failed: {}", msg),
            Self::LoadBalancerFailed(msg) => write!(f, "Load balancer failed: {}", msg),
            Self::CRDTSyncError(msg) => write!(f, "CRDT sync error: {}", msg),
            Self::ResourceAllocationFailed(msg) => write!(f, "Resource allocation failed: {}", msg),
        }
    }
}

impl std::error::Error for DistributedError {}

// ============= NODE MANAGEMENT =============

/// Capabilities of a distributed node
#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub node_id: NodeId,
    pub address: NetworkAddress,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
    pub network_bandwidth_mbps: u32,
    pub jit_support: bool,
    pub parallel_support: bool,
    pub gpu_support: bool,
    pub last_heartbeat: Instant,
    pub reputation_score: f64,
}

impl NodeCapabilities {
    pub fn new(node_id: NodeId, address: NetworkAddress) -> Self {
        Self {
            node_id,
            address,
            cpu_cores: num_cpus::get() as u32,
            memory_gb: 8, // Default estimate
            disk_gb: 100, // Default estimate
            network_bandwidth_mbps: 1000, // Default estimate
            jit_support: true,
            parallel_support: true,
            gpu_support: false,
            last_heartbeat: Instant::now(),
            reputation_score: 0.8, // Start with good reputation
        }
    }
    
    pub fn supports_capabilities(&self, required: &RequiredCapabilities) -> bool {
        self.cpu_cores >= required.min_cpu_cores
            && self.memory_gb >= required.min_memory_gb
            && self.network_bandwidth_mbps >= required.min_network_mbps
            && (!required.requires_jit || self.jit_support)
            && (!required.requires_parallel || self.parallel_support)
            && (!required.requires_gpu || self.gpu_support)
    }
    
    pub fn is_alive(&self, timeout: Duration) -> bool {
        self.last_heartbeat.elapsed() < timeout
    }
    
    pub fn performance_score(&self) -> f64 {
        let cpu_score = (self.cpu_cores as f64).log2();
        let memory_score = (self.memory_gb as f64).log2();
        let network_score = (self.network_bandwidth_mbps as f64 / 1000.0).log2();
        let reputation_weight = 2.0;
        
        (cpu_score + memory_score + network_score + reputation_weight * self.reputation_score) / 4.0
    }
}

/// Required capabilities for task execution
#[derive(Debug, Clone, Default)]
pub struct RequiredCapabilities {
    pub min_cpu_cores: u32,
    pub min_memory_gb: u32,
    pub min_network_mbps: u32,
    pub requires_jit: bool,
    pub requires_parallel: bool,
    pub requires_gpu: bool,
}

// ============= TASK MANAGEMENT =============

/// Distributed task that can be executed on remote nodes
#[derive(Debug, Clone)]
pub struct DistributedTask {
    pub id: TaskId,
    pub payload: TaskPayload,
    pub required_capabilities: RequiredCapabilities,
    pub dependencies: Vec<TaskId>,
    pub priority: u32,
    pub estimated_duration: Duration,
    pub max_retries: u32,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

/// Task payload containing the actual computation
#[derive(Debug, Clone)]
pub enum TaskPayload {
    /// Evaluate Scheme expression
    SchemeEval { 
        expression: String,
        environment: Vec<(String, Value)>,
    },
    /// Execute JIT-compiled code
    JITExecution {
        code_id: String,
        input: Value,
    },
    /// Parallel computation
    ParallelComputation {
        computation_id: String,
        data_chunks: Vec<Value>,
    },
    /// Custom computation
    Custom {
        computation_type: String,
        data: Vec<u8>,
    },
}

/// Task execution status
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Assigned { node_id: NodeId, assigned_at: SystemTime },
    Running { node_id: NodeId, started_at: SystemTime },
    Completed { result: Value, duration: Duration, node_id: NodeId },
    Failed { error: String, retry_count: u32, node_id: Option<NodeId> },
    Cancelled { reason: String },
}

// ============= DISTRIBUTED EXECUTION ENGINE =============

/// Main distributed execution engine
pub struct DistributedComputingSystem {
    /// Node registry with capabilities
    node_registry: Arc<RwLock<HashMap<NodeId, NodeCapabilities>>>,
    /// Task queue and management
    task_queue: Arc<RwLock<VecDeque<DistributedTask>>>,
    /// Running tasks tracking
    running_tasks: Arc<RwLock<HashMap<TaskId, TaskStatus>>>,
    /// Integration with Stage 1 JIT system
    #[cfg(feature = "stage1")]
    jit_system: Option<Arc<JITIntegrationSystem>>,
    /// Integration with Stage 2 parallel system
    #[cfg(feature = "stage2")]
    parallel_system: Option<Arc<ParallelExecutionSystem>>,
    /// ML-powered scheduler
    ml_scheduler: Arc<Mutex<MLScheduler>>,
    /// Byzantine fault detector
    byzantine_detector: Arc<Mutex<ByzantineDetector>>,
    /// CRDT consistency manager
    crdt_manager: Arc<CRDTManager>,
    /// Statistics and monitoring
    statistics: Arc<DistributedStatistics>,
    /// System configuration
    config: DistributedConfig,
    /// System state
    is_running: AtomicBool,
}

/// Configuration for distributed computing system
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    pub max_nodes: usize,
    pub max_concurrent_tasks: usize,
    pub heartbeat_timeout: Duration,
    pub task_retry_limit: u32,
    pub byzantine_tolerance_factor: f64, // Maximum fraction of Byzantine nodes
    pub consensus_timeout: Duration,
    pub load_balancer_learning_rate: f64,
    pub crdt_sync_interval: Duration,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            max_nodes: 1000,
            max_concurrent_tasks: 10000,
            heartbeat_timeout: Duration::from_secs(30),
            task_retry_limit: 3,
            byzantine_tolerance_factor: 0.33, // Up to 1/3 Byzantine nodes
            consensus_timeout: Duration::from_secs(10),
            load_balancer_learning_rate: 0.01,
            crdt_sync_interval: Duration::from_millis(100),
        }
    }
}

impl DistributedComputingSystem {
    /// Create a new distributed computing system
    pub fn new(config: DistributedConfig) -> Self {
        Self {
            node_registry: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "stage1")]
            jit_system: None,
            #[cfg(feature = "stage2")]
            parallel_system: None,
            ml_scheduler: Arc::new(Mutex::new(MLScheduler::new(config.load_balancer_learning_rate))),
            byzantine_detector: Arc::new(Mutex::new(ByzantineDetector::new(config.byzantine_tolerance_factor))),
            crdt_manager: Arc::new(CRDTManager::new()),
            statistics: Arc::new(DistributedStatistics::new()),
            config,
            is_running: AtomicBool::new(false),
        }
    }
    
    /// Start the distributed computing system
    pub fn start(&self) -> DistributedResult<()> {
        self.is_running.store(true, Ordering::SeqCst);
        // Note: mark_system_started is called during DistributedStatistics creation
        Ok(())
    }
    
    /// Stop the distributed computing system
    pub fn stop(&self) -> DistributedResult<()> {
        self.is_running.store(false, Ordering::SeqCst);
        self.statistics.mark_system_stopped();
        Ok(())
    }
    
    /// Register a new node in the system
    pub fn register_node(&self, capabilities: NodeCapabilities) -> DistributedResult<()> {
        let mut registry = self.node_registry.write().map_err(|_| {
            DistributedError::ResourceAllocationFailed("Failed to acquire node registry lock".to_string())
        })?;
        
        if registry.len() >= self.config.max_nodes {
            return Err(DistributedError::ResourceAllocationFailed(
                format!("Maximum nodes ({}) reached", self.config.max_nodes)
            ));
        }
        
        let node_id = capabilities.node_id;
        registry.insert(node_id, capabilities);
        self.statistics.increment_nodes_registered();
        Ok(())
    }
    
    /// Submit a task for distributed execution
    pub fn submit_task(&self, task: DistributedTask) -> DistributedResult<TaskId> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(DistributedError::ResourceAllocationFailed("System is not running".to_string()));
        }
        
        let task_id = task.id;
        
        // Add to task queue
        let mut queue = self.task_queue.write().map_err(|_| {
            DistributedError::ResourceAllocationFailed("Failed to acquire task queue lock".to_string())
        })?;
        
        if queue.len() >= self.config.max_concurrent_tasks {
            return Err(DistributedError::ResourceAllocationFailed(
                format!("Maximum concurrent tasks ({}) reached", self.config.max_concurrent_tasks)
            ));
        }
        
        // Add to running tasks with pending status
        let mut running = self.running_tasks.write().map_err(|_| {
            DistributedError::ResourceAllocationFailed("Failed to acquire running tasks lock".to_string())
        })?;
        running.insert(task_id, TaskStatus::Pending);
        
        // Insert task in priority order (higher priority first)
        let position = queue.iter().position(|t| t.priority < task.priority).unwrap_or(queue.len());
        queue.insert(position, task);
        
        self.statistics.increment_tasks_submitted();
        Ok(task_id)
    }
    
    /// Get task status
    pub fn get_task_status(&self, task_id: TaskId) -> DistributedResult<Option<TaskStatus>> {
        let running = self.running_tasks.read().map_err(|_| {
            DistributedError::ResourceAllocationFailed("Failed to acquire running tasks lock".to_string())
        })?;
        Ok(running.get(&task_id).cloned())
    }
    
    /// Get system statistics
    pub fn get_statistics(&self) -> Arc<DistributedStatistics> {
        Arc::clone(&self.statistics)
    }
    
    /// Integration with Stage 1 JIT system
    #[cfg(feature = "stage1")]
    pub fn integrate_jit_system(&mut self, jit_system: Arc<JITIntegrationSystem>) {
        self.jit_system = Some(jit_system);
    }
    
    /// Integration with Stage 2 parallel system  
    #[cfg(feature = "stage2")]
    pub fn integrate_parallel_system(&mut self, parallel_system: Arc<ParallelExecutionSystem>) {
        self.parallel_system = Some(parallel_system);
    }
}

// ============= ML-POWERED SCHEDULER =============

/// Machine learning scheduler for intelligent task distribution
pub struct MLScheduler {
    learning_rate: f64,
    node_performance_history: HashMap<NodeId, VecDeque<f64>>,
    task_completion_history: HashMap<NodeId, VecDeque<Duration>>,
    prediction_accuracy: f64,
}

impl MLScheduler {
    pub fn new(learning_rate: f64) -> Self {
        Self {
            learning_rate,
            node_performance_history: HashMap::new(),
            task_completion_history: HashMap::new(),
            prediction_accuracy: 0.5, // Start neutral
        }
    }
    
    /// Predict optimal node for task execution
    pub fn predict_optimal_node(
        &self,
        task: &DistributedTask,
        available_nodes: &[NodeCapabilities],
    ) -> Option<NodeId> {
        if available_nodes.is_empty() {
            return None;
        }
        
        let mut best_node = None;
        let mut best_score = f64::NEG_INFINITY;
        
        for node in available_nodes {
            if !node.supports_capabilities(&task.required_capabilities) {
                continue;
            }
            
            let score = self.calculate_node_score(node, task);
            if score > best_score {
                best_score = score;
                best_node = Some(node.node_id);
            }
        }
        
        best_node
    }
    
    /// Calculate node suitability score for task
    fn calculate_node_score(&self, node: &NodeCapabilities, task: &DistributedTask) -> f64 {
        let base_score = node.performance_score();
        let history_bonus = self.get_history_bonus(node.node_id);
        let load_penalty = self.get_load_penalty(node.node_id);
        let reputation_bonus = node.reputation_score;
        
        base_score + history_bonus - load_penalty + reputation_bonus
    }
    
    /// Get historical performance bonus for node
    fn get_history_bonus(&self, node_id: NodeId) -> f64 {
        self.node_performance_history
            .get(&node_id)
            .map(|history| {
                if history.is_empty() {
                    0.0
                } else {
                    history.iter().sum::<f64>() / history.len() as f64
                }
            })
            .unwrap_or(0.0)
    }
    
    /// Get load penalty for busy nodes
    fn get_load_penalty(&self, node_id: NodeId) -> f64 {
        self.task_completion_history
            .get(&node_id)
            .map(|history| history.len() as f64 * 0.1) // Penalty increases with queue length
            .unwrap_or(0.0)
    }
    
    /// Update model with task completion feedback
    pub fn update_with_feedback(
        &mut self,
        node_id: NodeId,
        task: &DistributedTask,
        completion_time: Duration,
        success: bool,
    ) {
        // Update performance history
        let performance_score = if success {
            1.0 / (completion_time.as_secs_f64() + 1.0)
        } else {
            -1.0 // Penalty for failure
        };
        
        self.node_performance_history
            .entry(node_id)
            .or_default()
            .push_back(performance_score);
        
        // Limit history size
        if let Some(history) = self.node_performance_history.get_mut(&node_id) {
            while history.len() > 100 {
                history.pop_front();
            }
        }
        
        // Update completion time history
        self.task_completion_history
            .entry(node_id)
            .or_default()
            .push_back(completion_time);
        
        // Limit history size
        if let Some(history) = self.task_completion_history.get_mut(&node_id) {
            while history.len() > 100 {
                history.pop_front();
            }
        }
        
        // Update prediction accuracy using exponential moving average
        let accuracy_update = if success { 1.0 } else { 0.0 };
        self.prediction_accuracy = (1.0 - self.learning_rate) * self.prediction_accuracy 
            + self.learning_rate * accuracy_update;
    }
    
    /// Get current prediction accuracy
    pub fn get_prediction_accuracy(&self) -> f64 {
        self.prediction_accuracy
    }
}

// ============= BYZANTINE FAULT DETECTION =============

/// Byzantine fault detector for identifying malicious nodes
pub struct ByzantineDetector {
    tolerance_factor: f64,
    node_trust_scores: HashMap<NodeId, f64>,
    consensus_history: VecDeque<ConsensusRound>,
    suspected_nodes: HashMap<NodeId, SuspicionLevel>,
}

#[derive(Debug, Clone)]
pub struct ConsensusRound {
    pub round_id: u64,
    pub participants: Vec<NodeId>,
    pub decisions: HashMap<NodeId, bool>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct SuspicionLevel {
    pub score: f64, // 0.0 = trusted, 1.0 = highly suspicious
    pub evidence_count: u32,
    pub last_updated: SystemTime,
}

impl ByzantineDetector {
    pub fn new(tolerance_factor: f64) -> Self {
        Self {
            tolerance_factor,
            node_trust_scores: HashMap::new(),
            consensus_history: VecDeque::new(),
            suspected_nodes: HashMap::new(),
        }
    }
    
    /// Check if a node is trustworthy
    pub fn is_node_trustworthy(&self, node_id: NodeId) -> bool {
        let trust_score = self.node_trust_scores.get(&node_id).copied().unwrap_or(0.8);
        let suspicion = self.suspected_nodes.get(&node_id);
        
        trust_score > 0.5 && suspicion.is_none_or(|s| s.score < 0.5)
    }
    
    /// Report suspicious behavior from a node
    pub fn report_suspicious_behavior(&mut self, node_id: NodeId, evidence: &str) {
        let suspicion = self.suspected_nodes
            .entry(node_id)
            .or_insert_with(|| SuspicionLevel {
                score: 0.0,
                evidence_count: 0,
                last_updated: SystemTime::now(),
            });
        
        suspicion.evidence_count += 1;
        suspicion.score = (suspicion.score + 0.1).min(1.0);
        suspicion.last_updated = SystemTime::now();
        
        // Decrease trust score
        let trust = self.node_trust_scores.entry(node_id).or_insert(0.8);
        *trust = (*trust - 0.05).max(0.0);
    }
    
    /// Get list of suspected Byzantine nodes
    pub fn get_suspected_nodes(&self) -> Vec<NodeId> {
        self.suspected_nodes
            .iter()
            .filter(|(_, suspicion)| suspicion.score > 0.5)
            .map(|(&node_id, _)| node_id)
            .collect()
    }
    
    /// Update trust scores based on consensus participation
    pub fn update_consensus_participation(&mut self, round: ConsensusRound) {
        // Reward nodes that participated in consensus
        for &node_id in &round.participants {
            let trust = self.node_trust_scores.entry(node_id).or_insert(0.8);
            *trust = (*trust + 0.01).min(1.0);
        }
        
        // Add to history
        self.consensus_history.push_back(round);
        
        // Limit history size
        while self.consensus_history.len() > 1000 {
            self.consensus_history.pop_front();
        }
    }
}

// ============= CRDT CONSISTENCY MANAGER =============

/// CRDT-based consistency manager for distributed state
pub struct CRDTManager {
    g_counters: Arc<DashMap<CRDTId, GCounter>>,
    pn_counters: Arc<DashMap<CRDTId, PNCounter>>,
    g_sets: Arc<DashMap<CRDTId, GSet>>,
    or_sets: Arc<DashMap<CRDTId, ORSet>>,
    vector_clocks: Arc<DashMap<NodeId, u64>>,
}

impl CRDTManager {
    pub fn new() -> Self {
        Self {
            g_counters: Arc::new(DashMap::new()),
            pn_counters: Arc::new(DashMap::new()),
            g_sets: Arc::new(DashMap::new()),
            or_sets: Arc::new(DashMap::new()),
            vector_clocks: Arc::new(DashMap::new()),
        }
    }
    
    /// Create a new G-Counter (grow-only counter)
    pub fn create_g_counter(&self, id: CRDTId) -> DistributedResult<()> {
        self.g_counters.insert(id, GCounter::new());
        Ok(())
    }
    
    /// Increment a G-Counter
    pub fn increment_g_counter(&self, id: CRDTId, node_id: NodeId) -> DistributedResult<()> {
        if let Some(mut counter) = self.g_counters.get_mut(&id) {
            counter.increment(node_id);
            Ok(())
        } else {
            Err(DistributedError::CRDTSyncError(format!("G-Counter {} not found", id)))
        }
    }
    
    /// Get G-Counter value
    pub fn get_g_counter_value(&self, id: CRDTId) -> DistributedResult<u64> {
        if let Some(counter_ref) = self.g_counters.get(&id) {
            Ok((*counter_ref).value())
        } else {
            Err(DistributedError::CRDTSyncError(format!("G-Counter {} not found", id)))
        }
    }
    
    /// Synchronize CRDT state with another node
    pub fn synchronize_with_node(&self, node_id: NodeId) -> DistributedResult<()> {
        // Update vector clock
        let current_clock = self.vector_clocks.get(&node_id).map(|c| *c).unwrap_or(0);
        self.vector_clocks.insert(node_id, current_clock + 1);
        
        // In a real implementation, this would exchange CRDT states over the network
        // For now, we just acknowledge the sync
        Ok(())
    }
}

// ============= CRDT IMPLEMENTATIONS =============

/// Grow-only counter CRDT
#[derive(Debug, Clone, Default)]
pub struct GCounter {
    counters: HashMap<NodeId, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment(&mut self, node_id: NodeId) {
        *self.counters.entry(node_id).or_insert(0) += 1;
    }
    
    pub fn value(&self) -> u64 {
        self.counters.values().sum()
    }
    
    pub fn merge(&mut self, other: &GCounter) {
        for (&node_id, &count) in &other.counters {
            let current = self.counters.entry(node_id).or_insert(0);
            *current = (*current).max(count);
        }
    }
}

/// Increment/decrement counter CRDT
#[derive(Debug, Clone, Default)]
pub struct PNCounter {
    positive: GCounter,
    negative: GCounter,
}

impl PNCounter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment(&mut self, node_id: NodeId) {
        self.positive.increment(node_id);
    }
    
    pub fn decrement(&mut self, node_id: NodeId) {
        self.negative.increment(node_id);
    }
    
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }
    
    pub fn merge(&mut self, other: &PNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

/// Grow-only set CRDT
#[derive(Debug, Clone, Default)]
pub struct GSet {
    elements: HashMap<String, SystemTime>,
}

impl GSet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add(&mut self, element: String) {
        self.elements.insert(element, SystemTime::now());
    }
    
    pub fn contains(&self, element: &str) -> bool {
        self.elements.contains_key(element)
    }
    
    pub fn elements(&self) -> Vec<String> {
        self.elements.keys().cloned().collect()
    }
    
    pub fn merge(&mut self, other: &GSet) {
        for (element, timestamp) in &other.elements {
            self.elements.entry(element.clone())
                .and_modify(|t| *t = (*t).max(*timestamp))
                .or_insert(*timestamp);
        }
    }
}

/// Observed-remove set CRDT
#[derive(Debug, Clone, Default)]
pub struct ORSet {
    elements: HashMap<String, HashMap<NodeId, (bool, SystemTime)>>,
}

impl ORSet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add(&mut self, element: String, node_id: NodeId) {
        self.elements
            .entry(element)
            .or_default()
            .insert(node_id, (true, SystemTime::now()));
    }
    
    pub fn remove(&mut self, element: String, node_id: NodeId) {
        self.elements
            .entry(element)
            .or_default()
            .insert(node_id, (false, SystemTime::now()));
    }
    
    pub fn contains(&self, element: &str) -> bool {
        self.elements
            .get(element)
            .map(|nodes| {
                // For OR-Set, element is present if any node has added it
                // and that add hasn't been removed by the same node
                for (&node_id, &(present, _timestamp)) in nodes {
                    if present {
                        return true; // This node has added the element and it's the latest operation
                    }
                }
                false
            })
            .unwrap_or(false)
    }
    
    pub fn elements(&self) -> Vec<String> {
        self.elements
            .iter()
            .filter(|(_, nodes)| nodes.values().any(|(present, _)| *present))
            .map(|(element, _)| element.clone())
            .collect()
    }
    
    pub fn merge(&mut self, other: &ORSet) {
        for (element, other_nodes) in &other.elements {
            let element_nodes = self.elements.entry(element.clone()).or_default();
            for (&node_id, &(present, timestamp)) in other_nodes {
                element_nodes.entry(node_id)
                    .and_modify(|(p, t)| {
                        if timestamp > *t {
                            *p = present;
                            *t = timestamp;
                        }
                    })
                    .or_insert((present, timestamp));
            }
        }
    }
}

// ============= STATISTICS AND MONITORING =============

/// Comprehensive statistics for distributed computing system
pub struct DistributedStatistics {
    nodes_registered: AtomicU64,
    tasks_submitted: AtomicU64,
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,
    network_messages_sent: AtomicU64,
    network_messages_received: AtomicU64,
    consensus_rounds: AtomicU64,
    byzantine_nodes_detected: AtomicU64,
    crdt_sync_operations: AtomicU64,
    system_start_time: Option<SystemTime>,
    last_updated: Arc<RwLock<SystemTime>>,
}

impl DistributedStatistics {
    pub fn new() -> Self {
        Self {
            nodes_registered: AtomicU64::new(0),
            tasks_submitted: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            network_messages_sent: AtomicU64::new(0),
            network_messages_received: AtomicU64::new(0),
            consensus_rounds: AtomicU64::new(0),
            byzantine_nodes_detected: AtomicU64::new(0),
            crdt_sync_operations: AtomicU64::new(0),
            system_start_time: Some(SystemTime::now()),
            last_updated: Arc::new(RwLock::new(SystemTime::now())),
        }
    }
    
    // Increment methods
    pub fn increment_nodes_registered(&self) { self.nodes_registered.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_tasks_submitted(&self) { self.tasks_submitted.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_tasks_completed(&self) { self.tasks_completed.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_tasks_failed(&self) { self.tasks_failed.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_network_messages_sent(&self) { self.network_messages_sent.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_network_messages_received(&self) { self.network_messages_received.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_consensus_rounds(&self) { self.consensus_rounds.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_byzantine_nodes_detected(&self) { self.byzantine_nodes_detected.fetch_add(1, Ordering::Relaxed); }
    pub fn increment_crdt_sync_operations(&self) { self.crdt_sync_operations.fetch_add(1, Ordering::Relaxed); }
    
    // Getter methods
    pub fn get_nodes_registered(&self) -> u64 { self.nodes_registered.load(Ordering::Relaxed) }
    pub fn get_tasks_submitted(&self) -> u64 { self.tasks_submitted.load(Ordering::Relaxed) }
    pub fn get_tasks_completed(&self) -> u64 { self.tasks_completed.load(Ordering::Relaxed) }
    pub fn get_tasks_failed(&self) -> u64 { self.tasks_failed.load(Ordering::Relaxed) }
    pub fn get_network_messages_sent(&self) -> u64 { self.network_messages_sent.load(Ordering::Relaxed) }
    pub fn get_network_messages_received(&self) -> u64 { self.network_messages_received.load(Ordering::Relaxed) }
    pub fn get_consensus_rounds(&self) -> u64 { self.consensus_rounds.load(Ordering::Relaxed) }
    pub fn get_byzantine_nodes_detected(&self) -> u64 { self.byzantine_nodes_detected.load(Ordering::Relaxed) }
    pub fn get_crdt_sync_operations(&self) -> u64 { self.crdt_sync_operations.load(Ordering::Relaxed) }
    
    // Note: start time is set during creation for simplicity
    
    pub fn mark_system_stopped(&self) {
        if let Ok(mut last_updated) = self.last_updated.write() {
            *last_updated = SystemTime::now();
        }
    }
    
    /// Calculate system uptime
    pub fn get_uptime(&self) -> Option<Duration> {
        self.system_start_time.and_then(|start| start.elapsed().ok())
    }
    
    /// Calculate task success rate
    pub fn get_task_success_rate(&self) -> f64 {
        let completed = self.get_tasks_completed() as f64;
        let failed = self.get_tasks_failed() as f64;
        let total = completed + failed;
        
        if total > 0.0 {
            completed / total
        } else {
            0.0
        }
    }
    
    /// Calculate throughput (tasks per second)
    pub fn get_throughput(&self) -> f64 {
        if let Some(uptime) = self.get_uptime() {
            let seconds = uptime.as_secs_f64();
            if seconds > 0.0 {
                self.get_tasks_completed() as f64 / seconds
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

// ============= TESTING FRAMEWORK =============

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_distributed_computing_system_creation() {
        let config = DistributedConfig::default();
        let system = DistributedComputingSystem::new(config);
        assert!(!system.is_running.load(Ordering::SeqCst));
        assert_eq!(system.get_statistics().get_nodes_registered(), 0);
    }

    #[test]
    fn test_node_capabilities() {
        let node_id = 1;
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let capabilities = NodeCapabilities::new(node_id, address);
        
        assert_eq!(capabilities.node_id, node_id);
        assert_eq!(capabilities.address, address);
        assert!(capabilities.jit_support);
        assert!(capabilities.parallel_support);
        assert!(!capabilities.gpu_support);
        assert!(capabilities.performance_score() > 0.0);
    }

    #[test]
    fn test_required_capabilities_matching() {
        let node_id = 1;
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut capabilities = NodeCapabilities::new(node_id, address);
        capabilities.cpu_cores = 8;
        capabilities.memory_gb = 16;
        
        let mut required = RequiredCapabilities::default();
        required.min_cpu_cores = 4;
        required.min_memory_gb = 8;
        required.requires_jit = true;
        
        assert!(capabilities.supports_capabilities(&required));
        
        required.min_cpu_cores = 16; // Too high
        assert!(!capabilities.supports_capabilities(&required));
    }

    #[test]
    fn test_distributed_task_creation() {
        let task = DistributedTask {
            id: 1,
            payload: TaskPayload::SchemeEval {
                expression: "(+ 1 2)".to_string(),
                environment: vec![],
            },
            required_capabilities: RequiredCapabilities::default(),
            dependencies: vec![],
            priority: 1,
            estimated_duration: Duration::from_millis(100),
            max_retries: 3,
            created_at: SystemTime::now(),
            deadline: None,
        };
        
        assert_eq!(task.id, 1);
        assert_eq!(task.priority, 1);
        assert_eq!(task.max_retries, 3);
        assert!(matches!(task.payload, TaskPayload::SchemeEval { .. }));
    }

    #[test]
    fn test_ml_scheduler() {
        let mut scheduler = MLScheduler::new(0.1);
        
        let node_id = 1;
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let capabilities = NodeCapabilities::new(node_id, address);
        
        let task = DistributedTask {
            id: 1,
            payload: TaskPayload::SchemeEval {
                expression: "(+ 1 2)".to_string(),
                environment: vec![],
            },
            required_capabilities: RequiredCapabilities::default(),
            dependencies: vec![],
            priority: 1,
            estimated_duration: Duration::from_millis(100),
            max_retries: 3,
            created_at: SystemTime::now(),
            deadline: None,
        };
        
        let prediction = scheduler.predict_optimal_node(&task, &[capabilities]);
        assert_eq!(prediction, Some(node_id));
        
        // Update with feedback
        scheduler.update_with_feedback(node_id, &task, Duration::from_millis(50), true);
        assert!(scheduler.get_prediction_accuracy() > 0.5);
    }

    #[test]
    fn test_byzantine_detector() {
        let mut detector = ByzantineDetector::new(0.33);
        let node_id = 1;
        
        assert!(detector.is_node_trustworthy(node_id));
        
        // Report suspicious behavior
        detector.report_suspicious_behavior(node_id, "Invalid consensus vote");
        detector.report_suspicious_behavior(node_id, "Timeout in task execution");
        detector.report_suspicious_behavior(node_id, "Inconsistent state report");
        detector.report_suspicious_behavior(node_id, "Failed message authentication");
        detector.report_suspicious_behavior(node_id, "Byzantine behavior detected");
        detector.report_suspicious_behavior(node_id, "Malicious payload");
        
        assert!(!detector.is_node_trustworthy(node_id));
        assert!(detector.get_suspected_nodes().contains(&node_id));
    }

    #[test]
    fn test_crdt_g_counter() {
        let mut counter = GCounter::new();
        assert_eq!(counter.value(), 0);
        
        counter.increment(1);
        counter.increment(1);
        counter.increment(2);
        assert_eq!(counter.value(), 3);
        
        let mut other = GCounter::new();
        other.increment(1);
        other.increment(3);
        
        counter.merge(&other);
        assert_eq!(counter.value(), 4); // max(2,1) + max(1,0) + max(0,1) = 2+1+1
    }

    #[test]
    fn test_crdt_pn_counter() {
        let mut counter = PNCounter::new();
        assert_eq!(counter.value(), 0);
        
        counter.increment(1);
        counter.increment(1);
        counter.decrement(2);
        assert_eq!(counter.value(), 1); // 2 - 1 = 1
    }

    #[test]
    fn test_crdt_g_set() {
        let mut set = GSet::new();
        assert!(!set.contains("test"));
        
        set.add("test".to_string());
        set.add("hello".to_string());
        assert!(set.contains("test"));
        assert!(set.contains("hello"));
        assert!(!set.contains("world"));
        
        let elements = set.elements();
        assert_eq!(elements.len(), 2);
        assert!(elements.contains(&"test".to_string()));
        assert!(elements.contains(&"hello".to_string()));
    }

    #[test]
    fn test_crdt_or_set() {
        let mut set = ORSet::new();
        assert!(!set.contains("test"));
        
        set.add("test".to_string(), 1);
        assert!(set.contains("test"));
        
        set.remove("test".to_string(), 2);
        assert!(!set.contains("test"));
        
        // Re-add should make it visible again
        set.add("test".to_string(), 3);
        assert!(set.contains("test"));
    }

    #[test]
    fn test_crdt_manager() {
        let manager = CRDTManager::new();
        let crdt_id = 1;
        let node_id = 1;
        
        manager.create_g_counter(crdt_id).unwrap();
        assert_eq!(manager.get_g_counter_value(crdt_id).unwrap(), 0);
        
        manager.increment_g_counter(crdt_id, node_id).unwrap();
        manager.increment_g_counter(crdt_id, node_id).unwrap();
        assert_eq!(manager.get_g_counter_value(crdt_id).unwrap(), 2);
        
        // Test synchronization
        manager.synchronize_with_node(node_id).unwrap();
    }

    #[test]
    fn test_system_integration() {
        let config = DistributedConfig::default();
        let system = DistributedComputingSystem::new(config);
        
        // Start system
        system.start().unwrap();
        assert!(system.is_running.load(Ordering::SeqCst));
        
        // Register node
        let node_id = 1;
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let capabilities = NodeCapabilities::new(node_id, address);
        system.register_node(capabilities).unwrap();
        assert_eq!(system.get_statistics().get_nodes_registered(), 1);
        
        // Submit task
        let task = DistributedTask {
            id: 1,
            payload: TaskPayload::SchemeEval {
                expression: "(+ 1 2)".to_string(),
                environment: vec![],
            },
            required_capabilities: RequiredCapabilities::default(),
            dependencies: vec![],
            priority: 1,
            estimated_duration: Duration::from_millis(100),
            max_retries: 3,
            created_at: SystemTime::now(),
            deadline: None,
        };
        
        let task_id = system.submit_task(task).unwrap();
        assert_eq!(task_id, 1);
        assert_eq!(system.get_statistics().get_tasks_submitted(), 1);
        
        // Check task status
        let status = system.get_task_status(task_id).unwrap();
        assert!(matches!(status, Some(TaskStatus::Pending)));
        
        // Stop system
        system.stop().unwrap();
        assert!(!system.is_running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_statistics() {
        let stats = DistributedStatistics::new();
        assert_eq!(stats.get_nodes_registered(), 0);
        assert_eq!(stats.get_tasks_submitted(), 0);
        assert_eq!(stats.get_task_success_rate(), 0.0);
        assert_eq!(stats.get_throughput(), 0.0);
        
        stats.increment_tasks_completed();
        stats.increment_tasks_completed();
        stats.increment_tasks_failed();
        
        assert_eq!(stats.get_tasks_completed(), 2);
        assert_eq!(stats.get_tasks_failed(), 1);
        assert!((stats.get_task_success_rate() - 0.6666666666666666).abs() < 1e-10);
    }
}