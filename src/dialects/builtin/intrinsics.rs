use crate::core::{
    BasicBlock, Graph, Intrinsic, IntrinsicTrait, LocationInfo, OperationBuilder, Region, SSACFG,
};
use crate::dialects::builtin::attributes::{Symbol, SymbolTable};
use crate::dialects::builtin::traits::{ProvidesSymbol, ProvidesSymbolTable};

// Module intrinsic.
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
    pub fn get_builder(&self, name: &str, loc: Option<LocationInfo>) -> OperationBuilder {
        let intr = Box::new(Module);
        let mut b = OperationBuilder::default(intr, loc);
        let r = Region::Undirected(Graph::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let st = SymbolTable::new();
        b.insert_attr("symbols", Box::new(st));
        let sym_name = Symbol::new(name);
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
    pub fn get_builder(&self, name: &str, loc: Option<LocationInfo>) -> OperationBuilder {
        let intr = Box::new(Func);
        let mut b = OperationBuilder::default(intr, loc);
        let r = Region::Directed(SSACFG::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let attr = Symbol::new(name);
        b.insert_attr("symbol", Box::new(attr));
        b
    }
}
