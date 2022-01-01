use abstraps::core::*;
use abstraps::dialects::builtin::*;
use abstraps::*;

#[test]
fn builtins_module_operation_0() -> Result<(), Report> {
    diagnostics_setup()?;
    let builder = Module.get_builder("foo", LocationInfo::Unknown)?;
    let op = builder.finish();
    assert!(op.is_ok());
    Ok(())
}

#[test]
fn builtins_func_operation_0() -> Result<(), Report> {
    let func = Func
        .get_builder("new_func", LocationInfo::Unknown)?
        .finish();
    // This should fail because `Func` needs terminators.
    assert!(func.is_err());
    Ok(())
}
