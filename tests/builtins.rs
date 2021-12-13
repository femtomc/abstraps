use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::Module;
use abstraps::ir::core::Intrinsic;

#[test]
fn builtins_module_operation() {
    let mut builder = Module.get_builder();
    let op = builder.finish();
    assert!(op.is_ok());
    println!("{}", op.unwrap());
}
