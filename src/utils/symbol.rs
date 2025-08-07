//! Symbol interning for efficient string handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::SymbolId;

/// A symbol interner for efficient string handling.
#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<String>,
    symbol_map: HashMap<String, SymbolId>,
}

impl SymbolTable {
    /// Creates a new symbol table.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            symbol_map: HashMap::new(),
        }
    }

    /// Interns a symbol and returns its ID.
    pub fn intern(&mut self, name: impl Into<String>) -> SymbolId {
        let name = name.into();
        
        if let Some(&id) = self.symbol_map.get(&name) {
            return id;
        }

        let id = SymbolId::new(self.symbols.len());
        self.symbols.push(name.clone());
        self.symbol_map.insert(name, id);
        id
    }

    /// Gets the name of a symbol by its ID.
    pub fn name(&self, id: SymbolId) -> Option<&str> {
        self.symbols.get(id.id()).map(|s| s.as_str())
    }

    /// Gets the ID of a symbol by its name.
    pub fn id(&self, name: &str) -> Option<SymbolId> {
        self.symbol_map.get(name).copied()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_SYMBOLS: Arc<Mutex<SymbolTable>> = Arc::new(Mutex::new(SymbolTable::new()));
}

/// Interns a symbol in the global symbol table.
pub fn intern_symbol(name: impl Into<String>) -> SymbolId {
    GLOBAL_SYMBOLS.lock().unwrap().intern(name)
}

/// Gets the name of a symbol from the global symbol table.
pub fn symbol_name(id: SymbolId) -> Option<String> {
    GLOBAL_SYMBOLS.lock().unwrap().name(id).map(|s| s.to_string())
}