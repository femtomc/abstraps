use crate::core::{Attribute, AttributeValue, Var};
use std::collections::HashMap;
use std::fmt;
use yansi::Paint;

#[derive(Debug)]
pub struct SymbolTable(HashMap<String, Var>);

impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ ")?;
        self.0.iter().fold(true, |first, elem| {
            if !first {
                write!(f, ", ");
            }
            write!(f, "{} => {}", Paint::blue(elem.0), elem.1);
            false
        });
        write!(f, " }}")?;
        Ok(())
    }
}

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

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Paint::blue(&self.0))
    }
}

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

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
