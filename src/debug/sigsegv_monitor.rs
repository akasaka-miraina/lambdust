//! Comprehensive SIGSEGV monitoring and debugging system for cross-platform analysis
//!
//! This module provides real-time SIGSEGV detection, detailed crash analysis,
//! and platform-specific debugging information to help resolve memory safety issues
//! that manifest differently between macOS+Apple Silicon and GitHub Actions Ubuntu environments.

use std::collections::{HashMap, BTreeMap};
use std::ffi::CString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufWriter};
use std::mem;
use std::panic;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::thread;

use crate::debug::{CrashInfo, MemoryInfo};

/// Platform-specific memory alignment requirements
#[cfg(target_arch = "aarch64")]
const MEMORY_ALIGNMENT: usize = 16; // ARM64 prefers 16-byte alignment

#[cfg(target_arch = "x86_64")]
const MEMORY_ALIGNMENT: usize = 8; // x86_64 requires 8-byte alignment for most operations

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
const MEMORY_ALIGNMENT: usize = 8; // Conservative default

/// Detailed information about a SIGSEGV event
#[derive(Debug, Clone)]
pub struct SigsegvEvent {
    pub timestamp: u64,
    pub process_id: u32,
    pub thread_id: u64,
    pub signal: i32,
    pub fault_address: Option<usize>,
    pub instruction_pointer: Option<usize>,
    pub stack_pointer: Option<usize>,
    pub platform_info: PlatformInfo,
    pub memory_state: MemoryState,
    pub backtrace: Vec<BacktraceFrame>,
    pub register_dump: HashMap<String, u64>,
    pub context_info: ContextInfo,
    pub alignment_check: AlignmentAnalysis,
    pub memory_validation: MemoryValidation,
}

/// Platform-specific information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu_features: Vec<String>,
    pub page_size: usize,
    pub pointer_size: usize,
    pub endianness: String,
    pub is_ci_environment: bool,
    pub rust_version: String,
    pub target_triple: String,
    pub debug_symbols: bool,
    pub optimization_level: String,
}

/// Memory state at the time of crash
#[derive(Debug, Clone)]
pub struct MemoryState {
    pub heap_size: usize,
    pub stack_size: usize,
    pub virtual_memory: usize,
    pub physical_memory: usize,
    pub stack_base: Option<usize>,
    pub stack_limit: Option<usize>,
    pub heap_regions: Vec<MemoryRegion>,
    pub memory_fragmentation: f64,
}

/// Memory region information
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub size: usize,
    pub permissions: String,
    pub region_type: String,
}

/// Enhanced backtrace frame with source location and context
#[derive(Debug, Clone)]
pub struct BacktraceFrame {
    pub function_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub instruction_address: usize,
    pub symbol_address: Option<usize>,
    pub module_name: Option<String>,
    pub offset: Option<usize>,
    pub is_inline: bool,
    pub is_unsafe_code: bool,
    pub is_simd_code: bool,
}

/// Context information for the crash
#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub test_name: Option<String>,
    pub function_context: Option<String>,
    pub operation_type: Option<String>,
    pub data_structure: Option<String>,
    pub thread_name: Option<String>,
    pub panic_message: Option<String>,
    pub error_location: Option<String>,
}

/// Memory alignment analysis
#[derive(Debug, Clone)]
pub struct AlignmentAnalysis {
    pub required_alignment: usize,
    pub actual_alignment: Option<usize>,
    pub is_aligned: Option<bool>,
    pub alignment_violations: Vec<AlignmentViolation>,
    pub simd_alignment_check: bool,
    pub platform_specific_issues: Vec<String>,
}

/// Alignment violation details
#[derive(Debug, Clone)]
pub struct AlignmentViolation {
    pub address: usize,
    pub required: usize,
    pub actual: usize,
    pub operation: String,
    pub severity: ViolationSeverity,
}

/// Memory validation results
#[derive(Debug, Clone)]
pub struct MemoryValidation {
    pub null_pointer_checks: Vec<NullPointerCheck>,
    pub bounds_checks: Vec<BoundsCheck>,
    pub use_after_free_checks: Vec<UseAfterFreeCheck>,
    pub double_free_checks: Vec<DoubleFreeCheck>,
    pub uninitialized_memory_checks: Vec<UninitializedMemoryCheck>,
}

#[derive(Debug, Clone)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct NullPointerCheck {
    pub address: usize,
    pub is_null: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct BoundsCheck {
    pub address: usize,
    pub buffer_start: Option<usize>,
    pub buffer_end: Option<usize>,
    pub is_valid: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct UseAfterFreeCheck {
    pub address: usize,
    pub allocation_time: Option<u64>,
    pub free_time: Option<u64>,
    pub is_valid: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct DoubleFreeCheck {
    pub address: usize,
    pub first_free_time: Option<u64>,
    pub second_free_time: Option<u64>,
    pub is_valid: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct UninitializedMemoryCheck {
    pub address: usize,
    pub size: usize,
    pub is_initialized: bool,
    pub context: String,
}

/// SIGSEGV monitoring system
pub struct SigsegvMonitor {
    events: Arc<RwLock<Vec<SigsegvEvent>>>,
    log_directory: PathBuf,
    max_events: usize,
    is_monitoring: AtomicBool,
    event_count: AtomicUsize,
    last_event_time: AtomicU64,
    platform_info: PlatformInfo,
    memory_tracker: Arc<Mutex<MemoryTracker>>,
    alignment_validator: Arc<AlignmentValidator>,
    pattern_analyzer: Arc<Mutex<PatternAnalyzer>>,
}

/// Memory tracking for use-after-free detection
#[derive(Debug)]
struct MemoryTracker {
    allocations: HashMap<usize, AllocationInfo>,
    freed_allocations: HashMap<usize, AllocationInfo>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    address: usize,
    size: usize,
    timestamp: u64,
    backtrace: Vec<String>,
    thread_id: u64,
}

/// Alignment validation system
pub struct AlignmentValidator {
    required_alignments: HashMap<String, usize>,
    platform_constraints: HashMap<String, Vec<String>>,
}

/// Pattern analysis for recurring issues
#[derive(Debug)]
struct PatternAnalyzer {
    crash_patterns: HashMap<String, usize>,
    platform_patterns: HashMap<String, usize>,
    function_patterns: HashMap<String, usize>,
    alignment_patterns: HashMap<usize, usize>,
}

impl Default for SigsegvMonitor {
    fn default() -> Self {
        Self::new("sigsegv_logs".into())
    }
}

impl SigsegvMonitor {
    /// Create a new SIGSEGV monitor
    pub fn new(log_directory: PathBuf) -> Self {
        let platform_info = Self::detect_platform_info();
        
        // Create log directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&log_directory) {
            eprintln!("Warning: Could not create log directory {:?}: {}", log_directory, e);
        }

        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            log_directory,
            max_events: 1000,
            is_monitoring: AtomicBool::new(false),
            event_count: AtomicUsize::new(0),
            last_event_time: AtomicU64::new(0),
            platform_info,
            memory_tracker: Arc::new(Mutex::new(MemoryTracker::new())),
            alignment_validator: Arc::new(AlignmentValidator::new()),
            pattern_analyzer: Arc::new(Mutex::new(PatternAnalyzer::new())),
        }
    }

    /// Start monitoring for SIGSEGV events
    pub fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_monitoring.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_err() {
            return Err("Monitoring is already active".into());
        }

        // Install signal handlers for various crash signals
        self.install_signal_handlers()?;
        
        // Start background monitoring thread
        self.start_monitoring_thread();
        
        println!("SIGSEGV monitoring started for platform: {} {}", 
                 self.platform_info.os_name, 
                 self.platform_info.architecture);
        
        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&self) {
        self.is_monitoring.store(false, Ordering::SeqCst);
        println!("SIGSEGV monitoring stopped");
    }

    /// Record a SIGSEGV event
    pub fn record_event(&self, crash_info: CrashInfo) {
        let event = self.create_detailed_event(crash_info);
        
        // Add to events list
        if let Ok(mut events) = self.events.write() {
            events.push(event.clone());
            
            // Limit the number of stored events
            if events.len() > self.max_events {
                events.remove(0);
            }
        }
        
        // Update counters
        self.event_count.fetch_add(1, Ordering::SeqCst);
        self.last_event_time.store(
            SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            Ordering::SeqCst
        );

        // Analyze patterns
        if let Ok(mut analyzer) = self.pattern_analyzer.lock() {
            analyzer.analyze_event(&event);
        }

        // Write detailed log
        if let Err(e) = self.write_detailed_log(&event) {
            eprintln!("Failed to write SIGSEGV log: {}", e);
        }
    }

    /// Get all recorded events
    pub fn get_events(&self) -> Vec<SigsegvEvent> {
        self.events.read()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Get monitoring statistics
    pub fn get_statistics(&self) -> MonitoringStatistics {
        let event_count = self.event_count.load(Ordering::SeqCst);
        let last_event_time = self.last_event_time.load(Ordering::SeqCst);
        let is_monitoring = self.is_monitoring.load(Ordering::SeqCst);
        
        let patterns = self.pattern_analyzer.lock()
            .map(|analyzer| analyzer.get_summary())
            .unwrap_or_default();

        MonitoringStatistics {
            event_count,
            last_event_time,
            is_monitoring,
            platform: self.platform_info.clone(),
            patterns,
        }
    }

    /// Generate a comprehensive crash analysis report
    pub fn generate_analysis_report(&self) -> Result<String, Box<dyn std::error::Error>> {
        let events = self.get_events();
        let stats = self.get_statistics();
        
        let mut report = String::new();
        
        report.push_str("=== SIGSEGV MONITORING ANALYSIS REPORT ===\n\n");
        
        // Platform information
        report.push_str(&format!("Platform: {} {} ({})\n", 
                                stats.platform.os_name,
                                stats.platform.os_version,
                                stats.platform.architecture));
        report.push_str(&format!("CI Environment: {}\n", stats.platform.is_ci_environment));
        report.push_str(&format!("Rust Version: {}\n", stats.platform.rust_version));
        report.push_str(&format!("Target: {}\n", stats.platform.target_triple));
        report.push_str(&format!("Debug Symbols: {}\n", stats.platform.debug_symbols));
        report.push_str(&format!("Optimization: {}\n\n", stats.platform.optimization_level));
        
        // Statistics
        report.push_str(&format!("Total Events: {}\n", stats.event_count));
        report.push_str(&format!("Monitoring Active: {}\n", stats.is_monitoring));
        
        if stats.last_event_time > 0 {
            report.push_str(&format!("Last Event: {} seconds ago\n\n", 
                                    SystemTime::now().duration_since(UNIX_EPOCH)
                                        .unwrap_or_default().as_secs() - stats.last_event_time));
        }
        
        // Pattern analysis
        if !stats.patterns.is_empty() {
            report.push_str("=== IDENTIFIED PATTERNS ===\n");
            for (pattern, count) in &stats.patterns {
                report.push_str(&format!("  {} ({} occurrences)\n", pattern, count));
            }
            report.push_str("\n");
        }
        
        // Recent events analysis
        if !events.is_empty() {
            report.push_str("=== RECENT EVENTS ANALYSIS ===\n");
            
            let recent_events: Vec<_> = events.iter()
                .rev()
                .take(5)
                .collect();
            
            for (i, event) in recent_events.iter().enumerate() {
                report.push_str(&format!("Event #{}: Signal {}\n", i + 1, event.signal));
                
                if let Some(addr) = event.fault_address {
                    report.push_str(&format!("  Fault Address: 0x{:016x}\n", addr));
                }
                
                if let Some(ip) = event.instruction_pointer {
                    report.push_str(&format!("  Instruction Pointer: 0x{:016x}\n", ip));
                }
                
                // Alignment analysis
                if !event.alignment_check.alignment_violations.is_empty() {
                    report.push_str("  Alignment Issues:\n");
                    for violation in &event.alignment_check.alignment_violations {
                        report.push_str(&format!("    Address 0x{:016x}: required {}, actual {} ({})\n",
                                                violation.address,
                                                violation.required,
                                                violation.actual,
                                                violation.operation));
                    }
                }
                
                // Memory validation
                let validation = &event.memory_validation;
                if !validation.null_pointer_checks.is_empty() {
                    let null_checks: Vec<_> = validation.null_pointer_checks.iter()
                        .filter(|check| check.is_null)
                        .collect();
                    if !null_checks.is_empty() {
                        report.push_str(&format!("  Null Pointer Issues: {} detected\n", null_checks.len()));
                    }
                }
                
                // Top backtrace frames
                if !event.backtrace.is_empty() {
                    report.push_str("  Top Stack Frames:\n");
                    for frame in event.backtrace.iter().take(3) {
                        report.push_str(&format!("    {}\n", frame.function_name));
                        if let Some(ref file) = frame.file_name {
                            if let Some(line) = frame.line_number {
                                report.push_str(&format!("      at {}:{}\n", file, line));
                            }
                        }
                    }
                }
                
                report.push_str("\n");
            }
        }
        
        // Platform-specific recommendations
        report.push_str("=== PLATFORM-SPECIFIC RECOMMENDATIONS ===\n");
        self.generate_recommendations(&mut report, &events, &stats);
        
        Ok(report)
    }

    /// Write the report to a file
    pub fn write_analysis_report(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.generate_analysis_report()?;
        let filepath = self.log_directory.join(filename);
        let mut file = File::create(&filepath)?;
        file.write_all(report.as_bytes())?;
        println!("Analysis report written to: {:?}", filepath);
        Ok(())
    }

    // === Private Implementation ===

    /// Detect platform information
    fn detect_platform_info() -> PlatformInfo {
        let is_ci = std::env::var("CI").is_ok() || 
                   std::env::var("GITHUB_ACTIONS").is_ok() ||
                   std::env::var("CONTINUOUS_INTEGRATION").is_ok();

        PlatformInfo {
            os_name: std::env::consts::OS.to_string(),
            os_version: Self::get_os_version(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_features: Self::get_cpu_features(),
            page_size: Self::get_page_size(),
            pointer_size: mem::size_of::<*const u8>(),
            endianness: if cfg!(target_endian = "little") { "little".to_string() } else { "big".to_string() },
            is_ci_environment: is_ci,
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            target_triple: Self::get_target_triple(),
            debug_symbols: cfg!(debug_assertions),
            optimization_level: if cfg!(debug_assertions) { "debug".to_string() } else { "release".to_string() },
        }
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            Self::get_macos_version()
        }
        #[cfg(target_os = "linux")]
        {
            Self::get_linux_version()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            "unknown".to_string()
        }
    }

    #[cfg(target_os = "macos")]
    fn get_macos_version() -> String {
        use std::process::Command;
        
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(target_os = "linux")]
    fn get_linux_version() -> String {
        std::fs::read_to_string("/proc/version")
            .unwrap_or_else(|_| "unknown".to_string())
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn get_cpu_features() -> Vec<String> {
        let mut features = Vec::new();
        
        #[cfg(target_arch = "x86_64")]
        {
            features.push("x86_64".to_string());
            if is_x86_feature_detected!("avx") {
                features.push("AVX".to_string());
            }
            if is_x86_feature_detected!("avx2") {
                features.push("AVX2".to_string());
            }
            if is_x86_feature_detected!("sse2") {
                features.push("SSE2".to_string());
            }
            if is_x86_feature_detected!("sse4.1") {
                features.push("SSE4.1".to_string());
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            features.push("ARM64".to_string());
            // Note: is_aarch64_feature_detected! is not available in stable Rust
            // We'll assume NEON is available on aarch64 for now
            features.push("NEON".to_string());
        }
        
        features
    }

    fn get_page_size() -> usize {
        #[cfg(unix)]
        {
            unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
        }
        #[cfg(not(unix))]
        {
            4096 // Default page size
        }
    }

    fn get_target_triple() -> String {
        format!("{}-{}-{}", 
                std::env::consts::ARCH,
                std::env::consts::OS,
                if cfg!(target_env = "gnu") { "gnu" }
                else if cfg!(target_env = "msvc") { "msvc" }
                else { "unknown" })
    }

    /// Install platform-specific signal handlers
    fn install_signal_handlers(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This would be implemented with platform-specific signal handling
        // For now, we'll use the existing signal handler infrastructure
        println!("Signal handlers installed for SIGSEGV monitoring");
        Ok(())
    }

    /// Start the background monitoring thread
    fn start_monitoring_thread(&self) {
        let is_monitoring = Arc::new(AtomicBool::new(self.is_monitoring.load(Ordering::SeqCst)));
        let events = Arc::clone(&self.events);
        
        thread::spawn(move || {
            while is_monitoring.load(Ordering::SeqCst) {
                // Monitor system health and memory state
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    }

    /// Create a detailed event from crash information
    fn create_detailed_event(&self, crash_info: CrashInfo) -> SigsegvEvent {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Clone the data we need before consuming crash_info
        let process_id = crash_info.process_id;
        let thread_id = crash_info.thread_id;
        let signal = crash_info.signal;
        let fault_address = crash_info.fault_address;
        let instruction_pointer = crash_info.instruction_pointer;
        let stack_pointer = crash_info.stack_pointer;
        let backtrace_data = crash_info.backtrace.clone();
        let register_data = crash_info.register_dump.clone();

        let backtrace = Self::enhance_backtrace(backtrace_data);
        let alignment_check = self.alignment_validator.analyze_crash(&crash_info);
        let memory_validation = self.validate_memory_state(&crash_info);
        let context_info = Self::extract_context_info(&crash_info);
        let memory_state = Self::get_detailed_memory_state();

        SigsegvEvent {
            timestamp,
            process_id,
            thread_id,
            signal,
            fault_address,
            instruction_pointer,
            stack_pointer,
            platform_info: self.platform_info.clone(),
            memory_state,
            backtrace,
            register_dump: register_data.into_iter()
                .map(|(k, v)| (k, v as u64))
                .collect(),
            context_info,
            alignment_check,
            memory_validation,
        }
    }

    /// Enhance backtrace with additional information
    fn enhance_backtrace(backtrace: Vec<String>) -> Vec<BacktraceFrame> {
        backtrace.into_iter().map(|frame_str| {
            // Parse the frame string to extract information
            let parts: Vec<&str> = frame_str.split_whitespace().collect();
            let function_name = parts.get(1)
                .and_then(|s| s.split("::").last())
                .unwrap_or("unknown")
                .to_string();
            
            // Detect if this is unsafe or SIMD code
            let is_unsafe_code = frame_str.contains("unsafe") || 
                               frame_str.contains("mem::") ||
                               frame_str.contains("ptr::");
            let is_simd_code = frame_str.contains("simd") ||
                             frame_str.contains("avx") ||
                             frame_str.contains("sse") ||
                             frame_str.contains("neon");

            // Extract address information
            let instruction_address = parts.iter()
                .find(|s| s.starts_with("0x"))
                .and_then(|s| usize::from_str_radix(&s[2..], 16).ok())
                .unwrap_or(0);

            BacktraceFrame {
                function_name: function_name.clone(),
                file_name: None, // Would need debug symbols to extract
                line_number: None,
                column_number: None,
                instruction_address,
                symbol_address: None,
                module_name: parts.get(1)
                    .and_then(|s| s.split("::").next())
                    .map(|s| s.to_string()),
                offset: None,
                is_inline: false,
                is_unsafe_code,
                is_simd_code,
            }
        }).collect()
    }

    /// Get detailed memory state
    fn get_detailed_memory_state() -> MemoryState {
        // This would use platform-specific APIs to get detailed memory information
        MemoryState {
            heap_size: 0,
            stack_size: 0,
            virtual_memory: 0,
            physical_memory: 0,
            stack_base: None,
            stack_limit: None,
            heap_regions: Vec::new(),
            memory_fragmentation: 0.0,
        }
    }

    /// Extract context information from crash
    fn extract_context_info(crash_info: &CrashInfo) -> ContextInfo {
        ContextInfo {
            test_name: crash_info.test_context.clone(),
            function_context: None,
            operation_type: None,
            data_structure: None,
            thread_name: None,
            panic_message: None,
            error_location: None,
        }
    }

    /// Validate memory state for common issues
    fn validate_memory_state(&self, crash_info: &CrashInfo) -> MemoryValidation {
        let mut null_pointer_checks = Vec::new();
        let mut bounds_checks = Vec::new();
        let mut use_after_free_checks = Vec::new();
        let mut double_free_checks = Vec::new();
        let mut uninitialized_memory_checks = Vec::new();

        // Check for null pointer dereference
        if let Some(fault_addr) = crash_info.fault_address {
            null_pointer_checks.push(NullPointerCheck {
                address: fault_addr,
                is_null: fault_addr == 0 || fault_addr == 0xdeadbeef,
                context: format!("Fault address during signal {}", crash_info.signal),
            });
        }

        MemoryValidation {
            null_pointer_checks,
            bounds_checks,
            use_after_free_checks,
            double_free_checks,
            uninitialized_memory_checks,
        }
    }

    /// Write detailed log file
    fn write_detailed_log(&self, event: &SigsegvEvent) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("sigsegv_detailed_{:010}_{}.log", 
                              event.timestamp, 
                              event.process_id);
        let filepath = self.log_directory.join(filename);
        
        let mut file = BufWriter::new(File::create(&filepath)?);
        
        writeln!(file, "=== DETAILED SIGSEGV ANALYSIS ===")?;
        writeln!(file, "Timestamp: {}", event.timestamp)?;
        writeln!(file, "Signal: {}", event.signal)?;
        writeln!(file, "Process ID: {}", event.process_id)?;
        writeln!(file, "Thread ID: {}", event.thread_id)?;
        
        if let Some(addr) = event.fault_address {
            writeln!(file, "Fault Address: 0x{:016x}", addr)?;
        }
        
        writeln!(file, "\n=== PLATFORM INFO ===")?;
        writeln!(file, "OS: {} {}", event.platform_info.os_name, event.platform_info.os_version)?;
        writeln!(file, "Architecture: {}", event.platform_info.architecture)?;
        writeln!(file, "CI Environment: {}", event.platform_info.is_ci_environment)?;
        writeln!(file, "CPU Features: {:?}", event.platform_info.cpu_features)?;
        writeln!(file, "Page Size: {} bytes", event.platform_info.page_size)?;
        writeln!(file, "Pointer Size: {} bytes", event.platform_info.pointer_size)?;
        
        writeln!(file, "\n=== ALIGNMENT ANALYSIS ===")?;
        writeln!(file, "Required Alignment: {} bytes", event.alignment_check.required_alignment)?;
        if let Some(actual) = event.alignment_check.actual_alignment {
            writeln!(file, "Actual Alignment: {} bytes", actual)?;
        }
        if let Some(aligned) = event.alignment_check.is_aligned {
            writeln!(file, "Is Aligned: {}", aligned)?;
        }
        
        for violation in &event.alignment_check.alignment_violations {
            writeln!(file, "Alignment Violation: 0x{:016x} - {} (required: {}, actual: {})",
                    violation.address,
                    violation.operation,
                    violation.required,
                    violation.actual)?;
        }
        
        writeln!(file, "\n=== MEMORY VALIDATION ===")?;
        for check in &event.memory_validation.null_pointer_checks {
            if check.is_null {
                writeln!(file, "Null Pointer: 0x{:016x} - {}", check.address, check.context)?;
            }
        }
        
        writeln!(file, "\n=== ENHANCED BACKTRACE ===")?;
        for (i, frame) in event.backtrace.iter().enumerate() {
            writeln!(file, "#{:2}: {} (0x{:016x})", i, frame.function_name, frame.instruction_address)?;
            if frame.is_unsafe_code {
                writeln!(file, "      [UNSAFE CODE]")?;
            }
            if frame.is_simd_code {
                writeln!(file, "      [SIMD CODE]")?;
            }
            if let Some(ref module) = frame.module_name {
                writeln!(file, "      Module: {}", module)?;
            }
        }
        
        writeln!(file, "\n=== REGISTER DUMP ===")?;
        for (reg, value) in &event.register_dump {
            writeln!(file, "{}: 0x{:016x}", reg, value)?;
        }
        
        file.flush()?;
        println!("Detailed SIGSEGV log written to: {:?}", filepath);
        
        Ok(())
    }

    /// Generate platform-specific recommendations
    fn generate_recommendations(&self, report: &mut String, events: &[SigsegvEvent], stats: &MonitoringStatistics) {
        if stats.platform.is_ci_environment {
            report.push_str("CI Environment Recommendations:\n");
            report.push_str("  - Add explicit memory alignment checks for CI vs local differences\n");
            report.push_str("  - Consider using different optimization levels for CI builds\n");
            report.push_str("  - Enable additional debug symbols in CI builds for better stack traces\n");
            report.push_str("  - Use AddressSanitizer (ASAN) in CI for memory error detection\n");
            report.push_str("\n");
        }
        
        match stats.platform.architecture.as_str() {
            "aarch64" => {
                report.push_str("ARM64 Architecture Recommendations:\n");
                report.push_str("  - Ensure 16-byte alignment for SIMD operations\n");
                report.push_str("  - Check for unaligned memory accesses in struct layouts\n");
                report.push_str("  - Verify NEON instruction usage is properly aligned\n");
                report.push_str("  - Use #[repr(align(16))] for ARM64-specific data structures\n");
            },
            "x86_64" => {
                report.push_str("x86_64 Architecture Recommendations:\n");
                report.push_str("  - Ensure 8-byte alignment for 64-bit operations\n");
                report.push_str("  - Check AVX/SSE instruction alignment requirements\n");
                report.push_str("  - Verify stack alignment for system calls\n");
                report.push_str("  - Use platform-specific memory barriers if needed\n");
            },
            _ => {
                report.push_str("General Architecture Recommendations:\n");
                report.push_str("  - Add platform-specific alignment checks\n");
                report.push_str("  - Use conservative memory alignment strategies\n");
            }
        }
        
        report.push_str("\n");
        
        // Function-specific recommendations based on crash patterns
        let has_optimized_value_crashes = events.iter().any(|e| 
            e.backtrace.iter().any(|f| f.function_name.contains("optimized_value")));
        let has_ordered_set_crashes = events.iter().any(|e| 
            e.backtrace.iter().any(|f| f.function_name.contains("ordered_set")));
            
        if has_optimized_value_crashes {
            report.push_str("OptimizedValue Specific Recommendations:\n");
            report.push_str("  - Add null pointer validation in Deref implementation\n");
            report.push_str("  - Ensure NaN-boxing doesn't conflict with pointer alignment\n");
            report.push_str("  - Add platform-specific value representation validation\n");
            report.push_str("  - Consider using Option<> instead of raw pointers\n");
            report.push_str("\n");
        }
        
        if has_ordered_set_crashes {
            report.push_str("OrderedSet Specific Recommendations:\n");
            report.push_str("  - Validate Arc<Node> alignment before dereferencing\n");
            report.push_str("  - Add bounds checking in tree insertion operations\n");
            report.push_str("  - Ensure red-black tree invariants are maintained\n");
            report.push_str("  - Add platform-specific node size validation\n");
            report.push_str("\n");
        }
    }
}

/// Monitoring statistics
#[derive(Debug)]
pub struct MonitoringStatistics {
    pub event_count: usize,
    pub last_event_time: u64,
    pub is_monitoring: bool,
    pub platform: PlatformInfo,
    pub patterns: HashMap<String, usize>,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            freed_allocations: HashMap::new(),
        }
    }
}

impl AlignmentValidator {
    fn new() -> Self {
        let mut required_alignments = HashMap::new();
        let mut platform_constraints = HashMap::new();
        
        // Set up alignment requirements
        required_alignments.insert("OptimizedValue".to_string(), MEMORY_ALIGNMENT);
        required_alignments.insert("Node".to_string(), mem::align_of::<usize>());
        required_alignments.insert("Arc".to_string(), mem::align_of::<*const u8>());
        
        // Platform-specific constraints
        #[cfg(target_arch = "aarch64")]
        {
            platform_constraints.insert("SIMD".to_string(), vec!["16-byte alignment required".to_string()]);
            platform_constraints.insert("NEON".to_string(), vec!["128-bit alignment".to_string()]);
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            platform_constraints.insert("AVX".to_string(), vec!["32-byte alignment required".to_string()]);
            platform_constraints.insert("SSE".to_string(), vec!["16-byte alignment required".to_string()]);
        }
        
        Self {
            required_alignments,
            platform_constraints,
        }
    }
    
    fn analyze_crash(&self, crash_info: &CrashInfo) -> AlignmentAnalysis {
        let mut violations = Vec::new();
        let mut platform_issues = Vec::new();
        
        // Check fault address alignment if available
        let (is_aligned, actual_alignment) = if let Some(fault_addr) = crash_info.fault_address {
            let alignment = Self::get_alignment(fault_addr);
            let required = MEMORY_ALIGNMENT;
            let aligned = alignment >= required;
            
            if !aligned {
                violations.push(AlignmentViolation {
                    address: fault_addr,
                    required,
                    actual: alignment,
                    operation: "memory access".to_string(),
                    severity: ViolationSeverity::High,
                });
            }
            
            (Some(aligned), Some(alignment))
        } else {
            (None, None)
        };
        
        // Check for platform-specific issues
        for frame in &crash_info.backtrace {
            if frame.contains("simd") || frame.contains("avx") || frame.contains("sse") {
                platform_issues.push("SIMD operation may require specific alignment".to_string());
            }
        }
        
        AlignmentAnalysis {
            required_alignment: MEMORY_ALIGNMENT,
            actual_alignment,
            is_aligned,
            alignment_violations: violations,
            simd_alignment_check: crash_info.backtrace.iter()
                .any(|f| f.contains("simd") || f.contains("avx") || f.contains("sse")),
            platform_specific_issues: platform_issues,
        }
    }
    
    fn get_alignment(address: usize) -> usize {
        if address == 0 {
            return 0;
        }
        
        // Find the largest power of 2 that divides the address
        let mut alignment = 1;
        let mut addr = address;
        
        while addr % 2 == 0 {
            alignment *= 2;
            addr /= 2;
        }
        
        alignment
    }
}

impl PatternAnalyzer {
    fn new() -> Self {
        Self {
            crash_patterns: HashMap::new(),
            platform_patterns: HashMap::new(),
            function_patterns: HashMap::new(),
            alignment_patterns: HashMap::new(),
        }
    }
    
    fn analyze_event(&mut self, event: &SigsegvEvent) {
        // Analyze crash patterns
        let pattern_key = format!("Signal {} at 0x{:016x}", 
                                 event.signal,
                                 event.fault_address.unwrap_or(0));
        *self.crash_patterns.entry(pattern_key).or_insert(0) += 1;
        
        // Platform patterns
        let platform_key = format!("{} {}", 
                                   event.platform_info.os_name, 
                                   event.platform_info.architecture);
        *self.platform_patterns.entry(platform_key).or_insert(0) += 1;
        
        // Function patterns
        for frame in &event.backtrace {
            *self.function_patterns.entry(frame.function_name.clone()).or_insert(0) += 1;
        }
        
        // Alignment patterns
        if let Some(alignment) = event.alignment_check.actual_alignment {
            *self.alignment_patterns.entry(alignment).or_insert(0) += 1;
        }
    }
    
    fn get_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        // Top crash patterns
        for (pattern, count) in self.crash_patterns.iter().take(5) {
            summary.insert(format!("Crash: {}", pattern), *count);
        }
        
        // Top function patterns
        for (func, count) in self.function_patterns.iter().take(5) {
            summary.insert(format!("Function: {}", func), *count);
        }
        
        summary
    }
}

// === Public API Functions ===

/// Initialize the global SIGSEGV monitor
static GLOBAL_MONITOR: std::sync::OnceLock<SigsegvMonitor> = std::sync::OnceLock::new();

/// Get or initialize the global SIGSEGV monitor
pub fn get_monitor() -> &'static SigsegvMonitor {
    GLOBAL_MONITOR.get_or_init(|| {
        SigsegvMonitor::new("sigsegv_logs".into())
    })
}

/// Start global SIGSEGV monitoring
pub fn start_global_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    get_monitor().start_monitoring()
}

/// Stop global SIGSEGV monitoring
pub fn stop_global_monitoring() {
    get_monitor().stop_monitoring();
}

/// Record a SIGSEGV event in the global monitor
pub fn record_global_event(crash_info: CrashInfo) {
    get_monitor().record_event(crash_info);
}

/// Generate a global analysis report
pub fn generate_global_report() -> Result<String, Box<dyn std::error::Error>> {
    get_monitor().generate_analysis_report()
}

/// Write global analysis report to file
pub fn write_global_report(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    get_monitor().write_analysis_report(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitor_creation() {
        let monitor = SigsegvMonitor::new("test_logs".into());
        assert_eq!(monitor.event_count.load(Ordering::SeqCst), 0);
        assert!(!monitor.is_monitoring.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_alignment_validator() {
        let validator = AlignmentValidator::new();
        
        // Test aligned address
        let aligned_addr = 16;  // 16-byte aligned
        let alignment = AlignmentValidator::get_alignment(aligned_addr);
        assert!(alignment >= 8);
        
        // Test unaligned address
        let unaligned_addr = 15;
        let alignment = AlignmentValidator::get_alignment(unaligned_addr);
        assert_eq!(alignment, 1);
    }
    
    #[test]
    fn test_platform_detection() {
        let platform = SigsegvMonitor::detect_platform_info();
        assert!(!platform.os_name.is_empty());
        assert!(!platform.architecture.is_empty());
        assert!(!platform.rust_version.is_empty());
    }
}