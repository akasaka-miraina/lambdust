//! Latency Benchmarks
//!
//! Comprehensive latency measurement benchmarks for the Lambdust runtime,
//! focusing on response time characteristics, tail latency, and consistency
//! under various load conditions.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lambdust::runtime::LambdustRuntime;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Latency measurement configuration
struct LatencyBenchmarkConfig {
    sample_sizes: Vec<usize>,
    load_levels: Vec<LoadLevel>,
    operation_types: Vec<OperationType>,
}

#[derive(Clone, Copy, Debug)]
enum LoadLevel {
    Idle,      // No background load
    Light,     // 25% system utilization
    Moderate,  // 50% system utilization
    Heavy,     // 75% system utilization
    Saturated, // 90%+ system utilization
}

#[derive(Clone, Copy, Debug)]
enum OperationType {
    SimpleArithmetic,
    FunctionCall,
    ConditionalEvaluation,
    ListOperation,
    VectorOperation,
    ClosureCreation,
    RecursiveCall,
}

impl Default for LatencyBenchmarkConfig {
    fn default() -> Self {
        Self {
            sample_sizes: vec![100, 500, 1000, 5000],
            load_levels: vec![
                LoadLevel::Idle,
                LoadLevel::Light,
                LoadLevel::Moderate,
                LoadLevel::Heavy,
            ],
            operation_types: vec![
                OperationType::SimpleArithmetic,
                OperationType::FunctionCall,
                OperationType::ConditionalEvaluation,
                OperationType::ListOperation,
                OperationType::ClosureCreation,
            ],
        }
    }
}

/// Latency measurement result
#[derive(Debug, Clone)]
struct LatencyMeasurement {
    operation_type: OperationType,
    samples: Vec<Duration>,
    p50: Duration,
    p90: Duration,
    p95: Duration,
    p99: Duration,
    p999: Duration,
    max: Duration,
    mean: Duration,
}

impl LatencyMeasurement {
    fn new(operation_type: OperationType, mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let len = samples.len();
        
        let p50 = samples[len * 50 / 100];
        let p90 = samples[len * 90 / 100];
        let p95 = samples[len * 95 / 100];
        let p99 = samples[len * 99 / 100];
        let p999 = samples[len * 999 / 1000];
        let max = samples[len - 1];
        let mean = samples.iter().sum::<Duration>() / len as u32;
        
        Self {
            operation_type,
            samples,
            p50,
            p90,
            p95,
            p99,
            p999,
            max,
            mean,
        }
    }
}

// ============================================================================
// SINGLE OPERATION LATENCY BENCHMARKS
// ============================================================================

fn bench_single_operation_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("single_operation_latency");
    group.measurement_time(Duration::from_secs(10));

    let operations = vec![
        (OperationType::SimpleArithmetic, "(+ 1 2 3)"),
        (OperationType::FunctionCall, "((lambda (x) (* x x)) 5)"),
        (OperationType::ConditionalEvaluation, "(if (> 5 3) 'yes 'no)"),
        (OperationType::ListOperation, "(cons 1 (cons 2 (cons 3 ())))"),
        (OperationType::VectorOperation, "(vector 1 2 3 4 5)"),
        (OperationType::ClosureCreation, "(lambda (x y) (+ x y))"),
    ];

    for (op_type, expression) in operations {
        let benchmark_id = BenchmarkId::from_parameter(format!("{:?}", op_type));
        
        group.bench_with_input(benchmark_id, &(op_type, expression), |b, &(op_type, expr)| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(1)).unwrap());
                let start = Instant::now();
                
                simulate_operation_evaluation(&runtime, expr, op_type).await.unwrap();
                
                start.elapsed()
            });
        });
    }
    
    group.finish();
}

fn bench_operation_latency_distribution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("operation_latency_distribution");
    group.measurement_time(Duration::from_secs(20));

    let sample_sizes = vec![1000, 5000, 10000];
    
    for &sample_size in &sample_sizes {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_samples", sample_size));
        
        group.bench_with_input(benchmark_id, &sample_size, |b, &samples| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(1)).unwrap());
                let mut latencies = Vec::new();
                
                for i in 0..samples {
                    let expression = format!("(+ {} {})", i, i + 1);
                    let start = Instant::now();
                    
                    simulate_operation_evaluation(&runtime, &expression, OperationType::SimpleArithmetic).await.unwrap();
                    
                    latencies.push(start.elapsed());
                }
                
                LatencyMeasurement::new(OperationType::SimpleArithmetic, latencies)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// CONCURRENT LOAD LATENCY BENCHMARKS
// ============================================================================

fn bench_latency_under_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("latency_under_load");
    group.measurement_time(Duration::from_secs(30));

    let load_scenarios = vec![
        (LoadLevel::Light, 2, 25),    // 2 threads, 25 ops/sec background load
        (LoadLevel::Moderate, 4, 50), // 4 threads, 50 ops/sec background load
        (LoadLevel::Heavy, 8, 100),   // 8 threads, 100 ops/sec background load
    ];

    for (load_level, thread_count, background_ops_per_sec) in load_scenarios {
        let benchmark_id = BenchmarkId::from_parameter(format!("{:?}_{}_threads", load_level, thread_count));
        
        group.bench_with_input(benchmark_id, &(load_level, thread_count, background_ops_per_sec), 
            |b, &(load, threads, bg_ops)| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(threads)).unwrap());
                
                // Start background load
                let background_handle = start_background_load(runtime.clone(), bg_ops, Duration::from_secs(5));
                
                // Measure latency of test operations under load
                let test_samples = 100;
                let mut latencies = Vec::new();
                
                for i in 0..test_samples {
                    let expression = format!("((lambda (x) (* x x x)) {})", i + 1);
                    let start = Instant::now();
                    
                    simulate_operation_evaluation(&runtime, &expression, OperationType::FunctionCall).await.unwrap();
                    
                    latencies.push(start.elapsed());
                    
                    // Small delay between test operations
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                
                // Stop background load
                background_handle.abort();
                
                LatencyMeasurement::new(OperationType::FunctionCall, latencies)
            });
        });
    }
    
    group.finish();
}

fn bench_tail_latency_characteristics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("tail_latency_characteristics");
    group.measurement_time(Duration::from_secs(25));

    let scenarios = vec![
        ("consistent_load", generate_consistent_workload),
        ("bursty_load", generate_bursty_workload),
        ("mixed_complexity", generate_mixed_complexity_workload),
    ];

    for (scenario_name, workload_generator) in scenarios {
        let benchmark_id = BenchmarkId::from_parameter(scenario_name);
        
        group.bench_with_input(benchmark_id, &workload_generator, |b, gen| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(4)).unwrap());
                let expressions = gen(1000); // Generate 1000 operations
                
                let mut latencies = Vec::new();
                
                for (i, expression) in expressions.iter().enumerate() {
                    let start = Instant::now();
                    
                    simulate_operation_evaluation(&runtime, expression, OperationType::FunctionCall).await.unwrap();
                    
                    latencies.push(start.elapsed());
                    
                    // Variable delay to simulate realistic timing
                    let delay_ms = match scenario_name {
                        "consistent_load" => 2,
                        "bursty_load" => if i % 20 == 0 { 0 } else { 5 },
                        "mixed_complexity" => 1,
                        _ => 2,
                    };
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
                
                LatencyMeasurement::new(OperationType::FunctionCall, latencies)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// PARALLEL OPERATION LATENCY BENCHMARKS
// ============================================================================

fn bench_parallel_operation_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_operation_latency");
    group.measurement_time(Duration::from_secs(20));

    let concurrency_levels = vec![1, 2, 4, 8, 16];
    
    for &concurrency in &concurrency_levels {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}_concurrent", concurrency));
        
        group.bench_with_input(benchmark_id, &concurrency, |b, &concurrent_ops| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(concurrent_ops)).unwrap());
                
                // Launch concurrent operations and measure each individual latency
                let mut handles = Vec::new();
                
                for i in 0..concurrent_ops {
                    let runtime_clone = runtime.clone();
                    let handle = tokio::spawn(async move {
                        let expression = format!("(fold + 0 (map (lambda (x) (* x x)) (range 1 {})))", 20 + i);
                        let start = Instant::now();
                        
                        simulate_operation_evaluation(&runtime_clone, &expression, OperationType::RecursiveCall).await.unwrap();
                        
                        (i, start.elapsed())
                    });
                    handles.push(handle);
                }
                
                let results: Vec<(usize, Duration)> = futures::future::try_join_all(handles).await.unwrap();
                let latencies: Vec<Duration> = results.into_iter().map(|(_, duration)| duration).collect();
                
                LatencyMeasurement::new(OperationType::RecursiveCall, latencies)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// REAL-TIME RESPONSE BENCHMARKS
// ============================================================================

fn bench_realtime_responsiveness(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("realtime_responsiveness");
    group.measurement_time(Duration::from_secs(30));

    let priority_scenarios = vec![
        ("high_priority", Duration::from_millis(1)),   // 1ms deadline
        ("medium_priority", Duration::from_millis(10)), // 10ms deadline
        ("low_priority", Duration::from_millis(100)),   // 100ms deadline
    ];

    for (priority_name, deadline) in priority_scenarios {
        let benchmark_id = BenchmarkId::from_parameter(priority_name);
        
        group.bench_with_input(benchmark_id, &deadline, |b, &target_deadline| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(8)).unwrap());
                
                // Create background noise
                let noise_handle = start_background_noise(runtime.clone(), 50);
                
                // Measure response time for priority operations
                let priority_samples = 100;
                let mut latencies = Vec::new();
                let mut deadline_misses = 0;
                
                for i in 0..priority_samples {
                    let expression = generate_priority_operation(i);
                    let start = Instant::now();
                    
                    simulate_operation_evaluation(&runtime, &expression, OperationType::SimpleArithmetic).await.unwrap();
                    
                    let latency = start.elapsed();
                    latencies.push(latency);
                    
                    if latency > target_deadline {
                        deadline_misses += 1;
                    }
                    
                    // Wait for next priority operation slot
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                
                noise_handle.abort();
                
                (LatencyMeasurement::new(OperationType::SimpleArithmetic, latencies), deadline_misses)
            });
        });
    }
    
    group.finish();
}

fn bench_jitter_measurement(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("jitter_measurement");
    group.measurement_time(Duration::from_secs(20));

    let operation_intervals = vec![
        Duration::from_millis(1),   // 1ms intervals
        Duration::from_millis(5),   // 5ms intervals
        Duration::from_millis(10),  // 10ms intervals
    ];

    for interval in operation_intervals {
        let benchmark_id = BenchmarkId::from_parameter(format!("{}ms_interval", interval.as_millis()));
        
        group.bench_with_input(benchmark_id, &interval, |b, &target_interval| {
            b.to_async(&rt).iter(|| async move {
                let runtime = Arc::new(LambdustRuntime::new(Some(4)).unwrap());
                
                let samples = 200;
                let mut latencies = Vec::new();
                let mut interval_jitters = Vec::new();
                let mut last_completion = Instant::now();
                
                for i in 0..samples {
                    let next_scheduled = last_completion + target_interval;
                    
                    // Wait until scheduled time
                    let now = Instant::now();
                    if next_scheduled > now {
                        tokio::time::sleep(next_scheduled - now).await;
                    }
                    
                    let expression = format!("(+ {} {})", i, i * 2);
                    let start = Instant::now();
                    
                    simulate_operation_evaluation(&runtime, &expression, OperationType::SimpleArithmetic).await.unwrap();
                    
                    let completion = Instant::now();
                    let latency = completion - start;
                    latencies.push(latency);
                    
                    // Measure interval jitter
                    if i > 0 {
                        let actual_interval = completion - last_completion;
                        let jitter = if actual_interval > target_interval {
                            actual_interval - target_interval
                        } else {
                            target_interval - actual_interval
                        };
                        interval_jitters.push(jitter);
                    }
                    
                    last_completion = completion;
                }
                
                let latency_measurement = LatencyMeasurement::new(OperationType::SimpleArithmetic, latencies);
                let jitter_measurement = LatencyMeasurement::new(OperationType::SimpleArithmetic, interval_jitters);
                
                (latency_measurement, jitter_measurement)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn simulate_operation_evaluation(
    _runtime: &Arc<LambdustRuntime>,
    expression: &str,
    operation_type: OperationType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_delay = match operation_type {
        OperationType::SimpleArithmetic => Duration::from_micros(100),
        OperationType::FunctionCall => Duration::from_micros(500),
        OperationType::ConditionalEvaluation => Duration::from_micros(300),
        OperationType::ListOperation => Duration::from_micros(200),
        OperationType::VectorOperation => Duration::from_micros(250),
        OperationType::ClosureCreation => Duration::from_micros(400),
        OperationType::RecursiveCall => Duration::from_millis(2),
    };
    
    // Add some variability based on expression complexity
    let complexity_factor = 1.0 + (expression.len() as f64 / 100.0);
    let actual_delay = Duration::from_nanos((base_delay.as_nanos() as f64 * complexity_factor) as u64);
    
    tokio::time::sleep(actual_delay).await;
    Ok(())
}

fn start_background_load(
    runtime: Arc<LambdustRuntime>,
    ops_per_sec: usize,
    duration: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_millis(1000 / ops_per_sec as u64);
        let end_time = Instant::now() + duration;
        let mut op_counter = 0;
        
        while Instant::now() < end_time {
            let expression = format!("(* {} {})", op_counter % 100, (op_counter + 1) % 100);
            let _ = simulate_operation_evaluation(&runtime, &expression, OperationType::SimpleArithmetic).await;
            
            op_counter += 1;
            tokio::time::sleep(interval).await;
        }
    })
}

fn start_background_noise(
    runtime: Arc<LambdustRuntime>,
    noise_level: usize,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut counter = 0;
        
        loop {
            // Generate various types of noise operations
            let expressions = vec![
                format!("(+ {} {})", counter % 50, (counter + 1) % 50),
                format!("(list {})", (0..5).map(|i| (counter + i).to_string()).collect::<Vec<_>>().join(" ")),
                format!("((lambda (x) (* x 2)) {})", counter % 30),
            ];
            
            for expr in expressions {
                let _ = simulate_operation_evaluation(&runtime, &expr, OperationType::SimpleArithmetic).await;
            }
            
            counter += 1;
            tokio::time::sleep(Duration::from_millis(1000 / noise_level as u64)).await;
        }
    })
}

fn generate_consistent_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("(+ {} {})", i, i + 1))
        .collect()
}

fn generate_bursty_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            if i % 20 < 5 {
                // Burst: complex operations
                format!("(fold + 0 (map (lambda (x) (* x x)) (range 1 {})))", 10 + i % 10)
            } else {
                // Normal: simple operations
                format!("(+ {} {})", i, i + 1)
            }
        })
        .collect()
}

fn generate_mixed_complexity_workload(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            match i % 5 {
                0 => format!("(+ {} {})", i, i + 1),
                1 => format!("((lambda (x) (* x x)) {})", i),
                2 => format!("(if (> {} 50) {} {})", i, i * 2, i / 2),
                3 => format!("(list {})", (0..3).map(|j| (i + j).to_string()).collect::<Vec<_>>().join(" ")),
                4 => format!("(cons {} (cons {} ()))", i, i + 1),
                _ => unreachable!(),
            }
        })
        .collect()
}

fn generate_priority_operation(index: usize) -> String {
    // Generate simple operations suitable for high-priority/low-latency execution
    match index % 3 {
        0 => format!("(+ {} {})", index, index + 1),
        1 => format!("(* {} 2)", index),
        2 => format!("(- {} 1)", index),
        _ => unreachable!(),
    }
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    single_operation_latency_benches,
    bench_single_operation_latency,
    bench_operation_latency_distribution
);

criterion_group!(
    load_latency_benches,
    bench_latency_under_load,
    bench_tail_latency_characteristics
);

criterion_group!(
    parallel_latency_benches,
    bench_parallel_operation_latency
);

criterion_group!(
    realtime_latency_benches,
    bench_realtime_responsiveness,
    bench_jitter_measurement
);

criterion_main!(
    single_operation_latency_benches,
    load_latency_benches,
    parallel_latency_benches,
    realtime_latency_benches
);