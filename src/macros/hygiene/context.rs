//! Macro expansion context tracking
//!
//! Tracks the context of macro expansion including depth, macro stack, and symbol generation.
//! Essential for maintaining proper hygiene across nested macro expansions.

use super::generator::SymbolGenerator;
use super::symbol::{HygienicSymbol, EnvironmentId, MacroSite};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Statistics about macro expansion
#[derive(Debug, Clone)]
pub struct ExpansionStats {
    /// Current expansion depth
    pub depth: usize,
    /// Number of macros in expansion chain
    pub macro_count: usize,
    /// Time elapsed since expansion started
    pub elapsed_time: Duration,
    /// Number of symbols bound in current context
    pub symbol_count: usize,
    /// Whether any macro appears multiple times in stack
    pub is_recursive: bool,
}

/// Context information for macro expansion
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    /// Current expansion depth (0 = top level)
    pub depth: usize,
    /// Stack of macro names being expanded (for nested expansions)
    pub macro_stack: Vec<String>,
    /// Symbol generator for unique IDs
    pub symbol_generator: SymbolGenerator,
    /// Mapping from original names to hygienic symbols in current context
    pub symbol_bindings: HashMap<String, HygienicSymbol>,
    /// Current environment ID
    pub environment_id: EnvironmentId,
    /// Whether we're in a macro definition vs expansion
    pub is_definition_context: bool,
    /// Maximum allowed expansion depth (prevents infinite recursion)
    pub max_expansion_depth: usize,
    /// Expansion timeout (prevents infinite loops)
    pub expansion_timeout: Duration,
    /// Start time of current expansion (for timeout tracking)
    pub expansion_start: Option<Instant>,
    /// Set of macro names seen in current expansion chain (for cycle detection)
    pub macro_cycle_set: HashSet<String>,
}

impl ExpansionContext {
    /// Create new expansion context from definition and usage environments
    pub fn new(
        definition_environment: super::environment::HygienicEnvironment,
        _usage_environment: super::environment::HygienicEnvironment,
    ) -> Self {
        // Combine environment IDs or create a new one
        let environment_id = definition_environment.environment_id();
        
        Self {
            depth: 0,
            macro_stack: Vec::new(),
            symbol_generator: SymbolGenerator::new(),
            symbol_bindings: HashMap::new(),
            environment_id,
            is_definition_context: false,
            max_expansion_depth: 256,
            expansion_timeout: Duration::from_millis(5000),
            expansion_start: None,
            macro_cycle_set: HashSet::new(),
        }
    }
    
    /// Create new expansion context with environment ID
    #[must_use] pub fn with_environment_id(environment_id: EnvironmentId) -> Self {
        Self {
            depth: 0,
            macro_stack: Vec::new(),
            symbol_generator: SymbolGenerator::new(),
            symbol_bindings: HashMap::new(),
            environment_id,
            is_definition_context: false,
            max_expansion_depth: 256, // Reasonable default
            expansion_timeout: Duration::from_millis(5000), // 5 second timeout
            expansion_start: None,
            macro_cycle_set: HashSet::new(),
        }
    }

    /// Generate a fresh identifier for syntax-case usage
    pub fn generate_fresh_identifier(&self, base_name: &str) -> Result<String, crate::error::LambdustError> {
        // Simple implementation for now - would be more sophisticated in production
        Ok(format!("{}_{}", base_name, self.symbol_generator.next_id()))
    }
    
    /// Create context for macro definition
    #[must_use] pub fn for_definition(environment_id: EnvironmentId) -> Self {
        let mut context = Self::with_environment_id(environment_id);
        context.is_definition_context = true;
        context
    }
    
    /// Enter a new macro expansion with safety checks
    pub fn enter_macro(&self, macro_name: String) -> Result<Self, String> {
        // Check expansion depth limit
        if self.depth >= self.max_expansion_depth {
            return Err(format!(
                "Maximum macro expansion depth exceeded ({}). Possible infinite recursion in macro: {}",
                self.max_expansion_depth, macro_name
            ));
        }
        
        // Check for direct cycles - only detect when a macro calls itself directly
        // Allow different macros to have the same name at different levels
        if !self.macro_stack.is_empty() {
            let consecutive_count = self.macro_stack.iter().rev().take_while(|name| *name == &macro_name).count();
            if consecutive_count > 0 {
                return Err(format!(
                    "Circular macro expansion detected: {} -> {}",
                    self.expansion_path(), macro_name
                ));
            }
        }
        
        // Check expansion timeout
        if let Some(start_time) = self.expansion_start {
            if start_time.elapsed() > self.expansion_timeout {
                return Err(format!(
                    "Macro expansion timeout exceeded ({:?}). Expansion path: {}",
                    self.expansion_timeout, self.expansion_path()
                ));
            }
        }
        
        let mut nested = self.clone();
        nested.depth += 1;
        nested.macro_stack.push(macro_name.clone());
        // Only add to cycle set if it's a direct recursion
        if nested.macro_stack.len() >= 2 {
            let len = nested.macro_stack.len();
            if nested.macro_stack[len-1] == nested.macro_stack[len-2] {
                nested.macro_cycle_set.insert(macro_name.clone());
            }
        }
        nested.symbol_generator.set_macro_context(macro_name, nested.depth);
        nested.is_definition_context = false;
        
        // Start timing if this is the first macro expansion
        if nested.expansion_start.is_none() {
            nested.expansion_start = Some(Instant::now());
        }
        
        Ok(nested)
    }
    
    /// Exit current macro expansion
    pub fn exit_macro(&mut self) -> Result<(), String> {
        if self.depth == 0 {
            return Err("Cannot exit macro: not in macro expansion".to_string());
        }
        
        self.depth -= 1;
        if let Some(macro_name) = self.macro_stack.pop() {
            self.macro_cycle_set.remove(&macro_name);
        }
        self.symbol_generator.exit_macro();
        
        // Clear expansion start time if we're back to top level
        if self.depth == 0 {
            self.expansion_start = None;
        }
        
        Ok(())
    }
    
    /// Get current macro name (if any)
    #[must_use] pub fn current_macro(&self) -> Option<&str> {
        self.macro_stack.last().map(std::string::String::as_str)
    }
    
    /// Check if we're currently expanding a specific macro
    #[must_use] pub fn is_expanding_macro(&self, macro_name: &str) -> bool {
        self.macro_stack.iter().any(|name| name == macro_name)
    }
    
    /// Generate unique symbol for pattern variable
    pub fn generate_pattern_variable(&mut self, base_name: &str) -> HygienicSymbol {
        let symbol = self.symbol_generator.generate_unique(base_name);
        self.symbol_bindings.insert(base_name.to_string(), symbol.clone());
        symbol
    }
    
    /// Generate symbol for template expansion
    pub fn generate_template_symbol(&mut self, base_name: &str) -> HygienicSymbol {
        // Check if we have a binding for this name first
        if let Some(existing) = self.symbol_bindings.get(base_name) {
            existing.clone()
        } else {
            // Generate new symbol for unbound template variable
            self.symbol_generator.generate_unique(base_name)
        }
    }
    
    /// Bind symbol in current context
    pub fn bind_symbol(&mut self, name: String, symbol: HygienicSymbol) {
        self.symbol_bindings.insert(name, symbol);
    }
    
    /// Lookup symbol binding in current context
    #[must_use] pub fn lookup_symbol(&self, name: &str) -> Option<&HygienicSymbol> {
        self.symbol_bindings.get(name)
    }
    
    /// Create macro site for current context
    #[must_use] pub fn current_macro_site(&self) -> MacroSite {
        let macro_name = self.current_macro()
            .unwrap_or("<top-level>")
            .to_string();
        
        MacroSite::new(macro_name, self.depth, self.environment_id)
    }
    
    /// Check if we're in a recursive macro expansion
    #[must_use] pub fn is_recursive_expansion(&self, macro_name: &str) -> bool {
        self.macro_stack.iter().filter(|&name| name == macro_name).count() > 1
    }
    
    /// Get expansion path as string (for debugging)
    #[must_use] pub fn expansion_path(&self) -> String {
        if self.macro_stack.is_empty() {
            "<top-level>".to_string()
        } else {
            self.macro_stack.join(" -> ")
        }
    }
    
    /// Clear symbol bindings (for fresh expansion)
    pub fn clear_bindings(&mut self) {
        self.symbol_bindings.clear();
    }
    
    /// Create child context with fresh bindings
    #[must_use] pub fn child_context(&self) -> Self {
        let mut child = self.clone();
        child.symbol_bindings.clear();
        child
    }
    
    /// Configure expansion limits for safety
    #[must_use] pub fn with_limits(mut self, max_depth: usize, timeout: Duration) -> Self {
        self.max_expansion_depth = max_depth;
        self.expansion_timeout = timeout;
        self
    }
    
    /// Check if expansion is still within safety limits
    pub fn is_within_limits(&self) -> Result<(), String> {
        if self.depth >= self.max_expansion_depth {
            return Err(format!("Depth limit exceeded: {}", self.depth));
        }
        
        if let Some(start_time) = self.expansion_start {
            if start_time.elapsed() > self.expansion_timeout {
                return Err(format!("Timeout exceeded: {:?}", start_time.elapsed()));
            }
        }
        
        Ok(())
    }
    
    /// Get expansion statistics
    #[must_use] pub fn expansion_stats(&self) -> ExpansionStats {
        let elapsed = self.expansion_start
            .map_or(Duration::ZERO, |start| start.elapsed());
            
        ExpansionStats {
            depth: self.depth,
            macro_count: self.macro_stack.len(),
            elapsed_time: elapsed,
            symbol_count: self.symbol_bindings.len(),
            is_recursive: self.has_recursive_macro(),
        }
    }
    
    /// Check if there are any recursive macro calls in the stack
    #[must_use] pub fn has_recursive_macro(&self) -> bool {
        let mut seen = HashSet::new();
        for macro_name in &self.macro_stack {
            if !seen.insert(macro_name) {
                return true;
            }
        }
        false
    }
    
    /// Validate macro interaction safety
    pub fn validate_macro_interaction(&self, new_macro: &str) -> Result<(), String> {
        // Check for known problematic macro combinations
        if let Some(current) = self.current_macro() {
            if self.is_problematic_combination(current, new_macro) {
                return Err(format!(
                    "Problematic macro combination detected: {current} -> {new_macro}"
                ));
            }
        }
        
        // Check for excessive nesting of the same macro type
        let same_macro_count = self.macro_stack
            .iter()
            .filter(|&name| name == new_macro)
            .count();
            
        if same_macro_count >= 10 { // Configurable threshold
            return Err(format!(
                "Excessive nesting of macro '{new_macro}' detected ({same_macro_count}x)"
            ));
        }
        
        Ok(())
    }
    
    /// Check if two macros form a problematic combination
    fn is_problematic_combination(&self, current: &str, new_macro: &str) -> bool {
        // Define known problematic combinations
        // This could be extended with configuration
        matches!((current, new_macro), 
            ("define-syntax", "define-syntax") |  // Nested syntax definitions
            ("syntax-case", "syntax-rules") |    // Mixing case and rules
            ("let-syntax", "letrec-syntax")      // Potential scoping issues
        )
    }
}

/// Context stack for managing nested expansions
#[derive(Debug, Clone)]
pub struct ExpansionStack {
    /// Stack of expansion contexts
    contexts: Vec<ExpansionContext>,
    /// Current context index
    current: usize,
}

impl ExpansionStack {
    /// Create new expansion stack with initial context
    #[must_use] pub fn new(initial_context: ExpansionContext) -> Self {
        Self {
            contexts: vec![initial_context],
            current: 0,
        }
    }
    
    /// Push new context onto stack
    pub fn push(&mut self, context: ExpansionContext) {
        self.contexts.push(context);
        self.current = self.contexts.len() - 1;
    }
    
    /// Pop context from stack
    pub fn pop(&mut self) -> Result<ExpansionContext, String> {
        if self.contexts.len() <= 1 {
            return Err("Cannot pop last context from stack".to_string());
        }
        
        let popped = self.contexts.pop().unwrap();
        self.current = self.contexts.len() - 1;
        Ok(popped)
    }
    
    /// Get current context
    #[must_use] pub fn current(&self) -> &ExpansionContext {
        &self.contexts[self.current]
    }
    
    /// Get mutable current context
    pub fn current_mut(&mut self) -> &mut ExpansionContext {
        &mut self.contexts[self.current]
    }
    
    /// Get depth of context stack
    #[must_use] pub fn depth(&self) -> usize {
        self.contexts.len()
    }
    
    /// Check if stack is at top level
    #[must_use] pub fn is_top_level(&self) -> bool {
        self.contexts.len() == 1
    }
}

/// Helper trait for testing `ExpansionContext` mutations
/// 
/// This trait provides utility methods for testing scenarios where
/// `ExpansionContext` needs to be converted to a mutable form.
#[allow(dead_code)]
trait ExpansionContextExt {
    /// Converts an `ExpansionContext` into a mutable version of itself
    /// 
    /// This method is primarily used in testing scenarios where
    /// mutation of the context is required for test setup.
    fn into_mut(self) -> Self;
}

impl ExpansionContextExt for ExpansionContext {
    fn into_mut(self) -> Self {
        self
    }
}
