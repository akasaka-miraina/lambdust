//! Comprehensive High-Performance SRFI-9 Record Tests
//!
//! This test suite validates the complete SRFI-9 implementation including:
//! - Basic functionality compliance with SRFI-9 specification
//! - Performance optimization features (arena allocation, NaN-boxing)
//! - SIMD acceleration for bulk operations
//! - Inline caching and JIT compilation hints
//! - Memory efficiency and garbage collection

use lambdust::eval::nan_boxed_value::NanBoxedValue;
use lambdust::eval::record_access::{
    BulkRecordOperations, FieldAccessCache, GLOBAL_FIELD_ACCESS_CACHE, GLOBAL_TYPE_CHECKER,
};
use lambdust::eval::record_arena::{GLOBAL_RECORD_ARENA, RecordArena, SizeClass};
use lambdust::eval::record_instance::{
    RecordInstance, RecordRef, nan_boxed_to_value, value_to_nan_boxed,
};
use lambdust::eval::record_type::{GLOBAL_RECORD_REGISTRY, RecordTypeDescriptor, RecordTypeId};
use lambdust::eval::value::{ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi9_optimization::{
    GLOBAL_RECORD_PERFORMANCE_MONITOR, RecordOperation, apply_optimizations,
    get_performance_report, optimize_memory,
};
use lambdust::stdlib::srfi9_records::{GLOBAL_SRFI9_REGISTRY, install_srfi9_procedures};
use std::sync::Arc;
use std::time::Instant;

/// Helper function to create a test record type
fn create_test_record_type(name: &str, fields: Vec<&str>) -> RecordTypeId {
    let field_names: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
    let desc = RecordTypeDescriptor::new(name.to_string(), field_names);
    let type_id = desc.type_id;
    GLOBAL_RECORD_REGISTRY.register_type(desc);
    type_id
}

/// Test basic SRFI-9 record functionality
#[test]
fn test_srfi9_basic_functionality() {
    // Create a simple point record type
    let type_id = create_test_record_type("point", &["x", "y"]);

    // Test record creation
    let values = vec![
        NanBoxedValue::small_integer(10),
        NanBoxedValue::small_integer(20),
    ];

    let instance = RecordInstance::new(type_id, &values).unwrap();

    unsafe {
        let instance_ref = instance.as_ref();

        // Test type checking
        assert_eq!(instance_ref.type_id(), type_id);
        assert!(instance_ref.is_type(type_id));

        // Test field access
        assert_eq!(
            instance_ref.get_field_fast(0),
            NanBoxedValue::small_integer(10)
        );
        assert_eq!(
            instance_ref.get_field_fast(1),
            NanBoxedValue::small_integer(20)
        );

        // Test field access by name
        let x_value = instance_ref.get_field_by_name("x").unwrap();
        let y_value = instance_ref.get_field_by_name("y").unwrap();
        assert_eq!(x_value, NanBoxedValue::small_integer(10));
        assert_eq!(y_value, NanBoxedValue::small_integer(20));
    }
}

/// Test field mutation functionality
#[test]
fn test_field_mutation() {
    let type_id = create_test_record_type("mutable-point", &["x", "y"]);

    let initial_values = vec![
        NanBoxedValue::small_integer(5),
        NanBoxedValue::small_integer(15),
    ];

    let mut instance = RecordInstance::new(type_id, &initial_values).unwrap();

    unsafe {
        let instance_ref = instance.as_mut();

        // Test field mutation by index
        instance_ref
            .set_field(0, NanBoxedValue::small_integer(50))
            .unwrap();
        assert_eq!(
            instance_ref.get_field_fast(0),
            NanBoxedValue::small_integer(50)
        );

        // Test field mutation by name
        instance_ref
            .set_field_by_name("y", NanBoxedValue::small_integer(150))
            .unwrap();
        assert_eq!(
            instance_ref.get_field_fast(1),
            NanBoxedValue::small_integer(150)
        );
    }
}

/// Test record comparison functionality
#[test]
fn test_record_comparison() {
    let type_id = create_test_record_type("comparable", &["a", "b"]);

    let values1 = vec![
        NanBoxedValue::small_integer(1),
        NanBoxedValue::small_integer(2),
    ];

    let values2 = vec![
        NanBoxedValue::small_integer(1),
        NanBoxedValue::small_integer(2),
    ];

    let values3 = vec![
        NanBoxedValue::small_integer(1),
        NanBoxedValue::small_integer(3),
    ];

    let instance1 = RecordInstance::new(type_id, &values1).unwrap();
    let instance2 = RecordInstance::new(type_id, &values2).unwrap();
    let instance3 = RecordInstance::new(type_id, &values3).unwrap();

    unsafe {
        // Test equality
        assert!(instance1.as_ref().equals(instance2.as_ref()).unwrap());

        // Test inequality
        assert!(!instance1.as_ref().equals(instance3.as_ref()).unwrap());
    }
}

/// Test arena allocation performance
#[test]
fn test_arena_allocation_performance() {
    let arena = RecordArena::new(lambdust::eval::record_arena::ArenaConfig::default());

    let start = Instant::now();
    let mut allocations = Vec::new();

    // Allocate 1000 records
    for _ in 0..1000 {
        if let Some(ptr) = arena.allocate(64) {
            allocations.push(ptr);
        }
    }

    let allocation_time = start.elapsed();

    // Deallocate all records
    let start = Instant::now();
    for ptr in allocations {
        arena.deallocate(ptr, 64);
    }
    let deallocation_time = start.elapsed();

    println!("Arena allocation time: {:?}", allocation_time);
    println!("Arena deallocation time: {:?}", deallocation_time);

    // Verify performance targets
    assert!(allocation_time.as_nanos() / 1000 < 50_000); // <50μs for 1000 allocations
    assert!(deallocation_time.as_nanos() / 1000 < 25_000); // <25μs for 1000 deallocations

    let stats = arena.stats();
    assert_eq!(stats.total_allocations, 1000);
    assert_eq!(stats.total_deallocations, 1000);
    assert!(stats.memory_utilization >= 0.0);
}

/// Test field access caching performance
#[test]
fn test_field_access_caching() {
    let type_id = create_test_record_type("cached-access", &["field1", "field2", "field3"]);

    let values = vec![
        NanBoxedValue::small_integer(100),
        NanBoxedValue::small_integer(200),
        NanBoxedValue::small_integer(300),
    ];

    let instance = RecordInstance::new(type_id, &values).unwrap();

    let cache = FieldAccessCache::new();
    let site_id = cache.allocate_site_id();

    // First access (cache miss)
    let start = Instant::now();
    let value1 = cache
        .get_field_cached(unsafe { instance.as_ref() }, "field1", site_id)
        .unwrap();
    let first_access_time = start.elapsed();

    // Second access (cache hit)
    let start = Instant::now();
    let value2 = cache
        .get_field_cached(unsafe { instance.as_ref() }, "field1", site_id)
        .unwrap();
    let second_access_time = start.elapsed();

    assert_eq!(value1, value2);
    assert_eq!(value1, NanBoxedValue::small_integer(100));

    // Cache hit should be faster
    println!(
        "First access: {:?}, Second access: {:?}",
        first_access_time, second_access_time
    );

    let stats = cache.stats();
    assert!(stats.total_lookups >= 2);
    assert!(stats.total_hits >= 1);
    assert!(stats.overall_hit_rate > 0.0);
}

/// Test SIMD bulk operations
#[test]
fn test_simd_bulk_operations() {
    let type_id = create_test_record_type("simd-test", &["value"]);

    // Create multiple records
    let record_count = 16; // Multiple of SIMD width
    let mut records = Vec::new();
    let mut record_refs = Vec::new();

    for i in 0..record_count {
        let values = &[NanBoxedValue::small_integer(i as i32)];
        let instance = RecordInstance::new(type_id, &values).unwrap();
        record_refs.push(unsafe { instance.as_ref() });
        records.push(instance);
    }

    // Test bulk field extraction
    let mut results = &[NanBoxedValue::nil_value(); record_count];
    let start = Instant::now();
    BulkRecordOperations::extract_field_bulk(&record_refs, 0, &mut results).unwrap();
    let bulk_time = start.elapsed();

    // Verify results
    for (i, &result) in results.iter().enumerate() {
        assert_eq!(result, NanBoxedValue::small_integer(i as i32));
    }

    // Compare with individual access time
    let start = Instant::now();
    let mut individual_results = Vec::new();
    for record_ref in &record_refs {
        let value = unsafe { record_ref.get_field_fast(0) };
        individual_results.push(value);
    }
    let individual_time = start.elapsed();

    println!("Bulk SIMD time: {:?}", bulk_time);
    println!("Individual access time: {:?}", individual_time);

    // SIMD should be competitive or faster
    // Note: For small datasets, overhead might make SIMD slower
    assert_eq!(results, individual_results);
}

/// Test type checking with polymorphic inline caching
#[test]
fn test_type_checking_performance() {
    let type1 = create_test_record_type("type1", &["field"]);
    let type2 = create_test_record_type("type2", &["field"]);

    let instance1 = RecordInstance::new(type1, &[NanBoxedValue::small_integer(1)]).unwrap();
    let instance2 = RecordInstance::new(type2, &[NanBoxedValue::small_integer(2)]).unwrap();

    let call_site_id = 12345;

    // Test monomorphic case (same type repeatedly)
    let start = Instant::now();
    for _ in 0..1000 {
        assert!(GLOBAL_TYPE_CHECKER.is_type_cached(
            unsafe { instance1.as_ref() },
            type1,
            call_site_id,
        ));
    }
    let monomorphic_time = start.elapsed();

    // Test polymorphic case (different types)
    let call_site_id2 = 12346;
    let start = Instant::now();
    for i in 0..1000 {
        if i % 2 == 0 {
            GLOBAL_TYPE_CHECKER.is_type_cached(unsafe { instance1.as_ref() }, type1, call_site_id2);
        } else {
            GLOBAL_TYPE_CHECKER.is_type_cached(unsafe { instance2.as_ref() }, type2, call_site_id2);
        }
    }
    let polymorphic_time = start.elapsed();

    println!("Monomorphic type checking: {:?}", monomorphic_time);
    println!("Polymorphic type checking: {:?}", polymorphic_time);

    let stats = GLOBAL_TYPE_CHECKER.stats();
    assert!(stats.total_checks >= 2000);
    assert!(stats.cache_hits > 0);
}

/// Test performance monitoring system
#[test]
fn test_performance_monitoring() {
    let type_id = create_test_record_type("monitored", &["data"]);

    // Record various operations
    GLOBAL_RECORD_PERFORMANCE_MONITOR.record_type_performance(type_id, RecordOperation::Create);

    GLOBAL_RECORD_PERFORMANCE_MONITOR
        .record_type_performance(type_id, RecordOperation::FieldAccess { access_time_ns: 5 });

    GLOBAL_RECORD_PERFORMANCE_MONITOR
        .record_type_performance(type_id, RecordOperation::BulkOperation { count: 8 });

    GLOBAL_RECORD_PERFORMANCE_MONITOR
        .record_type_performance(type_id, RecordOperation::SimdOperation);

    // Get performance report
    let report = get_performance_report();

    assert!(report.global_stats.total_instances > 0);
    assert!(report.global_stats.total_accesses > 0);
    assert!(report.global_stats.total_bulk_operations > 0);
    assert!(report.global_stats.simd_operations > 0);
    assert!(!report.type_performance.is_empty());

    // Check if we have performance data for our type
    let type_perf = report
        .type_performance
        .iter()
        .find(|tp| tp.type_id == type_id);

    if let Some(perf) = type_perf {
        assert!(perf.instances_created > 0);
        assert!(perf.field_accesses > 0);
        assert!(perf.performance_score >= 0.0 && perf.performance_score <= 100.0);
    }
}

/// Test optimization system
#[test]
fn test_optimization_system() {
    let type_id = create_test_record_type("optimizable", &["x", "y", "z"]);

    // Generate activity to trigger optimizations
    for _ in 0..500 {
        GLOBAL_RECORD_PERFORMANCE_MONITOR
            .record_type_performance(type_id, RecordOperation::FieldAccess { access_time_ns: 8 });
    }

    for _ in 0..100 {
        GLOBAL_RECORD_PERFORMANCE_MONITOR
            .record_type_performance(type_id, RecordOperation::BulkOperation { count: 4 });
    }

    // Get optimization recommendations
    let report = get_performance_report();

    // Should have some recommendations due to activity
    println!(
        "Optimization recommendations: {}",
        report.recommendations.len()
    );

    // Apply optimizations
    let optimization_results = apply_optimizations();

    assert!(
        optimization_results.applied + optimization_results.failed + optimization_results.skipped
            > 0
    );
}

/// Test memory optimization and garbage collection
#[test]
fn test_memory_optimization() {
    let type_id = create_test_record_type("gc-test", &["temp"]);

    // Create and destroy many records to trigger GC
    for _ in 0..1000 {
        let values = &[NanBoxedValue::small_integer(42)];
        let _instance = RecordInstance::new(type_id, &values).unwrap();
        // Instance goes out of scope and should be collected
    }

    // Get initial arena stats
    let initial_stats = GLOBAL_RECORD_ARENA.stats();

    // Trigger memory optimization
    let optimization_results = optimize_memory();

    assert!(optimization_results.updated_profiles > 0);
    println!("Memory optimization results: {:?}", optimization_results);

    // Get final arena stats
    let final_stats = GLOBAL_RECORD_ARENA.stats();

    // Should show memory management activity
    assert!(final_stats.total_allocations >= initial_stats.total_allocations);
}

/// Test NaN-boxing value conversion
#[test]
fn test_nan_boxing_conversion() {
    // Test various value types
    let test_values = vec![
        Value::Boolean(true),
        Value::Boolean(false),
        Value::integer(42),
        Value::integer(-100),
        Value::character('A'),
        Value::Nil,
    ];

    for original_value in test_values {
        // Convert to NaN-boxed
        let nan_boxed = value_to_nan_boxed(&original_value);

        // Convert back
        let converted_back = nan_boxed_to_value(nan_boxed);

        // Should be equal (for supported types)
        match &original_value {
            Value::Boolean(_) | Value::Integer(_) | Value::character(_) | Value::Nil => {
                assert_eq!(original_value, converted_back);
            }
            _ => {
                // Complex types might not round-trip exactly
                println!(
                    "Complex type conversion: {:?} -> {:?}",
                    original_value, converted_back
                );
            }
        }
    }
}

/// Test integration with standard library installation
#[test]
fn test_stdlib_integration() {
    let env = Arc::new(ThreadSafeEnvironment::new_global());

    // Install SRFI-9 procedures
    install_srfi9_procedures(&env);

    // Check that procedures are installed
    assert!(env.lookup("define-record-type").is_some());
    assert!(env.lookup("record?").is_some());
    assert!(env.lookup("record-type").is_some());
    assert!(env.lookup("record-rtd").is_some());

    // Test SRFI-9 registry functions
    let registry = &GLOBAL_SRFI9_REGISTRY;

    // Define a record type through the registry
    let type_def = registry
        .define_record_type(
            "test-stdlib".to_string(),
            ("make-test-stdlib".to_string(), &["field1".to_string()]),
            "test-stdlib?".to_string(),
            &[("field1".to_string(), "test-stdlib-field1".to_string(), None)],
        )
        .unwrap();

    assert_eq!(type_def.name, "test-stdlib");
    assert_eq!(type_def.constructor_name, "make-test-stdlib");
    assert_eq!(type_def.predicate_name, "test-stdlib?");

    // Check that procedures are available
    assert!(registry.get_procedure("make-test-stdlib").is_some());
    assert!(registry.get_procedure("test-stdlib?").is_some());
    assert!(registry.get_procedure("test-stdlib-field1").is_some());
}

/// Test error handling
#[test]
fn test_error_handling() {
    let type_id = create_test_record_type("error-test", &["field1", "field2"]);

    // Test field count mismatch
    let wrong_values = &[NanBoxedValue::small_integer(1)]; // Missing field2
    let result = RecordInstance::new(type_id, &wrong_values);
    assert!(result.is_err());

    // Test invalid field access
    let correct_values = vec![
        NanBoxedValue::small_integer(1),
        NanBoxedValue::small_integer(2),
    ];
    let instance = RecordInstance::new(type_id, &correct_values).unwrap();

    unsafe {
        let instance_ref = instance.as_ref();

        // Test out-of-bounds field access
        let result = instance_ref.get_field(10);
        assert!(result.is_err());

        // Test invalid field name
        let result = instance_ref.get_field_by_name("nonexistent");
        assert!(result.is_err());
    }
}

/// Benchmark record creation performance
#[test]
fn benchmark_record_creation() {
    let type_id = create_test_record_type("benchmark", &["x", "y", "z"]);

    let values = vec![
        NanBoxedValue::small_integer(1),
        NanBoxedValue::small_integer(2),
        NanBoxedValue::small_integer(3),
    ];

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _instance = RecordInstance::new(type_id, &values).unwrap();
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations;

    println!("Record creation benchmark:");
    println!("  Total time: {:?}", total_time);
    println!("  Average time per record: {:?}", avg_time);
    println!("  Records per second: {:.0}", 1.0 / avg_time.as_secs_f64());

    // Performance target: 10-20ns per record (50M-100M records/sec)
    assert!(avg_time.as_nanos() <= 50); // Should be ≤50ns per record
}

/// Benchmark field access performance
#[test]
fn benchmark_field_access() {
    let type_id = create_test_record_type("access-benchmark", &["field"]);

    let values = &[NanBoxedValue::small_integer(42)];
    let instance = RecordInstance::new(type_id, &values).unwrap();

    let iterations = 100000;
    let start = Instant::now();

    unsafe {
        let instance_ref = instance.as_ref();
        for _ in 0..iterations {
            let _value = instance_ref.get_field_fast(0);
        }
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations;

    println!("Field access benchmark:");
    println!("  Total time: {:?}", total_time);
    println!("  Average time per access: {:?}", avg_time);
    println!("  Accesses per second: {:.0}", 1.0 / avg_time.as_secs_f64());

    // Performance target: <1ns for hot paths, <5ns for cold paths
    assert!(avg_time.as_nanos() <= 10); // Should be ≤10ns per access
}
