use abstraps::core::builder::OperationBuilder;
use abstraps::core::ir::{Intrinsic, IntrinsicTrait, Var};
use abstraps::core::pass_manager::OperationPass;
use abstraps::dialects::builtin::intrinsics::{Func, Module};
use abstraps::dialects::builtin::passes::PopulateSymbolTablePass;

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
    pub fn get_builder(&self, operands: Vec<Var>) -> OperationBuilder {
        let intr = Box::new(Add);
        let mut b = OperationBuilder::default(intr);
        b.set_operands(operands);
        b
    }
}

#[test]
fn builtins_module_operation_1() -> anyhow::Result<()> {
    let mut module = Module.get_builder("foo");
    let mut func1 = Func.get_builder("new_func1");
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Add.get_builder(operands);
    let ret = func1.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret]);
    func1.push(add2)?;
    let mut func2 = Func.get_builder("new_func2");
    let operands = vec![func2.push_arg()?, func2.push_arg()?];
    let add1 = Add.get_builder(operands);
    let ret = func2.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret]);
    func2.push(add2)?;
    module.push(func1)?;
    module.push(func2)?;
    let end = module.finish();
    assert!(end.is_ok());
    let mut op = end.unwrap();
    let pass = PopulateSymbolTablePass;
    pass.apply(&mut op).unwrap();
    println!("{}", op);
    Ok(())
}
