use abstraps::core::*;
use abstraps::dialects::builtin::*;


#[test]
fn builtins_module_operation_0() {
    diagnostics_setup();
    let builder = Module.get_builder("foo", LocationInfo::Unknown);
    let op = builder.finish();
    assert!(op.is_ok());
}

#[test]
fn builtins_func_operation_0() {
    let func = Func.get_builder("new_func", LocationInfo::Unknown).finish();
    // This should fail because `Func` needs terminators.
    assert!(func.is_err());
}
