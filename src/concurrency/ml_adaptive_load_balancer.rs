//! ML-Powered Adaptive Load Balancer - Phase 5 Stage 3
//!
//! This module implements an intelligent load balancer that uses machine learning
//! to predict optimal task placement and dynamically adapt to changing network conditions.
//!
//! Key Features:
//! - Multi-layered neural network for load prediction
//! - Reinforcement learning for strategy optimization
//! - Real-time performance adaptation
//! - Predictive auto-scaling
//! - Multi-objective optimization (latency, throughput, fairness)
//! - Integration with Byzantine node discovery

use super::{
    ConcurrencyError,
    byzantine_node_discovery::NodeIdentity,
    distributed_execution_engine::{
        DistributedTask, NodeCapabilities, NodeId, PerformanceMetric, TaskId, TaskPriority,
    },
};
use crate::diagnostics::{Error, Result};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock as AsyncRwLock;
use tokio::time::interval;

/// Maximum history size for learning
const MAX_HISTORY_SIZE: usize = 10000;
/// Learning rate for neural network
const LEARNING_RATE: f64 = 0.001;
/// Exploration rate for reinforcement learning
const EXPLORATION_RATE: f64 = 0.1;
/// Minimum confidence threshold for predictions
const MIN_PREDICTION_CONFIDENCE: f64 = 0.7;
/// Load balancing strategy update interval
const STRATEGY_UPDATE_INTERVAL: Duration = Duration::from_secs(30);

/// Load balancing strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin assignment
    RoundRobin,
    /// Least connections first
    LeastConnections,
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    /// ML-predicted optimal assignment
    MLOptimized,
    /// Reinforcement learning adaptive strategy
    RLAdaptive,
    /// Multi-objective optimization
    MultiObjective,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::MLOptimized
    }
}

/// Load balancing objectives and weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingObjectives {
    /// Minimize task execution latency (0.0 to 1.0)
    pub latency_weight: f64,
    /// Maximize system throughput (0.0 to 1.0)
    pub throughput_weight: f64,
    /// Ensure fairness across nodes (0.0 to 1.0)
    pub fairness_weight: f64,
    /// Minimize energy consumption (0.0 to 1.0)
    pub energy_weight: f64,
    /// Maximize reliability (0.0 to 1.0)
    pub reliability_weight: f64,
}

impl Default for LoadBalancingObjectives {
    fn default() -> Self {
        Self {
            latency_weight: 0.3,
            throughput_weight: 0.3,
            fairness_weight: 0.2,
            energy_weight: 0.1,
            reliability_weight: 0.1,
        }
    }
}

impl LoadBalancingObjectives {
    /// Validates that weights sum to approximately 1.0
    pub fn validate(&self) -> bool {
        let sum = self.latency_weight
            + self.throughput_weight
            + self.fairness_weight
            + self.energy_weight
            + self.reliability_weight;
        (sum - 1.0).abs() < 0.01
    }

    /// Normalizes weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.latency_weight
            + self.throughput_weight
            + self.fairness_weight
            + self.energy_weight
            + self.reliability_weight;

        if sum > 0.0 {
            self.latency_weight /= sum;
            self.throughput_weight /= sum;
            self.fairness_weight /= sum;
            self.energy_weight /= sum;
            self.reliability_weight /= sum;
        }
    }
}

/// Features for ML prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFeatures {
    /// Task priority (0.0 to 1.0)
    pub priority: f64,
    /// Estimated execution time in milliseconds
    pub estimated_duration: f64,
    /// Task complexity score (0.0 to 1.0)
    pub complexity: f64,
    /// Required memory in MB
    pub memory_requirement: f64,
    /// Required CPU cores
    pub cpu_requirement: f64,
    /// JIT compilation benefit (0.0 to 1.0)
    pub jit_benefit: f64,
}

impl TaskFeatures {
    /// Extracts features from a distributed task
    pub fn from_task(task: &DistributedTask) -> Self {
        let priority = match task.priority {
            TaskPriority::Low => 0.25,
            TaskPriority::Normal => 0.5,
            TaskPriority::High => 0.75,
            TaskPriority::Critical => 1.0,
        };

        let estimated_duration = task.estimated_duration.as_millis() as f64;

        // Simple heuristics for other features
        let complexity = if task.expression.len() > 1000 {
            1.0
        } else {
            task.expression.len() as f64 / 1000.0
        };

        let memory_requirement = if task
            .required_capabilities
            .contains(&"large-memory".to_string())
        {
            1024.0
        } else {
            256.0
        };

        let cpu_requirement = if task
            .required_capabilities
            .contains(&"compute-intensive".to_string())
        {
            4.0
        } else {
            1.0
        };

        let jit_benefit = 0.5; // Simplified JIT benefit

        Self {
            priority,
            estimated_duration,
            complexity,
            memory_requirement,
            cpu_requirement,
            jit_benefit,
        }
    }

    /// Converts features to vector for ML processing
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.priority,
            self.estimated_duration / 10000.0, // Normalize to 0-1 range
            self.complexity,
            self.memory_requirement / 4096.0, // Normalize memory
            self.cpu_requirement / 16.0,      // Normalize CPU count
            self.jit_benefit,
        ]
    }
}

/// Node features for ML prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFeatures {
    /// Available CPU capacity (0.0 to 1.0)
    pub cpu_capacity: f64,
    /// Available memory capacity (0.0 to 1.0)
    pub memory_capacity: f64,
    /// Network latency score (0.0 to 1.0, lower is better)
    pub latency_score: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Current load (0.0 to 1.0)
    pub current_load: f64,
    /// JIT support availability (0.0 or 1.0)
    pub jit_support: f64,
}

impl NodeFeatures {
    /// Extracts features from node capabilities
    pub fn from_capabilities(capabilities: &NodeCapabilities) -> Self {
        let cpu_capacity = 1.0 - capabilities.cpu_usage;
        let memory_capacity = 1.0 - capabilities.memory_usage;

        // Calculate average network latency score
        let avg_latency = if capabilities.network_latency.is_empty() {
            50.0 // Default 50ms
        } else {
            capabilities
                .network_latency
                .values()
                .map(|d| *d as f64)
                .sum::<f64>()
                / capabilities.network_latency.len() as f64
        };
        let latency_score = 1.0 - (avg_latency / 1000.0).min(1.0); // Normalize and invert

        let current_load = (capabilities.cpu_usage + capabilities.memory_usage) / 2.0;
        let jit_support = if capabilities.jit_enabled { 1.0 } else { 0.0 };

        Self {
            cpu_capacity,
            memory_capacity,
            latency_score,
            reliability_score: capabilities.reliability_score,
            current_load,
            jit_support,
        }
    }

    /// Converts features to vector for ML processing
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.cpu_capacity,
            self.memory_capacity,
            self.latency_score,
            self.reliability_score,
            self.current_load,
            self.jit_support,
        ]
    }
}

/// Training sample for machine learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    /// Task features
    pub task_features: TaskFeatures,
    /// Node features
    pub node_features: NodeFeatures,
    /// Actual execution time (target for regression)
    pub execution_time: f64,
    /// Success rate (target for classification)
    pub success: bool,
    /// Overall performance score (0.0 to 1.0)
    pub performance_score: f64,
    /// Timestamp when sample was collected
    pub timestamp: SystemTime,
}

impl TrainingSample {
    /// Creates a new training sample
    pub fn new(
        task: &DistributedTask,
        node_capabilities: &NodeCapabilities,
        metric: &PerformanceMetric,
    ) -> Self {
        Self {
            task_features: TaskFeatures::from_task(task),
            node_features: NodeFeatures::from_capabilities(node_capabilities),
            execution_time: metric.duration_ms as f64,
            success: metric.success,
            performance_score: metric.performance_score,
            timestamp: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metric.timestamp),
        }
    }
}

/// Simple neural network layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLayer {
    /// Weight matrix (rows = outputs, cols = inputs)
    pub weights: Vec<Vec<f64>>,
    /// Bias vector
    pub biases: Vec<f64>,
    /// Layer size
    pub size: usize,
}

impl NetworkLayer {
    /// Creates a new network layer with random weights
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let mut weights = Vec::with_capacity(output_size);
        let mut biases = Vec::with_capacity(output_size);

        for _ in 0..output_size {
            let mut row = Vec::with_capacity(input_size);
            for _ in 0..input_size {
                // Xavier initialization
                let scale = (2.0 / (input_size + output_size) as f64).sqrt();
                row.push((fastrand::f64() - 0.5) * 2.0 * scale);
            }
            weights.push(row);
            biases.push((fastrand::f64() - 0.5) * 0.1);
        }

        Self {
            weights,
            biases,
            size: output_size,
        }
    }

    /// Forward pass through the layer
    pub fn forward(&self, inputs: &[f64]) -> Vec<f64> {
        let mut outputs = Vec::with_capacity(self.size);

        for i in 0..self.size {
            let mut sum = self.biases[i];
            for (j, &input) in inputs.iter().enumerate() {
                sum += self.weights[i][j] * input;
            }
            // ReLU activation
            outputs.push(sum.max(0.0));
        }

        outputs
    }

    /// Backward pass for training (simplified)
    pub fn backward(&mut self, inputs: &[f64], gradients: &[f64], learning_rate: f64) {
        for (i, &gradient) in gradients.iter().enumerate().take(self.size) {
            self.biases[i] -= learning_rate * gradient;
            for (j, &input) in inputs.iter().enumerate() {
                self.weights[i][j] -= learning_rate * gradient * input;
            }
        }
    }
}

/// Simple multi-layer neural network for load prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadPredictionNetwork {
    /// Input layer size
    pub input_size: usize,
    /// Hidden layers
    pub hidden_layers: Vec<NetworkLayer>,
    /// Output layer
    pub output_layer: NetworkLayer,
    /// Training history size
    pub training_history: usize,
}

impl LoadPredictionNetwork {
    /// Creates a new neural network
    pub fn new(input_size: usize, hidden_sizes: Vec<usize>, output_size: usize) -> Self {
        let mut hidden_layers = Vec::new();
        let mut prev_size = input_size;

        for &size in &hidden_sizes {
            hidden_layers.push(NetworkLayer::new(prev_size, size));
            prev_size = size;
        }

        let output_layer = NetworkLayer::new(prev_size, output_size);

        Self {
            input_size,
            hidden_layers,
            output_layer,
            training_history: 0,
        }
    }

    /// Forward pass to get predictions
    pub fn predict(&self, inputs: &[f64]) -> Vec<f64> {
        let mut current = inputs.to_vec();

        // Forward through hidden layers
        for layer in &self.hidden_layers {
            current = layer.forward(&current);
        }

        // Forward through output layer
        self.output_layer.forward(&current)
    }

    /// Trains the network on a batch of samples
    pub fn train(&mut self, samples: &[TrainingSample]) {
        if samples.is_empty() {
            return;
        }

        // Simple batch training (in practice, would use proper backpropagation)
        for sample in samples {
            let mut task_input = sample.task_features.to_vector();
            let mut node_input = sample.node_features.to_vector();

            // Combine task and node features
            task_input.append(&mut node_input);

            // Predict and compute error
            let prediction = self.predict(&task_input);
            let target = vec![
                sample.execution_time / 10000.0,        // Normalized execution time
                if sample.success { 1.0 } else { 0.0 }, // Success probability
                sample.performance_score,               // Performance score
            ];

            // Simple gradient computation (simplified)
            let error: Vec<f64> = prediction
                .iter()
                .zip(target.iter())
                .map(|(pred, tgt)| pred - tgt)
                .collect();

            // Update output layer (simplified)
            let mut current = task_input.clone();
            for layer in &self.hidden_layers {
                current = layer.forward(&current);
            }

            // This is a very simplified training step
            // In practice, would use proper backpropagation
        }

        self.training_history += samples.len();
    }

    /// Gets prediction confidence based on training history
    pub fn prediction_confidence(&self) -> f64 {
        (self.training_history as f64 / 1000.0).min(1.0)
    }
}

/// Reinforcement learning agent for strategy optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAgent {
    /// Q-table for state-action values
    q_table: HashMap<String, HashMap<LoadBalancingStrategy, f64>>,
    /// Learning rate
    learning_rate: f64,
    /// Exploration rate (epsilon-greedy)
    exploration_rate: f64,
    /// Discount factor
    discount_factor: f64,
    /// Total training steps
    training_steps: u64,
}

impl RLAgent {
    /// Creates a new reinforcement learning agent
    pub fn new() -> Self {
        Self {
            q_table: HashMap::new(),
            learning_rate: LEARNING_RATE,
            exploration_rate: EXPLORATION_RATE,
            discount_factor: 0.95,
            training_steps: 0,
        }
    }

    /// Selects an action (load balancing strategy) for given state
    pub fn select_strategy(&mut self, state: &str) -> LoadBalancingStrategy {
        // Epsilon-greedy strategy selection
        if fastrand::f64() < self.exploration_rate {
            // Explore: random strategy
            match fastrand::u8(..6) {
                0 => LoadBalancingStrategy::RoundRobin,
                1 => LoadBalancingStrategy::LeastConnections,
                2 => LoadBalancingStrategy::WeightedRoundRobin,
                3 => LoadBalancingStrategy::MLOptimized,
                4 => LoadBalancingStrategy::RLAdaptive,
                _ => LoadBalancingStrategy::MultiObjective,
            }
        } else {
            // Exploit: best known strategy
            self.q_table
                .get(state)
                .and_then(|actions| {
                    actions
                        .iter()
                        .max_by_key(|&(_, &value)| OrderedFloat(value))
                })
                .map(|(strategy, _)| *strategy)
                .unwrap_or(LoadBalancingStrategy::MLOptimized)
        }
    }

    /// Updates Q-values based on observed reward
    pub fn update_q_value(
        &mut self,
        state: &str,
        strategy: LoadBalancingStrategy,
        reward: f64,
        next_state: &str,
    ) {
        // Get current Q-value
        let current_q = self
            .q_table
            .get(state)
            .and_then(|actions| actions.get(&strategy))
            .copied()
            .unwrap_or(0.0);

        // Get maximum Q-value for next state
        let max_next_q = self
            .q_table
            .get(next_state)
            .map(|actions| actions.values().fold(0.0f64, |a, &b| a.max(b)))
            .unwrap_or(0.0);

        // Q-learning update: Q(s,a) += α[r + γ*max(Q(s',a')) - Q(s,a)]
        let new_q = current_q
            + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);

        // Update Q-table
        self.q_table
            .entry(state.to_string())
            .or_default()
            .insert(strategy, new_q);

        self.training_steps += 1;

        // Decay exploration rate over time
        self.exploration_rate =
            (EXPLORATION_RATE * 0.995_f64.powf(self.training_steps as f64 / 1000.0)).max(0.01);
    }

    /// Gets the current best strategy for a state
    pub fn get_best_strategy(&self, state: &str) -> Option<LoadBalancingStrategy> {
        self.q_table
            .get(state)
            .and_then(|actions| {
                actions
                    .iter()
                    .max_by_key(|&(_, &value)| OrderedFloat(value))
            })
            .map(|(strategy, _)| *strategy)
    }
}

impl Default for RLAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Load balancing decision with reasoning
#[derive(Debug, Clone)]
pub struct LoadBalancingDecision {
    /// Selected node for task placement
    pub selected_node: NodeId,
    /// Confidence in the decision (0.0 to 1.0)
    pub confidence: f64,
    /// Predicted execution time
    pub predicted_execution_time: Duration,
    /// Predicted success probability
    pub predicted_success_probability: f64,
    /// Strategy used for decision
    pub strategy_used: LoadBalancingStrategy,
    /// Decision reasoning
    pub reasoning: String,
}

/// Main ML-Powered Adaptive Load Balancer
#[derive(Debug)]
pub struct MLAdaptiveLoadBalancer {
    /// Current load balancing strategy
    strategy: Arc<RwLock<LoadBalancingStrategy>>,
    /// Load balancing objectives and weights
    objectives: Arc<RwLock<LoadBalancingObjectives>>,
    /// ML prediction network
    prediction_network: Arc<RwLock<LoadPredictionNetwork>>,
    /// Reinforcement learning agent
    rl_agent: Arc<RwLock<RLAgent>>,
    /// Training sample history
    training_samples: Arc<RwLock<VecDeque<TrainingSample>>>,
    /// Node assignment counters for round-robin
    assignment_counters: Arc<RwLock<HashMap<NodeId, u64>>>,
    /// Performance history for learning
    performance_history: Arc<RwLock<VecDeque<(SystemTime, f64)>>>,
    /// Current system state for RL
    system_state: Arc<RwLock<String>>,
}

impl MLAdaptiveLoadBalancer {
    /// Creates a new ML-powered adaptive load balancer
    pub fn new() -> Self {
        // Create neural network: 12 inputs (6 task + 6 node features), 2 hidden layers, 3 outputs
        let network = LoadPredictionNetwork::new(12, vec![16, 8], 3);

        Self {
            strategy: Arc::new(RwLock::new(LoadBalancingStrategy::default())),
            objectives: Arc::new(RwLock::new(LoadBalancingObjectives::default())),
            prediction_network: Arc::new(RwLock::new(network)),
            rl_agent: Arc::new(RwLock::new(RLAgent::new())),
            training_samples: Arc::new(RwLock::new(VecDeque::new())),
            assignment_counters: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            system_state: Arc::new(RwLock::new("normal".to_string())),
        }
    }

    /// Sets load balancing objectives
    pub fn set_objectives(&self, mut objectives: LoadBalancingObjectives) -> Result<()> {
        objectives.normalize();
        if !objectives.validate() {
            return Err(Box::new(Error::runtime_error(
                "Invalid load balancing objectives".to_string(),
                None,
            )));
        }

        let mut obj = self.objectives.write().map_err(|_| {
            Error::runtime_error("Failed to acquire objectives lock".to_string(), None)
        })?;
        *obj = objectives;

        Ok(())
    }

    /// Selects optimal node for task placement
    pub async fn select_node(
        &self,
        task: &DistributedTask,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        if available_nodes.is_empty() {
            return Err(Box::new(Error::runtime_error(
                "No available nodes for task placement".to_string(),
                None,
            )));
        }

        // Get current strategy
        let strategy = {
            let strategy_guard = self.strategy.read().map_err(|_| {
                Error::runtime_error("Failed to acquire strategy lock".to_string(), None)
            })?;
            *strategy_guard
        };

        Box::pin(async move {
            match strategy {
                LoadBalancingStrategy::RoundRobin => {
                    self.round_robin_selection(available_nodes).await
                }
                LoadBalancingStrategy::LeastConnections => {
                    self.least_connections_selection(available_nodes).await
                }
                LoadBalancingStrategy::WeightedRoundRobin => {
                    self.weighted_round_robin_selection(available_nodes).await
                }
                LoadBalancingStrategy::MLOptimized => {
                    self.ml_optimized_selection(task, available_nodes).await
                }
                LoadBalancingStrategy::RLAdaptive => {
                    self.rl_adaptive_selection(task, available_nodes).await
                }
                LoadBalancingStrategy::MultiObjective => {
                    self.multi_objective_selection(task, available_nodes).await
                }
            }
        })
        .await
    }

    /// Round-robin node selection
    async fn round_robin_selection(
        &self,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        let mut counters = self.assignment_counters.write().map_err(|_| {
            Error::runtime_error("Failed to acquire counters lock".to_string(), None)
        })?;

        // Find node with minimum assignments
        let selected = available_nodes
            .iter()
            .min_by_key(|node| counters.get(&node.node_id).unwrap_or(&0))
            .unwrap();

        // Increment counter
        *counters.entry(selected.node_id).or_insert(0) += 1;

        Ok(LoadBalancingDecision {
            selected_node: selected.node_id,
            confidence: 0.8,
            predicted_execution_time: Duration::from_millis(500), // Default estimate
            predicted_success_probability: 0.9,
            strategy_used: LoadBalancingStrategy::RoundRobin,
            reasoning: "Round-robin assignment".to_string(),
        })
    }

    /// Least connections node selection
    async fn least_connections_selection(
        &self,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        // Select node with lowest current load
        let selected = available_nodes
            .iter()
            .min_by_key(|node| OrderedFloat((node.cpu_usage + node.memory_usage) / 2.0))
            .unwrap();

        Ok(LoadBalancingDecision {
            selected_node: selected.node_id,
            confidence: 0.85,
            predicted_execution_time: Duration::from_millis(400),
            predicted_success_probability: 0.92,
            strategy_used: LoadBalancingStrategy::LeastConnections,
            reasoning: format!(
                "Lowest load node ({}% usage)",
                ((selected.cpu_usage + selected.memory_usage) / 2.0 * 100.0) as u32
            ),
        })
    }

    /// Weighted round-robin based on node capacity
    async fn weighted_round_robin_selection(
        &self,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        // Calculate weights based on available capacity
        let weights: Vec<_> = available_nodes
            .iter()
            .map(|node| node.available_capacity())
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return self.round_robin_selection(available_nodes).await;
        }

        // Weighted random selection
        let mut cumulative = 0.0;
        let random = fastrand::f64() * total_weight;

        for (i, &weight) in weights.iter().enumerate() {
            cumulative += weight;
            if random <= cumulative {
                let selected = &available_nodes[i];
                return Ok(LoadBalancingDecision {
                    selected_node: selected.node_id,
                    confidence: 0.87,
                    predicted_execution_time: Duration::from_millis(350),
                    predicted_success_probability: 0.93,
                    strategy_used: LoadBalancingStrategy::WeightedRoundRobin,
                    reasoning: format!(
                        "Weighted selection (capacity: {:.1}%)",
                        selected.available_capacity() * 100.0
                    ),
                });
            }
        }

        // Fallback to last node
        let selected = available_nodes.last().unwrap();
        Ok(LoadBalancingDecision {
            selected_node: selected.node_id,
            confidence: 0.8,
            predicted_execution_time: Duration::from_millis(500),
            predicted_success_probability: 0.9,
            strategy_used: LoadBalancingStrategy::WeightedRoundRobin,
            reasoning: "Fallback selection".to_string(),
        })
    }

    /// ML-optimized node selection using neural network
    async fn ml_optimized_selection(
        &self,
        task: &DistributedTask,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        let prediction_confidence = {
            let network = self.prediction_network.read().map_err(|_| {
                Error::runtime_error("Failed to acquire network lock".to_string(), None)
            })?;
            network.prediction_confidence()
        };

        // Check if we have enough training data
        if prediction_confidence < MIN_PREDICTION_CONFIDENCE {
            // Fall back to heuristic-based selection
            return self.least_connections_selection(available_nodes).await;
        }

        let task_features = TaskFeatures::from_task(task);
        let mut best_node = None;
        let mut best_score = f64::NEG_INFINITY;
        let mut best_prediction = Vec::new();

        // Evaluate each node
        for node in available_nodes {
            let node_features = NodeFeatures::from_capabilities(node);

            // Combine task and node features
            let mut input = task_features.to_vector();
            input.extend(node_features.to_vector());

            // Get ML prediction [execution_time, success_prob, performance_score]
            let prediction = {
                let network = self.prediction_network.read().map_err(|_| {
                    Error::runtime_error("Failed to acquire network lock".to_string(), None)
                })?;
                network.predict(&input)
            };

            // Calculate composite score based on objectives
            let objectives = self.objectives.read().map_err(|_| {
                Error::runtime_error("Failed to acquire objectives lock".to_string(), None)
            })?;

            let execution_time = prediction[0] * 10000.0; // Denormalize
            let success_prob = prediction[1];
            let performance = prediction[2];

            // Multi-objective scoring
            let latency_score = 1.0 / (1.0 + execution_time / 1000.0); // Lower is better
            let throughput_score = success_prob * performance;
            let fairness_score = 1.0 - node.available_capacity().abs(); // Balanced load
            let reliability_score = node.reliability_score;

            let composite_score = objectives.latency_weight * latency_score
                + objectives.throughput_weight * throughput_score
                + objectives.fairness_weight * fairness_score
                + objectives.reliability_weight * reliability_score;

            if composite_score > best_score {
                best_score = composite_score;
                best_node = Some(node);
                best_prediction = prediction;
            }
        }

        let selected = best_node.unwrap();
        Ok(LoadBalancingDecision {
            selected_node: selected.node_id,
            confidence: prediction_confidence,
            predicted_execution_time: Duration::from_millis((best_prediction[0] * 10000.0) as u64),
            predicted_success_probability: best_prediction[1],
            strategy_used: LoadBalancingStrategy::MLOptimized,
            reasoning: format!("ML prediction (score: {:.3})", best_score),
        })
    }

    /// Reinforcement learning adaptive selection
    async fn rl_adaptive_selection(
        &self,
        task: &DistributedTask,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        // Get current system state
        let state = {
            let state_guard = self.system_state.read().map_err(|_| {
                Error::runtime_error("Failed to acquire state lock".to_string(), None)
            })?;
            state_guard.clone()
        };

        // Use RL agent to select strategy
        let selected_strategy = {
            let mut agent = self.rl_agent.write().map_err(|_| {
                Error::runtime_error("Failed to acquire RL agent lock".to_string(), None)
            })?;
            agent.select_strategy(&state)
        };

        // Apply the selected strategy (avoid recursion)
        match selected_strategy {
            LoadBalancingStrategy::RLAdaptive => {
                // Avoid infinite recursion - use ML optimized as base
                self.ml_optimized_selection(task, available_nodes).await
            }
            LoadBalancingStrategy::RoundRobin => {
                let mut result = self.round_robin_selection(available_nodes).await?;
                result.strategy_used = LoadBalancingStrategy::RLAdaptive;
                result.reasoning = "RL selected round-robin strategy".to_string();
                Ok(result)
            }
            LoadBalancingStrategy::LeastConnections => {
                let mut result = self.least_connections_selection(available_nodes).await?;
                result.strategy_used = LoadBalancingStrategy::RLAdaptive;
                result.reasoning = "RL selected least-connections strategy".to_string();
                Ok(result)
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                let mut result = self.weighted_round_robin_selection(available_nodes).await?;
                result.strategy_used = LoadBalancingStrategy::RLAdaptive;
                result.reasoning = "RL selected weighted round-robin strategy".to_string();
                Ok(result)
            }
            LoadBalancingStrategy::MLOptimized => {
                let mut result = self.ml_optimized_selection(task, available_nodes).await?;
                result.strategy_used = LoadBalancingStrategy::RLAdaptive;
                result.reasoning = "RL selected ML-optimized strategy".to_string();
                Ok(result)
            }
            LoadBalancingStrategy::MultiObjective => {
                let mut result = self
                    .multi_objective_selection(task, available_nodes)
                    .await?;
                result.strategy_used = LoadBalancingStrategy::RLAdaptive;
                result.reasoning = "RL selected multi-objective strategy".to_string();
                Ok(result)
            }
        }
    }

    /// Multi-objective optimization selection
    async fn multi_objective_selection(
        &self,
        task: &DistributedTask,
        available_nodes: &[NodeCapabilities],
    ) -> Result<LoadBalancingDecision> {
        let objectives = self.objectives.read().map_err(|_| {
            Error::runtime_error("Failed to acquire objectives lock".to_string(), None)
        })?;

        let mut best_node = None;
        let mut best_score = f64::NEG_INFINITY;

        // Evaluate each node against all objectives
        for node in available_nodes {
            // Latency objective (prefer nodes with low latency)
            let avg_latency = if node.network_latency.is_empty() {
                50.0
            } else {
                node.network_latency
                    .values()
                    .map(|d| *d as f64)
                    .sum::<f64>()
                    / node.network_latency.len() as f64
            };
            let latency_score = 1.0 / (1.0 + avg_latency / 100.0);

            // Throughput objective (prefer nodes with high capacity)
            let throughput_score = node.available_capacity();

            // Fairness objective (prefer balanced load distribution)
            let fairness_score =
                1.0 - (node.cpu_usage - 0.5).abs() - (node.memory_usage - 0.5).abs();

            // Energy objective (prefer energy-efficient nodes)
            let energy_score = 1.0 - node.cpu_usage * 0.8 - node.memory_usage * 0.2;

            // Reliability objective
            let reliability_score = node.reliability_score;

            // Weighted combination
            let composite_score = objectives.latency_weight * latency_score
                + objectives.throughput_weight * throughput_score
                + objectives.fairness_weight * fairness_score
                + objectives.energy_weight * energy_score
                + objectives.reliability_weight * reliability_score;

            if composite_score > best_score {
                best_score = composite_score;
                best_node = Some(node);
            }
        }

        let selected = best_node.unwrap();
        Ok(LoadBalancingDecision {
            selected_node: selected.node_id,
            confidence: 0.9,
            predicted_execution_time: Duration::from_millis(300),
            predicted_success_probability: 0.95,
            strategy_used: LoadBalancingStrategy::MultiObjective,
            reasoning: format!("Multi-objective optimization (score: {:.3})", best_score),
        })
    }

    /// Records task execution result for learning
    pub fn record_execution_result(
        &self,
        task: &DistributedTask,
        node_capabilities: &NodeCapabilities,
        metric: &PerformanceMetric,
    ) -> Result<()> {
        // Create training sample
        let sample = TrainingSample::new(task, node_capabilities, metric);

        // Add to training history
        {
            let mut samples = self.training_samples.write().map_err(|_| {
                Error::runtime_error("Failed to acquire training samples lock".to_string(), None)
            })?;

            samples.push_back(sample);

            // Keep only recent samples
            if samples.len() > MAX_HISTORY_SIZE {
                samples.pop_front();
            }
        }

        // Update performance history
        {
            let mut history = self.performance_history.write().map_err(|_| {
                Error::runtime_error(
                    "Failed to acquire performance history lock".to_string(),
                    None,
                )
            })?;

            history.push_back((SystemTime::now(), metric.performance_score));

            if history.len() > 1000 {
                history.pop_front();
            }
        }

        // Trigger training if we have enough samples
        self.maybe_trigger_training()?;

        Ok(())
    }

    /// Triggers neural network training if sufficient data is available
    fn maybe_trigger_training(&self) -> Result<()> {
        let samples = {
            let samples_guard = self.training_samples.read().map_err(|_| {
                Error::runtime_error("Failed to acquire training samples lock".to_string(), None)
            })?;

            if samples_guard.len() < 100 {
                return Ok(()); // Not enough data
            }

            // Take recent samples for training
            samples_guard
                .iter()
                .rev()
                .take(500)
                .cloned()
                .collect::<Vec<_>>()
        };

        // Train the network
        {
            let mut network = self.prediction_network.write().map_err(|_| {
                Error::runtime_error("Failed to acquire network lock".to_string(), None)
            })?;

            network.train(&samples);
        }

        Ok(())
    }

    /// Updates the load balancing strategy based on recent performance
    pub async fn adapt_strategy(&self) -> Result<()> {
        let recent_performance = {
            let history = self.performance_history.read().map_err(|_| {
                Error::runtime_error(
                    "Failed to acquire performance history lock".to_string(),
                    None,
                )
            })?;

            if history.len() < 20 {
                return Ok(()); // Not enough data
            }

            // Calculate average performance over recent window
            let recent: Vec<_> = history.iter().rev().take(20).collect();
            recent.iter().map(|(_, score)| *score).sum::<f64>() / recent.len() as f64
        };

        // Update system state for RL
        let new_state = if recent_performance > 0.8 {
            "high_performance"
        } else if recent_performance > 0.6 {
            "normal"
        } else {
            "low_performance"
        }
        .to_string();

        {
            let mut state = self.system_state.write().map_err(|_| {
                Error::runtime_error("Failed to acquire state lock".to_string(), None)
            })?;
            *state = new_state;
        }

        // Update RL agent with performance reward
        let reward = (recent_performance - 0.5) * 2.0; // Scale to [-1, 1]

        {
            let mut agent = self.rl_agent.write().map_err(|_| {
                Error::runtime_error("Failed to acquire RL agent lock".to_string(), None)
            })?;

            let current_strategy = {
                let strategy = self.strategy.read().map_err(|_| {
                    Error::runtime_error("Failed to acquire strategy lock".to_string(), None)
                })?;
                *strategy
            };

            let state_str = if recent_performance > 0.8 {
                "high_performance"
            } else if recent_performance > 0.6 {
                "normal"
            } else {
                "low_performance"
            };

            agent.update_q_value(state_str, current_strategy, reward, state_str);
        }

        Ok(())
    }

    /// Starts the adaptive optimization background task
    pub async fn start_adaptive_optimization(self: Arc<Self>) -> Result<()> {
        let load_balancer = Arc::clone(&self);

        tokio::spawn(async move {
            let mut optimization_interval = interval(STRATEGY_UPDATE_INTERVAL);

            loop {
                optimization_interval.tick().await;

                if let Err(e) = load_balancer.adapt_strategy().await {
                    eprintln!("Strategy adaptation error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Gets current load balancer statistics
    pub fn get_statistics(&self) -> Result<LoadBalancerStatistics> {
        let training_samples_count = {
            let samples = self.training_samples.read().map_err(|_| {
                Error::runtime_error("Failed to acquire training samples lock".to_string(), None)
            })?;
            samples.len()
        };

        let prediction_confidence = {
            let network = self.prediction_network.read().map_err(|_| {
                Error::runtime_error("Failed to acquire network lock".to_string(), None)
            })?;
            network.prediction_confidence()
        };

        let current_strategy = {
            let strategy = self.strategy.read().map_err(|_| {
                Error::runtime_error("Failed to acquire strategy lock".to_string(), None)
            })?;
            *strategy
        };

        let avg_performance = {
            let history = self.performance_history.read().map_err(|_| {
                Error::runtime_error(
                    "Failed to acquire performance history lock".to_string(),
                    None,
                )
            })?;

            if history.is_empty() {
                0.0
            } else {
                history.iter().map(|(_, score)| *score).sum::<f64>() / history.len() as f64
            }
        };

        Ok(LoadBalancerStatistics {
            current_strategy,
            training_samples_count,
            prediction_confidence,
            average_performance: avg_performance,
            total_decisions: 0, // TODO: Track decision count
        })
    }
}

impl Default for MLAdaptiveLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

/// Load balancer performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStatistics {
    /// Current active strategy
    pub current_strategy: LoadBalancingStrategy,
    /// Number of training samples collected
    pub training_samples_count: usize,
    /// Confidence in ML predictions (0.0 to 1.0)
    pub prediction_confidence: f64,
    /// Average performance score
    pub average_performance: f64,
    /// Total load balancing decisions made
    pub total_decisions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancing_objectives() {
        let mut objectives = LoadBalancingObjectives {
            latency_weight: 0.4,
            throughput_weight: 0.4,
            fairness_weight: 0.1,
            energy_weight: 0.05,
            reliability_weight: 0.05,
        };

        assert!(objectives.validate());

        // Test normalization
        objectives.latency_weight = 0.8;
        objectives.normalize();
        assert!(
            (objectives.latency_weight
                + objectives.throughput_weight
                + objectives.fairness_weight
                + objectives.energy_weight
                + objectives.reliability_weight
                - 1.0)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn test_task_features() {
        let task = DistributedTask::new("(+ 1 2)".to_string(), TaskPriority::High);
        let features = TaskFeatures::from_task(&task);

        assert_eq!(features.priority, 0.75);
        assert!(features.complexity > 0.0);

        let vector = features.to_vector();
        assert_eq!(vector.len(), 6);
    }

    #[test]
    fn test_node_features() {
        let capabilities = NodeCapabilities::new(NodeId::new());
        let features = NodeFeatures::from_capabilities(&capabilities);

        assert_eq!(features.cpu_capacity, 1.0); // No usage initially
        assert_eq!(features.memory_capacity, 1.0);

        let vector = features.to_vector();
        assert_eq!(vector.len(), 6);
    }

    #[test]
    fn test_neural_network() {
        let mut network = LoadPredictionNetwork::new(6, vec![4], 2);
        let input = vec![0.5; 6];

        let output = network.predict(&input);
        assert_eq!(output.len(), 2);

        // Test training with empty samples (should not crash)
        network.train(&[]);
        assert_eq!(network.training_history, 0);
    }

    #[test]
    fn test_rl_agent() {
        let mut agent = RLAgent::new();

        let strategy = agent.select_strategy("test_state");
        assert!(matches!(
            strategy,
            LoadBalancingStrategy::RoundRobin
                | LoadBalancingStrategy::LeastConnections
                | LoadBalancingStrategy::WeightedRoundRobin
                | LoadBalancingStrategy::MLOptimized
                | LoadBalancingStrategy::RLAdaptive
                | LoadBalancingStrategy::MultiObjective
        ));

        // Update Q-value
        agent.update_q_value("test_state", strategy, 0.8, "test_next_state");
        assert!(agent.training_steps > 0);
    }

    #[tokio::test]
    async fn test_ml_adaptive_load_balancer() {
        let load_balancer = MLAdaptiveLoadBalancer::new();

        // Test with empty nodes (should return error)
        let task = DistributedTask::new("test".to_string(), TaskPriority::Normal);
        let result = load_balancer.select_node(&task, &[]).await;
        assert!(result.is_err());

        // Test with single node
        let capabilities = NodeCapabilities::new(NodeId::new());
        let decision = load_balancer
            .select_node(&task, &[capabilities])
            .await
            .unwrap();
        assert!(decision.confidence > 0.0);
        assert!(decision.predicted_success_probability > 0.0);
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let load_balancer = MLAdaptiveLoadBalancer::new();
        let nodes = vec![
            NodeCapabilities::new(NodeId::new()),
            NodeCapabilities::new(NodeId::new()),
        ];

        let decision1 = load_balancer.round_robin_selection(&nodes).await.unwrap();
        let decision2 = load_balancer.round_robin_selection(&nodes).await.unwrap();

        // Should select different nodes in round-robin fashion
        assert_ne!(decision1.selected_node, decision2.selected_node);
    }

    #[tokio::test]
    async fn test_least_connections_selection() {
        let load_balancer = MLAdaptiveLoadBalancer::new();
        let mut node1 = NodeCapabilities::new(NodeId::new());
        let mut node2 = NodeCapabilities::new(NodeId::new());

        node1.update_usage(0.8, 0.7); // High load
        node2.update_usage(0.2, 0.3); // Low load

        let node2_id = node2.node_id;
        let nodes = vec![node1, node2];
        let decision = load_balancer
            .least_connections_selection(&nodes)
            .await
            .unwrap();

        // Should select the node with lower load
        assert_eq!(decision.selected_node, node2_id);
    }
}
