//! Distributed System Configuration Types
//!
//! This module provides the canonical configuration types for all distributed systems
//! to avoid type conflicts and ensure consistency across modules.

#[cfg(feature = "async-runtime")]
use crate::concurrency::actors::SupervisionStrategy;
#[cfg(feature = "async-runtime")]
use crate::concurrency::distributed::NodeId;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Hybrid JIT configuration (canonical definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridJitConfig {
    /// Enable LLVM backend for JIT compilation. Provides extensive optimizations but slower compile times.
    pub enable_llvm: bool,
    /// Enable Cranelift backend for JIT compilation. Faster compilation with good optimization.
    pub enable_cranelift: bool,
    /// JIT optimization level (0-3). Higher values provide better runtime performance.
    pub optimization_level: u8,
    /// Minimum invocation count before JIT compilation is triggered.
    pub jit_threshold: u32,
    /// Enable parallel JIT compilation for improved throughput on multi-core systems.
    pub parallel_compilation: bool,
    /// Size of the compiled code cache in bytes.
    pub cache_size: usize,
}

impl Default for HybridJitConfig {
    fn default() -> Self {
        HybridJitConfig {
            enable_llvm: false,
            enable_cranelift: true,
            optimization_level: 2,
            jit_threshold: 100,
            parallel_compilation: true,
            cache_size: 1024 * 1024, // 1MB
        }
    }
}

/// Distributed continuation configuration (canonical definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedContinuationConfig {
    /// Enable distributed execution of continuations across multiple nodes.
    pub enable_distributed_execution: bool,
    /// Maximum time to wait for continuation execution before timeout.
    pub continuation_timeout: Duration,
    /// Maximum number of hops a continuation can make across network nodes.
    pub max_hops: u32,
    /// Configuration for continuation serialization during network transfer.
    pub serialization_config: SerializationConfig,
    /// Configuration for distributed execution strategies.
    pub execution_config: ExecutionConfig,
    /// Configuration for distributed system tracing and monitoring.
    pub tracing_config: TracingConfig,
}

impl Default for DistributedContinuationConfig {
    fn default() -> Self {
        DistributedContinuationConfig {
            enable_distributed_execution: true,
            continuation_timeout: Duration::from_secs(30),
            max_hops: 10,
            serialization_config: SerializationConfig::default(),
            execution_config: ExecutionConfig::default(),
            tracing_config: TracingConfig::default(),
        }
    }
}

/// Fault tolerance configuration (canonical definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultToleranceConfig {
    /// Maximum number of restart attempts before giving up
    pub max_restart_attempts: u32,
    /// Time to wait between restart attempts
    pub restart_backoff: Duration,
    #[cfg(feature = "async-runtime")]
    /// Strategy for supervising failed processes
    pub supervision_strategy: SupervisionStrategy,
    /// Whether to enable automatic recovery from failures
    pub enable_automatic_recovery: bool,
    /// Maximum number of retry attempts for operations
    pub max_retry_attempts: u32,
    /// Timeout for recovery operations
    pub recovery_timeout: Duration,
    /// Configuration for supervision policies
    pub supervision_config: SupervisionConfig,
    /// Configuration for failure detection
    pub detection_config: DetectionConfig,
    /// Configuration for recovery procedures
    pub recovery_config: RecoveryConfig,
    /// Configuration for health monitoring
    pub health_config: HealthConfig,
}

impl Default for FaultToleranceConfig {
    fn default() -> Self {
        FaultToleranceConfig {
            max_restart_attempts: 3,
            restart_backoff: Duration::from_secs(1),
            #[cfg(feature = "async-runtime")]
            supervision_strategy: SupervisionStrategy::Restart,
            enable_automatic_recovery: true,
            max_retry_attempts: 5,
            recovery_timeout: Duration::from_secs(10),
            supervision_config: SupervisionConfig::default(),
            detection_config: DetectionConfig::default(),
            recovery_config: RecoveryConfig::default(),
            health_config: HealthConfig::default(),
        }
    }
}

/// Load balancing configuration (canonical definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy (e.g., "round_robin", "least_connections", "continuation_affinity")
    pub strategy: String,
    /// Resource utilization threshold that triggers rebalancing (0.0-1.0)
    pub rebalance_threshold: f64,
    /// Time interval between load rebalancing evaluations
    pub rebalance_interval: Duration,
    /// Enable continuation-aware load balancing to maintain state locality
    pub enable_continuation_awareness: bool,
    /// Interval for performing health checks on cluster nodes
    pub health_check_interval: Duration,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        LoadBalancingConfig {
            strategy: "least_connections".to_string(),
            rebalance_threshold: 0.8,
            rebalance_interval: Duration::from_secs(60),
            enable_continuation_awareness: true,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Cluster registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRegistryConfig {
    /// Time interval for sending heartbeat messages to maintain node liveness
    pub heartbeat_interval: Duration,
    /// Maximum time to wait for node response before marking it as unreachable
    pub node_timeout: Duration,
    /// Interval for synchronizing registry state across cluster nodes
    pub registry_sync_interval: Duration,
}

impl Default for ClusterRegistryConfig {
    fn default() -> Self {
        ClusterRegistryConfig {
            heartbeat_interval: Duration::from_secs(5),
            node_timeout: Duration::from_secs(30),
            registry_sync_interval: Duration::from_secs(10),
        }
    }
}

/// Serialization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationConfig {
    /// Enable compression for continuation serialization to reduce network overhead
    pub enable_compression: bool,
    /// Compression level (1-9) with higher values providing better compression
    pub compression_level: u8,
    /// Maximum allowed size for serialized continuation data in bytes
    pub max_serialization_size: usize,
    /// Enable checksum validation for data integrity verification
    pub enable_checksum: bool,
}

impl Default for SerializationConfig {
    fn default() -> Self {
        SerializationConfig {
            enable_compression: true,
            compression_level: 6,
            max_serialization_size: 1024 * 1024, // 1MB
            enable_checksum: true,
        }
    }
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Enable parallel execution of continuations across multiple threads
    pub enable_parallel_execution: bool,
    /// Maximum number of continuations that can execute concurrently
    pub max_parallel_continuations: u32,
    /// Maximum time allowed for continuation execution before timeout
    pub execution_timeout: Duration,
    /// Number of continuations to batch together for efficient execution
    pub batch_size: u32,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            enable_parallel_execution: true,
            max_parallel_continuations: 10,
            execution_timeout: Duration::from_secs(30),
            batch_size: 5,
        }
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing for continuation execution across nodes
    pub enable_continuation_tracing: bool,
    /// Size of the trace buffer for storing execution traces
    pub trace_buffer_size: usize,
    /// Tracing verbosity level ("error", "warn", "info", "debug", "trace")
    pub trace_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        TracingConfig {
            enable_continuation_tracing: true,
            trace_buffer_size: 10000,
            trace_level: "info".to_string(),
        }
    }
}

/// Supervision configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionConfig {
    /// Maximum depth of supervisor hierarchy for fault isolation
    pub max_supervision_levels: u32,
    /// Maximum time a supervisor waits for child process response
    pub supervisor_timeout: Duration,
    /// Number of failures before escalating to higher supervisor level
    pub escalation_threshold: u32,
}

impl Default for SupervisionConfig {
    fn default() -> Self {
        SupervisionConfig {
            max_supervision_levels: 5,
            supervisor_timeout: Duration::from_secs(30),
            escalation_threshold: 3,
        }
    }
}

/// Detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Interval between health check probes to detect node failures
    pub health_check_interval: Duration,
    /// Maximum time to wait for health check response before considering failure
    pub failure_detection_timeout: Duration,
    /// Number of consecutive failures required to mark a node as failed
    pub failure_threshold: u32,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        DetectionConfig {
            health_check_interval: Duration::from_secs(5),
            failure_detection_timeout: Duration::from_secs(15),
            failure_threshold: 3,
        }
    }
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Enable automatic recovery procedures when failures are detected
    pub enable_automatic_recovery: bool,
    /// Maximum time allowed for recovery operations to complete
    pub recovery_timeout: Duration,
    /// Maximum number of recovery attempts before giving up
    pub max_recovery_attempts: u32,
    /// Exponential backoff delay between recovery attempts
    pub recovery_backoff: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        RecoveryConfig {
            enable_automatic_recovery: true,
            recovery_timeout: Duration::from_secs(30),
            max_recovery_attempts: 5,
            recovery_backoff: Duration::from_millis(500),
        }
    }
}

/// Health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Interval for performing health status evaluations
    pub health_check_interval: Duration,
    /// Health score threshold (0.0-1.0) below which nodes are considered unhealthy
    pub health_threshold: f64,
    /// Enable automatic healing procedures for degraded nodes
    pub enable_auto_healing: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        HealthConfig {
            health_check_interval: Duration::from_secs(10),
            health_threshold: 0.8,
            enable_auto_healing: true,
        }
    }
}

/// Resource utilization (now with serialization support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization as percentage (0.0-100.0)
    pub cpu_percent: f64,
    /// Memory utilization as percentage (0.0-100.0)
    pub memory_percent: f64,
    /// Disk utilization as percentage (0.0-100.0)
    pub disk_percent: f64,
    /// Network utilization as percentage (0.0-100.0)
    pub network_percent: f64,
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        ResourceUtilization {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            disk_percent: 0.0,
            network_percent: 0.0,
        }
    }
}

/// Distributed actor configuration (consolidated)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DistributedActorConfig {
    /// Unique identifier for this node in the distributed cluster
    #[cfg(feature = "async-runtime")]
    #[serde(default = "NodeId::new")]
    pub node_id: NodeId,
    /// Configuration for hybrid JIT compilation system
    pub jit_config: HybridJitConfig,
    /// Configuration for distributed continuation execution
    pub continuation_config: DistributedContinuationConfig,
    /// Configuration for cluster node registry and discovery
    pub registry_config: ClusterRegistryConfig,
    /// Configuration for load balancing across cluster nodes
    pub load_balancer_config: LoadBalancingConfig,
    /// Configuration for fault tolerance and recovery mechanisms
    pub fault_tolerance_config: FaultToleranceConfig,
}
