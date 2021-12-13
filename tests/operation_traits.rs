use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::{Func, Module, ProvidesSymbolTable};
use abstraps::ir::core::{check_trait, Intrinsic};

#[test]
fn operation_traits_module_operation_0() {
    let mut builder = Module.get_builder();
    let op = builder.finish();
    assert!(op.is_ok());
    match check_trait::<ProvidesSymbolTable>(&op.unwrap()) {
        None => assert!(false),
        Some(v) => assert!(v),
    }
}
