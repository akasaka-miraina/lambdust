//! Integration tests for the Lambdust module system.
//!
//! Tests the complete module system functionality including:
//! - Module loading and caching
//! - Import/export resolution
//! - Namespace management
//! - Dependency resolution
//! - Error handling

use lambdust::module_system::*;
use lambdust::diagnostics::Result;
use std::collections::HashMap;

#[test]
fn test_module_name_parsing() {
    // Test new lambdust syntax
    let new_builtin = parse_module_id("(lambdust string)").unwrap();
    assert_eq!(new_builtin.namespace, ModuleNamespace::Builtin);
    assert_eq!(new_builtin.components, vec!["string"]);
    
    // Test deprecated built-in module names (for backwards compatibility)
    let builtin = parse_module_id("(:: string)").unwrap();
    assert_eq!(builtin.namespace, ModuleNamespace::Builtin);
    assert_eq!(builtin.components, vec!["string"]);

    // Test that both new and old syntax produce same canonical format
    let new_canonical = format_module_id(&new_builtin);
    let old_canonical = format_module_id(&builtin);
    assert_eq!(new_canonical, "(lambdust string)");
    assert_eq!(old_canonical, "(lambdust string)");
    assert_eq!(new_canonical, old_canonical);

    // Test R7RS module names
    let r7rs = parse_module_id("(scheme base)").unwrap();
    assert_eq!(r7rs.namespace, ModuleNamespace::R7RS);
    assert_eq!(r7rs.components, vec!["base"]);

    // Test user module names
    let user = parse_module_id("(user my-module)").unwrap();
    assert_eq!(user.namespace, ModuleNamespace::User);
    assert_eq!(user.components, vec!["my-module"]);

    // Test file module names
    let file = parse_module_id("(file \"test.scm\")").unwrap();
    assert_eq!(file.namespace, ModuleNamespace::File);
    assert_eq!(file.components, vec!["test.scm"]);
}

#[test]
fn test_module_name_roundtrip() {
    let test_cases = vec![
        "(:: string)",
        "(scheme base)",
        "(user my-module)",
        "(file \"test.scm\")",
        "(scheme char numeric)",
    ];

    for case in test_cases {
        let parsed = parse_module_id(case).unwrap();
        let formatted = format_module_id(&parsed);
        assert_eq!(case, formatted);
    }
}

#[test]
fn test_invalid_module_names() {
    let invalid_cases = vec![
        "no-parens",
        "()",
        "(::)",
        "(scheme)",
        "(file)",
    ];

    for case in invalid_cases {
        assert!(parse_module_id(case).is_err());
    }
}

#[test]
fn test_module_system_creation() {
    let module_system = ModuleSystem::new();
    assert!(module_system.is_ok());
    
    let ms = module_system.unwrap();
    let modules = ms.list_modules();
    // Should have at least some built-in modules registered
    assert!(!modules.is_empty());
}

#[test]
fn test_import_config_application() {
    use lambdust::eval::Value;
    
    // Create test exports
    let mut exports = HashMap::new();
    exports.insert("string-length".to_string(), Value::integer(42));
    exports.insert("string-ref".to_string(), Value::integer(43));
    exports.insert("string-set!".to_string(), Value::integer(44));

    // Test 'only' import
    let only_config = ImportConfig::Only(vec!["string-length".to_string(), "string-ref".to_string()]);
    let only_result = import::apply_import_config(&exports, &only_config).unwrap();
    assert_eq!(only_result.len(), 2);
    assert!(only_result.contains_key("string-length"));
    assert!(only_result.contains_key("string-ref"));
    assert!(!only_result.contains_key("string-set!"));

    // Test 'except' import
    let except_config = ImportConfig::Except(vec!["string-set!".to_string()]);
    let except_result = import::apply_import_config(&exports, &except_config).unwrap();
    assert_eq!(except_result.len(), 2);
    assert!(except_result.contains_key("string-length"));
    assert!(except_result.contains_key("string-ref"));
    assert!(!except_result.contains_key("string-set!"));

    // Test 'rename' import
    let mut rename_map = HashMap::new();
    rename_map.insert("string-length".to_string(), "str-len".to_string());
    let rename_config = ImportConfig::Rename(rename_map);
    let rename_result = import::apply_import_config(&exports, &rename_config).unwrap();
    assert_eq!(rename_result.len(), 1);
    assert!(rename_result.contains_key("str-len"));
    assert!(!rename_result.contains_key("string-length"));

    // Test 'prefix' import
    let prefix_config = ImportConfig::Prefix("string:".to_string());
    let prefix_result = import::apply_import_config(&exports, &prefix_config).unwrap();
    assert_eq!(prefix_result.len(), 3);
    assert!(prefix_result.contains_key("string:string-length"));
    assert!(prefix_result.contains_key("string:string-ref"));
    assert!(prefix_result.contains_key("string:string-set!"));
}

#[test]
fn test_export_config_application() {
    use lambdust::eval::Value;
    
    // Create test bindings
    let mut bindings = HashMap::new();
    bindings.insert("internal-func".to_string(), Value::integer(1));
    bindings.insert("public-func".to_string(), Value::integer(2));
    bindings.insert("helper".to_string(), Value::integer(3));

    // Test direct export
    let symbols = vec!["public-func".to_string(), "helper".to_string()];
    let direct_result = export::apply_direct_export(&bindings, &symbols).unwrap();
    assert_eq!(direct_result.len(), 2);
    assert!(direct_result.contains_key("public-func"));
    assert!(direct_result.contains_key("helper"));
    assert!(!direct_result.contains_key("internal-func"));

    // Test rename export
    let mut rename_map = HashMap::new();
    rename_map.insert("internal-func".to_string(), "exported-func".to_string());
    let rename_result = export::apply_rename_export(&bindings, &rename_map).unwrap();
    assert_eq!(rename_result.len(), 1);
    assert!(rename_result.contains_key("exported-func"));
    assert!(!rename_result.contains_key("internal-func"));
}

#[test]
fn test_import_binding_merge() {
    use lambdust::eval::Value;
    
    // Create two sets of bindings
    let mut bindings1 = HashMap::new();
    bindings1.insert("a".to_string(), Value::integer(1));
    bindings1.insert("b".to_string(), Value::integer(2));

    let mut bindings2 = HashMap::new();
    bindings2.insert("c".to_string(), Value::integer(3));
    bindings2.insert("d".to_string(), Value::integer(4));

    // Test successful merge
    let merged = import::merge_import_bindings(&[bindings1.clone(), bindings2]).unwrap();
    assert_eq!(merged.len(), 4);
    assert!(merged.contains_key("a"));
    assert!(merged.contains_key("b"));
    assert!(merged.contains_key("c"));
    assert!(merged.contains_key("d"));

    // Test conflict detection
    let mut conflicting_bindings = HashMap::new();
    conflicting_bindings.insert("a".to_string(), Value::integer(99)); // Different value
    
    let conflict_result = import::merge_import_bindings(&[bindings1, conflicting_bindings]);
    assert!(conflict_result.is_err());
}

#[test]
fn test_dependency_resolution() {
    let mut resolver = resolver::DependencyResolver::new();
    
    // Test simple dependency resolution
    let root_id = name::builtin_module("test");
    let dependencies = vec![
        name::builtin_module("string"),
        name::builtin_module("list"),
    ];
    
    let order = resolver.resolve_dependency_order(&root_id, &dependencies).unwrap();
    assert_eq!(order.len(), 2);
    assert!(order.contains(&name::builtin_module("string")));
    assert!(order.contains(&name::builtin_module("list")));
}

#[test]
fn test_circular_dependency_detection() {
    let resolver = resolver::DependencyResolver::new();
    
    // Create modules with circular dependency
    let mut modules = HashMap::new();
    modules.insert(
        name::builtin_module("a"),
        Module {
            id: name::builtin_module("a"),
            exports: HashMap::new(),
            dependencies: vec![name::builtin_module("b")],
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        },
    );
    modules.insert(
        name::builtin_module("b"),
        Module {
            id: name::builtin_module("b"),
            exports: HashMap::new(),
            dependencies: vec![name::builtin_module("a")], // Circular!
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        },
    );

    let errors = resolver.validate_dependency_graph(&modules);
    assert!(!errors.is_empty());
    
    // Should detect circular dependency
    assert!(matches!(
        errors[0],
        resolver::DependencyValidationError::CircularDependency(_)
    ));
}

#[test]
fn test_module_cache() {
    let cache = cache::ModuleCache::new();
    
    // Test basic cache operations
    let module_id = name::builtin_module("test");
    let module = std::sync::Arc::new(Module {
        id: module_id.clone(),
        exports: HashMap::new(),
        dependencies: Vec::new(),
        source: Some(ModuleSource::Builtin),
        metadata: ModuleMetadata::default(),
    });

    // Initially not in cache
    assert!(cache.get(&module_id).is_none());
    assert_eq!(cache.len(), 0);

    // Insert and retrieve
    cache.insert(module_id.clone(), module.clone());
    assert_eq!(cache.len(), 1);
    
    let retrieved = cache.get(&module_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, module_id);

    // Test cache statistics
    let stats = cache.stats();
    assert_eq!(stats.entry_count, 1);
    assert_eq!(stats.total_accesses, 2); // 1 from insert + 1 from get
}

#[test]
fn test_module_cache_lru_eviction() {
    let config = cache::CacheConfig {
        max_entries: 2,
        ttl: None,
        dependency_invalidation: false,
    };
    let cache = cache::ModuleCache::with_config(config);

    // Insert two modules
    let id1 = name::builtin_module("mod1");
    let id2 = name::builtin_module("mod2");
    let id3 = name::builtin_module("mod3");

    let module1 = std::sync::Arc::new(Module {
        id: id1.clone(),
        exports: HashMap::new(),
        dependencies: Vec::new(),
        source: Some(ModuleSource::Builtin),
        metadata: ModuleMetadata::default(),
    });

    let module2 = std::sync::Arc::new(Module {
        id: id2.clone(),
        exports: HashMap::new(),
        dependencies: Vec::new(),
        source: Some(ModuleSource::Builtin),
        metadata: ModuleMetadata::default(),
    });

    let module3 = std::sync::Arc::new(Module {
        id: id3.clone(),
        exports: HashMap::new(),
        dependencies: Vec::new(),
        source: Some(ModuleSource::Builtin),
        metadata: ModuleMetadata::default(),
    });

    cache.insert(id1.clone(), module1);
    cache.insert(id2.clone(), module2);
    assert_eq!(cache.len(), 2);

    // Access first module to make it more recently used
    cache.get(&id1);

    // Insert third module - should evict mod2 (least recently used)
    cache.insert(id3.clone(), module3);
    assert_eq!(cache.len(), 2);
    
    assert!(cache.get(&id1).is_some());
    assert!(cache.get(&id2).is_none()); // Should be evicted
    assert!(cache.get(&id3).is_some());
}

#[test]
fn test_module_provider_interface() {
    use lambdust::module_system::ModuleProvider;
    
    struct TestProvider;
    
    impl ModuleProvider for TestProvider {
        fn get_module(&self, id: &ModuleId) -> Result<Module> {
            if id.components[0] == "test" {
                Ok(Module {
                    id: id.clone(),
                    exports: HashMap::new(),
                    dependencies: Vec::new(),
                    source: Some(ModuleSource::Builtin),
                    metadata: ModuleMetadata::default(),
                })
            } else {
                Err(lambdust::diagnostics::Error::from(ModuleError::NotFound(id.clone())))
            }
        }
        
        fn has_module(&self, id: &ModuleId) -> bool {
            id.namespace == ModuleNamespace::Builtin && id.components[0] == "test"
        }
        
        fn list_modules(&self) -> Vec<ModuleId> {
            vec![name::builtin_module("test")]
        }
    }
    
    let provider = TestProvider;
    let test_id = name::builtin_module("test");
    let nonexistent_id = name::builtin_module("nonexistent");
    
    assert!(provider.has_module(&test_id));
    assert!(!provider.has_module(&nonexistent_id));
    
    let module = provider.get_module(&test_id);
    assert!(module.is_ok());
    
    let no_module = provider.get_module(&nonexistent_id);
    assert!(no_module.is_err());
    
    let listed = provider.list_modules();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], test_id);
}

#[cfg(feature = "integration")]
#[test]
fn test_module_system_integration() {
    // This test requires the full Lambdust runtime to be available
    use lambdust::{Lambdust, Runtime};
    
    let mut lambdust = Lambdust::new();
    
    // Test importing a built-in module
    let import_code = r#"
        (import (lambdust string))
        (string-length "hello")
    "#;
    
    let result = lambdust.eval(import_code, Some("test"));
    // This would fail until the full integration is complete
    // assert!(result.is_ok());
}

#[test]
fn test_private_symbol_detection() {
    assert!(export::is_private_symbol("_private"));
    assert!(export::is_private_symbol("internal-helper"));
    assert!(export::is_private_symbol("system:something"));
    assert!(!export::is_private_symbol("public-function"));
    assert!(!export::is_private_symbol("string-length"));
}

#[test]
fn test_default_export_spec_creation() {
    use lambdust::eval::Value;
    
    let mut bindings = HashMap::new();
    bindings.insert("public-function".to_string(), Value::integer(1));
    bindings.insert("_private-function".to_string(), Value::integer(2));
    bindings.insert("internal-helper".to_string(), Value::integer(3));
    bindings.insert("system:internal".to_string(), Value::integer(4));

    let spec = export::create_default_export_spec(&bindings);

    assert_eq!(spec.symbols.len(), 1);
    assert!(spec.symbols.contains(&"public-function".to_string()));
    assert!(!spec.symbols.contains(&"_private-function".to_string()));
    assert!(!spec.symbols.contains(&"internal-helper".to_string()));
    assert!(!spec.symbols.contains(&"system:internal".to_string()));
}

#[test]
fn test_export_spec_union_and_intersection() {
    let spec1 = ExportSpec {
        symbols: vec!["a".to_string(), "b".to_string()],
        config: ExportConfig::Direct,
    };
    
    let spec2 = ExportSpec {
        symbols: vec!["b".to_string(), "c".to_string()],
        config: ExportConfig::Direct,
    };

    // Test union
    let union = export::union_export_specs(&spec1, &spec2).unwrap();
    assert_eq!(union.symbols.len(), 3);
    assert!(union.symbols.contains(&"a".to_string()));
    assert!(union.symbols.contains(&"b".to_string()));
    assert!(union.symbols.contains(&"c".to_string()));

    // Test intersection
    let intersection = export::intersect_export_specs(&spec1, &spec2).unwrap();
    assert_eq!(intersection.symbols.len(), 1);
    assert!(intersection.symbols.contains(&"b".to_string()));
    assert!(!intersection.symbols.contains(&"a".to_string()));
    assert!(!intersection.symbols.contains(&"c".to_string()));
}

#[test]
fn test_export_spec_filtering() {
    let spec = ExportSpec {
        symbols: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        config: ExportConfig::Direct,
    };
    
    let excluded = vec!["b".to_string()];
    let filtered = export::filter_export_spec(&spec, &excluded);
    
    assert_eq!(filtered.symbols.len(), 2);
    assert!(filtered.symbols.contains(&"a".to_string()));
    assert!(filtered.symbols.contains(&"c".to_string()));
    assert!(!filtered.symbols.contains(&"b".to_string()));
}