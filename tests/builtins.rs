use abstraps::dialects::builtin::*;
use abstraps::*;

#[test]
fn builtins_module_operation_0() {
    diagnostics_setup();
    let builder = Module.get_builder("foo", LocationInfo::Unknown);
    let op = builder.finish();
    assert!(op.is_ok());
    println!("{}", op.unwrap());
}

#[test]
fn builtins_module_operation_1() {
    let mut module = Module.get_builder("foo", LocationInfo::Unknown);
    let func = Func.get_builder("new_func", LocationInfo::Unknown).finish();
    assert!(func.is_ok());
    let f = func.unwrap();
    module.push_op(f);
    let end = module.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
}
