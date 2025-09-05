//! Platform detection and cross-platform analysis for SIGSEGV debugging

use std::collections::HashMap;
use std::fmt;

/// Platform information for crash analysis
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub page_size: usize,
    pub pointer_size: usize,
    pub endianness: String,
    pub features: Vec<String>,
    pub rust_version: String,
    pub compilation_target: String,
    pub debug_symbols: bool,
    pub optimization_level: String,
}

impl fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== PLATFORM INFO ===")?;
        writeln!(f, "OS: {} {}", self.os_name, self.os_version)?;
        writeln!(f, "Architecture: {}", self.architecture)?;
        writeln!(f, "CPU: {} ({} cores)", self.cpu_model, self.cpu_cores)?;
        writeln!(f, "Memory: {} GB", self.total_memory / (1024 * 1024 * 1024))?;
        writeln!(f, "Page Size: {} bytes", self.page_size)?;
        writeln!(f, "Pointer Size: {} bytes", self.pointer_size)?;
        writeln!(f, "Endianness: {}", self.endianness)?;
        writeln!(f, "Rust Version: {}", self.rust_version)?;
        writeln!(f, "Target: {}", self.compilation_target)?;
        writeln!(f, "Debug Symbols: {}", self.debug_symbols)?;
        writeln!(f, "Optimization: {}", self.optimization_level)?;
        
        if !self.features.is_empty() {
            writeln!(f, "CPU Features: {}", self.features.join(", "))?;
        }
        
        Ok(())
    }
}

/// Detect current platform information
pub fn detect_platform() -> PlatformInfo {
    PlatformInfo {
        os_name: detect_os_name(),
        os_version: detect_os_version(),
        architecture: detect_architecture(),
        cpu_model: detect_cpu_model(),
        cpu_cores: detect_cpu_cores(),
        total_memory: detect_total_memory(),
        page_size: detect_page_size(),
        pointer_size: std::mem::size_of::<usize>(),
        endianness: detect_endianness(),
        features: detect_cpu_features(),
        rust_version: detect_rust_version(),
        compilation_target: detect_compilation_target(),
        debug_symbols: detect_debug_symbols(),
        optimization_level: detect_optimization_level(),
    }
}

fn detect_os_name() -> String {
    std::env::consts::OS.to_string()
}

fn detect_os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        detect_macos_version()
    }
    #[cfg(target_os = "linux")]
    {
        detect_linux_version()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

#[cfg(target_os = "macos")]
fn detect_macos_version() -> String {
    use std::process::Command;
    
    if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_version() -> String {
    use std::fs;
    
    // Try to read from /etc/os-release
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let version = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                return version.to_string();
            }
        }
    }
    
    // Fallback to kernel version
    if let Ok(content) = fs::read_to_string("/proc/version") {
        content.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
    } else {
        "unknown".to_string()
    }
}

fn detect_architecture() -> String {
    std::env::consts::ARCH.to_string()
}

fn detect_cpu_model() -> String {
    #[cfg(target_os = "macos")]
    {
        detect_cpu_model_macos()
    }
    #[cfg(target_os = "linux")]
    {
        detect_cpu_model_linux()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

#[cfg(target_os = "macos")]
fn detect_cpu_model_macos() -> String {
    use std::process::Command;
    
    if let Ok(output) = Command::new("sysctl").arg("-n").arg("machdep.cpu.brand_string").output() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(target_os = "linux")]
fn detect_cpu_model_linux() -> String {
    use std::fs;
    
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(model) = line.split(':').nth(1) {
                    return model.trim().to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

fn detect_cpu_cores() -> usize {
    num_cpus::get()
}

fn detect_total_memory() -> u64 {
    #[cfg(target_os = "macos")]
    {
        detect_memory_macos()
    }
    #[cfg(target_os = "linux")]
    {
        detect_memory_linux()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0
    }
}

#[cfg(target_os = "macos")]
fn detect_memory_macos() -> u64 {
    use std::process::Command;
    
    if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let size_str = output_str.trim();
        size_str.parse().unwrap_or(0)
    } else {
        0
    }
}

#[cfg(target_os = "linux")]
fn detect_memory_linux() -> u64 {
    use std::fs;
    
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024; // Convert from KB to bytes
                    }
                }
            }
        }
    }
    0
}

fn detect_page_size() -> usize {
    #[cfg(not(windows))]
    {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
    #[cfg(windows)]
    {
        // Default Windows page size is 4KB
        4096
    }
}

fn detect_endianness() -> String {
    if cfg!(target_endian = "little") {
        "little".to_string()
    } else {
        "big".to_string()
    }
}

fn detect_cpu_features() -> Vec<String> {
    let mut features = Vec::new();
    
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse") { features.push("SSE".to_string()); }
        if is_x86_feature_detected!("sse2") { features.push("SSE2".to_string()); }
        if is_x86_feature_detected!("sse3") { features.push("SSE3".to_string()); }
        if is_x86_feature_detected!("ssse3") { features.push("SSSE3".to_string()); }
        if is_x86_feature_detected!("sse4.1") { features.push("SSE4.1".to_string()); }
        if is_x86_feature_detected!("sse4.2") { features.push("SSE4.2".to_string()); }
        if is_x86_feature_detected!("avx") { features.push("AVX".to_string()); }
        if is_x86_feature_detected!("avx2") { features.push("AVX2".to_string()); }
        if is_x86_feature_detected!("fma") { features.push("FMA".to_string()); }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM features would be detected here if available in std
        features.push("ARM64".to_string());
        // Note: Rust std doesn't provide ARM feature detection like x86
    }
    
    features
}

fn detect_rust_version() -> String {
    option_env!("CARGO_PKG_RUST_VERSION")
        .unwrap_or("unknown")
        .to_string()
}

fn detect_compilation_target() -> String {
    std::env::consts::ARCH.to_string() + "-" + std::env::consts::OS
}

fn detect_debug_symbols() -> bool {
    cfg!(debug_assertions)
}

fn detect_optimization_level() -> String {
    if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "release".to_string()
    }
}

/// Compare two platform infos to identify differences
pub fn compare_platforms(platform1: &PlatformInfo, platform2: &PlatformInfo) -> PlatformComparison {
    let mut differences = Vec::new();
    
    if platform1.os_name != platform2.os_name {
        differences.push(format!("OS: {} vs {}", platform1.os_name, platform2.os_name));
    }
    
    if platform1.architecture != platform2.architecture {
        differences.push(format!("Architecture: {} vs {}", platform1.architecture, platform2.architecture));
    }
    
    if platform1.pointer_size != platform2.pointer_size {
        differences.push(format!("Pointer size: {} vs {}", platform1.pointer_size, platform2.pointer_size));
    }
    
    if platform1.endianness != platform2.endianness {
        differences.push(format!("Endianness: {} vs {}", platform1.endianness, platform2.endianness));
    }
    
    if platform1.page_size != platform2.page_size {
        differences.push(format!("Page size: {} vs {}", platform1.page_size, platform2.page_size));
    }
    
    // Compare CPU features
    let features1: std::collections::HashSet<_> = platform1.features.iter().collect();
    let features2: std::collections::HashSet<_> = platform2.features.iter().collect();
    
    let unique_to_1: Vec<_> = features1.difference(&features2).collect();
    let unique_to_2: Vec<_> = features2.difference(&features1).collect();
    
    if !unique_to_1.is_empty() {
        differences.push(format!("Features unique to platform 1: {:?}", unique_to_1));
    }
    
    if !unique_to_2.is_empty() {
        differences.push(format!("Features unique to platform 2: {:?}", unique_to_2));
    }
    
    let compatibility_risk = calculate_compatibility_risk(&differences);
    PlatformComparison {
        platform1: platform1.clone(),
        platform2: platform2.clone(),
        differences,
        compatibility_risk,
    }
}

#[derive(Debug)]
pub struct PlatformComparison {
    pub platform1: PlatformInfo,
    pub platform2: PlatformInfo,
    pub differences: Vec<String>,
    pub compatibility_risk: CompatibilityRisk,
}

#[derive(Debug, PartialEq)]
pub enum CompatibilityRisk {
    Low,
    Medium,
    High,
}

fn calculate_compatibility_risk(differences: &[String]) -> CompatibilityRisk {
    let mut risk_score = 0;
    
    for diff in differences {
        if diff.contains("OS:") || diff.contains("Architecture:") {
            risk_score += 3; // High risk for OS/arch differences
        } else if diff.contains("Pointer size:") || diff.contains("Endianness:") {
            risk_score += 2; // Medium-high risk for fundamental differences
        } else if diff.contains("Features") {
            risk_score += 1; // Lower risk for feature differences
        }
    }
    
    match risk_score {
        0..=1 => CompatibilityRisk::Low,
        2..=3 => CompatibilityRisk::Medium,
        _ => CompatibilityRisk::High,
    }
}

impl fmt::Display for PlatformComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== PLATFORM COMPARISON ===")?;
        writeln!(f, "Compatibility Risk: {:?}", self.compatibility_risk)?;
        writeln!(f, "\nDifferences:")?;
        
        if self.differences.is_empty() {
            writeln!(f, "  No significant differences detected")?;
        } else {
            for diff in &self.differences {
                writeln!(f, "  - {}", diff)?;
            }
        }
        
        writeln!(f, "\n--- Platform 1 ---")?;
        write!(f, "{}", self.platform1)?;
        
        writeln!(f, "\n--- Platform 2 ---")?;
        write!(f, "{}", self.platform2)?;
        
        Ok(())
    }
}

/// Get environment-specific information
pub fn get_environment_info() -> HashMap<String, String> {
    let mut env_info = HashMap::new();
    
    // Rust-specific environment variables
    if let Ok(val) = std::env::var("CARGO_CFG_TARGET_ARCH") {
        env_info.insert("CARGO_CFG_TARGET_ARCH".to_string(), val);
    }
    if let Ok(val) = std::env::var("CARGO_CFG_TARGET_OS") {
        env_info.insert("CARGO_CFG_TARGET_OS".to_string(), val);
    }
    if let Ok(val) = std::env::var("CARGO_CFG_TARGET_FEATURE") {
        env_info.insert("CARGO_CFG_TARGET_FEATURE".to_string(), val);
    }
    
    // CI environment detection
    if let Ok(val) = std::env::var("GITHUB_ACTIONS") {
        env_info.insert("CI_PROVIDER".to_string(), format!("GitHub Actions ({})", val));
    }
    if let Ok(val) = std::env::var("RUNNER_OS") {
        env_info.insert("CI_RUNNER_OS".to_string(), val);
    }
    if let Ok(val) = std::env::var("RUNNER_ARCH") {
        env_info.insert("CI_RUNNER_ARCH".to_string(), val);
    }
    
    // Memory and performance related
    if let Ok(val) = std::env::var("RUST_MIN_STACK") {
        env_info.insert("RUST_MIN_STACK".to_string(), val);
    }
    if let Ok(val) = std::env::var("RUST_BACKTRACE") {
        env_info.insert("RUST_BACKTRACE".to_string(), val);
    }
    
    env_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        assert!(!platform.os_name.is_empty());
        assert!(!platform.architecture.is_empty());
        assert!(platform.cpu_cores > 0);
        assert!(platform.pointer_size > 0);
    }

    #[test]
    fn test_platform_comparison() {
        let platform1 = PlatformInfo {
            os_name: "macos".to_string(),
            architecture: "aarch64".to_string(),
            pointer_size: 8,
            // ... other fields with default values for testing
            os_version: "14.0".to_string(),
            cpu_model: "Apple M1".to_string(),
            cpu_cores: 8,
            total_memory: 16_000_000_000,
            page_size: 4096,
            endianness: "little".to_string(),
            features: vec!["ARM64".to_string()],
            rust_version: "1.82.0".to_string(),
            compilation_target: "aarch64-macos".to_string(),
            debug_symbols: true,
            optimization_level: "debug".to_string(),
        };

        let platform2 = PlatformInfo {
            os_name: "linux".to_string(),
            architecture: "x86_64".to_string(),
            pointer_size: 8,
            // ... other fields
            os_version: "Ubuntu 22.04".to_string(),
            cpu_model: "Intel Core i7".to_string(),
            cpu_cores: 4,
            total_memory: 8_000_000_000,
            page_size: 4096,
            endianness: "little".to_string(),
            features: vec!["SSE".to_string(), "AVX".to_string()],
            rust_version: "1.82.0".to_string(),
            compilation_target: "x86_64-linux".to_string(),
            debug_symbols: true,
            optimization_level: "debug".to_string(),
        };

        let comparison = compare_platforms(&platform1, &platform2);
        assert_eq!(comparison.compatibility_risk, CompatibilityRisk::High);
        assert!(!comparison.differences.is_empty());
    }
}