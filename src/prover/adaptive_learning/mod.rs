//! Adaptive Theorem Learning System
//!
//! This module implements a sophisticated learning system that automatically
//! discovers optimization patterns from real-world Scheme code, accumulates
//! knowledge, and strengthens the theorem derivation system for increasingly
//! sophisticated evaluator performance.
//!
//! ## Implementation Status: RESEARCH PROTOTYPE
//!
//! This module contains experimental research code for adaptive theorem learning.
//! Many structures are currently stubs with planned implementation in Phase 9.
//!
//! ## Module Organization
//!
//! - `core_systems`: Main learning system and pattern discovery engine
//! - `pattern_types`: Data structures for patterns and knowledge representation
//! - `performance_analysis`: Performance analysis and validation components
//!
//! ## TODO Phase 9 Implementation Plan:
//! - Implement machine learning algorithms for pattern discovery
//! - Add persistent knowledge base with serialization support
//! - Implement performance feedback integration
//! - Add real-time adaptation mechanisms
//! - Integrate with existing optimization pipeline
//! - Add statistical analysis and validation

pub mod core_systems;
pub mod pattern_types;
pub mod performance_analysis;

// Re-export main types for backward compatibility
pub use core_systems::{
    AdaptiveTheoremLearningSystem,
    PatternDiscoveryEngine,
    TheoremKnowledgeBase,
};

pub use pattern_types::{
    DiscoveredPattern,
    OccurrenceContext,
    SourceInfo,
    ContextPerformanceData,
    StyleIndicators,
    PatternType,
};

pub use performance_analysis::{
    PerformanceAnalyzer,
    LearnedOptimizationPattern,
    LearnedPerformanceCharacteristics,
    MemoryImpactData,
    ScalabilityCharacteristics,
    PerformanceInsight,
    PerformanceImpactQuantification,
};