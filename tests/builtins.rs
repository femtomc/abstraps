use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::{Func, Module};
use abstraps::ir::core::Intrinsic;

#[test]
fn builtins_module_operation_0() {
    let mut builder = Module.get_builder();
    let op = builder.finish();
    assert!(op.is_ok());
    println!("{}", op.unwrap());
}

#[test]
fn builtins_module_operation_1() {
    let module = Module.get_builder().name("foo");
    let func = Func.get_builder().name("new_func").finish();
    assert!(func.is_ok());
    let f = func.unwrap();
    let end = module.push_op(f).finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
}
