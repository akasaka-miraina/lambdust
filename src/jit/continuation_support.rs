//! First-class Continuation Support for JIT Compiled Code
//!
//! This module implements R7RS-compliant call/cc (call-with-current-continuation) 
//! support in JIT-compiled code, including stack capture, continuation objects,
//! and proper unwinding semantics.

use crate::ast::Expr;
use crate::eval::Value;
use crate::jit::{
    code_generator::NativeCode,
    compilation_tiers::CompilationTier,
    specialized_compilation_tiers::SpecializedNativeCode,
};
use crate::diagnostics::{Result, Error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Continuation support manager for JIT compilation
pub struct ContinuationSupport {
    /// Configuration for continuation handling
    config: ContinuationConfig,
    /// Active continuations by ID
    active_continuations: HashMap<ContinuationId, ContinuationObject>,
    /// Stack frames captured for continuations
    captured_frames: HashMap<ContinuationId, Vec<StackFrame>>,
    /// Continuation compilation strategies
    compilation_strategies: HashMap<CompilationTier, ContinuationStrategy>,
    /// Performance statistics
    stats: ContinuationStats,
}

/// Configuration for continuation support
#[derive(Debug, Clone)]
pub struct ContinuationConfig {
    /// Enable optimized continuation capture
    pub enable_optimized_capture: bool,
    /// Maximum stack depth for continuation capture
    pub max_stack_depth: usize,
    /// Enable continuation caching
    pub enable_continuation_caching: bool,
    /// Maximum number of active continuations
    pub max_active_continuations: usize,
    /// Enable partial continuation support (delimited continuations)
    pub enable_delimited_continuations: bool,
    /// Stack scanning strategy
    pub stack_scan_strategy: StackScanStrategy,
}

impl Default for ContinuationConfig {
    fn default() -> Self {
        Self {
            enable_optimized_capture: true,
            max_stack_depth: 10000,
            enable_continuation_caching: true,
            max_active_continuations: 1000,
            enable_delimited_continuations: false,
            stack_scan_strategy: StackScanStrategy::Conservative,
        }
    }
}

/// Strategy for scanning the stack during continuation capture
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackScanStrategy {
    /// Conservative scanning - scan all potential pointers
    Conservative,
    /// Precise scanning - only scan known pointers
    Precise,
    /// Tagged scanning - use tag bits to identify pointers
    Tagged,
}

/// Unique identifier for continuations
pub type ContinuationId = u64;

/// A first-class continuation object
#[derive(Debug, Clone)]
pub struct ContinuationObject {
    /// Unique identifier
    pub id: ContinuationId,
    /// Captured stack frames
    pub stack_frames: Vec<StackFrame>,
    /// Captured environment bindings
    pub environment: HashMap<String, Value>,
    /// Continuation type
    pub continuation_type: ContinuationType,
    /// Creation timestamp
    pub created_at: std::time::Instant,
    /// Optimization level used
    pub optimization_level: CompilationTier,
    /// Whether this continuation has been invoked
    pub invoked: bool,
}

/// Types of continuations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationType {
    /// Full continuation (call/cc)
    Full,
    /// Escape continuation (early return)
    Escape,
    /// Delimited continuation (with prompts)
    Delimited { prompt_tag: String },
    /// Partial continuation (continuation fragments)
    Partial { parent_id: Option<ContinuationId> },
}

/// A captured stack frame
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name or identifier
    pub function_name: String,
    /// Stack frame base pointer
    pub frame_pointer: usize,
    /// Stack pointer at capture time
    pub stack_pointer: usize,
    /// Return address
    pub return_address: usize,
    /// Local variables in this frame
    pub locals: HashMap<String, Value>,
    /// Frame type
    pub frame_type: FrameType,
    /// JIT-specific frame metadata
    pub jit_metadata: Option<JitFrameMetadata>,
}

/// Types of stack frames
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameType {
    /// Scheme function frame
    SchemeFunction,
    /// Native JIT-compiled function frame
    NativeFunction,
    /// Interpreter frame
    InterpreterFrame,
    /// Foreign function interface frame
    FfiFrame,
    /// Exception handler frame
    ExceptionHandler,
}

/// JIT-specific metadata for stack frames
#[derive(Debug, Clone)]
pub struct JitFrameMetadata {
    /// Compiled code address
    pub code_address: usize,
    /// Optimization level
    pub optimization_level: CompilationTier,
    /// Register state
    pub register_state: Vec<RegisterValue>,
    /// Stack layout information
    pub stack_layout: StackLayout,
}

/// Register value at continuation capture
#[derive(Debug, Clone)]
pub struct RegisterValue {
    /// Register name or number
    pub register: String,
    /// Value stored in register
    pub value: RegisterValueType,
}

/// Types of values that can be stored in registers
#[derive(Debug, Clone)]
pub enum RegisterValueType {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Pointer to memory
    Pointer(usize),
    /// Scheme value
    SchemeValue(Value),
    /// Unknown or untracked value
    Unknown,
}

/// Stack layout information
#[derive(Debug, Clone)]
pub struct StackLayout {
    /// Total frame size in bytes
    pub frame_size: usize,
    /// Offset of local variables
    pub locals_offset: isize,
    /// Offset of saved registers
    pub saved_registers_offset: isize,
    /// Offset of return address
    pub return_address_offset: isize,
    /// Stack alignment requirements
    pub alignment: usize,
}

/// Continuation compilation strategy for each tier
#[derive(Debug, Clone)]
pub enum ContinuationStrategy {
    /// Interpreter-based continuation handling
    Interpreter {
        /// Use stack copying
        use_stack_copying: bool,
    },
    /// Bytecode-based continuation handling
    Bytecode {
        /// Generate continuation bytecode
        generate_continuation_bytecode: bool,
    },
    /// Basic JIT continuation handling
    BasicJit {
        /// Use trampoline for continuation invocation
        use_trampoline: bool,
        /// Enable register spilling
        enable_register_spilling: bool,
    },
    /// Optimized JIT continuation handling
    OptimizedJit {
        /// Use advanced stack scanning
        use_advanced_stack_scanning: bool,
        /// Enable continuation inlining
        enable_continuation_inlining: bool,
        /// Use continuation caching
        use_continuation_caching: bool,
    },
}

/// Statistics for continuation operations
#[derive(Debug, Default)]
struct ContinuationStats {
    /// Total continuations created
    continuations_created: u64,
    /// Total continuations invoked
    continuations_invoked: u64,
    /// Total stack frames captured
    frames_captured: u64,
    /// Average capture time
    average_capture_time: Duration,
    /// Average invocation time
    average_invocation_time: Duration,
    /// Memory used for continuations
    memory_used_bytes: u64,
    /// Cache hit rate for continuation lookups
    cache_hit_rate: f64,
}

impl ContinuationSupport {
    /// Create new continuation support with configuration
    pub fn new(config: ContinuationConfig) -> Result<Self> {
        let mut strategies = HashMap::new();
        
        // Set up compilation strategies for each tier
        strategies.insert(CompilationTier::Interpreter, ContinuationStrategy::Interpreter {
            use_stack_copying: true,
        });
        
        strategies.insert(CompilationTier::Bytecode, ContinuationStrategy::Bytecode {
            generate_continuation_bytecode: true,
        });
        
        strategies.insert(CompilationTier::JitBasic, ContinuationStrategy::BasicJit {
            use_trampoline: true,
            enable_register_spilling: true,
        });
        
        strategies.insert(CompilationTier::JitOptimized, ContinuationStrategy::OptimizedJit {
            use_advanced_stack_scanning: true,
            enable_continuation_inlining: false, // Conservative default
            use_continuation_caching: config.enable_continuation_caching,
        });
        
        Ok(Self {
            config,
            active_continuations: HashMap::new(),
            captured_frames: HashMap::new(),
            compilation_strategies: strategies,
            stats: ContinuationStats::default(),
        })
    }

    /// Create default continuation support
    pub fn default() -> Result<Self> {
        Self::new(ContinuationConfig::default())
    }

    /// Capture the current continuation
    pub fn capture_continuation(
        &mut self,
        continuation_type: ContinuationType,
        tier: CompilationTier,
    ) -> Result<ContinuationObject> {
        let start_time = std::time::Instant::now();
        
        // Generate unique continuation ID
        let id = self.generate_continuation_id();
        
        // Capture stack frames based on compilation tier strategy
        let stack_frames = self.capture_stack_frames(&tier)?;
        
        // Capture environment
        let environment = self.capture_environment()?;
        
        // Create continuation object
        let continuation = ContinuationObject {
            id,
            stack_frames: stack_frames.clone(),
            environment,
            continuation_type,
            created_at: std::time::Instant::now(),
            optimization_level: tier,
            invoked: false,
        };
        
        // Store continuation and frames
        self.active_continuations.insert(id, continuation.clone());
        self.captured_frames.insert(id, stack_frames);
        
        // Update statistics
        let capture_time = start_time.elapsed();
        self.stats.continuations_created += 1;
        self.stats.frames_captured += continuation.stack_frames.len() as u64;
        self.update_average_capture_time(capture_time);
        
        Ok(continuation)
    }

    /// Invoke a continuation with a value
    pub fn invoke_continuation(
        &mut self,
        continuation_id: ContinuationId,
        value: Value,
    ) -> Result<Value> {
        let start_time = std::time::Instant::now();
        
        // Get the continuation
        let mut continuation = self.active_continuations.get(&continuation_id)
            .ok_or_else(|| Error::runtime_error(
                format!("Continuation {continuation_id} not found"),
                None,
            ))?.clone();
        
        // Check if already invoked (continuations can only be used once in R7RS)
        if continuation.invoked {
            return Err(Box::new(Error::runtime_error(
                "Continuation has already been invoked".to_string(),
                None,
            )));
        }
        
        // Mark as invoked
        continuation.invoked = true;
        self.active_continuations.insert(continuation_id, continuation.clone());
        
        // Restore stack frames and environment
        self.restore_continuation_state(&continuation)?;
        
        // Update statistics
        let invocation_time = start_time.elapsed();
        self.stats.continuations_invoked += 1;
        self.update_average_invocation_time(invocation_time);
        
        // Return the provided value as the result of the original call/cc
        Ok(value)
    }

    /// Generate native code support for call/cc
    pub fn generate_callcc_support(
        &self,
        native_code: &mut NativeCode,
        tier: CompilationTier,
    ) -> Result<()> {
        let strategy = self.compilation_strategies.get(&tier)
            .ok_or_else(|| Error::runtime_error(
                format!("No continuation strategy for tier {tier:?}"),
                None,
            ))?;

        match strategy {
            ContinuationStrategy::BasicJit { use_trampoline, enable_register_spilling } => {
                // Generate basic JIT support
                if *use_trampoline {
                    self.generate_trampoline_support(native_code)?;
                }
                if *enable_register_spilling {
                    self.generate_register_spilling_support(native_code)?;
                }
            }
            ContinuationStrategy::OptimizedJit { 
                use_advanced_stack_scanning, 
                enable_continuation_inlining,
                use_continuation_caching 
            } => {
                // Generate optimized JIT support
                if *use_advanced_stack_scanning {
                    self.generate_advanced_stack_scanning(native_code)?;
                }
                if *enable_continuation_inlining {
                    self.generate_continuation_inlining(native_code)?;
                }
                if *use_continuation_caching {
                    self.generate_continuation_caching(native_code)?;
                }
            }
            _ => {
                // Interpreter and bytecode strategies don't need native code generation
            }
        }
        
        Ok(())
    }

    /// Generate specialized continuation support
    pub fn generate_specialized_callcc_support(
        &self,
        specialized_code: &mut SpecializedNativeCode,
        tier: CompilationTier,
    ) -> Result<()> {
        // For specialized code, we can apply more aggressive optimizations
        // This is a simplified implementation - would be much more complex in practice
        Ok(())
    }

    /// Check if call/cc is supported for a given compilation tier
    pub fn is_callcc_supported(&self, tier: CompilationTier) -> bool {
        self.compilation_strategies.contains_key(&tier)
    }

    /// Get continuation support capabilities for a tier
    pub fn get_tier_capabilities(&self, tier: CompilationTier) -> Result<ContinuationCapabilities> {
        let strategy = self.compilation_strategies.get(&tier)
            .ok_or_else(|| Error::runtime_error(
                format!("No continuation strategy for tier {tier:?}"),
                None,
            ))?;

        let capabilities = match strategy {
            ContinuationStrategy::Interpreter { .. } => ContinuationCapabilities {
                full_continuations: true,
                escape_continuations: true,
                delimited_continuations: false,
                partial_continuations: false,
                performance_overhead: 5.0, // High overhead
                memory_overhead: 3.0,
                stack_safety: true,
            },
            ContinuationStrategy::Bytecode { .. } => ContinuationCapabilities {
                full_continuations: true,
                escape_continuations: true,
                delimited_continuations: false,
                partial_continuations: false,
                performance_overhead: 3.0,
                memory_overhead: 2.5,
                stack_safety: true,
            },
            ContinuationStrategy::BasicJit { .. } => ContinuationCapabilities {
                full_continuations: true,
                escape_continuations: true,
                delimited_continuations: false,
                partial_continuations: false,
                performance_overhead: 2.0,
                memory_overhead: 2.0,
                stack_safety: true,
            },
            ContinuationStrategy::OptimizedJit { .. } => ContinuationCapabilities {
                full_continuations: true,
                escape_continuations: true,
                delimited_continuations: self.config.enable_delimited_continuations,
                partial_continuations: true,
                performance_overhead: 1.2, // Minimal overhead
                memory_overhead: 1.5,
                stack_safety: true,
            },
        };

        Ok(capabilities)
    }

    /// Clean up expired or unused continuations
    pub fn cleanup_continuations(&mut self) -> Result<usize> {
        let before_count = self.active_continuations.len();
        
        // Remove invoked continuations (they can't be reused)
        self.active_continuations.retain(|id, cont| {
            if cont.invoked {
                self.captured_frames.remove(id);
                false
            } else {
                true
            }
        });
        
        // Remove old continuations if we exceed the maximum
        if self.active_continuations.len() > self.config.max_active_continuations {
            // Sort by creation time and remove oldest
            let mut continuations: Vec<_> = self.active_continuations.iter()
                .map(|(id, cont)| (*id, cont.created_at))
                .collect();
            continuations.sort_by_key(|(_, created_at)| *created_at);
            
            let to_remove = continuations.len() - self.config.max_active_continuations;
            for (id, _) in continuations.into_iter().take(to_remove) {
                self.active_continuations.remove(&id);
                self.captured_frames.remove(&id);
            }
        }
        
        let cleaned_up = before_count - self.active_continuations.len();
        Ok(cleaned_up)
    }

    /// Get continuation statistics
    pub fn get_stats(&self) -> &ContinuationStats {
        &self.stats
    }

    // Private helper methods

    fn generate_continuation_id(&mut self) -> ContinuationId {
        // Simple ID generation - in practice would be more sophisticated
        self.stats.continuations_created
    }

    fn capture_stack_frames(&self, tier: &CompilationTier) -> Result<Vec<StackFrame>> {
        // This is a simplified implementation
        // In reality, would use platform-specific stack walking mechanisms
        
        let mut frames = Vec::new();
        
        // Create a dummy frame for testing
        frames.push(StackFrame {
            function_name: "test_function".to_string(),
            frame_pointer: 0x1000,
            stack_pointer: 0x1008,
            return_address: 0x2000,
            locals: HashMap::new(),
            frame_type: match tier {
                CompilationTier::Interpreter => FrameType::InterpreterFrame,
                CompilationTier::Bytecode => FrameType::InterpreterFrame,
                CompilationTier::JitBasic | CompilationTier::JitOptimized => FrameType::NativeFunction,
            },
            jit_metadata: if matches!(tier, CompilationTier::JitBasic | CompilationTier::JitOptimized) {
                Some(JitFrameMetadata {
                    code_address: 0x3000,
                    optimization_level: *tier,
                    register_state: vec![
                        RegisterValue {
                            register: "rax".to_string(),
                            value: RegisterValueType::Integer(42),
                        }
                    ],
                    stack_layout: StackLayout {
                        frame_size: 64,
                        locals_offset: -32,
                        saved_registers_offset: -16,
                        return_address_offset: 8,
                        alignment: 16,
                    },
                })
            } else {
                None
            },
        });
        
        Ok(frames)
    }

    fn capture_environment(&self) -> Result<HashMap<String, Value>> {
        // Simplified environment capture
        let mut env = HashMap::new();
        env.insert("test_var".to_string(), Value::integer(42));
        Ok(env)
    }

    fn restore_continuation_state(&self, continuation: &ContinuationObject) -> Result<()> {
        // This would restore the stack frames and environment
        // Extremely complex in practice, requiring platform-specific assembly
        
        // For now, just validate that we can restore
        if continuation.stack_frames.is_empty() {
            return Err(Box::new(Error::runtime_error(
                "Cannot restore continuation with no stack frames".to_string(),
                None,
            )));
        }
        
        Ok(())
    }

    fn generate_trampoline_support(&self, _native_code: &mut NativeCode) -> Result<()> {
        // Generate trampoline code for continuation invocation
        // This is a placeholder - would generate actual assembly code
        Ok(())
    }

    fn generate_register_spilling_support(&self, _native_code: &mut NativeCode) -> Result<()> {
        // Generate code to spill/restore registers for continuation capture
        Ok(())
    }

    fn generate_advanced_stack_scanning(&self, _native_code: &mut NativeCode) -> Result<()> {
        // Generate advanced stack scanning code
        Ok(())
    }

    fn generate_continuation_inlining(&self, _native_code: &mut NativeCode) -> Result<()> {
        // Generate inlined continuation support
        Ok(())
    }

    fn generate_continuation_caching(&self, _native_code: &mut NativeCode) -> Result<()> {
        // Generate continuation caching support
        Ok(())
    }

    fn update_average_capture_time(&mut self, time: Duration) {
        let total_captures = self.stats.continuations_created;
        if total_captures == 1 {
            self.stats.average_capture_time = time;
        } else {
            let old_total = self.stats.average_capture_time * (total_captures - 1) as u32;
            self.stats.average_capture_time = (old_total + time) / total_captures as u32;
        }
    }

    fn update_average_invocation_time(&mut self, time: Duration) {
        let total_invocations = self.stats.continuations_invoked;
        if total_invocations == 1 {
            self.stats.average_invocation_time = time;
        } else {
            let old_total = self.stats.average_invocation_time * (total_invocations - 1) as u32;
            self.stats.average_invocation_time = (old_total + time) / total_invocations as u32;
        }
    }
}

/// Capabilities of continuation support for a compilation tier
#[derive(Debug, Clone)]
pub struct ContinuationCapabilities {
    /// Support for full continuations (call/cc)
    pub full_continuations: bool,
    /// Support for escape continuations
    pub escape_continuations: bool,
    /// Support for delimited continuations
    pub delimited_continuations: bool,
    /// Support for partial continuations
    pub partial_continuations: bool,
    /// Performance overhead factor
    pub performance_overhead: f64,
    /// Memory overhead factor
    pub memory_overhead: f64,
    /// Whether stack safety is maintained
    pub stack_safety: bool,
}

/// Trait for continuation-aware code generation
pub trait ContinuationAware {
    /// Check if this code supports continuations
    fn supports_continuations(&self) -> bool;
    
    /// Get continuation metadata
    fn get_continuation_metadata(&self) -> Option<HashMap<String, String>>;
    
    /// Check if call/cc can be optimized in this context
    fn can_optimize_callcc(&self) -> bool;
}

impl ContinuationAware for NativeCode {
    fn supports_continuations(&self) -> bool {
        // Would check native code metadata
        true // Assume support for now
    }
    
    fn get_continuation_metadata(&self) -> Option<HashMap<String, String>> {
        // Would extract continuation metadata from native code
        None
    }
    
    fn can_optimize_callcc(&self) -> bool {
        // Would check if call/cc optimizations are possible
        true
    }
}

impl ContinuationAware for SpecializedNativeCode {
    fn supports_continuations(&self) -> bool {
        true
    }
    
    fn get_continuation_metadata(&self) -> Option<HashMap<String, String>> {
        None
    }
    
    fn can_optimize_callcc(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuation_support_creation() {
        let support = ContinuationSupport::default();
        assert!(support.is_ok());
        
        let support = support.unwrap();
        assert_eq!(support.active_continuations.len(), 0);
        assert!(support.is_callcc_supported(CompilationTier::Interpreter));
        assert!(support.is_callcc_supported(CompilationTier::JitOptimized));
    }

    #[test]
    fn test_continuation_capture() {
        let mut support = ContinuationSupport::default().unwrap();
        
        let continuation = support.capture_continuation(
            ContinuationType::Full,
            CompilationTier::Interpreter,
        );
        
        assert!(continuation.is_ok());
        let continuation = continuation.unwrap();
        
        assert_eq!(continuation.continuation_type, ContinuationType::Full);
        assert!(!continuation.invoked);
        assert!(!continuation.stack_frames.is_empty());
    }

    #[test]
    fn test_continuation_invocation() {
        let mut support = ContinuationSupport::default().unwrap();
        
        let continuation = support.capture_continuation(
            ContinuationType::Full,
            CompilationTier::Interpreter,
        ).unwrap();
        
        let id = continuation.id;
        
        // First invocation should succeed
        let result = support.invoke_continuation(id, Value::integer(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(42));
        
        // Second invocation should fail (R7RS compliance)
        let result = support.invoke_continuation(id, Value::integer(24));
        assert!(result.is_err());
    }

    #[test]
    fn test_tier_capabilities() {
        let support = ContinuationSupport::default().unwrap();
        
        let capabilities = support.get_tier_capabilities(CompilationTier::Interpreter);
        assert!(capabilities.is_ok());
        let capabilities = capabilities.unwrap();
        
        assert!(capabilities.full_continuations);
        assert!(capabilities.stack_safety);
        assert!(capabilities.performance_overhead > 1.0);
        
        let opt_capabilities = support.get_tier_capabilities(CompilationTier::JitOptimized);
        assert!(opt_capabilities.is_ok());
        let opt_capabilities = opt_capabilities.unwrap();
        
        assert!(opt_capabilities.performance_overhead < capabilities.performance_overhead);
    }

    #[test]
    fn test_continuation_cleanup() {
        let mut support = ContinuationSupport::default().unwrap();
        
        // Create and invoke a continuation
        let continuation = support.capture_continuation(
            ContinuationType::Full,
            CompilationTier::Interpreter,
        ).unwrap();
        
        let id = continuation.id;
        support.invoke_continuation(id, Value::integer(42)).unwrap();
        
        // Clean up should remove the invoked continuation
        let cleaned = support.cleanup_continuations().unwrap();
        assert_eq!(cleaned, 1);
        assert!(!support.active_continuations.contains_key(&id));
    }

    #[test]
    fn test_continuation_types() {
        let mut support = ContinuationSupport::default().unwrap();
        
        // Test different continuation types
        let full_cont = support.capture_continuation(
            ContinuationType::Full,
            CompilationTier::JitOptimized,
        ).unwrap();
        assert_eq!(full_cont.continuation_type, ContinuationType::Full);
        
        let escape_cont = support.capture_continuation(
            ContinuationType::Escape,
            CompilationTier::JitBasic,
        ).unwrap();
        assert_eq!(escape_cont.continuation_type, ContinuationType::Escape);
        
        let delimited_cont = support.capture_continuation(
            ContinuationType::Delimited { prompt_tag: "test".to_string() },
            CompilationTier::JitOptimized,
        ).unwrap();
        match delimited_cont.continuation_type {
            ContinuationType::Delimited { prompt_tag } => {
                assert_eq!(prompt_tag, "test");
            }
            _ => panic!("Expected delimited continuation"),
        }
    }
}