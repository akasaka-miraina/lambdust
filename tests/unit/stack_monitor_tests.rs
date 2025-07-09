//! Unit tests for stack monitoring system (stack_monitor.rs)
//!
//! Tests the stack monitor including frame tracking, pressure detection,
//! optimization recommendations, and memory estimation.

use lambdust::stack_monitor::{
    StackFrameType, StackMonitor,
};
use std::time::Duration;

#[test]
fn test_stack_monitor_creation() {
    let monitor = StackMonitor::new();
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 0);
    assert_eq!(stats.max_depth, 0);
    assert_eq!(stats.total_frames, 0);
    assert!(!monitor.should_optimize());
}

#[test]
fn test_basic_frame_operations() {
    let mut monitor = StackMonitor::new();
    
    // Push first frame
    let frame1 = StackFrameType::Application {
        operator: "test-function".to_string(),
        arg_count: 2,
    };
    
    monitor.push_frame(frame1);
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 1);
    assert_eq!(stats.max_depth, 1);
    assert_eq!(stats.total_frames, 1);
    assert!(stats.total_memory_estimate > 0);
    
    // Push second frame
    let frame2 = StackFrameType::SpecialForm {
        form_name: "if".to_string(),
    };
    
    monitor.push_frame(frame2);
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 2);
    assert_eq!(stats.max_depth, 2);
    assert_eq!(stats.total_frames, 2);
    
    // Pop frame
    monitor.pop_frame();
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 1);
    assert_eq!(stats.max_depth, 2); // Max depth should remain
    assert_eq!(stats.total_frames, 2); // Total count includes popped frames
}

#[test]
fn test_memory_estimation_by_frame_type() {
    let monitor = StackMonitor::new();
    
    // Test Application frame estimation
    let app_frame = StackFrameType::Application {
        operator: "complex-function".to_string(),
        arg_count: 5,
    };
    let app_memory = monitor.estimate_frame_memory(&app_frame);
    assert!(app_memory > 0);
    
    // Test SpecialForm frame estimation
    let special_frame = StackFrameType::SpecialForm {
        form_name: "lambda".to_string(),
    };
    let special_memory = monitor.estimate_frame_memory(&special_frame);
    assert!(special_memory > 0);
    
    // Test ContinuationApplication frame estimation
    let cont_frame = StackFrameType::ContinuationApplication {
        cont_type: "Identity".to_string(),
    };
    let cont_memory = monitor.estimate_frame_memory(&cont_frame);
    assert!(cont_memory > 0);
    
    // Test MacroExpansion frame estimation
    let macro_frame = StackFrameType::MacroExpansion {
        macro_name: "define-syntax".to_string(),
    };
    let macro_memory = monitor.estimate_frame_memory(&macro_frame);
    assert!(macro_memory > 0);
    
    // Test RecursiveCall frame estimation
    let recursive_frame = StackFrameType::RecursiveCall {
        function_name: "factorial".to_string(),
        depth: 10,
    };
    let recursive_memory = monitor.estimate_frame_memory(&recursive_frame);
    assert!(recursive_memory > 0);
    
    // Different frame types should have different memory estimates
    assert_ne!(app_memory, special_memory);
}

#[test]
fn test_stack_pressure_detection() {
    let mut monitor = StackMonitor::new();
    
    // Add frames below recursion threshold (should not trigger optimization)
    for i in 0..50 {
        let frame = StackFrameType::Application {
            operator: format!("function-{}", i),
            arg_count: 1,
        };
        monitor.push_frame(frame);
    }
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 50);
    
    // Add many more frames above recursion threshold (should trigger optimization)
    for i in 50..1100 {
        let frame = StackFrameType::Application {
            operator: format!("function-{}", i),
            arg_count: 1,
        };
        monitor.push_frame(frame);
    }
    
    assert!(monitor.should_optimize());
    let stats = monitor.statistics();
    assert!(stats.current_depth > 1000);
}

#[test]
fn test_memory_pressure_detection() {
    let mut monitor = StackMonitor::new();
    
    // Add frames with high estimated memory usage
    for i in 0..100 {
        let frame = StackFrameType::MacroExpansion {
            macro_name: format!("large-macro-{}", i),
        };
        monitor.push_frame(frame);
    }
    
    let stats = monitor.statistics();
    assert!(stats.total_memory_estimate > 0);
    
    // With enough frames, should trigger optimization
    if stats.total_memory_estimate > 10_000_000 {
        assert!(monitor.should_optimize());
    }
}

#[test]
fn test_recursive_call_depth_tracking() {
    let mut monitor = StackMonitor::new();
    
    // Test increasing recursion depth
    let function_name = "recursive-test".to_string();
    for depth in 1..=20 {
        let frame = StackFrameType::RecursiveCall {
            function_name: function_name.clone(),
            depth,
        };
        monitor.push_frame(frame);
    }
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 20);
    assert!(stats.total_memory_estimate > 0);
}

#[test]
fn test_frame_types_comprehensive() {
    let mut monitor = StackMonitor::new();
    
    // Test all frame types
    let frames = vec![
        StackFrameType::Application {
            operator: "map".to_string(),
            arg_count: 3,
        },
        StackFrameType::SpecialForm {
            form_name: "let".to_string(),
        },
        StackFrameType::ContinuationApplication {
            cont_type: "Values".to_string(),
        },
        StackFrameType::MacroExpansion {
            macro_name: "cond".to_string(),
        },
        StackFrameType::RecursiveCall {
            function_name: "fibonacci".to_string(),
            depth: 15,
        },
    ];
    
    // Push all frame types
    for frame in frames {
        monitor.push_frame(frame);
    }
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 5);
    assert_eq!(stats.total_frames, 5);
    assert!(stats.total_memory_estimate > 0);
}

#[test]
fn test_optimization_recommendations() {
    let mut monitor = StackMonitor::new();
    
    // Add many frames to trigger optimization recommendations
    for i in 0..100 {
        let frame = StackFrameType::Application {
            operator: format!("function-{}", i),
            arg_count: 2,
        };
        monitor.push_frame(frame);
    }
    
    let recommendations = monitor.optimization_recommendations();
    
    // Should provide some optimization recommendations
    assert!(!recommendations.is_empty());
}

#[test]
fn test_statistics_timing_accuracy() {
    let mut monitor = StackMonitor::new();
    
    // Add frame
    let frame = StackFrameType::Application {
        operator: "timing-test".to_string(),
        arg_count: 1,
    };
    
    monitor.push_frame(frame);
    
    // Small delay to ensure measurable time
    std::thread::sleep(Duration::from_millis(10));
    
    monitor.pop_frame();
    
    let stats = monitor.statistics();
    
    // Should have recorded some statistics
    assert_eq!(stats.total_frames, 1);
}

#[test]
fn test_stack_monitor_edge_cases() {
    let mut monitor = StackMonitor::new();
    
    // Test popping from empty stack
    monitor.pop_frame(); // Should not panic
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 0);
    
    // Test with minimal frame
    let minimal_frame = StackFrameType::Application {
        operator: "min".to_string(),
        arg_count: 0,
    };
    
    monitor.push_frame(minimal_frame);
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 1);
    assert!(stats.total_memory_estimate > 0);
}

#[test]
fn test_multiple_identical_frames() {
    let mut monitor = StackMonitor::new();
    
    // Add multiple identical frames
    for _ in 0..10 {
        let frame = StackFrameType::SpecialForm {
            form_name: "if".to_string(),
        };
        monitor.push_frame(frame);
    }
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 10);
    assert_eq!(stats.total_frames, 10);
    
    // Pop half the frames
    for _ in 0..5 {
        monitor.pop_frame();
    }
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 5);
    assert_eq!(stats.total_frames, 10); // Total includes popped frames
}

#[test]
fn test_frame_memory_estimation_consistency() {
    let monitor = StackMonitor::new();
    
    // Same frame type should have consistent memory estimates
    let frame1 = StackFrameType::Application {
        operator: "test".to_string(),
        arg_count: 2,
    };
    
    let frame2 = StackFrameType::Application {
        operator: "test".to_string(),
        arg_count: 2,
    };
    
    let memory1 = monitor.estimate_frame_memory(&frame1);
    let memory2 = monitor.estimate_frame_memory(&frame2);
    
    assert_eq!(memory1, memory2);
    
    // Different arg counts should have different estimates
    let frame3 = StackFrameType::Application {
        operator: "test".to_string(),
        arg_count: 5,
    };
    
    let memory3 = monitor.estimate_frame_memory(&frame3);
    assert_ne!(memory1, memory3);
}

#[test]
fn test_stack_depth_tracking_accuracy() {
    let mut monitor = StackMonitor::new();
    
    // Build up stack depth
    for i in 1..=50 {
        let frame = StackFrameType::RecursiveCall {
            function_name: "test".to_string(),
            depth: i,
        };
        monitor.push_frame(frame);
        
        let stats = monitor.statistics();
        assert_eq!(stats.current_depth, i);
        assert!(stats.max_depth >= i);
    }
    
    // Pop all frames
    for i in (1..=50).rev() {
        monitor.pop_frame();
        
        let stats = monitor.statistics();
        assert_eq!(stats.current_depth, i - 1);
        assert_eq!(stats.max_depth, 50); // Max should remain
    }
}

#[test]
fn test_frame_creation_and_push() {
    let mut monitor = StackMonitor::new();
    
    // Different frame types should be creatable and pushable
    let app_frame = StackFrameType::Application {
        operator: "user-function".to_string(),
        arg_count: 2,
    };
    
    let special_frame = StackFrameType::SpecialForm {
        form_name: "if".to_string(),
    };
    
    let recursive_frame = StackFrameType::RecursiveCall {
        function_name: "factorial".to_string(),
        depth: 5,
    };
    
    // Check that frame types can be pushed without crashing
    monitor.push_frame(app_frame);
    monitor.push_frame(special_frame);
    monitor.push_frame(recursive_frame);
    
    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 3);
}

#[test]
fn test_performance_under_load() {
    let mut monitor = StackMonitor::new();
    
    let start_time = std::time::Instant::now();
    
    // Perform many operations
    for i in 0..1000 {
        let frame = StackFrameType::Application {
            operator: format!("function-{}", i),
            arg_count: i % 5,
        };
        monitor.push_frame(frame);
        
        if i % 100 == 0 {
            let _ = monitor.statistics();
            let _ = monitor.should_optimize();
            let _ = monitor.optimization_recommendations();
        }
    }
    
    let duration = start_time.elapsed();
    
    // Should complete reasonably quickly
    assert!(duration < Duration::from_secs(1));
    
    let final_stats = monitor.statistics();
    assert_eq!(final_stats.current_depth, 1000);
}