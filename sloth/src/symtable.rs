use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::rc::Rc;

use crate::parser::ast::TypeIdentifier;

#[derive(Debug, Default)]
struct Scope {
    parent: Option<Rc<Scope>>,
    symbols: RefCell<HashMap<String, Symbol>>,
}

#[derive(Debug)]
pub struct SymbolTable(Rc<Scope>);

impl SymbolTable {
    pub fn new() -> Self {
        Self(Rc::new(Scope {
            parent: None,
            ..Default::default()
        }))
    }

    pub fn make_child(&self) -> Self {
        Self(Rc::new(Scope {
            parent: Some(self.0.clone()),
            ..Default::default()
        }))
    }

    pub fn parent(&self) -> Option<Self> {
        Some(Self(self.0.parent.clone()?))
    }

    pub fn contains(&self, identifier: &str) -> bool {
        for scope in self.iter() {
            if scope.symbols.borrow().contains_key(identifier) {
                return true;
            }
        }

        false
    }

    pub fn get(&self, identifier: &str) -> Option<Ref<'_, Symbol>> {
        for scope in self.iter() {
            let reference = scope.symbols.borrow();
            if let Ok(symbol) = Ref::filter_map(reference, |it| it.get(identifier)) {
                return Some(symbol);
            }
        }

        None
    }

    pub fn get_type(&self, identifier: &TypeIdentifier) -> Option<Type> {
        let symbol = self.get(&identifier.name)?;
        if let Symbol::Type(ref typ) = *symbol {
            let mut typ = typ.clone();
            if identifier.is_list {
                typ = Type::Array { typ: Box::new(typ) };
            }
            return Some(typ);
        }

        None
    }

    pub fn get_value(&self, identifier: &str) -> Option<ValueSymbol> {
        let symbol = self.get(identifier)?;
        if let Symbol::Value(ref symbol) = *symbol {
            return Some(symbol.clone());
        }

        None
    }

    pub fn get_mut(&self, identifier: &str) -> Option<RefMut<'_, Symbol>> {
        for scope in self.iter() {
            let reference = scope.symbols.borrow_mut();
            if let Ok(symbol) = RefMut::filter_map(reference, |it| it.get_mut(identifier)) {
                return Some(symbol);
            }
        }

        None
    }

    pub fn insert(&mut self, identifier: String, symbol: Symbol) -> bool {
        let mut reference = self.0.symbols.borrow_mut();
        if let Vacant(e) = reference.entry(identifier) {
            e.insert(symbol);
            return true;
        }

        false
    }

    fn iter(&self) -> Iter<'_> {
        Iter {
            next: Some(&self.0),
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SymbolTable {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct Iter<'a> {
    next: Option<&'a Scope>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Scope;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|scope| {
            self.next = scope.parent.as_deref();
            scope
        })
    }
}

#[derive(Clone, Debug)]
pub enum Symbol {
    /// Symbol referencing a compile time type, such as the Int symbol
    Type(Type),
    /// Symbol referencing a runtime value, such as the println symbol
    Value(ValueSymbol),
}

#[derive(Clone, Debug)]
pub struct ValueSymbol {
    pub typ: Type,
    pub id: i32,
    pub mutable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Void,
    Integer,
    Float,
    Boolean,
    String,
    Iterator {
        typ: Box<Type>,
    },
    Function {
        inputs: Vec<Type>,
        output: Box<Type>,
    },
    Struct {
        properties: HashMap<String, Type>,
    },
    Array {
        typ: Box<Type>,
    },
}
