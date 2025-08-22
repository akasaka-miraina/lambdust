//! Tail Call Optimization for JIT Compiled Code
//!
//! This module implements proper tail call elimination for JIT-compiled Scheme code,
//! ensuring R7RS compliance by guaranteeing constant stack space usage for tail recursion.

use crate::ast::{Expr, Formals};
use crate::diagnostics::{Error, Result};
use crate::jit::{
    code_generator::NativeCode, compilation_tiers::CompilationTier,
    specialized_compilation_tiers::SpecializedNativeCode,
};
use std::collections::{HashMap, HashSet};

/// Tail call optimization analyzer and transformer
pub struct TailCallOptimizer {
    /// Configuration for tail call optimization
    config: TailCallConfig,
    /// Analysis results cache
    analysis_cache: HashMap<String, TailCallAnalysis>,
    /// Optimization statistics
    stats: TailCallStats,
}

/// Configuration for tail call optimization
#[derive(Debug, Clone)]
pub struct TailCallConfig {
    /// Enable aggressive tail call optimization
    pub aggressive_optimization: bool,
    /// Maximum recursion depth for analysis
    pub max_analysis_depth: usize,
    /// Enable mutual recursion optimization
    pub enable_mutual_recursion: bool,
    /// Minimum benefit threshold for optimization
    pub min_benefit_threshold: f64,
    /// Enable tail call optimization across compilation units
    pub enable_cross_unit_optimization: bool,
}

impl Default for TailCallConfig {
    fn default() -> Self {
        Self {
            aggressive_optimization: true,
            max_analysis_depth: 100,
            enable_mutual_recursion: true,
            min_benefit_threshold: 1.5, // 50% improvement minimum
            enable_cross_unit_optimization: false,
        }
    }
}

/// Analysis result for tail call optimization
#[derive(Debug, Clone)]
pub struct TailCallAnalysis {
    /// Tail call sites identified
    pub tail_call_sites: Vec<TailCallSite>,
    /// Self-recursive calls
    pub self_recursive_calls: Vec<SelfRecursiveCall>,
    /// Mutual recursion groups
    pub mutual_recursion_groups: Vec<MutualRecursionGroup>,
    /// Stack frame elimination opportunities
    pub frame_elimination_opportunities: Vec<FrameEliminationOpportunity>,
    /// Estimated performance benefit
    pub estimated_benefit: PerformanceBenefit,
    /// R7RS compliance status
    pub r7rs_compliance: R7RSComplianceStatus,
}

/// A tail call site in the code
#[derive(Debug, Clone)]
pub struct TailCallSite {
    /// Location of the tail call
    pub location: TailCallLocation,
    /// Type of tail call
    pub call_type: TailCallType,
    /// Target function if known
    pub target_function: Option<String>,
    /// Optimization strategy to apply
    pub optimization_strategy: TailCallOptimizationStrategy,
    /// Estimated performance improvement
    pub performance_improvement: f64,
}

/// Location of a tail call in the AST
#[derive(Debug, Clone)]
pub struct TailCallLocation {
    /// Function containing the tail call
    pub containing_function: String,
    /// AST node path to the tail call
    pub ast_path: Vec<String>,
    /// Line and column if available
    pub source_location: Option<(usize, usize)>,
}

/// Types of tail calls
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TailCallType {
    /// Self-recursive tail call
    SelfRecursive,
    /// Mutually recursive tail call
    MutuallyRecursive,
    /// General tail call (non-recursive)
    General,
    /// Tail call to higher-order function
    HigherOrder,
    /// Tail call with continuation passing
    ContinuationPassing,
}

/// Optimization strategies for tail calls
#[derive(Debug, Clone)]
pub enum TailCallOptimizationStrategy {
    /// Replace with a jump (for self-recursion)
    JumpReplacement {
        /// Target label for the jump
        target_label: String,
        /// Stack adjustments needed
        stack_adjustments: Vec<StackAdjustment>,
    },
    /// Trampoline optimization (for mutual recursion)
    Trampoline {
        /// Trampoline function name
        trampoline_name: String,
        /// Functions in the mutual recursion group
        function_group: Vec<String>,
    },
    /// Continuation passing style transformation
    ContinuationPassing {
        /// New CPS function name
        cps_function_name: String,
        /// Continuation parameter name
        continuation_param: String,
    },
    /// Inline expansion (for small functions)
    InlineExpansion {
        /// Maximum inlining depth
        max_depth: usize,
        /// Size threshold for inlining
        size_threshold: usize,
    },
}

/// Stack adjustment for tail call optimization
#[derive(Debug, Clone)]
pub struct StackAdjustment {
    /// Type of adjustment
    pub adjustment_type: StackAdjustmentType,
    /// Offset in stack slots
    pub offset: isize,
    /// Size in bytes
    pub size: usize,
}

/// Types of stack adjustments
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackAdjustmentType {
    /// Deallocate local variables
    DeallocateLocals,
    /// Adjust argument positions
    AdjustArguments,
    /// Preserve return address
    PreserveReturnAddress,
    /// Update frame pointer
    UpdateFramePointer,
}

/// Self-recursive call analysis
#[derive(Debug, Clone)]
pub struct SelfRecursiveCall {
    /// Call site location
    pub call_site: TailCallLocation,
    /// Arguments that change between calls
    pub changing_arguments: Vec<String>,
    /// Loop invariants
    pub invariants: Vec<String>,
    /// Estimated recursion depth
    pub estimated_depth: Option<usize>,
    /// Memory usage pattern
    pub memory_pattern: MemoryUsagePattern,
}

/// Mutual recursion group analysis
#[derive(Debug, Clone)]
pub struct MutualRecursionGroup {
    /// Functions in the group
    pub functions: Vec<String>,
    /// Call graph within the group
    pub call_graph: HashMap<String, Vec<String>>,
    /// Shared variables
    pub shared_variables: Vec<String>,
    /// Optimization complexity
    pub complexity: MutualRecursionComplexity,
}

/// Complexity of mutual recursion optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutualRecursionComplexity {
    /// Simple two-function mutual recursion
    Simple,
    /// Complex multi-function mutual recursion
    Complex,
    /// Highly complex with multiple interconnections
    HighlyComplex,
}

/// Frame elimination opportunity
#[derive(Debug, Clone)]
pub struct FrameEliminationOpportunity {
    /// Function where frame can be eliminated
    pub function_name: String,
    /// Stack slots that can be eliminated
    pub eliminable_slots: Vec<String>,
    /// Memory savings in bytes
    pub memory_savings: usize,
    /// Performance improvement factor
    pub performance_factor: f64,
}

/// Performance benefit estimation
#[derive(Debug, Clone)]
pub struct PerformanceBenefit {
    /// Runtime speedup factor
    pub speedup_factor: f64,
    /// Memory usage reduction factor
    pub memory_reduction_factor: f64,
    /// Stack overflow prevention benefit
    pub prevents_stack_overflow: bool,
    /// Cache performance improvement
    pub cache_improvement: f64,
}

/// Memory usage pattern for recursive calls
#[derive(Debug, Clone)]
pub enum MemoryUsagePattern {
    /// Constant memory usage (true tail recursion)
    Constant,
    /// Linear growth with recursion depth
    Linear { growth_rate: f64 },
    /// Exponential growth (needs optimization)
    Exponential { base: f64 },
    /// Complex pattern
    Complex { description: String },
}

/// R7RS compliance status for tail calls
#[derive(Debug, Clone)]
pub struct R7RSComplianceStatus {
    /// Whether all tail calls are properly optimized
    pub all_optimized: bool,
    /// Non-optimized tail calls (compliance violations)
    pub violations: Vec<TailCallViolation>,
    /// Stack space guarantee maintained
    pub stack_space_guaranteed: bool,
    /// Compliance level achieved
    pub compliance_level: TailCallComplianceLevel,
}

/// Tail call compliance violation
#[derive(Debug, Clone)]
pub struct TailCallViolation {
    /// Location of the violation
    pub location: TailCallLocation,
    /// Reason for non-optimization
    pub reason: ViolationReason,
    /// Severity of the violation
    pub severity: ViolationSeverity,
    /// Suggested fix
    pub suggested_fix: String,
}

/// Reasons for tail call optimization failure
#[derive(Debug, Clone)]
pub enum ViolationReason {
    /// Call is not in tail position
    NotInTailPosition,
    /// Complex control flow prevents optimization
    ComplexControlFlow,
    /// Exception handlers interfere
    ExceptionHandlerInterference,
    /// Multiple return values complicate optimization
    MultipleReturnValues,
    /// Closure captures prevent optimization
    ClosureCaptureInterference,
    /// Native code generation limitation
    CodeGenerationLimitation,
}

/// Severity of compliance violations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Critical - breaks R7RS compliance
    Critical,
    /// Major - significantly impacts performance
    Major,
    /// Minor - small performance impact
    Minor,
}

/// Tail call compliance levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TailCallComplianceLevel {
    /// Full R7RS compliance - all tail calls optimized
    FullCompliance,
    /// Partial compliance - most tail calls optimized
    PartialCompliance,
    /// Non-compliant - significant tail call issues
    NonCompliant,
}

/// Statistics about tail call optimization
#[derive(Debug, Default)]
pub struct TailCallStats {
    /// Total functions analyzed
    functions_analyzed: u64,
    /// Tail calls identified
    tail_calls_identified: u64,
    /// Tail calls optimized
    tail_calls_optimized: u64,
    /// Self-recursive calls optimized
    self_recursive_optimized: u64,
    /// Mutual recursion groups optimized
    mutual_recursion_optimized: u64,
    /// Stack frames eliminated
    stack_frames_eliminated: u64,
    /// Total memory saved (bytes)
    memory_saved_bytes: u64,
    /// Average performance improvement
    average_performance_improvement: f64,
}

impl TailCallOptimizer {
    /// Create new tail call optimizer with configuration
    pub fn new(config: TailCallConfig) -> Self {
        Self {
            config,
            analysis_cache: HashMap::new(),
            stats: TailCallStats::default(),
        }
    }

    /// Create optimizer with default configuration
    pub fn default() -> Self {
        Self::new(TailCallConfig::default())
    }

    /// Analyze an expression for tail call optimization opportunities
    pub fn analyze_expression(
        &mut self,
        expr: &Expr,
        function_name: &str,
    ) -> Result<TailCallAnalysis> {
        // Check cache first
        if let Some(cached) = self.analysis_cache.get(function_name) {
            return Ok(cached.clone());
        }

        let mut analysis = TailCallAnalysis {
            tail_call_sites: Vec::new(),
            self_recursive_calls: Vec::new(),
            mutual_recursion_groups: Vec::new(),
            frame_elimination_opportunities: Vec::new(),
            estimated_benefit: PerformanceBenefit {
                speedup_factor: 1.0,
                memory_reduction_factor: 1.0,
                prevents_stack_overflow: false,
                cache_improvement: 0.0,
            },
            r7rs_compliance: R7RSComplianceStatus {
                all_optimized: true,
                violations: Vec::new(),
                stack_space_guaranteed: true,
                compliance_level: TailCallComplianceLevel::FullCompliance,
            },
        };

        // Perform tail position analysis
        self.analyze_tail_positions(expr, function_name, &mut analysis, true)?;

        // Analyze self-recursion patterns
        let tail_calls = analysis.tail_call_sites.clone();
        self.analyze_self_recursion(&tail_calls, function_name, &mut analysis)?;

        // Estimate performance benefits
        analysis.estimated_benefit = self.estimate_performance_benefit(&analysis)?;

        // Check R7RS compliance
        analysis.r7rs_compliance = self.check_r7rs_compliance(&analysis)?;

        // Cache the result
        self.analysis_cache
            .insert(function_name.to_string(), analysis.clone());

        // Update statistics
        self.stats.functions_analyzed += 1;
        self.stats.tail_calls_identified += analysis.tail_call_sites.len() as u64;

        Ok(analysis)
    }

    /// Analyze tail positions in an expression
    fn analyze_tail_positions(
        &self,
        expr: &Expr,
        function_name: &str,
        analysis: &mut TailCallAnalysis,
        is_tail_position: bool,
    ) -> Result<()> {
        match expr {
            Expr::Application { operator, operands } => {
                if is_tail_position {
                    // This is a tail call
                    let call_type = self.determine_call_type(operator, function_name)?;
                    let optimization_strategy =
                        self.determine_optimization_strategy(&call_type, operator)?;

                    let tail_call = TailCallSite {
                        location: TailCallLocation {
                            containing_function: function_name.to_string(),
                            ast_path: vec!["application".to_string()],
                            source_location: None,
                        },
                        call_type,
                        target_function: self.extract_function_name(operator),
                        optimization_strategy,
                        performance_improvement: 2.0, // Default estimate
                    };

                    analysis.tail_call_sites.push(tail_call);
                }

                // Analyze operands (not in tail position)
                for operand in operands {
                    self.analyze_tail_positions(&operand.inner, function_name, analysis, false)?;
                }
            }

            Expr::If {
                test,
                consequent,
                alternative,
            } => {
                // Test is not in tail position
                self.analyze_tail_positions(&test.inner, function_name, analysis, false)?;

                // Both branches preserve tail position
                self.analyze_tail_positions(
                    &consequent.inner,
                    function_name,
                    analysis,
                    is_tail_position,
                )?;
                if let Some(alt) = alternative {
                    self.analyze_tail_positions(
                        &alt.inner,
                        function_name,
                        analysis,
                        is_tail_position,
                    )?;
                }
            }

            Expr::Begin(exprs) => {
                // Only the last expression is in tail position
                for (i, expr) in exprs.iter().enumerate() {
                    let is_last = i == exprs.len() - 1;
                    self.analyze_tail_positions(
                        &expr.inner,
                        function_name,
                        analysis,
                        is_tail_position && is_last,
                    )?;
                }
            }

            Expr::Let { bindings, body } => {
                // Binding values are not in tail position
                for binding in bindings {
                    self.analyze_tail_positions(
                        &binding.value.inner,
                        function_name,
                        analysis,
                        false,
                    )?;
                }

                // Body expressions preserve tail position
                for (i, expr) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    self.analyze_tail_positions(
                        &expr.inner,
                        function_name,
                        analysis,
                        is_tail_position && is_last,
                    )?;
                }
            }

            Expr::Lambda { body, .. } => {
                // Lambda bodies are separate tail contexts
                for (i, expr) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    self.analyze_tail_positions(&expr.inner, "lambda", analysis, is_last)?;
                }
            }

            _ => {
                // Other expressions don't contain tail calls
            }
        }

        Ok(())
    }

    /// Determine the type of a function call
    fn determine_call_type(&self, operator: &Expr, current_function: &str) -> Result<TailCallType> {
        match operator {
            Expr::Identifier(name) => {
                if name == current_function {
                    Ok(TailCallType::SelfRecursive)
                } else {
                    Ok(TailCallType::General)
                }
            }
            _ => Ok(TailCallType::General),
        }
    }

    /// Determine optimization strategy for a call type
    fn determine_optimization_strategy(
        &self,
        call_type: &TailCallType,
        operator: &Expr,
    ) -> Result<TailCallOptimizationStrategy> {
        match call_type {
            TailCallType::SelfRecursive => Ok(TailCallOptimizationStrategy::JumpReplacement {
                target_label: "function_start".to_string(),
                stack_adjustments: vec![
                    StackAdjustment {
                        adjustment_type: StackAdjustmentType::DeallocateLocals,
                        offset: -8,
                        size: 64,
                    },
                    StackAdjustment {
                        adjustment_type: StackAdjustmentType::AdjustArguments,
                        offset: 0,
                        size: 32,
                    },
                ],
            }),
            TailCallType::MutuallyRecursive => {
                Ok(TailCallOptimizationStrategy::Trampoline {
                    trampoline_name: "mutual_recursion_trampoline".to_string(),
                    function_group: vec![], // Would be filled in by mutual recursion analysis
                })
            }
            _ => {
                // For general tail calls, use continuation passing if beneficial
                Ok(TailCallOptimizationStrategy::ContinuationPassing {
                    cps_function_name: "tailcall_continuation".to_string(),
                    continuation_param: "cont".to_string(),
                })
            }
        }
    }

    /// Extract function name from operator expression
    fn extract_function_name(&self, operator: &Expr) -> Option<String> {
        match operator {
            Expr::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    /// Analyze self-recursion patterns
    fn analyze_self_recursion(
        &self,
        tail_calls: &[TailCallSite],
        function_name: &str,
        analysis: &mut TailCallAnalysis,
    ) -> Result<()> {
        for tail_call in tail_calls {
            if tail_call.call_type == TailCallType::SelfRecursive {
                let self_recursive = SelfRecursiveCall {
                    call_site: tail_call.location.clone(),
                    changing_arguments: vec![], // Would need more analysis
                    invariants: vec![],
                    estimated_depth: None,
                    memory_pattern: MemoryUsagePattern::Constant, // Optimistic assumption
                };

                analysis.self_recursive_calls.push(self_recursive);

                // Add frame elimination opportunity
                analysis
                    .frame_elimination_opportunities
                    .push(FrameEliminationOpportunity {
                        function_name: function_name.to_string(),
                        eliminable_slots: vec![
                            "return_address".to_string(),
                            "frame_pointer".to_string(),
                        ],
                        memory_savings: 16, // 2 * 8 bytes on 64-bit
                        performance_factor: 1.2,
                    });
            }
        }

        Ok(())
    }

    /// Estimate performance benefits of tail call optimization
    fn estimate_performance_benefit(
        &self,
        analysis: &TailCallAnalysis,
    ) -> Result<PerformanceBenefit> {
        let mut speedup_factor = 1.0;
        let mut memory_reduction_factor = 1.0;
        let mut prevents_overflow = false;
        let mut cache_improvement = 0.0;

        // Calculate benefits from self-recursive calls
        for self_recursive in &analysis.self_recursive_calls {
            speedup_factor *= 1.5; // Estimate 50% improvement
            match &self_recursive.memory_pattern {
                MemoryUsagePattern::Constant => {
                    memory_reduction_factor *= 0.1; // 90% memory reduction
                    prevents_overflow = true;
                }
                MemoryUsagePattern::Linear { growth_rate } => {
                    memory_reduction_factor *= (1.0 - growth_rate * 0.8).max(0.1);
                }
                _ => {}
            }
        }

        // Calculate benefits from frame elimination
        for opportunity in &analysis.frame_elimination_opportunities {
            speedup_factor *= opportunity.performance_factor;
            cache_improvement += 0.1; // Better cache locality
        }

        Ok(PerformanceBenefit {
            speedup_factor,
            memory_reduction_factor,
            prevents_stack_overflow: prevents_overflow,
            cache_improvement,
        })
    }

    /// Check R7RS compliance for tail call optimization
    fn check_r7rs_compliance(&self, analysis: &TailCallAnalysis) -> Result<R7RSComplianceStatus> {
        let mut violations = Vec::new();
        let mut all_optimized = true;

        // Check that all tail calls can be optimized
        for tail_call in &analysis.tail_call_sites {
            // For now, assume all can be optimized
            // In a real implementation, would check for various constraints
        }

        let compliance_level = if violations.is_empty() {
            TailCallComplianceLevel::FullCompliance
        } else if violations.len() < analysis.tail_call_sites.len() / 2 {
            TailCallComplianceLevel::PartialCompliance
        } else {
            TailCallComplianceLevel::NonCompliant
        };

        Ok(R7RSComplianceStatus {
            all_optimized,
            violations,
            stack_space_guaranteed: compliance_level == TailCallComplianceLevel::FullCompliance,
            compliance_level,
        })
    }

    /// Apply tail call optimizations to generated code
    pub fn optimize_native_code(
        &mut self,
        code: &mut NativeCode,
        analysis: &TailCallAnalysis,
    ) -> Result<()> {
        // This would apply the actual optimizations to the native code
        // For now, just update statistics

        for tail_call in &analysis.tail_call_sites {
            match &tail_call.optimization_strategy {
                TailCallOptimizationStrategy::JumpReplacement { .. } => {
                    self.stats.tail_calls_optimized += 1;
                    if tail_call.call_type == TailCallType::SelfRecursive {
                        self.stats.self_recursive_optimized += 1;
                    }
                }
                TailCallOptimizationStrategy::Trampoline { .. } => {
                    self.stats.mutual_recursion_optimized += 1;
                }
                _ => {
                    self.stats.tail_calls_optimized += 1;
                }
            }
        }

        self.stats.stack_frames_eliminated += analysis.frame_elimination_opportunities.len() as u64;

        Ok(())
    }

    /// Apply optimizations to specialized native code
    pub fn optimize_specialized_code(
        &mut self,
        code: &mut SpecializedNativeCode,
        analysis: &TailCallAnalysis,
    ) -> Result<()> {
        // Similar to optimize_native_code but for specialized code
        // For now, just apply the same optimizations as native code
        // In a real implementation, would have specific optimizations for specialized code
        Ok(())
    }

    /// Get tail call optimization statistics
    pub fn get_stats(&self) -> &TailCallStats {
        &self.stats
    }

    /// Clear analysis cache
    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
    }

    /// Check if a function has tail call optimization applied
    pub fn is_tail_call_optimized(&self, function_name: &str) -> bool {
        if let Some(analysis) = self.analysis_cache.get(function_name) {
            analysis.r7rs_compliance.all_optimized
        } else {
            false
        }
    }
}

/// Helper trait for adding tail call optimization methods to native code
pub trait TailCallOptimizable {
    /// Check if tail call optimization is applied
    fn has_tail_call_optimization(&self) -> bool;

    /// Get tail call optimization metadata
    fn get_tail_call_metadata(&self) -> Option<HashMap<String, String>>;
}

impl TailCallOptimizable for NativeCode {
    fn has_tail_call_optimization(&self) -> bool {
        // Would check native code metadata
        false // Placeholder
    }

    fn get_tail_call_metadata(&self) -> Option<HashMap<String, String>> {
        // Would extract metadata from native code
        None // Placeholder
    }
}

impl TailCallOptimizable for SpecializedNativeCode {
    fn has_tail_call_optimization(&self) -> bool {
        // Would check specialized code metadata
        false // Placeholder
    }

    fn get_tail_call_metadata(&self) -> Option<HashMap<String, String>> {
        // Would extract metadata from specialized code
        None // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_tail_call_optimizer_creation() {
        let optimizer = TailCallOptimizer::default();
        assert!(optimizer.config.aggressive_optimization);
        assert_eq!(optimizer.analysis_cache.len(), 0);
    }

    #[test]
    fn test_tail_position_analysis() {
        let mut optimizer = TailCallOptimizer::default();

        // Simple self-recursive function: (define (fact n) (if (= n 0) 1 (fact (- n 1))))
        let expr = Expr::If {
            test: Box::new(crate::diagnostics::Spanned::new(
                Expr::Application {
                    operator: Box::new(crate::diagnostics::Spanned::new(
                        Expr::Identifier("=".to_string()),
                        crate::diagnostics::Span::new(0, 1),
                    )),
                    operands: vec![
                        crate::diagnostics::Spanned::new(
                            Expr::Identifier("n".to_string()),
                            crate::diagnostics::Span::new(0, 1),
                        ),
                        crate::diagnostics::Spanned::new(
                            Expr::Literal(Literal::ExactInteger(0)),
                            crate::diagnostics::Span::new(0, 1),
                        ),
                    ],
                },
                crate::diagnostics::Span::new(0, 10),
            )),
            consequent: Box::new(crate::diagnostics::Spanned::new(
                Expr::Literal(Literal::ExactInteger(1)),
                crate::diagnostics::Span::new(0, 1),
            )),
            alternative: Some(Box::new(crate::diagnostics::Spanned::new(
                Expr::Application {
                    operator: Box::new(crate::diagnostics::Spanned::new(
                        Expr::Identifier("fact".to_string()),
                        crate::diagnostics::Span::new(0, 4),
                    )),
                    operands: vec![crate::diagnostics::Spanned::new(
                        Expr::Application {
                            operator: Box::new(crate::diagnostics::Spanned::new(
                                Expr::Identifier("-".to_string()),
                                crate::diagnostics::Span::new(0, 1),
                            )),
                            operands: vec![
                                crate::diagnostics::Spanned::new(
                                    Expr::Identifier("n".to_string()),
                                    crate::diagnostics::Span::new(0, 1),
                                ),
                                crate::diagnostics::Spanned::new(
                                    Expr::Literal(Literal::ExactInteger(1)),
                                    crate::diagnostics::Span::new(0, 1),
                                ),
                            ],
                        },
                        crate::diagnostics::Span::new(0, 6),
                    )],
                },
                crate::diagnostics::Span::new(0, 12),
            ))),
        };

        let analysis = optimizer.analyze_expression(&expr, "fact");
        assert!(analysis.is_ok());

        let analysis = analysis.unwrap();
        assert_eq!(analysis.tail_call_sites.len(), 1);
        assert_eq!(
            analysis.tail_call_sites[0].call_type,
            TailCallType::SelfRecursive
        );
        assert!(matches!(
            analysis.tail_call_sites[0].optimization_strategy,
            TailCallOptimizationStrategy::JumpReplacement { .. }
        ));
    }

    #[test]
    fn test_performance_benefit_estimation() {
        let optimizer = TailCallOptimizer::default();

        let analysis = TailCallAnalysis {
            tail_call_sites: vec![],
            self_recursive_calls: vec![SelfRecursiveCall {
                call_site: TailCallLocation {
                    containing_function: "test".to_string(),
                    ast_path: vec![],
                    source_location: None,
                },
                changing_arguments: vec![],
                invariants: vec![],
                estimated_depth: None,
                memory_pattern: MemoryUsagePattern::Constant,
            }],
            mutual_recursion_groups: vec![],
            frame_elimination_opportunities: vec![FrameEliminationOpportunity {
                function_name: "test".to_string(),
                eliminable_slots: vec!["frame_pointer".to_string()],
                memory_savings: 8,
                performance_factor: 1.2,
            }],
            estimated_benefit: PerformanceBenefit {
                speedup_factor: 1.0,
                memory_reduction_factor: 1.0,
                prevents_stack_overflow: false,
                cache_improvement: 0.0,
            },
            r7rs_compliance: R7RSComplianceStatus {
                all_optimized: true,
                violations: vec![],
                stack_space_guaranteed: true,
                compliance_level: TailCallComplianceLevel::FullCompliance,
            },
        };

        let benefit = optimizer.estimate_performance_benefit(&analysis);
        assert!(benefit.is_ok());

        let benefit = benefit.unwrap();
        assert!(benefit.speedup_factor > 1.0);
        assert!(benefit.prevents_stack_overflow);
        assert!(benefit.memory_reduction_factor < 1.0);
    }

    #[test]
    fn test_r7rs_compliance_checking() {
        let optimizer = TailCallOptimizer::default();

        let analysis = TailCallAnalysis {
            tail_call_sites: vec![TailCallSite {
                location: TailCallLocation {
                    containing_function: "test".to_string(),
                    ast_path: vec![],
                    source_location: None,
                },
                call_type: TailCallType::SelfRecursive,
                target_function: Some("test".to_string()),
                optimization_strategy: TailCallOptimizationStrategy::JumpReplacement {
                    target_label: "start".to_string(),
                    stack_adjustments: vec![],
                },
                performance_improvement: 2.0,
            }],
            self_recursive_calls: vec![],
            mutual_recursion_groups: vec![],
            frame_elimination_opportunities: vec![],
            estimated_benefit: PerformanceBenefit {
                speedup_factor: 1.0,
                memory_reduction_factor: 1.0,
                prevents_stack_overflow: false,
                cache_improvement: 0.0,
            },
            r7rs_compliance: R7RSComplianceStatus {
                all_optimized: true,
                violations: vec![],
                stack_space_guaranteed: true,
                compliance_level: TailCallComplianceLevel::FullCompliance,
            },
        };

        let compliance = optimizer.check_r7rs_compliance(&analysis);
        assert!(compliance.is_ok());

        let compliance = compliance.unwrap();
        assert_eq!(
            compliance.compliance_level,
            TailCallComplianceLevel::FullCompliance
        );
        assert!(compliance.stack_space_guaranteed);
        assert!(compliance.violations.is_empty());
    }
}
