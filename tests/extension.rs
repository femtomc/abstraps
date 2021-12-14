use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::{Func, Module};
use abstraps::ir::core::{BasicBlock, Intrinsic, IntrinsicTrait, Var};
use anyhow;

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
    let mut func = Func.get_builder("new_func");
    let operands = vec![func.push_arg()?, func.push_arg()?];
    let mut add = Add.get_builder(operands);
    let op = func.push(add)?;
    let end = module.push(op)?.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
    Ok(())
}
