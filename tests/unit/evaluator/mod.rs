//! Evaluator unit tests module

pub mod adaptive_optimization_engine_tests;
pub mod compact_continuation_tests;
pub mod control_flow_tests;
pub mod dynamic_points_tests;
pub mod dynamic_wind_tests;
pub mod exceptions_tests;
pub mod expression_analyzer_tests;
pub mod phase5_raii_unified_tests;
pub mod phase6a_trampoline_tests;
// Phase 6-B-Step1: DoLoop specialized continuation tests
pub mod phase6b_doloop_continuation_tests;
// Phase 6-B-Step2: Unified continuation pooling tests
pub mod phase6b_continuation_pooling_tests;
// Phase 6-B-Step3: Inline evaluation system tests
pub mod phase6b_inline_evaluation_tests;
pub mod phase6c_jit_loop_tests;
// Runtime Executor JIT Integration tests
pub mod runtime_executor_jit_tests;
// Phase 6-D: Tail call optimization tests
pub mod phase6d_tail_call_tests;
// Phase 6-D: LLVM backend tests
pub mod phase6d_llvm_backend_tests;
pub mod special_forms_tests;
pub mod store_tests;
// AST Converter tests
pub mod ast_converter_tests;
// Import system tests
pub mod imports_tests;
// Semantic evaluator tests
pub mod semantic_evaluator_tests;
// Phase 3: EvaluatorInterface unified evaluation system tests
pub mod evaluator_interface_tests;
// Memory management tests
pub mod memory_tests;
// Theorem proving tests
pub mod theorem_proving_tests;
// Theorem proving test fixes
pub mod theorem_proving_test_fix;