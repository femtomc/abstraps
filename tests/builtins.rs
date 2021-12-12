use abstraps;
use abstraps::ir::builder::{OperationBuilder, Setup};
use abstraps::ir::builtin::{BuiltinAttribute, BuiltinIntrinsic};
use abstraps::ir::core::Verify;
use serde::Serialize;

#[test]
fn builtins_module_operation() {
    let mut builder =
        OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::new(BuiltinIntrinsic::Module);
    let op = builder.finish();
    assert!(op.is_ok());
}

#[test]
fn builtins_func_operation() {
    let mut builder =
        OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::new(BuiltinIntrinsic::Func)
            .name("my_func");
    let op = builder.finish();
    assert!(op.is_ok());
}
