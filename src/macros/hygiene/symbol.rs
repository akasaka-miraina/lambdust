//! Hygienic symbol system
//!
//! Implements unique symbols with hygiene information to prevent symbol collision
//! in macro expansion. Each symbol carries information about its definition site
//! and usage context.

use std::fmt;
use std::hash::{Hash, Hasher};

/// Unique identifier for symbols in hygienic macro system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u64);

impl SymbolId {
    /// Create new symbol ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get raw ID value
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Environment identifier for lexical scope tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvironmentId(pub u64);

impl EnvironmentId {
    /// Create new environment ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get raw ID value
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Information about macro expansion site
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroSite {
    /// Name of the macro that introduced this symbol
    pub macro_name: String,
    /// Depth of macro expansion (0 = top level)
    pub depth: usize,
    /// Lexical environment ID at definition site
    pub environment_id: EnvironmentId,
    /// Source location information (optional)
    pub source_location: Option<SourceLocation>,
}

impl MacroSite {
    /// Create new macro site
    pub fn new(
        macro_name: String,
        depth: usize,
        environment_id: EnvironmentId,
    ) -> Self {
        Self {
            macro_name,
            depth,
            environment_id,
            source_location: None,
        }
    }
    
    /// Create macro site with source location
    pub fn with_location(
        macro_name: String,
        depth: usize,
        environment_id: EnvironmentId,
        location: SourceLocation,
    ) -> Self {
        Self {
            macro_name,
            depth,
            environment_id,
            source_location: Some(location),
        }
    }
}

/// Source code location information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// File name or identifier
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Hygienic symbol with unique identity and macro expansion context
#[derive(Debug, Clone)]
pub struct HygienicSymbol {
    /// Original symbol name
    pub name: String,
    /// Unique identifier for this symbol instance
    pub id: SymbolId,
    /// Site where this symbol was defined/introduced
    pub definition_site: MacroSite,
    /// Site where this symbol is being used (if different from definition)
    pub usage_site: Option<MacroSite>,
    /// Whether this symbol was introduced by macro expansion
    pub is_macro_introduced: bool,
}

impl HygienicSymbol {
    /// Create new hygienic symbol
    pub fn new(
        name: String,
        id: SymbolId,
        definition_site: MacroSite,
    ) -> Self {
        Self {
            name,
            id,
            definition_site,
            usage_site: None,
            is_macro_introduced: true,
        }
    }
    
    /// Create hygienic symbol from user code (not macro-introduced)
    pub fn from_user_code(
        name: String,
        id: SymbolId,
        environment_id: EnvironmentId,
    ) -> Self {
        let definition_site = MacroSite::new(
            "<user-code>".to_string(),
            0,
            environment_id,
        );
        
        Self {
            name,
            id,
            definition_site,
            usage_site: None,
            is_macro_introduced: false,
        }
    }
    
    /// Set usage site information
    pub fn with_usage_site(mut self, usage_site: MacroSite) -> Self {
        self.usage_site = Some(usage_site);
        self
    }
    
    /// Get the original name without hygiene decorations
    pub fn original_name(&self) -> &str {
        &self.name
    }
    
    /// Get unique name with hygiene information
    pub fn unique_name(&self) -> String {
        if self.is_macro_introduced {
            format!("λ${}#{}", self.name, self.id.0)
        } else {
            self.name.clone()
        }
    }
    
    /// Check if this symbol is the same as another (ignoring usage sites)
    pub fn is_same_symbol(&self, other: &HygienicSymbol) -> bool {
        self.id == other.id && self.name == other.name
    }
    
    /// Check if this symbol can reference the same binding as another
    pub fn can_reference_same_binding(&self, other: &HygienicSymbol) -> bool {
        // Same symbol ID means same binding
        if self.id == other.id {
            return true;
        }
        
        // If both are from user code with same name and environment, they refer to same binding
        if !self.is_macro_introduced && !other.is_macro_introduced {
            return self.name == other.name 
                && self.definition_site.environment_id == other.definition_site.environment_id;
        }
        
        false
    }
}

impl PartialEq for HygienicSymbol {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Eq for HygienicSymbol {}

impl Hash for HygienicSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl fmt::Display for HygienicSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_macro_introduced {
            write!(f, "{}#{}", self.name, self.id.0)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_id_creation() {
        let id = SymbolId::new(42);
        assert_eq!(id.id(), 42);
        assert_eq!(format!("{}", id), "#42");
    }
    
    #[test]
    fn test_environment_id_creation() {
        let env_id = EnvironmentId::new(123);
        assert_eq!(env_id.id(), 123);
    }
    
    #[test]
    fn test_macro_site_creation() {
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("test-macro".to_string(), 2, env_id);
        
        assert_eq!(site.macro_name, "test-macro");
        assert_eq!(site.depth, 2);
        assert_eq!(site.environment_id, env_id);
        assert!(site.source_location.is_none());
    }
    
    #[test]
    fn test_hygienic_symbol_creation() {
        let id = SymbolId::new(1);
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("test-macro".to_string(), 0, env_id);
        
        let symbol = HygienicSymbol::new("x".to_string(), id, site);
        
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.id, id);
        assert_eq!(symbol.original_name(), "x");
        assert_eq!(symbol.unique_name(), "λ$x#1");
        assert!(symbol.is_macro_introduced);
    }
    
    #[test]
    fn test_user_code_symbol() {
        let id = SymbolId::new(2);
        let env_id = EnvironmentId::new(1);
        
        let symbol = HygienicSymbol::from_user_code("y".to_string(), id, env_id);
        
        assert_eq!(symbol.name, "y");
        assert_eq!(symbol.unique_name(), "y"); // No decoration for user code
        assert!(!symbol.is_macro_introduced);
    }
    
    #[test]
    fn test_symbol_equality() {
        let id = SymbolId::new(1);
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("test".to_string(), 0, env_id);
        
        let symbol1 = HygienicSymbol::new("x".to_string(), id, site.clone());
        let symbol2 = HygienicSymbol::new("x".to_string(), id, site);
        
        assert_eq!(symbol1, symbol2);
        assert!(symbol1.is_same_symbol(&symbol2));
    }
    
    #[test]
    fn test_binding_reference_compatibility() {
        let env_id = EnvironmentId::new(1);
        
        // Same user code symbols can reference same binding
        let user1 = HygienicSymbol::from_user_code("x".to_string(), SymbolId::new(1), env_id);
        let user2 = HygienicSymbol::from_user_code("x".to_string(), SymbolId::new(2), env_id);
        assert!(user1.can_reference_same_binding(&user2));
        
        // Macro-introduced symbols with same ID reference same binding
        let site = MacroSite::new("test".to_string(), 0, env_id);
        let macro1 = HygienicSymbol::new("temp".to_string(), SymbolId::new(3), site.clone());
        let macro2 = HygienicSymbol::new("temp".to_string(), SymbolId::new(3), site);
        assert!(macro1.can_reference_same_binding(&macro2));
        
        // Different macro-introduced symbols don't reference same binding
        let site1 = MacroSite::new("test1".to_string(), 0, env_id);
        let site2 = MacroSite::new("test2".to_string(), 0, env_id);
        let macro3 = HygienicSymbol::new("temp".to_string(), SymbolId::new(4), site1);
        let macro4 = HygienicSymbol::new("temp".to_string(), SymbolId::new(5), site2);
        assert!(!macro3.can_reference_same_binding(&macro4));
    }
}