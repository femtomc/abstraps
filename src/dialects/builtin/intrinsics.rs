use crate::core::*;
use crate::dialects::builtin::*;
use crate::*;

intrinsic! {
    Module: ["builtin", "module"],
    [ProvidesSymbolTable], extern: []
}

impl Module {
    pub fn get_builder(&self, name: &str, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Module);
        let mut b = OperationBuilder::default(intr, loc);
        let r = Region::Undirected(Graph::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let st = SymbolTableAttr::new();
        b.insert_attr("symbols", Box::new(st));
        let sym_name = SymbolAttr::new(name);
        b.insert_attr("symbol", Box::new(sym_name));
        b
    }
}

intrinsic! {
    Func: ["builtin", "func"],
    [ProvidesSymbol, ProvidesLinkage, FunctionLike, RequiresTerminators],
    extern: []
}

impl Func {
    pub fn get_builder(&self, name: &str, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Func);
        let mut b = OperationBuilder::default(intr, loc);
        let r = Region::Directed(SSACFG::default());
        b.push_region(r);
        let blk = BasicBlock::default();
        b.push_block(blk);
        let attr = SymbolAttr::new(name);
        b.insert_attr("symbol", Box::new(attr));
        let lattr = LinkageAttr::Private;
        b.insert_attr("linkage", Box::new(lattr));
        b
    }
}
