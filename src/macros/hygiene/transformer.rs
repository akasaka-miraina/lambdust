//! Hygienic syntax-rules transformer
//!
//! Implements the core hygienic macro transformation logic, handling symbol renaming
//! and hygiene preservation during macro expansion.

use super::symbol::{HygienicSymbol, MacroSite};
use super::environment::{HygienicEnvironment, SymbolResolution};
use super::context::ExpansionContext;
use super::renaming::{RenamingStrategy, SymbolRenamer};
use crate::macros::{Pattern, SyntaxRule, Template, NestedEllipsisProcessor, BindingValue};
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;

/// Pattern bindings for macro expansion
pub type PatternBindings = HashMap<String, Expr>;

/// Advanced optimization configuration for hygienic transformers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    /// Minimal optimization for debugging
    Development,
    /// Balanced optimization for general use
    Balanced,
    /// Maximum optimization for production
    Production,
    /// Custom optimization with specific settings
    Custom {
        /// Enable pattern and template caching
        enable_caching: bool,
        /// Enable machine learning-inspired renaming heuristics
        enable_intelligent_renaming: bool,
        /// Enable scope depth analysis for conflict detection
        enable_scope_analysis: bool,
        /// Enable advanced pattern matching optimization
        enable_pattern_optimization: bool,
    },
}

/// Performance metrics for transformer operations
#[derive(Debug, Clone, Default)]
pub struct TransformerMetrics {
    /// Total transformations performed
    pub transformations_count: u64,
    /// Successful transformations
    pub successful_transformations: u64,
    /// Pattern matching attempts
    pub pattern_matches_attempted: u64,
    /// Successful pattern matches
    pub pattern_matches_successful: u64,
    /// Template substitutions performed
    pub template_substitutions: u64,
    /// Symbol renamings performed
    pub symbol_renamings: u64,
    /// Total processing time (nanoseconds)
    pub total_processing_time_ns: u64,
    /// Cache hits for pattern matching
    pub pattern_cache_hits: u64,
    /// Cache misses for pattern matching
    pub pattern_cache_misses: u64,
}

/// Hygienic version of syntax-rules transformer with advanced optimization
#[derive(Debug, Clone)]
pub struct HygienicSyntaxRulesTransformer {
    /// Literal identifiers that shouldn't be renamed
    pub literals: Vec<String>,
    /// Transformation rules
    pub rules: Vec<SyntaxRule>,
    /// Symbol renaming strategy
    pub renaming_strategy: RenamingStrategy,
    /// Advanced symbol renamer instance
    pub symbol_renamer: SymbolRenamer,
    /// Lexical environment at definition site
    pub definition_environment: Rc<HygienicEnvironment>,
    /// Macro name for debugging
    pub macro_name: String,
    /// Optimization level for performance tuning
    pub optimization_level: OptimizationLevel,
    /// Performance metrics
    pub metrics: TransformerMetrics,
    /// Pattern matching cache for performance
    pub pattern_cache: HashMap<String, bool>,
    /// Template substitution cache
    pub template_cache: HashMap<String, Expr>,
    /// SRFI 46 nested ellipsis processor
    pub ellipsis_processor: NestedEllipsisProcessor,
    /// Enable SRFI 46 nested ellipsis support
    pub enable_srfi46: bool,
}

impl HygienicSyntaxRulesTransformer {
    /// Create new hygienic transformer with intelligent defaults
    pub fn new(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::intelligent();
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::Intelligent,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Balanced,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true, // Enable by default for modern Scheme compatibility
        }
    }
    
    /// Create transformer with custom renaming strategy
    pub fn with_renaming_strategy(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
        strategy: RenamingStrategy,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::new(strategy.clone());
        
        Self {
            literals,
            rules,
            renaming_strategy: strategy,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Balanced,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true,
        }
    }
    
    /// Create high-performance transformer for production use
    pub fn optimized(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut symbol_renamer = SymbolRenamer::optimized();
        // Configure for high-performance macro usage
        symbol_renamer.set_strategy(RenamingStrategy::PerformanceOptimized);
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::PerformanceOptimized,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Production,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::with_capacity(1000), // Pre-allocated cache
            template_cache: HashMap::with_capacity(500),
            ellipsis_processor: NestedEllipsisProcessor::with_max_depth(15), // Higher depth for production
            enable_srfi46: true,
        }
    }
    
    /// Create scope-aware transformer for complex macro systems
    pub fn scope_aware(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::scope_aware();
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::ScopeAware,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Custom {
                enable_caching: true,
                enable_intelligent_renaming: false,
                enable_scope_analysis: true,
                enable_pattern_optimization: true,
            },
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true,
        }
    }
    
    /// Create transformer with custom optimization settings
    pub fn with_optimization(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
        optimization: OptimizationLevel,
    ) -> Self {
        let (symbol_renamer, strategy) = match &optimization {
            OptimizationLevel::Development => {
                (SymbolRenamer::new(RenamingStrategy::Conservative), RenamingStrategy::Conservative)
            }
            OptimizationLevel::Balanced => {
                (SymbolRenamer::intelligent(), RenamingStrategy::Intelligent)
            }
            OptimizationLevel::Production => {
                (SymbolRenamer::optimized(), RenamingStrategy::PerformanceOptimized)
            }
            OptimizationLevel::Custom { enable_intelligent_renaming, .. } => {
                if *enable_intelligent_renaming {
                    (SymbolRenamer::intelligent(), RenamingStrategy::Intelligent)
                } else {
                    (SymbolRenamer::scope_aware(), RenamingStrategy::ScopeAware)
                }
            }
        };
        
        let (pattern_capacity, template_capacity) = match optimization {
            OptimizationLevel::Production => (1000, 500),
            OptimizationLevel::Balanced => (200, 100),
            _ => (50, 25),
        };
        
        let ellipsis_depth = match optimization {
            OptimizationLevel::Production => 20,
            OptimizationLevel::Balanced => 10,
            _ => 5,
        };
        
        Self {
            literals,
            rules,
            renaming_strategy: strategy,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: optimization,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::with_capacity(pattern_capacity),
            template_cache: HashMap::with_capacity(template_capacity),
            ellipsis_processor: NestedEllipsisProcessor::with_max_depth(ellipsis_depth),
            enable_srfi46: true,
        }
    }
    
    /// Apply hygienic transformation with advanced optimization and safety checks (mutable version)
    pub fn transform_hygienic_optimized(
        &mut self,
        input: &[Expr],
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.metrics.transformations_count += 1;
        
        // Check optimization-specific preprocessing
        let should_use_cache = matches!(self.optimization_level,
            OptimizationLevel::Custom { enable_pattern_optimization: true, .. } 
            | OptimizationLevel::Production 
            | OptimizationLevel::Balanced
        );
        
        if should_use_cache {
            // Try cache lookup for performance
            let cache_key = format!("{}:{}", self.macro_name, input.len());
            if let Some(cached_result) = self.template_cache.get(&cache_key).cloned() {
                self.metrics.successful_transformations += 1;
                self.update_processing_time(start_time);
                return Ok(cached_result);
            }
        }
        
        // Reconstruct the full expression including macro name
        let mut full_expr = vec![Expr::Variable(self.macro_name.clone())];
        full_expr.extend_from_slice(input);
        let expr = Expr::List(full_expr);
        
        // Try each rule until one matches with performance tracking
        let rules_len = self.rules.len();
        for rule_index in 0..rules_len {
            self.metrics.pattern_matches_attempted += 1;
            
            // Clone the rule to avoid borrowing issues
            let rule = self.rules[rule_index].clone();
            
            match self.match_pattern_optimized(&rule.pattern, &expr, usage_environment, rule_index) {
                Ok(bindings) => {
                    self.metrics.pattern_matches_successful += 1;
                    
                    match self.substitute_template_optimized(
                        &rule.template,
                        bindings,
                        usage_environment,
                    ) {
                        Ok(result) => {
                            self.metrics.successful_transformations += 1;
                            self.metrics.template_substitutions += 1;
                            
                            // Cache result if optimization is enabled
                            if should_use_cache {
                                let cache_key = format!("{}:{}", self.macro_name, input.len());
                                self.template_cache.insert(cache_key, result.clone());
                            }
                            
                            self.update_processing_time(start_time);
                            return Ok(result);
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(_) => {} // Try next rule
            }
        }
        
        self.update_processing_time(start_time);
        Err(LambdustError::runtime_error(format!(
            "No matching rule for macro {} with {} arguments. Rules attempted: {}, Pattern matches: {}/{}",
            self.macro_name,
            input.len(),
            self.rules.len(),
            self.metrics.pattern_matches_successful,
            self.metrics.pattern_matches_attempted
        )))
    }
    
    /// Apply hygienic transformation (immutable compatibility version)
    pub fn transform_hygienic(
        &self,
        input: &[Expr],
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Reconstruct the full expression including macro name
        let mut full_expr = vec![Expr::Variable(self.macro_name.clone())];
        full_expr.extend_from_slice(input);
        let expr = Expr::List(full_expr);
        
        // Try each rule until one matches (without advanced optimization)
        for rule in &self.rules {
            if let Ok(bindings) = self.match_pattern(&rule.pattern, &expr, usage_environment) {
                return self.substitute_template(
                    &rule.template,
                    bindings,
                    usage_environment,
                );
            }
        }
        
        Err(LambdustError::runtime_error(format!(
            "No matching rule for macro {} with {} arguments",
            self.macro_name,
            input.len()
        )))
    }
    
    /// Update processing time metrics
    fn update_processing_time(&mut self, start_time: u64) {
        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.metrics.total_processing_time_ns += end_time.saturating_sub(start_time);
    }
    
    /// Optimized pattern matching with caching and performance tracking
    fn match_pattern_optimized(
        &mut self,
        pattern: &Pattern,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
        rule_index: usize,
    ) -> Result<PatternBindings> {
        let should_cache = matches!(self.optimization_level, 
            OptimizationLevel::Production | OptimizationLevel::Balanced);
        
        // Check cache for pattern matching results if optimization is enabled
        if should_cache {
            let cache_key = format!("{}:{}:{}", self.macro_name, rule_index, input);
            if let Some(&cached_result) = self.pattern_cache.get(&cache_key) {
                self.metrics.pattern_cache_hits += 1;
                return if cached_result {
                    let mut bindings = HashMap::new();
                    self.match_pattern_recursive(pattern, input, &mut bindings, usage_environment)?;
                    Ok(bindings)
                } else {
                    Err(LambdustError::runtime_error("Cached pattern mismatch".to_string()))
                };
            }
            self.metrics.pattern_cache_misses += 1;
        }
        
        let mut bindings = HashMap::new();
        let result = self.match_pattern_recursive(pattern, input, &mut bindings, usage_environment);
        
        // Cache the result
        if should_cache {
            let cache_key = format!("{}:{}:{}", self.macro_name, rule_index, input);
            self.pattern_cache.insert(cache_key, result.is_ok());
        }
        
        result.map(|()| bindings)
    }
    
    /// Match pattern against input with hygiene (original method)
    fn match_pattern(
        &self,
        pattern: &Pattern,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
    ) -> Result<PatternBindings> {
        let mut bindings = HashMap::new();
        let mut cloned_self = self.clone();
        cloned_self.match_pattern_recursive(pattern, input, &mut bindings, usage_environment)?;
        Ok(bindings)
    }
    
    /// Recursive pattern matching
    fn match_pattern_recursive(
        &mut self,
        pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match pattern {
            Pattern::Literal(literal) => {
                self.match_literal(literal, expr)
            }
            Pattern::Variable(var) => {
                bindings.insert(var.clone(), expr.clone());
                Ok(())
            }
            Pattern::List(patterns) => {
                self.match_list_pattern(patterns, expr, bindings, usage_environment)
            }
            Pattern::Ellipsis(sub_pattern) => {
                self.match_ellipsis_pattern(sub_pattern, expr, bindings, usage_environment)
            }
            Pattern::NestedEllipsis(sub_pattern, level) => {
                // SRFI 46 nested ellipsis support
                if self.enable_srfi46 {
                    self.match_nested_ellipsis_pattern(sub_pattern, expr, *level, bindings, usage_environment)
                } else {
                    // Fallback to regular ellipsis
                    self.match_ellipsis_pattern(sub_pattern, expr, bindings, usage_environment)
                }
            }
            Pattern::Dotted(_, _) | Pattern::Vector(_) => {
                // TODO: Implement dotted and vector pattern support
                Err(LambdustError::runtime_error("Dotted and vector patterns not yet supported in hygienic transformer".to_string()))
            }
            Pattern::HygienicVariable(_) => {
                // Hygienic variables always match
                Ok(())
            }
            Pattern::SyntaxObject(inner_pattern) => {
                // Match against the inner pattern
                self.match_pattern_recursive(inner_pattern, expr, bindings, usage_environment)
            }
            Pattern::Any => {
                // Any pattern always matches
                Ok(())
            }
            // Advanced patterns not yet implemented in hygienic transformer
            Pattern::Conditional { .. } | Pattern::TypeGuard { .. } | Pattern::And(_)
            | Pattern::Or(_) | Pattern::Not(_) | Pattern::Range { .. } | Pattern::Regex(_) => {
                Err(LambdustError::runtime_error(
                    "Advanced patterns not yet supported in hygienic transformer".to_string(),
                ))
            }
        }
    }
    
    /// Match literal pattern
    fn match_literal(&self, literal: &str, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Variable(name) | Expr::HygienicVariable(HygienicSymbol { name, .. }) => {
                if name == literal {
                    Ok(())
                } else {
                    Err(LambdustError::runtime_error(format!(
                        "Expected literal '{literal}', got '{name}'"
                    )))
                }
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Expected literal '{literal}', got non-symbol"
            ))),
        }
    }
    
    /// Match list pattern
    fn match_list_pattern(
        &mut self,
        patterns: &[Pattern],
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match expr {
            Expr::List(exprs) => {
                if patterns.len() != exprs.len() {
                    return Err(LambdustError::runtime_error(format!(
                        "Pattern length {} doesn't match expression length {}",
                        patterns.len(),
                        exprs.len()
                    )));
                }
                
                for (pattern, expr) in patterns.iter().zip(exprs.iter()) {
                    self.match_pattern_recursive(pattern, expr, bindings, usage_environment)?;
                }
                Ok(())
            }
            _ => Err(LambdustError::runtime_error(
                "Expected list for list pattern".to_string(),
            )),
        }
    }
    
    /// Match ellipsis pattern
    fn match_ellipsis_pattern(
        &mut self,
        sub_pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match expr {
            Expr::List(exprs) => {
                // Match each expression against the sub-pattern
                for expr in exprs {
                    self.match_pattern_recursive(sub_pattern, expr, bindings, usage_environment)?;
                }
                Ok(())
            }
            _ => {
                // Single expression, try to match against sub-pattern
                self.match_pattern_recursive(sub_pattern, expr, bindings, usage_environment)
            }
        }
    }
    
    /// Optimized template substitution with advanced symbol renaming
    fn substitute_template_optimized(
        &mut self,
        template: &Template,
        bindings: PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Create expansion context
        let mut expansion_context = ExpansionContext::new(
            (*self.definition_environment).clone(),
            usage_environment.clone()
        );
        
        // Add pattern bindings to context
        for (name, expr) in &bindings {
            if let Some(symbol) = self.extract_symbol_from_binding(expr) {
                expansion_context.bind_symbol(name.clone(), symbol);
            }
        }
        
        // Substitute template with optimization tracking
        let result = self.substitute_template_recursive(
            template,
            &bindings,
            &mut expansion_context,
            usage_environment,
        )?;
        
        // Apply advanced hygienic renaming with integrated symbol renamer
        let renamed_result = self.symbol_renamer.rename_symbols(
            &result, 
            &mut expansion_context, 
            usage_environment
        )?;
        
        // Update symbol renaming metrics
        let renamer_stats = self.symbol_renamer.performance_stats();
        self.metrics.symbol_renamings += renamer_stats.symbols_renamed;
        
        Ok(renamed_result)
    }
    
    /// Substitute template with hygienic renaming (original method)
    fn substitute_template(
        &self,
        template: &Template,
        bindings: PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Create expansion context
        let mut expansion_context = ExpansionContext::new(
            (*self.definition_environment).clone(),
            usage_environment.clone()
        );
        
        // Add pattern bindings to context
        for (name, expr) in &bindings {
            if let Some(symbol) = self.extract_symbol_from_binding(expr) {
                expansion_context.bind_symbol(name.clone(), symbol);
            }
        }
        
        // Substitute template
        let mut cloned_self = self.clone();
        let result = cloned_self.substitute_template_recursive(
            template,
            &bindings,
            &mut expansion_context,
            usage_environment,
        )?;
        
        // Apply hygienic renaming using internal renamer (immutable access)
        let mut temp_renamer = self.symbol_renamer.clone();
        temp_renamer.rename_symbols(&result, &mut expansion_context, usage_environment)
    }
    
    /// Recursive template substitution
    fn substitute_template_recursive(
        &mut self,
        template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match template {
            Template::Literal(literal) => {
                // Check if literal should be treated hygienically
                if self.literals.contains(literal) {
                    // Literal identifiers maintain their identity
                    Ok(Expr::Variable(literal.clone()))
                } else {
                    // Other literals might be hygienic symbols from definition site
                    self.resolve_template_symbol(literal, expansion_context)
                }
            }
            Template::Variable(var) => {
                if let Some(bound_expr) = bindings.get(var) {
                    Ok(bound_expr.clone())
                } else {
                    // Unbound template variable - resolve from definition environment
                    self.resolve_template_symbol(var, expansion_context)
                }
            }
            Template::List(templates) => {
                let substituted: Result<Vec<_>> = templates
                    .iter()
                    .map(|t| self.substitute_template_recursive(t, bindings, expansion_context, usage_environment))
                    .collect();
                Ok(Expr::List(substituted?))
            }
            Template::Ellipsis(sub_template) => {
                // Handle ellipsis expansion
                self.expand_ellipsis_template(sub_template, bindings, expansion_context, usage_environment)
            }
            Template::NestedEllipsis(sub_template, level) => {
                // SRFI 46 nested ellipsis support
                if self.enable_srfi46 {
                    self.expand_nested_ellipsis_template(sub_template, bindings, *level, expansion_context, usage_environment)
                } else {
                    // Fallback to regular ellipsis
                    self.expand_ellipsis_template(sub_template, bindings, expansion_context, usage_environment)
                }
            }
            Template::Dotted(_, _) | Template::Vector(_) => {
                // TODO: Implement dotted and vector template support
                Err(LambdustError::runtime_error("Dotted and vector templates not yet supported in hygienic transformer".to_string()))
            }
            Template::HygienicVariable(symbol) => {
                // Generate fresh hygienic identifier
                let fresh_name = expansion_context.generate_template_symbol(symbol.original_name());
                Ok(Expr::Variable(fresh_name.unique_name()))
            }
            Template::SyntaxObject(inner_template) => {
                // Process inner template as syntax object
                self.substitute_template_recursive(inner_template, bindings, expansion_context, usage_environment)
            }
            // Advanced templates not yet implemented in hygienic transformer
            Template::Conditional { .. } | Template::Repeat { .. } | Template::Transform { .. } => {
                Err(LambdustError::runtime_error(
                    "Advanced templates not yet supported in hygienic transformer".to_string(),
                ))
            }
        }
    }
    
    /// Resolve symbol from template
    fn resolve_template_symbol(
        &self,
        name: &str,
        expansion_context: &mut ExpansionContext,
    ) -> Result<Expr> {
        // First check if we have a hygienic binding in expansion context
        if let Some(symbol) = expansion_context.lookup_symbol(name) {
            return Ok(Expr::HygienicVariable(symbol.clone()));
        }
        
        // Check definition environment for symbol
        match self.definition_environment.resolve_symbol(name) {
            SymbolResolution::Hygienic(symbol) => {
                Ok(Expr::HygienicVariable(symbol))
            }
            SymbolResolution::Traditional(_) => {
                // Use traditional variable but mark for potential renaming
                Ok(Expr::Variable(name.to_string()))
            }
            SymbolResolution::Unbound(_) => {
                // Generate new hygienic symbol for introduced variable
                let symbol = expansion_context.generate_template_symbol(name);
                Ok(Expr::HygienicVariable(symbol))
            }
        }
    }
    
    /// Expand ellipsis template
    fn expand_ellipsis_template(
        &mut self,
        sub_template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // For simplicity, expand once for now
        // Real implementation would need to handle multiple ellipsis variables
        let expanded = self.substitute_template_recursive(
            sub_template,
            bindings,
            expansion_context,
            usage_environment,
        )?;
        Ok(expanded)
    }
    
    /// Match nested ellipsis pattern (SRFI 46)
    fn match_nested_ellipsis_pattern(
        &mut self,
        sub_pattern: &Pattern,
        expr: &Expr,
        nesting_level: usize,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        // Create temporary expansion context for ellipsis processing
        let expansion_context = ExpansionContext::new(
            (*self.definition_environment).clone(),
            usage_environment.clone()
        );
        
        // Use the nested ellipsis processor
        match self.ellipsis_processor.match_nested_ellipsis(
            &Pattern::NestedEllipsis(Box::new(sub_pattern.clone()), nesting_level),
            expr,
            nesting_level,
            &expansion_context,
        ) {
            Ok(match_result) => {
                if match_result.success {
                    // Merge the bindings from ellipsis matching
                    for (var, binding_value) in match_result.bindings {
                        let expr = match binding_value {
                            BindingValue::Single(expr) => expr,
                            BindingValue::List(exprs) => Expr::List(exprs),
                            BindingValue::SyntaxObject(obj) => obj.expression,
                        };
                        bindings.insert(var, expr);
                    }
                    Ok(())
                } else {
                    Err(LambdustError::syntax_error(format!(
                        "Failed to match nested ellipsis pattern at level {nesting_level}"
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }
    
    /// Expand nested ellipsis template (SRFI 46)
    fn expand_nested_ellipsis_template(
        &mut self,
        sub_template: &Template,
        bindings: &PatternBindings,
        nesting_level: usize,
        expansion_context: &mut ExpansionContext,
        _usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Convert PatternBindings to the format expected by ellipsis processor
        let ellipsis_bindings = self.convert_pattern_bindings_to_ellipsis(bindings);
        
        // Use the nested ellipsis processor for expansion
        match self.ellipsis_processor.expand_nested_ellipsis(
            &Template::NestedEllipsis(Box::new(sub_template.clone()), nesting_level),
            &ellipsis_bindings,
            nesting_level,
            expansion_context,
        ) {
            Ok(expanded) => Ok(expanded),
            Err(e) => Err(e),
        }
    }
    
    /// Convert `PatternBindings` to format expected by ellipsis processor
    fn convert_pattern_bindings_to_ellipsis(
        &self,
        bindings: &PatternBindings,
    ) -> HashMap<String, crate::macros::BindingValue> {
        let mut ellipsis_bindings = HashMap::new();
        
        for (var, expr) in bindings {
            let binding_value = crate::macros::BindingValue::Single(expr.clone());
            ellipsis_bindings.insert(var.clone(), binding_value);
        }
        
        ellipsis_bindings
    }
    
    /// Extract hygienic symbol from pattern binding
    fn extract_symbol_from_binding(&self, expr: &Expr) -> Option<HygienicSymbol> {
        match expr {
            Expr::HygienicVariable(symbol) => Some(symbol.clone()),
            Expr::Variable(name) => {
                // Convert traditional variable to hygienic symbol
                let env_id = self.definition_environment.id;
                let macro_site = MacroSite::new(
                    self.macro_name.clone(),
                    0,
                    env_id,
                );
                let symbol_id = super::generator::SymbolGenerator::generate_symbol_id();
                Some(HygienicSymbol::new(name.clone(), symbol_id, macro_site))
            }
            _ => None,
        }
    }
    
    /// Get comprehensive performance metrics
    #[must_use] pub fn performance_metrics(&self) -> &TransformerMetrics {
        &self.metrics
    }
    
    /// Get detailed performance analysis
    #[must_use] pub fn performance_analysis(&self) -> String {
        let success_rate = if self.metrics.transformations_count > 0 {
            (self.metrics.successful_transformations as f64 / self.metrics.transformations_count as f64) * 100.0
        } else {
            0.0
        };
        
        let pattern_match_rate = if self.metrics.pattern_matches_attempted > 0 {
            (self.metrics.pattern_matches_successful as f64 / self.metrics.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        };
        
        let cache_hit_rate = if self.metrics.pattern_cache_hits + self.metrics.pattern_cache_misses > 0 {
            (self.metrics.pattern_cache_hits as f64 / (self.metrics.pattern_cache_hits + self.metrics.pattern_cache_misses) as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_processing_time = if self.metrics.transformations_count > 0 {
            (self.metrics.total_processing_time_ns as f64 / self.metrics.transformations_count as f64) / 1000.0
        } else {
            0.0
        };
        
        format!(
            "Hygienic Transformer Performance Analysis:\n\
             Macro: {}\n\
             Optimization Level: {:?}\n\
             Renaming Strategy: {:?}\n\
             ─────────────────────────────────────────\n\
             Transformations: {} total, {} successful ({:.1}%)\n\
             Pattern Matching: {}/{} successful ({:.1}%)\n\
             Template Substitutions: {}\n\
             Symbol Renamings: {}\n\
             Cache Performance: {:.1}% hit rate ({}/{} requests)\n\
             Processing Time: {:.2}μs average\n\
             Total Processing: {:.2}ms\n\
             Rules: {} defined\n\
             Cache Size: {} patterns, {} templates",
            self.macro_name,
            self.optimization_level,
            self.renaming_strategy,
            self.metrics.transformations_count,
            self.metrics.successful_transformations,
            success_rate,
            self.metrics.pattern_matches_successful,
            self.metrics.pattern_matches_attempted,
            pattern_match_rate,
            self.metrics.template_substitutions,
            self.metrics.symbol_renamings,
            cache_hit_rate,
            self.metrics.pattern_cache_hits,
            self.metrics.pattern_cache_hits + self.metrics.pattern_cache_misses,
            avg_processing_time,
            self.metrics.total_processing_time_ns as f64 / 1_000_000.0,
            self.rules.len(),
            self.pattern_cache.len(),
            self.template_cache.len()
        )
    }
    
    /// Get optimization recommendations
    #[must_use] pub fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze success rates
        let success_rate = if self.metrics.transformations_count > 0 {
            (self.metrics.successful_transformations as f64 / self.metrics.transformations_count as f64) * 100.0
        } else {
            0.0
        };
        
        if success_rate < 80.0 {
            recommendations.push("Consider reviewing macro rules for better pattern coverage".to_string());
        }
        
        // Analyze cache performance
        let cache_hit_rate = if self.metrics.pattern_cache_hits + self.metrics.pattern_cache_misses > 0 {
            (self.metrics.pattern_cache_hits as f64 / (self.metrics.pattern_cache_hits + self.metrics.pattern_cache_misses) as f64) * 100.0
        } else {
            0.0
        };
        
        if cache_hit_rate < 50.0 && matches!(self.optimization_level, OptimizationLevel::Development) {
            recommendations.push("Consider upgrading to Balanced or Production optimization level".to_string());
        }
        
        // Analyze processing time
        let avg_processing_time = if self.metrics.transformations_count > 0 {
            self.metrics.total_processing_time_ns as f64 / self.metrics.transformations_count as f64
        } else {
            0.0
        };
        
        if avg_processing_time > 10000.0 { // 10μs
            match self.optimization_level {
                OptimizationLevel::Development => {
                    recommendations.push("Consider upgrading to Balanced optimization for better performance".to_string());
                }
                OptimizationLevel::Balanced => {
                    recommendations.push("Consider upgrading to Production optimization for maximum performance".to_string());
                }
                _ => {
                    recommendations.push("Consider using PerformanceOptimized renaming strategy".to_string());
                }
            }
        }
        
        // Analyze rule efficiency
        let pattern_match_rate = if self.metrics.pattern_matches_attempted > 0 {
            (self.metrics.pattern_matches_successful as f64 / self.metrics.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        };
        
        if pattern_match_rate < 30.0 {
            recommendations.push("Consider reordering rules to put more common patterns first".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is optimal for current usage pattern".to_string());
        }
        
        recommendations
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = TransformerMetrics::default();
        if matches!(self.optimization_level, OptimizationLevel::Production | OptimizationLevel::Balanced) {
            self.pattern_cache.clear();
            self.template_cache.clear();
        }
    }
    
    /// Optimize caches based on usage patterns
    pub fn optimize_caches(&mut self) {
        // Remove least recently used entries if caches are too large
        if self.pattern_cache.len() > 2000 {
            self.pattern_cache.clear();
        }
        
        if self.template_cache.len() > 1000 {
            self.template_cache.clear();
        }
        
        // Optimize symbol renamer caches
        self.symbol_renamer.optimize_caches();
    }
    
    /// Get current optimization level
    #[must_use] pub fn optimization_level(&self) -> &OptimizationLevel {
        &self.optimization_level
    }
    
    /// Change optimization level (clears caches)
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
        self.pattern_cache.clear();
        self.template_cache.clear();
        
        // Update symbol renamer strategy based on optimization level
        let new_strategy = match level {
            OptimizationLevel::Development => RenamingStrategy::Conservative,
            OptimizationLevel::Balanced => RenamingStrategy::Intelligent,
            OptimizationLevel::Production => RenamingStrategy::PerformanceOptimized,
            OptimizationLevel::Custom { enable_intelligent_renaming: true, .. } => {
                RenamingStrategy::Intelligent
            }
            OptimizationLevel::Custom { enable_scope_analysis: true, .. } => {
                RenamingStrategy::ScopeAware
            }
            _ => RenamingStrategy::Conservative,
        };
        
        self.symbol_renamer.set_strategy(new_strategy.clone());
        self.renaming_strategy = new_strategy;
    }
    
    /// Get symbol renamer statistics
    #[must_use] pub fn renamer_stats(&self) -> &super::renaming::RenamingStats {
        self.symbol_renamer.performance_stats()
    }
    
    /// Enable or disable SRFI 46 nested ellipsis support
    pub fn set_srfi46_support(&mut self, enable: bool) {
        self.enable_srfi46 = enable;
    }
    
    /// Check if SRFI 46 support is enabled
    #[must_use] pub fn is_srfi46_enabled(&self) -> bool {
        self.enable_srfi46
    }
    
    /// Get SRFI 46 ellipsis processor metrics
    #[must_use] pub fn ellipsis_metrics(&self) -> &crate::macros::EllipsisMetrics {
        self.ellipsis_processor.metrics()
    }
    
    /// Create SRFI 46 enabled transformer
    pub fn with_srfi46(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut transformer = Self::new(literals, rules, definition_environment, macro_name);
        transformer.enable_srfi46 = true;
        transformer.ellipsis_processor = NestedEllipsisProcessor::with_max_depth(20);
        transformer
    }
    
    /// Create legacy transformer (SRFI 46 disabled)
    pub fn legacy(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut transformer = Self::new(literals, rules, definition_environment, macro_name);
        transformer.enable_srfi46 = false;
        transformer
    }
    
    /// Get comprehensive analysis including SRFI 46 metrics
    #[must_use] pub fn comprehensive_analysis(&self) -> String {
        let basic_analysis = self.performance_analysis();
        let ellipsis_metrics = self.ellipsis_metrics();
        
        format!(
            "{}\n\
             ─────────────────────────────────────────\n\
             SRFI 46 Nested Ellipsis Analysis:\n\
             Enabled: {}\n\
             {}",
            basic_analysis,
            self.enable_srfi46,
            ellipsis_metrics.format_summary()
        )
    }
}

impl TransformerMetrics {
    /// Get transformation success rate
    #[must_use] pub fn success_rate(&self) -> f64 {
        if self.transformations_count > 0 {
            (self.successful_transformations as f64 / self.transformations_count as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get pattern matching efficiency
    #[must_use] pub fn pattern_match_efficiency(&self) -> f64 {
        if self.pattern_matches_attempted > 0 {
            (self.pattern_matches_successful as f64 / self.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get cache hit rate
    #[must_use] pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.pattern_cache_hits + self.pattern_cache_misses;
        if total_requests > 0 {
            (self.pattern_cache_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get average processing time per transformation (microseconds)
    #[must_use] pub fn average_processing_time_us(&self) -> f64 {
        if self.transformations_count > 0 {
            (self.total_processing_time_ns as f64 / self.transformations_count as f64) / 1000.0
        } else {
            0.0
        }
    }
    
    /// Format metrics for display
    #[must_use] pub fn format_summary(&self) -> String {
        format!(
            "Transformer Metrics Summary:\n\
             Transformations: {} ({:.1}% success)\n\
             Pattern Matches: {}/{} ({:.1}% efficiency)\n\
             Template Substitutions: {}\n\
             Symbol Renamings: {}\n\
             Cache Performance: {:.1}% hit rate\n\
             Average Time: {:.2}μs per transformation",
            self.transformations_count,
            self.success_rate(),
            self.pattern_matches_successful,
            self.pattern_matches_attempted,
            self.pattern_match_efficiency(),
            self.template_substitutions,
            self.symbol_renamings,
            self.cache_hit_rate(),
            self.average_processing_time_us()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_environment() -> Rc<HygienicEnvironment> {
        Rc::new(HygienicEnvironment::new())
    }

    #[test]
    fn test_hygienic_transformer_creation() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env,
            "test-macro".to_string(),
        );
        
        assert_eq!(transformer.macro_name, "test-macro");
        assert!(transformer.literals.is_empty());
        assert!(transformer.rules.is_empty());
    }
    
    #[test]
    fn test_literal_matching() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec!["else".to_string()],
            vec![],
            env,
            "test-macro".to_string(),
        );
        
        let literal_expr = Expr::Variable("else".to_string());
        assert!(transformer.match_literal("else", &literal_expr).is_ok());
        
        let wrong_expr = Expr::Variable("other".to_string());
        assert!(transformer.match_literal("else", &wrong_expr).is_err());
    }
    
    #[test]
    fn test_pattern_variable_binding() {
        let env = create_test_environment();
        let mut transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env.clone(),
            "test-macro".to_string(),
        );
        
        let mut bindings = HashMap::new();
        let test_expr = Expr::Variable("test-value".to_string());
        
        let result = transformer.match_pattern_recursive(
            &Pattern::Variable("x".to_string()),
            &test_expr,
            &mut bindings,
            &env,
        );
        
        assert!(result.is_ok());
        assert_eq!(bindings.get("x"), Some(&test_expr));
    }
    
    #[test]
    fn test_list_pattern_matching() {
        let env = create_test_environment();
        let mut transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env.clone(),
            "test-macro".to_string(),
        );
        
        let pattern = Pattern::List(vec![
            Pattern::Literal("if".to_string()),
            Pattern::Variable("test".to_string()),
            Pattern::Variable("then".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Variable("condition".to_string()),
            Expr::Variable("body".to_string()),
        ]);
        
        let mut bindings = HashMap::new();
        let result = transformer.match_pattern_recursive(
            &pattern,
            &expr,
            &mut bindings,
            &env,
        );
        
        assert!(result.is_ok());
        assert_eq!(bindings.len(), 2);
        assert!(bindings.contains_key("test"));
        assert!(bindings.contains_key("then"));
    }
}