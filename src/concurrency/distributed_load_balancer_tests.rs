//! Phase 3.3 Advanced Load Balancing System - Comprehensive Test Suite
//!
//! Test suite validating:
//! - 80% ML prediction accuracy requirement
//! - 90% dynamic rebalancing efficiency requirement
//! - 5ms real-time decision making requirement
//! - Full JIT optimization integration
//! - Complete system integration with existing distributed systems

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrency::distributed::NodeId;
    use crate::concurrency::distributed_continuation_system::ResourceRequirements;
    use crate::concurrency::distributed_load_balancer::{
        ClusterState, LoadBalancingConfig, NodeInfo,
    };
    use crate::concurrency::distributed_load_balancer_phase3::*;
    use crate::diagnostics::Error;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant, SystemTime};
    use tokio::time::timeout;

    /// Test fixture for Phase 3.3 load balancing system
    struct Phase3TestFixture {
        advanced_load_balancer: AdvancedContinuationAwareLoadBalancer,
        test_cluster: TestCluster,
        test_patterns: Vec<ContinuationExecutionPattern>,
    }

    impl Phase3TestFixture {
        async fn new() -> Self {
            let test_cluster = TestCluster::new(8).await; // 8-node test cluster
            let config = AdvancedLoadBalancingConfig {
                ml_config: MlPredictionConfig {
                    model_type: "neural_network".to_string(),
                    prediction_accuracy_target: 0.80,
                    training_data_size: 10000,
                    feature_dimension: 7,
                },
                decision_config: RealTimeDecisionConfig {
                    decision_timeout: Duration::from_millis(5),
                    cache_size: 1000,
                    optimization_level: 3,
                },
                rebalancing_config: DynamicRebalancingConfig {
                    efficiency_target: 0.90,
                    max_disruption_time: Duration::from_secs(2),
                    rebalancing_trigger_threshold: 0.3,
                },
                jit_config: JitOptimizationConfig {
                    enable_aggressive_optimization: true,
                    compilation_timeout: Duration::from_millis(500),
                    optimization_level: 3,
                },
            };

            let core_balancer = Arc::new(test_cluster.create_core_load_balancer());
            let advanced_load_balancer =
                AdvancedContinuationAwareLoadBalancer::new(core_balancer, config).unwrap();

            let test_patterns = Self::generate_test_patterns();

            Phase3TestFixture {
                advanced_load_balancer,
                test_cluster,
                test_patterns,
            }
        }

        fn generate_test_patterns() -> Vec<ContinuationExecutionPattern> {
            vec![
                // CPU-intensive pattern
                ContinuationExecutionPattern {
                    signature: "cpu_intensive_computation".to_string(),
                    execution_segments: vec![ExecutionSegment {
                        id: "compute_heavy".to_string(),
                        jit_compilable: true,
                        hotness_score: 0.9,
                        expected_jit_speedup: 2.5,
                        jit_compilation_time: Duration::from_millis(200),
                        jit_memory_usage: 512,
                        memory_usage: 1024,
                        allocation_pattern: MemoryAllocationPattern::Sequential,
                        cpu_intensity: 0.95,
                        parallelization_score: 0.8,
                        data_dependencies: vec!["input_data".to_string()],
                        sync_overhead: 0.1,
                    }],
                    estimated_memory_usage: 2048,
                    estimated_execution_time: Duration::from_millis(500),
                    confidence_score: 0.85,
                    historical_performance: HashMap::new(),
                },
                // Memory-intensive pattern
                ContinuationExecutionPattern {
                    signature: "memory_intensive_processing".to_string(),
                    execution_segments: vec![ExecutionSegment {
                        id: "large_data_processing".to_string(),
                        jit_compilable: false,
                        hotness_score: 0.3,
                        expected_jit_speedup: 1.0,
                        jit_compilation_time: Duration::ZERO,
                        jit_memory_usage: 0,
                        memory_usage: 16384, // 16GB
                        allocation_pattern: MemoryAllocationPattern::Random,
                        cpu_intensity: 0.4,
                        parallelization_score: 0.2,
                        data_dependencies: vec![],
                        sync_overhead: 0.05,
                    }],
                    estimated_memory_usage: 20480, // 20GB
                    estimated_execution_time: Duration::from_millis(2000),
                    confidence_score: 0.75,
                    historical_performance: HashMap::new(),
                },
                // JIT-optimizable pattern
                ContinuationExecutionPattern {
                    signature: "jit_optimal_computation".to_string(),
                    execution_segments: vec![ExecutionSegment {
                        id: "hot_loop".to_string(),
                        jit_compilable: true,
                        hotness_score: 0.95,
                        expected_jit_speedup: 4.2,
                        jit_compilation_time: Duration::from_millis(150),
                        jit_memory_usage: 256,
                        memory_usage: 512,
                        allocation_pattern: MemoryAllocationPattern::Batched { batch_size: 64 },
                        cpu_intensity: 0.85,
                        parallelization_score: 0.9,
                        data_dependencies: vec![],
                        sync_overhead: 0.02,
                    }],
                    estimated_memory_usage: 1024,
                    estimated_execution_time: Duration::from_millis(300),
                    confidence_score: 0.95,
                    historical_performance: HashMap::new(),
                },
            ]
        }
    }

    /// Test 1: 5ms Decision Time Requirement
    #[tokio::test]
    async fn test_5ms_decision_time_requirement() {
        let fixture = Phase3TestFixture::new().await;
        let mut decision_times = Vec::new();
        let test_iterations = 1000;

        println!(
            "Testing 5ms decision time requirement with {} iterations",
            test_iterations
        );

        for i in 0..test_iterations {
            let pattern = &fixture.test_patterns[i % fixture.test_patterns.len()];
            let resource_requirements = ResourceRequirements {
                memory_mb: pattern.estimated_memory_usage / (1024 * 1024), // Convert to MB
                cpu_cores: 2,
                network_bandwidth_mbps: 100,
                storage_gb: 10,
            };

            let start_time = Instant::now();

            let decision = fixture
                .advanced_load_balancer
                .make_intelligent_placement_decision(
                    pattern,
                    &resource_requirements,
                    &fixture.test_cluster.get_cluster_state(),
                )
                .await
                .unwrap();

            let decision_time = start_time.elapsed();
            decision_times.push(decision_time);

            // Verify decision validity
            assert!(decision.confidence > 0.0 && decision.confidence <= 1.0);
            assert!(!decision.alternative_nodes.is_empty());
            assert!(decision.expected_performance_improvement >= 0.0);
        }

        // Analyze results
        let average_time = decision_times.iter().sum::<Duration>() / decision_times.len() as u32;
        let max_time = decision_times.iter().max().unwrap();
        let times_within_target = decision_times
            .iter()
            .filter(|&&t| t <= Duration::from_millis(5))
            .count();
        let success_rate = (times_within_target as f64 / test_iterations as f64) * 100.0;

        println!("5ms Decision Time Test Results:");
        println!("  Average decision time: {:?}", average_time);
        println!("  Maximum decision time: {:?}", max_time);
        println!("  Success rate (≤5ms): {:.2}%", success_rate);
        println!(
            "  Decisions within target: {}/{}",
            times_within_target, test_iterations
        );

        // Assert requirements
        assert!(
            success_rate >= 95.0,
            "Only {:.1}% of decisions met 5ms target, required ≥95%",
            success_rate
        );
        assert!(
            average_time <= Duration::from_millis(3),
            "Average time {:?} exceeds 3ms target",
            average_time
        );
        assert!(
            *max_time <= Duration::from_millis(10),
            "Maximum time {:?} exceeds 10ms absolute limit",
            max_time
        );
    }

    /// Test 2: 80% ML Prediction Accuracy Requirement
    #[tokio::test]
    async fn test_80_percent_ml_prediction_accuracy() {
        let fixture = Phase3TestFixture::new().await;
        let mut prediction_results = Vec::new();
        let test_scenarios = 500;

        println!(
            "Testing 80% ML prediction accuracy with {} scenarios",
            test_scenarios
        );

        for scenario in 0..test_scenarios {
            let pattern = &fixture.test_patterns[scenario % fixture.test_patterns.len()];
            let cluster_state = fixture.test_cluster.get_cluster_state();

            // Get ML prediction
            let pattern_analysis = fixture
                .advanced_load_balancer
                .analyze_continuation_pattern_rapidly(pattern)
                .await
                .unwrap();

            let ml_prediction = fixture
                .advanced_load_balancer
                .ml_predictor
                .predict_optimal_placement(&pattern_analysis, &cluster_state)
                .await
                .unwrap();

            // Determine actual optimal node through exhaustive testing
            let actual_optimal = fixture
                .test_cluster
                .determine_actual_optimal_node(pattern)
                .await
                .unwrap();

            // Find predicted optimal node
            let predicted_optimal = ml_prediction
                .node_scores
                .iter()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(node_id, _)| *node_id)
                .unwrap();

            let is_correct = predicted_optimal == actual_optimal;
            prediction_results.push(PredictionResult {
                scenario,
                pattern_type: pattern.signature.clone(),
                predicted_node: predicted_optimal,
                actual_optimal_node: actual_optimal,
                is_correct,
                confidence: ml_prediction.confidence,
            });
        }

        // Analyze results
        let correct_predictions = prediction_results.iter().filter(|r| r.is_correct).count();
        let accuracy = (correct_predictions as f64 / test_scenarios as f64) * 100.0;
        let average_confidence =
            prediction_results.iter().map(|r| r.confidence).sum::<f64>() / test_scenarios as f64;

        // Break down accuracy by pattern type
        let mut accuracy_by_pattern = HashMap::new();
        for pattern in &fixture.test_patterns {
            let pattern_results: Vec<_> = prediction_results
                .iter()
                .filter(|r| r.pattern_type == pattern.signature)
                .collect();
            let pattern_correct = pattern_results.iter().filter(|r| r.is_correct).count();
            let pattern_accuracy = (pattern_correct as f64 / pattern_results.len() as f64) * 100.0;
            accuracy_by_pattern.insert(pattern.signature.clone(), pattern_accuracy);
        }

        println!("80% ML Prediction Accuracy Test Results:");
        println!("  Overall accuracy: {:.2}%", accuracy);
        println!(
            "  Correct predictions: {}/{}",
            correct_predictions, test_scenarios
        );
        println!("  Average confidence: {:.3}", average_confidence);
        println!("  Accuracy by pattern type:");
        for (pattern_type, accuracy) in &accuracy_by_pattern {
            println!("    {}: {:.2}%", pattern_type, accuracy);
        }

        // Assert requirements
        assert!(
            accuracy >= 80.0,
            "ML prediction accuracy {:.1}% below 80% requirement",
            accuracy
        );
        assert!(
            average_confidence >= 0.7,
            "Average confidence {:.3} below 0.7 threshold",
            average_confidence
        );

        // Ensure each pattern type meets minimum accuracy
        for (pattern_type, pattern_accuracy) in accuracy_by_pattern {
            assert!(
                pattern_accuracy >= 70.0,
                "Pattern {} accuracy {:.1}% below 70% minimum",
                pattern_type,
                pattern_accuracy
            );
        }
    }

    /// Test 3: 90% Dynamic Rebalancing Efficiency Requirement
    #[tokio::test]
    async fn test_90_percent_rebalancing_efficiency() {
        let fixture = Phase3TestFixture::new().await;
        let mut efficiency_results = Vec::new();
        let test_scenarios = 100;

        println!(
            "Testing 90% rebalancing efficiency with {} scenarios",
            test_scenarios
        );

        for scenario in 0..test_scenarios {
            // Create imbalanced cluster state
            let mut imbalanced_cluster = fixture.test_cluster.clone();
            imbalanced_cluster
                .create_load_imbalance(scenario as f64 / 100.0)
                .await;

            let cluster_state = imbalanced_cluster.get_cluster_state();
            let rebalancing_trigger = match scenario % 4 {
                0 => RebalancingTrigger::LoadImbalance { threshold: 0.3 },
                1 => RebalancingTrigger::ResourceExhaustion {
                    node_id: cluster_state.nodes.keys().next().copied().unwrap(),
                },
                2 => RebalancingTrigger::PerformanceDegradation {
                    degradation_pct: 25.0,
                },
                _ => RebalancingTrigger::ScheduledRebalancing,
            };

            let rebalancing_result = fixture
                .advanced_load_balancer
                .execute_dynamic_rebalancing(&cluster_state, rebalancing_trigger)
                .await
                .unwrap();

            efficiency_results.push(RebalancingEfficiencyResult {
                scenario,
                initial_imbalance: imbalanced_cluster.calculate_load_imbalance(),
                final_imbalance: imbalanced_cluster
                    .calculate_load_imbalance_after_rebalancing(&rebalancing_result),
                efficiency: rebalancing_result.efficiency,
                rebalancing_time: rebalancing_result.rebalancing_time,
                nodes_affected: rebalancing_result.nodes_rebalanced,
                service_availability: rebalancing_result
                    .rebalancing_execution
                    .disruption_metrics
                    .service_availability,
            });
        }

        // Analyze results
        let average_efficiency =
            efficiency_results.iter().map(|r| r.efficiency).sum::<f64>() / test_scenarios as f64;
        let min_efficiency = efficiency_results
            .iter()
            .map(|r| r.efficiency)
            .fold(1.0f64, |a, b| a.min(b));
        let scenarios_meeting_target = efficiency_results
            .iter()
            .filter(|r| r.efficiency >= 0.90)
            .count();
        let success_rate = (scenarios_meeting_target as f64 / test_scenarios as f64) * 100.0;
        let average_availability = efficiency_results
            .iter()
            .map(|r| r.service_availability)
            .sum::<f64>()
            / test_scenarios as f64;

        println!("90% Rebalancing Efficiency Test Results:");
        println!("  Average efficiency: {:.2}%", average_efficiency * 100.0);
        println!("  Minimum efficiency: {:.2}%", min_efficiency * 100.0);
        println!("  Success rate (≥90%): {:.2}%", success_rate);
        println!(
            "  Scenarios meeting target: {}/{}",
            scenarios_meeting_target, test_scenarios
        );
        println!(
            "  Average service availability: {:.3}%",
            average_availability * 100.0
        );

        // Assert requirements
        assert!(
            success_rate >= 85.0,
            "Only {:.1}% scenarios met 90% efficiency target",
            success_rate
        );
        assert!(
            average_efficiency >= 0.90,
            "Average efficiency {:.1}% below 90% requirement",
            average_efficiency * 100.0
        );
        assert!(
            min_efficiency >= 0.75,
            "Minimum efficiency {:.1}% below 75% acceptable threshold",
            min_efficiency * 100.0
        );
        assert!(
            average_availability >= 0.95,
            "Average availability {:.1}% below 95% requirement",
            average_availability * 100.0
        );
    }

    /// Test 4: JIT Integration Optimization
    #[tokio::test]
    async fn test_jit_integration_optimization() {
        let fixture = Phase3TestFixture::new().await;
        let mut jit_optimization_results = Vec::new();

        println!("Testing JIT integration optimization");

        for pattern in &fixture.test_patterns {
            let cluster_state = fixture.test_cluster.get_cluster_state();
            let pattern_analysis = fixture
                .advanced_load_balancer
                .analyze_continuation_pattern_rapidly(pattern)
                .await
                .unwrap();

            let ml_prediction = fixture
                .advanced_load_balancer
                .ml_predictor
                .predict_optimal_placement(&pattern_analysis, &cluster_state)
                .await
                .unwrap();

            let real_time_optimization = fixture
                .advanced_load_balancer
                .decision_engine
                .optimize_for_current_conditions(
                    &ml_prediction,
                    &cluster_state,
                    &ResourceRequirements {
                        memory_mb: pattern.estimated_memory_usage / (1024 * 1024),
                        cpu_cores: 2,
                        network_bandwidth_mbps: 100,
                        storage_gb: 10,
                    },
                )
                .await
                .unwrap();

            let jit_optimization_score = fixture
                .advanced_load_balancer
                .jit_optimizer
                .calculate_jit_optimization_potential(&pattern_analysis, &real_time_optimization)
                .await
                .unwrap();

            let jit_compatible_segments = pattern
                .execution_segments
                .iter()
                .filter(|s| s.jit_compilable)
                .count();

            jit_optimization_results.push(JitOptimizationResult {
                pattern_name: pattern.signature.clone(),
                jit_compatible_segments,
                total_segments: pattern.execution_segments.len(),
                expected_improvement: jit_optimization_score.expected_improvement,
                confidence: jit_optimization_score.confidence,
                compilation_time: jit_optimization_score.jit_compilation_time,
            });
        }

        // Analyze results
        let average_improvement = jit_optimization_results
            .iter()
            .map(|r| r.expected_improvement)
            .sum::<f64>()
            / jit_optimization_results.len() as f64;

        let average_confidence = jit_optimization_results
            .iter()
            .map(|r| r.confidence)
            .sum::<f64>()
            / jit_optimization_results.len() as f64;

        println!("JIT Integration Optimization Test Results:");
        println!(
            "  Average expected improvement: {:.2}%",
            average_improvement * 100.0
        );
        println!("  Average confidence: {:.3}", average_confidence);

        for result in &jit_optimization_results {
            println!(
                "  Pattern {}: {:.1}% improvement, {:.0}% JIT compatible",
                result.pattern_name,
                result.expected_improvement * 100.0,
                (result.jit_compatible_segments as f64 / result.total_segments as f64) * 100.0
            );
        }

        // Assert JIT integration effectiveness
        assert!(
            average_improvement >= 0.15,
            "Average JIT improvement {:.1}% below 15% threshold",
            average_improvement * 100.0
        );
        assert!(
            average_confidence >= 0.7,
            "Average JIT confidence {:.3} below 0.7 threshold",
            average_confidence
        );

        // Verify JIT-optimizable patterns get significant benefits
        let jit_optimizable_results: Vec<_> = jit_optimization_results
            .iter()
            .filter(|r| (r.jit_compatible_segments as f64 / r.total_segments as f64) > 0.5)
            .collect();

        if !jit_optimizable_results.is_empty() {
            let jit_average_improvement = jit_optimizable_results
                .iter()
                .map(|r| r.expected_improvement)
                .sum::<f64>()
                / jit_optimizable_results.len() as f64;

            assert!(
                jit_average_improvement >= 0.25,
                "JIT-optimizable patterns only show {:.1}% improvement, expected ≥25%",
                jit_average_improvement * 100.0
            );
        }
    }

    /// Test 5: System Integration and Performance
    #[tokio::test]
    async fn test_system_integration_performance() {
        let fixture = Phase3TestFixture::new().await;
        let integration_start = Instant::now();

        println!("Testing complete system integration");

        // Test comprehensive workflow
        let mut workflow_results = Vec::new();

        for (i, pattern) in fixture.test_patterns.iter().enumerate() {
            let workflow_start = Instant::now();

            // Step 1: Make intelligent placement decision
            let placement_decision = fixture
                .advanced_load_balancer
                .make_intelligent_placement_decision(
                    pattern,
                    &ResourceRequirements {
                        memory_mb: pattern.estimated_memory_usage / (1024 * 1024),
                        cpu_cores: 4,
                        network_bandwidth_mbps: 1000,
                        storage_gb: 50,
                    },
                    &fixture.test_cluster.get_cluster_state(),
                )
                .await
                .unwrap();

            // Step 2: Execute dynamic rebalancing if needed
            let rebalancing_result = if i % 3 == 0 {
                Some(
                    fixture
                        .advanced_load_balancer
                        .execute_dynamic_rebalancing(
                            &fixture.test_cluster.get_cluster_state(),
                            RebalancingTrigger::ScheduledRebalancing,
                        )
                        .await
                        .unwrap(),
                )
            } else {
                None
            };

            let workflow_time = workflow_start.elapsed();

            workflow_results.push(WorkflowResult {
                pattern_name: pattern.signature.clone(),
                placement_decision_time: workflow_time,
                placement_confidence: placement_decision.confidence,
                rebalancing_executed: rebalancing_result.is_some(),
                rebalancing_efficiency: rebalancing_result.as_ref().map(|r| r.efficiency),
                total_workflow_time: workflow_time,
            });
        }

        let total_integration_time = integration_start.elapsed();

        // Analyze integration performance
        let average_workflow_time = workflow_results
            .iter()
            .map(|r| r.total_workflow_time)
            .sum::<Duration>()
            / workflow_results.len() as u32;

        let average_placement_confidence = workflow_results
            .iter()
            .map(|r| r.placement_confidence)
            .sum::<f64>()
            / workflow_results.len() as f64;

        let rebalancing_workflows = workflow_results
            .iter()
            .filter(|r| r.rebalancing_executed)
            .count();
        let average_rebalancing_efficiency = workflow_results
            .iter()
            .filter_map(|r| r.rebalancing_efficiency)
            .sum::<f64>()
            / rebalancing_workflows.max(1) as f64;

        println!("System Integration Performance Test Results:");
        println!(
            "  Total integration test time: {:?}",
            total_integration_time
        );
        println!("  Average workflow time: {:?}", average_workflow_time);
        println!(
            "  Average placement confidence: {:.3}",
            average_placement_confidence
        );
        println!(
            "  Rebalancing workflows executed: {}/{}",
            rebalancing_workflows,
            workflow_results.len()
        );
        println!(
            "  Average rebalancing efficiency: {:.2}%",
            average_rebalancing_efficiency * 100.0
        );

        // Assert integration performance
        assert!(
            average_workflow_time <= Duration::from_millis(20),
            "Average workflow time {:?} exceeds 20ms limit",
            average_workflow_time
        );
        assert!(
            average_placement_confidence >= 0.8,
            "Average placement confidence {:.3} below 0.8 threshold",
            average_placement_confidence
        );

        if rebalancing_workflows > 0 {
            assert!(
                average_rebalancing_efficiency >= 0.85,
                "Average rebalancing efficiency {:.1}% below 85% threshold",
                average_rebalancing_efficiency * 100.0
            );
        }
    }

    /// Test 6: Performance Statistics and Metrics
    #[tokio::test]
    async fn test_performance_statistics_collection() {
        let fixture = Phase3TestFixture::new().await;

        println!("Testing performance statistics collection");

        // Generate workload to populate metrics
        for _ in 0..50 {
            let pattern = &fixture.test_patterns[0];
            let _ = fixture
                .advanced_load_balancer
                .make_intelligent_placement_decision(
                    pattern,
                    &ResourceRequirements {
                        memory_mb: 1024,
                        cpu_cores: 2,
                        network_bandwidth_mbps: 100,
                        storage_gb: 10,
                    },
                    &fixture.test_cluster.get_cluster_state(),
                )
                .await
                .unwrap();
        }

        // Execute some rebalancing operations
        for _ in 0..5 {
            let _ = fixture
                .advanced_load_balancer
                .execute_dynamic_rebalancing(
                    &fixture.test_cluster.get_cluster_state(),
                    RebalancingTrigger::ScheduledRebalancing,
                )
                .await
                .unwrap();
        }

        // Get performance statistics
        let performance_stats = fixture
            .advanced_load_balancer
            .get_performance_statistics()
            .await
            .unwrap();

        println!("Performance Statistics:");
        println!(
            "  Total decisions made: {}",
            performance_stats.total_decisions_made
        );
        println!(
            "  Average decision time: {:?}",
            performance_stats.average_decision_time
        );
        println!(
            "  Decision target achievement rate: {:.2}%",
            performance_stats.decision_target_achievement_rate
        );
        println!(
            "  Average prediction confidence: {:.2}%",
            performance_stats.average_prediction_confidence
        );
        println!(
            "  Total rebalancing operations: {}",
            performance_stats.total_rebalancing_operations
        );
        println!(
            "  Rebalancing efficiency rate: {:.2}%",
            performance_stats.rebalancing_efficiency_rate
        );
        println!(
            "  Average rebalancing efficiency: {:.2}%",
            performance_stats.average_rebalancing_efficiency
        );

        // Assert metrics collection
        assert!(
            performance_stats.total_decisions_made >= 50,
            "Expected at least 50 decisions"
        );
        assert!(
            performance_stats.total_rebalancing_operations >= 5,
            "Expected at least 5 rebalancing operations"
        );
        assert!(
            performance_stats.average_decision_time <= Duration::from_millis(10),
            "Average decision time {:?} exceeds 10ms",
            performance_stats.average_decision_time
        );
        assert!(
            performance_stats.average_prediction_confidence >= 70.0,
            "Average prediction confidence {:.1}% below 70%",
            performance_stats.average_prediction_confidence
        );
    }

    // ============================================================================
    // Supporting Test Infrastructure
    // ============================================================================

    /// Test cluster simulator
    #[derive(Clone)]
    struct TestCluster {
        nodes: HashMap<NodeId, TestNode>,
        cluster_state: ClusterState,
    }

    impl TestCluster {
        async fn new(node_count: usize) -> Self {
            let mut nodes = HashMap::new();
            let mut cluster_nodes = HashMap::new();

            for i in 0..node_count {
                let node_id = NodeId::from(i as u64);
                let test_node = TestNode::new(i);
                let cluster_node = test_node.to_node_info();

                nodes.insert(node_id, test_node);
                cluster_nodes.insert(node_id, cluster_node);
            }

            TestCluster {
                nodes,
                cluster_state: ClusterState {
                    nodes: cluster_nodes,
                    last_update: SystemTime::now(),
                    cluster_id: "test-cluster".to_string(),
                },
            }
        }

        fn get_cluster_state(&self) -> ClusterState {
            self.cluster_state.clone()
        }

        fn create_core_load_balancer(
            &self,
        ) -> crate::concurrency::distributed_load_balancer::DistributedLoadBalancer {
            // Create a simplified load balancer for testing
            let config = LoadBalancingConfig {
                strategy: "continuation_aware".to_string(),
                rebalance_threshold: 0.8,
                rebalance_interval: Duration::from_secs(30),
            };

            let node_id = self.cluster_state.nodes.keys().next().copied().unwrap();
            crate::concurrency::distributed_load_balancer::DistributedLoadBalancer::new(
                node_id, config,
            )
            .unwrap()
        }

        async fn determine_actual_optimal_node(
            &self,
            pattern: &ContinuationExecutionPattern,
        ) -> Result<NodeId> {
            // Simulate exhaustive testing to find actual optimal node
            let mut best_node = None;
            let mut best_score = 0.0;

            for (node_id, test_node) in &self.nodes {
                let score = self.calculate_node_suitability_score(test_node, pattern);
                if score > best_score {
                    best_score = score;
                    best_node = Some(*node_id);
                }
            }

            best_node.ok_or_else(|| Error::runtime_error("No optimal node found".to_string(), None))
        }

        fn calculate_node_suitability_score(
            &self,
            node: &TestNode,
            pattern: &ContinuationExecutionPattern,
        ) -> f64 {
            let mut score = 0.0;

            // CPU suitability
            let cpu_demand = pattern
                .execution_segments
                .iter()
                .map(|s| s.cpu_intensity)
                .sum::<f64>()
                / pattern.execution_segments.len() as f64;
            if node.cpu_utilization < 0.7 && cpu_demand > 0.5 {
                score += 0.3;
            }

            // Memory suitability
            let memory_demand = pattern.estimated_memory_usage;
            if node.available_memory_gb * 1024 * 1024 * 1024 > memory_demand * 2 {
                score += 0.3;
            }

            // JIT suitability
            let jit_segments = pattern
                .execution_segments
                .iter()
                .filter(|s| s.jit_compilable)
                .count();
            if node.jit_capable && jit_segments > 0 {
                score += 0.4 * (jit_segments as f64 / pattern.execution_segments.len() as f64);
            }

            score
        }

        async fn create_load_imbalance(&mut self, imbalance_factor: f64) {
            // Create artificial load imbalance for testing
            let node_ids: Vec<_> = self.nodes.keys().copied().collect();

            for (i, node_id) in node_ids.iter().enumerate() {
                if let Some(test_node) = self.nodes.get_mut(node_id) {
                    if i < node_ids.len() / 2 {
                        // Overload first half of nodes
                        test_node.cpu_utilization = (0.8 + imbalance_factor * 0.2).min(0.95);
                        test_node.memory_utilization = (0.7 + imbalance_factor * 0.2).min(0.9);
                    } else {
                        // Underload second half of nodes
                        test_node.cpu_utilization = (0.2 - imbalance_factor * 0.15).max(0.05);
                        test_node.memory_utilization = (0.3 - imbalance_factor * 0.2).max(0.1);
                    }
                }
            }

            // Update cluster state
            for (node_id, test_node) in &self.nodes {
                if let Some(cluster_node) = self.cluster_state.nodes.get_mut(node_id) {
                    *cluster_node = test_node.to_node_info();
                }
            }
        }

        fn calculate_load_imbalance(&self) -> f64 {
            let loads: Vec<f64> = self.nodes.values().map(|n| n.cpu_utilization).collect();
            let mean_load = loads.iter().sum::<f64>() / loads.len() as f64;
            let variance =
                loads.iter().map(|l| (l - mean_load).powi(2)).sum::<f64>() / loads.len() as f64;
            variance.sqrt()
        }

        fn calculate_load_imbalance_after_rebalancing(
            &self,
            _result: &DynamicRebalancingResult,
        ) -> f64 {
            // Simulate improved balance after rebalancing
            self.calculate_load_imbalance() * 0.3 // 70% improvement
        }
    }

    /// Individual test node
    #[derive(Debug, Clone)]
    struct TestNode {
        id: usize,
        cpu_capacity_cores: u32,
        cpu_utilization: f64,
        memory_capacity_gb: u64,
        memory_utilization: f64,
        available_memory_gb: u64,
        network_bandwidth_gbps: f64,
        jit_capable: bool,
        performance_score: f64,
    }

    impl TestNode {
        fn new(id: usize) -> Self {
            let cpu_capacity = 8 + (id % 4) as u32 * 4; // 8-20 cores
            let memory_capacity = 32 + (id % 4) as u64 * 32; // 32-128 GB
            let cpu_utilization = 0.3 + (id as f64 * 0.1) % 0.5; // 0.3-0.8
            let memory_utilization = 0.2 + (id as f64 * 0.08) % 0.4; // 0.2-0.6

            TestNode {
                id,
                cpu_capacity_cores: cpu_capacity,
                cpu_utilization,
                memory_capacity_gb: memory_capacity,
                memory_utilization,
                available_memory_gb: memory_capacity
                    - (memory_capacity as f64 * memory_utilization) as u64,
                network_bandwidth_gbps: 10.0 + (id as f64 * 2.0),
                jit_capable: id % 3 != 0, // 2/3 of nodes are JIT capable
                performance_score: 0.7 + (id as f64 * 0.05) % 0.3,
            }
        }

        fn to_node_info(&self) -> NodeInfo {
            NodeInfo {
                node_id: NodeId::from(self.id as u64),
                cpu_cores: self.cpu_capacity_cores,
                cpu_usage: self.cpu_utilization,
                memory_total_gb: self.memory_capacity_gb,
                memory_usage: self.memory_utilization,
                available_memory: self.available_memory_gb * 1024 * 1024 * 1024, // Convert to bytes
                network_latency: Duration::from_millis(1 + (self.id % 10) as u64),
                supports_jit_compilation: self.jit_capable,
                jit_performance_factor: if self.jit_capable {
                    self.performance_score
                } else {
                    0.0
                },
                cpu_performance_score: self.performance_score,
                continuation_performance_history: HashMap::new(),
                is_healthy: true,
                last_heartbeat: SystemTime::now(),
            }
        }
    }

    /// Test result structures
    #[derive(Debug)]
    struct PredictionResult {
        scenario: usize,
        pattern_type: String,
        predicted_node: NodeId,
        actual_optimal_node: NodeId,
        is_correct: bool,
        confidence: f64,
    }

    #[derive(Debug)]
    struct RebalancingEfficiencyResult {
        scenario: usize,
        initial_imbalance: f64,
        final_imbalance: f64,
        efficiency: f64,
        rebalancing_time: Duration,
        nodes_affected: usize,
        service_availability: f64,
    }

    #[derive(Debug)]
    struct JitOptimizationResult {
        pattern_name: String,
        jit_compatible_segments: usize,
        total_segments: usize,
        expected_improvement: f64,
        confidence: f64,
        compilation_time: Duration,
    }

    #[derive(Debug)]
    struct WorkflowResult {
        pattern_name: String,
        placement_decision_time: Duration,
        placement_confidence: f64,
        rebalancing_executed: bool,
        rebalancing_efficiency: Option<f64>,
        total_workflow_time: Duration,
    }

    /// Integration test for complete Phase 3.3 system validation
    #[tokio::test]
    async fn test_phase3_complete_system_validation() {
        println!("=== Phase 3.3 Complete System Validation ===");

        let validation_start = Instant::now();

        // Run all validation tests
        println!("1. Running 5ms decision time validation...");
        test_5ms_decision_time_requirement().await;

        println!("2. Running 80% ML prediction accuracy validation...");
        test_80_percent_ml_prediction_accuracy().await;

        println!("3. Running 90% rebalancing efficiency validation...");
        test_90_percent_rebalancing_efficiency().await;

        println!("4. Running JIT integration optimization validation...");
        test_jit_integration_optimization().await;

        println!("5. Running system integration performance validation...");
        test_system_integration_performance().await;

        println!("6. Running performance statistics validation...");
        test_performance_statistics_collection().await;

        let total_validation_time = validation_start.elapsed();

        println!("=== Phase 3.3 Validation Summary ===");
        println!("✅ 5ms decision time requirement: PASSED");
        println!("✅ 80% ML prediction accuracy requirement: PASSED");
        println!("✅ 90% rebalancing efficiency requirement: PASSED");
        println!("✅ JIT integration optimization: PASSED");
        println!("✅ System integration performance: PASSED");
        println!("✅ Performance statistics collection: PASSED");
        println!("⏱️  Total validation time: {:?}", total_validation_time);
        println!("🎉 Phase 3.3 Advanced Load Balancing System: ALL TESTS PASSED");

        // Assert overall validation success
        assert!(
            total_validation_time <= Duration::from_secs(120),
            "Complete validation took {:?}, should complete within 2 minutes",
            total_validation_time
        );
    }
}
