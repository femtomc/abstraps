use crate::core::builder::OperationBuilder;
use crate::core::graph::Graph;
use crate::core::ir::{BasicBlock, Intrinsic, IntrinsicTrait, Var};
use crate::core::region::Region;
use crate::core::ssacfg::SSACFG;
use crate::dialects::builtin::attributes::Symbol;
use crate::dialects::builtin::traits::ProvidesSymbol;

// Call intrinsic.
#[derive(Debug)]
pub struct Call;

impl Intrinsic for Call {
    fn get_namespace(&self) -> &str {
        "std"
    }

    fn get_name(&self) -> &str {
        "call"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        let st = Box::new(ProvidesSymbol);
        vec![st]
    }
}

impl Call {
    pub fn get_builder(&self, name: &str, operands: Vec<Var>) -> OperationBuilder {
        let intr = Box::new(Call);
        let mut b = OperationBuilder::default(intr);
        let sym_name = Symbol::new(name);
        b.set_operands(operands);
        b.insert_attr("symbol", Box::new(sym_name));
        b
    }
}

// Return intrinsic.
#[derive(Debug)]
pub struct Return;

impl Intrinsic for Return {
    fn get_namespace(&self) -> &str {
        "std"
    }

    fn get_name(&self) -> &str {
        "return"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        Vec::new()
    }
}

impl Return {
    pub fn get_builder(&self, operands: Vec<Var>) -> OperationBuilder {
        let intr = Box::new(Return);
        let mut b = OperationBuilder::default(intr);
        b.set_operands(operands);
        b
    }
}
