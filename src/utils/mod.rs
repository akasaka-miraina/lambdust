//! Utility functions and helpers for the Lambdust implementation.

pub mod advanced_memory_pool;
pub mod cache;
pub mod gc;
pub mod gc_integration;
pub mod memory_pool;
pub mod profiler;
pub mod string_interner;
pub mod symbol;
pub mod symbol_id;

pub use advanced_memory_pool::{
    ConsPool, GlobalPoolStats, MemoryPool, PoolConfig, PoolManager, PoolStats, SmallObjectPool,
    global_pool_manager,
};
pub use cache::{CacheStats, LruCache, MemoCache};
pub use gc::*;
pub use gc_integration::{
    GcEnvironment, GcIntegration, GcIntegrationConfig, GcRootScanResult, GcValue, maybe_gc_alloc,
    scan_value_for_gc_integration,
};
pub use memory_pool::*;
pub use profiler::*;
pub use string_interner::{
    InternedId, InternedString, PooledString, StringInterner, StringPool, SymbolInterner,
    SymbolInternerStats, get_pooled_string, global_interner_stats, global_string_pool_stats,
    global_symbol_interner_stats, intern,
};
pub use symbol::*;
pub use symbol_id::*;
