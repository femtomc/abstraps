use crate::ir::builder::OperationBuilder;
use crate::ir::core::{
    Attribute, AttributeValue, BasicBlock, Intrinsic, IntrinsicTrait, Operation, Region, Var,
};
use crate::ir::graph::Graph;
use crate::ir::ssacfg::SSACFG;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct SymbolTable(HashMap<String, Var>);

impl Attribute for SymbolTable {
    fn get_value(&self) -> &dyn AttributeValue {
        &self.0
    }

    fn get_value_mut(&mut self) -> &mut dyn AttributeValue {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct ProvidesSymbolTable;

impl IntrinsicTrait for ProvidesSymbolTable {
    fn verify(&self, op: &Operation) -> bool {
        true
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

    fn get_builder(&self) -> OperationBuilder {
        let intr = Box::new(Module);
        let mut b = OperationBuilder::default(intr);
        let r = Region::Undirected(Graph::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let st = SymbolTable(HashMap::new());
        b.insert_attr("symbols", Box::new(st));
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
        Vec::new()
    }

    fn get_builder(&self) -> OperationBuilder {
        let intr = Box::new(Func);
        let mut b = OperationBuilder::default(intr);
        let r = Region::Directed(SSACFG::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
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

impl OperationBuilder {
    pub fn name(mut self, name: &str) -> Self {
        let attr = SymbolAttribute(name.to_string());
        self.insert_attr("symbol", Box::new(attr));
        self
    }
}
