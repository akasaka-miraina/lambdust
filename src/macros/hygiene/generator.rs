//! Symbol generator for unique hygienic symbols
//!
//! Provides generation of unique symbol identifiers and names for hygienic macro expansion.
//! Ensures that each macro-introduced symbol has a unique identity to prevent collisions.

use super::symbol::{HygienicSymbol, SymbolId, MacroSite, EnvironmentId};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for symbol generation
static SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global counter for environment generation
static ENVIRONMENT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generator for unique hygienic symbols
#[derive(Debug, Clone)]
pub struct SymbolGenerator {
    /// Prefix for generated symbol names
    prefix: String,
    /// Current macro expansion context
    current_macro: Option<String>,
    /// Current expansion depth
    current_depth: usize,
    /// Current environment ID
    current_environment: EnvironmentId,
}

impl SymbolGenerator {
    /// Create new symbol generator
    pub fn new() -> Self {
        Self {
            prefix: "λ$".to_string(),
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
        }
    }
    
    /// Create symbol generator with custom prefix
    pub fn with_prefix(prefix: String) -> Self {
        Self {
            prefix,
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
        }
    }
    
    /// Set current macro expansion context
    pub fn set_macro_context(&mut self, macro_name: String, depth: usize) {
        self.current_macro = Some(macro_name);
        self.current_depth = depth;
    }
    
    /// Set current environment
    pub fn set_environment(&mut self, env_id: EnvironmentId) {
        self.current_environment = env_id;
    }
    
    /// Generate unique symbol ID
    pub fn generate_symbol_id() -> SymbolId {
        let id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        SymbolId::new(id)
    }

    /// Get next ID for context usage
    pub fn next_id(&self) -> u64 {
        SYMBOL_COUNTER.load(Ordering::SeqCst)
    }
    
    /// Generate unique environment ID
    pub fn generate_environment_id() -> EnvironmentId {
        let id = ENVIRONMENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        EnvironmentId::new(id)
    }
    
    /// Generate unique hygienic symbol
    pub fn generate_unique(&mut self, base_name: &str) -> HygienicSymbol {
        let id = Self::generate_symbol_id();
        let definition_site = self.current_site();
        
        HygienicSymbol::new(
            base_name.to_string(),
            id,
            definition_site,
        )
    }
    
    /// Generate unique symbol with explicit macro context
    pub fn generate_with_context(
        &self,
        base_name: &str,
        macro_name: &str,
        depth: usize,
        env_id: EnvironmentId,
    ) -> HygienicSymbol {
        let id = Self::generate_symbol_id();
        let definition_site = MacroSite::new(
            macro_name.to_string(),
            depth,
            env_id,
        );
        
        HygienicSymbol::new(
            base_name.to_string(),
            id,
            definition_site,
        )
    }
    
    /// Generate symbol from user code (non-macro)
    pub fn generate_user_symbol(&self, name: &str) -> HygienicSymbol {
        let id = Self::generate_symbol_id();
        HygienicSymbol::from_user_code(
            name.to_string(),
            id,
            self.current_environment,
        )
    }
    
    /// Get current macro site information
    fn current_site(&self) -> MacroSite {
        let macro_name = self.current_macro
            .as_deref()
            .unwrap_or("<unknown>");
        
        MacroSite::new(
            macro_name.to_string(),
            self.current_depth,
            self.current_environment,
        )
    }
    
    /// Create scoped generator for nested macro expansion
    pub fn enter_macro(&self, macro_name: String) -> Self {
        let mut nested = self.clone();
        nested.current_macro = Some(macro_name);
        nested.current_depth += 1;
        nested
    }
    
    /// Reset to parent scope
    pub fn exit_macro(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
        if self.current_depth == 0 {
            self.current_macro = None;
        }
    }
}

impl Default for SymbolGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for symbol generation
pub struct SymbolUtils;

impl SymbolUtils {
    /// Check if a name looks like a generated symbol
    pub fn is_generated_name(name: &str) -> bool {
        name.starts_with("λ$") || name.contains('#')
    }
    
    /// Extract base name from generated symbol name
    pub fn extract_base_name(name: &str) -> Option<&str> {
        if let Some(stripped) = name.strip_prefix("λ$") {
            if let Some(hash_pos) = stripped.find('#') {
                Some(&stripped[..hash_pos])
            } else {
                Some(stripped)
            }
        } else {
            None
        }
    }
    
    /// Extract symbol ID from generated name
    pub fn extract_symbol_id(name: &str) -> Option<SymbolId> {
        if let Some(hash_pos) = name.find('#') {
            let id_str = &name[hash_pos + 1..];
            if let Ok(id) = id_str.parse::<u64>() {
                Some(SymbolId::new(id))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_generator_creation() {
        let gen = SymbolGenerator::new();
        assert_eq!(gen.prefix, "λ$");
        assert!(gen.current_macro.is_none());
        assert_eq!(gen.current_depth, 0);
    }
    
    #[test]
    fn test_custom_prefix() {
        let gen = SymbolGenerator::with_prefix("test$".to_string());
        assert_eq!(gen.prefix, "test$");
    }
    
    #[test]
    fn test_unique_symbol_generation() {
        let mut gen = SymbolGenerator::new();
        gen.set_macro_context("test-macro".to_string(), 1);
        
        let symbol1 = gen.generate_unique("temp");
        let symbol2 = gen.generate_unique("temp");
        
        // Should have different IDs
        assert_ne!(symbol1.id, symbol2.id);
        
        // Should have same base name
        assert_eq!(symbol1.original_name(), "temp");
        assert_eq!(symbol2.original_name(), "temp");
        
        // Should be macro-introduced
        assert!(symbol1.is_macro_introduced);
        assert!(symbol2.is_macro_introduced);
    }
    
    #[test]
    fn test_user_symbol_generation() {
        let gen = SymbolGenerator::new();
        let symbol = gen.generate_user_symbol("user-var");
        
        assert_eq!(symbol.original_name(), "user-var");
        assert_eq!(symbol.unique_name(), "user-var"); // No decoration
        assert!(!symbol.is_macro_introduced);
    }
    
    #[test]
    fn test_macro_context_nesting() {
        let gen = SymbolGenerator::new();
        let nested = gen.enter_macro("outer-macro".to_string());
        
        assert_eq!(nested.current_depth, 1);
        assert_eq!(nested.current_macro, Some("outer-macro".to_string()));
        
        let double_nested = nested.enter_macro("inner-macro".to_string());
        assert_eq!(double_nested.current_depth, 2);
        assert_eq!(double_nested.current_macro, Some("inner-macro".to_string()));
    }
    
    #[test]
    fn test_global_id_uniqueness() {
        // Generate multiple IDs to ensure uniqueness
        let id1 = SymbolGenerator::generate_symbol_id();
        let id2 = SymbolGenerator::generate_symbol_id();
        let id3 = SymbolGenerator::generate_symbol_id();
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_environment_id_generation() {
        let env1 = SymbolGenerator::generate_environment_id();
        let env2 = SymbolGenerator::generate_environment_id();
        
        assert_ne!(env1, env2);
    }
    
    #[test]
    fn test_symbol_utils_generated_name_detection() {
        assert!(SymbolUtils::is_generated_name("λ$temp#123"));
        assert!(SymbolUtils::is_generated_name("something#456"));
        assert!(!SymbolUtils::is_generated_name("normal-name"));
        assert!(!SymbolUtils::is_generated_name("lambda-var"));
    }
    
    #[test]
    fn test_symbol_utils_base_name_extraction() {
        assert_eq!(SymbolUtils::extract_base_name("λ$temp#123"), Some("temp"));
        assert_eq!(SymbolUtils::extract_base_name("λ$variable#456"), Some("variable"));
        assert_eq!(SymbolUtils::extract_base_name("normal-name"), None);
    }
    
    #[test]
    fn test_symbol_utils_id_extraction() {
        let id = SymbolUtils::extract_symbol_id("λ$temp#123");
        assert_eq!(id, Some(SymbolId::new(123)));
        
        let id2 = SymbolUtils::extract_symbol_id("something#456");
        assert_eq!(id2, Some(SymbolId::new(456)));
        
        let id3 = SymbolUtils::extract_symbol_id("no-id-here");
        assert_eq!(id3, None);
    }
}