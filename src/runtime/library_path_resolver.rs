//! Library path resolution system for Lambdust.
//!
//! This module provides centralized library path discovery that supports:
//! - LAMBDUST_LIB_DIR environment variable for installed deployments
//! - Fallback to development paths (./stdlib)
//! - Multiple search path priorities with graceful degradation
//! - Comprehensive error reporting for missing libraries

use crate::diagnostics::{Error, Result};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;

/// Centralized library path resolver for all Lambdust components.
#[derive(Debug, Clone)]
pub struct LibraryPathResolver {
    /// Primary library directory (from LAMBDUST_LIB_DIR or auto-detected)
    primary_lib_dir: Option<PathBuf>,
    /// Additional search paths in priority order
    search_paths: Vec<PathBuf>,
    /// Whether development mode paths should be included
    include_dev_paths: bool,
    /// Cache of resolved paths to avoid repeated filesystem operations
    #[allow(dead_code)]
    path_cache: std::collections::HashMap<String, Option<PathBuf>>,
}

/// Configuration for library path resolution.
#[derive(Debug, Clone)]
pub struct LibraryPathConfig {
    /// Whether to include development paths (./stdlib)
    pub include_dev_paths: bool,
    /// Additional search paths to include
    pub additional_paths: Vec<PathBuf>,
    /// Whether to use aggressive caching
    pub enable_caching: bool,
    /// Override for LAMBDUST_LIB_DIR (for testing)
    pub lib_dir_override: Option<PathBuf>,
}

impl Default for LibraryPathConfig {
    fn default() -> Self {
        Self {
            include_dev_paths: true,
            additional_paths: Vec::new(),
            enable_caching: true,
            lib_dir_override: None,
        }
    }
}

impl LibraryPathResolver {
    /// Creates a new library path resolver with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(LibraryPathConfig::default())
    }

    /// Creates a library path resolver with custom configuration.
    pub fn with_config(config: LibraryPathConfig) -> Result<Self> {
        let mut resolver = Self {
            primary_lib_dir: None,
            search_paths: Vec::new(), 
            include_dev_paths: config.include_dev_paths,
            path_cache: std::collections::HashMap::new(),
        };

        // Determine primary library directory
        resolver.primary_lib_dir = resolver.determine_primary_lib_dir(config.lib_dir_override.as_deref())?;

        // Build search paths in priority order
        resolver.build_search_paths(&config.additional_paths);

        Ok(resolver)
    }

    /// Gets the primary library directory path.
    pub fn primary_lib_dir(&self) -> Option<&Path> {
        self.primary_lib_dir.as_deref()
    }

    /// Gets all search paths in priority order.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Resolves the path to a library subdirectory (e.g., "r7rs", "bootstrap", "modules").
    pub fn resolve_lib_subdir(&self, subdir: &str) -> Result<PathBuf> {
        // Check primary library directory first
        if let Some(primary) = &self.primary_lib_dir {
            let subdir_path = primary.join(subdir);
            if subdir_path.exists() && subdir_path.is_dir() {
                return Ok(subdir_path);
            }
        }

        // Check search paths
        for search_path in &self.search_paths {
            let subdir_path = search_path.join(subdir);
            if subdir_path.exists() && subdir_path.is_dir() {
                return Ok(subdir_path);
            }
        }

        Err(Box::new(Error::io_error(format!(
            "Library subdirectory '{}' not found in any search path. \
             Primary lib dir: {:?}, Search paths: {:?}. \
             Consider setting LAMBDUST_LIB_DIR environment variable.",
            subdir, self.primary_lib_dir, self.search_paths
        ))))
    }

    /// Resolves the full path to a specific library file.
    pub fn resolve_library_file(&self, subdir: &str, filename: &str) -> Result<PathBuf> {
        println!("Debug: LibraryPathResolver::resolve_library_file called: subdir={subdir}, filename={filename}");
        println!("Debug: Primary lib dir: {:?}", self.primary_lib_dir);
        println!("Debug: Search paths count: {}", self.search_paths.len());
        
        // Check primary library directory first
        if let Some(primary) = &self.primary_lib_dir {
            let file_path = primary.join(subdir).join(filename);
            println!("Debug: Checking primary path: {}", file_path.display());
            if file_path.exists() && file_path.is_file() {
                println!("Debug: Found file at primary path: {}", file_path.display());
                return Ok(file_path);
            }
        }

        // Check search paths
        for (i, search_path) in self.search_paths.iter().enumerate() {
            let file_path = search_path.join(subdir).join(filename);
            println!("Debug: Checking search path {}: {}", i, file_path.display());
            if file_path.exists() && file_path.is_file() {
                println!("Debug: Found file at search path {}: {}", i, file_path.display());
                return Ok(file_path);
            }
        }

        println!("Debug: Library file not found in any path");
        Err(Box::new(Error::io_error(format!(
            "Library file '{}/{}' not found in any search path. \
             Primary lib dir: {:?}, Search paths: {:?}. \
             Consider setting LAMBDUST_LIB_DIR environment variable.",
            subdir, filename, self.primary_lib_dir, self.search_paths
        ))))
    }

    /// Finds all available library files in a subdirectory.
    pub fn find_library_files(&self, subdir: &str, extension: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        // Search in primary library directory
        if let Some(primary) = &self.primary_lib_dir {
            let dir_path = primary.join(subdir);
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(extension) {
                            files.push(entry.path());
                        }
                    }
                }
            }
        }

        // Search in other search paths (avoiding duplicates)
        for search_path in &self.search_paths {
            let dir_path = search_path.join(subdir);
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(extension) {
                            let entry_path = entry.path();
                            // Avoid duplicates by checking if we already have this filename
                            if !files.iter().any(|f| f.file_name() == entry_path.file_name()) {
                                files.push(entry_path);
                            }
                        }
                    }
                }
            }
        }

        files
    }

    /// Checks if the library system is properly configured.
    pub fn validate_library_setup(&self) -> Result<LibraryValidationReport> {
        let mut report = LibraryValidationReport {
            primary_lib_dir_valid: false,
            found_search_paths: Vec::new(),
            missing_critical_subdirs: Vec::new(),
            found_library_files: std::collections::HashMap::new(),
            recommendations: Vec::new(),
        };

        // Validate primary library directory
        if let Some(primary) = &self.primary_lib_dir {
            report.primary_lib_dir_valid = primary.exists() && primary.is_dir();
            if report.primary_lib_dir_valid {
                report.found_search_paths.push(primary.clone());
            }
        }

        // Check search paths
        for path in &self.search_paths {
            if path.exists() && path.is_dir() {
                report.found_search_paths.push(path.clone());
            }
        }

        // Check for critical subdirectories
        let critical_subdirs = ["bootstrap", "r7rs", "modules"];
        for &subdir in &critical_subdirs {
            if self.resolve_lib_subdir(subdir).is_err() {
                report.missing_critical_subdirs.push(subdir.to_string());
            }
        }

        // Count library files in each subdirectory
        for &subdir in &critical_subdirs {
            let files = self.find_library_files(subdir, ".scm");
            report.found_library_files.insert(subdir.to_string(), files.len());
        }

        // Generate recommendations
        if !report.primary_lib_dir_valid && env::var("LAMBDUST_LIB_DIR").is_err() {
            report.recommendations.push(
                "Consider setting LAMBDUST_LIB_DIR environment variable to point to your library directory".to_string()
            );
        }

        if report.found_search_paths.is_empty() {
            report.recommendations.push(
                "No valid library directories found. Ensure stdlib directory exists in your installation".to_string()
            );
        }

        if !report.missing_critical_subdirs.is_empty() {
            report.recommendations.push(format!(
                "Missing critical library subdirectories: {}. Check your installation",
                report.missing_critical_subdirs.join(", ")
            ));
        }

        Ok(report)
    }

    /// Determines the primary library directory from environment or auto-detection.
    fn determine_primary_lib_dir(&self, override_path: Option<&Path>) -> Result<Option<PathBuf>> {
        // Use override if provided (for testing)
        if let Some(override_path) = override_path {
            return Ok(Some(override_path.to_path_buf()));
        }

        // Check LAMBDUST_LIB_DIR environment variable first
        if let Ok(lib_dir_str) = env::var("LAMBDUST_LIB_DIR") {
            let lib_dir = PathBuf::from(lib_dir_str);
            if lib_dir.exists() && lib_dir.is_dir() {
                return Ok(Some(lib_dir));
            } else {
                return Err(Box::new(Error::io_error(format!(
                    "LAMBDUST_LIB_DIR points to invalid directory: {}. \
                     Directory does not exist or is not accessible.",
                    lib_dir.display()
                ))));
            }
        }

        // Auto-detect based on executable location and current directory
        // This will be populated by build_search_paths, so return None for now
        Ok(None)
    }

    /// Builds the search paths list in priority order.
    fn build_search_paths(&mut self, additional_paths: &[PathBuf]) {
        self.search_paths.clear();

        // Add primary lib dir if available
        if let Some(primary) = &self.primary_lib_dir {
            self.search_paths.push(primary.clone());
        }

        // Add additional paths from configuration
        for path in additional_paths {
            if path.exists() && path.is_dir() {
                self.search_paths.push(path.clone());
            }
        }

        // Add auto-detected paths
        self.add_auto_detected_paths();

        // Add development paths if enabled
        if self.include_dev_paths {
            self.add_development_paths();
        }

        // Remove duplicates while preserving order
        self.search_paths.dedup();
    }

    /// Adds automatically detected library paths.
    fn add_auto_detected_paths(&mut self) {
        // Path relative to executable (for cargo install scenario)
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check ../lib/lambdust (typical Unix installation)
                if let Some(install_root) = exe_dir.parent() {
                    let lib_path = install_root.join("lib").join("lambdust");
                    if lib_path.exists() && lib_path.is_dir() {
                        self.search_paths.push(lib_path);
                    }
                }
                
                // Check ./stdlib (binary next to stdlib)
                let exe_stdlib = exe_dir.join("stdlib");
                if exe_stdlib.exists() && exe_stdlib.is_dir() {
                    self.search_paths.push(exe_stdlib);
                }
            }
        }

        // System-wide installation paths
        #[cfg(unix)]
        {
            let system_paths = [
                "/usr/local/share/lambdust",
                "/usr/share/lambdust",
                "/opt/lambdust/lib",
            ];
            
            for &path_str in &system_paths {
                let path = PathBuf::from(path_str);
                if path.exists() && path.is_dir() {
                    self.search_paths.push(path);
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows-specific paths
            if let Ok(program_files) = env::var("ProgramFiles") {
                let lambdust_path = PathBuf::from(program_files).join("Lambdust").join("lib");
                if lambdust_path.exists() && lambdust_path.is_dir() {
                    self.search_paths.push(lambdust_path);
                }
            }
        }
    }

    /// Adds development-specific paths.
    fn add_development_paths(&mut self) {
        // Current directory stdlib (development scenario)
        if let Ok(current_dir) = env::current_dir() {
            let dev_stdlib = current_dir.join("stdlib"); 
            if dev_stdlib.exists() && dev_stdlib.is_dir() {
                self.search_paths.push(dev_stdlib);
            }
        }

        // User-specific library directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_lib = home_dir.join(".lambdust").join("lib");
            if user_lib.exists() && user_lib.is_dir() {
                self.search_paths.push(user_lib);
            }
        }
    }
}

/// Validation report for the library system setup.
#[derive(Debug, Clone)]
pub struct LibraryValidationReport {
    /// Whether the primary library directory is valid
    pub primary_lib_dir_valid: bool,
    /// List of valid search paths found
    pub found_search_paths: Vec<PathBuf>,
    /// List of missing critical subdirectories
    pub missing_critical_subdirs: Vec<String>,
    /// Count of library files found in each subdirectory
    pub found_library_files: std::collections::HashMap<String, usize>,
    /// Recommendations for fixing issues
    pub recommendations: Vec<String>,
}

impl LibraryValidationReport {
    /// Checks if the library system is in a usable state.
    pub fn is_usable(&self) -> bool {
        !self.found_search_paths.is_empty() && self.missing_critical_subdirs.len() < 2
    }

    /// Gets a summary of the validation results.
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("Library validation summary:\n");
        summary.push_str(&format!("• Primary lib dir valid: {}\n", self.primary_lib_dir_valid));
        summary.push_str(&format!("• Valid search paths found: {}\n", self.found_search_paths.len()));
        summary.push_str(&format!("• Missing critical subdirs: {}\n", self.missing_critical_subdirs.len()));
        
        if !self.missing_critical_subdirs.is_empty() {
            summary.push_str(&format!("  Missing: {}\n", self.missing_critical_subdirs.join(", ")));
        }
        
        for (subdir, count) in &self.found_library_files {
            summary.push_str(&format!("• {count} library files in {subdir}\n"));
        }
        
        if !self.recommendations.is_empty() {
            summary.push_str("\nRecommendations:\n");
            for rec in &self.recommendations {
                summary.push_str(&format!("• {rec}\n"));
            }
        }
        
        summary
    }
}

// External dependency for home directory detection
#[cfg(not(test))]
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

// Mock implementation for tests
#[cfg(test)]
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        Some(PathBuf::from("/tmp/test-home"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_library_path_resolver_creation() {
        let resolver = LibraryPathResolver::new();
        assert!(resolver.is_ok());
    }

    #[test]
    fn test_with_lambdust_lib_dir_env() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create some subdirectories
        fs::create_dir_all(lib_dir.join("r7rs")).unwrap();
        fs::create_dir_all(lib_dir.join("bootstrap")).unwrap();
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir.clone()),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        assert_eq!(resolver.primary_lib_dir(), Some(lib_dir.as_path()));
        
        let r7rs_path = resolver.resolve_lib_subdir("r7rs");
        assert!(r7rs_path.is_ok());
        assert_eq!(r7rs_path.unwrap(), lib_dir.join("r7rs"));
    }

    #[test]
    fn test_library_file_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create test structure
        let r7rs_dir = lib_dir.join("r7rs");
        fs::create_dir_all(&r7rs_dir).unwrap();
        fs::write(r7rs_dir.join("base.scm"), "(define-library (scheme base) ...)").unwrap();
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        
        // Test successful resolution
        let base_file = resolver.resolve_library_file("r7rs", "base.scm");
        assert!(base_file.is_ok());
        
        // Test missing file
        let missing_file = resolver.resolve_library_file("r7rs", "missing.scm");
        assert!(missing_file.is_err());
    }

    #[test]
    fn test_find_library_files() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create test files
        let r7rs_dir = lib_dir.join("r7rs");
        fs::create_dir_all(&r7rs_dir).unwrap();
        fs::write(r7rs_dir.join("base.scm"), "").unwrap();
        fs::write(r7rs_dir.join("char.scm"), "").unwrap();
        fs::write(r7rs_dir.join("readme.txt"), "").unwrap(); // Should be ignored
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        let files = resolver.find_library_files("r7rs", ".scm");
        
        assert_eq!(files.len(), 2);
        let filenames: Vec<String> = files.iter()
            .filter_map(|p| p.file_name()?.to_str())
            .map(|s| s.to_string())
            .collect();
        assert!(filenames.contains(&"base.scm".to_string()));
        assert!(filenames.contains(&"char.scm".to_string()));
        assert!(!filenames.contains(&"readme.txt".to_string()));
    }

    #[test]
    fn test_validation_report() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create partial structure (missing some critical subdirs)
        fs::create_dir_all(lib_dir.join("r7rs")).unwrap();
        fs::write(lib_dir.join("r7rs").join("base.scm"), "").unwrap();
        // Don't create bootstrap directory to test missing subdir detection
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        let report = resolver.validate_library_setup().unwrap();
        
        assert!(report.primary_lib_dir_valid);
        assert_eq!(report.found_search_paths.len(), 1);
        assert!(report.missing_critical_subdirs.contains(&"bootstrap".to_string()));
        assert_eq!(report.found_library_files.get("r7rs"), Some(&1));
        
        let summary = report.summary();
        assert!(summary.contains("Primary lib dir valid: true"));
        assert!(summary.contains("Missing: bootstrap"));
    }

    #[test]
    fn test_error_handling_invalid_lib_dir() {
        let config = LibraryPathConfig {
            lib_dir_override: Some(PathBuf::from("/nonexistent/path")),
            ..Default::default()
        };
        
        let result = LibraryPathResolver::with_config(config);
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("LAMBDUST_LIB_DIR points to invalid directory"));
    }

    #[test]
    fn test_search_path_priority() {
        let temp_dir = TempDir::new().unwrap();
        let primary_dir = temp_dir.path().join("primary");
        let secondary_dir = temp_dir.path().join("secondary");
        
        // Create same file in both directories
        fs::create_dir_all(primary_dir.join("r7rs")).unwrap();
        fs::create_dir_all(secondary_dir.join("r7rs")).unwrap();
        fs::write(primary_dir.join("r7rs").join("base.scm"), "primary version").unwrap();
        fs::write(secondary_dir.join("r7rs").join("base.scm"), "secondary version").unwrap();
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(primary_dir.clone()),
            additional_paths: vec![secondary_dir],
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        let resolved_file = resolver.resolve_library_file("r7rs", "base.scm").unwrap();
        
        // Should resolve to primary directory (higher priority)
        assert_eq!(resolved_file, primary_dir.join("r7rs").join("base.scm"));
        
        let content = fs::read_to_string(&resolved_file).unwrap();
        assert_eq!(content, "primary version");
    }
}