//! Integration Bridge for Lambdust's Advanced Type System.
//!
//! This module provides the bridge between Lambdust's existing primitives,
//! the new advanced type system, and R7RS-large compliance. It handles:
//! - Compatibility with existing 42+ primitives
//! - Dynamic/static type coercion
//! - Performance optimization through primitive specialization
//! - Gradual migration path from dynamic to static typing

#![allow(missing_docs)]

use super::{Type, TypeVar, TypeScheme, TypeChecker, TypeLevel};
use super::algebraic::{Pattern, PatternMatcher};
use super::advanced_type_classes::AdvancedTypeClassEnv;
use super::r7rs_integration::R7RSIntegration;
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::fmt;

/// The main integration bridge for Lambdust's type system.
pub struct TypeSystemBridge {
    /// Current type level being used
    type_level: TypeLevel,
    /// Core type checker
    type_checker: TypeChecker,
    /// Advanced type class environment
    advanced_classes: AdvancedTypeClassEnv,
    /// R7RS integration layer
    r7rs_integration: R7RSIntegration,
    /// Pattern matcher for algebraic types
    pattern_matcher: PatternMatcher,
    /// Primitive optimization cache
    primitive_cache: Arc<RwLock<HashMap<String, OptimizedPrimitive>>>,
    /// Type migration state
    migration_state: MigrationState,
}

/// State tracking the migration from dynamic to static typing.
#[derive(Debug, Clone)]
pub struct MigrationState {
    /// Functions that have been migrated to static typing
    static_functions: HashSet<String>,
    /// Functions that have type annotations but remain dynamic
    annotated_functions: HashMap<String, TypeScheme>,
    /// Functions with inferred types but not yet specialized
    inferred_functions: HashMap<String, TypeScheme>,
    /// Migration warnings and suggestions
    migration_warnings: Vec<MigrationWarning>,
}

/// Warning about potential type migration issues.
#[derive(Debug, Clone)]
pub struct MigrationWarning {
    /// Warning message
    pub message: String,
    /// Location where the warning applies
    pub span: Option<Span>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Severity level
    pub severity: WarningSeverity,
}

/// Severity of a migration warning.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// An optimized primitive that can use type information for better performance.
#[derive(Debug, Clone)]
pub struct OptimizedPrimitive {
    /// Original primitive procedure
    pub base: PrimitiveProcedure,
    /// Optimized implementations for specific type combinations
    pub specializations: HashMap<TypeSignature, PrimitiveImpl>,
    /// Usage statistics for optimization decisions  
    pub stats: PrimitiveStats,
}

/// Type signature for primitive specialization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeSignature {
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type
    pub return_type: Type,
}

/// Statistics for primitive usage optimization.
#[derive(Debug, Clone)]
pub struct PrimitiveStats {
    /// Total call count
    pub call_count: u64,
    /// Calls with specific type combinations
    pub type_calls: HashMap<TypeSignature, u64>,
    /// Average execution time per call (nanoseconds)
    pub avg_execution_time: u64,
    /// Memory usage per call (bytes)
    pub avg_memory_usage: u64,
}

/// Configuration for type system integration.
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Enable type inference for primitives
    pub infer_primitive_types: bool,
    /// Enable primitive specialization
    pub specialize_primitives: bool,
    /// Enable gradual type checking
    pub gradual_typing: bool,
    /// Enable pattern matching compilation
    pub compile_patterns: bool,
    /// Maximum type recursion depth
    pub max_recursion_depth: usize,
    /// Performance optimization level
    pub optimization_level: OptimizationLevel,
}

/// Level of performance optimization to apply.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl TypeSystemBridge {
    /// Creates a new type system bridge.
    pub fn new(_config: IntegrationConfig) -> Self {
        Self {
            type_level: TypeLevel::Dynamic,
            type_checker: TypeChecker::new(TypeLevel::Dynamic),
            advanced_classes: AdvancedTypeClassEnv::default(),
            r7rs_integration: R7RSIntegration::new(),
            pattern_matcher: PatternMatcher::new(),
            primitive_cache: Arc::new(RwLock::new(HashMap::new())),
            migration_state: MigrationState::new(),
        }
    }

    /// Sets the current type level.
    pub fn set_type_level(&mut self, level: TypeLevel) {
        self.type_level = level;
        self.type_checker = TypeChecker::new(level);
    }

    /// Gets the current type level.
    pub fn type_level(&self) -> TypeLevel {
        self.type_level
    }

    /// Integrates a primitive procedure with the type system.
    pub fn integrate_primitive(&mut self, name: String, primitive: PrimitiveProcedure) -> Result<()> {
        // Infer or specify the type scheme for this primitive
        let type_scheme = self.infer_primitive_type(&name, &primitive)?;
        
        // Add to type environment
        self.type_checker.env_mut().bind(name.clone()), type_scheme.clone());
        
        // Create optimized primitive
        let optimized = OptimizedPrimitive {
            base: primitive,
            specializations: HashMap::new(),
            stats: PrimitiveStats::new(),
        };
        
        // Cache the optimized primitive
        self.primitive_cache.write().unwrap().insert(name, optimized);
        
        Ok(())
    }

    /// Infers the type scheme for a primitive procedure.
    fn infer_primitive_type(&self, name: &str, primitive: &PrimitiveProcedure) -> Result<TypeScheme> {
        match name {
            // Arithmetic operations
            "+" | "-" | "*" | "/" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![super::Constraint { class: "Num".to_string(), type_: Type::named_var("a") }],
                Type::function(
                    vec![Type::named_var("a"), Type::named_var("a")],
                    Type::named_var("a"),
                ),
            )),
            
            // Comparison operations
            "=" | "<" | ">" | "<=" | ">=" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![super::Constraint { class: "Ord".to_string(), type_: Type::named_var("a") }],
                Type::function(
                    vec![Type::named_var("a"), Type::named_var("a")],
                    Type::Boolean,
                ),
            )),
            
            // List operations
            "cons" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![],
                Type::function(
                    vec![Type::named_var("a"), Type::list(Type::named_var("a"))],
                    Type::list(Type::named_var("a")),
                ),
            )),
            "car" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![],
                Type::function(
                    vec![Type::pair(Type::named_var("a"), Type::Dynamic)],
                    Type::named_var("a"),
                ),
            )),
            "cdr" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a"), TypeVar::with_name("b")],
                vec![],
                Type::function(
                    vec![Type::pair(Type::named_var("a"), Type::named_var("b"))],
                    Type::named_var("b"),
                ),
            )),
            
            // I/O operations
            "display" | "write" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![super::Constraint { class: "Show".to_string(), type_: Type::named_var("a") }],
                Type::Effectful {
                    input: Box::new(Type::named_var("a")),
                    effects: vec![super::Effect::IO],
                    output: Box::new(Type::Unit),
                },
            )),
            
            // String operations
            "string-append" => Ok(TypeScheme::monomorphic(
                Type::function(
                    vec![Type::String, Type::String],
                    Type::String,
                ),
            )),
            "string-length" => Ok(TypeScheme::monomorphic(
                Type::function(vec![Type::String], Type::Number),
            )),
            
            // Vector operations
            "vector-ref" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![],
                Type::function(
                    vec![Type::vector(Type::named_var("a")), Type::Number],
                    Type::named_var("a"),
                ),
            )),
            "vector-set!" => Ok(TypeScheme::polymorphic(
                vec![TypeVar::with_name("a")],
                vec![],
                Type::Effectful {
                    input: Box::new(Type::vector(Type::named_var("a"))),
                    effects: vec![super::Effect::State(Type::Unit)],
                    output: Box::new(Type::Unit),
                },
            )),
            
            // Generic fallback
            _ => {
                // Try to infer from arity information
                let param_types = (0..primitive.arity_min)
                    .map(|i| Type::named_var(&format!("a{}", i)))
                    .collect();
                let return_type = Type::named_var("result");
                
                Ok(TypeScheme::polymorphic(
                    (0..=primitive.arity_min).map(|i| TypeVar::with_name(&format!("a{}", i))).collect(),
                    vec![],
                    Type::function(param_types, return_type),
                ))
            }
        }
    }

    /// Specializes a primitive for specific type arguments.
    pub fn specialize_primitive(
        &mut self, 
        name: &str, 
        type_args: &[Type]
    ) -> Result<Option<PrimitiveImpl>> {
        let cache = self.primitive_cache.read().unwrap();
        if let Some(optimized) = cache.get(name) {
            // Create type signature
            let sig = TypeSignature {
                params: type_args.to_vec(),
                return_type: Type::Dynamic, // Would be inferred
            };
            
            // Check if we have a specialization
            if let Some(specialized) = optimized.specializations.get(&sig) {
                return Ok(Some(specialized.clone()));
            }
            
            // Generate specialization if needed
            drop(cache); // Release read lock
            return self.generate_specialization(name, &sig);
        }
        
        Ok(None)
    }

    /// Generates a specialized implementation for a primitive.
    fn generate_specialization(&mut self, name: &str, sig: &TypeSignature) -> Result<Option<PrimitiveImpl>> {
        match name {
            "+" if sig.params.len() == 2 && sig.params.iter().all(|t| *t == Type::Number) => {
                // Specialized number addition
                Ok(Some(PrimitiveImpl::RustFn(|args| {
                    if let (Some(n1), Some(n2)) = (args[0].as_number(), args[1].as_number()) {
                        Ok(Value::number(n1 + n2))
                    } else {
                        Err(Box::new(Error::runtime_error("Type error in specialized +".to_string(), None))
                    }
                })))
            }
            "*" if sig.params.len() == 2 && sig.params.iter().all(|t| *t == Type::Number) => {
                // Specialized number multiplication
                Ok(Some(PrimitiveImpl::RustFn(|args| {
                    if let (Some(n1), Some(n2)) = (args[0].as_number(), args[1].as_number()) {
                        Ok(Value::number(n1 * n2))
                    } else {
                        Err(Box::new(Error::runtime_error("Type error in specialized *".to_string(), None))
                    }
                })))
            }
            "string-append" if sig.params.iter().all(|t| *t == Type::String) => {
                // Specialized string concatenation
                Ok(Some(PrimitiveImpl::RustFn(|args| {
                    let mut result = String::new();
                    for arg in args {
                        if let Some(s) = arg.as_string() {
                            result.push_str(s);
                        } else {
                            return Err(Box::new(Error::runtime_error("Type error in specialized string-append".to_string(), None));
                        }
                    }
                    Ok(Value::string(result))
                })))
            }
            _ => Ok(None), // No specialization available
        }
    }

    /// Checks if a value matches a pattern with type safety.
    pub fn type_safe_pattern_match(&mut self, pattern: &Pattern, value: &Value, expected_type: &Type) -> Result<bool> {
        // First check if the value matches the expected type
        if !self.value_matches_type(value, expected_type)? {
            return Ok(false);
        }
        
        // Then check pattern matching
        self.pattern_matcher.compile_match(&super::algebraic::MatchExpression {
            scrutinee: "value".to_string(), // Simplified
            clauses: vec![super::algebraic::MatchClause {
                pattern: pattern.clone()),
                guard: None,
                body: "true".to_string(),
                span: None,
            }],
            span: None,
        })?;
        
        // Simplified: assume pattern matches if we get here
        Ok(true)
    }

    /// Checks if a value matches a type (gradual typing).
    pub fn value_matches_type(&self, value: &Value, ty: &Type) -> Result<bool> {
        match ty {
            Type::Dynamic => Ok(true),
            Type::Number => Ok(value.is_number()),
            Type::String => Ok(value.is_string()),
            Type::Boolean => Ok(matches!(value, Value::Literal(crate::ast::Literal::Boolean(_)))),
            Type::Symbol => Ok(value.is_symbol()),
            Type::List(_) => Ok(value.is_list()),
            Type::Vector(_) => Ok(value.is_vector()),
            Type::Pair(_, _) => Ok(value.is_pair()),
            _ => {
                // For complex types, use R7RS integration
                self.r7rs_integration.validate_gradual_typing(ty, value)
            }
        }
    }

    /// Migrates a function from dynamic to static typing.
    pub fn migrate_function(&mut self, name: String, type_scheme: TypeScheme) -> Result<()> {
        // Add to migration state
        self.migration_state.static_functions.insert(name.clone());
        
        // Update type environment
        self.type_checker.env_mut().bind(name.clone()), type_scheme);
        
        // Check for potential issues
        self.check_migration_issues(&name)?;
        
        Ok(())
    }

    /// Checks for potential issues when migrating a function.
    fn check_migration_issues(&mut self, name: &str) -> Result<()> {
        // Check if function is called with incompatible types
        // This is simplified - a real implementation would analyze call sites
        
        if name.starts_with("string-") && self.migration_state.inferred_functions.contains_key(name) {
            self.migration_state.migration_warnings.push(MigrationWarning {
                message: format!("Function {} migrated to static typing - verify all call sites use strings", name),
                span: None,
                suggestion: Some("Add type annotations to caller functions".to_string()),
                severity: WarningSeverity::Warning,
            });
        }
        
        Ok(())
    }

    /// Gets migration warnings.
    pub fn migration_warnings(&self) -> &[MigrationWarning] {
        &self.migration_state.migration_warnings
    }

    /// Optimizes performance based on type information.
    pub fn optimize_for_types(&mut self, env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
        // Analyze usage patterns in the environment
        for name in env.all_variable_names() {
            if let Some(optimized) = self.primitive_cache.read().unwrap().get(&name) {
                self.analyze_primitive_usage(&name, optimized)?;
            }
        }
        
        Ok(())
    }

    /// Analyzes primitive usage patterns for optimization.
    fn analyze_primitive_usage(&self, name: &str, optimized: &OptimizedPrimitive) -> Result<()> {
        // Find the most common type signatures
        let mut most_common: Option<(TypeSignature, u64)> = None;
        
        for (sig, count) in &optimized.stats.type_calls {
            if let Some((_, current_max)) = &most_common {
                if count > current_max {
                    most_common = Some((sig.clone()), *count));
                }
            } else {
                most_common = Some((sig.clone()), *count));
            }
        }
        
        // Suggest specialization if beneficial
        if let Some((sig, count)) = most_common {
            if count > 100 && !optimized.specializations.contains_key(&sig) {
                println!("Suggestion: Specialize {} for signature {:?} (used {} times)", name, sig, count);
            }
        }
        
        Ok(())
    }

    /// Creates a performance report.
    pub fn performance_report(&self) -> PerformanceReport {
        let cache = self.primitive_cache.read().unwrap();
        let mut report = PerformanceReport::new();
        
        for (name, optimized) in cache.iter() {
            let primitive_report = PrimitivePerformanceReport {
                name: name.clone()),
                total_calls: optimized.stats.call_count,
                specializations: optimized.specializations.len(),
                avg_execution_time: optimized.stats.avg_execution_time,
                avg_memory_usage: optimized.stats.avg_memory_usage,
            };
            report.add_primitive(primitive_report);
        }
        
        report
    }
}

impl MigrationState {
    /// Creates a new empty migration state.
    pub fn new() -> Self {
        Self {
            static_functions: HashSet::new(),
            annotated_functions: HashMap::new(),
            inferred_functions: HashMap::new(),
            migration_warnings: Vec::new(),
        }
    }

    /// Checks if a function has been migrated to static typing.
    pub fn is_static(&self, name: &str) -> bool {
        self.static_functions.contains(name)
    }

    /// Gets the migration progress as a percentage.
    pub fn progress(&self) -> f64 {
        let total = self.static_functions.len() + self.annotated_functions.len() + self.inferred_functions.len();
        if total == 0 {
            0.0
        } else {
            (self.static_functions.len() as f64 / total as f64) * 100.0
        }
    }
}

impl PrimitiveStats {
    /// Creates new empty primitive statistics.
    pub fn new() -> Self {
        Self {
            call_count: 0,
            type_calls: HashMap::new(),
            avg_execution_time: 0,
            avg_memory_usage: 0,
        }
    }

    /// Records a call with the given type signature and performance metrics.
    pub fn record_call(&mut self, sig: TypeSignature, execution_time: u64, memory_usage: u64) {
        self.call_count += 1;
        *self.type_calls.entry(sig).or_insert(0) += 1;
        
        // Update running averages
        self.avg_execution_time = ((self.avg_execution_time * (self.call_count - 1)) + execution_time) / self.call_count;
        self.avg_memory_usage = ((self.avg_memory_usage * (self.call_count - 1)) + memory_usage) / self.call_count;
    }
}

/// Performance report for the type system integration.
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Individual primitive reports
    pub primitives: Vec<PrimitivePerformanceReport>,
    /// Overall statistics
    pub total_calls: u64,
    /// Total specializations created
    pub total_specializations: usize,
}

/// Performance report for a single primitive.
#[derive(Debug, Clone)]
pub struct PrimitivePerformanceReport {
    /// Primitive name
    pub name: String,
    /// Total number of calls
    pub total_calls: u64,
    /// Number of specializations
    pub specializations: usize,
    /// Average execution time (nanoseconds)
    pub avg_execution_time: u64,
    /// Average memory usage (bytes)
    pub avg_memory_usage: u64,
}

impl PerformanceReport {
    /// Creates a new empty performance report.
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
            total_calls: 0,
            total_specializations: 0,
        }
    }

    /// Adds a primitive report.
    pub fn add_primitive(&mut self, report: PrimitivePerformanceReport) {
        self.total_calls += report.total_calls;
        self.total_specializations += report.specializations;
        self.primitives.push(report);
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            infer_primitive_types: true,
            specialize_primitives: true,
            gradual_typing: true,
            compile_patterns: true,
            max_recursion_depth: 100,
            optimization_level: OptimizationLevel::Basic,
        }
    }
}

impl Default for MigrationState {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PrimitiveStats {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WarningSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarningSeverity::Info => write!(f, "INFO"),
            WarningSeverity::Warning => write!(f, "WARNING"),
            WarningSeverity::Error => write!(f, "ERROR"),
        }
    }
}

impl fmt::Display for MigrationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.severity, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (Suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Type System Performance Report")?;
        writeln!(f, "==============================")?;
        writeln!(f, "Total calls: {}", self.total_calls)?;
        writeln!(f, "Total specializations: {}", self.total_specializations)?;
        writeln!(f)?;
        
        for primitive in &self.primitives {
            writeln!(f, "Primitive: {}", primitive.name)?;
            writeln!(f, "  Calls: {}", primitive.total_calls)?;
            writeln!(f, "  Specializations: {}", primitive.specializations)?;
            writeln!(f, "  Avg execution time: {}ns", primitive.avg_execution_time)?;
            writeln!(f, "  Avg memory usage: {} bytes", primitive.avg_memory_usage)?;
            writeln!(f)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let config = IntegrationConfig::default();
        let bridge = TypeSystemBridge::new(config);
        assert_eq!(bridge.type_level(), TypeLevel::Dynamic);
    }

    #[test]
    fn test_primitive_integration() {
        let config = IntegrationConfig::default();
        let mut bridge = TypeSystemBridge::new(config);
        
        let add_primitive = PrimitiveProcedure {
            name: "+".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|_| Ok(Value::Unspecified)),
            effects: vec![crate::effects::Effect::Pure],
        };
        
        let result = bridge.integrate_primitive("+".to_string(), add_primitive);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_type_matching() {
        let config = IntegrationConfig::default();
        let bridge = TypeSystemBridge::new(config);
        
        let number_val = Value::integer(42);
        let string_val = Value::string("hello");
        
        assert!(bridge.value_matches_type(&number_val, &Type::Number).unwrap());
        assert!(!bridge.value_matches_type(&number_val, &Type::String).unwrap());
        assert!(bridge.value_matches_type(&string_val, &Type::String).unwrap());
        assert!(bridge.value_matches_type(&number_val, &Type::Dynamic).unwrap());
    }

    #[test]
    fn test_migration_state() {
        let mut state = MigrationState::new();
        assert_eq!(state.progress(), 0.0);
        
        state.static_functions.insert("test-func".to_string());
        state.annotated_functions.insert("other-func".to_string(), TypeScheme::monomorphic(Type::Number));
        
        assert!(state.is_static("test-func"));
        assert!(!state.is_static("other-func"));
        assert_eq!(state.progress(), 50.0);
    }

    #[test]
    fn test_primitive_stats() {
        let mut stats = PrimitiveStats::new();
        
        let sig = TypeSignature {
            params: vec![Type::Number, Type::Number],
            return_type: Type::Number,
        };
        
        stats.record_call(sig.clone()), 100, 64);
        stats.record_call(sig.clone()), 200, 128);
        
        assert_eq!(stats.call_count, 2);
        assert_eq!(stats.avg_execution_time, 150);
        assert_eq!(stats.avg_memory_usage, 96);
        assert_eq!(*stats.type_calls.get(&sig).unwrap(), 2);
    }

    #[test]
    fn test_performance_report() {
        let mut report = PerformanceReport::new();
        
        let primitive_report = PrimitivePerformanceReport {
            name: "+".to_string(),
            total_calls: 100,
            specializations: 2,
            avg_execution_time: 50,
            avg_memory_usage: 32,
        };
        
        report.add_primitive(primitive_report);
        
        assert_eq!(report.total_calls, 100);
        assert_eq!(report.total_specializations, 2);
        assert_eq!(report.primitives.len(), 1);
    }
}