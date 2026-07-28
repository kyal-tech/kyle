// kyc_core::symbol — Interned symbol table for efficient string management
//
// Symbols are interned strings identified by a usize ID.
// This provides fast comparison and compact storage.

use std::collections::HashMap;

pub type Symbol = usize;

pub const INVALID_SYMBOL: Symbol = usize::MAX;

#[derive(Clone, Debug)]
pub struct SymbolTable {
    strings: Vec<String>,
    lookup: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&id) = self.lookup.get(s) {
            return id;
        }
        let id = self.strings.len();
        self.strings.push(s.to_string());
        self.lookup.insert(s.to_string(), id);
        id
    }

    pub fn lookup(&self, s: &str) -> Option<Symbol> {
        self.lookup.get(s).copied()
    }

    pub fn get(&self, sym: Symbol) -> Option<&str> {
        self.strings.get(sym).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Symbol, &str)> {
        self.strings.iter().enumerate().map(|(i, s)| (i, s.as_str()))
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
