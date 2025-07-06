//! Optimized value representation for Phase 4 memory efficiency
//!
//! This module provides memory-optimized value types using:
//! - SmallInt: Stack-allocated small integers
//! - ShortString: Inline short strings without heap allocation
//! - Tagged union optimization: Efficient representation selection

use crate::lexer::SchemeNumber;
use std::fmt;

/// Memory-optimized value representation
/// Uses tagged unions and inline storage for common small values
#[derive(Clone)]
pub enum OptimizedValue {
    /// Undefined value (used for uninitialized variables)
    Undefined,
    /// Boolean values (1 bit + tag)
    Boolean(bool),
    /// Small integers stored on stack (-128 to 127)
    SmallInt(i8),
    /// Large numbers stored on heap
    Number(SchemeNumber),
    /// Short strings stored inline (up to 15 bytes)
    ShortString(ShortStringData),
    /// Long strings stored on heap
    String(String),
    /// Character values (4 bytes)
    Character(char),
    /// Short symbols stored inline (up to 15 bytes)
    ShortSymbol(ShortStringData),
    /// Long symbols stored on heap
    Symbol(String),
    /// Other value types (unchanged for now)
    Complex(super::Value),
}

/// Inline string storage for strings up to 15 bytes
/// Uses SSO (Small String Optimization) pattern
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShortStringData {
    /// String bytes (up to 15 bytes)
    bytes: [u8; 15],
    /// Length of the string (0-15)
    len: u8,
}

impl ShortStringData {
    /// Create new short string from string slice
    pub fn new(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() > 15 {
            return None;
        }
        
        let mut data = ShortStringData {
            bytes: [0; 15],
            len: bytes.len() as u8,
        };
        
        data.bytes[..bytes.len()].copy_from_slice(bytes);
        Some(data)
    }
    
    /// Get string slice from short string data
    pub fn as_str(&self) -> &str {
        let bytes = &self.bytes[..self.len as usize];
        // SAFETY: We only store valid UTF-8 strings
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
    
    /// Get length of the string
    pub fn len(&self) -> usize {
        self.len as usize
    }
    
    /// Check if string is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Convert to owned String
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl fmt::Debug for ShortStringData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShortString(\"{}\")", self.as_str())
    }
}

impl fmt::Display for ShortStringData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl OptimizedValue {
    /// Create an integer value using optimal representation
    pub fn new_int(n: i64) -> Self {
        if n >= i8::MIN as i64 && n <= i8::MAX as i64 {
            OptimizedValue::SmallInt(n as i8)
        } else {
            OptimizedValue::Number(SchemeNumber::Integer(n))
        }
    }
    
    /// Create a string value using optimal representation
    pub fn new_string(s: String) -> Self {
        if let Some(short) = ShortStringData::new(&s) {
            OptimizedValue::ShortString(short)
        } else {
            OptimizedValue::String(s)
        }
    }
    
    /// Create a string value from &str using optimal representation
    pub fn new_string_ref(s: &str) -> Self {
        if let Some(short) = ShortStringData::new(s) {
            OptimizedValue::ShortString(short)
        } else {
            OptimizedValue::String(s.to_string())
        }
    }
    
    /// Create a symbol value using optimal representation
    pub fn new_symbol(s: String) -> Self {
        if let Some(short) = ShortStringData::new(&s) {
            OptimizedValue::ShortSymbol(short)
        } else {
            OptimizedValue::Symbol(s)
        }
    }
    
    /// Create a symbol value from &str using optimal representation
    pub fn new_symbol_ref(s: &str) -> Self {
        if let Some(short) = ShortStringData::new(s) {
            OptimizedValue::ShortSymbol(short)
        } else {
            OptimizedValue::Symbol(s.to_string())
        }
    }
    
    /// Convert to standard Value (for compatibility)
    pub fn to_standard_value(self) -> super::Value {
        match self {
            OptimizedValue::Undefined => super::Value::Undefined,
            OptimizedValue::Boolean(b) => super::Value::Boolean(b),
            OptimizedValue::SmallInt(n) => super::Value::Number(SchemeNumber::Integer(n as i64)),
            OptimizedValue::Number(n) => super::Value::Number(n),
            OptimizedValue::ShortString(s) => super::Value::String(s.to_string()),
            OptimizedValue::String(s) => super::Value::String(s),
            OptimizedValue::Character(c) => super::Value::Character(c),
            OptimizedValue::ShortSymbol(s) => super::Value::Symbol(s.to_string()),
            OptimizedValue::Symbol(s) => super::Value::Symbol(s),
            OptimizedValue::Complex(v) => v,
        }
    }
    
    /// Create from standard Value (for compatibility)
    pub fn from_standard_value(value: super::Value) -> Self {
        match value {
            super::Value::Undefined => OptimizedValue::Undefined,
            super::Value::Boolean(b) => OptimizedValue::Boolean(b),
            super::Value::Number(SchemeNumber::Integer(n)) => OptimizedValue::new_int(n),
            super::Value::Number(n) => OptimizedValue::Number(n),
            super::Value::String(s) => OptimizedValue::new_string(s),
            super::Value::Character(c) => OptimizedValue::Character(c),
            super::Value::Symbol(s) => OptimizedValue::new_symbol(s),
            other => OptimizedValue::Complex(other),
        }
    }
    
    /// Get memory usage estimate in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            OptimizedValue::Undefined => 0,
            OptimizedValue::Boolean(_) => 1,
            OptimizedValue::SmallInt(_) => 1,
            OptimizedValue::Number(n) => std::mem::size_of_val(n),
            OptimizedValue::ShortString(_) => 16, // 15 bytes + 1 length byte
            OptimizedValue::String(s) => s.capacity(),
            OptimizedValue::Character(_) => 4,
            OptimizedValue::ShortSymbol(_) => 16, // 15 bytes + 1 length byte
            OptimizedValue::Symbol(s) => s.capacity(),
            OptimizedValue::Complex(v) => std::mem::size_of_val(v), // Approximation
        }
    }
    
    /// Check if this is a numeric value
    pub fn is_number(&self) -> bool {
        matches!(self, OptimizedValue::SmallInt(_) | OptimizedValue::Number(_))
    }
    
    /// Check if this is a string value
    pub fn is_string(&self) -> bool {
        matches!(self, OptimizedValue::ShortString(_) | OptimizedValue::String(_))
    }
    
    /// Check if this is a symbol value
    pub fn is_symbol(&self) -> bool {
        matches!(self, OptimizedValue::ShortSymbol(_) | OptimizedValue::Symbol(_))
    }
    
    /// Get as integer if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            OptimizedValue::SmallInt(n) => Some(*n as i64),
            OptimizedValue::Number(SchemeNumber::Integer(n)) => Some(*n),
            _ => None,
        }
    }
    
    /// Get as string if possible
    pub fn as_string(&self) -> Option<String> {
        match self {
            OptimizedValue::ShortString(s) => Some(s.to_string()),
            OptimizedValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }
    
    /// Get as string reference if possible
    pub fn as_string_ref(&self) -> Option<&str> {
        match self {
            OptimizedValue::ShortString(s) => Some(s.as_str()),
            OptimizedValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get as symbol if possible
    pub fn as_symbol(&self) -> Option<String> {
        match self {
            OptimizedValue::ShortSymbol(s) => Some(s.to_string()),
            OptimizedValue::Symbol(s) => Some(s.clone()),
            _ => None,
        }
    }
    
    /// Get as symbol reference if possible
    pub fn as_symbol_ref(&self) -> Option<&str> {
        match self {
            OptimizedValue::ShortSymbol(s) => Some(s.as_str()),
            OptimizedValue::Symbol(s) => Some(s),
            _ => None,
        }
    }
    
    /// Check if value is truthy (for conditional evaluation)
    pub fn is_truthy(&self) -> bool {
        match self {
            OptimizedValue::Boolean(false) => false,
            _ => true,
        }
    }
    
    /// Check if this is an inline value (stored on stack)
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            OptimizedValue::Undefined
                | OptimizedValue::Boolean(_)
                | OptimizedValue::SmallInt(_)
                | OptimizedValue::ShortString(_)
                | OptimizedValue::Character(_)
                | OptimizedValue::ShortSymbol(_)
        )
    }
    
    /// Get the value type as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            OptimizedValue::Undefined => "undefined",
            OptimizedValue::Boolean(_) => "boolean",
            OptimizedValue::SmallInt(_) | OptimizedValue::Number(_) => "number",
            OptimizedValue::ShortString(_) | OptimizedValue::String(_) => "string",
            OptimizedValue::Character(_) => "character",
            OptimizedValue::ShortSymbol(_) | OptimizedValue::Symbol(_) => "symbol",
            OptimizedValue::Complex(v) => match v {
                super::Value::Pair(_) => "pair",
                super::Value::Nil => "nil",
                super::Value::Procedure(_) => "procedure",
                super::Value::Vector(_) => "vector",
                super::Value::Port(_) => "port",
                super::Value::Record(_) => "record",
                super::Value::Values(_) => "values",
                super::Value::Continuation(_) => "continuation",
                super::Value::Promise(_) => "promise",
                super::Value::HashTable(_) => "hash-table",
                super::Value::Box(_) => "box",
                super::Value::Comparator(_) => "comparator",
                super::Value::StringCursor(_) => "string-cursor",
                _ => "unknown",
            },
        }
    }
}

impl fmt::Debug for OptimizedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizedValue::Undefined => write!(f, "Undefined"),
            OptimizedValue::Boolean(b) => write!(f, "Boolean({})", b),
            OptimizedValue::SmallInt(n) => write!(f, "SmallInt({})", n),
            OptimizedValue::Number(n) => write!(f, "Number({:?})", n),
            OptimizedValue::ShortString(s) => write!(f, "ShortString(\"{}\")", s.as_str()),
            OptimizedValue::String(s) => write!(f, "String(\"{}\")", s),
            OptimizedValue::Character(c) => write!(f, "Character('{}')", c),
            OptimizedValue::ShortSymbol(s) => write!(f, "ShortSymbol({})", s.as_str()),
            OptimizedValue::Symbol(s) => write!(f, "Symbol({})", s),
            OptimizedValue::Complex(v) => write!(f, "Complex({:?})", v),
        }
    }
}

impl PartialEq for OptimizedValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OptimizedValue::Undefined, OptimizedValue::Undefined) => true,
            (OptimizedValue::Boolean(a), OptimizedValue::Boolean(b)) => a == b,
            (OptimizedValue::SmallInt(a), OptimizedValue::SmallInt(b)) => a == b,
            (OptimizedValue::SmallInt(a), OptimizedValue::Number(SchemeNumber::Integer(b))) => {
                *a as i64 == *b
            }
            (OptimizedValue::Number(SchemeNumber::Integer(a)), OptimizedValue::SmallInt(b)) => {
                *a == *b as i64
            }
            (OptimizedValue::Number(a), OptimizedValue::Number(b)) => a == b,
            (OptimizedValue::ShortString(a), OptimizedValue::ShortString(b)) => a == b,
            (OptimizedValue::ShortString(a), OptimizedValue::String(b)) => a.as_str() == b,
            (OptimizedValue::String(a), OptimizedValue::ShortString(b)) => a == b.as_str(),
            (OptimizedValue::String(a), OptimizedValue::String(b)) => a == b,
            (OptimizedValue::Character(a), OptimizedValue::Character(b)) => a == b,
            (OptimizedValue::ShortSymbol(a), OptimizedValue::ShortSymbol(b)) => a == b,
            (OptimizedValue::ShortSymbol(a), OptimizedValue::Symbol(b)) => a.as_str() == b,
            (OptimizedValue::Symbol(a), OptimizedValue::ShortSymbol(b)) => a == b.as_str(),
            (OptimizedValue::Symbol(a), OptimizedValue::Symbol(b)) => a == b,
            (OptimizedValue::Complex(a), OptimizedValue::Complex(b)) => {
                // For complex values, we need to implement proper equality
                // This is a simplified version
                std::ptr::eq(a, b)
            }
            _ => false,
        }
    }
}

/// Value optimization statistics
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of small integers optimized
    pub small_ints: usize,
    /// Number of short strings optimized
    pub short_strings: usize,
    /// Number of short symbols optimized
    pub short_symbols: usize,
    /// Total memory saved in bytes
    pub memory_saved: usize,
    /// Total values processed
    pub total_values: usize,
}

impl OptimizationStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        OptimizationStats {
            small_ints: 0,
            short_strings: 0,
            short_symbols: 0,
            memory_saved: 0,
            total_values: 0,
        }
    }
    
    /// Record optimization of a value
    pub fn record(&mut self, original: &super::Value, optimized: &OptimizedValue) {
        self.total_values += 1;
        
        let original_size = std::mem::size_of_val(original);
        let optimized_size = optimized.memory_size();
        
        if optimized_size < original_size {
            self.memory_saved += original_size - optimized_size;
        }
        
        match optimized {
            OptimizedValue::SmallInt(_) => self.small_ints += 1,
            OptimizedValue::ShortString(_) => self.short_strings += 1,
            OptimizedValue::ShortSymbol(_) => self.short_symbols += 1,
            _ => {}
        }
    }
    
    /// Get optimization ratio (0.0 to 1.0)
    pub fn optimization_ratio(&self) -> f64 {
        if self.total_values == 0 {
            0.0
        } else {
            (self.small_ints + self.short_strings + self.short_symbols) as f64 / self.total_values as f64
        }
    }
    
    /// Get memory savings ratio (0.0 to 1.0)
    pub fn memory_savings_ratio(&self) -> f64 {
        if self.total_values == 0 {
            0.0
        } else {
            self.memory_saved as f64 / (self.total_values * std::mem::size_of::<super::Value>()) as f64
        }
    }
}

impl Default for OptimizationStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch optimizer for converting standard values to optimized values
#[derive(Debug)]
pub struct ValueOptimizer {
    stats: OptimizationStats,
}

impl ValueOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        ValueOptimizer {
            stats: OptimizationStats::new(),
        }
    }
    
    /// Optimize a single value
    pub fn optimize(&mut self, value: super::Value) -> OptimizedValue {
        let optimized = OptimizedValue::from_standard_value(value.clone());
        self.stats.record(&value, &optimized);
        optimized
    }
    
    /// Optimize a batch of values
    pub fn optimize_batch(&mut self, values: Vec<super::Value>) -> Vec<OptimizedValue> {
        values.into_iter().map(|v| self.optimize(v)).collect()
    }
    
    /// Get optimization statistics
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = OptimizationStats::new();
    }
}

impl Default for ValueOptimizer {
    fn default() -> Self {
        Self::new()
    }
}