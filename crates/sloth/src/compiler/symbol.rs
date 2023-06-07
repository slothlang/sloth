use std::collections::HashMap;

pub struct SymbolTableStack {
    inner: Vec<SymbolTable>,
}

impl SymbolTableStack {
    pub fn push_scope(&mut self) {
        self.inner.push(SymbolTable::default());
    }

    pub fn pop_scope(&mut self) -> bool {
        if self.inner.len() == 1 {
            return false;
        }

        self.inner.pop();
        true
    }

    pub fn get_symbol(&self, identifier: &str) -> Option<&Symbol> {
        for table in self.inner.iter().rev() {
            if let Some(symbol) = table.get(identifier) {
                return Some(symbol);
            }
        }

        None
    }

    pub fn push_symbol(&mut self, identifier: impl Into<String>, symbol: Symbol) {
        let table = self
            .inner
            .last_mut()
            .expect("Symbol table stack should always have at least 1 table");
        table.insert(identifier.into(), symbol);
    }
}

impl Default for SymbolTableStack {
    fn default() -> Self {
        Self {
            inner: vec![SymbolTable::default()],
        }
    }
}

// x 0x00
//   - x 0x01
//   - y 0x02
// y 0x01

pub type SymbolTable = HashMap<String, Symbol>;

pub struct Symbol {
    pub typ: SymbolType,
}

pub enum SymbolType {
    Function(Function),
    Variable(Variable),
    Constant(Constant),
}

pub struct Function {
    pub arity: u8,
    pub returns_value: bool,
    // TODO: Types
}

pub struct Variable {
    pub idx: u16,
    // TODO: Types
}

pub struct Constant {
    pub idx: u16,
    // TODO: Types
}
