use abstraps::core::{
    diagnostics_setup, AnalysisManager, Intrinsic, IntrinsicTrait, LatticeJoin, LocationInfo,
    OperationBuilder, TypeKey, Var,
};
use abstraps::dialects::builtin::intrinsics::Func;
use color_eyre::Report;

#[derive(Debug)]
pub struct Add;

impl Intrinsic for Add {
    fn get_namespace(&self) -> &str {
        "arith"
    }

    fn get_name(&self) -> &str {
        "add"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        Vec::new()
    }
}

impl Add {
    pub fn get_builder(&self, operands: Vec<Var>, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Add);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ArithLattice {
    Int64,
}

impl LatticeJoin for ArithLattice {
    fn join(&self, _other: &ArithLattice) -> ArithLattice {
        self.clone()
    }
}

#[test]
fn typeinf_0() -> Result<(), Report> {
    diagnostics_setup();
    let mut func1 = Func.get_builder("new_func1", LocationInfo::Unknown);
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func1.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    func1.push(add2)?;
    let end = func1.finish();
    assert!(end.is_ok());
    let op = end.unwrap();
    let key = TypeKey::new(
        "new_func1",
        vec![Some(ArithLattice::Int64), Some(ArithLattice::Int64)],
    );
    let mut am = AnalysisManager::new();
    am.analyze(key, &op);
    Ok(())
}
