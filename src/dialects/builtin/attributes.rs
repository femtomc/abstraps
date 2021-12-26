use crate::*;
use std::collections::HashMap;
use std::fmt;
use yansi::Paint;

#[derive(Debug)]
pub enum ConstantAttr {
    Int(i64, usize),
    Float(f64, usize),
}

impl fmt::Display for ConstantAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstantAttr::Int(v, _) => write!(f, "{{ {} }}", v),
            ConstantAttr::Float(v, _) => write!(f, "{{ {} }}", v),
        }
    }
}

impl Attribute for ConstantAttr {}

impl AttributeValue<ConstantAttr> for ConstantAttr {
    fn get_value(&self) -> &ConstantAttr {
        self
    }

    fn get_value_mut(&mut self) -> &mut ConstantAttr {
        self
    }
}

interfaces!(
    ConstantAttr: dyn Attribute,
    dyn fmt::Display,
    dyn fmt::Debug,
    dyn AttributeValue<ConstantAttr>
);

#[derive(Debug)]
pub struct SymbolTableAttr(HashMap<String, Var>);

impl fmt::Display for SymbolTableAttr {
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

impl Attribute for SymbolTableAttr {}

impl AttributeValue<HashMap<String, Var>> for SymbolTableAttr {
    fn get_value(&self) -> &HashMap<String, Var> {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut HashMap<String, Var> {
        &mut self.0
    }
}

impl SymbolTableAttr {
    pub fn new() -> SymbolTableAttr {
        SymbolTableAttr(HashMap::new())
    }
}

interfaces!(
    SymbolTableAttr: dyn Attribute,
    dyn fmt::Display,
    dyn std::fmt::Debug,
    dyn AttributeValue<HashMap<String, Var>>
);

#[derive(Debug)]
pub struct SymbolAttr(String);

impl Attribute for SymbolAttr {}

impl fmt::Display for SymbolAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Paint::blue(&self.0))
    }
}

impl AttributeValue<String> for SymbolAttr {
    fn get_value(&self) -> &String {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl SymbolAttr {
    pub fn new(s: &str) -> SymbolAttr {
        SymbolAttr(s.to_string())
    }
}

interfaces!(
    SymbolAttr: dyn Attribute,
    dyn std::fmt::Display,
    dyn std::fmt::Debug,
    dyn AttributeValue<String>
);
