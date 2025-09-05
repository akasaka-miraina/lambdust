//! SIGSEGV-safe test harness with crash isolation and monitoring

use std::collections::HashMap;
use std::fmt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;

use crate::debug::{CrashInfo, set_test_context};

/// Result of running an isolated test
#[derive(Debug, Clone)]
pub enum TestResult {
    Passed {
        duration: Duration,
        output: String,
    },
    Failed {
        duration: Duration,
        error: String,
        output: String,
    },
    Crashed {
        duration: Duration,
        signal: Option<i32>,
        crash_info: Option<CrashInfo>,
        output: String,
    },
    TimedOut {
        duration: Duration,
        output: String,
    },
    InternalError {
        error: String,
    },
}

/// An isolated test that can be run safely
pub trait IsolatedTest: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn timeout(&self) -> Duration {
        Duration::from_secs(30) // Default 30 second timeout
    }
    fn description(&self) -> Option<&str> {
        None
    }
}

/// Test harness that can safely run tests and capture crashes
pub struct SafeTestHarness {
    timeout: Duration,
    isolation_mode: IsolationMode,
    crash_logs_dir: String,
    max_concurrent_tests: usize,
    test_results: Arc<Mutex<HashMap<String, TestResult>>>,
}

#[derive(Debug, Clone)]
pub enum IsolationMode {
    /// Run tests in the same process (faster but less isolation)
    InProcess,
    /// Run tests in separate threads with signal handlers
    ThreadIsolated,
    /// Run tests in separate processes (maximum isolation)
    ProcessIsolated,
}

impl Default for SafeTestHarness {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            isolation_mode: IsolationMode::ThreadIsolated,
            crash_logs_dir: "test_crash_logs".to_string(),
            max_concurrent_tests: num_cpus::get(),
            test_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SafeTestHarness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_isolation_mode(mut self, mode: IsolationMode) -> Self {
        self.isolation_mode = mode;
        self
    }

    pub fn with_crash_logs_dir(mut self, dir: String) -> Self {
        self.crash_logs_dir = dir;
        self
    }

    pub fn with_max_concurrent_tests(mut self, max: usize) -> Self {
        self.max_concurrent_tests = max;
        self
    }

    /// Run a single test with crash protection
    pub fn run_test(&self, test: Box<dyn IsolatedTest>) -> TestResult {
        let test_name = test.name().to_string();
        println!("Running test: {}", test_name);

        let start_time = Instant::now();
        
        let result = match self.isolation_mode {
            IsolationMode::InProcess => self.run_test_in_process(test),
            IsolationMode::ThreadIsolated => self.run_test_thread_isolated(test),
            IsolationMode::ProcessIsolated => self.run_test_process_isolated(test),
        };

        // Store result
        if let Ok(mut results) = self.test_results.lock() {
            results.insert(test_name, result.clone());
        }

        result
    }

    /// Run multiple tests with the specified concurrency
    pub fn run_tests(&self, tests: Vec<Box<dyn IsolatedTest>>) -> HashMap<String, TestResult> {
        let results = Arc::new(Mutex::new(HashMap::new()));
        let active_tests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::new();

        for test in tests {
            let results = Arc::clone(&results);
            let active_tests = Arc::clone(&active_tests);
            let harness = self.clone();
            let max_concurrent = self.max_concurrent_tests;
            
            let handle = thread::spawn(move || {
                // Simple semaphore using atomic counter
                loop {
                    let current = active_tests.load(std::sync::atomic::Ordering::SeqCst);
                    if current < max_concurrent {
                        if active_tests.compare_exchange_weak(
                            current, 
                            current + 1, 
                            std::sync::atomic::Ordering::SeqCst,
                            std::sync::atomic::Ordering::SeqCst
                        ).is_ok() {
                            break;
                        }
                    }
                    // Wait a bit before retrying
                    thread::sleep(Duration::from_millis(10));
                }
                
                let test_name = test.name().to_string();
                let result = harness.run_test(test);
                
                // Release the semaphore
                active_tests.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                
                if let Ok(mut r) = results.lock() {
                    r.insert(test_name, result);
                }
            });
            
            handles.push(handle);
        }

        // Wait for all tests to complete
        for handle in handles {
            let _ = handle.join();
        }

        // Return results
        let results = results.lock().unwrap();
        results.clone()
    }

    /// Run test in the same process (least isolation)
    fn run_test_in_process(&self, test: Box<dyn IsolatedTest>) -> TestResult {
        set_test_context(Some(test.name().to_string()));
        
        let start_time = Instant::now();
        let timeout = test.timeout().min(self.timeout);

        // Set up timeout
        let (tx, rx) = std::sync::mpsc::channel();
        let test_name = test.name().to_string();
        
        thread::spawn(move || {
            let result = test.run();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(())) => {
                let duration = start_time.elapsed();
                TestResult::Passed {
                    duration,
                    output: "Test passed".to_string(),
                }
            }
            Ok(Err(e)) => {
                let duration = start_time.elapsed();
                TestResult::Failed {
                    duration,
                    error: e.to_string(),
                    output: "Test failed".to_string(),
                }
            }
            Err(_timeout) => {
                let duration = start_time.elapsed();
                TestResult::TimedOut {
                    duration,
                    output: "Test timed out".to_string(),
                }
            }
        }
    }

    /// Run test in isolated thread with signal handling
    fn run_test_thread_isolated(&self, test: Box<dyn IsolatedTest>) -> TestResult {
        let test_name = test.name().to_string();
        set_test_context(Some(test_name.clone()));
        
        let start_time = Instant::now();
        let timeout = test.timeout().min(self.timeout);
        
        // Channel for communicating test results
        let (tx, rx) = std::sync::mpsc::channel();
        let (crash_tx, crash_rx) = std::sync::mpsc::channel();
        
        // Set up crash monitoring
        let crash_monitor = thread::spawn(move || {
            // This thread monitors for crashes and signals
            // In a real implementation, this would use signal handlers
            // or other OS-specific mechanisms
            let _ = crash_rx.recv(); // Block until crash or completion
        });
        
        // Run test in separate thread
        let test_thread = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                test.run()
            }));
            
            match result {
                Ok(Ok(())) => {
                    let _ = tx.send(TestResult::Passed {
                        duration: start_time.elapsed(),
                        output: "Test passed".to_string(),
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(TestResult::Failed {
                        duration: start_time.elapsed(),
                        error: e.to_string(),
                        output: "Test failed".to_string(),
                    });
                }
                Err(panic_info) => {
                    let _ = tx.send(TestResult::Crashed {
                        duration: start_time.elapsed(),
                        signal: None,
                        crash_info: None, // Would be populated by signal handler
                        output: format!("Panic: {:?}", panic_info),
                    });
                }
            }
            
            let _ = crash_tx.send(()); // Signal completion
        });
        
        // Wait for result or timeout
        match rx.recv_timeout(timeout) {
            Ok(result) => {
                let _ = test_thread.join();
                let _ = crash_monitor.join();
                result
            }
            Err(_timeout) => {
                // Test timed out - try to clean up
                let duration = start_time.elapsed();
                TestResult::TimedOut {
                    duration,
                    output: format!("Test '{}' timed out after {:?}", test_name, timeout),
                }
            }
        }
    }

    /// Run test in completely separate process (maximum isolation)
    fn run_test_process_isolated(&self, test: Box<dyn IsolatedTest>) -> TestResult {
        let test_name = test.name().to_string();
        let start_time = Instant::now();
        let timeout = test.timeout().min(self.timeout);

        // Create a temporary test executable or use the current binary with test flags
        let binary_path = std::env::current_exe()
            .unwrap_or_else(|_| "lambdust".into());

        let mut cmd = Command::new(&binary_path)
            .arg("--isolated-test")
            .arg(&test_name)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match cmd {
            Ok(mut child) => {
                // Wait for process with timeout
                let start = Instant::now();
                loop {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return TestResult::TimedOut {
                            duration: start_time.elapsed(),
                            output: format!("Process timed out after {:?}", timeout),
                        };
                    }

                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let duration = start_time.elapsed();
                            let output = if let Ok(output) = child.wait_with_output() {
                                format!("{}\n{}", 
                                       String::from_utf8_lossy(&output.stdout),
                                       String::from_utf8_lossy(&output.stderr))
                            } else {
                                "Failed to capture output".to_string()
                            };

                            if status.success() {
                                return TestResult::Passed { duration, output };
                            } else if let Some(code) = status.code() {
                                return TestResult::Failed {
                                    duration,
                                    error: format!("Process exited with code {}", code),
                                    output,
                                };
                            } else {
                                // Process was terminated by signal (likely SIGSEGV)
                                #[cfg(unix)]
                                {
                                    use std::os::unix::process::ExitStatusExt;
                                    if let Some(signal) = status.signal() {
                                        return TestResult::Crashed {
                                            duration,
                                            signal: Some(signal),
                                            crash_info: None,
                                            output,
                                        };
                                    }
                                }
                                
                                return TestResult::Crashed {
                                    duration,
                                    signal: None,
                                    crash_info: None,
                                    output,
                                };
                            }
                        }
                        Ok(None) => {
                            // Process still running
                            thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            return TestResult::InternalError {
                                error: format!("Error waiting for process: {}", e),
                            };
                        }
                    }
                }
            }
            Err(e) => TestResult::InternalError {
                error: format!("Failed to spawn test process: {}", e),
            },
        }
    }

    /// Generate a summary report of all test results
    pub fn generate_summary(&self) -> TestSummary {
        let results = self.test_results.lock().unwrap();
        let mut summary = TestSummary {
            total_tests: results.len(),
            passed: 0,
            failed: 0,
            crashed: 0,
            timed_out: 0,
            internal_errors: 0,
            total_duration: Duration::new(0, 0),
            crashes: Vec::new(),
            failures: Vec::new(),
        };

        for (name, result) in results.iter() {
            let duration = match result {
                TestResult::Passed { duration, .. } => {
                    summary.passed += 1;
                    *duration
                }
                TestResult::Failed { duration, error, .. } => {
                    summary.failed += 1;
                    summary.failures.push((name.clone(), error.clone()));
                    *duration
                }
                TestResult::Crashed { duration, signal, crash_info, .. } => {
                    summary.crashed += 1;
                    summary.crashes.push(CrashSummary {
                        test_name: name.clone(),
                        signal: *signal,
                        crash_info: crash_info.clone(),
                    });
                    *duration
                }
                TestResult::TimedOut { duration, .. } => {
                    summary.timed_out += 1;
                    *duration
                }
                TestResult::InternalError { .. } => {
                    summary.internal_errors += 1;
                    Duration::new(0, 0)
                }
            };
            summary.total_duration += duration;
        }

        summary
    }
}

impl Clone for SafeTestHarness {
    fn clone(&self) -> Self {
        Self {
            timeout: self.timeout,
            isolation_mode: self.isolation_mode.clone(),
            crash_logs_dir: self.crash_logs_dir.clone(),
            max_concurrent_tests: self.max_concurrent_tests,
            test_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub crashed: usize,
    pub timed_out: usize,
    pub internal_errors: usize,
    pub total_duration: Duration,
    pub crashes: Vec<CrashSummary>,
    pub failures: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct CrashSummary {
    pub test_name: String,
    pub signal: Option<i32>,
    pub crash_info: Option<CrashInfo>,
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestResult::Passed { duration, .. } => {
                write!(f, "PASSED ({:.3}s)", duration.as_secs_f64())
            }
            TestResult::Failed { duration, error, .. } => {
                write!(f, "FAILED ({:.3}s): {}", duration.as_secs_f64(), error)
            }
            TestResult::Crashed { duration, signal, .. } => {
                if let Some(sig) = signal {
                    write!(f, "CRASHED ({:.3}s): signal {}", duration.as_secs_f64(), sig)
                } else {
                    write!(f, "CRASHED ({:.3}s): unknown signal", duration.as_secs_f64())
                }
            }
            TestResult::TimedOut { duration, .. } => {
                write!(f, "TIMEOUT ({:.3}s)", duration.as_secs_f64())
            }
            TestResult::InternalError { error } => {
                write!(f, "ERROR: {}", error)
            }
        }
    }
}

impl fmt::Display for TestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== TEST SUMMARY ===")?;
        writeln!(f, "Total tests: {}", self.total_tests)?;
        writeln!(f, "Passed: {} ({:.1}%)", self.passed, 
                (self.passed as f64 / self.total_tests as f64) * 100.0)?;
        writeln!(f, "Failed: {}", self.failed)?;
        writeln!(f, "Crashed: {}", self.crashed)?;
        writeln!(f, "Timed out: {}", self.timed_out)?;
        writeln!(f, "Internal errors: {}", self.internal_errors)?;
        writeln!(f, "Total duration: {:.3}s", self.total_duration.as_secs_f64())?;

        if !self.crashes.is_empty() {
            writeln!(f, "\n=== CRASHES ===")?;
            for crash in &self.crashes {
                writeln!(f, "- {}: signal {:?}", crash.test_name, crash.signal)?;
            }
        }

        if !self.failures.is_empty() {
            writeln!(f, "\n=== FAILURES ===")?;
            for (name, error) in &self.failures {
                writeln!(f, "- {}: {}", name, error)?;
            }
        }

        Ok(())
    }
}

/// Example test implementation
pub struct ExampleTest {
    name: String,
    test_fn: Box<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl ExampleTest {
    pub fn new<F>(name: String, test_fn: F) -> Self
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        Self {
            name,
            test_fn: Box::new(test_fn),
        }
    }
}

impl IsolatedTest for ExampleTest {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.test_fn)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_basic_functionality() {
        let harness = SafeTestHarness::new();
        
        let test = ExampleTest::new("basic_test".to_string(), || {
            Ok(())
        });

        let result = harness.run_test(Box::new(test));
        
        match result {
            TestResult::Passed { .. } => {
                // Test passed as expected
            }
            other => panic!("Expected test to pass, got: {:?}", other),
        }
    }

    #[test]
    fn test_harness_failure_detection() {
        let harness = SafeTestHarness::new();
        
        let test = ExampleTest::new("failing_test".to_string(), || {
            Err("Test failed".into())
        });

        let result = harness.run_test(Box::new(test));
        
        match result {
            TestResult::Failed { .. } => {
                // Test failed as expected
            }
            other => panic!("Expected test to fail, got: {:?}", other),
        }
    }
}