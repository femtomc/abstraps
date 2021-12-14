use abstraps;
use abstraps::ir::builder::OperationBuilder;
use abstraps::ir::builtin::{Func, Module, ProvidesSymbol, ProvidesSymbolTable, SymbolTable};
use abstraps::ir::core::{Intrinsic, IntrinsicTrait, Var};
use anyhow;
use anyhow::bail;

// This shows usage of the "trait" interface
// (which, again, is similar to the MLIR version)
// This uses pretty advanced trait object / casting features,
// but this is one way to allow behavioral extensions to
// dynamic objects (which are required to support dialect
// extensions to the IR).

#[test]
fn operation_traits_module_operation_0() -> anyhow::Result<()> {
    let mut builder = Module.get_builder();
    let tr = builder.get_trait::<ProvidesSymbolTable>()?;
    let attr = tr.get_attribute_mut(&mut builder)?;
    let mut q = attr.downcast_mut::<SymbolTable>().unwrap();
    q.insert("this", Var::new(1));
    println!("{:?}", q);
    let mut op = builder.finish();
    assert!(op.is_ok());
    Ok(())
}
