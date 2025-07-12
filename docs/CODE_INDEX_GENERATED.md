# Lambdust Code Index (Auto-Generated)

**Generated**: 2025-07-12 16:56:12  
**Total Files**: 398  
**Total Structs**: 722 (Public: 716)  
**Total Enums**: 291 (Public: 291)  
**Total Functions**: 1970 (Public: 147)  
**Total Methods**: 0  
**Total Lines**: 161,615  

## 📋 File Index

### 🏗️ Core Infrastructure

#### debug_environment.rs
**Lines**: 123 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### src/environment.rs
**Lines**: 777 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `SharedEnvironment` (Line 16)
- `MutableEnvironment` (Line 502)

---

#### src/error.rs
**Lines**: 563 | **Structs**: 4 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `SourcePosition` (Line 12)
- `SourceSpan` (Line 53)
- `StackFrame` (Line 112)
- ... and 1 more

**Public Enums**:
- `FrameType` (Line 123)
- `LambdustError` (Line 177)

---

#### src/lib.rs
**Lines**: 788 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Interpreter` (Line 556)
- `EmbeddedInterpreter` (Line 582)

---

#### src/value/continuation.rs
**Lines**: 24 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Continuation` (Line 9)
- `StackFrame` (Line 18)

---

#### src/value/conversions.rs
**Lines**: 157 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/conversions_tests.rs
**Lines**: 669 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/custom_predicates.rs
**Lines**: 238 | **Structs**: 2 | **Enums**: 0 | **Functions**: 3

**Public Structs**:
- `CustomPredicateInfo` (Line 14)
- `CustomPredicateRegistry` (Line 25)

**Key Public Functions**:
- `global_custom_predicate_registry()` (Line 133)
- `register_global_custom_predicate()` (Line 138)
- `evaluate_global_custom_predicate()` (Line 150)

---

#### src/value/display.rs
**Lines**: 247 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/equality.rs
**Lines**: 172 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/lazy_vector.rs
**Lines**: 374 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `MemoryStats` (Line 236)

**Public Enums**:
- `VectorStorage` (Line 18)

---

#### src/value/list.rs
**Lines**: 67 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/mod.rs
**Lines**: 95 | **Structs**: 0 | **Enums**: 1 | **Functions**: 0

**Public Enums**:
- `Value` (Line 39)

---

#### src/value/optimized.rs
**Lines**: 466 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `ShortStringData` (Line 40)
- `OptimizationStats` (Line 353)
- `ValueOptimizer` (Line 426)

**Public Enums**:
- `OptimizedValue` (Line 14)

---

#### src/value/pair.rs
**Lines**: 101 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PairData` (Line 12)

---

#### src/value/port.rs
**Lines**: 32 | **Structs**: 0 | **Enums**: 1 | **Functions**: 0

**Public Enums**:
- `Port` (Line 5)

---

#### src/value/predicates.rs
**Lines**: 136 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/value/procedure.rs
**Lines**: 182 | **Structs**: 0 | **Enums**: 1 | **Functions**: 0

**Public Enums**:
- `Procedure` (Line 10)

---

#### src/value/promise.rs
**Lines**: 31 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `Promise` (Line 10)

**Public Enums**:
- `PromiseState` (Line 17)

---

#### src/value/record.rs
**Lines**: 49 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `RecordType` (Line 7)
- `Record` (Line 20)

---

### 🚀 Evaluator System

#### src/evaluator/ast_converter.rs
**Lines**: 70 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/backward_compatibility.rs
**Lines**: 792 | **Structs**: 5 | **Enums**: 4 | **Functions**: 0

**Public Structs**:
- `LegacyEvaluatorAdapter` (Line 20)
- `MigrationStatistics` (Line 36)
- `CompatibilityMode` (Line 58)
- ... and 2 more

**Public Enums**:
- `ErrorHandlingStrategy` (Line 77)
- `MigrationType` (Line 112)
- `RiskLevel` (Line 131)
- ... and 1 more

---

#### src/evaluator/church_rosser_proof.rs
**Lines**: 2614 | **Structs**: 56 | **Enums**: 15 | **Functions**: 0

**Public Structs**:
- `ChurchRosserProofEngine` (Line 19)
- `ConfluenceVerifier` (Line 45)
- `TerminationVerifier` (Line 59)
- ... and 53 more

**Public Enums**:
- `ReductionRule` (Line 169)
- `DiamondProperty` (Line 243)
- `PositionStep` (Line 266)
- ... and 12 more

---

#### src/evaluator/combinators.rs
**Lines**: 688 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `CombinatorStats` (Line 41)

**Public Enums**:
- `CombinatorExpr` (Line 13)

---

#### src/evaluator/continuation.rs
**Lines**: 876 | **Structs**: 3 | **Enums**: 4 | **Functions**: 0

**Public Structs**:
- `EnvironmentRef` (Line 175)
- `DoLoopState` (Line 378)
- `DynamicPoint` (Line 462)

**Public Enums**:
- `LightContinuation` (Line 15)
- `CompactContinuation` (Line 125)
- `InlineContinuation` (Line 135)
- ... and 1 more

---

#### src/evaluator/continuation_pooling.rs
**Lines**: 688 | **Structs**: 4 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `PoolStatistics` (Line 87)
- `TypedContinuationPool` (Line 154)
- `ContinuationPoolManager` (Line 249)
- ... and 1 more

**Public Enums**:
- `ContinuationType` (Line 17)

---

#### src/evaluator/control_flow/call_cc.rs
**Lines**: 98 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `eval_call_cc()` (Line 14)

---

#### src/evaluator/control_flow/continuations.rs
**Lines**: 88 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `apply_control_flow_continuation()` (Line 11)

---

#### src/evaluator/control_flow/do_loops.rs
**Lines**: 456 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

**Key Public Functions**:
- `eval_do()` (Line 69)

---

#### src/evaluator/control_flow/doloop_continuation.rs
**Lines**: 416 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `DoLoopContinuationPool` (Line 210)

---

#### src/evaluator/control_flow/dynamic_wind.rs
**Lines**: 106 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `eval_dynamic_wind()` (Line 14)

---

#### src/evaluator/control_flow/exceptions.rs
**Lines**: 399 | **Structs**: 1 | **Enums**: 0 | **Functions**: 5

**Public Structs**:
- `GuardHandler` (Line 18)

**Key Public Functions**:
- `eval_raise()` (Line 37)
- `eval_with_exception_handler()` (Line 54)
- `eval_guard()` (Line 91)

---

#### src/evaluator/control_flow/mod.rs
**Lines**: 33 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/control_flow/multi_values.rs
**Lines**: 109 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

**Key Public Functions**:
- `eval_values()` (Line 14)
- `eval_call_with_values()` (Line 44)

---

#### src/evaluator/control_flow/promises.rs
**Lines**: 123 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

**Key Public Functions**:
- `eval_delay()` (Line 17)
- `eval_lazy()` (Line 39)
- `eval_force()` (Line 62)
- `eval_promise_predicate()` (Line 86)

---

#### src/evaluator/evaluation.rs
**Lines**: 29 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `ExceptionHandlerInfo` (Line 23)

**Public Enums**:
- `EvalOrder` (Line 12)

---

#### src/evaluator/evaluation_mode_selector.rs
**Lines**: 658 | **Structs**: 6 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `EvaluationModeSelector` (Line 11)
- `PerformanceStats` (Line 58)
- `ModeDecision` (Line 78)
- ... and 3 more

**Public Enums**:
- `ExpressionType` (Line 31)
- `PerformanceTrend` (Line 148)

---

#### src/evaluator/evaluator_interface.rs
**Lines**: 909 | **Structs**: 6 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `EvaluationConfig` (Line 36)
- `EvaluationResult` (Line 53)
- `PerformanceMetrics` (Line 72)
- ... and 3 more

**Public Enums**:
- `EvaluationMode` (Line 23)

---

#### src/evaluator/execution_context.rs
**Lines**: 931 | **Structs**: 15 | **Enums**: 8 | **Functions**: 0

**Public Structs**:
- `ExecutionContext` (Line 24)
- `StaticAnalysisResult` (Line 52)
- `VariableUsage` (Line 128)
- ... and 12 more

**Public Enums**:
- `StaticCallPattern` (Line 92)
- `VariableTypeHint` (Line 144)
- `OptimizationLevel` (Line 202)
- ... and 5 more

---

#### src/evaluator/expression_analyzer.rs
**Lines**: 1295 | **Structs**: 3 | **Enums**: 3 | **Functions**: 0

**Public Structs**:
- `AnalysisResult` (Line 16)
- `ExpressionAnalyzer` (Line 105)
- `OptimizationStats` (Line 1270)

**Public Enums**:
- `TypeHint` (Line 35)
- `EvaluationComplexity` (Line 60)
- `OptimizationHint` (Line 88)

---

#### src/evaluator/external_provers.rs
**Lines**: 810 | **Structs**: 5 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `ProverConfig` (Line 29)
- `ExternalVerificationResult` (Line 46)
- `AgdaProver` (Line 83)
- ... and 2 more

**Public Enums**:
- `ExternalProver` (Line 16)

---

#### src/evaluator/formal_verification.rs
**Lines**: 1535 | **Structs**: 18 | **Enums**: 10 | **Functions**: 0

**Public Structs**:
- `FormalVerificationEngine` (Line 23)
- `VerificationConfiguration` (Line 57)
- `VerificationStatistics` (Line 101)
- ... and 15 more

**Public Enums**:
- `VerificationDepth` (Line 88)
- `FormalVerificationStatus` (Line 182)
- `TheoremProvingStatus` (Line 218)
- ... and 7 more

---

#### src/evaluator/higher_order.rs
**Lines**: 771 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/hotpath_analysis.rs
**Lines**: 1474 | **Structs**: 32 | **Enums**: 12 | **Functions**: 0

**Public Structs**:
- `AdvancedHotPathDetector` (Line 26)
- `FrequencyTracker` (Line 54)
- `ExecutionRecord` (Line 70)
- ... and 29 more

**Public Enums**:
- `AllocationType` (Line 120)
- `MemoryAccessType` (Line 283)
- `StridePattern` (Line 302)
- ... and 9 more

---

#### src/evaluator/imports.rs
**Lines**: 129 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/inline_evaluation.rs
**Lines**: 656 | **Structs**: 2 | **Enums**: 3 | **Functions**: 0

**Public Structs**:
- `HotPathDetector` (Line 123)
- `InlineEvaluator` (Line 197)

**Public Enums**:
- `InlineHint` (Line 16)
- `InlineResult` (Line 27)
- `ContinuationWeight` (Line 36)

---

#### src/evaluator/jit_loop_optimization.rs
**Lines**: 1184 | **Structs**: 8 | **Enums**: 4 | **Functions**: 0

**Public Structs**:
- `JitHotPathDetector` (Line 123)
- `CompiledLoop` (Line 136)
- `LoopPatternAnalyzer` (Line 228)
- ... and 5 more

**Public Enums**:
- `LoopPattern` (Line 23)
- `JitHint` (Line 68)
- `IterationStrategy` (Line 81)
- ... and 1 more

---

#### src/evaluator/llvm_backend.rs
**Lines**: 796 | **Structs**: 8 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `LLVMInstruction` (Line 21)
- `LLVMFunction` (Line 109)
- `LLVMCodeGenerator` (Line 191)
- ... and 5 more

**Public Enums**:
- `LLVMOptimizationLevel` (Line 206)

---

#### src/evaluator/memory.rs
**Lines**: 453 | **Structs**: 3 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Store` (Line 59)
- `StoreStatistics` (Line 84)

---

#### src/evaluator/memory_tests.rs
**Lines**: 541 | **Structs**: 0 | **Enums**: 0 | **Functions**: 32

---

#### src/evaluator/migration_strategy.rs
**Lines**: 1190 | **Structs**: 21 | **Enums**: 10 | **Functions**: 0

**Public Structs**:
- `MigrationStrategy` (Line 17)
- `MigrationPhase` (Line 42)
- `MigrationCondition` (Line 70)
- ... and 18 more

**Public Enums**:
- `RiskLevel` (Line 380)
- `ConditionType` (Line 393)
- `CriterionType` (Line 410)
- ... and 7 more

---

#### src/evaluator/mod.rs
**Lines**: 1382 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `eval_with_formal_semantics()` (Line 1378)

---

#### src/evaluator/performance_measurement/analysis.rs
**Lines**: 739 | **Structs**: 17 | **Enums**: 16 | **Functions**: 0

**Public Structs**:
- `AnalysisEngine` (Line 20)
- `OptimizationEffectVerifier` (Line 42)
- `AnalysisConfiguration` (Line 55)
- ... and 14 more

**Public Enums**:
- `AnalysisDepth` (Line 77)
- `StatisticalMethod` (Line 90)
- `OutlierDetectionMethod` (Line 120)
- ... and 13 more

---

#### src/evaluator/performance_measurement/benchmarking.rs
**Lines**: 471 | **Structs**: 10 | **Enums**: 3 | **Functions**: 0

**Public Structs**:
- `BenchmarkSuite` (Line 18)
- `Benchmark` (Line 31)
- `MicroBenchmark` (Line 50)
- ... and 7 more

**Public Enums**:
- `BenchmarkType` (Line 69)
- `ExecutionOrder` (Line 141)
- `BenchmarkStatus` (Line 252)

---

#### src/evaluator/performance_measurement/configuration.rs
**Lines**: 366 | **Structs**: 7 | **Enums**: 7 | **Functions**: 0

**Public Structs**:
- `MeasurementConfiguration` (Line 11)
- `OutputConfiguration` (Line 42)
- `WarmupConfiguration` (Line 117)
- ... and 4 more

**Public Enums**:
- `OutputFormat` (Line 67)
- `OutputDestination` (Line 84)
- `VerbosityLevel` (Line 104)
- ... and 4 more

---

#### src/evaluator/performance_measurement/core_types.rs
**Lines**: 332 | **Structs**: 9 | **Enums**: 6 | **Functions**: 0

**Public Structs**:
- `MetricCollector` (Line 48)
- `MetricData` (Line 64)
- `SamplingConfiguration` (Line 104)
- ... and 6 more

**Public Enums**:
- `MetricType` (Line 15)
- `MetricValue` (Line 83)
- `SamplingStrategy` (Line 117)
- ... and 3 more

---

#### src/evaluator/performance_measurement/evaluator_comparison.rs
**Lines**: 593 | **Structs**: 8 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `EvaluatorComparison` (Line 19)
- `ComparisonResult` (Line 32)
- `EvaluationMetrics` (Line 51)
- ... and 5 more

**Public Enums**:
- `PerformanceCategory` (Line 100)
- `TrendDirection` (Line 166)

---

#### src/evaluator/performance_measurement/metrics.rs
**Lines**: 384 | **Structs**: 5 | **Enums**: 3 | **Functions**: 0

**Public Structs**:
- `MetricsManager` (Line 16)
- `MetricsConfiguration` (Line 32)
- `CollectorConfiguration` (Line 54)
- ... and 2 more

**Public Enums**:
- `StoragePolicy` (Line 70)
- `CompressionLevel` (Line 105)
- `CompressionAlgorithm` (Line 116)

---

#### src/evaluator/performance_measurement/mod.rs
**Lines**: 424 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PerformanceMeasurementSystem` (Line 87)
- `SystemStatistics` (Line 109)
- `ComprehensiveMeasurementResult` (Line 290)
- ... and 1 more

---

#### src/evaluator/performance_measurement/performance_reports.rs
**Lines**: 698 | **Structs**: 8 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `PerformanceReportGenerator` (Line 16)
- `ReportConfig` (Line 27)
- `PerformanceDataPoint` (Line 72)
- ... and 5 more

**Public Enums**:
- `ReportFormat` (Line 44)
- `ReportType` (Line 57)

---

#### src/evaluator/performance_measurement/practical_benchmarks.rs
**Lines**: 502 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PracticalBenchmarkSuite` (Line 16)
- `BenchmarkResult` (Line 27)
- `ComprehensiveBenchmarkResults` (Line 48)
- ... and 1 more

---

#### src/evaluator/performance_measurement/regression_detection.rs
**Lines**: 543 | **Structs**: 9 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `RegressionDetector` (Line 16)
- `PerformanceBaseline` (Line 31)
- `BaselineMetrics` (Line 48)
- ... and 6 more

**Public Enums**:
- `AlertSeverity` (Line 131)

---

#### src/evaluator/performance_measurement_system.rs
**Lines**: 71 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/pico_environment.rs
**Lines**: 243 | **Structs**: 1 | **Enums**: 0 | **Functions**: 6

**Public Structs**:
- `PicoFeatures` (Line 130)

**Key Public Functions**:
- `create_pico_initial_environment()` (Line 23)
- `pico_builtin_placeholder()` (Line 68)
- `get_pico_builtin_names()` (Line 74)
- `is_pico_builtin()` (Line 89)
- `get_pico_features()` (Line 95)

---

#### src/evaluator/pico_evaluator.rs
**Lines**: 955 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PicoEvaluator` (Line 41)

---

#### src/evaluator/raii_store.rs
**Lines**: 830 | **Structs**: 5 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `RaiiLocation` (Line 41)
- `RaiiStoreStatistics` (Line 138)
- `RaiiStoreManager` (Line 161)
- ... and 1 more

---

#### src/evaluator/runtime_executor.rs
**Lines**: 3456 | **Structs**: 20 | **Enums**: 8 | **Functions**: 0

**Public Structs**:
- `ExpressionAnalysisResult` (Line 67)
- `OptimizedTailCall` (Line 410)
- `JitCompiledCode` (Line 426)
- ... and 17 more

**Public Enums**:
- `RuntimeOptimizationLevel` (Line 31)
- `CallPattern` (Line 95)
- `ExecutionFrequency` (Line 123)
- ... and 5 more

---

#### src/evaluator/runtime_optimization/caching_and_dependencies.rs
**Lines**: 881 | **Structs**: 22 | **Enums**: 11 | **Functions**: 0

**Public Structs**:
- `OptimizationCache` (Line 19)
- `CacheEntry` (Line 38)
- `CacheMetadata` (Line 66)
- ... and 19 more

**Public Enums**:
- `CacheStrategy` (Line 85)
- `DependencyNodeType` (Line 221)
- `CircularDependencySeverity` (Line 253)
- ... and 8 more

---

#### src/evaluator/runtime_optimization/core_types.rs
**Lines**: 648 | **Structs**: 11 | **Enums**: 13 | **Functions**: 0

**Public Structs**:
- `OptimizationStrategy` (Line 15)
- `ApplicabilityCondition` (Line 108)
- `OptimizationImpact` (Line 216)
- ... and 8 more

**Public Enums**:
- `OptimizationStrategyType` (Line 40)
- `ConditionType` (Line 124)
- `ConditionPredicate` (Line 170)
- ... and 10 more

---

#### src/evaluator/runtime_optimization/mod.rs
**Lines**: 789 | **Structs**: 18 | **Enums**: 10 | **Functions**: 0

**Public Structs**:
- `CorrectnessGuarantor` (Line 52)
- `VerificationConfiguration` (Line 68)
- `VerificationRecord` (Line 100)
- ... and 15 more

**Public Enums**:
- `VerificationLevel` (Line 87)
- `VerificationResult` (Line 119)
- `MessageLevel` (Line 163)
- ... and 7 more

---

#### src/evaluator/runtime_optimization/optimization_manager.rs
**Lines**: 641 | **Structs**: 18 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `IntegratedOptimizationManager` (Line 19)
- `OptimizationStrategySelector` (Line 45)
- `OptimizationExecutor` (Line 65)
- ... and 15 more

**Public Enums**:
- `DiagnosticLevel` (Line 185)
- `TrendDirection` (Line 332)

---

#### src/evaluator/runtime_optimization/performance_monitoring.rs
**Lines**: 821 | **Structs**: 19 | **Enums**: 7 | **Functions**: 0

**Public Structs**:
- `OptimizationPerformanceMonitor` (Line 15)
- `ExecutionRecord` (Line 37)
- `MemoryUsageRecord` (Line 59)
- ... and 16 more

**Public Enums**:
- `AlertDestination` (Line 138)
- `AnomalyDetectionMethod` (Line 229)
- `AnomalySeverity` (Line 293)
- ... and 4 more

---

#### src/evaluator/runtime_optimization_integration.rs
**Lines**: 95 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic/mod.rs
**Lines**: 46 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic/semantic_builtins.rs
**Lines**: 11 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic/semantic_continuation.rs
**Lines**: 264 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic/semantic_core.rs
**Lines**: 426 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `SemanticEvaluator` (Line 21)
- `ReductionStats` (Line 37)

---

#### src/evaluator/semantic/semantic_reduction.rs
**Lines**: 307 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic/semantic_special_forms.rs
**Lines**: 340 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/semantic_correctness.rs
**Lines**: 851 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `CorrectnessProof` (Line 41)
- `SemanticCorrectnessProver` (Line 56)

**Public Enums**:
- `CorrectnessProperty` (Line 20)

---

#### src/evaluator/special_forms.rs
**Lines**: 1295 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/tail_call_optimization.rs
**Lines**: 878 | **Structs**: 7 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `TailCallContext` (Line 24)
- `TailCallAnalyzer` (Line 96)
- `FunctionSignature` (Line 105)
- ... and 4 more

**Public Enums**:
- `ArgEvaluationStrategy` (Line 452)
- `OptimizationLevel` (Line 465)

---

#### src/evaluator/theorem_proving.rs
**Lines**: 1151 | **Structs**: 15 | **Enums**: 9 | **Functions**: 0

**Public Structs**:
- `TheoremProvingSupport` (Line 14)
- `ProofState` (Line 27)
- `ProofGoal` (Line 40)
- ... and 12 more

**Public Enums**:
- `Statement` (Line 56)
- `GoalType` (Line 124)
- `ProofTermType` (Line 216)
- ... and 6 more

---

#### src/evaluator/theorem_proving_test_fix.rs
**Lines**: 209 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/theorem_proving_tests.rs
**Lines**: 421 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/evaluator/trampoline.rs
**Lines**: 953 | **Structs**: 0 | **Enums**: 2 | **Functions**: 0

**Public Enums**:
- `ContinuationThunk` (Line 20)
- `Bounce` (Line 63)

---

#### src/evaluator/typed_special_forms.rs
**Lines**: 701 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `TypedParameter` (Line 14)
- `TypedLambda` (Line 23)
- `TypedDefine` (Line 36)
- ... and 1 more

---

#### src/evaluator/types.rs
**Lines**: 1207 | **Structs**: 3 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `StoreStatisticsWrapper` (Line 67)
- `MemoryStrategy` (Line 102)
- `Evaluator` (Line 133)

---

#### src/evaluator/verification_system.rs
**Lines**: 874 | **Structs**: 6 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `VerificationConfig` (Line 22)
- `VerificationResult` (Line 56)
- `VerificationAnalysis` (Line 77)
- ... and 3 more

**Public Enums**:
- `VerificationStatus` (Line 41)

---

### 🎯 AST and Parsing

#### src/ast.rs
**Lines**: 287 | **Structs**: 0 | **Enums**: 2 | **Functions**: 0

**Public Enums**:
- `Expr` (Line 8)
- `Literal` (Line 33)

---

#### src/lexer.rs
**Lines**: 427 | **Structs**: 1 | **Enums**: 2 | **Functions**: 1

**Public Structs**:
- `Lexer` (Line 136)

**Public Enums**:
- `Token` (Line 9)
- `SchemeNumber` (Line 43)

**Key Public Functions**:
- `tokenize()` (Line 417)

---

#### src/parser.rs
**Lines**: 285 | **Structs**: 1 | **Enums**: 0 | **Functions**: 3

**Public Structs**:
- `Parser` (Line 12)

**Key Public Functions**:
- `parse()` (Line 210)
- `parse_multiple()` (Line 233)
- `parse_with_loop_detection()` (Line 243)

---

### 🧮 Memory Management

#### analyze_memory.rs
**Lines**: 85 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### benches/memory_optimization.rs
**Lines**: 157 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### examples/analyze_memory.rs
**Lines**: 118 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/string_memory_analysis.rs
**Lines**: 74 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### src/adaptive_memory.rs
**Lines**: 541 | **Structs**: 6 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `AdaptiveMemoryManager` (Line 93)
- `MemorySnapshot` (Line 108)
- `MemoryConfig` (Line 123)
- ... and 3 more

**Public Enums**:
- `MemoryPressure` (Line 68)
- `AllocationStrategy` (Line 81)

---

#### src/memory_pool.rs
**Lines**: 418 | **Structs**: 5 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `SymbolInterner` (Line 23)
- `ValuePool` (Line 63)
- `PoolStats` (Line 173)
- ... and 2 more

---

### 📝 Macro System

#### src/macros/builtin.rs
**Lines**: 629 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

**Key Public Functions**:
- `expand_let()` (Line 12)
- `expand_let_star()` (Line 68)
- `expand_letrec()` (Line 113)
- `expand_cond()` (Line 180)
- `expand_case()` (Line 248)
- ... and 3 more

---

#### src/macros/do_notation.rs
**Lines**: 487 | **Structs**: 3 | **Enums**: 1 | **Functions**: 1

**Public Structs**:
- `DoNotationExpander` (Line 12)
- `MonadInstance` (Line 22)
- `DoBlock` (Line 52)

**Public Enums**:
- `DoBinding` (Line 35)

**Key Public Functions**:
- `register_mdo_macro()` (Line 311)

---

#### src/macros/expander.rs
**Lines**: 548 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `MacroExpander` (Line 31)

---

#### src/macros/hygiene/context.rs
**Lines**: 575 | **Structs**: 3 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `ExpansionStats` (Line 13)
- `ExpansionContext` (Line 28)
- `ExpansionStack` (Line 336)

---

#### src/macros/hygiene/environment.rs
**Lines**: 483 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `SymbolCache` (Line 19)
- `HygienicEnvironment` (Line 63)
- `HygienicMacro` (Line 83)

**Public Enums**:
- `SymbolResolution` (Line 356)

---

#### src/macros/hygiene/generator.rs
**Lines**: 848 | **Structs**: 3 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `SymbolCache` (Line 32)
- `SymbolGenerator` (Line 45)
- `PerformanceStats` (Line 483)

**Public Enums**:
- `GenerationStrategy` (Line 19)
- `UseCase` (Line 500)

---

#### src/macros/hygiene/mod.rs
**Lines**: 28 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/macros/hygiene/renaming.rs
**Lines**: 1048 | **Structs**: 5 | **Enums**: 8 | **Functions**: 0

**Public Structs**:
- `CustomRenamingRule` (Line 39)
- `RenamingPattern` (Line 48)
- `SymbolRenamer` (Line 160)
- ... and 2 more

**Public Enums**:
- `RenamingStrategy` (Line 18)
- `PatternMatcher` (Line 65)
- `ScopeConstraint` (Line 80)
- ... and 5 more

---

#### src/macros/hygiene/symbol.rs
**Lines**: 312 | **Structs**: 3 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `MacroSite` (Line 50)
- `SourceLocation` (Line 94)
- `HygienicSymbol` (Line 105)

---

#### src/macros/hygiene/transformer.rs
**Lines**: 1264 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `TransformerMetrics` (Line 44)
- `HygienicSyntaxRulesTransformer` (Line 67)

**Public Enums**:
- `OptimizationLevel` (Line 22)

---

#### src/macros/mod.rs
**Lines**: 58 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `expand_macro()` (Line 44)

---

#### src/macros/mod_tests.rs
**Lines**: 1310 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/macros/pattern_matching.rs
**Lines**: 719 | **Structs**: 6 | **Enums**: 5 | **Functions**: 0

**Public Structs**:
- `SyntaxRule` (Line 145)
- `SyntaxCaseClause` (Line 157)
- `MatchResult` (Line 178)
- ... and 3 more

**Public Enums**:
- `TypePattern` (Line 10)
- `Pattern` (Line 35)
- `Template` (Line 91)
- ... and 2 more

---

#### src/macros/srfi46_ellipsis.rs
**Lines**: 949 | **Structs**: 5 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `MultiDimBinding` (Line 15)
- `NDimensionalArray` (Line 41)
- `EllipsisContext` (Line 50)
- ... and 2 more

**Public Enums**:
- `MultiDimValue` (Line 26)

---

#### src/macros/syntax_case.rs
**Lines**: 730 | **Structs**: 2 | **Enums**: 0 | **Functions**: 2

**Public Structs**:
- `SyntaxCaseTransformer` (Line 21)
- `SyntaxCaseMacro` (Line 589)

**Key Public Functions**:
- `parse_syntax_case_pattern()` (Line 631)
- `parse_syntax_case_template()` (Line 685)

---

#### src/macros/syntax_rules.rs
**Lines**: 487 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `SyntaxRulesTransformer` (Line 24)

---

#### src/macros/types.rs
**Lines**: 56 | **Structs**: 0 | **Enums**: 2 | **Functions**: 0

**Public Enums**:
- `Macro` (Line 12)
- `BindingValue` (Line 48)

---

### 🔧 Built-in Functions

#### src/builtins/arithmetic.rs
**Lines**: 1207 | **Structs**: 0 | **Enums**: 0 | **Functions**: 52

**Key Public Functions**:
- `register_arithmetic_functions()` (Line 14)

---

#### src/builtins/control_flow.rs
**Lines**: 135 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

**Key Public Functions**:
- `register_control_flow_functions()` (Line 8)

---

#### src/builtins/custom_predicates.rs
**Lines**: 304 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

**Key Public Functions**:
- `register_custom_predicate_functions()` (Line 9)

---

#### src/builtins/error_handling.rs
**Lines**: 51 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

**Key Public Functions**:
- `register_error_functions()` (Line 8)

---

#### src/builtins/higher_order.rs
**Lines**: 538 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

**Key Public Functions**:
- `register_higher_order_functions()` (Line 11)
- `map_implementation()` (Line 92)
- `for_each_implementation()` (Line 171)
- `apply_implementation()` (Line 246)
- `filter_implementation()` (Line 317)
- ... and 2 more

---

#### src/builtins/io.rs
**Lines**: 98 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

**Key Public Functions**:
- `register_io_functions()` (Line 9)

---

#### src/builtins/lazy.rs
**Lines**: 93 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

**Key Public Functions**:
- `register_lazy_functions()` (Line 13)
- `force_promise()` (Line 71)

---

#### src/builtins/list_ops.rs
**Lines**: 215 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

**Key Public Functions**:
- `register_list_functions()` (Line 9)

---

#### src/builtins/macro_expansion.rs
**Lines**: 418 | **Structs**: 3 | **Enums**: 0 | **Functions**: 6

**Public Structs**:
- `ExpansionResult` (Line 26)
- `ExpansionInfo` (Line 37)
- `MacroExpander` (Line 51)

**Key Public Functions**:
- `register_macro_expansion_functions()` (Line 18)

---

#### src/builtins/misc.rs
**Lines**: 192 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

**Key Public Functions**:
- `register_misc_functions()` (Line 8)

---

#### src/builtins/mod.rs
**Lines**: 56 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/builtins/predicates.rs
**Lines**: 124 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

**Key Public Functions**:
- `register_predicate_functions()` (Line 12)

---

#### src/builtins/srfi.rs
**Lines**: 158 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

**Key Public Functions**:
- `register_srfi_functions()` (Line 12)
- `extract_integer_from_number()` (Line 136)

---

#### src/builtins/store.rs
**Lines**: 168 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

**Key Public Functions**:
- `register_store_functions()` (Line 13)
- `collect_garbage_with_evaluator()` (Line 125)
- `set_memory_limit_with_evaluator()` (Line 130)

---

#### src/builtins/string_char.rs
**Lines**: 361 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

**Key Public Functions**:
- `register_string_char_functions()` (Line 14)

---

#### src/builtins/utils.rs
**Lines**: 506 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

**Key Public Functions**:
- `check_arity()` (Line 12)
- `check_arity_range()` (Line 20)
- `expect_number()` (Line 37)
- `expect_string()` (Line 47)
- `expect_two_strings()` (Line 54)
- ... and 11 more

---

#### src/builtins/vector.rs
**Lines**: 184 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

**Key Public Functions**:
- `register_vector_functions()` (Line 10)

---

### 🧪 Testing

#### tests/basic_formal_verification_test.rs
**Lines**: 182 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/begin_define_test.rs
**Lines**: 54 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### tests/formal_verification_basic_test.rs
**Lines**: 278 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/infinite_loop_detection_test.rs
**Lines**: 179 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/integration/bridge_tests.rs
**Lines**: 153 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/call_cc_deep_exit_tests.rs
**Lines**: 148 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/integration/error_handling_tests.rs
**Lines**: 202 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/evaluator_tests.rs
**Lines**: 231 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/exception_handling_tests.rs
**Lines**: 228 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/execution_context_integration_tests.rs
**Lines**: 474 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### tests/integration/expression_analyzer_integration_tests.rs
**Lines**: 345 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

---

#### tests/integration/integration_tests.rs
**Lines**: 85 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/mod.rs
**Lines**: 28 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/module_system_integration_tests.rs
**Lines**: 225 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/r7rs_compliance_tests.rs
**Lines**: 308 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi46_tests.rs
**Lines**: 536 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi_128_tests.rs
**Lines**: 184 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/integration/srfi_130_tests.rs
**Lines**: 243 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### tests/integration/srfi_13_tests.rs
**Lines**: 390 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi_1_tests.rs
**Lines**: 296 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi_69_tests.rs
**Lines**: 425 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi_97_tests.rs
**Lines**: 184 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/srfi_tests.rs
**Lines**: 153 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/syntax_case_tests.rs
**Lines**: 616 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/integration/syntax_rules_tests.rs
**Lines**: 471 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/lambda_arithmetic_test.rs
**Lines**: 39 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/lazy_vector_safety_test.rs
**Lines**: 151 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/mod.rs
**Lines**: 13 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/simple_lambda_test.rs
**Lines**: 39 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/adaptive_memory_tests.rs
**Lines**: 156 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/ast_tests.rs
**Lines**: 68 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### tests/unit/bridge_tests.rs
**Lines**: 55 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### tests/unit/bridge_trait_tests.rs
**Lines**: 1044 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/builtins/arithmetic_tests.rs
**Lines**: 932 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

---

#### tests/unit/builtins/custom_predicates_tests.rs
**Lines**: 356 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

---

#### tests/unit/builtins/error_handling_tests.rs
**Lines**: 566 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/builtins/higher_order_tests.rs
**Lines**: 713 | **Structs**: 0 | **Enums**: 0 | **Functions**: 19

---

#### tests/unit/builtins/io_tests.rs
**Lines**: 642 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

---

#### tests/unit/builtins/lazy_tests.rs
**Lines**: 55 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/builtins/list_ops_tests.rs
**Lines**: 787 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/builtins/misc_tests.rs
**Lines**: 868 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

---

#### tests/unit/builtins/mod.rs
**Lines**: 26 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/builtins/predicates_tests.rs
**Lines**: 856 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/builtins/srfi_13_tests.rs
**Lines**: 126 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/builtins/srfi_1_tests.rs
**Lines**: 86 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### tests/unit/builtins/srfi_69_tests.rs
**Lines**: 205 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/builtins/srfi_tests.rs
**Lines**: 77 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

---

#### tests/unit/builtins/store_tests.rs
**Lines**: 604 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/builtins/string_char_tests.rs
**Lines**: 1273 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/environment/cow_tests.rs
**Lines**: 354 | **Structs**: 0 | **Enums**: 0 | **Functions**: 20

---

#### tests/unit/environment/mod.rs
**Lines**: 4 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/environment_tests.rs
**Lines**: 107 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/error_handling_tests.rs
**Lines**: 772 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/evaluator/ast_converter_tests.rs
**Lines**: 450 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### tests/unit/evaluator/compact_continuation_tests.rs
**Lines**: 272 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### tests/unit/evaluator/control_flow_tests.rs
**Lines**: 416 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/evaluator/dynamic_points_tests.rs
**Lines**: 190 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/evaluator/dynamic_wind_tests.rs
**Lines**: 346 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/evaluator/evaluator_interface_tests.rs
**Lines**: 451 | **Structs**: 0 | **Enums**: 0 | **Functions**: 23

---

#### tests/unit/evaluator/exceptions_tests.rs
**Lines**: 629 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/evaluator/expression_analyzer_tests.rs
**Lines**: 723 | **Structs**: 0 | **Enums**: 0 | **Functions**: 20

---

#### tests/unit/evaluator/imports_tests.rs
**Lines**: 397 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/evaluator/mod.rs
**Lines**: 34 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/evaluator/phase5_raii_unified_tests.rs
**Lines**: 186 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### tests/unit/evaluator/phase6a_trampoline_tests.rs
**Lines**: 566 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/evaluator/phase6b_continuation_pooling_tests.rs
**Lines**: 565 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/evaluator/phase6b_doloop_continuation_tests.rs
**Lines**: 372 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/evaluator/phase6b_inline_evaluation_tests.rs
**Lines**: 355 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### tests/unit/evaluator/phase6c_jit_loop_tests.rs
**Lines**: 570 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/evaluator/phase6d_llvm_backend_tests.rs
**Lines**: 437 | **Structs**: 0 | **Enums**: 0 | **Functions**: 29

---

#### tests/unit/evaluator/phase6d_tail_call_tests.rs
**Lines**: 522 | **Structs**: 0 | **Enums**: 0 | **Functions**: 21

---

#### tests/unit/evaluator/runtime_executor_jit_tests.rs
**Lines**: 495 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/evaluator/semantic_evaluator_tests.rs
**Lines**: 618 | **Structs**: 0 | **Enums**: 0 | **Functions**: 32

---

#### tests/unit/evaluator/special_forms_tests.rs
**Lines**: 472 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/evaluator/store_tests.rs
**Lines**: 325 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/evaluator_import_tests.rs
**Lines**: 390 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/evaluator_tests.rs
**Lines**: 409 | **Structs**: 0 | **Enums**: 0 | **Functions**: 32

---

#### tests/unit/ffi_enhanced_tests.rs
**Lines**: 361 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### tests/unit/higher_order_tests.rs
**Lines**: 118 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### tests/unit/host_tests.rs
**Lines**: 139 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### tests/unit/interpreter_tests.rs
**Lines**: 132 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### tests/unit/lambda_integration_tests.rs
**Lines**: 145 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/lexer_tests.rs
**Lines**: 916 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/lib_tests.rs
**Lines**: 21 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/lsp/completion_tests.rs
**Lines**: 404 | **Structs**: 0 | **Enums**: 0 | **Functions**: 19

---

#### tests/unit/lsp/diagnostics_tests.rs
**Lines**: 430 | **Structs**: 0 | **Enums**: 0 | **Functions**: 20

---

#### tests/unit/lsp/document_tests.rs
**Lines**: 435 | **Structs**: 0 | **Enums**: 0 | **Functions**: 22

---

#### tests/unit/lsp/hover_tests.rs
**Lines**: 455 | **Structs**: 0 | **Enums**: 0 | **Functions**: 19

---

#### tests/unit/lsp/integration_tests.rs
**Lines**: 510 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

---

#### tests/unit/lsp/mod.rs
**Lines**: 166 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/lsp/position_tests.rs
**Lines**: 400 | **Structs**: 0 | **Enums**: 0 | **Functions**: 27

---

#### tests/unit/lsp/server_tests.rs
**Lines**: 419 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/lsp/symbols_tests.rs
**Lines**: 459 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

---

#### tests/unit/macros/hygienic_integration_tests.rs
**Lines**: 370 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/macros/mod.rs
**Lines**: 5 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/macros/nested_macro_tests.rs
**Lines**: 346 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### tests/unit/macros/performance_tests.rs
**Lines**: 241 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### tests/unit/macros_tests.rs
**Lines**: 236 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/marshal_tests.rs
**Lines**: 167 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/memory_pool_tests.rs
**Lines**: 135 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### tests/unit/memory_safety/concurrent_safety_tests.rs
**Lines**: 537 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/memory_safety/memory_leak_tests.rs
**Lines**: 431 | **Structs**: 0 | **Enums**: 0 | **Functions**: 16

---

#### tests/unit/memory_safety/memory_pressure_tests.rs
**Lines**: 511 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/memory_safety/mod.rs
**Lines**: 451 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/memory_safety/resource_exhaustion_tests.rs
**Lines**: 537 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/memory_safety/stack_overflow_tests.rs
**Lines**: 425 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/mod.rs
**Lines**: 44 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/module_system_tests.rs
**Lines**: 56 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### tests/unit/parser_tests.rs
**Lines**: 1103 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### tests/unit/performance_optimization_tests.rs
**Lines**: 220 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### tests/unit/phase_3c_optimization_tests.rs
**Lines**: 471 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/repl_tests.rs
**Lines**: 273 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/srfi/mod.rs
**Lines**: 20 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/srfi/mod_tests.rs
**Lines**: 34 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/srfi/registry_tests.rs
**Lines**: 29 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/srfi/srfi_128_tests.rs
**Lines**: 207 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

---

#### tests/unit/srfi/srfi_130_tests.rs
**Lines**: 368 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/srfi/srfi_134_tests.rs
**Lines**: 233 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### tests/unit/srfi/srfi_136_tests.rs
**Lines**: 218 | **Structs**: 0 | **Enums**: 0 | **Functions**: 14

---

#### tests/unit/srfi/srfi_139_tests.rs
**Lines**: 165 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

---

#### tests/unit/srfi/srfi_140_tests.rs
**Lines**: 184 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/srfi/srfi_141_tests.rs
**Lines**: 451 | **Structs**: 0 | **Enums**: 0 | **Functions**: 10

---

#### tests/unit/srfi/srfi_45_tests.rs
**Lines**: 45 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### tests/unit/srfi/srfi_46_tests.rs
**Lines**: 25 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### tests/unit/srfi/srfi_69_enhanced_tests.rs
**Lines**: 346 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### tests/unit/srfi/srfi_97_tests.rs
**Lines**: 70 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

---

#### tests/unit/srfi/srfi_9_tests.rs
**Lines**: 47 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### tests/unit/stack_monitor_tests.rs
**Lines**: 415 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/type_system/mod.rs
**Lines**: 229 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/type_system/polynomial_types_tests.rs
**Lines**: 408 | **Structs**: 0 | **Enums**: 0 | **Functions**: 20

---

#### tests/unit/type_system/type_system_integration_tests.rs
**Lines**: 514 | **Structs**: 0 | **Enums**: 0 | **Functions**: 13

---

#### tests/unit/type_system/universe_polymorphic_tests.rs
**Lines**: 579 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/value/custom_predicates_tests.rs
**Lines**: 326 | **Structs**: 0 | **Enums**: 0 | **Functions**: 15

---

#### tests/unit/value/mod.rs
**Lines**: 7 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### tests/unit/value/optimized_tests.rs
**Lines**: 418 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/value/procedure_tests.rs
**Lines**: 580 | **Structs**: 0 | **Enums**: 0 | **Functions**: 22

---

#### tests/unit/value/promise_tests.rs
**Lines**: 418 | **Structs**: 0 | **Enums**: 0 | **Functions**: 18

---

#### tests/unit/value/record_tests.rs
**Lines**: 403 | **Structs**: 0 | **Enums**: 0 | **Functions**: 22

---

#### tests/unit/value_tests.rs
**Lines**: 762 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

### 🔌 Host Integration

#### src/host.rs
**Lines**: 379 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `FunctionSignature` (Line 73)
- `HostFunctionRegistry` (Line 151)

**Public Enums**:
- `ValueType` (Line 17)

---

### 📊 Type System

#### src/type_system/dependent_types.rs
**Lines**: 118 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PiType` (Line 19)
- `SigmaType` (Line 30)

---

#### src/type_system/homotopy_types.rs
**Lines**: 67 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `HoTTType` (Line 8)
- `UnivalenceAxiom` (Line 32)

**Public Enums**:
- `HigherStructure` (Line 17)

---

#### src/type_system/hott_types.rs
**Lines**: 500 | **Structs**: 10 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `HITConstructor` (Line 52)
- `PathConstructor` (Line 63)
- `UnivalenceAxiom` (Line 75)
- ... and 7 more

**Public Enums**:
- `HoTTType` (Line 11)

---

#### src/type_system/incremental_inference.rs
**Lines**: 915 | **Structs**: 5 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `InferenceCacheEntry` (Line 18)
- `DependencyTracker` (Line 35)
- `IncrementalConfig` (Line 122)
- ... and 2 more

**Public Enums**:
- `CachePolicy` (Line 109)

---

#### src/type_system/mod.rs
**Lines**: 283 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `PolynomialUniverseSystem` (Line 48)

---

#### src/type_system/monad_algebra.rs
**Lines**: 515 | **Structs**: 5 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `MonadStructure` (Line 17)
- `MonadOperation` (Line 32)
- `DistributiveLaw` (Line 57)
- ... and 2 more

---

#### src/type_system/monad_transformers.rs
**Lines**: 851 | **Structs**: 19 | **Enums**: 9 | **Functions**: 0

**Public Structs**:
- `MonadTransformer` (Line 16)
- `TransformerParameter` (Line 33)
- `LiftOperation` (Line 46)
- ... and 16 more

**Public Enums**:
- `MonadTransformerType` (Line 57)
- `LiftImplementation` (Line 81)
- `DerivationRule` (Line 97)
- ... and 6 more

---

#### src/type_system/natural_models.rs
**Lines**: 65 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `NaturalModel` (Line 9)
- `UniverseFunction` (Line 20)

---

#### src/type_system/parallel_type_checker.rs
**Lines**: 785 | **Structs**: 7 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `TypeCheckTask` (Line 18)
- `TypeCheckResult` (Line 49)
- `TypeCheckError` (Line 62)
- ... and 4 more

**Public Enums**:
- `Priority` (Line 36)

---

#### src/type_system/polynomial_types.rs
**Lines**: 486 | **Structs**: 3 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `Constructor` (Line 50)
- `Parameter` (Line 61)
- `PolynomialFunctor` (Line 149)

**Public Enums**:
- `BaseType` (Line 27)
- `PolynomialType` (Line 70)

---

#### src/type_system/standard_transformers.rs
**Lines**: 942 | **Structs**: 0 | **Enums**: 0 | **Functions**: 45

**Key Public Functions**:
- `create_state_transformer()` (Line 13)
- `create_reader_transformer()` (Line 42)
- `create_writer_transformer()` (Line 69)
- `create_maybe_transformer()` (Line 97)
- `create_except_transformer()` (Line 116)
- ... and 2 more

---

#### src/type_system/standard_universe_classes.rs
**Lines**: 746 | **Structs**: 0 | **Enums**: 0 | **Functions**: 11

**Key Public Functions**:
- `create_functor_class()` (Line 11)
- `create_applicative_class()` (Line 203)
- `create_monad_class()` (Line 316)
- `initialize_standard_classes()` (Line 608)
- `create_list_functor_instance()` (Line 622)
- ... and 1 more

---

#### src/type_system/type_checker.rs
**Lines**: 315 | **Structs**: 3 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `TypeCheckResult` (Line 11)
- `TypeContext` (Line 22)
- `TypeChecker` (Line 61)

---

#### src/type_system/type_inference.rs
**Lines**: 466 | **Structs**: 4 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `InferenceContext` (Line 11)
- `TypeConstraint` (Line 24)
- `TypeSubstitution` (Line 46)
- ... and 1 more

**Public Enums**:
- `ConstraintKind` (Line 35)

---

#### src/type_system/universe_levels.rs
**Lines**: 278 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `UniverseHierarchy` (Line 11)

---

#### src/type_system/universe_polymorphic_classes.rs
**Lines**: 686 | **Structs**: 13 | **Enums**: 7 | **Functions**: 0

**Public Structs**:
- `UniversePolymorphicClass` (Line 13)
- `UniversePolymorphicParameter` (Line 30)
- `UniversePolymorphicMethod` (Line 74)
- ... and 10 more

**Public Enums**:
- `UniverseConstraint` (Line 41)
- `KindConstraint` (Line 61)
- `UniversePolymorphicType` (Line 87)
- ... and 4 more

---

### 🎨 SRFI Implementation

#### src/srfi/mod.rs
**Lines**: 142 | **Structs**: 1 | **Enums**: 0 | **Functions**: 1

**Public Structs**:
- `SrfiImport` (Line 56)

**Key Public Functions**:
- `parse_srfi_import()` (Line 89)

---

#### src/srfi/registry.rs
**Lines**: 167 | **Structs**: 1 | **Enums**: 0 | **Functions**: 1

**Public Structs**:
- `SrfiRegistry` (Line 9)

---

#### src/srfi/srfi_1.rs
**Lines**: 486 | **Structs**: 0 | **Enums**: 0 | **Functions**: 24

**Key Public Functions**:
- `register_srfi_1_functions()` (Line 11)
- `take()` (Line 83)
- `drop()` (Line 125)
- `concatenate()` (Line 171)
- `delete_duplicates()` (Line 192)

---

#### src/srfi/srfi_111.rs
**Lines**: 199 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Box` (Line 17)

---

#### src/srfi/srfi_113.rs
**Lines**: 473 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Set` (Line 15)
- `Bag` (Line 126)

---

#### src/srfi/srfi_125.rs
**Lines**: 260 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/srfi/srfi_128.rs
**Lines**: 526 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Comparator` (Line 19)

---

#### src/srfi/srfi_13/comparison.rs
**Lines**: 209 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

**Key Public Functions**:
- `register_functions()` (Line 10)
- `string_compare()` (Line 31)
- `string_compare_ci()` (Line 72)
- `string_hash()` (Line 115)
- `string_hash_ci()` (Line 167)

---

#### src/srfi/srfi_13/constructors.rs
**Lines**: 88 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

**Key Public Functions**:
- `register_functions()` (Line 11)
- `string_every()` (Line 37)
- `string_any()` (Line 68)

---

#### src/srfi/srfi_13/joining.rs
**Lines**: 105 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

**Key Public Functions**:
- `register_functions()` (Line 10)
- `string_concatenate()` (Line 36)

---

#### src/srfi/srfi_13/mod.rs
**Lines**: 133 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `register_srfi_13_functions()` (Line 24)

---

#### src/srfi/srfi_13/modification.rs
**Lines**: 329 | **Structs**: 0 | **Enums**: 0 | **Functions**: 17

**Key Public Functions**:
- `register_functions()` (Line 11)
- `string_take()` (Line 54)
- `string_drop()` (Line 98)
- `string_take_right()` (Line 142)
- `string_drop_right()` (Line 188)

---

#### src/srfi/srfi_13/search.rs
**Lines**: 167 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

**Key Public Functions**:
- `register_functions()` (Line 11)

---

#### src/srfi/srfi_130.rs
**Lines**: 539 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `StringCursor` (Line 14)

---

#### src/srfi/srfi_132.rs
**Lines**: 318 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### src/srfi/srfi_133.rs
**Lines**: 372 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/srfi/srfi_134.rs
**Lines**: 511 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Ideque` (Line 15)

---

#### src/srfi/srfi_135.rs
**Lines**: 763 | **Structs**: 0 | **Enums**: 1 | **Functions**: 2

**Public Enums**:
- `Text` (Line 16)

**Key Public Functions**:
- `textual_to_text()` (Line 248)
- `textual_to_string()` (Line 259)

---

#### src/srfi/srfi_136.rs
**Lines**: 619 | **Structs**: 4 | **Enums**: 0 | **Functions**: 8

**Public Structs**:
- `RecordTypeDescriptor` (Line 16)
- `FieldSpec` (Line 31)
- `ExtendedRecordType` (Line 96)
- ... and 1 more

---

#### src/srfi/srfi_137.rs
**Lines**: 327 | **Structs**: 1 | **Enums**: 0 | **Functions**: 5

**Public Structs**:
- `UniqueTypeInstance` (Line 16)

---

#### src/srfi/srfi_138.rs
**Lines**: 532 | **Structs**: 4 | **Enums**: 2 | **Functions**: 3

**Public Structs**:
- `CompilationContext` (Line 45)
- `CompiledCode` (Line 100)
- `SchemeCompiler` (Line 148)
- ... and 1 more

**Public Enums**:
- `CompilationTarget` (Line 15)
- `OptimizationLevel` (Line 30)

---

#### src/srfi/srfi_139.rs
**Lines**: 246 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### src/srfi/srfi_140.rs
**Lines**: 597 | **Structs**: 0 | **Enums**: 1 | **Functions**: 10

**Public Enums**:
- `IString` (Line 15)

---

#### src/srfi/srfi_141.rs
**Lines**: 318 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/srfi/srfi_45.rs
**Lines**: 168 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### src/srfi/srfi_46.rs
**Lines**: 58 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/srfi/srfi_69/conversion.rs
**Lines**: 139 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

**Key Public Functions**:
- `register_functions()` (Line 12)
- `hash_table_to_alist()` (Line 35)
- `alist_to_hash_table()` (Line 72)
- `hash_table_copy()` (Line 120)

---

#### src/srfi/srfi_69/core.rs
**Lines**: 178 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

**Key Public Functions**:
- `register_functions()` (Line 12)
- `make_hash_table()` (Line 34)
- `hash_table_ref()` (Line 71)
- `hash_table_ref_default()` (Line 105)
- `hash_table_set()` (Line 126)
- ... and 1 more

---

#### src/srfi/srfi_69/hash_functions.rs
**Lines**: 181 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

**Key Public Functions**:
- `register_functions()` (Line 10)
- `hash_value()` (Line 27)
- `string_hash_impl()` (Line 86)
- `string_ci_hash_impl()` (Line 133)

---

#### src/srfi/srfi_69/higher_order.rs
**Lines**: 128 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

**Key Public Functions**:
- `register_functions()` (Line 11)
- `hash_table_walk()` (Line 28)
- `hash_table_fold()` (Line 60)
- `hash_table_merge()` (Line 93)

---

#### src/srfi/srfi_69/mod.rs
**Lines**: 171 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

**Key Public Functions**:
- `register_srfi_69_functions()` (Line 29)

---

#### src/srfi/srfi_69/queries.rs
**Lines**: 146 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

**Key Public Functions**:
- `register_functions()` (Line 12)
- `hash_table_exists()` (Line 36)
- `hash_table_size()` (Line 64)
- `hash_table_keys()` (Line 93)
- `hash_table_values()` (Line 125)

---

#### src/srfi/srfi_69/types.rs
**Lines**: 151 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `HashTable` (Line 11)

**Public Enums**:
- `HashKey` (Line 24)

---

#### src/srfi/srfi_9.rs
**Lines**: 229 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### src/srfi/srfi_97.rs
**Lines**: 224 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

### 🛠️ Utilities

#### benches/performance_benchmark.rs
**Lines**: 174 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### benches/performance_optimization.rs
**Lines**: 150 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### benchmark.rs
**Lines**: 105 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### debug_continuations.rs
**Lines**: 40 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### debug_simple.rs
**Lines**: 50 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### debug_srfi_136.rs
**Lines**: 56 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### debug_trace_test.rs
**Lines**: 51 | **Structs**: 0 | **Enums**: 0 | **Functions**: 2

---

#### examples/advanced_formal_verification_demo.rs
**Lines**: 494 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/advanced_hygienic_transformer_demo.rs
**Lines**: 422 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/advanced_symbol_generator_demo.rs
**Lines**: 255 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/advanced_symbol_renamer_demo.rs
**Lines**: 411 | **Structs**: 0 | **Enums**: 0 | **Functions**: 9

---

#### examples/benchmark.rs
**Lines**: 128 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/bridge_example.rs
**Lines**: 260 | **Structs**: 2 | **Enums**: 0 | **Functions**: 1

---

#### examples/control_flow_demo.rs
**Lines**: 115 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/cow_environment_demo.rs
**Lines**: 200 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/custom_predicates_demo.rs
**Lines**: 383 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### examples/error_reporting_demo.rs
**Lines**: 238 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/execution_context_demo.rs
**Lines**: 245 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/execution_context_performance_demo.rs
**Lines**: 420 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### examples/formal_verification_demo.rs
**Lines**: 421 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/ghc_benchmark_demo.rs
**Lines**: 381 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/hygienic_macro_demo.rs
**Lines**: 114 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### examples/hygienic_when_macro.rs
**Lines**: 149 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

---

#### examples/incremental_inference_demo.rs
**Lines**: 234 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### examples/macro_expand_builtin_test.rs
**Lines**: 221 | **Structs**: 0 | **Enums**: 0 | **Functions**: 12

---

#### examples/macro_expand_demo.rs
**Lines**: 154 | **Structs**: 0 | **Enums**: 0 | **Functions**: 6

---

#### examples/mdo_syntax_demo.rs
**Lines**: 167 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/monad_transformer_demo.rs
**Lines**: 491 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### examples/new_architecture_demo.rs
**Lines**: 110 | **Structs**: 0 | **Enums**: 0 | **Functions**: 5

---

#### examples/performance_cow_demo.rs
**Lines**: 86 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/performance_demo.rs
**Lines**: 111 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/performance_final_demo.rs
**Lines**: 117 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/performance_report_demo.rs
**Lines**: 98 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/performance_simple_test.rs
**Lines**: 66 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/performance_with_builtins_demo.rs
**Lines**: 124 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/r7rs_pico_demo.rs
**Lines**: 231 | **Structs**: 0 | **Enums**: 0 | **Functions**: 3

---

#### examples/r7rs_standard_functions_demo.rs
**Lines**: 280 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/simple_benchmark.rs
**Lines**: 163 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/srfi46_nested_ellipsis_demo.rs
**Lines**: 435 | **Structs**: 0 | **Enums**: 0 | **Functions**: 8

---

#### examples/srfi_9_demo.rs
**Lines**: 202 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

#### examples/string_optimization_demo.rs
**Lines**: 94 | **Structs**: 0 | **Enums**: 0 | **Functions**: 4

---

#### examples/universe_polymorphic_demo.rs
**Lines**: 377 | **Structs**: 0 | **Enums**: 0 | **Functions**: 7

---

#### src/benchmarks/ghc_comparison.rs
**Lines**: 884 | **Structs**: 9 | **Enums**: 5 | **Functions**: 0

**Public Structs**:
- `PerformanceMetrics` (Line 19)
- `GHCComparisonResult` (Line 36)
- `GHCReferenceMetrics` (Line 53)
- ... and 6 more

**Public Enums**:
- `GHCOptimizationLevel` (Line 68)
- `GHCBenchmarkCategory` (Line 94)
- `StatisticalMethod` (Line 154)
- ... and 2 more

---

#### src/benchmarks/mod.rs
**Lines**: 328 | **Structs**: 6 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `BenchmarkCoordinator` (Line 18)
- `BenchmarkSuiteResult` (Line 42)
- `TestResult` (Line 53)
- ... and 3 more

**Public Enums**:
- `TestStatus` (Line 79)
- `VerbosityLevel` (Line 122)

---

#### src/bin/repl.rs
**Lines**: 1023 | **Structs**: 4 | **Enums**: 0 | **Functions**: 1

**Public Structs**:
- `DebugState` (Line 67)
- `ReplConfig` (Line 77)
- `SchemeHelper` (Line 104)
- ... and 1 more

---

#### src/bridge.rs
**Lines**: 578 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `ExternalObject` (Line 120)
- `ObjectRegistry` (Line 145)
- `LambdustBridge` (Line 318)

---

#### src/cps_inlining.rs
**Lines**: 526 | **Structs**: 4 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `CpsInliner` (Line 25)
- `InliningConfig` (Line 36)
- `InliningStats` (Line 60)
- ... and 1 more

**Public Enums**:
- `InliningDecision` (Line 13)
- `ChainStrategy` (Line 394)

---

#### src/debug/mod.rs
**Lines**: 497 | **Structs**: 1 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `TraceEntry` (Line 22)

**Public Enums**:
- `TraceLevel` (Line 47)

---

#### src/embedded_evaluator.rs
**Lines**: 637 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `EmbeddedEnvironment` (Line 49)
- `EmbeddedEvaluator` (Line 97)

**Public Enums**:
- `EmbeddedValue` (Line 17)

---

#### src/error_tests.rs
**Lines**: 946 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/ffi.rs
**Lines**: 662 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `LambdustContext` (Line 61)
- `MemoryTracker` (Line 84)
- `CallbackInfo` (Line 102)

**Public Enums**:
- `LambdustErrorCode` (Line 26)

---

#### src/ffi_enhanced.rs
**Lines**: 462 | **Structs**: 0 | **Enums**: 0 | **Functions**: 0

---

#### src/formal_verification.rs
**Lines**: 1504 | **Structs**: 15 | **Enums**: 9 | **Functions**: 0

**Public Structs**:
- `FormalVerificationEngine` (Line 17)
- `ProofObligation` (Line 45)
- `FormalStatement` (Line 101)
- ... and 12 more

**Public Enums**:
- `ProofCategory` (Line 73)
- `QuantifierType` (Line 133)
- `ProofPriority` (Line 146)
- ... and 6 more

---

#### src/interpreter.rs
**Lines**: 268 | **Structs**: 1 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `LambdustInterpreter` (Line 20)

---

#### src/lsp/completion.rs
**Lines**: 539 | **Structs**: 4 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `CompletionItem` (Line 41)
- `TextEdit` (Line 72)
- `CompletionContext` (Line 81)
- ... and 1 more

**Public Enums**:
- `CompletionItemKind` (Line 16)
- `ExpressionContext` (Line 106)

---

#### src/lsp/diagnostics.rs
**Lines**: 583 | **Structs**: 5 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `Diagnostic` (Line 28)
- `DiagnosticRelatedInformation` (Line 53)
- `Location` (Line 63)
- ... and 2 more

**Public Enums**:
- `DiagnosticSeverity` (Line 15)
- `DiagnosticTag` (Line 73)

---

#### src/lsp/document.rs
**Lines**: 566 | **Structs**: 5 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `DocumentManager` (Line 13)
- `Document` (Line 26)
- `DocumentMetadata` (Line 54)
- ... and 2 more

**Public Enums**:
- `LineEndings` (Line 73)

---

#### src/lsp/handlers.rs
**Lines**: 516 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `HandlerContext` (Line 15)
- `MessageRouter` (Line 202)
- `RequestHandlers` (Line 214)
- ... and 1 more

---

#### src/lsp/hover.rs
**Lines**: 468 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `HoverInfo` (Line 14)
- `HoverMetadata` (Line 27)
- `HoverProvider` (Line 73)

**Public Enums**:
- `SymbolType` (Line 46)

---

#### src/lsp/mod.rs
**Lines**: 176 | **Structs**: 2 | **Enums**: 1 | **Functions**: 1

**Public Structs**:
- `LspConfig` (Line 36)
- `LspCapabilities` (Line 79)

**Public Enums**:
- `LspError` (Line 131)

**Key Public Functions**:
- `initialize_lsp_server()` (Line 125)

---

#### src/lsp/position.rs
**Lines**: 432 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `Position` (Line 12)
- `Range` (Line 21)
- `Span` (Line 30)
- ... and 1 more

---

#### src/lsp/server.rs
**Lines**: 444 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `LambdustLanguageServer` (Line 19)
- `InitializeParams` (Line 71)
- `WorkspaceFolder` (Line 96)

**Public Enums**:
- `ServerState` (Line 56)

---

#### src/lsp/symbols.rs
**Lines**: 685 | **Structs**: 6 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `Symbol` (Line 14)
- `Location` (Line 33)
- `SymbolMetadata` (Line 92)
- ... and 3 more

**Public Enums**:
- `SymbolKind` (Line 43)
- `SymbolTag` (Line 114)

---

#### src/marshal.rs
**Lines**: 338 | **Structs**: 1 | **Enums**: 1 | **Functions**: 3

**Public Structs**:
- `TypeSafeMarshaller` (Line 58)

**Public Enums**:
- `MarshalError` (Line 16)

**Key Public Functions**:
- `scheme_string_to_c()` (Line 293)
- `c_int_to_scheme()` (Line 310)
- `scheme_to_c_int()` (Line 315)

---

#### src/module_system.rs
**Lines**: 296 | **Structs**: 2 | **Enums**: 1 | **Functions**: 2

**Public Structs**:
- `LibraryImport` (Line 22)
- `ModuleSystem` (Line 28)

**Public Enums**:
- `ImportSpec` (Line 13)

---

#### src/optimization/mod.rs
**Lines**: 373 | **Structs**: 5 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `EvolvingOptimizationEngine` (Line 22)
- `PerformanceMetrics` (Line 35)
- `EvolutionResult` (Line 239)
- ... and 2 more

**Public Enums**:
- `OptimizationResult` (Line 228)

---

#### src/optimization/theorem_derivation.rs
**Lines**: 386 | **Structs**: 2 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `LearnedPattern` (Line 31)
- `TheoremDerivationEngine` (Line 43)

**Public Enums**:
- `InferenceRule` (Line 14)

---

#### src/optimization/verified_optimization.rs
**Lines**: 229 | **Structs**: 4 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `OptimizationController` (Line 33)
- `OptimizationStats` (Line 42)
- `VerificationSystem` (Line 52)
- ... and 1 more

---

#### src/optimized_collections.rs
**Lines**: 335 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `SliceRef` (Line 17)
- `CowVec` (Line 156)

---

#### src/parser/cycle_detector.rs
**Lines**: 511 | **Structs**: 4 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `Cycle` (Line 8)
- `CycleDetectionResult` (Line 28)
- `CycleDetector` (Line 36)

**Public Enums**:
- `CycleType` (Line 17)

---

#### src/parser/dependency_analyzer.rs
**Lines**: 437 | **Structs**: 3 | **Enums**: 1 | **Functions**: 0

**Public Structs**:
- `DependencyNode` (Line 8)
- `DependencyGraph` (Line 34)
- `DependencyAnalyzer` (Line 102)

**Public Enums**:
- `DependencyType` (Line 21)

---

#### src/parser/loop_detection.rs
**Lines**: 485 | **Structs**: 3 | **Enums**: 1 | **Functions**: 2

**Public Structs**:
- `LoopDetectionConfig` (Line 11)
- `InfiniteLoopInfo` (Line 48)
- `InfiniteLoopDetector` (Line 60)

**Public Enums**:
- `InfiniteLoopType` (Line 35)

**Key Public Functions**:
- `check_for_infinite_loops()` (Line 311)
- `check_for_infinite_loops_with_config()` (Line 324)

---

#### src/stack_monitor.rs
**Lines**: 413 | **Structs**: 3 | **Enums**: 2 | **Functions**: 0

**Public Structs**:
- `StackFrame` (Line 11)
- `StackMonitor` (Line 57)
- `StackStatistics` (Line 303)

**Public Enums**:
- `StackFrameType` (Line 24)
- `OptimizationRecommendation` (Line 322)

---

#### src/wasm.rs
**Lines**: 378 | **Structs**: 2 | **Enums**: 0 | **Functions**: 0

**Public Structs**:
- `WasmLambdustInterpreter` (Line 27)
- `WasiLambdustInterpreter` (Line 226)

---

#### test_numbers.rs
**Lines**: 2 | **Structs**: 0 | **Enums**: 0 | **Functions**: 1

---

## 🔍 Detailed API Index

### Key Structures

#### BindingContext
**Location**: `src/evaluator/hotpath_analysis.rs:187`

```rust
pub struct BindingContext {
    /// Active variable bindings
    pub bindings: HashMap<String, String>,
    
    /// Binding creation timestamp
    // ...
```

#### CallStackContext
**Location**: `src/evaluator/hotpath_analysis.rs:171`

```rust
pub struct CallStackContext {
    /// Call stack depth
    pub depth: usize,
    
    /// Caller information
    // ...
```

#### CompilationContext
**Location**: `src/srfi/srfi_138.rs:45`

```rust
pub struct CompilationContext {
    /// Compilation target
    pub target: CompilationTarget,
    /// Optimization level
    pub optimization: OptimizationLevel,
    // ...
```

#### CompletionContext
**Location**: `src/lsp/completion.rs:81`

```rust
pub struct CompletionContext {
    /// Position where completion was triggered
    pub position: Position,
    
    /// Character that triggered completion (if any)
    // ...
```

#### ContextualFrequencyTracker
**Location**: `src/evaluator/hotpath_analysis.rs:158`

```rust
pub struct ContextualFrequencyTracker {
    /// Call stack contexts
    call_contexts: HashMap<String, CallStackContext>,
    
    /// Variable binding contexts
    // ...
```

#### EllipsisContext
**Location**: `src/macros/srfi46_ellipsis.rs:50`

```rust
pub struct EllipsisContext {
    /// Current ellipsis nesting depth
    pub current_depth: usize,
    /// Maximum observed nesting depth
    pub max_depth: usize,
    // ...
```

#### EmbeddedEnvironment
**Location**: `src/embedded_evaluator.rs:49`

```rust
pub struct EmbeddedEnvironment {
    bindings: HashMap<String, EmbeddedValue>,
    parent: Option<Rc<EmbeddedEnvironment>>,
}
```

#### EmbeddedEvaluator
**Location**: `src/embedded_evaluator.rs:97`

```rust
pub struct EmbeddedEvaluator {
    global_env: Rc<EmbeddedEnvironment>,
}
```

#### EnvironmentRef
**Location**: `src/evaluator/continuation.rs:175`

```rust
pub struct EnvironmentRef {
    /// Weak reference to environment to avoid reference cycles
    env: std::rc::Weak<Environment>,
    /// Fallback strong reference for critical operations
    strong_ref: Option<Rc<Environment>>,
    // ...
```

#### ErrorContext
**Location**: `src/error.rs:150`

```rust
pub struct ErrorContext {
    /// Source location where error occurred
    pub location: SourceSpan,
    /// Call stack trace
    pub stack_trace: Vec<StackFrame>,
    // ...
```

#### EvaluationContext
**Location**: `src/evaluator/evaluation_mode_selector.rs:129`

```rust
pub struct EvaluationContext {
    /// Current recursion depth
    pub recursion_depth: usize,

    /// Available memory (bytes)
    // ...
```

#### Evaluator
**Location**: `src/evaluator/types.rs:133`

```rust
pub struct Evaluator {
    /// Memory management strategy
    memory_strategy: MemoryStrategy,
    /// Dynamic points stack for dynamic-wind semantics
    dynamic_points: Vec<DynamicPoint>,
    // ...
```

#### EvaluatorComparison
**Location**: `src/evaluator/performance_measurement/evaluator_comparison.rs:19`

```rust
pub struct EvaluatorComparison {
    /// Semantic evaluator (reference implementation)
    semantic_evaluator: SemanticEvaluator,
    /// Runtime executor (optimized implementation)
    runtime_executor: RuntimeExecutor,
    // ...
```

#### EvaluatorInterface
**Location**: `src/evaluator/evaluator_interface.rs:114`

```rust
pub struct EvaluatorInterface {
    /// Semantic evaluator (reference implementation)
    semantic_evaluator: SemanticEvaluator,
    /// Runtime executor (optimized implementation)
    runtime_executor: RuntimeExecutor,
    // ...
```

#### ExecutionContext
**Location**: `src/evaluator/execution_context.rs:24`

```rust
pub struct ExecutionContext {
    /// The expression to be executed
    pub expression: Expr,
    
    /// Runtime environment for variable lookups
    // ...
```

#### ExecutionContextBuilder
**Location**: `src/evaluator/execution_context.rs:710`

```rust
pub struct ExecutionContextBuilder {
    context: ExecutionContext,
}
```

#### ExpansionContext
**Location**: `src/macros/hygiene/context.rs:28`

```rust
pub struct ExpansionContext {
    /// Current expansion depth (0 = top level)
    pub depth: usize,
    /// Stack of macro names being expanded (for nested expansions)
    pub macro_stack: Vec<String>,
    // ...
```

#### HandlerContext
**Location**: `src/lsp/handlers.rs:15`

```rust
pub struct HandlerContext {
    /// Language server instance
    pub server: Arc<RwLock<LambdustLanguageServer>>,
}
```

#### HygienicEnvironment
**Location**: `src/macros/hygiene/environment.rs:63`

```rust
pub struct HygienicEnvironment {
    /// Unique identifier for this environment
    pub id: EnvironmentId,
    /// Traditional environment for backward compatibility
    pub inner: Rc<Environment>,
    // ...
```

#### InferenceContext
**Location**: `src/type_system/type_inference.rs:11`

```rust
pub struct InferenceContext {
    /// Variable type assignments
    type_env: HashMap<String, PolynomialType>,
    /// Type variable constraints
    constraints: Vec<TypeConstraint>,
    // ...
```

#### InlineEvaluator
**Location**: `src/evaluator/inline_evaluation.rs:197`

```rust
pub struct InlineEvaluator {
    /// Hot path detector
    hot_path_detector: HotPathDetector,
    /// Successfully inlined continuation count
    inlined_count: usize,
    // ...
```

#### LambdustContext
**Location**: `src/ffi.rs:61`

```rust
pub struct LambdustContext {
    /// Core Scheme interpreter instance
    pub interpreter: LambdustInterpreter,
    /// Bridge for host function integration
    pub bridge: LambdustBridge,
    // ...
```

#### LegacyEvaluatorAdapter
**Location**: `src/evaluator/backward_compatibility.rs:20`

```rust
pub struct LegacyEvaluatorAdapter {
    /// New evaluator interface
    evaluator_interface: Arc<Mutex<EvaluatorInterface>>,

    /// Configuration mapping for legacy settings
    // ...
```

#### MeasureValue
**Location**: `src/evaluator/church_rosser_proof.rs:358`

```rust
pub struct MeasureValue {
    /// 数値測度
    pub numeric_value: usize,

    /// 構造的測度
    // ...
```

#### MeasurementEnvironment
**Location**: `src/evaluator/performance_measurement/core_types.rs:165`

```rust
pub struct MeasurementEnvironment {
    /// 最適化レベル
    pub optimization_level: RuntimeOptimizationLevel,
    
    /// 評価モード
    // ...
```

#### ModuleContext
**Location**: `src/evaluator/hotpath_analysis.rs:200`

```rust
pub struct ModuleContext {
    /// Module name
    pub module_name: String,
    
    /// Imported modules
    // ...
```

#### MutableEnvironment
**Location**: `src/environment.rs:502`

```rust
pub struct MutableEnvironment {
    inner: RefCell<SharedEnvironment>,
}
```

#### OptimizationExecutionContext
**Location**: `src/evaluator/runtime_optimization/optimization_manager.rs:75`

```rust
pub struct OptimizationExecutionContext {
    /// 最適化レベル
    pub optimization_level: RuntimeOptimizationLevel,

    /// 環境情報
    // ...
```

#### Parser
**Location**: `src/parser.rs:12`

```rust
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    /// Configuration for infinite loop detection
    loop_detection_config: LoopDetectionConfig,
    // ...
```

#### PicoEvaluator
**Location**: `src/evaluator/pico_evaluator.rs:41`

```rust
pub struct PicoEvaluator {
    /// Maximum recursion depth to prevent stack overflow
    max_recursion_depth: usize,
    /// Current recursion depth
    current_depth: usize,
    // ...
```

#### ProofContext
**Location**: `src/evaluator/theorem_proving.rs:154`

```rust
pub struct ProofContext {
    /// Variable bindings
    pub variables: HashMap<String, Expr>,

    /// Type assumptions
    // ...
```

#### SemanticEvaluator
**Location**: `src/evaluator/semantic/semantic_core.rs:21`

```rust
pub struct SemanticEvaluator {
    /// Global environment containing R7RS standard library
    #[allow(dead_code)]
    global_env: Rc<Environment>,

    // ...
```

#### SharedEnvironment
**Location**: `src/environment.rs:16`

```rust
pub struct SharedEnvironment {
    /// Local bindings for this environment frame
    /// Only contains bindings added to this specific frame
    local_bindings: HashMap<String, Value>,

    // ...
```

#### TailCallContext
**Location**: `src/evaluator/tail_call_optimization.rs:24`

```rust
pub struct TailCallContext {
    /// Whether the current position is a tail position
    pub is_tail_position: bool,
    /// Current function being analyzed (for self-recursion detection)
    pub current_function: Option<String>,
    // ...
```

#### TypeCheckError
**Location**: `src/type_system/parallel_type_checker.rs:62`

```rust
pub struct TypeCheckError {
    /// Task identifier
    pub task_id: TaskId,
    /// The actual error
    pub error: LambdustError,
    // ...
```

#### TypeContext
**Location**: `src/type_system/type_checker.rs:22`

```rust
pub struct TypeContext {
    /// Variable type bindings
    variables: HashMap<String, PolynomialType>,
    /// Universe level context
    universe_level: UniverseLevel,
    // ...
```

#### ValueOptimizer
**Location**: `src/value/optimized.rs:426`

```rust
pub struct ValueOptimizer {
    stats: OptimizationStats,
}
```

#### ValuePool
**Location**: `src/memory_pool.rs:63`

```rust
pub struct ValuePool {
    /// Cached boolean values (true/false singletons)
    boolean_cache: [Value; 2],
    /// Cached nil singleton
    nil_singleton: Value,
    // ...
```

### Key Enumerations

#### AnalysisResultValue
**Location**: `src/evaluator/performance_measurement/analysis.rs:284`

```rust
pub enum AnalysisResultValue {
    /// 数値
    Numeric(f64),
    
    /// パーセンテージ
    // ...
```

#### BindingValue
**Location**: `src/macros/types.rs:48`

```rust
pub enum BindingValue {
    /// Single expression binding
    Single(Expr),
    /// Multiple expressions (from ellipsis)
    Multiple(Vec<Expr>),
    // ...
```

#### CombinatorExpr
**Location**: `src/evaluator/combinators.rs:13`

```rust
pub enum CombinatorExpr {
    /// S combinator: S x y z = x z (y z)
    S,

    /// K combinator: K x y = x
    // ...
```

#### ConstraintValue
**Location**: `src/evaluator/runtime_optimization/core_types.rs:334`

```rust
pub enum ConstraintValue {
    /// 期間
    Duration(Duration),
    /// メモリサイズ（バイト）
    MemorySize(usize),
    // ...
```

#### EmbeddedValue
**Location**: `src/embedded_evaluator.rs:17`

```rust
pub enum EmbeddedValue {
    /// Undefined/uninitialized
    Undefined,
    /// Boolean values
    Boolean(bool),
    // ...
```

#### ErrorHandlingStrategy
**Location**: `src/evaluator/backward_compatibility.rs:77`

```rust
pub enum ErrorHandlingStrategy {
    /// Strict: fail immediately on any compatibility issue without attempting recovery
    Strict,
    /// Graceful: log warnings and continue operation with degraded functionality
    Graceful,
    // ...
```

#### Expr
**Location**: `src/ast.rs:8`

```rust
pub enum Expr {
    /// Literal values
    Literal(Literal),
    /// Variable reference
    Variable(String),
    // ...
```

#### ExpressionContext
**Location**: `src/lsp/completion.rs:106`

```rust
pub enum ExpressionContext {
    /// At the beginning of an expression (after '(')
    ExpressionStart,
    
    /// In function position (first element of list)
    // ...
```

#### ExpressionType
**Location**: `src/evaluator/runtime_optimization/core_types.rs:423`

```rust
pub enum ExpressionType {
    /// リテラル
    Literal,
    /// 変数
    Variable,
    // ...
```

#### LambdustError
**Location**: `src/error.rs:177`

```rust
pub enum LambdustError {
    /// Lexical analysis errors
    #[error("Lexer error: {message}")]
    LexerError {
        /// Error message
    // ...
```

#### LambdustErrorCode
**Location**: `src/ffi.rs:26`

```rust
pub enum LambdustErrorCode {
    /// Operation successful
    Success = 0,
    /// General error
    Error = 1,
    // ...
```

#### LspError
**Location**: `src/lsp/mod.rs:131`

```rust
pub enum LspError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Document not found: {0}")]
    // ...
```

#### MarshalError
**Location**: `src/marshal.rs:16`

```rust
pub enum MarshalError {
    /// Type mismatch between expected and actual types
    TypeMismatch {
        /// Expected type name
        expected: String,
    // ...
```

#### MetricValue
**Location**: `src/evaluator/performance_measurement/core_types.rs:83`

```rust
pub enum MetricValue {
    /// 時間値
    Duration(Duration),
    /// メモリサイズ（バイト）
    MemorySize(usize),
    // ...
```

#### MultiDimValue
**Location**: `src/macros/srfi46_ellipsis.rs:26`

```rust
pub enum MultiDimValue {
    /// Scalar value (depth 0)
    Scalar(Expr),
    /// 1D array of values (depth 1)
    Array1D(Vec<Expr>),
    // ...
```

#### OptimizedValue
**Location**: `src/value/optimized.rs:14`

```rust
pub enum OptimizedValue {
    /// Undefined value (used for uninitialized variables)
    Undefined,
    /// Boolean values (1 bit + tag)
    Boolean(bool),
    // ...
```

#### Token
**Location**: `src/lexer.rs:9`

```rust
pub enum Token {
    /// Left parenthesis '('
    LeftParen,
    /// Right parenthesis ')'
    RightParen,
    // ...
```

#### UniverseExpression
**Location**: `src/type_system/universe_polymorphic_classes.rs:113`

```rust
pub enum UniverseExpression {
    /// Variable: u
    Variable(String),
    /// Literal: 0, 1, 2, ...
    Literal(UniverseLevel),
    // ...
```

#### Value
**Location**: `src/value/mod.rs:39`

```rust
pub enum Value {
    /// Undefined value (used for uninitialized variables)
    Undefined,
    /// Boolean values
    Boolean(bool),
    // ...
```

#### ValueType
**Location**: `src/host.rs:17`

```rust
pub enum ValueType {
    /// Any value type
    Any,
    /// Boolean type
    Boolean,
    // ...
```

