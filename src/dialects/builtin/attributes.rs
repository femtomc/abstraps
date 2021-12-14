use crate::core::ir::{Attribute, AttributeValue, Var};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable(HashMap<String, Var>);

impl Attribute for SymbolTable {
    fn get_value(&self) -> &dyn AttributeValue {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut dyn AttributeValue {
        &mut self.0
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable(HashMap::new())
    }

    pub fn insert(&mut self, s: &str, v: Var) {
        self.0.insert(s.to_string(), v);
    }
}

#[derive(Debug)]
pub struct Symbol(String);

impl Attribute for Symbol {
    fn get_value(&self) -> &dyn AttributeValue {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut dyn AttributeValue {
        &mut self.0
    }
}

impl Symbol {
    pub fn new(s: &str) -> Symbol {
        Symbol(s.to_string())
    }
}
