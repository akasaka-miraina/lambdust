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
    #[must_use] pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get raw ID value
    #[must_use] pub fn id(&self) -> u64 {
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
    #[must_use] pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get raw ID value
    #[must_use] pub fn id(&self) -> u64 {
        self.0
    }
}

/// Information about macro expansion site
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    #[must_use] pub fn new(
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
    #[must_use] pub fn with_location(
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    #[must_use] pub fn new(
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
    #[must_use] pub fn from_user_code(
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
    #[must_use] pub fn with_usage_site(mut self, usage_site: MacroSite) -> Self {
        self.usage_site = Some(usage_site);
        self
    }
    
    /// Get the original name without hygiene decorations
    #[must_use] pub fn original_name(&self) -> &str {
        &self.name
    }
    
    /// Get unique name with hygiene information
    #[must_use] pub fn unique_name(&self) -> String {
        if self.is_macro_introduced {
            format!("λ${}#{}", self.name, self.id.0)
        } else {
            self.name.clone()
        }
    }
    
    /// Check if this symbol is the same as another (ignoring usage sites)
    #[must_use] pub fn is_same_symbol(&self, other: &HygienicSymbol) -> bool {
        self.id == other.id && self.name == other.name
    }
    
    /// Check if this symbol can reference the same binding as another
    #[must_use] pub fn can_reference_same_binding(&self, other: &HygienicSymbol) -> bool {
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