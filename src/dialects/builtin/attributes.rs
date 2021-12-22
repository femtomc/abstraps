use crate::*;
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

impl Attribute for SymbolTable {}

impl AttributeValue<HashMap<String, Var>> for SymbolTable {
    fn get_value(&self) -> &HashMap<String, Var> {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut HashMap<String, Var> {
        &mut self.0
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable(HashMap::new())
    }
}

interfaces!(
    SymbolTable: dyn Attribute,
    dyn fmt::Display,
    dyn std::fmt::Debug,
    dyn AttributeValue<HashMap<String, Var>>
);

#[derive(Debug)]
pub struct Symbol(String);

impl Attribute for Symbol {}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Paint::blue(&self.0))
    }
}

impl AttributeValue<String> for Symbol {
    fn get_value(&self) -> &String {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl Symbol {
    pub fn new(s: &str) -> Symbol {
        Symbol(s.to_string())
    }
}

interfaces!(
    Symbol: dyn Attribute,
    dyn std::fmt::Display,
    dyn std::fmt::Debug,
    dyn AttributeValue<String>
);
