//! Crash pattern analysis and automated debugging assistance

use std::collections::{HashMap, HashSet};
use std::fmt;
use crate::debug::CrashInfo;

/// Patterns identified in crashes
#[derive(Debug, Clone, PartialEq)]
pub enum CrashPattern {
    /// Null pointer dereference
    NullPointerDereference {
        location: String,
        frequency: usize,
    },
    /// Use after free
    UseAfterFree {
        location: String,
        frequency: usize,
        allocation_location: Option<String>,
    },
    /// Buffer overflow
    BufferOverflow {
        location: String,
        frequency: usize,
        buffer_info: Option<String>,
    },
    /// Stack overflow
    StackOverflow {
        location: String,
        frequency: usize,
        stack_depth: Option<usize>,
    },
    /// Double free
    DoubleFree {
        location: String,
        frequency: usize,
    },
    /// Unaligned memory access
    UnalignedAccess {
        location: String,
        frequency: usize,
        address: Option<usize>,
        alignment: Option<usize>,
    },
    /// Platform-specific crash
    PlatformSpecific {
        platform: String,
        pattern: String,
        frequency: usize,
    },
    /// SIMD instruction crash
    SimdCrash {
        location: String,
        instruction: Option<String>,
        frequency: usize,
    },
    /// Unsafe code crash
    UnsafeCode {
        location: String,
        frequency: usize,
        unsafe_operation: String,
    },
}

/// Crash analyzer that identifies patterns and suggests fixes
pub struct CrashAnalyzer {
    crashes: Vec<CrashInfo>,
    patterns: Vec<CrashPattern>,
    known_issues: HashMap<String, String>,
}

impl CrashAnalyzer {
    pub fn new() -> Self {
        Self {
            crashes: Vec::new(),
            patterns: Vec::new(),
            known_issues: Self::initialize_known_issues(),
        }
    }

    /// Add a crash to the analyzer
    pub fn add_crash(&mut self, crash: CrashInfo) {
        self.crashes.push(crash);
        self.analyze_patterns();
    }

    /// Add multiple crashes at once
    pub fn add_crashes(&mut self, crashes: Vec<CrashInfo>) {
        self.crashes.extend(crashes);
        self.analyze_patterns();
    }

    /// Analyze all crashes and identify patterns
    pub fn analyze_patterns(&mut self) {
        self.patterns.clear();
        
        // Group crashes by location and signal
        let mut location_groups: HashMap<String, Vec<CrashInfo>> = HashMap::new();
        let mut signal_groups: HashMap<i32, Vec<CrashInfo>> = HashMap::new();
        
        for crash in &self.crashes {
            // Extract key location from backtrace
            let key_location = self.extract_key_location(&crash.backtrace);
            location_groups.entry(key_location).or_default().push(crash.clone());
            signal_groups.entry(crash.signal).or_default().push(crash.clone());
        }

        // Analyze SIGSEGV crashes
        if let Some(sigsegv_crashes) = signal_groups.get(&libc::SIGSEGV) {
            self.analyze_sigsegv_patterns(sigsegv_crashes);
        }

        // Analyze SIGABRT crashes
        if let Some(sigabrt_crashes) = signal_groups.get(&libc::SIGABRT) {
            self.analyze_sigabrt_patterns(sigabrt_crashes);
        }

        // Analyze location-based patterns
        for (location, crashes) in location_groups {
            if crashes.len() > 1 {
                self.analyze_location_patterns(&location, &crashes);
            }
        }

        // Analyze platform-specific patterns
        self.analyze_platform_patterns();
    }

    fn analyze_sigsegv_patterns(&mut self, crashes: &[CrashInfo]) {
        for crash in crashes {
            let key_location = self.extract_key_location(&crash.backtrace);
            
            // Check for null pointer dereference
            if let Some(fault_addr) = crash.fault_address {
                if fault_addr == 0 || fault_addr < 0x1000 {
                    self.add_or_update_pattern(CrashPattern::NullPointerDereference {
                        location: key_location.clone(),
                        frequency: 1,
                    });
                }
                
                // Check for unaligned access (common on ARM)
                if fault_addr % std::mem::size_of::<usize>() != 0 {
                    self.add_or_update_pattern(CrashPattern::UnalignedAccess {
                        location: key_location.clone(),
                        frequency: 1,
                        address: Some(fault_addr),
                        alignment: Some(std::mem::size_of::<usize>()),
                    });
                }
            }
            
            // Analyze backtrace for unsafe code patterns
            for frame in &crash.backtrace {
                if frame.contains("unsafe") || self.contains_unsafe_operations(frame) {
                    let unsafe_op = self.identify_unsafe_operation(frame);
                    self.add_or_update_pattern(CrashPattern::UnsafeCode {
                        location: frame.clone(),
                        frequency: 1,
                        unsafe_operation: unsafe_op,
                    });
                }
                
                // Check for SIMD-related crashes
                if self.is_simd_related(frame) {
                    let instruction = self.extract_simd_instruction(frame);
                    self.add_or_update_pattern(CrashPattern::SimdCrash {
                        location: frame.clone(),
                        instruction,
                        frequency: 1,
                    });
                }
            }
        }
    }

    fn analyze_sigabrt_patterns(&mut self, crashes: &[CrashInfo]) {
        for crash in crashes {
            let key_location = self.extract_key_location(&crash.backtrace);
            
            // SIGABRT often indicates double free or heap corruption
            for frame in &crash.backtrace {
                if frame.contains("free") || frame.contains("deallocate") {
                    self.add_or_update_pattern(CrashPattern::DoubleFree {
                        location: key_location.clone(),
                        frequency: 1,
                    });
                }
            }
        }
    }

    fn analyze_location_patterns(&mut self, location: &str, crashes: &[CrashInfo]) {
        // Look for patterns in repeated crashes at the same location
        let frequency = crashes.len();
        
        // Check if this is a known problematic area
        if location.contains("optimized_value") {
            self.add_or_update_pattern(CrashPattern::UnsafeCode {
                location: location.to_string(),
                frequency,
                unsafe_operation: "optimized value manipulation".to_string(),
            });
        }
        
        if location.contains("ordered_set") || location.contains("red_black") {
            self.add_or_update_pattern(CrashPattern::UseAfterFree {
                location: location.to_string(),
                frequency,
                allocation_location: None,
            });
        }
        
        if location.contains("simd") {
            self.add_or_update_pattern(CrashPattern::SimdCrash {
                location: location.to_string(),
                instruction: None,
                frequency,
            });
        }
    }

    fn analyze_platform_patterns(&mut self) {
        let mut platform_crashes: HashMap<String, Vec<CrashInfo>> = HashMap::new();
        
        for crash in &self.crashes {
            platform_crashes.entry(crash.platform_info.clone()).or_default().push(crash.clone());
        }

        // Look for platform-specific issues
        for (platform, crashes) in platform_crashes {
            if !crashes.is_empty() {
                // Check for Apple Silicon vs x86_64 differences
                if platform.contains("aarch64") {
                    // ARM-specific patterns
                    for crash in &crashes {
                        if crash.signal == libc::SIGSEGV {
                            let key_location = self.extract_key_location(&crash.backtrace);
                            self.add_or_update_pattern(CrashPattern::PlatformSpecific {
                                platform: "aarch64".to_string(),
                                pattern: format!("ARM64 memory alignment issue at {}", key_location),
                                frequency: 1,
                            });
                        }
                    }
                }
                
                if platform.contains("x86_64") && platform.contains("linux") {
                    // Linux x86_64 CI-specific patterns
                    for crash in &crashes {
                        let key_location = self.extract_key_location(&crash.backtrace);
                        self.add_or_update_pattern(CrashPattern::PlatformSpecific {
                            platform: "x86_64-linux".to_string(),
                            pattern: format!("CI environment issue at {}", key_location),
                            frequency: 1,
                        });
                    }
                }
            }
        }
    }

    fn add_or_update_pattern(&mut self, new_pattern: CrashPattern) {
        // Try to find existing pattern and merge
        let mut pattern_found = false;
        for pattern in &mut self.patterns {
            if Self::patterns_match_static(pattern, &new_pattern) {
                Self::merge_patterns_static(pattern, &new_pattern);
                pattern_found = true;
                break;
            }
        }
        
        if !pattern_found {
            // No existing pattern found, add new one
            self.patterns.push(new_pattern);
        }
    }

    fn patterns_match_static(p1: &CrashPattern, p2: &CrashPattern) -> bool {
        use CrashPattern::*;
        
        match (p1, p2) {
            (NullPointerDereference { location: l1, .. }, NullPointerDereference { location: l2, .. }) => l1 == l2,
            (UseAfterFree { location: l1, .. }, UseAfterFree { location: l2, .. }) => l1 == l2,
            (BufferOverflow { location: l1, .. }, BufferOverflow { location: l2, .. }) => l1 == l2,
            (StackOverflow { location: l1, .. }, StackOverflow { location: l2, .. }) => l1 == l2,
            (DoubleFree { location: l1, .. }, DoubleFree { location: l2, .. }) => l1 == l2,
            (UnalignedAccess { location: l1, .. }, UnalignedAccess { location: l2, .. }) => l1 == l2,
            (SimdCrash { location: l1, .. }, SimdCrash { location: l2, .. }) => l1 == l2,
            (UnsafeCode { location: l1, .. }, UnsafeCode { location: l2, .. }) => l1 == l2,
            (PlatformSpecific { platform: p1, pattern: pt1, .. }, 
             PlatformSpecific { platform: p2, pattern: pt2, .. }) => p1 == p2 && pt1 == pt2,
            _ => false,
        }
    }

    fn merge_patterns_static(existing: &mut CrashPattern, new: &CrashPattern) {
        use CrashPattern::*;
        
        match (existing, new) {
            (NullPointerDereference { frequency: ref mut f1, .. }, NullPointerDereference { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (UseAfterFree { frequency: ref mut f1, .. }, UseAfterFree { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (BufferOverflow { frequency: ref mut f1, .. }, BufferOverflow { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (StackOverflow { frequency: ref mut f1, .. }, StackOverflow { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (DoubleFree { frequency: ref mut f1, .. }, DoubleFree { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (UnalignedAccess { frequency: ref mut f1, .. }, UnalignedAccess { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (SimdCrash { frequency: ref mut f1, .. }, SimdCrash { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (UnsafeCode { frequency: ref mut f1, .. }, UnsafeCode { frequency: f2, .. }) => {
                *f1 += f2;
            }
            (PlatformSpecific { frequency: ref mut f1, .. }, PlatformSpecific { frequency: f2, .. }) => {
                *f1 += f2;
            }
            _ => {}
        }
    }

    fn extract_key_location(&self, backtrace: &[String]) -> String {
        // Find the first frame that's in our codebase
        for frame in backtrace {
            if frame.contains("lambdust::") || frame.contains("src/") {
                // Extract just the function/file part
                if let Some(start) = frame.find("lambdust::") {
                    if let Some(end) = frame[start..].find(' ') {
                        return frame[start..start + end].to_string();
                    }
                }
                return frame.clone();
            }
        }
        
        // Fallback to first frame
        backtrace.first().unwrap_or(&"unknown".to_string()).clone()
    }

    fn contains_unsafe_operations(&self, frame: &str) -> bool {
        let unsafe_keywords = [
            "from_raw", "as_ptr", "as_mut_ptr", "offset", "add", "sub",
            "read", "write", "copy", "transmute", "union", "deref_mut",
        ];
        
        unsafe_keywords.iter().any(|&keyword| frame.contains(keyword))
    }

    fn identify_unsafe_operation(&self, frame: &str) -> String {
        if frame.contains("from_raw") { "raw pointer conversion".to_string() }
        else if frame.contains("transmute") { "memory transmutation".to_string() }
        else if frame.contains("offset") || frame.contains("add") || frame.contains("sub") { "pointer arithmetic".to_string() }
        else if frame.contains("read") || frame.contains("write") { "raw memory access".to_string() }
        else if frame.contains("union") { "union field access".to_string() }
        else { "unknown unsafe operation".to_string() }
    }

    fn is_simd_related(&self, frame: &str) -> bool {
        let simd_keywords = ["simd", "avx", "sse", "neon", "vector", "_mm", "_m128", "_m256"];
        simd_keywords.iter().any(|&keyword| frame.to_lowercase().contains(keyword))
    }

    fn extract_simd_instruction(&self, frame: &str) -> Option<String> {
        // Try to extract SIMD instruction names
        let simd_instructions = [
            "_mm_load_ps", "_mm_store_ps", "_mm_add_ps", "_mm_mul_ps",
            "_mm256_load_ps", "_mm256_store_ps", "_mm256_add_ps",
            "vld1q_f32", "vst1q_f32", "vaddq_f32", "vmulq_f32",
        ];
        
        for &instruction in &simd_instructions {
            if frame.contains(instruction) {
                return Some(instruction.to_string());
            }
        }
        
        None
    }

    fn initialize_known_issues() -> HashMap<String, String> {
        let mut issues = HashMap::new();
        
        // Known issues in the codebase
        issues.insert(
            "optimized_value".to_string(),
            "OptimizedValue uses unsafe pointer operations that may cause SIGSEGV on different platforms".to_string()
        );
        
        issues.insert(
            "ordered_set".to_string(),
            "Red-black tree implementation may have use-after-free issues".to_string()
        );
        
        issues.insert(
            "simd_optimization".to_string(),
            "SIMD code may behave differently on Apple Silicon vs x86_64".to_string()
        );
        
        issues
    }

    /// Generate suggestions for fixing identified patterns
    pub fn generate_suggestions(&self) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();
        
        for pattern in &self.patterns {
            let suggestion = match pattern {
                CrashPattern::NullPointerDereference { location, frequency } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::High,
                        description: format!("Add null pointer checks before dereferencing at {}", location),
                        suggested_fix: "Add explicit null checks or use Option<> types".to_string(),
                        code_example: Some("if ptr.is_null() { return Err(\"Null pointer\"); }".to_string()),
                    }
                }
                CrashPattern::UseAfterFree { location, frequency, .. } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::Critical,
                        description: format!("Memory is being accessed after being freed at {}", location),
                        suggested_fix: "Use Rust's ownership system or smart pointers (Arc, Rc)".to_string(),
                        code_example: Some("let data = Arc::new(data); // Share ownership".to_string()),
                    }
                }
                CrashPattern::UnalignedAccess { location, .. } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::Medium,
                        description: format!("Unaligned memory access at {}", location),
                        suggested_fix: "Ensure proper memory alignment, especially for SIMD operations".to_string(),
                        code_example: Some("#[repr(align(16))] struct AlignedData { ... }".to_string()),
                    }
                }
                CrashPattern::PlatformSpecific { platform, pattern: pat, .. } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::Medium,
                        description: format!("Platform-specific issue on {}: {}", platform, pat),
                        suggested_fix: "Add platform-specific conditional compilation".to_string(),
                        code_example: Some("#[cfg(target_arch = \"aarch64\")] // ARM-specific code".to_string()),
                    }
                }
                CrashPattern::SimdCrash { location, .. } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::High,
                        description: format!("SIMD operation crash at {}", location),
                        suggested_fix: "Add runtime CPU feature detection and fallback code".to_string(),
                        code_example: Some("if is_x86_feature_detected!(\"avx2\") { /* AVX2 code */ } else { /* fallback */ }".to_string()),
                    }
                }
                CrashPattern::UnsafeCode { location, unsafe_operation, .. } => {
                    FixSuggestion {
                        pattern: pattern.clone(),
                        priority: Priority::High,
                        description: format!("Unsafe {} operation at {}", unsafe_operation, location),
                        suggested_fix: "Review unsafe code for memory safety violations".to_string(),
                        code_example: Some("// Add bounds checking and null pointer validation".to_string()),
                    }
                }
                _ => continue,
            };
            
            suggestions.push(suggestion);
        }
        
        // Sort by priority
        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));
        
        suggestions
    }

    /// Get all identified patterns
    pub fn get_patterns(&self) -> &[CrashPattern] {
        &self.patterns
    }

    /// Get crash statistics
    pub fn get_statistics(&self) -> CrashStatistics {
        let mut stats = CrashStatistics {
            total_crashes: self.crashes.len(),
            signals: HashMap::new(),
            platforms: HashMap::new(),
            most_common_location: None,
            pattern_count: self.patterns.len(),
        };

        // Count by signal
        for crash in &self.crashes {
            *stats.signals.entry(crash.signal).or_insert(0) += 1;
        }

        // Count by platform
        for crash in &self.crashes {
            *stats.platforms.entry(crash.platform_info.clone()).or_insert(0) += 1;
        }

        // Find most common crash location
        let mut location_counts: HashMap<String, usize> = HashMap::new();
        for crash in &self.crashes {
            let location = self.extract_key_location(&crash.backtrace);
            *location_counts.entry(location).or_insert(0) += 1;
        }

        if let Some((location, count)) = location_counts.iter().max_by_key(|(_, &count)| count) {
            stats.most_common_location = Some((location.clone(), *count));
        }

        stats
    }
}

#[derive(Debug)]
pub struct FixSuggestion {
    pub pattern: CrashPattern,
    pub priority: Priority,
    pub description: String,
    pub suggested_fix: String,
    pub code_example: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct CrashStatistics {
    pub total_crashes: usize,
    pub signals: HashMap<i32, usize>,
    pub platforms: HashMap<String, usize>,
    pub most_common_location: Option<(String, usize)>,
    pub pattern_count: usize,
}

impl fmt::Display for CrashPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrashPattern::NullPointerDereference { location, frequency } => {
                write!(f, "Null pointer dereference at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::UseAfterFree { location, frequency, .. } => {
                write!(f, "Use after free at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::BufferOverflow { location, frequency, .. } => {
                write!(f, "Buffer overflow at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::StackOverflow { location, frequency, .. } => {
                write!(f, "Stack overflow at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::DoubleFree { location, frequency } => {
                write!(f, "Double free at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::UnalignedAccess { location, frequency, .. } => {
                write!(f, "Unaligned memory access at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::PlatformSpecific { platform, pattern, frequency } => {
                write!(f, "Platform-specific issue on {}: {} ({} occurrences)", platform, pattern, frequency)
            }
            CrashPattern::SimdCrash { location, frequency, .. } => {
                write!(f, "SIMD operation crash at {} ({} occurrences)", location, frequency)
            }
            CrashPattern::UnsafeCode { location, unsafe_operation, frequency } => {
                write!(f, "Unsafe {} at {} ({} occurrences)", unsafe_operation, location, frequency)
            }
        }
    }
}

impl fmt::Display for FixSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{:?}] {}", self.priority, self.description)?;
        writeln!(f, "  Fix: {}", self.suggested_fix)?;
        if let Some(ref example) = self.code_example {
            writeln!(f, "  Example: {}", example)?;
        }
        Ok(())
    }
}

/// Convenience function to analyze crashes
pub fn analyze_crashes(crashes: Vec<CrashInfo>) -> CrashAnalyzer {
    let mut analyzer = CrashAnalyzer::new();
    analyzer.add_crashes(crashes);
    analyzer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_crash_pattern_analysis() {
        let mut analyzer = CrashAnalyzer::new();
        
        let crash = CrashInfo {
            signal: libc::SIGSEGV,
            signal_name: "SIGSEGV",
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            process_id: 1234,
            thread_id: 5678,
            fault_address: Some(0), // Null pointer
            instruction_pointer: None,
            stack_pointer: None,
            backtrace: vec![
                "lambdust::eval::optimized_value::deref+0x10".to_string(),
                "main+0x20".to_string(),
            ],
            register_dump: HashMap::new(),
            memory_info: crate::debug::signal_handler::MemoryInfo {
                heap_size: 1024,
                stack_size: 512,
                virtual_memory: 2048,
                physical_memory: 1536,
            },
            platform_info: "test-platform".to_string(),
            test_context: None,
        };

        analyzer.add_crash(crash);
        
        let patterns = analyzer.get_patterns();
        assert!(!patterns.is_empty());
        
        // Should detect null pointer dereference
        assert!(patterns.iter().any(|p| matches!(p, CrashPattern::NullPointerDereference { .. })));
    }
}