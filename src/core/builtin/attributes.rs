use crate::core::*;
use crate::*;
use std::collections::HashMap;
use std::fmt;
use yansi::Paint;

#[derive(Debug)]
pub enum ConstantAttr {
    Integer(i64, usize),
    Float(f64, usize),
}

impl fmt::Display for ConstantAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstantAttr::Integer(v, _) => write!(f, "{{ {} }}", v),
            ConstantAttr::Float(v, _) => write!(f, "{{ {} }}", v),
        }
    }
}

attribute! {
    ConstantAttr: "builtin.value",
    trait: ProvidesConstantAttr
}

#[derive(Debug)]
pub enum LinkageAttr {
    Private,
    External,
}

impl fmt::Display for LinkageAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkageAttr::Private => write!(f, "{}", Paint::blue("private").bold()),
            LinkageAttr::External => write!(f, "{}", Paint::blue("external").bold()),
        }
    }
}

attribute! {
    LinkageAttr: "builtin.linkage",
    trait: ProvidesLinkageAttr
}

#[derive(Debug)]
pub struct SymbolTableAttr(HashMap<String, Var>);

impl fmt::Display for SymbolTableAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ ")?;
        let l = self.0.len();
        for (ind, v) in self.0.iter().enumerate() {
            match ind == l - 1 {
                true => write!(f, "{} => {}", Paint::blue(v.0), v.1)?,
                false => write!(f, "{} => {}, ", Paint::blue(v.0), v.1)?,
            };
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl SymbolTableAttr {
    pub fn insert(&mut self, s: String, v: Var) {
        self.0.insert(s, v);
    }

    pub fn new() -> SymbolTableAttr {
        SymbolTableAttr(HashMap::new())
    }
}

attribute! {
    SymbolTableAttr: "builtin.symbols",
    trait: ProvidesSymbolTableAttr
}

#[derive(Debug)]
pub enum SymbolVisibility {
    Public,
    Private,
    Nested,
}

impl fmt::Display for SymbolVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolVisibility::Public => write!(f, "{}", Paint::white("public")),
            SymbolVisibility::Private => write!(f, "{}", Paint::white("private")),
            SymbolVisibility::Nested => write!(f, "{}", Paint::white("nested")),
        }
    }
}

#[derive(Debug)]
pub struct SymbolAttr(String, SymbolVisibility);

impl fmt::Display for SymbolAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", Paint::white(&self.1), Paint::blue(&self.0))
    }
}

impl SymbolAttr {
    pub fn new(s: &str) -> SymbolAttr {
        SymbolAttr(s.to_string(), SymbolVisibility::Public)
    }
}

attribute! {
    SymbolAttr: "builtin.symbol",
    trait: ProvidesSymbolAttr
}
