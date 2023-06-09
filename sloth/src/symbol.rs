use std::collections::HashMap;

// TODO: Change name with some sort of path to make modules possible

#[derive(Debug)]
pub struct SymbolTableStack {
    tables: Vec<SymbolTable>,
}

impl Default for SymbolTableStack {
    fn default() -> Self {
        Self {
            tables: vec![SymbolTable::default()],
        }
    }
}

impl SymbolTableStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for table in self.tables.iter().rev() {
            if let Some(symbol) = table.get(name) {
                return Some(symbol);
            }
        }

        None
    }

    /// Returning true means a symbol was overriden
    pub fn insert(&mut self, name: impl Into<String>, symbol: Symbol) -> bool {
        let head = self.tables.len() - 1;
        self.tables[head].insert(name, symbol)
    }

    pub fn push(&mut self) {
        self.tables.push(SymbolTable::default());
    }

    pub fn pop(&mut self) -> bool {
        if self.tables.len() <= 1 {
            // Symbol table stacks must always have atleast 1 stack
            return false;
        }

        self.tables.pop();
        true
    }

    pub fn root(&self) -> &SymbolTable {
        &self.tables[0]
    }

    pub fn root_mut(&mut self) -> &mut SymbolTable {
        &mut self.tables[0]
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Returning true means a symbol was overriden
    pub fn insert(&mut self, name: impl Into<String>, symbol: Symbol) -> bool {
        self.symbols.insert(name.into(), symbol).is_some()
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub typ: Option<SymbolType>,
}

#[derive(Debug)]
pub enum SymbolType {
    Function,
}
