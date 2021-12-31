use crate::core::*;
use crate::dialects::base::*;
use crate::dialects::builtin::*;
use crate::*;

// Constant intrinsic.
intrinsic!(Constant: ["base", "constant"], [ConstantLike], extern: []);

impl Constant {
    pub fn get_builder(&self, val: ConstantAttr, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Constant);
        let mut b = OperationBuilder::default(intr, loc);
        b.insert_attr("value", Box::new(val));
        b
    }
}

// Call intrinsic.
intrinsic!(Call: ["base", "call"], [], extern: []);

impl Call {
    pub fn get_builder(
        &self,
        name: &str,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> OperationBuilder {
        let intr = Box::new(Call);
        let mut b = OperationBuilder::default(intr, loc);
        let sym_name = SymbolAttr::new(name);
        b.set_operands(operands);
        b.insert_attr("symbol", Box::new(sym_name));
        b
    }
}

// Return intrinsic.
intrinsic!(Return: ["base", "return"], [Terminator], extern: []);

impl Return {
    pub fn get_builder(&self, operands: Vec<Var>, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Return);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b
    }
}

// Branch intrinsic.
intrinsic!(Branch: ["base", "branch"], [Terminator], extern: []);

impl Branch {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        blks: Vec<usize>,
        loc: LocationInfo,
    ) -> OperationBuilder {
        let intr = Box::new(Branch);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b.set_successors(blks);
        b
    }
}

// Conditional branch intrinsic.
intrinsic!(ConditionalBranch: ["base", "br"], [Terminator], extern: []);

impl ConditionalBranch {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        blks: Vec<usize>,
        loc: LocationInfo,
    ) -> OperationBuilder {
        let intr = Box::new(ConditionalBranch);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b.set_successors(blks);
        b
    }
}
