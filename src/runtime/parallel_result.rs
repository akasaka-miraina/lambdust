use crate::diagnostics::Result;
use crate::eval::Value;

/// Result of parallel evaluation.
#[derive(Debug, Clone)]
pub struct ParallelResult {
    /// The evaluation results in order
    pub results: Vec<Result<Value>>,
    /// Timing information
    pub elapsed: std::time::Duration,
    /// Number of threads used
    pub threads_used: usize,
}

impl ParallelResult {
    /// Creates a new parallel result.
    pub fn new(
        results: Vec<Result<Value>>, 
        elapsed: std::time::Duration, 
        threads_used: usize
    ) -> Self {
        Self {
            results,
            elapsed,
            threads_used,
        }
    }
    
    /// Gets the results.
    pub fn results(&self) -> &[Result<Value>] {
        &self.results
    }
    
    /// Gets the elapsed time.
    pub fn elapsed(&self) -> std::time::Duration {
        self.elapsed
    }
    
    /// Gets the number of threads used.
    pub fn threads_used(&self) -> usize {
        self.threads_used
    }
    
    /// Returns true if all results are successful.
    pub fn all_succeeded(&self) -> bool {
        self.results.iter().all(|r| r.is_ok())
    }
    
    /// Returns the first error, if any.
    pub fn first_error(&self) -> Option<&crate::diagnostics::Error> {
        self.results.iter().find_map(|r| r.as_ref().err())
    }
}