//! SRFI 46: Basic syntax-rules extensions - Advanced Nested Ellipsis Support
//!
//! This module implements the advanced nested ellipsis functionality defined in SRFI 46,
//! enabling sophisticated pattern matching and template expansion with multiple levels
//! of ellipsis nesting. This is one of the most complex features in Scheme macro systems.

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use super::pattern_matching::{Pattern, Template, BindingValue, MatchResult};
use super::hygiene::ExpansionContext;
use std::collections::HashMap;

/// Multi-dimensional ellipsis binding for nested ellipsis patterns
#[derive(Debug, Clone)]
pub struct MultiDimBinding {
    /// The variable name being bound
    pub variable: String,
    /// Nested structure of bound values
    pub values: MultiDimValue,
    /// Nesting depth of this binding
    pub depth: usize,
}

/// Multi-dimensional value structure for nested ellipsis
#[derive(Debug, Clone)]
pub enum MultiDimValue {
    /// Scalar value (depth 0)
    Scalar(Expr),
    /// 1D array of values (depth 1)
    Array1D(Vec<Expr>),
    /// 2D array of values (depth 2)
    Array2D(Vec<Vec<Expr>>),
    /// 3D array of values (depth 3)
    Array3D(Vec<Vec<Vec<Expr>>>),
    /// General N-dimensional structure (for arbitrary depth)
    ArrayND(Box<NDimensionalArray>),
}

/// N-dimensional array structure for arbitrary nesting depths
#[derive(Debug, Clone)]
pub struct NDimensionalArray {
    /// The nested data structure
    pub data: Vec<MultiDimValue>,
    /// Current dimension level
    pub dimension: usize,
}

/// Ellipsis expansion context for tracking nesting levels
#[derive(Debug, Clone)]
pub struct EllipsisContext {
    /// Current ellipsis nesting depth
    pub current_depth: usize,
    /// Maximum observed nesting depth
    pub max_depth: usize,
    /// Multi-dimensional bindings by variable name
    pub multi_bindings: HashMap<String, MultiDimBinding>,
    /// Ellipsis iteration counts at each level
    pub iteration_counts: Vec<usize>,
}

/// Advanced nested ellipsis processor
#[derive(Debug, Clone)]
pub struct NestedEllipsisProcessor {
    /// Maximum supported nesting depth (prevents stack overflow)
    max_nesting_depth: usize,
    /// Performance metrics
    metrics: EllipsisMetrics,
}

/// Performance metrics for ellipsis processing
#[derive(Debug, Clone, Default)]
pub struct EllipsisMetrics {
    /// Total pattern matches attempted
    pub pattern_matches_attempted: u64,
    /// Successful pattern matches
    pub pattern_matches_successful: u64,
    /// Template expansions performed
    pub template_expansions: u64,
    /// Maximum nesting depth encountered
    pub max_nesting_depth_seen: usize,
    /// Total processing time (nanoseconds)
    pub total_processing_time_ns: u64,
}

impl NestedEllipsisProcessor {
    /// Create new nested ellipsis processor
    #[must_use] pub fn new() -> Self {
        Self {
            max_nesting_depth: 10, // Reasonable safety limit
            metrics: EllipsisMetrics::default(),
        }
    }
    
    /// Get maximum nesting depth
    #[must_use] pub fn max_depth(&self) -> usize {
        self.max_nesting_depth
    }
    
    /// Create processor with custom depth limit
    #[must_use] pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_nesting_depth: max_depth,
            metrics: EllipsisMetrics::default(),
        }
    }
    
    /// Match nested ellipsis pattern against expression
    pub fn match_nested_ellipsis(
        &mut self,
        pattern: &Pattern,
        expr: &Expr,
        nesting_level: usize,
        context: &ExpansionContext,
    ) -> Result<MatchResult> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.metrics.pattern_matches_attempted += 1;
        
        // Safety check for nesting depth
        if nesting_level > self.max_nesting_depth {
            return Err(LambdustError::runtime_error(format!(
                "Nested ellipsis depth {} exceeds maximum allowed depth {}",
                nesting_level, self.max_nesting_depth
            )));
        }
        
        let mut ellipsis_ctx = EllipsisContext::new();
        let result = self.match_nested_ellipsis_recursive(
            pattern, 
            expr, 
            nesting_level, 
            context, 
            &mut ellipsis_ctx
        );
        
        // Update metrics
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.metrics.total_processing_time_ns += end_time.saturating_sub(start_time);
        self.metrics.max_nesting_depth_seen = self.metrics.max_nesting_depth_seen.max(nesting_level);
        
        if result.is_ok() {
            self.metrics.pattern_matches_successful += 1;
        }
        
        result
    }
    
    /// Recursive nested ellipsis pattern matching
    fn match_nested_ellipsis_recursive(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        nesting_level: usize,
        context: &ExpansionContext,
        ellipsis_ctx: &mut EllipsisContext,
    ) -> Result<MatchResult> {
        match pattern {
            Pattern::NestedEllipsis(sub_pattern, level) => {
                if *level != nesting_level {
                    return Err(LambdustError::syntax_error(format!(
                        "Ellipsis nesting level mismatch: expected {level}, got {nesting_level}"
                    )));
                }
                
                self.match_multi_dimensional_pattern(
                    sub_pattern, 
                    expr, 
                    nesting_level, 
                    context, 
                    ellipsis_ctx
                )
            }
            
            Pattern::List(patterns) => {
                match expr {
                    Expr::List(exprs) => {
                        self.match_nested_list_pattern(
                            patterns, 
                            exprs, 
                            nesting_level, 
                            context, 
                            ellipsis_ctx
                        )
                    }
                    _ => Ok(MatchResult::failed()),
                }
            }
            
            Pattern::Variable(var) => {
                let binding = MultiDimBinding {
                    variable: var.clone(),
                    values: MultiDimValue::Scalar(expr.clone()),
                    depth: nesting_level,
                };
                
                ellipsis_ctx.multi_bindings.insert(var.clone(), binding);
                Ok(MatchResult::success())
            }
            
            _ => {
                // Delegate to standard pattern matching for non-ellipsis patterns
                Ok(MatchResult::failed()) // Simplified for now
            }
        }
    }
    
    /// Match multi-dimensional pattern structure
    fn match_multi_dimensional_pattern(
        &self,
        sub_pattern: &Pattern,
        expr: &Expr,
        nesting_level: usize,
        context: &ExpansionContext,
        ellipsis_ctx: &mut EllipsisContext,
    ) -> Result<MatchResult> {
        match expr {
            Expr::List(exprs) => {
                let mut collected_bindings = HashMap::new();
                
                // Process each expression in the list
                for (i, sub_expr) in exprs.iter().enumerate() {
                    let mut sub_ctx = ellipsis_ctx.clone();
                    sub_ctx.current_depth = nesting_level;
                    
                    let sub_result = self.match_nested_ellipsis_recursive(
                        sub_pattern,
                        sub_expr,
                        nesting_level.saturating_sub(1),
                        context,
                        &mut sub_ctx,
                    )?;
                    
                    // Collect bindings into multi-dimensional structure
                    self.accumulate_multi_dim_bindings(
                        &sub_result,
                        &mut collected_bindings,
                        i,
                        nesting_level,
                    );
                }
                
                // Convert collected bindings to final result
                let final_result = MatchResult::success();
                for (var, multi_binding) in collected_bindings {
                    ellipsis_ctx.multi_bindings.insert(var, multi_binding);
                }
                
                Ok(final_result)
            }
            
            _ => {
                // For scalar values at the deepest level
                if nesting_level == 1 {
                    self.match_nested_ellipsis_recursive(
                        sub_pattern,
                        expr,
                        0,
                        context,
                        ellipsis_ctx,
                    )
                } else {
                    Ok(MatchResult::failed())
                }
            }
        }
    }
    
    /// Match nested list pattern with ellipsis awareness
    fn match_nested_list_pattern(
        &self,
        patterns: &[Pattern],
        exprs: &[Expr],
        nesting_level: usize,
        context: &ExpansionContext,
        ellipsis_ctx: &mut EllipsisContext,
    ) -> Result<MatchResult> {
        let mut pattern_idx = 0;
        let mut expr_idx = 0;
        let mut final_result = MatchResult::success();
        
        while pattern_idx < patterns.len() && expr_idx < exprs.len() {
            let pattern = &patterns[pattern_idx];
            
            if let Pattern::NestedEllipsis(sub_pattern, level) = pattern {
                // Handle nested ellipsis in list context
                let remaining_patterns = patterns.len() - pattern_idx - 1;
                let remaining_exprs = exprs.len() - expr_idx;
                
                if remaining_exprs < remaining_patterns {
                    return Ok(MatchResult::failed());
                }
                
                let ellipsis_count = remaining_exprs - remaining_patterns;
                
                // Process ellipsis matches
                for i in 0..ellipsis_count {
                    if expr_idx + i >= exprs.len() {
                        break;
                    }
                    
                    let sub_result = self.match_nested_ellipsis_recursive(
                        sub_pattern,
                        &exprs[expr_idx + i],
                        *level,
                        context,
                        ellipsis_ctx,
                    )?;
                    
                    // Merge results
                    self.merge_ellipsis_results(&sub_result, &mut final_result, i)?;
                }
                
                expr_idx += ellipsis_count;
                pattern_idx += 1;
            } else {
                // Regular pattern matching
                let sub_result = self.match_nested_ellipsis_recursive(
                    pattern,
                    &exprs[expr_idx],
                    nesting_level,
                    context,
                    ellipsis_ctx,
                )?;
                
                if !sub_result.success {
                    return Ok(MatchResult::failed());
                }
                
                // Merge bindings
                for (var, binding) in sub_result.bindings {
                    final_result.bindings.insert(var, binding);
                }
                
                pattern_idx += 1;
                expr_idx += 1;
            }
        }
        
        // Check if all patterns and expressions were consumed
        if pattern_idx == patterns.len() && expr_idx == exprs.len() {
            Ok(final_result)
        } else {
            Ok(MatchResult::failed())
        }
    }
    
    /// Accumulate multi-dimensional bindings from sub-matches
    fn accumulate_multi_dim_bindings(
        &self,
        sub_result: &MatchResult,
        collected: &mut HashMap<String, MultiDimBinding>,
        _index: usize,
        nesting_level: usize,
    ) {
        for (var, binding) in &sub_result.bindings {
            match binding {
                BindingValue::Single(expr) => {
                    let entry = collected.entry(var.clone()).or_insert_with(|| {
                        MultiDimBinding {
                            variable: var.clone(),
                            values: MultiDimValue::Array1D(Vec::new()),
                            depth: nesting_level,
                        }
                    });
                    
                    // Add to appropriate dimensional array
                    if let MultiDimValue::Array1D(ref mut vec) = &mut entry.values {
                        vec.push(expr.clone());
                    } else {
                        // Handle dimension mismatch
                    }
                }
                
                BindingValue::List(exprs) => {
                    let entry = collected.entry(var.clone()).or_insert_with(|| {
                        MultiDimBinding {
                            variable: var.clone(),
                            values: MultiDimValue::Array2D(Vec::new()),
                            depth: nesting_level,
                        }
                    });
                    
                    if let MultiDimValue::Array2D(ref mut vec2d) = &mut entry.values {
                        vec2d.push(exprs.clone());
                    } else {
                        // Handle dimension mismatch
                    }
                }
                
                _ => {
                    // Handle other binding types
                }
            }
        }
    }
    
    /// Merge ellipsis results into final result
    fn merge_ellipsis_results(
        &self,
        sub_result: &MatchResult,
        final_result: &mut MatchResult,
        _iteration_index: usize,
    ) -> Result<()> {
        for (var, binding) in &sub_result.bindings {
            match final_result.bindings.get_mut(var) {
                Some(BindingValue::List(ref mut list)) => {
                    match binding {
                        BindingValue::Single(expr) => {
                            list.push(expr.clone());
                        }
                        BindingValue::List(exprs) => {
                            list.extend_from_slice(exprs);
                        }
                        _ => {}
                    }
                }
                None => {
                    match binding {
                        BindingValue::Single(expr) => {
                            final_result.bindings.insert(
                                var.clone(),
                                BindingValue::List(vec![expr.clone()]),
                            );
                        }
                        BindingValue::List(exprs) => {
                            final_result.bindings.insert(
                                var.clone(),
                                BindingValue::List(exprs.clone()),
                            );
                        }
                        _ => {}
                    }
                }
                Some(_) => {
                    // Convert existing binding to list if needed
                    if let Some(existing) = final_result.bindings.remove(var) {
                        let mut new_list = match existing {
                            BindingValue::Single(expr) => vec![expr],
                            BindingValue::List(exprs) => exprs,
                            _ => vec![],
                        };
                        
                        match binding {
                            BindingValue::Single(expr) => new_list.push(expr.clone()),
                            BindingValue::List(exprs) => new_list.extend_from_slice(exprs),
                            _ => {}
                        }
                        
                        final_result.bindings.insert(
                            var.clone(),
                            BindingValue::List(new_list),
                        );
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Expand nested ellipsis template
    pub fn expand_nested_ellipsis(
        &mut self,
        template: &Template,
        bindings: &HashMap<String, BindingValue>,
        nesting_level: usize,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        self.metrics.template_expansions += 1;
        
        match template {
            Template::NestedEllipsis(sub_template, level) => {
                if *level != nesting_level {
                    return Err(LambdustError::syntax_error(format!(
                        "Template ellipsis nesting level mismatch: expected {level}, got {nesting_level}"
                    )));
                }
                
                self.expand_multi_dimensional_template(
                    sub_template,
                    bindings,
                    nesting_level,
                    context,
                )
            }
            
            Template::List(templates) => {
                self.expand_nested_list_template(
                    templates,
                    bindings,
                    nesting_level,
                    context,
                )
            }
            
            Template::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    self.expand_binding_value(binding, nesting_level)
                } else {
                    Ok(Expr::Variable(var.clone()))
                }
            }
            
            Template::Literal(lit) => {
                Ok(Expr::Variable(lit.clone()))
            }
            
            _ => {
                // Handle other template types
                Ok(Expr::Variable("unsupported-template".to_string()))
            }
        }
    }
    
    /// Expand multi-dimensional template structure
    fn expand_multi_dimensional_template(
        &self,
        sub_template: &Template,
        bindings: &HashMap<String, BindingValue>,
        nesting_level: usize,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        // Find variables in the template that need ellipsis expansion
        let ellipsis_vars = self.find_ellipsis_variables(sub_template, bindings);
        
        if ellipsis_vars.is_empty() {
            // No ellipsis variables, just expand normally
            return self.expand_template_simple(sub_template, bindings, context);
        }
        
        // Get the dimensions of ellipsis expansion
        let expansion_dims = self.calculate_expansion_dimensions(&ellipsis_vars, bindings)?;
        
        // Expand template according to dimensions
        self.expand_template_with_dimensions(
            sub_template,
            bindings,
            &expansion_dims,
            nesting_level,
            context,
        )
    }
    
    /// Expand nested list template
    fn expand_nested_list_template(
        &self,
        templates: &[Template],
        bindings: &HashMap<String, BindingValue>,
        nesting_level: usize,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        let mut expanded_elements = Vec::new();
        
        for template in templates {
            if let Template::NestedEllipsis(sub_template, level) = template {
                // Expand ellipsis template
                let expanded = self.expand_multi_dimensional_template(
                    sub_template,
                    bindings,
                    *level,
                    context,
                )?;
                
                // Add expanded elements to list
                match expanded {
                    Expr::List(exprs) => expanded_elements.extend(exprs),
                    expr => expanded_elements.push(expr),
                }
            } else {
                // Regular template expansion
                let mut processor = NestedEllipsisProcessor::new();
                let expanded = processor.expand_nested_ellipsis(
                    template,
                    bindings,
                    nesting_level,
                    context,
                )?;
                expanded_elements.push(expanded);
            }
        }
        
        Ok(Expr::List(expanded_elements))
    }
    
    /// Find variables that require ellipsis expansion
    fn find_ellipsis_variables(
        &self,
        template: &Template,
        bindings: &HashMap<String, BindingValue>,
    ) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_template_variables(template, &mut vars);
        
        // Filter to only those that have list bindings (ellipsis expansion candidates)
        vars.into_iter()
            .filter(|var| {
                matches!(bindings.get(var), Some(BindingValue::List(_)))
            })
            .collect()
    }
    
    /// Collect all variables from template recursively
    fn collect_template_variables(&self, template: &Template, vars: &mut Vec<String>) {
        match template {
            Template::Variable(var) => {
                vars.push(var.clone());
            }
            Template::List(templates) => {
                for t in templates {
                    self.collect_template_variables(t, vars);
                }
            }
            Template::NestedEllipsis(sub_template, _) => {
                self.collect_template_variables(sub_template, vars);
            }
            _ => {}
        }
    }
    
    /// Calculate expansion dimensions for ellipsis variables
    fn calculate_expansion_dimensions(
        &self,
        ellipsis_vars: &[String],
        bindings: &HashMap<String, BindingValue>,
    ) -> Result<Vec<usize>> {
        let mut dimensions = Vec::new();
        
        for var in ellipsis_vars {
            if let Some(BindingValue::List(exprs)) = bindings.get(var) {
                dimensions.push(exprs.len());
            }
        }
        
        // All ellipsis variables should have the same dimensions
        if dimensions.iter().all(|&d| d == dimensions[0]) {
            Ok(dimensions)
        } else {
            Err(LambdustError::syntax_error(
                "Ellipsis variables have mismatched dimensions".to_string()
            ))
        }
    }
    
    /// Expand template with specific dimensions
    fn expand_template_with_dimensions(
        &self,
        template: &Template,
        bindings: &HashMap<String, BindingValue>,
        dimensions: &[usize],
        nesting_level: usize,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        if dimensions.is_empty() {
            return self.expand_template_simple(template, bindings, context);
        }
        
        let primary_dim = dimensions[0];
        let mut expanded_list = Vec::new();
        
        for i in 0..primary_dim {
            // Create iteration-specific bindings
            let iter_bindings = self.create_iteration_bindings(bindings, i);
            
            // Recursively expand with remaining dimensions
            let remaining_dims = &dimensions[1..];
            let expanded = if remaining_dims.is_empty() {
                self.expand_template_simple(template, &iter_bindings, context)?
            } else {
                self.expand_template_with_dimensions(
                    template,
                    &iter_bindings,
                    remaining_dims,
                    nesting_level.saturating_sub(1),
                    context,
                )?
            };
            
            expanded_list.push(expanded);
        }
        
        Ok(Expr::List(expanded_list))
    }
    
    /// Create bindings for specific iteration
    fn create_iteration_bindings(
        &self,
        bindings: &HashMap<String, BindingValue>,
        iteration: usize,
    ) -> HashMap<String, BindingValue> {
        let mut iter_bindings = HashMap::new();
        
        for (var, binding) in bindings {
            match binding {
                BindingValue::List(exprs) => {
                    if iteration < exprs.len() {
                        iter_bindings.insert(
                            var.clone(),
                            BindingValue::Single(exprs[iteration].clone()),
                        );
                    }
                }
                _ => {
                    iter_bindings.insert(var.clone(), binding.clone());
                }
            }
        }
        
        iter_bindings
    }
    
    /// Simple template expansion (no ellipsis)
    fn expand_template_simple(
        &self,
        template: &Template,
        bindings: &HashMap<String, BindingValue>,
        _context: &ExpansionContext,
    ) -> Result<Expr> {
        match template {
            Template::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    self.expand_binding_value(binding, 0)
                } else {
                    Ok(Expr::Variable(var.clone()))
                }
            }
            
            Template::Literal(lit) => {
                Ok(Expr::Variable(lit.clone()))
            }
            
            Template::List(templates) => {
                let mut expanded = Vec::new();
                for t in templates {
                    let e = self.expand_template_simple(t, bindings, _context)?;
                    expanded.push(e);
                }
                Ok(Expr::List(expanded))
            }
            
            _ => {
                Ok(Expr::Variable("template-expansion-placeholder".to_string()))
            }
        }
    }
    
    /// Expand binding value based on nesting level
    fn expand_binding_value(&self, binding: &BindingValue, nesting_level: usize) -> Result<Expr> {
        match binding {
            BindingValue::Single(expr) => Ok(expr.clone()),
            
            BindingValue::List(exprs) => {
                if nesting_level > 0 {
                    Ok(Expr::List(exprs.clone()))
                } else {
                    // At top level, list bindings should be expanded as separate elements
                    // This is handled by the ellipsis expansion logic
                    Ok(Expr::List(exprs.clone()))
                }
            }
            
            BindingValue::SyntaxObject(obj) => {
                Ok(obj.expression.clone())
            }
        }
    }
    
    /// Get performance metrics
    #[must_use] pub fn metrics(&self) -> &EllipsisMetrics {
        &self.metrics
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = EllipsisMetrics::default();
    }
}

impl Default for NestedEllipsisProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EllipsisContext {
    /// Create new ellipsis context
    #[must_use] pub fn new() -> Self {
        Self {
            current_depth: 0,
            max_depth: 0,
            multi_bindings: HashMap::new(),
            iteration_counts: Vec::new(),
        }
    }
    
    /// Enter deeper ellipsis level
    pub fn enter_level(&mut self) {
        self.current_depth += 1;
        self.max_depth = self.max_depth.max(self.current_depth);
        self.iteration_counts.push(0);
    }
    
    /// Exit ellipsis level
    pub fn exit_level(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
            self.iteration_counts.pop();
        }
    }
    
    /// Increment iteration count at current level
    pub fn increment_iteration(&mut self) {
        if let Some(count) = self.iteration_counts.last_mut() {
            *count += 1;
        }
    }
    
    /// Match nested pattern (placeholder implementation)
    pub fn match_nested_pattern(
        &self,
        _pattern: &crate::macros::Pattern,
        _expr: &crate::ast::Expr,
        _level: u32,
    ) -> crate::error::Result<std::collections::HashMap<String, crate::macros::BindingValue>> {
        // TODO: Implement proper nested pattern matching
        Ok(std::collections::HashMap::new())
    }
    
    /// Alternative match nested pattern with all parameters (placeholder implementation)
    pub fn match_nested_pattern_full(
        &self,
        _pattern: &crate::macros::Pattern,
        _expr: &crate::ast::Expr,
        _level: u32,
        _bindings: &mut std::collections::HashMap<String, crate::ast::Expr>,
        _usage_env: &crate::environment::Environment,
    ) -> crate::error::Result<()> {
        // TODO: Implement proper nested pattern matching
        Ok(())
    }
    
    /// Expand nested template (placeholder implementation)
    pub fn expand_nested_template(
        &self,
        _template: &crate::macros::Template,
        _bindings: &std::collections::HashMap<String, crate::macros::BindingValue>,
        _level: u32,
        _context: &crate::macros::hygiene::ExpansionContext,
        _usage_env: &crate::environment::Environment,
        _literals: &std::collections::HashSet<String>,
    ) -> crate::error::Result<crate::ast::Expr> {
        // TODO: Implement proper nested template expansion
        Ok(crate::ast::Expr::Literal(crate::ast::Literal::Nil))
    }
}

impl Default for EllipsisContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchResult {
    /// Create successful match result
    #[must_use] pub fn success() -> Self {
        Self {
            bindings: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            success: true,
        }
    }
    
    /// Create failed match result
    #[must_use] pub fn failed() -> Self {
        Self {
            bindings: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            success: false,
        }
    }
}

impl EllipsisMetrics {
    /// Get success rate as percentage
    #[must_use] pub fn success_rate(&self) -> f64 {
        if self.pattern_matches_attempted > 0 {
            (self.pattern_matches_successful as f64 / self.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get average processing time per operation (microseconds)
    #[must_use] pub fn average_processing_time_us(&self) -> f64 {
        if self.pattern_matches_attempted > 0 {
            (self.total_processing_time_ns as f64 / self.pattern_matches_attempted as f64) / 1000.0
        } else {
            0.0
        }
    }
    
    /// Format metrics for display
    #[must_use] pub fn format_summary(&self) -> String {
        format!(
            "SRFI 46 Nested Ellipsis Metrics:\n\
             Pattern Matches: {}/{} ({:.1}% success)\n\
             Template Expansions: {}\n\
             Max Nesting Depth: {}\n\
             Average Time: {:.2}μs per operation\n\
             Total Processing: {:.2}ms",
            self.pattern_matches_successful,
            self.pattern_matches_attempted,
            self.success_rate(),
            self.template_expansions,
            self.max_nesting_depth_seen,
            self.average_processing_time_us(),
            self.total_processing_time_ns as f64 / 1_000_000.0
        )
    }
}
