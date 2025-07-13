#[cfg(test)]
mod tests {
    use crate::macros::hygiene::symbol::*;

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
        let site = MacroSite::new("user-code".to_string(), 0, env_id);
        
        let mut symbol = HygienicSymbol::new("y".to_string(), id, site);
        symbol.is_macro_introduced = false; // User code symbol
        
        assert_eq!(symbol.name, "y");
        assert_eq!(symbol.original_name(), "y");
        assert_eq!(symbol.unique_name(), "y#2");
        assert!(!symbol.is_macro_introduced);
    }
    
    #[test]
    fn test_symbol_equality() {
        let id1 = SymbolId::new(1);
        let id2 = SymbolId::new(1);
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("test".to_string(), 0, env_id);
        
        let symbol1 = HygienicSymbol::new("x".to_string(), id1, site.clone());
        let symbol2 = HygienicSymbol::new("x".to_string(), id2, site);
        
        assert_eq!(symbol1, symbol2);
    }
    
    #[test]
    fn test_symbol_inequality() {
        let id1 = SymbolId::new(1);
        let id2 = SymbolId::new(2);
        let env_id = EnvironmentId::new(1);
        let site1 = MacroSite::new("test1".to_string(), 0, env_id);
        let site2 = MacroSite::new("test2".to_string(), 0, env_id);
        
        let symbol1 = HygienicSymbol::new("x".to_string(), id1, site1);
        let symbol2 = HygienicSymbol::new("x".to_string(), id2, site2);
        
        assert_ne!(symbol1, symbol2);
    }
    
    #[test]
    fn test_symbol_debug_format() {
        let id = SymbolId::new(42);
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("debug-test".to_string(), 1, env_id);
        let symbol = HygienicSymbol::new("test_var".to_string(), id, site);
        
        let debug_str = format!("{:?}", symbol);
        assert!(debug_str.contains("test_var"));
        assert!(debug_str.contains("#42"));
    }
    
    #[test]
    fn test_can_reference_same_binding() {
        let env_id = EnvironmentId::new(1);
        let site1 = MacroSite::new("macro1".to_string(), 0, env_id);
        let site2 = MacroSite::new("macro2".to_string(), 0, env_id);
        
        // Same symbol ID should reference same binding
        let symbol1 = HygienicSymbol::new("x".to_string(), SymbolId::new(1), site1.clone());
        let symbol2 = HygienicSymbol::new("x".to_string(), SymbolId::new(1), site2.clone());
        assert!(symbol1.can_reference_same_binding(&symbol2));
        
        // Different symbol IDs should not reference same binding
        let symbol3 = HygienicSymbol::new("x".to_string(), SymbolId::new(2), site1);
        let symbol4 = HygienicSymbol::new("x".to_string(), SymbolId::new(3), site2);
        assert!(!symbol3.can_reference_same_binding(&symbol4));
    }
    
    #[test]
    fn test_macro_site_with_location() {
        let env_id = EnvironmentId::new(1);
        let mut site = MacroSite::new("located-macro".to_string(), 1, env_id);
        site.source_location = Some("file.scm:10:5".to_string());
        
        let symbol = HygienicSymbol::new("located_var".to_string(), SymbolId::new(5), site);
        
        assert_eq!(symbol.site.source_location, Some("file.scm:10:5".to_string()));
    }
    
    #[test]
    fn test_complex_macro_nesting() {
        let env_id1 = EnvironmentId::new(1);
        let env_id2 = EnvironmentId::new(2);
        
        let outer_site = MacroSite::new("outer-macro".to_string(), 0, env_id1);
        let inner_site = MacroSite::new("inner-macro".to_string(), 1, env_id2);
        
        let outer_symbol = HygienicSymbol::new("nested".to_string(), SymbolId::new(10), outer_site);
        let inner_symbol = HygienicSymbol::new("nested".to_string(), SymbolId::new(11), inner_site);
        
        // Same name but different sites should not reference same binding
        assert!(!outer_symbol.can_reference_same_binding(&inner_symbol));
        
        // Different names should not reference same binding even with same sites
        let env_id = EnvironmentId::new(1);
        let site = MacroSite::new("same-macro".to_string(), 0, env_id);
        let var1 = HygienicSymbol::new("var1".to_string(), SymbolId::new(20), site.clone());
        let var2 = HygienicSymbol::new("var2".to_string(), SymbolId::new(21), site);
        assert!(!var1.can_reference_same_binding(&var2));
    }
    
    #[test]
    fn test_hygiene_with_name_collision() {
        let env_id = EnvironmentId::new(1);
        let site1 = MacroSite::new("macro1".to_string(), 0, env_id);
        let site2 = MacroSite::new("macro2".to_string(), 0, env_id);
        
        // Same original name but different macro context should have different hygiene
        let macro1 = HygienicSymbol::new("temp".to_string(), SymbolId::new(4), site1);
        let macro2 = HygienicSymbol::new("temp".to_string(), SymbolId::new(5), site2);
        assert!(!macro1.can_reference_same_binding(&macro2));
    }
}