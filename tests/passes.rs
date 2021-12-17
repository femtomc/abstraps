use abstraps::core::{
    Intrinsic, IntrinsicTrait, LocationInfo, OperationBuilder, OperationPass, OperationPassManager,
    PassManager, Var,
};
use abstraps::dialects::builtin::intrinsics::{Func, Module};
use abstraps::dialects::builtin::passes::PopulateSymbolTablePass;
use abstraps::dialects::std::intrinsics::{Call, Return};

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

#[test]
fn builtins_module_operation_1() -> anyhow::Result<()> {
    let mut module = Module.get_builder("foo", LocationInfo::Unknown);
    let mut func1 = Func.get_builder("new_func1", LocationInfo::Unknown);
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func1.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    func1.push(add2)?;
    let mut func2 = Func.get_builder("new_func2", LocationInfo::Unknown);
    let operands = vec![func2.push_arg()?, func2.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func2.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    let v = func2.push(add2)?;
    let call2 = Call.get_builder("new_func1", vec![v, v], LocationInfo::Unknown);
    let v = func2.push(call2)?;
    let ret2 = Return.get_builder(vec![v], LocationInfo::Unknown);
    func2.push(ret2)?;
    module.push(func1)?;
    module.push(func2)?;
    let end = module.finish();
    assert!(end.is_ok());
    let mut op = end.unwrap();
    let mut pm = OperationPassManager::<Module>::new();
    pm.push(Box::new(PopulateSymbolTablePass));
    pm.prewalk(&mut op).unwrap();
    println!("{}", op);
    Ok(())
}
