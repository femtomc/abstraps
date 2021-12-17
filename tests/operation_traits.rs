use abstraps::core::{IntrinsicTrait, LocationInfo, Var};
use abstraps::dialects::builtin::attributes::SymbolTable;
use abstraps::dialects::builtin::intrinsics::Module;
use abstraps::dialects::builtin::traits::ProvidesSymbolTable;

// This shows usage of the "trait" interface
// (which, again, is similar to the MLIR version)
// This uses pretty advanced trait object / casting features,
// but this is one way to allow behavioral extensions to
// dynamic objects (which are required to support dialect
// extensions to the IR).

#[test]
fn operation_traits_module_operation_0() -> anyhow::Result<()> {
    let mut builder = Module.get_builder("foo", None);
    let o = builder.finish();
    assert!(o.is_ok());
    let mut op = o.unwrap();
    let tr = op.get_trait::<ProvidesSymbolTable>()?;
    let attr = tr.get_attribute_mut(&mut op)?;
    let q = attr.downcast_mut::<SymbolTable>().unwrap();
    q.insert("this", Var::new(1));
    println!("{:?}", q);
    Ok(())
}
