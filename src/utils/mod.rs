//! Utility functions and helpers for the Lambdust implementation.

pub mod symbol;
pub mod string_interner;
pub mod memory_pool;
pub mod advanced_memory_pool;
pub mod gc;
pub mod gc_integration;
pub mod profiler;
pub mod symbol_id;
pub mod cache;

pub use symbol::*;
pub use string_interner::{
    StringInterner, InternedString, InternedId, 
    SymbolInterner, SymbolInternerStats, StringPool, PooledString,
    intern, get_pooled_string, global_interner_stats, 
    global_symbol_interner_stats, global_string_pool_stats
};
pub use memory_pool::*;
pub use advanced_memory_pool::{
    PoolManager, MemoryPool, PoolConfig, PoolStats, GlobalPoolStats,
    ConsPool, SmallObjectPool, global_pool_manager
};
pub use gc::*;
pub use gc_integration::{
    GcValue, GcEnvironment, GcIntegration, GcIntegrationConfig,
    GcRootScanResult, maybe_gc_alloc, scan_value_for_gc_integration
};
pub use profiler::*;
pub use symbol_id::*;
pub use cache::{LruCache, MemoCache, CacheStats};