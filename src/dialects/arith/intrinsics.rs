use crate::core::*;
use crate::dialects::arith::traits::*;
use crate::dialects::builtin::NonVariadic;
use crate::*;

intrinsic! {
    /// Floating point addition operation.
    /// Supports elementwise mapping over rank matching tensors.
    Addf: ["arith", "addf"],
    [Elementwise],
    extern: [NonVariadic]
}

impl NonVariadic for Addf {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_operands().len() != 2 {
            bail!(format!(
                "{} is non-variadic, and supports a fixed number (2) of operands.",
                op.get_intrinsic(),
            ));
        }
        Ok(())
    }
}

impl Addf {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Addf);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        Ok(b)
    }
}

intrinsic! {
    Addi: ["arith", "addi"],
    [Elementwise, Commutative],
    extern: [NonVariadic]
}

impl NonVariadic for Addi {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_operands().len() != 2 {
            bail!(format!(
                "{} is non-variadic, and supports a fixed number (2) of operands.",
                op.get_intrinsic(),
            ));
        }
        Ok(())
    }
}

impl Addi {
    pub fn get_builder(
        &self,
        operands: Vec<Var>,
        loc: LocationInfo,
    ) -> Result<OperationBuilder, Report> {
        let intr = Box::new(Addi);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        Ok(b)
    }
}

intrinsic! {
    Andi: ["arith", "andi"],
    [Elementwise, Commutative],
    extern: []
}

intrinsic! {
    Bitcast: ["arith", "bitcast"],
    [Elementwise],
    extern: []
}

intrinsic! {
    Cmpf: ["arith", "cmpf"],
    [Elementwise],
    extern: []
}

intrinsic! {
    Cmpi: ["arith", "cmpi"],
    [Elementwise],
    extern: []
}

intrinsic! {
    Divf: ["arith", "divf"],
    [Elementwise],
    extern: []
}
