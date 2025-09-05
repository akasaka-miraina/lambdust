//! Feature Registry System
//!
//! Central registry for all available features, categorized by type.
//! Provides O(1) feature lookup and efficient library identifier matching.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Categories of features available in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureCategory {
    /// Core Lambdust features (lambdust, r7rs-small, etc.)
    Core,
    /// SRFI implementations (srfi-1, srfi-9, etc.)
    Srfi(u32),
    /// Platform-specific features (unix, windows, x86-64, etc.)
    Platform,
    /// Runtime capability features (simd, nan-boxing, jit, etc.)
    Capability,
    /// Available library modules
    Library,
    /// User-defined or external features
    External,
}

/// Information about a specific feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// Feature name
    pub name: String,
    /// Category classification  
    pub category: FeatureCategory,
    /// Human-readable description
    pub description: String,
    /// Whether this feature is enabled by default
    pub default_enabled: bool,
    /// Runtime detection function (for capability features)
    pub detector: Option<String>,
    /// Dependencies required for this feature
    pub dependencies: Vec<String>,
}

impl FeatureInfo {
    /// Creates a new core feature
    pub fn core(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: FeatureCategory::Core,
            description: description.into(),
            default_enabled: true,
            detector: None,
            dependencies: Vec::new(),
        }
    }

    /// Creates a new SRFI feature
    pub fn srfi(number: u32, description: impl Into<String>) -> Self {
        Self {
            name: format!("srfi-{}", number),
            category: FeatureCategory::Srfi(number),
            description: description.into(),
            default_enabled: false,
            detector: None,
            dependencies: Vec::new(),
        }
    }

    /// Creates a new platform feature
    pub fn platform(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: FeatureCategory::Platform,
            description: description.into(),
            default_enabled: false,
            detector: Some("platform_detector".to_string()),
            dependencies: Vec::new(),
        }
    }

    /// Creates a new capability feature
    pub fn capability(
        name: impl Into<String>,
        description: impl Into<String>,
        detector: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category: FeatureCategory::Capability,
            description: description.into(),
            default_enabled: false,
            detector,
            dependencies: Vec::new(),
        }
    }

    /// Creates a new library feature
    pub fn library(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: FeatureCategory::Library,
            description: description.into(),
            default_enabled: false,
            detector: None,
            dependencies: Vec::new(),
        }
    }

    /// Adds dependencies to this feature
    pub fn with_dependencies(mut self, deps: impl IntoIterator<Item = String>) -> Self {
        self.dependencies.extend(deps);
        self
    }
}

/// Library path trie for efficient hierarchical matching  
#[derive(Debug, Clone, Default)]
struct LibraryTrie {
    children: HashMap<String, Box<LibraryTrie>>,
    is_endpoint: bool,
    metadata: Option<String>,
}

impl LibraryTrie {
    /// Inserts a library path into the trie
    fn insert(&mut self, path: &[String], metadata: Option<String>) {
        let mut current = self;
        for component in path {
            current = current
                .children
                .entry(component.clone())
                .or_insert_with(|| Box::new(LibraryTrie::default()));
        }
        current.is_endpoint = true;
        current.metadata = metadata;
    }

    /// Checks if a library path exists in the trie
    fn contains(&self, path: &[String]) -> bool {
        let mut current = self;
        for component in path {
            match current.children.get(component) {
                Some(child) => current = child,
                None => return false,
            }
        }
        current.is_endpoint
    }

    /// Gets metadata for a library path
    fn get_metadata(&self, path: &[String]) -> Option<&String> {
        let mut current = self;
        for component in path {
            current = current.children.get(component)?;
        }
        if current.is_endpoint {
            current.metadata.as_ref()
        } else {
            None
        }
    }
}

/// Central feature registry
#[derive(Debug, Clone)]
pub struct FeatureRegistry {
    /// All registered features by name
    features: HashMap<String, FeatureInfo>,
    /// Feature names by category
    features_by_category: HashMap<FeatureCategory, HashSet<String>>,
    /// SRFI features indexed by number
    srfi_map: HashMap<u32, String>,
    /// Library identifier trie for efficient lookup
    library_trie: LibraryTrie,
    /// Platform-specific features (detected at runtime)
    platform_features: HashSet<String>,
    /// Capability features (detected at runtime)  
    capability_features: HashMap<String, bool>,
}

impl FeatureRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            features_by_category: HashMap::new(),
            srfi_map: HashMap::new(),
            library_trie: LibraryTrie::default(),
            platform_features: HashSet::new(),
            capability_features: HashMap::new(),
        }
    }

    /// Registers a feature in the registry
    pub fn register_feature(&mut self, info: FeatureInfo) {
        let name = info.name.clone();
        let category = info.category.clone();

        // Index by category
        self.features_by_category
            .entry(category.clone())
            .or_default()
            .insert(name.clone());

        // Special handling for SRFI features
        if let FeatureCategory::Srfi(number) = &category {
            self.srfi_map.insert(*number, name.clone());
        }

        // Store the feature
        self.features.insert(name, info);
    }

    /// Registers a library in the registry
    pub fn register_library(&mut self, path: &[String], description: Option<String>) {
        self.library_trie.insert(path, description.clone());

        // Also register as a feature
        let name = format!("({})", path.join(" "));
        let info = FeatureInfo::library(
            name,
            description.unwrap_or_else(|| "Library module".to_string()),
        );
        self.register_feature(info);
    }

    /// Checks if a feature exists
    pub fn has_feature(&self, name: &str) -> bool {
        if self.features.contains_key(name) {
            return true;
        }

        // Check for SRFI shorthand (srfi-N)
        if let Some(number) = parse_srfi_name(name) {
            return self.srfi_map.contains_key(&number);
        }

        // Check platform features
        if self.platform_features.contains(name) {
            return true;
        }

        // Check capability features
        self.capability_features.get(name).copied().unwrap_or(false)
    }

    /// Checks if a library is available
    pub fn has_library(&self, path: &[String]) -> bool {
        self.library_trie.contains(path)
    }

    /// Gets feature information
    pub fn get_feature(&self, name: &str) -> Option<&FeatureInfo> {
        self.features.get(name)
    }

    /// Gets library metadata
    pub fn get_library_metadata(&self, path: &[String]) -> Option<&String> {
        self.library_trie.get_metadata(path)
    }

    /// Lists all features in a category
    pub fn features_in_category(&self, category: &FeatureCategory) -> Option<&HashSet<String>> {
        self.features_by_category.get(category)
    }

    /// Lists all registered features
    pub fn all_features(&self) -> impl Iterator<Item = &FeatureInfo> {
        self.features.values()
    }

    /// Adds a platform feature (typically called by detector)
    pub fn add_platform_feature(&mut self, name: String) {
        self.platform_features.insert(name);
    }

    /// Sets capability feature status (typically called by detector)
    pub fn set_capability_feature(&mut self, name: String, available: bool) {
        self.capability_features.insert(name, available);
    }

    /// Gets SRFI feature name by number
    pub fn get_srfi_feature(&self, number: u32) -> Option<&String> {
        self.srfi_map.get(&number)
    }

    /// Lists all available SRFIs
    pub fn available_srfis(&self) -> impl Iterator<Item = (u32, &String)> {
        self.srfi_map.iter().map(|(k, v)| (*k, v))
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register core Lambdust features
        registry.register_feature(FeatureInfo::core(
            "lambdust",
            "Lambdust Scheme implementation",
        ));
        registry.register_feature(FeatureInfo::core(
            "r7rs-small",
            "R7RS-small standard compliance",
        ));
        registry.register_feature(FeatureInfo::core(
            "ieee-float",
            "IEEE 754 floating point support",
        ));
        registry.register_feature(FeatureInfo::core(
            "full-unicode",
            "Full Unicode character support",
        ));
        registry.register_feature(FeatureInfo::core("ratios", "Exact rational number support"));
        registry.register_feature(FeatureInfo::core("complex", "Complex number support"));

        // Register SRFI features (commonly implemented)
        registry.register_feature(FeatureInfo::srfi(
            0,
            "Feature-based conditional expansion of code",
        ));
        registry.register_feature(FeatureInfo::srfi(1, "List Library"));
        registry.register_feature(FeatureInfo::srfi(9, "Defining Record Types"));
        registry.register_feature(FeatureInfo::srfi(23, "Error reporting mechanism"));
        registry.register_feature(FeatureInfo::srfi(39, "Parameter objects"));

        // Register Lambdust-specific capability features
        registry.register_feature(FeatureInfo::capability(
            "nan-boxing",
            "NaN-boxing value representation",
            Some("nan_boxing_detector".to_string()),
        ));
        registry.register_feature(FeatureInfo::capability(
            "simd",
            "SIMD instruction support",
            Some("simd_detector".to_string()),
        ));
        registry.register_feature(FeatureInfo::capability(
            "jit",
            "Just-in-time compilation",
            Some("jit_detector".to_string()),
        ));
        registry.register_feature(FeatureInfo::capability(
            "parallel-gc",
            "Parallel garbage collection",
            Some("parallel_gc_detector".to_string()),
        ));
        registry.register_feature(FeatureInfo::capability(
            "distributed",
            "Distributed computing support",
            Some("distributed_detector".to_string()),
        ));

        // Register common libraries
        registry.register_library(
            &["lambdust".to_string(), "core".to_string()],
            Some("Core Lambdust library".to_string()),
        );
        registry.register_library(
            &["lambdust".to_string(), "numeric".to_string()],
            Some("Numeric library".to_string()),
        );
        registry.register_library(
            &["lambdust".to_string(), "containers".to_string()],
            Some("Container data structures".to_string()),
        );
        registry.register_library(
            &["scheme".to_string(), "base".to_string()],
            Some("R7RS base library".to_string()),
        );
        registry.register_library(
            &["scheme".to_string(), "char".to_string()],
            Some("Character library".to_string()),
        );

        registry
    }
}

/// Parses SRFI feature names like "srfi-1" to extract the number
fn parse_srfi_name(name: &str) -> Option<u32> {
    if let Some(suffix) = name.strip_prefix("srfi-") {
        suffix.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_registration() {
        let mut registry = FeatureRegistry::new();

        let feature = FeatureInfo::core("test-feature", "A test feature");
        registry.register_feature(feature);

        assert!(registry.has_feature("test-feature"));
        assert!(!registry.has_feature("unknown-feature"));

        let info = registry.get_feature("test-feature").unwrap();
        assert_eq!(info.name, "test-feature");
        assert_eq!(info.description, "A test feature");
    }

    #[test]
    fn test_library_registration() {
        let mut registry = FeatureRegistry::new();

        let path = vec!["test".to_string(), "lib".to_string()];
        registry.register_library(&path, Some("Test library".to_string()));

        assert!(registry.has_library(&path));
        assert!(!registry.has_library(&["unknown".to_string()]));

        let metadata = registry.get_library_metadata(&path).unwrap();
        assert_eq!(metadata, "Test library");
    }

    #[test]
    fn test_library_trie() {
        let mut trie = LibraryTrie::default();

        let path1 = vec!["srfi".to_string(), "1".to_string()];
        let path2 = vec!["srfi".to_string(), "9".to_string()];
        let path3 = vec!["lambdust".to_string(), "core".to_string()];

        trie.insert(&path1, Some("SRFI-1".to_string()));
        trie.insert(&path2, Some("SRFI-9".to_string()));
        trie.insert(&path3, Some("Core".to_string()));

        assert!(trie.contains(&path1));
        assert!(trie.contains(&path2));
        assert!(trie.contains(&path3));
        assert!(!trie.contains(&["unknown".to_string()]));

        assert_eq!(trie.get_metadata(&path1), Some(&"SRFI-1".to_string()));
        assert_eq!(trie.get_metadata(&["unknown".to_string()]), None);
    }

    #[test]
    fn test_srfi_parsing() {
        assert_eq!(parse_srfi_name("srfi-1"), Some(1));
        assert_eq!(parse_srfi_name("srfi-23"), Some(23));
        assert_eq!(parse_srfi_name("not-srfi"), None);
        assert_eq!(parse_srfi_name("srfi-invalid"), None);
    }

    #[test]
    fn test_default_registry() {
        let registry = FeatureRegistry::default();

        // Should have core features
        assert!(registry.has_feature("lambdust"));
        assert!(registry.has_feature("r7rs-small"));
        assert!(registry.has_feature("ieee-float"));

        // Should have SRFI features
        assert!(registry.has_feature("srfi-0"));
        assert!(registry.has_feature("srfi-1"));
        assert!(registry.has_feature("srfi-9"));

        // Should have capability features
        assert!(registry.has_feature("nan-boxing"));
        assert!(registry.has_feature("simd"));
        assert!(registry.has_feature("jit"));

        // Should have libraries
        assert!(registry.has_library(&["lambdust".to_string(), "core".to_string()]));
        assert!(registry.has_library(&["scheme".to_string(), "base".to_string()]));
    }

    #[test]
    fn test_categories() {
        let registry = FeatureRegistry::default();

        let core_features = registry
            .features_in_category(&FeatureCategory::Core)
            .unwrap();
        assert!(core_features.contains("lambdust"));
        assert!(core_features.contains("r7rs-small"));

        let srfi_features = registry.features_in_category(&FeatureCategory::Srfi(1));
        assert!(srfi_features.is_none() || srfi_features.unwrap().contains("srfi-1"));
    }
}
