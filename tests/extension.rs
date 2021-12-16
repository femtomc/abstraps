
use abstraps::core::builder::OperationBuilder;
use abstraps::core::ir::{Intrinsic, IntrinsicTrait, Var};
use abstraps::dialects::builtin::intrinsics::{Func, Module};


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
    let add1 = Add.get_builder(operands);
    let ret = func.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret]);
    func.push(add2)?;
    module.push(func)?;
    let end = module.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
    Ok(())
}
