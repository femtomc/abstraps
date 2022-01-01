use crate::core::*;
use crate::dialects::builtin::*;
use crate::*;

intrinsic! {
    Constant: ["base", "constant"],
    [ProvidesConstantAttr],
    extern: []
}

impl Constant {
    pub fn get_builder(
        &self,
        val: ConstantAttr,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Constant);
        let mut b = OperationBuilder::default(intr, loc);
        b.insert_attr("value", Box::new(val));
        Ok(b)
    }
}

intrinsic! {
    Call: ["base", "call"],
    [ProvidesSymbolAttr],
    extern: []
}

impl Call {
    pub fn get_builder(
        &self,
        name: &str,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Call);
        let mut b = OperationBuilder::default(intr, loc);
        let sym_name = SymbolAttr::new(name);
        b.set_operands(operands);
        b.insert_attr("builtin.symbol", Box::new(sym_name));
        Ok(b)
    }
}

intrinsic! {
    Return: ["base", "return"],
    [Terminator],
    extern: []
}

impl Return {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Return);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        Ok(b)
    }
}

intrinsic! {
    Branch: ["base", "branch"],
    [Terminator],
    extern: []
}

impl Branch {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        blks: Vec<usize>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Branch);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b.set_successors(blks);
        Ok(b)
    }
}

intrinsic! {
    ConditionalBranch: ["base", "br"],
    [Terminator],
    extern: []
}

impl ConditionalBranch {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        blks: Vec<usize>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(ConditionalBranch);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b.set_successors(blks);
        Ok(b)
    }
}
