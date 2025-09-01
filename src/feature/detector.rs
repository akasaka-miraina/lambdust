//! Platform and Capability Feature Detection
//!
//! Runtime detection of platform-specific features and system capabilities.
//! Detects OS, architecture, CPU features, and runtime optimizations.

use super::{FeatureCategory, FeatureInfo, FeatureRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Platform information detected at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system name
    pub os: String,
    /// Architecture name  
    pub arch: String,
    /// Target triple
    pub target: String,
    /// Pointer width in bits
    pub pointer_width: u8,
    /// Endianness (little or big)
    pub endian: String,
    /// Additional OS-specific features
    pub os_features: Vec<String>,
}

/// CPU capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// CPU vendor
    pub vendor: Option<String>,
    /// CPU brand string
    pub brand: Option<String>,
    /// Number of physical cores
    pub cores: Option<u32>,
    /// Number of logical threads
    pub threads: Option<u32>,
    /// Available SIMD instruction sets
    pub simd_features: Vec<String>,
    /// Cache sizes (L1, L2, L3)
    pub cache_sizes: HashMap<String, u64>,
    /// Additional CPU features
    pub cpu_features: Vec<String>,
}

/// Runtime capability flags
#[derive(Debug, Clone, Default)]
pub struct RuntimeCapabilities {
    /// NaN-boxing value representation available
    pub nan_boxing: bool,
    /// SIMD operations available
    pub simd: bool,
    /// JIT compilation available
    pub jit: bool,
    /// Parallel garbage collection available
    pub parallel_gc: bool,
    /// Distributed computing support available
    pub distributed: bool,
    /// Unsafe operations allowed
    pub unsafe_ops: bool,
    /// Thread-local storage available
    pub thread_local: bool,
    /// Atomic operations available
    pub atomics: bool,
}

/// Feature detector for platform and runtime capabilities
pub struct FeatureDetector {
    platform_info: PlatformInfo,
    capability_info: CapabilityInfo,
    runtime_capabilities: RuntimeCapabilities,
}

impl FeatureDetector {
    /// Creates a new feature detector and runs detection
    pub fn new() -> Self {
        let platform_info = Self::detect_platform();
        let capability_info = Self::detect_capabilities();
        let runtime_capabilities =
            Self::detect_runtime_capabilities(&platform_info, &capability_info);

        Self {
            platform_info,
            capability_info,
            runtime_capabilities,
        }
    }

    /// Updates a feature registry with detected features
    pub fn update_registry(&self, registry: &mut FeatureRegistry) {
        // Register platform features
        self.register_platform_features(registry);

        // Register capability features
        self.register_capability_features(registry);

        // Register runtime features
        self.register_runtime_features(registry);
    }

    /// Detects platform information
    fn detect_platform() -> PlatformInfo {
        PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            target: get_target_triple(),
            pointer_width: std::mem::size_of::<usize>() as u8 * 8,
            endian: if cfg!(target_endian = "little") {
                "little".to_string()
            } else {
                "big".to_string()
            },
            os_features: detect_os_features(),
        }
    }

    /// Detects CPU capabilities
    fn detect_capabilities() -> CapabilityInfo {
        CapabilityInfo {
            vendor: detect_cpu_vendor(),
            brand: detect_cpu_brand(),
            cores: detect_physical_cores(),
            threads: detect_logical_threads(),
            simd_features: detect_simd_features(),
            cache_sizes: detect_cache_sizes(),
            cpu_features: detect_cpu_features(),
        }
    }

    /// Detects runtime capabilities
    fn detect_runtime_capabilities(
        _platform: &PlatformInfo,
        capability: &CapabilityInfo,
    ) -> RuntimeCapabilities {
        RuntimeCapabilities {
            nan_boxing: detect_nan_boxing_support(),
            simd: !capability.simd_features.is_empty(),
            jit: detect_jit_support(),
            parallel_gc: detect_parallel_gc_support(),
            distributed: detect_distributed_support(),
            unsafe_ops: cfg!(feature = "unsafe-ops"),
            thread_local: has_thread_local_support(),
            atomics: has_atomic_support(),
        }
    }

    /// Registers platform-specific features
    fn register_platform_features(&self, registry: &mut FeatureRegistry) {
        let platform = &self.platform_info;

        // Operating system
        registry.add_platform_feature(platform.os.clone());
        registry.register_feature(FeatureInfo::platform(
            &platform.os,
            format!("Running on {}", platform.os),
        ));

        // Architecture
        registry.add_platform_feature(platform.arch.clone());
        registry.register_feature(FeatureInfo::platform(
            &platform.arch,
            format!("Running on {} architecture", platform.arch),
        ));

        // Pointer width
        let pointer_feature = format!("pointer-{}-bit", platform.pointer_width);
        registry.add_platform_feature(pointer_feature.clone());
        registry.register_feature(FeatureInfo::platform(
            pointer_feature,
            format!("{}-bit pointers", platform.pointer_width),
        ));

        // Endianness
        let endian_feature = format!("{}-endian", platform.endian);
        registry.add_platform_feature(endian_feature.clone());
        registry.register_feature(FeatureInfo::platform(
            endian_feature,
            format!("{} endian byte order", platform.endian),
        ));

        // OS-specific features
        for feature in &platform.os_features {
            registry.add_platform_feature(feature.clone());
            registry.register_feature(FeatureInfo::platform(
                feature,
                format!("OS-specific feature: {}", feature),
            ));
        }
    }

    /// Registers capability features
    fn register_capability_features(&self, registry: &mut FeatureRegistry) {
        let capability = &self.capability_info;

        // SIMD features
        for simd_feature in &capability.simd_features {
            registry.set_capability_feature(simd_feature.clone(), true);
            registry.register_feature(FeatureInfo::capability(
                simd_feature,
                format!("SIMD instruction set: {}", simd_feature),
                None,
            ));
        }

        // CPU features
        for cpu_feature in &capability.cpu_features {
            registry.set_capability_feature(cpu_feature.clone(), true);
            registry.register_feature(FeatureInfo::capability(
                cpu_feature,
                format!("CPU feature: {}", cpu_feature),
                None,
            ));
        }

        // Core count features
        if let Some(cores) = capability.cores {
            if cores >= 2 {
                registry.set_capability_feature("multicore".to_string(), true);
                registry.register_feature(FeatureInfo::capability(
                    "multicore",
                    format!("Multi-core CPU ({} cores)", cores),
                    None,
                ));
            }

            if cores >= 4 {
                registry.set_capability_feature("quad-core".to_string(), true);
                registry.register_feature(FeatureInfo::capability(
                    "quad-core",
                    format!("Quad-core+ CPU ({} cores)", cores),
                    None,
                ));
            }
        }
    }

    /// Registers runtime capability features
    fn register_runtime_features(&self, registry: &mut FeatureRegistry) {
        let runtime = &self.runtime_capabilities;

        registry.set_capability_feature("nan-boxing".to_string(), runtime.nan_boxing);
        registry.set_capability_feature("simd".to_string(), runtime.simd);
        registry.set_capability_feature("jit".to_string(), runtime.jit);
        registry.set_capability_feature("parallel-gc".to_string(), runtime.parallel_gc);
        registry.set_capability_feature("distributed".to_string(), runtime.distributed);
        registry.set_capability_feature("unsafe-ops".to_string(), runtime.unsafe_ops);
        registry.set_capability_feature("thread-local".to_string(), runtime.thread_local);
        registry.set_capability_feature("atomics".to_string(), runtime.atomics);
    }

    /// Gets platform information
    pub fn platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }

    /// Gets capability information
    pub fn capability_info(&self) -> &CapabilityInfo {
        &self.capability_info
    }

    /// Gets runtime capabilities
    pub fn runtime_capabilities(&self) -> &RuntimeCapabilities {
        &self.runtime_capabilities
    }
}

impl Default for FeatureDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Platform detection helper functions

fn get_target_triple() -> String {
    // In real implementation, this would detect the actual target triple
    // For now, use a common default
    format!(
        "{}-unknown-{}",
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}

fn detect_os_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(unix)]
    features.push("unix".to_string());

    #[cfg(windows)]
    features.push("windows".to_string());

    #[cfg(target_family = "wasm")]
    features.push("wasm".to_string());

    features
}

fn detect_cpu_vendor() -> Option<String> {
    // In real implementation, would use cpuid or similar
    // For now, return None to indicate unavailable
    None
}

fn detect_cpu_brand() -> Option<String> {
    // In real implementation, would read CPU brand string
    None
}

fn detect_physical_cores() -> Option<u32> {
    // Use std::thread::available_parallelism as approximation
    std::thread::available_parallelism()
        .ok()
        .map(|n| n.get() as u32)
}

fn detect_logical_threads() -> Option<u32> {
    // Same as physical cores for now
    detect_physical_cores()
}

fn detect_simd_features() -> Vec<String> {
    let mut features = Vec::new();

    // Detect common SIMD features based on target
    #[cfg(target_arch = "x86_64")]
    {
        // In real implementation, would use cpuid to detect these
        if cfg!(target_feature = "sse2") {
            features.push("sse2".to_string());
        }
        if cfg!(target_feature = "avx") {
            features.push("avx".to_string());
        }
        if cfg!(target_feature = "avx2") {
            features.push("avx2".to_string());
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if cfg!(target_feature = "neon") {
            features.push("neon".to_string());
        }
    }

    features
}

fn detect_cache_sizes() -> HashMap<String, u64> {
    // In real implementation, would detect actual cache sizes
    HashMap::new()
}

fn detect_cpu_features() -> Vec<String> {
    let mut features = Vec::new();

    // Detect common CPU features
    #[cfg(target_arch = "x86_64")]
    {
        if cfg!(target_feature = "fma") {
            features.push("fma".to_string());
        }
        if cfg!(target_feature = "aes") {
            features.push("aes".to_string());
        }
    }

    features
}

fn detect_nan_boxing_support() -> bool {
    // NaN-boxing requires 64-bit pointers and IEEE 754 floats
    std::mem::size_of::<usize>() >= 8
}

fn detect_jit_support() -> bool {
    // JIT requires memory mapping capabilities
    // For now, assume available on desktop platforms
    cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )) && !cfg!(target_family = "wasm")
}

fn detect_parallel_gc_support() -> bool {
    // Parallel GC requires threading support
    detect_physical_cores().is_some_and(|cores| cores > 1)
}

fn detect_distributed_support() -> bool {
    // Distributed computing requires networking
    // For now, assume available on non-wasm platforms
    !cfg!(target_family = "wasm")
}

fn has_thread_local_support() -> bool {
    // Thread-local storage support
    !cfg!(target_family = "wasm")
}

fn has_atomic_support() -> bool {
    // Atomic operations support
    true // Generally available on all modern targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = FeatureDetector::new();

        // Should have detected basic platform info
        assert!(!detector.platform_info.os.is_empty());
        assert!(!detector.platform_info.arch.is_empty());
        assert!(detector.platform_info.pointer_width > 0);
    }

    #[test]
    fn test_platform_detection() {
        let platform = FeatureDetector::detect_platform();

        assert_eq!(platform.os, std::env::consts::OS);
        assert_eq!(platform.arch, std::env::consts::ARCH);
        assert!(platform.pointer_width == 32 || platform.pointer_width == 64);
        assert!(platform.endian == "little" || platform.endian == "big");
    }

    #[test]
    fn test_capability_detection() {
        let capability = FeatureDetector::detect_capabilities();

        // Should detect at least some basic info
        assert!(capability.cores.is_some() || capability.threads.is_some());
    }

    #[test]
    fn test_registry_update() {
        let mut registry = FeatureRegistry::new();
        let detector = FeatureDetector::new();

        detector.update_registry(&mut registry);

        // Should have registered platform features
        assert!(registry.has_feature(&detector.platform_info.os));
        assert!(registry.has_feature(&detector.platform_info.arch));

        // Should have registered runtime capabilities
        let runtime = detector.runtime_capabilities();
        if runtime.simd {
            assert!(registry.has_feature("simd"));
        }
        if runtime.jit {
            assert!(registry.has_feature("jit"));
        }
    }

    #[test]
    fn test_nan_boxing_detection() {
        let support = detect_nan_boxing_support();

        // Should correlate with pointer size
        if std::mem::size_of::<usize>() >= 8 {
            assert!(support);
        }
    }

    #[test]
    fn test_simd_detection() {
        let features = detect_simd_features();

        // Features should be valid strings
        for feature in &features {
            assert!(!feature.is_empty());
        }
    }

    #[test]
    fn test_os_features_detection() {
        let features = detect_os_features();

        // Should detect at least unix or windows
        #[cfg(unix)]
        assert!(features.contains(&"unix".to_string()));

        #[cfg(windows)]
        assert!(features.contains(&"windows".to_string()));
    }
}
