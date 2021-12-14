use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::{Func, Module};
use abstraps::ir::core::{BasicBlock, Intrinsic, IntrinsicTrait};

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
    pub fn get_builder(&self) -> OperationBuilder {
        let intr = Box::new(Add);
        let mut b = OperationBuilder::default(intr);
        let blk = BasicBlock::default();
        b
    }
}

#[test]
fn builtins_module_operation_1() {
    let mut module = Module.get_builder("foo");
    let mut func = Func.get_builder("new_func");
    let mut add = Add.get_builder();
    let operands = vec![func.push_arg().unwrap(), func.push_arg().unwrap()];
    add.set_operands(operands);
    let op = func.push_op(add.finish().unwrap()).finish().unwrap();
    let end = module.push_op(op).finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
}
