use crate::dialects::builtin::*;
use crate::*;

// Call intrinsic.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Call;

impl Intrinsic for Call {
    fn get_namespace(&self) -> &str {
        "std"
    }

    fn get_name(&self) -> &str {
        "call"
    }
}

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

interfaces!(Call: dyn ObjectClone, dyn Intrinsic);

// Return intrinsic.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Return;

impl Intrinsic for Return {
    fn get_namespace(&self) -> &str {
        "std"
    }

    fn get_name(&self) -> &str {
        "return"
    }
}

impl Return {
    pub fn get_builder(&self, operands: Vec<Var>, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Return);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b
    }
}

interfaces!(Return: dyn ObjectClone, dyn Intrinsic);
