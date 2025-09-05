//! SIGSEGV and other signal handlers with detailed crash information capture

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Information captured during a crash
#[derive(Debug, Clone)]
pub struct CrashInfo {
    pub signal: i32,
    pub signal_name: &'static str,
    pub timestamp: u64,
    pub process_id: u32,
    pub thread_id: u64,
    pub fault_address: Option<usize>,
    pub instruction_pointer: Option<usize>,
    pub stack_pointer: Option<usize>,
    pub backtrace: Vec<String>,
    pub register_dump: HashMap<String, usize>,
    pub memory_info: MemoryInfo,
    pub platform_info: String,
    pub test_context: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub heap_size: usize,
    pub stack_size: usize,
    pub virtual_memory: usize,
    pub physical_memory: usize,
}

/// Global crash handler state
static SIGNAL_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
static CRASH_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT_TEST_CONTEXT: Mutex<Option<String>> = Mutex::new(None);

/// SIGSEGV handler implementation
pub struct SigsegvHandler {
    crash_log_path: String,
    max_backtrace_depth: usize,
    enable_register_dump: bool,
}

impl Default for SigsegvHandler {
    fn default() -> Self {
        Self {
            crash_log_path: "crash_logs".to_string(),
            max_backtrace_depth: 50,
            enable_register_dump: true,
        }
    }
}

impl SigsegvHandler {
    pub fn new(crash_log_path: String) -> Self {
        Self {
            crash_log_path,
            max_backtrace_depth: 50,
            enable_register_dump: true,
        }
    }

    pub fn with_max_backtrace_depth(mut self, depth: usize) -> Self {
        self.max_backtrace_depth = depth;
        self
    }

    pub fn with_register_dump(mut self, enable: bool) -> Self {
        self.enable_register_dump = enable;
        self
    }
}

impl fmt::Display for CrashInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== CRASH REPORT ===")?;
        writeln!(f, "Signal: {} ({})", self.signal, self.signal_name)?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(f, "Process ID: {}", self.process_id)?;
        writeln!(f, "Thread ID: {}", self.thread_id)?;
        writeln!(f, "Platform: {}", self.platform_info)?;
        
        if let Some(addr) = self.fault_address {
            writeln!(f, "Fault Address: 0x{:016x}", addr)?;
        }
        
        if let Some(ip) = self.instruction_pointer {
            writeln!(f, "Instruction Pointer: 0x{:016x}", ip)?;
        }
        
        if let Some(sp) = self.stack_pointer {
            writeln!(f, "Stack Pointer: 0x{:016x}", sp)?;
        }

        if let Some(ref context) = self.test_context {
            writeln!(f, "Test Context: {}", context)?;
        }

        writeln!(f, "\nMemory Info:")?;
        writeln!(f, "  Heap Size: {} bytes", self.memory_info.heap_size)?;
        writeln!(f, "  Stack Size: {} bytes", self.memory_info.stack_size)?;
        writeln!(f, "  Virtual Memory: {} bytes", self.memory_info.virtual_memory)?;
        writeln!(f, "  Physical Memory: {} bytes", self.memory_info.physical_memory)?;

        if !self.register_dump.is_empty() {
            writeln!(f, "\nRegister Dump:")?;
            for (reg, value) in &self.register_dump {
                writeln!(f, "  {}: 0x{:016x}", reg, value)?;
            }
        }

        writeln!(f, "\nBacktrace:")?;
        for (i, frame) in self.backtrace.iter().enumerate() {
            writeln!(f, "  #{:2}: {}", i, frame)?;
        }

        Ok(())
    }
}

/// Set the current test context for crash reporting
pub fn set_test_context(context: Option<String>) {
    if let Ok(mut ctx) = CURRENT_TEST_CONTEXT.lock() {
        *ctx = context;
    }
}

/// Get current memory information
fn get_memory_info() -> MemoryInfo {
    // Platform-specific memory information gathering
    #[cfg(target_os = "macos")]
    {
        get_memory_info_macos()
    }
    #[cfg(target_os = "linux")]
    {
        get_memory_info_linux()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        MemoryInfo {
            heap_size: 0,
            stack_size: 0,
            virtual_memory: 0,
            physical_memory: 0,
        }
    }
}

#[cfg(target_os = "macos")]
fn get_memory_info_macos() -> MemoryInfo {
    use libc::{getpid, proc_pidinfo, proc_taskinfo, PROC_PIDTASKINFO};
    use std::mem;

    #[repr(C)]
    struct ProcTaskInfo {
        pti_virtual_size: u64,
        pti_resident_size: u64,
        pti_total_user: u64,
        pti_total_system: u64,
        pti_threads_user: u64,
        pti_threads_system: u64,
        pti_policy: i32,
        pti_faults: i32,
        pti_pageins: i32,
        pti_cow_faults: i32,
        pti_messages_sent: i32,
        pti_messages_received: i32,
        pti_syscalls_mach: i32,
        pti_syscalls_unix: i32,
        pti_csw: i32,
        pti_threadnum: i32,
        pti_numrunning: i32,
        pti_priority: i32,
    }

    unsafe {
        let mut task_info: ProcTaskInfo = mem::zeroed();
        let pid = getpid();
        let ret = proc_pidinfo(
            pid,
            PROC_PIDTASKINFO,
            0,
            &mut task_info as *mut _ as *mut libc::c_void,
            mem::size_of::<ProcTaskInfo>() as i32,
        );

        if ret == mem::size_of::<ProcTaskInfo>() as i32 {
            MemoryInfo {
                heap_size: task_info.pti_virtual_size as usize,
                stack_size: 0, // Not directly available
                virtual_memory: task_info.pti_virtual_size as usize,
                physical_memory: task_info.pti_resident_size as usize,
            }
        } else {
            MemoryInfo {
                heap_size: 0,
                stack_size: 0,
                virtual_memory: 0,
                physical_memory: 0,
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn get_memory_info_linux() -> MemoryInfo {
    use std::fs;
    
    let status_content = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut virtual_memory = 0;
    let mut physical_memory = 0;
    
    for line in status_content.lines() {
        if line.starts_with("VmSize:") {
            if let Some(size_str) = line.split_whitespace().nth(1) {
                virtual_memory = size_str.parse::<usize>().unwrap_or(0) * 1024; // Convert from kB
            }
        } else if line.starts_with("VmRSS:") {
            if let Some(size_str) = line.split_whitespace().nth(1) {
                physical_memory = size_str.parse::<usize>().unwrap_or(0) * 1024; // Convert from kB
            }
        }
    }

    MemoryInfo {
        heap_size: virtual_memory,
        stack_size: 0, // Would need to parse /proc/self/maps for accurate stack size
        virtual_memory,
        physical_memory,
    }
}

/// Generate a detailed backtrace
fn generate_backtrace(max_depth: usize) -> Vec<String> {
    let mut backtrace_strings = Vec::new();
    
    // Use backtrace crate for detailed stack trace
    let bt = backtrace::Backtrace::new();
    
    for (i, frame) in bt.frames().iter().enumerate().take(max_depth) {
        let mut frame_info = format!("#{:02}: ", i);
        
        if frame.symbols().is_empty() {
            frame_info.push_str(&format!("0x{:016x} - <unknown>", frame.ip() as usize));
        } else {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    frame_info.push_str(&format!("{}", name));
                } else {
                    frame_info.push_str("<unknown symbol>");
                }
                
                if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                    frame_info.push_str(&format!(" at {}:{}", file.display(), line));
                }
                
                frame_info.push_str(&format!(" (0x{:016x})", frame.ip() as usize));
                break; // Only show first symbol per frame
            }
        }
        
        backtrace_strings.push(frame_info);
    }
    
    backtrace_strings
}

/// Get platform-specific register dump
fn get_register_dump() -> HashMap<String, usize> {
    let mut registers = HashMap::new();
    
    // This would require platform-specific assembly or libc calls
    // For now, we'll add placeholder values
    #[cfg(target_arch = "x86_64")]
    {
        // Would use getcontext() or similar to get actual register values
        registers.insert("rax".to_string(), 0);
        registers.insert("rbx".to_string(), 0);
        registers.insert("rcx".to_string(), 0);
        registers.insert("rdx".to_string(), 0);
        registers.insert("rsi".to_string(), 0);
        registers.insert("rdi".to_string(), 0);
        registers.insert("rbp".to_string(), 0);
        registers.insert("rsp".to_string(), 0);
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 registers
        for i in 0..31 {
            registers.insert(format!("x{}", i), 0);
        }
        registers.insert("sp".to_string(), 0);
        registers.insert("pc".to_string(), 0);
    }
    
    registers
}

/// Create crash information from signal
fn create_crash_info(signal: i32, fault_addr: Option<usize>) -> CrashInfo {
    let signal_name = match signal {
        libc::SIGSEGV => "SIGSEGV",
        libc::SIGABRT => "SIGABRT",
        libc::SIGILL => "SIGILL",
        libc::SIGFPE => "SIGFPE",
        libc::SIGBUS => "SIGBUS",
        _ => "UNKNOWN",
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let test_context = CURRENT_TEST_CONTEXT.lock()
        .ok()
        .and_then(|ctx| ctx.clone());

    CrashInfo {
        signal,
        signal_name,
        timestamp,
        process_id: std::process::id(),
        thread_id: unsafe { libc::pthread_self() } as u64,
        fault_address: fault_addr,
        instruction_pointer: None, // Would need signal context
        stack_pointer: None,       // Would need signal context
        backtrace: generate_backtrace(50),
        register_dump: get_register_dump(),
        memory_info: get_memory_info(),
        platform_info: crate::debug::platform_detector::detect_platform().to_string(),
        test_context,
    }
}

/// Write crash info to log file
fn write_crash_log(crash_info: &CrashInfo, log_dir: &str) {
    let crash_id = CRASH_COUNT.fetch_add(1, Ordering::SeqCst);
    let filename = format!("{}/crash_{:04}_{}.log", log_dir, crash_id, crash_info.timestamp);
    
    // Ensure log directory exists
    if let Err(_) = std::fs::create_dir_all(log_dir) {
        eprintln!("Failed to create crash log directory: {}", log_dir);
        return;
    }

    match OpenOptions::new().create(true).write(true).open(&filename) {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", crash_info) {
                eprintln!("Failed to write crash log: {}", e);
            } else {
                eprintln!("Crash log written to: {}", filename);
            }
        }
        Err(e) => {
            eprintln!("Failed to create crash log file {}: {}", filename, e);
        }
    }
}

/// Signal handler implementation
extern "C" fn signal_handler(signal: libc::c_int) {
    let crash_info = create_crash_info(signal, None);
    
    // Write to stderr immediately
    eprintln!("\n{}", crash_info);
    
    // Write to log file
    write_crash_log(&crash_info, "crash_logs");
    
    // Restore default handler and re-raise signal
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

/// Install signal handlers for crash detection
pub fn install_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
    if SIGNAL_HANDLER_INSTALLED.load(Ordering::SeqCst) {
        return Ok(());
    }

    unsafe {
        // Install handlers for various crash signals
        libc::signal(libc::SIGSEGV, signal_handler as *const extern "C" fn(libc::c_int) as usize);
        libc::signal(libc::SIGABRT, signal_handler as *const extern "C" fn(libc::c_int) as usize);
        libc::signal(libc::SIGILL, signal_handler as *const extern "C" fn(libc::c_int) as usize);
        libc::signal(libc::SIGFPE, signal_handler as *const extern "C" fn(libc::c_int) as usize);
        libc::signal(libc::SIGBUS, signal_handler as *const extern "C" fn(libc::c_int) as usize);
    }

    // Also set up panic handler
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let crash_info = CrashInfo {
            signal: -1, // Special value for panic
            signal_name: "PANIC",
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            process_id: std::process::id(),
            thread_id: unsafe { libc::pthread_self() } as u64,
            fault_address: None,
            instruction_pointer: None,
            stack_pointer: None,
            backtrace: generate_backtrace(50),
            register_dump: HashMap::new(),
            memory_info: get_memory_info(),
            platform_info: crate::debug::platform_detector::detect_platform().to_string(),
            test_context: CURRENT_TEST_CONTEXT.lock().ok().and_then(|ctx| ctx.clone()),
        };

        eprintln!("\n=== PANIC DETECTED ===");
        eprintln!("{}", panic_info);
        eprintln!("{}", crash_info);
        write_crash_log(&crash_info, "crash_logs");
        
        // Call original panic handler
        default_hook(panic_info);
    }));

    SIGNAL_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
    println!("Signal handlers installed for crash detection");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_info_display() {
        let crash_info = CrashInfo {
            signal: libc::SIGSEGV,
            signal_name: "SIGSEGV",
            timestamp: 1234567890,
            process_id: 1234,
            thread_id: 5678,
            fault_address: Some(0xdeadbeef),
            instruction_pointer: Some(0x12345678),
            stack_pointer: Some(0x87654321),
            backtrace: vec!["main+0x10".to_string(), "start+0x5".to_string()],
            register_dump: {
                let mut map = HashMap::new();
                map.insert("rax".to_string(), 0x1111111111111111);
                map
            },
            memory_info: MemoryInfo {
                heap_size: 1024,
                stack_size: 512,
                virtual_memory: 2048,
                physical_memory: 1536,
            },
            platform_info: "test-platform".to_string(),
            test_context: Some("test_function".to_string()),
        };

        let output = format!("{}", crash_info);
        assert!(output.contains("SIGSEGV"));
        assert!(output.contains("0xdeadbeef"));
        assert!(output.contains("test_function"));
    }
}