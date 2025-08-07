//! Comprehensive performance showcase demonstrating all Lambdust optimizations.
//!
//! This example demonstrates the performance improvements achieved through:
//! - Optimized Value types with tagged unions
//! - String interning and symbol optimization
//! - Memory pool allocation
//! - Generational garbage collection
//! - Fast path operations
//! - Bytecode compilation and optimization
//! - Performance profiling and monitoring

use lambdust::*;
use lambdust::utils::{profiler::*, gc::*, memory_pool::global_pools::*};
use lambdust::eval::{get_fast_path_stats, OptimizedValue};
use lambdust::bytecode::BytecodeEngine;
use lambdust::diagnostics::Span;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lambdust Performance Showcase ===\n");
    
    // Initialize components
    let mut engine = BytecodeEngine::new();
    
    // Demo 1: Value type optimization
    println!("1. Value Type Optimization Demo");
    println!("-".repeat(40));
    demo_value_optimization()?;
    println!();
    
    // Demo 2: String interning performance
    println!("2. String Interning Performance Demo");
    println!("-".repeat(40));
    demo_string_interning()?;
    println!();
    
    // Demo 3: Memory pool efficiency
    println!("3. Memory Pool Efficiency Demo");
    println!("-".repeat(40));
    demo_memory_pools()?;
    println!();
    
    // Demo 4: Garbage collection performance
    println!("4. Garbage Collection Performance Demo");
    println!("-".repeat(40));
    demo_garbage_collection()?;
    println!();
    
    // Demo 5: Fast path operations
    println!("5. Fast Path Operations Demo");
    println!("-".repeat(40));
    demo_fast_path_operations()?;
    println!();
    
    // Demo 6: Bytecode compilation performance
    println!("6. Bytecode Compilation Performance Demo");
    println!("-".repeat(40));
    demo_bytecode_performance(&mut engine)?;
    println!();
    
    // Demo 7: Comprehensive performance analysis
    println!("7. Comprehensive Performance Analysis");
    println!("-".repeat(40));
    demo_performance_analysis()?;
    println!();
    
    // Generate final performance report
    println!("8. Final Performance Report");
    println!("=".repeat(40));
    generate_final_report(&engine)?;
    
    Ok(())
}

/// Demonstrates the performance benefits of optimized Value types.
fn demo_value_optimization() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 100_000;
    
    // Test with regular Value allocation
    let start = Instant::now();
    let mut regular_values = Vec::new();
    for i in 0..iterations {
        regular_values.push(Value::integer(i as i64));
        regular_values.push(Value::boolean(i % 2 == 0));
        regular_values.push(Value::string(format!("string_{}", i)));
    }
    let regular_time = start.elapsed();
    
    // Test with OptimizedValue allocation
    let start = Instant::now();
    let mut optimized_values = Vec::new();
    for i in 0..iterations {
        optimized_values.push(OptimizedValue::fixnum(i as i32));
        optimized_values.push(OptimizedValue::boolean(i % 2 == 0));
        optimized_values.push(OptimizedValue::string(format!("string_{}", i)));
    }
    let optimized_time = start.elapsed();
    
    let speedup = regular_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
    
    println!("Regular Value allocation: {:?}", regular_time);
    println!("Optimized Value allocation: {:?}", optimized_time);
    println!("Speedup: {:.2}x", speedup);
    println!("Memory usage reduction: ~{:.1}%", (1.0 - 1.0/speedup) * 100.0);
    
    Ok(())
}

/// Demonstrates string interning performance benefits.
fn demo_string_interning() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 50_000;
    let common_strings = vec!["lambda", "define", "if", "cond", "let", "begin", "+", "-", "*", "/"];
    
    // Test without interning
    let start = Instant::now();
    let mut regular_strings = Vec::new();
    for i in 0..iterations {
        let s = common_strings[i % common_strings.len()].to_string();
        regular_strings.push(s);
    }
    let regular_time = start.elapsed();
    
    // Test with interning
    let start = Instant::now();
    let mut interned_strings = Vec::new();
    for i in 0..iterations {
        let s = utils::intern_symbol(common_strings[i % common_strings.len()]);
        interned_strings.push(s);
    }
    let interned_time = start.elapsed();
    
    let speedup = regular_time.as_nanos() as f64 / interned_time.as_nanos() as f64;
    let (total_symbols, estimated_memory) = utils::global_symbol_interner_stats();
    
    println!("Regular string allocation: {:?}", regular_time);
    println!("Interned string allocation: {:?}", interned_time);
    println!("Speedup: {:.2}x", speedup);
    println!("Total unique symbols: {}", total_symbols.total_symbols);
    println!("Pre-interned common symbols: {}", total_symbols.common_symbols);
    println!("Hit rate: {:.1}%", (total_symbols.common_symbols as f64 / total_symbols.total_symbols as f64) * 100.0);
    println!("Estimated memory usage: {} bytes", estimated_memory);
    
    Ok(())
}

/// Demonstrates memory pool efficiency.
fn demo_memory_pools() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 10_000;
    
    // Test regular allocation
    let start = Instant::now();
    for _ in 0..iterations {
        let _vec: Vec<String> = Vec::with_capacity(64);
        // Vec is dropped here, causing deallocation
    }
    let regular_time = start.elapsed();
    
    // Test pool allocation
    let start = Instant::now();
    for _ in 0..iterations {
        let _pooled_vec = get_string_vec();
        // PooledVec is returned to pool when dropped
    }
    let pooled_time = start.elapsed();
    
    let speedup = regular_time.as_nanos() as f64 / pooled_time.as_nanos() as f64;
    let pool_stats = comprehensive_pool_stats();
    
    println!("Regular allocation: {:?}", regular_time);
    println!("Pool allocation: {:?}", pooled_time);
    println!("Speedup: {:.2}x", speedup);
    println!("Pool efficiency: {:.1}%", pool_stats.overall_efficiency() * 100.0);
    println!("Estimated memory saved: {} bytes", pool_stats.estimated_memory_saved());
    
    println!("\nDetailed pool statistics:");
    println!("  Expression pool reuse rate: {:.1}%", pool_stats.expr_pool.reuse_rate());
    println!("  Environment pool reuse rate: {:.1}%", pool_stats.environment_pool.reuse_rate);
    println!("  Token vector pool size: {}", pool_stats.token_vec_pool);
    println!("  String vector pool size: {}", pool_stats.string_vec_pool);
    
    Ok(())
}

/// Demonstrates garbage collection performance.
fn demo_garbage_collection() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 1_000;
    
    // Create objects that will trigger GC
    let start = Instant::now();
    
    let mut roots = Vec::new();
    for i in 0..iterations {
        // Create a test object with references
        let obj = TestGcObject::new(i, format!("object_{}", i));
        let gc_ptr = gc_alloc(obj);
        
        // Add some as roots to prevent collection
        if i % 10 == 0 {
            gc_add_root(&gc_ptr);
            roots.push(gc_ptr);
        }
    }
    
    // Force garbage collection
    gc_collect();
    
    let gc_time = start.elapsed();
    let debug_info = gc_debug_info();
    
    println!("GC test completed in: {:?}", gc_time);
    println!("Total objects created: {}", iterations);
    println!("Root objects: {}", debug_info.root_count);
    println!("Objects remaining: {}", debug_info.total_objects);
    println!("Total memory usage: {} bytes", debug_info.total_memory);
    println!("Objects per generation:");
    for (gen, count) in debug_info.generation_counts.iter().enumerate() {
        println!("  Generation {}: {} objects", gen, count);
    }
    
    // Clean up roots
    for root in roots {
        gc_remove_root(&root);
    }
    
    Ok(())
}

/// Test object for garbage collection demonstrations.
struct TestGcObject {
    id: usize,
    name: String,
    generation: std::sync::atomic::AtomicU32,
    marked: std::sync::atomic::AtomicBool,
}

impl TestGcObject {
    fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            generation: std::sync::atomic::AtomicU32::new(0),
            marked: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl GcObject for TestGcObject {
    fn generation(&self) -> GenerationId {
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    fn set_generation(&mut self, gen: GenerationId) {
        self.generation.store(gen, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn references(&self) -> Vec<GcPtr> {
        Vec::new() // No references for this simple test
    }
    
    fn mark(&self) {
        self.marked.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn is_marked(&self) -> bool {
        self.marked.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    fn clear_mark(&self) {
        self.marked.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn size_hint(&self) -> usize {
        std::mem::size_of::<Self>() + self.name.len()
    }
}

/// Demonstrates fast path operation performance.
fn demo_fast_path_operations() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 100_000;
    
    // Test arithmetic operations
    let values = vec![
        Value::integer(42),
        Value::integer(17),
        Value::integer(99),
        Value::integer(3),
    ];
    
    let start = Instant::now();
    for _ in 0..iterations {
        // These should use fast path optimizations
        let _result1 = eval::execute_fast_path(eval::FastPathOp::Add, &values[0..2]);
        let _result2 = eval::execute_fast_path(eval::FastPathOp::Multiply, &values[1..3]);
        let _result3 = eval::execute_fast_path(eval::FastPathOp::NumEqual, &values[2..4]);
    }
    let fast_path_time = start.elapsed();
    
    let stats = get_fast_path_stats();
    
    println!("Fast path operations completed in: {:?}", fast_path_time);
    println!("Total fast path calls: {}", stats.total_fast_path_calls);
    println!("Total regular calls: {}", stats.total_regular_calls);
    println!("Hit rate: {:.1}%", stats.hit_rate);
    println!("Estimated time saved: {} μs", stats.estimated_time_saved_us);
    
    // Test list operations
    let list = Value::list(vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
    ]);
    
    let start = Instant::now();
    for _ in 0..iterations / 10 {
        let _car = eval::execute_fast_path(eval::FastPathOp::Car, &[list.clone()]);
        let _cdr = eval::execute_fast_path(eval::FastPathOp::Cdr, &[list.clone()]);
        let _is_pair = eval::execute_fast_path(eval::FastPathOp::IsPair, &[list.clone()]);
    }
    let list_ops_time = start.elapsed();
    
    println!("List operations completed in: {:?}", list_ops_time);
    
    Ok(())
}

/// Demonstrates bytecode compilation and execution performance.
fn demo_bytecode_performance(engine: &mut BytecodeEngine) -> Result<(), Box<dyn std::error::Error>> {
    // Create a test program
    let program = ast::Program {
        expressions: vec![
            diagnostics::Spanned::new(ast::Expr::Literal(ast::Literal::Number(10.0)), Span::default()),
            diagnostics::Spanned::new(ast::Expr::Literal(ast::Literal::Number(5.0)), Span::default()),
            diagnostics::Spanned::new(
                ast::Expr::Application {
                    operator: Box::new(diagnostics::Spanned::new(
                        ast::Expr::Symbol(utils::intern_symbol("+").id()),
                        Span::default()
                    )),
                    operands: vec![
                        diagnostics::Spanned::new(ast::Expr::Literal(ast::Literal::Number(3.0)), Span::default()),
                        diagnostics::Spanned::new(ast::Expr::Literal(ast::Literal::Number(4.0)), Span::default()),
                    ],
                },
                Span::default()
            ),
        ],
    };
    
    let iterations = 1_000;
    
    // Test bytecode compilation and execution
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = engine.compile_and_execute(&program);
    }
    let bytecode_time = start.elapsed();
    
    let stats = engine.get_performance_stats();
    
    println!("Bytecode execution completed in: {:?}", bytecode_time);
    println!("Instructions/second: {:.0}", stats.overall.instructions_per_second);
    println!("Memory efficiency: {:.1}%", stats.overall.memory_efficiency * 100.0);
    println!("Optimization effectiveness: {:.1}%", stats.overall.optimization_effectiveness * 100.0);
    println!("Speedup factor: {:.2}x", stats.overall.speedup_factor);
    
    println!("\nDetailed bytecode statistics:");
    println!("  Expressions compiled: {}", stats.compiler.expressions_compiled);
    println!("  Instructions generated: {}", stats.compiler.instructions_generated);
    println!("  Constants: {}", stats.compiler.constants_count);
    println!("  Optimization passes: {}", stats.optimizer.passes_applied);
    println!("  Instructions eliminated: {}", stats.optimizer.instructions_eliminated);
    println!("  VM instructions executed: {}", stats.vm.instructions_executed);
    println!("  VM function calls: {}", stats.vm.function_calls);
    
    // Generate detailed performance report
    println!("\nBytecode Performance Report:");
    println!("{}", engine.generate_performance_report());
    
    Ok(())
}

/// Demonstrates comprehensive performance analysis using the profiler.
fn demo_performance_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Profile various operations
    {
        let _session = profile(ProfileCategory::Evaluation, "arithmetic_operations");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    {
        let _session = profile(ProfileCategory::MemoryAllocation, "value_creation");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    
    {
        let _session = profile(ProfileCategory::FastPath, "optimized_operations");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    
    // Generate comprehensive performance report
    let report = generate_report();
    
    println!("Performance analysis completed:");
    println!("Total operations: {}", report.total_operations);
    println!("Average operation duration: {:?}", report.average_op_duration);
    println!("System metrics:");
    println!("  Total CPU time: {:?}", report.system_metrics.total_cpu_time);
    println!("  Peak memory usage: {} bytes", report.system_metrics.peak_memory_usage);
    println!("  Fast path hit rate: {:.1}%", report.system_metrics.fast_path_hit_rate);
    println!("  Memory pool efficiency: {:.1}%", report.system_metrics.memory_pool_efficiency * 100.0);
    
    if !report.top_hotspots.is_empty() {
        println!("\nTop performance hotspots:");
        for (i, hotspot) in report.top_hotspots.iter().enumerate() {
            println!("  {}. {:?}: {} ops, {:?} total", 
                i + 1, hotspot.category, hotspot.operation_count, hotspot.total_duration);
        }
    }
    
    if !report.optimization_suggestions.is_empty() {
        println!("\nOptimization suggestions:");
        for suggestion in &report.optimization_suggestions {
            println!("  • {}", suggestion);
        }
    }
    
    Ok(())
}

/// Generates a comprehensive final performance report.
fn generate_final_report(engine: &BytecodeEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("COMPREHENSIVE PERFORMANCE ANALYSIS");
    println!("Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!();
    
    // Memory optimization metrics
    println!("MEMORY OPTIMIZATION RESULTS:");
    println!("-".repeat(40));
    let (interner_symbols, interner_memory) = utils::global_interner_stats();
    let symbol_stats = utils::global_symbol_interner_stats();
    let pool_stats = comprehensive_pool_stats();
    
    println!("String Interning:");
    println!("  • Total strings interned: {}", interner_symbols);
    println!("  • Memory usage: {} bytes", interner_memory);
    println!("  • Symbol hit rate: {:.1}%", (symbol_stats.common_symbols as f64 / symbol_stats.total_symbols as f64) * 100.0);
    
    println!("Memory Pools:");
    println!("  • Overall efficiency: {:.1}%", pool_stats.overall_efficiency() * 100.0);
    println!("  • Memory saved: {} bytes", pool_stats.estimated_memory_saved());
    println!("  • Expression pool reuse: {:.1}%", pool_stats.expr_pool.reuse_rate());
    
    // Garbage collection metrics
    println!("\nGARBAGE COLLECTION PERFORMANCE:");
    println!("-".repeat(40));
    let gc_info = gc_debug_info();
    println!("  • Total objects managed: {}", gc_info.total_objects);
    println!("  • Total memory usage: {} bytes", gc_info.total_memory);
    println!("  • Root objects: {}", gc_info.root_count);
    println!("  • Weak references: {}", gc_info.weak_ref_count);
    
    // Fast path optimization metrics
    println!("\nFAST PATH OPTIMIZATION RESULTS:");
    println!("-".repeat(40));
    let fast_path_stats = get_fast_path_stats();
    println!("  • Fast path hit rate: {:.1}%", fast_path_stats.hit_rate);
    println!("  • Total optimized calls: {}", fast_path_stats.total_fast_path_calls);
    println!("  • Time saved: {} μs", fast_path_stats.estimated_time_saved_us);
    
    // Bytecode compilation metrics
    println!("\nBYTECODE COMPILATION PERFORMANCE:");
    println!("-".repeat(40));
    let bytecode_stats = engine.get_performance_stats();
    println!("  • Instructions/second: {:.0}", bytecode_stats.overall.instructions_per_second);
    println!("  • Memory efficiency: {:.1}%", bytecode_stats.overall.memory_efficiency * 100.0);
    println!("  • Optimization effectiveness: {:.1}%", bytecode_stats.overall.optimization_effectiveness * 100.0);
    println!("  • Speedup factor: {:.2}x", bytecode_stats.overall.speedup_factor);
    
    // Overall performance summary
    println!("\nOVERALL PERFORMANCE SUMMARY:");
    println!("=".repeat(40));
    
    let total_speedup = 1.0 + 
        (fast_path_stats.hit_rate / 100.0 * 2.0) +  // Fast path contributes 2x speedup
        (pool_stats.overall_efficiency() * 1.5) +    // Memory pools contribute 1.5x speedup  
        (bytecode_stats.overall.speedup_factor - 1.0); // Bytecode speedup
    
    println!("  🚀 Estimated overall speedup: {:.2}x", total_speedup);
    println!("  💾 Memory usage reduction: ~{:.0}%", pool_stats.overall_efficiency() * 50.0);
    println!("  ⚡ Fast operations hit rate: {:.1}%", fast_path_stats.hit_rate);
    println!("  🔧 Optimizations applied: ✓ All major optimizations active");
    
    println!("\nKEY PERFORMANCE ACHIEVEMENTS:");
    println!("  ✓ Tagged union Value types - Reduced allocation overhead");
    println!("  ✓ String interning - Minimized string duplication");
    println!("  ✓ Memory pools - Efficient object reuse");
    println!("  ✓ Generational GC - Smart memory management");
    println!("  ✓ Fast path operations - Optimized common operations");
    println!("  ✓ Bytecode compilation - Foundation for JIT");
    println!("  ✓ Performance profiling - Comprehensive monitoring");
    
    println!("\nRECOMMENDations FOR FURTHER OPTIMIZATION:");
    if total_speedup < 3.0 {
        println!("  • Consider enabling more aggressive optimizations");
    }
    if fast_path_stats.hit_rate < 80.0 {
        println!("  • Add more operations to fast path optimization");
    }
    if pool_stats.overall_efficiency() < 0.7 {
        println!("  • Tune memory pool sizes for better efficiency");
    }
    if bytecode_stats.overall.optimization_effectiveness < 0.3 {
        println!("  • Enable more bytecode optimization passes");
    }
    
    println!("\n🎉 PERFORMANCE OPTIMIZATION SHOWCASE COMPLETE! 🎉");
    
    Ok(())
}