//! Debug utilities for memory safety, performance profiling, and diagnostics.

pub mod memory_safety;

pub use memory_safety::{
    check_memory_safety, detect_and_report_cycles, CycleTrackable, 
    SafeArc, MemorySafetyReport, ObjectId, ReferenceType
};