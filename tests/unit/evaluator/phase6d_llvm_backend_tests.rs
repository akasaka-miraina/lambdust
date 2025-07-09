//! Phase 6-D: LLVM Backend Tests for Tail Call Optimization
//!
//! Tests the LLVM backend implementation that provides compiler-level
//! tail call optimization using LLVM's native intrinsics and optimization passes.

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{
    LLVMCodeGenerator, LLVMCompilerIntegration, LLVMFunction, LLVMInstruction,
    LLVMOptimizationLevel, LLVMOptimizationStats, LLVMTailCallIntrinsic, TailCallContext,
};
use lambdust::lexer::SchemeNumber;

#[test]
fn test_llvm_instruction_creation() {
    let instruction = LLVMInstruction::new(
        "call".to_string(),
        vec!["@factorial".to_string(), "%arg1".to_string()],
    );

    assert_eq!(instruction.opcode, "call");
    assert_eq!(instruction.operands.len(), 2);
    assert!(instruction.result.is_none());
    assert!(instruction.attributes.is_empty());
}

#[test]
fn test_llvm_instruction_with_tail_call() {
    let instruction = LLVMInstruction::new(
        "call".to_string(),
        vec!["@factorial".to_string(), "%arg1".to_string()],
    )
    .with_tail_call()
    .with_result("%result".to_string());

    assert_eq!(instruction.opcode, "call");
    assert!(instruction.attributes.contains(&"tail".to_string()));
    assert_eq!(instruction.result, Some("%result".to_string()));
}

#[test]
fn test_llvm_instruction_with_musttail() {
    let instruction = LLVMInstruction::new(
        "call".to_string(),
        vec!["@factorial".to_string(), "%arg1".to_string()],
    )
    .with_musttail()
    .with_result("%result".to_string());

    assert!(instruction.attributes.contains(&"musttail".to_string()));
}

#[test]
fn test_llvm_instruction_ir_generation() {
    let instruction = LLVMInstruction::new(
        "call".to_string(),
        vec!["@factorial".to_string(), "%arg1".to_string()],
    )
    .with_tail_call()
    .with_result("%result".to_string());

    let ir = instruction.to_llvm_ir();
    assert!(ir.contains("%result ="));
    assert!(ir.contains("tail"));
    assert!(ir.contains("call"));
    assert!(ir.contains("@factorial"));
    assert!(ir.contains("%arg1"));
}

#[test]
fn test_llvm_function_creation() {
    let function = LLVMFunction::new("factorial".to_string(), "i8*".to_string());

    assert_eq!(function.name, "factorial");
    assert_eq!(function.return_type, "i8*");
    assert!(function.parameters.is_empty());
    assert!(function.body.is_empty());
    assert!(!function.uses_tail_calls);
}

#[test]
fn test_llvm_function_parameter_addition() {
    let mut function = LLVMFunction::new("factorial".to_string(), "i8*".to_string());
    function.add_parameter("i8*".to_string(), "n".to_string());

    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0], ("i8*".to_string(), "n".to_string()));
}

#[test]
fn test_llvm_function_instruction_addition() {
    let mut function = LLVMFunction::new("factorial".to_string(), "i8*".to_string());

    let instruction = LLVMInstruction::new(
        "call".to_string(),
        vec!["@factorial".to_string(), "%arg1".to_string()],
    )
    .with_tail_call();

    function.add_instruction(instruction);

    assert_eq!(function.body.len(), 1);
    assert!(function.uses_tail_calls);
}

#[test]
fn test_llvm_function_ir_generation() {
    let mut function = LLVMFunction::new("test".to_string(), "i8*".to_string());
    function.add_parameter("i8*".to_string(), "arg".to_string());

    let instruction = LLVMInstruction::new(
        "ret".to_string(),
        vec!["i8*".to_string(), "%arg".to_string()],
    );
    function.add_instruction(instruction);

    let ir = function.to_llvm_ir();
    assert!(ir.contains("define i8* @test(i8* %arg)"));
    assert!(ir.contains("ret i8*"));
}

#[test]
fn test_llvm_code_generator_creation() {
    let generator = LLVMCodeGenerator::new();

    assert_eq!(*generator.optimization_level(), LLVMOptimizationLevel::O2);
}

#[test]
fn test_llvm_code_generator_with_optimization_level() {
    let generator = LLVMCodeGenerator::with_optimization_level(LLVMOptimizationLevel::O3);

    assert_eq!(*generator.optimization_level(), LLVMOptimizationLevel::O3);
}

#[test]
fn test_llvm_code_generator_function_management() {
    let mut generator = LLVMCodeGenerator::new();

    // Start function
    let result = generator.start_function("test".to_string(), "i8*".to_string());
    assert!(result.is_ok());

    // Add parameter
    let result = generator.add_parameter("i8*".to_string(), "arg".to_string());
    assert!(result.is_ok());

    // Finish function
    let result = generator.finish_function();
    assert!(result.is_ok());
}

#[test]
fn test_llvm_code_generator_duplicate_function_error() {
    let mut generator = LLVMCodeGenerator::new();

    let result1 = generator.start_function("test".to_string(), "i8*".to_string());
    assert!(result1.is_ok());

    let result2 = generator.finish_function();
    assert!(result2.is_ok());

    // Try to define same function again
    let result3 = generator.start_function("test".to_string(), "i8*".to_string());
    assert!(result3.is_err());
}

#[test]
fn test_llvm_code_generator_literal_generation() {
    let mut generator = LLVMCodeGenerator::new();

    generator
        .start_function("test".to_string(), "i8*".to_string())
        .unwrap();

    let literal = Literal::Number(SchemeNumber::Integer(42));
    let result = generator.generate_literal(&literal);

    assert!(result.is_ok());
    let register = result.unwrap();
    assert!(register.starts_with("%r"));
}

#[test]
fn test_llvm_code_generator_variable_generation() {
    let mut generator = LLVMCodeGenerator::new();

    generator
        .start_function("test".to_string(), "i8*".to_string())
        .unwrap();

    let result = generator.generate_variable("x");

    assert!(result.is_ok());
    let register = result.unwrap();
    assert!(register.starts_with("%r"));
}

#[test]
fn test_llvm_code_generator_function_call_generation() {
    let mut generator = LLVMCodeGenerator::new();

    generator
        .start_function("test".to_string(), "i8*".to_string())
        .unwrap();

    let exprs = vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ];

    let context = TailCallContext::new();
    let result = generator.generate_function_call(&exprs, &context);

    assert!(result.is_ok());
    let register = result.unwrap();
    assert!(register.starts_with("%r"));
}

#[test]
fn test_llvm_code_generator_tail_call_optimization() {
    let mut generator = LLVMCodeGenerator::new();

    generator
        .start_function("factorial".to_string(), "i8*".to_string())
        .unwrap();

    let exprs = vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ];

    // Test with tail position context
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;

    let result = generator.generate_function_call(&exprs, &context);
    assert!(result.is_ok());
}

#[test]
fn test_llvm_code_generator_module_generation() {
    let generator = LLVMCodeGenerator::new();

    let module_ir = generator.generate_module();

    assert!(module_ir.contains("Lambdust LLVM Module"));
    assert!(module_ir.contains("target datalayout"));
    assert!(module_ir.contains("target triple"));
    assert!(module_ir.contains("declare i8* @scheme_alloc"));
}

#[test]
fn test_llvm_code_generator_complete_function_generation() {
    let mut generator = LLVMCodeGenerator::new();

    let params = vec!["n".to_string()];
    let body = Expr::Variable("n".to_string());

    let result = generator.generate_function("identity".to_string(), params, &body);

    assert!(result.is_ok());
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define i8* @identity"));
    assert!(llvm_ir.contains("i8* %n"));
    assert!(llvm_ir.contains("ret i8*"));
}

#[test]
fn test_llvm_code_generator_optimization_stats() {
    let mut generator = LLVMCodeGenerator::new();

    // Generate function with tail call
    generator
        .start_function("factorial".to_string(), "i8*".to_string())
        .unwrap();

    let tail_call =
        LLVMInstruction::new("call".to_string(), vec!["@factorial".to_string()]).with_tail_call();

    generator
        .add_instruction_to_current_function(tail_call)
        .unwrap();
    generator.finish_function().unwrap();

    let stats = generator.get_optimization_stats();
    assert_eq!(stats.total_functions, 1);
    assert_eq!(stats.tail_call_optimized_functions, 1);
    assert_eq!(stats.tail_call_instructions, 1);
    assert!(stats.tail_call_ratio() > 0.0);
}

#[test]
fn test_llvm_tail_call_intrinsic_creation() {
    let intrinsic = LLVMTailCallIntrinsic::new();

    let stats = intrinsic.get_stats();
    assert_eq!(stats.intrinsic_calls, 0);
    assert_eq!(stats.successful_optimizations, 0);
    assert_eq!(stats.failed_optimizations, 0);
}

#[test]
fn test_llvm_compiler_integration_creation() {
    let integration = LLVMCompilerIntegration::new();

    let stats = integration.get_stats();
    assert_eq!(stats.compilation_requests, 0);
    assert_eq!(stats.successful_compilations, 0);
    assert_eq!(stats.failed_compilations, 0);
}

#[test]
fn test_llvm_compiler_integration_with_optimization_level() {
    let integration = LLVMCompilerIntegration::with_optimization_level(LLVMOptimizationLevel::O3);

    // Integration should be created successfully with specified optimization level
    assert_eq!(integration.get_stats().compilation_requests, 0);
}

#[test]
fn test_llvm_compiler_integration_expression_compilation() {
    let mut integration = LLVMCompilerIntegration::new();

    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let context = TailCallContext::new();

    // This will fail because we need to start a function first
    let result = integration.compile_with_tail_calls(&expr, &context);
    assert!(result.is_err());

    let stats = integration.get_stats();
    assert_eq!(stats.compilation_requests, 1);
    assert_eq!(stats.failed_compilations, 1);
}

#[test]
fn test_llvm_compiler_integration_optimization_passes() {
    let mut integration = LLVMCompilerIntegration::new();

    let module_ir = "; Test module\ndefine i8* @test() {\n  ret i8* null\n}\n";
    let result = integration.run_optimization_passes(module_ir);

    assert!(result.is_ok());
    let optimized_ir = result.unwrap();
    assert!(optimized_ir.contains("Optimized with LLVM"));
    assert!(optimized_ir.contains("Tail call optimization enabled"));

    let stats = integration.get_stats();
    assert_eq!(stats.optimization_passes, 1);
}

#[test]
fn test_llvm_optimization_level_variants() {
    assert_ne!(LLVMOptimizationLevel::O0, LLVMOptimizationLevel::O1);
    assert_ne!(LLVMOptimizationLevel::O2, LLVMOptimizationLevel::O3);
    assert_ne!(LLVMOptimizationLevel::Os, LLVMOptimizationLevel::Oz);
}

#[test]
fn test_llvm_code_generator_clear() {
    let mut generator = LLVMCodeGenerator::new();

    // Generate some functions
    generator
        .start_function("test1".to_string(), "i8*".to_string())
        .unwrap();
    generator.finish_function().unwrap();

    generator
        .start_function("test2".to_string(), "i8*".to_string())
        .unwrap();
    generator.finish_function().unwrap();

    // Clear should remove all functions
    generator.clear();

    let stats = generator.get_optimization_stats();
    assert_eq!(stats.total_functions, 0);
}

#[test]
fn test_llvm_compiler_integration_statistics_reset() {
    let mut integration = LLVMCompilerIntegration::new();

    // Generate some activity
    let module_ir = "; Test";
    let _result = integration.run_optimization_passes(module_ir);

    // Reset statistics
    integration.reset_stats();

    let stats = integration.get_stats();
    assert_eq!(stats.compilation_requests, 0);
    assert_eq!(stats.optimization_passes, 0);
}

#[test]
fn test_llvm_optimization_stats_ratios() {
    let mut stats = LLVMOptimizationStats::default();

    // Initially should be 0.0
    assert_eq!(stats.tail_call_ratio(), 0.0);
    assert_eq!(stats.instruction_optimization_ratio(), 0.0);

    // Add some data
    stats.total_functions = 10;
    stats.tail_call_optimized_functions = 5;
    stats.total_instructions = 100;
    stats.tail_call_instructions = 20;
    stats.musttail_instructions = 10;

    assert_eq!(stats.tail_call_ratio(), 0.5);
    assert_eq!(stats.instruction_optimization_ratio(), 0.3);
}

#[test]
fn test_llvm_backend_integration_with_tail_call_optimizer() {
    let mut generator = LLVMCodeGenerator::new();

    // Test that the generator properly integrates with tail call optimizer
    generator
        .start_function("test".to_string(), "i8*".to_string())
        .unwrap();

    let exprs = vec![
        Expr::Variable("test".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ];

    let context = TailCallContext::new();
    let result = generator.generate_function_call(&exprs, &context);

    // Should generate successfully even if optimization fails
    assert!(result.is_ok());
}
