use crate::ir::builder::OperationBuilder;
use crate::ir::core::{
    Attribute, AttributeValue, BasicBlock, Intrinsic, IntrinsicTrait, Operation, Region,
    SupportsVerification, Var,
};
use crate::ir::graph::Graph;
use crate::ir::ssacfg::SSACFG;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fmt;

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
    pub fn insert(&mut self, s: &str, v: Var) {
        self.0.insert(s.to_string(), v);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProvidesSymbolTable;

impl IntrinsicTrait for ProvidesSymbolTable {
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()> {
        if !op.get_attributes().contains_key("symbols") {
            bail!("Operation attribute map does not contain the `symbols` key.")
        }
        let attr = op.get_attributes().get("symbols").unwrap();
        match attr.downcast_ref::<SymbolTable>() {
            Some(v) => Ok(()),
            None => bail!("The attribute value indexed by `symbols` is not a `SymbolTable`."),
        }
    }

    fn get_attribute_mut<'a>(
        &self,
        b: &'a mut OperationBuilder,
    ) -> anyhow::Result<&'a mut Box<dyn Attribute>> {
        match b.get_attributes_mut().get_mut("symbols") {
            None => bail!("Failed to get `symbols` key in operation attributes map."),
            Some(v) => Ok(v),
        }
    }
}

// Module operation.
#[derive(Debug)]
pub struct Module;

impl Intrinsic for Module {
    fn get_namespace(&self) -> &str {
        "builtin"
    }

    fn get_name(&self) -> &str {
        "module"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        let st = Box::new(ProvidesSymbolTable);
        vec![st]
    }
}

impl Module {
    pub fn get_builder(&self, name: &str) -> OperationBuilder {
        let intr = Box::new(Module);
        let mut b = OperationBuilder::default(intr);
        let r = Region::Undirected(Graph::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let st = SymbolTable(HashMap::new());
        b.insert_attr("symbols", Box::new(st));
        let sym_name = SymbolAttribute(name.to_string());
        b.insert_attr("symbol", Box::new(sym_name));
        b
    }
}

// Function operation.
#[derive(Debug)]
pub struct Func;
impl Intrinsic for Func {
    fn get_namespace(&self) -> &str {
        "builtin"
    }

    fn get_name(&self) -> &str {
        "func"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        let s = Box::new(ProvidesSymbol);
        vec![s]
    }
}

impl Func {
    pub fn get_builder(&self, name: &str) -> OperationBuilder {
        let intr = Box::new(Func);
        let mut b = OperationBuilder::default(intr);
        let r = Region::Directed(SSACFG::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let attr = SymbolAttribute(name.to_string());
        b.insert_attr("symbol", Box::new(attr));
        b
    }
}

#[derive(Debug)]
struct SymbolAttribute(String);

impl Attribute for SymbolAttribute {
    fn get_value(&self) -> &dyn AttributeValue {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut dyn AttributeValue {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProvidesSymbol;

impl IntrinsicTrait for ProvidesSymbol {
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()> {
        if !op.get_attributes().contains_key("symbol") {
            bail!("Operation attribute map does not contain the `symbol` key.")
        }
        let attr = op.get_attributes().get("symbol").unwrap();
        match attr.downcast_ref::<SymbolAttribute>() {
            Some(v) => Ok(()),
            None => bail!("The attribute value indexed by `symbol` is not a `Symbol`."),
        }
    }
}
