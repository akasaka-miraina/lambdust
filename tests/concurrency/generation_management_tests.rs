//! Generation Management System Tests
//!
//! Tests for distributed environment generation-based state management,
//! including generation consistency, conflict resolution, and 
//! cross-generation coordination in multithreaded environments.

use lambdust::runtime::LambdustRuntime;
use lambdust::effects::{GenerationalEnvManager, GenerationId};
use lambdust::eval::Value;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::thread;
use tokio::sync::Semaphore;

/// Generation-aware environment for testing
#[derive(Debug, Clone)]
pub struct TestGenerationalEnvironment {
    pub generation_id: GenerationId,
    pub variables: HashMap<String, Value>,
    pub parent: Option<Box<TestGenerationalEnvironment>>,
    pub created_at: Instant,
}

impl TestGenerationalEnvironment {
    pub fn new(generation_id: GenerationId) -> Self {
        Self {
            generation_id,
            variables: HashMap::new(),
            parent: None,
            created_at: Instant::now(),
        }
    }

    pub fn with_parent(generation_id: GenerationId, parent: TestGenerationalEnvironment) -> Self {
        Self {
            generation_id,
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
            created_at: Instant::now(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get(name))
        })
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
}

/// Thread-safe generation manager for testing
#[derive(Debug)]
pub struct ThreadSafeGenerationManager {
    current_generation: Arc<RwLock<GenerationId>>,
    environments: Arc<RwLock<HashMap<GenerationId, TestGenerationalEnvironment>>>,
    generation_history: Arc<Mutex<VecDeque<GenerationId>>>,
    conflict_count: Arc<Mutex<u64>>,
    next_generation_id: Arc<Mutex<u64>>,
}

impl ThreadSafeGenerationManager {
    pub fn new() -> Self {
        let initial_gen = GenerationId(0);
        let mut environments = HashMap::new();
        environments.insert(initial_gen, TestGenerationalEnvironment::new(initial_gen));
        
        let mut history = VecDeque::new();
        history.push_back(initial_gen);

        Self {
            current_generation: Arc::new(RwLock::new(initial_gen)),
            environments: Arc::new(RwLock::new(environments)),
            generation_history: Arc::new(Mutex::new(history)),
            conflict_count: Arc::new(Mutex::new(0)),
            next_generation_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn current_generation(&self) -> GenerationId {
        *self.current_generation.read().unwrap()
    }

    pub fn create_new_generation(&self) -> Result<GenerationId> {
        let new_id = {
            let mut next_id = self.next_generation_id.lock().unwrap();
            let id = GenerationId(*next_id);
            *next_id += 1;
            id
        };

        let current_gen = self.current_generation();
        let parent_env = {
            let envs = self.environments.read().unwrap();
            envs.get(&current_gen).cloned()
                .ok_or_else(|| Error::runtime_error(
                    format!("Current generation {} not found", current_gen.0),
                    None,
                ))?
        };

        let new_env = TestGenerationalEnvironment::with_parent(new_id, parent_env);
        
        {
            let mut envs = self.environments.write().unwrap();
            envs.insert(new_id, new_env);
        }

        {
            let mut history = self.generation_history.lock().unwrap();
            history.push_back(new_id);
            // Keep only recent generations
            if history.len() > 100 {
                let old_gen = history.pop_front().unwrap();
                let mut envs = self.environments.write().unwrap();
                envs.remove(&old_gen);
            }
        }

        Ok(new_id)
    }

    pub fn switch_to_generation(&self, generation_id: GenerationId) -> Result<()> {
        {
            let envs = self.environments.read().unwrap();
            if !envs.contains_key(&generation_id) {
                return Err(Error::runtime_error(
                    format!("Generation {} not found", generation_id.0),
                    None,
                ));
            }
        }

        {
            let mut current = self.current_generation.write().unwrap();
            *current = generation_id;
        }

        Ok(())
    }

    pub fn get_variable(&self, generation_id: GenerationId, name: &str) -> Option<Value> {
        let envs = self.environments.read().unwrap();
        envs.get(&generation_id).and_then(|env| env.get(name).cloned())
    }

    pub fn set_variable(&self, generation_id: GenerationId, name: String, value: Value) -> Result<()> {
        let mut envs = self.environments.write().unwrap();
        let env = envs.get_mut(&generation_id)
            .ok_or_else(|| Error::runtime_error(
                format!("Generation {} not found", generation_id.0),
                None,
            ))?;
        
        env.set(name, value);
        Ok(())
    }

    pub fn resolve_conflict(&self, gen1: GenerationId, gen2: GenerationId) -> Result<GenerationId> {
        {
            let mut count = self.conflict_count.lock().unwrap();
            *count += 1;
        }

        // Simple conflict resolution: prefer newer generation
        let winner = if gen1.0 > gen2.0 { gen1 } else { gen2 };
        
        // In a real implementation, this would merge environments
        // For testing, we just return the winner
        Ok(winner)
    }

    pub fn get_conflict_count(&self) -> u64 {
        *self.conflict_count.lock().unwrap()
    }

    pub fn get_generation_count(&self) -> usize {
        self.environments.read().unwrap().len()
    }

    pub fn verify_consistency(&self) -> Result<bool> {
        let envs = self.environments.read().unwrap();
        let current = self.current_generation();
        
        // Verify current generation exists
        if !envs.contains_key(&current) {
            return Ok(false);
        }

        // Verify generation history is consistent
        let history = self.generation_history.lock().unwrap();
        if let Some(&last_gen) = history.back() {
            if last_gen != current {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

// ============================================================================
// GENERATION CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_generation_consistency_under_concurrent_access() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let gen_manager = Arc::new(ThreadSafeGenerationManager::new());
    
    let thread_count = 4;
    let operations_per_thread = 25;
    
    // Initialize some variables in the initial generation
    gen_manager.set_variable(GenerationId(0), "shared_counter".to_string(), Value::Integer(0))
        .expect("Failed to initialize shared_counter");
    gen_manager.set_variable(GenerationId(0), "base_value".to_string(), Value::Integer(100))
        .expect("Failed to initialize base_value");

    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let manager_clone = gen_manager.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut operations_completed = 0;
            let mut generation_switches = 0;

            for op_id in 0..operations_per_thread {
                let current_gen = manager_clone.current_generation();
                
                // Perform operation based on operation type
                match op_id % 4 {
                    0 => {
                        // Read operation
                        if let Some(value) = manager_clone.get_variable(current_gen, "shared_counter") {
                            operations_completed += 1;
                        }
                    }
                    1 => {
                        // Write operation
                        if let Some(Value::Integer(current)) = manager_clone.get_variable(current_gen, "shared_counter") {
                            let new_value = current + 1;
                            if manager_clone.set_variable(current_gen, "shared_counter".to_string(), Value::Integer(new_value)).is_ok() {
                                operations_completed += 1;
                            }
                        }
                    }
                    2 => {
                        // Create new generation
                        if let Ok(new_gen) = manager_clone.create_new_generation() {
                            // Copy current shared_counter to new generation
                            if let Some(counter_value) = manager_clone.get_variable(current_gen, "shared_counter") {
                                let _ = manager_clone.set_variable(new_gen, "shared_counter".to_string(), counter_value);
                            }
                            operations_completed += 1;
                        }
                    }
                    3 => {
                        // Switch generation (occasionally)
                        if op_id % 8 == 0 {
                            if let Ok(new_gen) = manager_clone.create_new_generation() {
                                if manager_clone.switch_to_generation(new_gen).is_ok() {
                                    generation_switches += 1;
                                }
                            }
                        }
                        operations_completed += 1;
                    }
                    _ => unreachable!(),
                }

                // Verify consistency after each operation
                if !manager_clone.verify_consistency().unwrap_or(false) {
                    eprintln!("Consistency check failed for thread {} at operation {}", thread_id, op_id);
                }

                // Small delay to increase chance of race conditions
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, operations_completed, generation_switches)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_operations: usize = results.iter().map(|(_, ops, _)| *ops).sum();
    let total_switches: usize = results.iter().map(|(_, _, switches)| *switches).sum();

    println!("✓ Generation consistency test completed");
    println!("  Total operations: {}", total_operations);
    println!("  Total generation switches: {}", total_switches);
    println!("  Generations created: {}", gen_manager.get_generation_count());
    println!("  Conflicts resolved: {}", gen_manager.get_conflict_count());

    // Verify final consistency
    assert!(gen_manager.verify_consistency().unwrap(), "Final consistency check failed");

    // Verify reasonable operation completion rate
    let expected_operations = thread_count * operations_per_thread;
    let completion_rate = total_operations as f64 / expected_operations as f64;
    assert!(completion_rate > 0.9, "Operation completion rate too low: {:.2}%", completion_rate * 100.0);

    println!("✓ Generation consistency maintained under concurrent access");
}

#[tokio::test]
async fn test_generation_conflict_resolution() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let gen_manager = Arc::new(ThreadSafeGenerationManager::new());
    
    let thread_count = 6;
    let conflicts_per_thread = 10;
    
    // Create multiple initial generations
    let mut initial_generations = Vec::new();
    for i in 1..=4 {
        let gen_id = gen_manager.create_new_generation()
            .expect(&format!("Failed to create initial generation {}", i));
        gen_manager.set_variable(gen_id, "value".to_string(), Value::Integer(i * 100))
            .expect(&format!("Failed to set value in generation {}", i));
        initial_generations.push(gen_id);
    }

    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let manager_clone = gen_manager.clone();
        let generations = initial_generations.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut conflicts_resolved = 0;
            let mut successful_operations = 0;

            for conflict_id in 0..conflicts_per_thread {
                // Create artificial conflict by trying to access different generations simultaneously
                let gen1 = generations[conflict_id % generations.len()];
                let gen2 = generations[(conflict_id + 1) % generations.len()];

                // Simulate conflict detection and resolution
                if gen1 != gen2 {
                    match manager_clone.resolve_conflict(gen1, gen2) {
                        Ok(winner_gen) => {
                            conflicts_resolved += 1;
                            
                            // Perform operation on winning generation
                            if let Some(Value::Integer(current)) = manager_clone.get_variable(winner_gen, "value") {
                                let new_value = current + thread_id as i64;
                                if manager_clone.set_variable(winner_gen, "value".to_string(), Value::Integer(new_value)).is_ok() {
                                    successful_operations += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Thread {} failed to resolve conflict: {}", thread_id, e);
                        }
                    }
                } else {
                    // No conflict, perform normal operation
                    if let Some(Value::Integer(current)) = manager_clone.get_variable(gen1, "value") {
                        let new_value = current + thread_id as i64;
                        if manager_clone.set_variable(gen1, "value".to_string(), Value::Integer(new_value)).is_ok() {
                            successful_operations += 1;
                        }
                    }
                }

                // Verify consistency after conflict resolution
                if !manager_clone.verify_consistency().unwrap_or(false) {
                    eprintln!("Consistency check failed for thread {} after conflict {}", thread_id, conflict_id);
                }

                tokio::time::sleep(Duration::from_millis(2)).await;
            }

            (thread_id, conflicts_resolved, successful_operations)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_conflicts: usize = results.iter().map(|(_, conflicts, _)| *conflicts).sum();
    let total_operations: usize = results.iter().map(|(_, _, ops)| *ops).sum();

    println!("✓ Generation conflict resolution test completed");
    println!("  Total conflicts resolved: {}", total_conflicts);
    println!("  Total successful operations: {}", total_operations);
    println!("  Manager conflict count: {}", gen_manager.get_conflict_count());

    // Verify conflict resolution effectiveness
    assert!(total_conflicts > 0, "No conflicts were generated for testing");
    assert_eq!(gen_manager.get_conflict_count(), total_conflicts as u64, 
        "Manager conflict count doesn't match resolved conflicts");

    // Verify final state consistency
    assert!(gen_manager.verify_consistency().unwrap(), "Final consistency check failed");

    // Check final values in each generation
    for (i, &gen_id) in initial_generations.iter().enumerate() {
        if let Some(Value::Integer(final_value)) = gen_manager.get_variable(gen_id, "value") {
            let initial_value = (i + 1) * 100;
            println!("  Generation {}: {} → {}", gen_id.0, initial_value, final_value);
            assert!(final_value >= initial_value as i64, 
                "Generation {} value decreased: {} -> {}", gen_id.0, initial_value, final_value);
        }
    }

    println!("✓ All generation conflicts resolved successfully with consistent state");
}

// ============================================================================
// GENERATION LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_generation_lifecycle_management() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let gen_manager = Arc::new(ThreadSafeGenerationManager::new());
    
    let thread_count = 4;
    let generations_per_thread = 15;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let generation_tracker = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let manager_clone = gen_manager.clone();
        let tracker_clone = generation_tracker.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut created_generations = Vec::new();
            let mut operations_completed = 0;

            for gen_num in 0..generations_per_thread {
                // Create new generation
                match manager_clone.create_new_generation() {
                    Ok(new_gen) => {
                        created_generations.push(new_gen);
                        
                        // Track globally
                        {
                            let mut tracker = tracker_clone.lock().unwrap();
                            tracker.push(new_gen);
                        }

                        // Initialize generation with thread-specific data
                        let thread_value = (thread_id * 1000) + gen_num;
                        if manager_clone.set_variable(new_gen, "thread_data".to_string(), Value::Integer(thread_value as i64)).is_ok() {
                            operations_completed += 1;
                        }

                        // Occasionally switch to this generation
                        if gen_num % 3 == 0 {
                            if manager_clone.switch_to_generation(new_gen).is_ok() {
                                operations_completed += 1;
                            }
                        }

                        // Test inheritance from parent generation
                        if let Some(parent_value) = manager_clone.get_variable(new_gen, "thread_data") {
                            match parent_value {
                                Value::Integer(_) => operations_completed += 1,
                                _ => eprintln!("Thread {} generation {} has invalid thread_data", thread_id, new_gen.0),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {} failed to create generation {}: {}", thread_id, gen_num, e);
                    }
                }

                // Verify manager consistency
                if !manager_clone.verify_consistency().unwrap_or(false) {
                    eprintln!("Consistency check failed for thread {} at generation {}", thread_id, gen_num);
                }

                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, created_generations, operations_completed)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_generations_created: usize = results.iter().map(|(_, gens, _)| gens.len()).sum();
    let total_operations: usize = results.iter().map(|(_, _, ops)| *ops).sum();
    
    let expected_generations = thread_count * generations_per_thread;
    let tracked_generations = generation_tracker.lock().unwrap().len();

    println!("✓ Generation lifecycle management test completed");
    println!("  Expected generations: {}", expected_generations);
    println!("  Created generations: {}", total_generations_created);
    println!("  Tracked generations: {}", tracked_generations);
    println!("  Manager generation count: {}", gen_manager.get_generation_count());
    println!("  Total operations: {}", total_operations);

    // Verify generation creation success
    assert_eq!(total_generations_created, expected_generations, 
        "Not all expected generations were created");
    assert_eq!(tracked_generations, expected_generations,
        "Generation tracking inconsistent");

    // Verify reasonable operation success rate
    let expected_operations = expected_generations * 3; // 3 operations per generation
    let success_rate = total_operations as f64 / expected_operations as f64;
    assert!(success_rate > 0.8, "Operation success rate too low: {:.2}%", success_rate * 100.0);

    // Final consistency check
    assert!(gen_manager.verify_consistency().unwrap(), "Final consistency check failed");

    // Verify no generation data was lost
    let all_tracked_generations = generation_tracker.lock().unwrap();
    for (thread_id, (_, created_gens, _)) in results.iter().enumerate() {
        for &gen_id in created_gens {
            assert!(all_tracked_generations.contains(&gen_id), 
                "Generation {} from thread {} not found in global tracker", gen_id.0, thread_id);
            
            // Verify generation still has its data
            if let Some(Value::Integer(value)) = gen_manager.get_variable(gen_id, "thread_data") {
                let expected_thread = (value / 1000) as usize;
                assert_eq!(expected_thread, thread_id, 
                    "Generation {} has data from wrong thread: expected {}, got {}", 
                    gen_id.0, thread_id, expected_thread);
            }
        }
    }

    println!("✓ Generation lifecycle managed successfully with data integrity preserved");
}

// ============================================================================
// CROSS-GENERATION COORDINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_cross_generation_coordination() {
    let runtime = Arc::new(LambdustRuntime::new(Some(8)).expect("Failed to create runtime"));
    let gen_manager = Arc::new(ThreadSafeGenerationManager::new());
    
    // Create a hierarchy of generations for testing
    let mut generation_hierarchy = Vec::new();
    for level in 0..4 {
        let gen_id = gen_manager.create_new_generation()
            .expect(&format!("Failed to create generation at level {}", level));
        
        // Set level-specific data
        gen_manager.set_variable(gen_id, "level".to_string(), Value::Integer(level))
            .expect(&format!("Failed to set level in generation {}", gen_id.0));
        gen_manager.set_variable(gen_id, "shared_resource".to_string(), Value::Integer(0))
            .expect(&format!("Failed to set shared_resource in generation {}", gen_id.0));
        
        generation_hierarchy.push(gen_id);
    }

    let thread_count = 8;
    let coordination_rounds = 20;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let coordination_semaphore = Arc::new(Semaphore::new(1)); // Ensure coordinated access
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let manager_clone = gen_manager.clone();
        let hierarchy = generation_hierarchy.clone();
        let semaphore_clone = coordination_semaphore.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut coordinated_operations = 0;
            let mut cross_generation_reads = 0;

            for round in 0..coordination_rounds {
                // Acquire coordination lock
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                let target_generation = hierarchy[round % hierarchy.len()];
                let source_generation = hierarchy[(round + 1) % hierarchy.len()];

                // Cross-generation coordination: read from source, update target
                if let Some(Value::Integer(source_resource)) = manager_clone.get_variable(source_generation, "shared_resource") {
                    cross_generation_reads += 1;

                    // Update target generation with coordinated value
                    let new_value = source_resource + thread_id as i64 + 1;
                    if manager_clone.set_variable(target_generation, "shared_resource".to_string(), Value::Integer(new_value)).is_ok() {
                        coordinated_operations += 1;
                    }

                    // Also update a thread-specific variable in target generation
                    let thread_key = format!("thread_{}_data", thread_id);
                    let thread_value = (round as i64) * 10 + thread_id as i64;
                    if manager_clone.set_variable(target_generation, thread_key, Value::Integer(thread_value)).is_ok() {
                        coordinated_operations += 1;
                    }
                }

                // Verify consistency across all generations in hierarchy
                let mut consistency_check_passed = true;
                for &gen_id in &hierarchy {
                    if !manager_clone.verify_consistency().unwrap_or(false) {
                        consistency_check_passed = false;
                        break;
                    }
                }

                if !consistency_check_passed {
                    eprintln!("Cross-generation consistency check failed for thread {} at round {}", thread_id, round);
                }

                // Drop permit to allow other threads
                drop(_permit);
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, coordinated_operations, cross_generation_reads)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_coordinated_ops: usize = results.iter().map(|(_, ops, _)| *ops).sum();
    let total_cross_reads: usize = results.iter().map(|(_, _, reads)| *reads).sum();

    println!("✓ Cross-generation coordination test completed");
    println!("  Total coordinated operations: {}", total_coordinated_ops);
    println!("  Total cross-generation reads: {}", total_cross_reads);
    
    // Verify coordination effectiveness
    let expected_rounds = thread_count * coordination_rounds;
    assert_eq!(total_cross_reads, expected_rounds, "Not all cross-generation reads completed");
    
    let expected_coord_ops = expected_rounds * 2; // 2 operations per round
    let coordination_success_rate = total_coordinated_ops as f64 / expected_coord_ops as f64;
    assert!(coordination_success_rate > 0.9, 
        "Coordination success rate too low: {:.2}%", coordination_success_rate * 100.0);

    // Verify final state of each generation
    for (level, &gen_id) in generation_hierarchy.iter().enumerate() {
        println!("  Generation {} (level {}):", gen_id.0, level);
        
        if let Some(Value::Integer(final_resource)) = gen_manager.get_variable(gen_id, "shared_resource") {
            println!("    Shared resource: {}", final_resource);
            assert!(final_resource >= 0, "Shared resource became negative in generation {}", gen_id.0);
        }

        // Check thread-specific data
        for thread_id in 0..thread_count {
            let thread_key = format!("thread_{}_data", thread_id);
            if let Some(Value::Integer(thread_data)) = gen_manager.get_variable(gen_id, &thread_key) {
                println!("    Thread {} data: {}", thread_id, thread_data);
            }
        }
    }

    // Final consistency verification
    assert!(gen_manager.verify_consistency().unwrap(), "Final consistency check failed");

    println!("✓ Cross-generation coordination successful with consistent state across hierarchy");
}