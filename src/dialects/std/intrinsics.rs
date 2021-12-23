use crate::dialects::builtin::*;
use crate::*;

// Call intrinsic.
intrinsic!(Call, "std", "call");

impl Call {
    pub fn get_builder(
        &self,
        name: &str,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> OperationBuilder {
        let intr = Box::new(Call);
        let mut b = OperationBuilder::default(intr, loc);
        let sym_name = Symbol::new(name);
        b.set_operands(operands);
        b.insert_attr("symbol", Box::new(sym_name));
        b
    }
}

// Return intrinsic.
intrinsic!(Return, "std", "return");

impl Return {
    pub fn get_builder(&self, operands: Vec<Var>, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Return);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b
    }
}
