use abstraps::core::LocationInfo;
use abstraps::dialects::builtin::intrinsics::{Func, Module};

#[test]
fn builtins_module_operation_0() {
    let builder = Module.get_builder("foo", None);
    let op = builder.finish();
    assert!(op.is_ok());
    println!("{}", op.unwrap());
}

#[test]
fn builtins_module_operation_1() {
    let mut module = Module.get_builder("foo", None);
    let func = Func.get_builder("new_func", None).finish();
    assert!(func.is_ok());
    let f = func.unwrap();
    module.push_op(f);
    let end = module.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
}
