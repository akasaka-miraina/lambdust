//! Core Types for Formal Verification
//!
//! このモジュールは形式的検証システムの基本型定義を含みます。
//! 証明義務、形式的文、証明証拠、検証結果などの型定義が含まれます。

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Proof obligation that must be formally verified
#[derive(Debug, Clone)]
pub struct ProofObligation {
    /// Unique identifier for this obligation
    pub id: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Category of the proof obligation
    pub category: ProofCategory,
    
    /// Mathematical statement to prove
    pub statement: FormalStatement,
    
    /// Priority level for proof scheduling
    pub priority: ProofPriority,
    
    /// Current proof status
    pub status: ProofStatus,
    
    /// Associated evidence or partial proofs
    pub evidence: Vec<ProofEvidence>,
    
    /// Dependencies on other proof obligations
    pub dependencies: Vec<String>,
}

/// Categories of proof obligations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProofCategory {
    /// Universe polymorphism correctness
    UniversePolymorphism,
    
    /// Combinatory logic equivalence
    CombinatoryLogic,
    
    /// Homotopy type theory consistency
    HomotopyTypeTheory,
    
    /// Monad transformer composition laws
    MonadTransformers,
    
    /// Semantic evaluator correctness
    SemanticCorrectness,
    
    /// Type system soundness
    TypeSystemSoundness,
    
    /// Memory safety guarantees
    MemorySafety,
    
    /// Performance bounds
    PerformanceBounds,
}

/// Mathematical statement in formal logic
#[derive(Debug, Clone)]
pub struct FormalStatement {
    /// Statement in first-order logic
    pub formula: String,
    
    /// Preconditions that must hold
    pub preconditions: Vec<String>,
    
    /// Postconditions that are guaranteed
    pub postconditions: Vec<String>,
    
    /// Quantified variables
    pub quantifiers: Vec<Quantifier>,
    
    /// Associated Agda/Coq/Lean code
    pub formal_code: Option<String>,
}

/// Quantifier in formal logic
#[derive(Debug, Clone)]
pub struct Quantifier {
    /// Variable name
    pub variable: String,
    
    /// Quantifier type (forall, exists)
    pub quantifier_type: QuantifierType,
    
    /// Type/domain of the variable
    pub domain: String,
}

/// Types of logical quantifiers
#[derive(Debug, Clone, PartialEq)]
pub enum QuantifierType {
    /// Universal quantification (∀)
    ForAll,
    
    /// Existential quantification (∃)
    Exists,
    
    /// Unique existence (∃!)
    ExistsUnique,
}

/// Priority levels for proof obligations
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum ProofPriority {
    /// Critical soundness properties
    Critical = 0,
    
    /// Important correctness properties
    High = 1,
    
    /// Performance and optimization properties
    Medium = 2,
    
    /// Nice-to-have theoretical properties
    Low = 3,
}

/// Status of a proof obligation
#[derive(Debug, Clone, PartialEq)]
pub enum ProofStatus {
    /// Not yet attempted
    Pending,
    
    /// Currently being worked on
    InProgress,
    
    /// Proof completed successfully
    Proven,
    
    /// Proof failed (counterexample found)
    Disproven,
    
    /// Proof timed out or resource exhausted
    Timeout,
    
    /// Proof skipped due to dependencies
    Skipped,
}

/// Evidence supporting a proof
#[derive(Debug, Clone)]
pub enum ProofEvidence {
    /// Property-based test results
    PropertyTests {
        /// Number of tests that passed
        passed: usize,
        /// Number of tests that failed
        failed: usize,
        /// Counterexamples found during testing
        counterexamples: Vec<String>,
    },
    
    /// Formal proof in external tool
    FormalProof {
        /// External proof tool used
        tool: ProofTool,
        /// Path to proof file
        proof_file: PathBuf,
        /// Checksum for proof integrity
        checksum: String,
    },
    
    /// Automatic prover result
    AutomaticProof {
        /// Name of the automatic prover
        prover: String,
        /// Steps in the proof derivation
        steps: Vec<ProofStep>,
        /// Time taken to find the proof
        time_taken: Duration,
    },
    
    /// Manual verification
    ManualVerification {
        /// Name or identifier of the verifier
        verifier: String,
        /// Date of verification
        date: String,
        /// Additional verification notes
        notes: String,
    },
}

/// External proof tools
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProofTool {
    /// Agda proof assistant
    Agda,
    
    /// Coq proof assistant  
    Coq,
    
    /// Lean proof assistant
    Lean,
    
    /// Isabelle/HOL
    Isabelle,
    
    /// PVS specification language
    PVS,
    
    /// TLA+ specification language
    TLA,
}

/// Single step in a proof derivation
#[derive(Debug, Clone)]
pub struct ProofStep {
    /// Step number
    pub step_number: usize,
    
    /// Rule or inference used
    pub rule: String,
    
    /// Premises for this step
    pub premises: Vec<String>,
    
    /// Conclusion derived
    pub conclusion: String,
    
    /// Justification or explanation
    pub justification: String,
}

/// Result of formal verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// ID of the proof obligation
    pub obligation_id: String,
    
    /// Overall verification outcome
    pub result: VerificationOutcome,
    
    /// Evidence collected during verification
    pub evidence: Vec<ProofEvidence>,
    
    /// Time taken for verification
    pub time_taken: Duration,
    
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    
    /// Any issues or warnings found
    pub issues: Vec<VerificationIssue>,
}

/// Possible outcomes of verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationOutcome {
    /// Verification succeeded
    Success,
    
    /// Verification failed (counterexample found)
    Failure,
    
    /// Verification incomplete (timeout or resource limit)
    Incomplete,
    
    /// Verification skipped (dependencies not met)
    Skipped,
}

/// Issues found during verification
#[derive(Debug, Clone)]
pub struct VerificationIssue {
    /// Severity level
    pub severity: IssueSeverity,
    
    /// Issue description
    pub description: String,
    
    /// Location in code (if applicable)
    pub location: Option<String>,
    
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Severity levels for verification issues
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    /// Critical issue that prevents verification
    Critical,
    
    /// Important issue that should be addressed
    Warning,
    
    /// Minor issue or suggestion
    Info,
}

/// Statistics about verification activities
#[derive(Debug, Clone, Default)]
pub struct VerificationStatistics {
    /// Total number of proof obligations processed
    pub total_obligations: usize,
    
    /// Number of obligations successfully proven
    pub proven_obligations: usize,
    
    /// Number of obligations that failed proof
    pub failed_obligations: usize,
    
    /// Number of obligations skipped
    pub skipped_obligations: usize,
    
    /// Total time spent on verification
    pub total_time: Duration,
    
    /// Average time per obligation
    pub average_time: Duration,
    
    /// Number of property tests executed
    pub property_tests_run: usize,
    
    /// Number of external tool invocations
    pub external_tool_calls: usize,
}

/// Configuration for formal verification
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Maximum time to spend on each obligation
    pub max_time_per_obligation: Duration,
    
    /// Number of property test cases
    pub property_test_cases: usize,
    
    /// Enable external proof tools
    pub enable_external_tools: bool,
    
    /// Agda executable path
    pub agda_path: Option<PathBuf>,
    
    /// Coq executable path
    pub coq_path: Option<PathBuf>,
    
    /// Lean executable path
    pub lean_path: Option<PathBuf>,
    
    /// Verification cache size
    pub cache_size: usize,
}

/// Manager for proof obligations
#[derive(Debug)]
pub struct ProofObligationManager {
    /// Active proof obligations
    pub obligations: HashMap<String, ProofObligation>,
    
    /// Dependency graph
    pub dependencies: HashMap<String, Vec<String>>,
    
    /// Obligation categories
    pub categories: HashMap<ProofCategory, Vec<String>>,
}

impl ProofObligationManager {
    /// Create a new proof obligation manager
    pub fn new() -> Self {
        Self {
            obligations: HashMap::new(),
            dependencies: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// Add a new proof obligation
    pub fn add_obligation(&mut self, obligation: ProofObligation) {
        let id = obligation.id.clone();
        let category = obligation.category.clone();
        
        // Add to category index
        self.categories.entry(category).or_insert_with(Vec::new).push(id.clone());
        
        // Store the obligation
        self.obligations.insert(id, obligation);
    }
    
    /// Get a proof obligation by ID
    pub fn get_obligation(&self, id: &str) -> Option<&ProofObligation> {
        self.obligations.get(id)
    }
    
    /// Get all obligations in a category
    pub fn get_obligations_by_category(&self, category: &ProofCategory) -> Vec<&ProofObligation> {
        self.categories.get(category)
            .map(|ids| ids.iter().filter_map(|id| self.obligations.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get obligations ready for proof (dependencies satisfied)
    pub fn get_ready_obligations(&self) -> Vec<&ProofObligation> {
        self.obligations.values()
            .filter(|obligation| {
                obligation.status == ProofStatus::Pending &&
                obligation.dependencies.iter().all(|dep_id| {
                    self.obligations.get(dep_id)
                        .map(|dep| dep.status == ProofStatus::Proven)
                        .unwrap_or(false)
                })
            })
            .collect()
    }
    
    /// Update obligation status
    pub fn update_status(&mut self, id: &str, status: ProofStatus) {
        if let Some(obligation) = self.obligations.get_mut(id) {
            obligation.status = status;
        }
    }
    
    /// Add evidence to an obligation
    pub fn add_evidence(&mut self, id: &str, evidence: ProofEvidence) {
        if let Some(obligation) = self.obligations.get_mut(id) {
            obligation.evidence.push(evidence);
        }
    }
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            max_time_per_obligation: Duration::from_secs(300), // 5 minutes
            property_test_cases: 1000,
            enable_external_tools: false, // Disabled by default
            agda_path: None,
            coq_path: None,
            lean_path: None,
            cache_size: 1000,
        }
    }
}

impl Default for ProofObligationManager {
    fn default() -> Self {
        Self::new()
    }
}