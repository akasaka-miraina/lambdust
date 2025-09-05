//! Memory allocation tracking and leak detection for SIGSEGV debugging

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use std::backtrace::Backtrace;
use std::fmt;

/// Information about a memory allocation
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub address: usize,
    pub size: usize,
    pub timestamp: u64,
    pub backtrace: String,
    pub thread_id: u64,
    pub tag: Option<String>,
}

/// Memory leak report
#[derive(Debug)]
pub struct LeakReport {
    pub total_leaked_bytes: usize,
    pub total_leaked_allocations: usize,
    pub leaks_by_location: HashMap<String, Vec<AllocationInfo>>,
}

/// Global memory tracking state
static MEMORY_TRACKER: Mutex<Option<MemoryTracker>> = Mutex::new(None);

/// Memory allocation tracker
pub struct MemoryTracker {
    allocations: Arc<RwLock<HashMap<usize, AllocationInfo>>>,
    peak_memory: Arc<Mutex<usize>>,
    current_memory: Arc<Mutex<usize>>,
    enable_backtrace: bool,
    max_tracked_allocations: usize,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            peak_memory: Arc::new(Mutex::new(0)),
            current_memory: Arc::new(Mutex::new(0)),
            enable_backtrace: true,
            max_tracked_allocations: 10000,
        }
    }
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_backtrace(mut self, enable: bool) -> Self {
        self.enable_backtrace = enable;
        self
    }

    pub fn with_max_tracked_allocations(mut self, max: usize) -> Self {
        self.max_tracked_allocations = max;
        self
    }

    /// Initialize global memory tracker
    pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
        let tracker = MemoryTracker::new();
        let mut global_tracker = MEMORY_TRACKER.lock().unwrap();
        *global_tracker = Some(tracker);
        println!("Memory tracker initialized");
        Ok(())
    }

    /// Clean up memory tracker
    pub fn cleanup() {
        let mut global_tracker = MEMORY_TRACKER.lock().unwrap();
        if let Some(ref tracker) = *global_tracker {
            let report = tracker.generate_leak_report();
            if report.total_leaked_allocations > 0 {
                println!("Memory leaks detected during cleanup:");
                println!("{}", report);
            }
        }
        *global_tracker = None;
    }

    /// Record a memory allocation
    pub fn record_allocation(&self, address: usize, size: usize, tag: Option<String>) {
        if address == 0 {
            return; // Ignore null allocations
        }

        let allocations_count = {
            let allocations = self.allocations.read().unwrap();
            allocations.len()
        };

        // Don't track too many allocations to avoid memory overhead
        if allocations_count >= self.max_tracked_allocations {
            return;
        }

        let backtrace_str = if self.enable_backtrace {
            let bt = Backtrace::capture();
            format!("{}", bt)
        } else {
            String::new()
        };

        let info = AllocationInfo {
            address,
            size,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            backtrace: backtrace_str,
            thread_id: crate::debug::signal_handler::get_thread_id(),
            tag,
        };

        {
            let mut allocations = self.allocations.write().unwrap();
            allocations.insert(address, info);
        }

        // Update current and peak memory
        {
            let mut current = self.current_memory.lock().unwrap();
            *current += size;
            let mut peak = self.peak_memory.lock().unwrap();
            if *current > *peak {
                *peak = *current;
            }
        }
    }

    /// Record a memory deallocation
    pub fn record_deallocation(&self, address: usize) {
        if address == 0 {
            return; // Ignore null deallocations
        }

        let size = {
            let mut allocations = self.allocations.write().unwrap();
            if let Some(info) = allocations.remove(&address) {
                info.size
            } else {
                // Double-free or untracked allocation
                eprintln!("Warning: Attempting to deallocate untracked address: 0x{:x}", address);
                return;
            }
        };

        // Update current memory
        {
            let mut current = self.current_memory.lock().unwrap();
            if *current >= size {
                *current -= size;
            } else {
                eprintln!("Warning: Memory tracking inconsistency - current < deallocated");
                *current = 0;
            }
        }
    }

    /// Generate a leak report
    pub fn generate_leak_report(&self) -> LeakReport {
        let allocations = self.allocations.read().unwrap();
        let mut leaks_by_location = HashMap::new();
        let mut total_leaked_bytes = 0;

        for info in allocations.values() {
            total_leaked_bytes += info.size;
            
            // Extract the key location from backtrace (first non-system frame)
            let location_key = extract_key_location(&info.backtrace);
            leaks_by_location.entry(location_key).or_insert_with(Vec::new).push(info.clone());
        }

        LeakReport {
            total_leaked_bytes,
            total_leaked_allocations: allocations.len(),
            leaks_by_location,
        }
    }

    /// Get current memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        let current = *self.current_memory.lock().unwrap();
        let peak = *self.peak_memory.lock().unwrap();
        let tracked_allocations = self.allocations.read().unwrap().len();

        MemoryStats {
            current_bytes: current,
            peak_bytes: peak,
            tracked_allocations,
        }
    }

    /// Check for potential memory corruption patterns
    pub fn check_corruption_patterns(&self) -> Vec<CorruptionPattern> {
        let allocations = self.allocations.read().unwrap();
        let mut patterns = Vec::new();

        // Check for suspicious allocation patterns
        let mut size_histogram = HashMap::new();
        for info in allocations.values() {
            *size_histogram.entry(info.size).or_insert(0) += 1;
        }

        // Look for excessive small allocations (potential fragmentation)
        let small_alloc_count = size_histogram.iter()
            .filter(|(&size, _)| size <= 64)
            .map(|(_, &count)| count)
            .sum::<usize>();
        
        if small_alloc_count > 1000 {
            patterns.push(CorruptionPattern::ExcessiveSmallAllocations(small_alloc_count));
        }

        // Look for very large allocations
        for (&size, &count) in &size_histogram {
            if size > 10 * 1024 * 1024 { // > 10MB
                patterns.push(CorruptionPattern::LargeAllocation { size, count });
            }
        }

        patterns
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub current_bytes: usize,
    pub peak_bytes: usize,
    pub tracked_allocations: usize,
}

#[derive(Debug)]
pub enum CorruptionPattern {
    ExcessiveSmallAllocations(usize),
    LargeAllocation { size: usize, count: usize },
    SuspiciousPointerPattern(String),
}

/// Extract key location from backtrace for grouping leaks
fn extract_key_location(backtrace: &str) -> String {
    // Find first non-system frame
    for line in backtrace.lines() {
        let line = line.trim();
        if line.contains("src/") || line.contains("lambdust::") {
            return line.to_string();
        }
    }
    
    // Fallback to first line if no source location found
    backtrace.lines().next().unwrap_or("unknown").to_string()
}

impl fmt::Display for LeakReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== MEMORY LEAK REPORT ===")?;
        writeln!(f, "Total leaked bytes: {}", self.total_leaked_bytes)?;
        writeln!(f, "Total leaked allocations: {}", self.total_leaked_allocations)?;
        
        if !self.leaks_by_location.is_empty() {
            writeln!(f, "\nLeaks by location:")?;
            for (location, leaks) in &self.leaks_by_location {
                let total_bytes: usize = leaks.iter().map(|l| l.size).sum();
                writeln!(f, "  {} ({} bytes in {} allocations)", location, total_bytes, leaks.len())?;
                
                // Show first few allocations as examples
                for leak in leaks.iter().take(3) {
                    writeln!(f, "    - {} bytes at 0x{:x} (timestamp: {})", 
                            leak.size, leak.address, leak.timestamp)?;
                }
                
                if leaks.len() > 3 {
                    writeln!(f, "    ... and {} more", leaks.len() - 3)?;
                }
            }
        }
        
        Ok(())
    }
}

impl fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory Stats:")?;
        writeln!(f, "  Current: {} bytes", self.current_bytes)?;
        writeln!(f, "  Peak: {} bytes", self.peak_bytes)?;
        writeln!(f, "  Tracked allocations: {}", self.tracked_allocations)?;
        Ok(())
    }
}

/// Global functions for easy integration
pub fn record_allocation(address: usize, size: usize, tag: Option<String>) {
    if let Ok(tracker) = MEMORY_TRACKER.lock() {
        if let Some(ref t) = *tracker {
            t.record_allocation(address, size, tag);
        }
    }
}

pub fn record_deallocation(address: usize) {
    if let Ok(tracker) = MEMORY_TRACKER.lock() {
        if let Some(ref t) = *tracker {
            t.record_deallocation(address);
        }
    }
}

pub fn get_memory_stats() -> Option<MemoryStats> {
    if let Ok(tracker) = MEMORY_TRACKER.lock() {
        tracker.as_ref().map(|t| t.get_memory_stats())
    } else {
        None
    }
}

pub fn generate_leak_report() -> Option<LeakReport> {
    if let Ok(tracker) = MEMORY_TRACKER.lock() {
        tracker.as_ref().map(|t| t.generate_leak_report())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_basic() {
        let tracker = MemoryTracker::new().with_backtrace(false);
        
        // Record allocation
        tracker.record_allocation(0x1000, 100, Some("test".to_string()));
        
        let stats = tracker.get_memory_stats();
        assert_eq!(stats.current_bytes, 100);
        assert_eq!(stats.tracked_allocations, 1);
        
        // Record deallocation
        tracker.record_deallocation(0x1000);
        
        let stats = tracker.get_memory_stats();
        assert_eq!(stats.current_bytes, 0);
        assert_eq!(stats.tracked_allocations, 0);
    }

    #[test]
    fn test_leak_report() {
        let tracker = MemoryTracker::new().with_backtrace(false);
        
        // Record allocation without deallocation
        tracker.record_allocation(0x1000, 100, Some("leaked".to_string()));
        tracker.record_allocation(0x2000, 200, Some("leaked".to_string()));
        
        let report = tracker.generate_leak_report();
        assert_eq!(report.total_leaked_bytes, 300);
        assert_eq!(report.total_leaked_allocations, 2);
    }
}